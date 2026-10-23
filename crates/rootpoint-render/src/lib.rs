//! The slide renderer of rootpoint: draws one slide of a document into an RGBA image, at any size,
//! from the same layout in slide units.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod images;
mod text;

use std::ops::Range;
use std::sync::Arc;

pub use images::{DecodeError, ImageSet};
use parley::Layout;
use rootpoint_model::{
    AssetId, Content, Document, ImageFit, Node, NodeId, Rect as SlideRect, SlideId, TextContent,
    VAlign, area_rects,
};
pub use text::{FontError, FontSet};
use vello_cpu::color::AlphaColor;
use vello_cpu::color::palette::css::{BLACK, WHITE};
use vello_cpu::kurbo::{Affine, Point, Rect, Shape};
use vello_cpu::peniko::ImageSampler;
use vello_cpu::{
    Image, ImageSource, Level, Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources,
};

/// Which SIMD code the rasterizer runs. `Native` is the fastest the machine has; `Portable` is
/// the scalar code, whose pixels do not depend on the machine's SIMD, for golden images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    Native,
    Portable,
}

/// The largest side of an image, in pixels. vello_cpu rounds a size up to its tiles in 16 bits,
/// which overflows above 65408 wide or 65532 high; this limit stays well below.
pub const MAX_SIDE: u16 = 16384;

/// Why a slide was not drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderError {
    UnknownSlide(SlideId),
    /// A side of zero pixels, or of more than [`MAX_SIDE`].
    BadSize {
        width: u16,
        height: u16,
    },
}

/// What a render could not draw as the document asks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// The image assets with no decoded image in the `ImageSet`; their areas are drawn grey.
    pub missing_images: Vec<AssetId>,
    /// The text areas whose font no font of the `FontSet` has; the built-in sans stands in.
    pub font_fallbacks: Vec<NodeId>,
    /// The text areas holding a character that no font of the `FontSet` has, drawn as a box.
    pub missing_glyphs: Vec<NodeId>,
    /// The text areas whose glyphs could not all be drawn, as from a damaged font file.
    pub undrawn_text: Vec<NodeId>,
    /// For each text area, the byte range of its text on each line, as laid out.
    pub lines: Vec<(NodeId, Vec<Range<usize>>)>,
}

/// A drawn slide: premultiplied RGBA pixels, row by row, and what the render could not draw.
pub struct Rendered {
    pub pixmap: Pixmap,
    pub report: Report,
}

impl Rendered {
    /// The image encoded as PNG bytes.
    pub fn png(self) -> Vec<u8> {
        self.pixmap
            .into_png()
            .expect("an RGBA pixmap always encodes")
    }
}

/// The space between a text and the edges of its area, in slide units.
const TEXT_PADDING: f64 = 24.0;

/// The grey of an image area whose image is missing.
const MISSING: AlphaColor<vello_cpu::color::Srgb> = AlphaColor::from_rgba8(224, 224, 224, 255);

/// Draw the slide `slide` of `doc` into a `width`×`height` image. The slide is scaled uniformly to
/// the largest size that fits, centred, and the rest is black. Text is laid out on every call.
pub fn render_slide(
    doc: &Document,
    slide: SlideId,
    width: u16,
    height: u16,
    fonts: &mut FontSet,
    images: &ImageSet,
    precision: Precision,
) -> Result<Rendered, RenderError> {
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err(RenderError::BadSize { width, height });
    }
    let the_slide = doc.slide(slide).ok_or(RenderError::UnknownSlide(slide))?;
    let level = match precision {
        Precision::Native => Level::try_detect().unwrap_or(Level::baseline()),
        Precision::Portable => Level::fallback(),
    };
    let mut ctx = RenderContext::new_with(
        width,
        height,
        RenderSettings {
            level,
            num_threads: 0,
        },
    );
    let mut report = Report::default();
    // A new `Resources` for each render: keeping one across renders made a warm 4K render of the
    // mixed slide slower, about 31-36 ms against 18-22 ms, release build, measured at review.
    let mut resources = Resources::new();
    ctx.set_paint(BLACK);
    ctx.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    let to_pixels = slide_transform(doc, width, height);
    let whole = doc.aspect.slide_rect();
    ctx.set_paint(WHITE);
    ctx.fill_rect(&on_pixels(to_pixels, whole));
    for (id, area) in area_rects(the_slide, doc.aspect) {
        let Some(Node::Area(a)) = the_slide.root.find(id) else {
            continue;
        };
        // The clip is snapped to whole pixels, so that two areas sharing an edge share a pixel
        // boundary: an anti-aliased clip on a fractional edge would let the white slide show
        // through the seam.
        let clip = on_pixels(to_pixels, area);
        ctx.set_transform(Affine::IDENTITY);
        ctx.push_clip_layer(&clip.to_path(0.1));
        match &a.content {
            Content::Text(content) => {
                let (laid, origin) = lay_out_area(fonts, content, area);
                if laid.fell_back {
                    report.font_fallbacks.push(id);
                }
                if laid.missing_glyph {
                    report.missing_glyphs.push(id);
                }
                ctx.set_transform(to_pixels * Affine::translate(origin));
                let c = content.style.colour;
                ctx.set_paint(AlphaColor::<vello_cpu::color::Srgb>::from_rgba8(
                    c.r, c.g, c.b, c.a,
                ));
                if !text::draw_text(&mut ctx, &mut resources, &laid) {
                    report.undrawn_text.push(id);
                }
                report.lines.push((id, laid.lines));
            }
            Content::Image(image) => match images.get(image.asset) {
                Some(pixmap) => draw_image(&mut ctx, to_pixels, area, clip, pixmap, image.fit),
                None => {
                    report.missing_images.push(image.asset);
                    ctx.set_paint(MISSING);
                    ctx.fill_rect(&clip);
                }
            },
        }
        ctx.pop_layer();
    }
    ctx.flush();
    let mut pixmap = Pixmap::new(width, height);
    ctx.render_with(&mut pixmap, &mut resources, RasterizerSettings::default());
    Ok(Rendered { pixmap, report })
}

/// The text of one area as `render_slide` draws it: its layout, in slide units, and the point of the
/// slide, in slide units, where the layout's origin is drawn. An editor that places a caret or a
/// selection on this layout, with Parley's `Selection` and `Cursor`, places it on the drawn glyphs.
pub struct AreaText {
    pub layout: Layout<()>,
    pub origin: (f64, f64),
}

/// The text of the area `area` of the slide `slide`, laid out as `render_slide` lays it out; `None`
/// when the slide or the area is not there, or the area holds no text.
pub fn area_text(
    doc: &Document,
    slide: SlideId,
    area: NodeId,
    fonts: &mut FontSet,
) -> Option<AreaText> {
    let the_slide = doc.slide(slide)?;
    let rect = area_rects(the_slide, doc.aspect)
        .into_iter()
        .find(|(id, _)| *id == area)?
        .1;
    let Some(Node::Area(a)) = the_slide.root.find(area) else {
        return None;
    };
    let Content::Text(content) = &a.content else {
        return None;
    };
    let (laid, origin) = lay_out_area(fonts, content, rect);
    Some(AreaText {
        layout: laid.layout,
        origin,
    })
}

/// A text area's layout, wrapped to the area's width less its padding, and the point its origin is
/// drawn at: inset by the padding, and moved down by the vertical alignment's share of the room
/// left. `render_slide` and `area_text` both lay a text out here, so they give one layout.
fn lay_out_area(
    fonts: &mut FontSet,
    content: &TextContent,
    area: SlideRect,
) -> (text::TextLayout, (f64, f64)) {
    let inner = (area.w - 2.0 * TEXT_PADDING).max(0.0);
    let laid = text::layout_text(fonts, content, inner as f32);
    let free = area.h - 2.0 * TEXT_PADDING - f64::from(laid.layout.height());
    let dy = match content.style.v_align {
        VAlign::Top => 0.0,
        VAlign::Middle => free / 2.0,
        VAlign::Bottom => free,
    };
    (laid, (area.x + TEXT_PADDING, area.y + TEXT_PADDING + dy))
}

/// How slide units are carried to the pixels of an image: a point `(u, v)` of the slide is at the
/// pixel `(x + scale * u, y + scale * v)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToPixels {
    pub scale: f64,
    pub x: f64,
    pub y: f64,
}

/// Where `render_slide` draws the slides of `doc` in a `width`×`height` image: scaled uniformly to
/// the largest size that fits, and centred. A caller that places marks over the image, or tests a
/// pointer against the slide, uses it to agree with the drawing.
pub fn slide_to_pixels(doc: &Document, width: u16, height: u16) -> ToPixels {
    let whole = doc.aspect.slide_rect();
    let (w, h) = (f64::from(width), f64::from(height));
    let scale = (w / whole.w).min(h / whole.h);
    ToPixels {
        scale,
        x: (w - whole.w * scale) / 2.0,
        y: (h - whole.h * scale) / 2.0,
    }
}

/// The transform of [`slide_to_pixels`], as the rasterizer takes it.
fn slide_transform(doc: &Document, width: u16, height: u16) -> Affine {
    let t = slide_to_pixels(doc, width, height);
    Affine::translate((t.x, t.y)) * Affine::scale(t.scale)
}

/// A rectangle in slide units, carried to pixels and rounded to whole pixels.
fn on_pixels(to_pixels: Affine, r: SlideRect) -> Rect {
    let a = to_pixels * Point::new(r.x, r.y);
    let b = to_pixels * Point::new(r.x + r.w, r.y + r.h);
    Rect::new(a.x.round(), a.y.round(), b.x.round(), b.y.round())
}

/// Draw an image into its area: `Fit` scales it to fit inside and centres it; `Fill` scales it to
/// cover the area and centres it, the clip of the area cropping the rest.
fn draw_image(
    ctx: &mut RenderContext,
    to_pixels: Affine,
    area: SlideRect,
    clip: Rect,
    pixmap: &Arc<Pixmap>,
    fit: ImageFit,
) {
    let (iw, ih) = (f64::from(pixmap.width()), f64::from(pixmap.height()));
    let (sx, sy) = (area.w / iw, area.h / ih);
    let scale = match fit {
        ImageFit::Fit => sx.min(sy),
        ImageFit::Fill => sx.max(sy),
    };
    let x = area.x + (area.w - iw * scale) / 2.0;
    let y = area.y + (area.h - ih * scale) / 2.0;
    let image_to_pixels = to_pixels * Affine::translate((x, y)) * Affine::scale(scale);
    ctx.set_paint(Image {
        image: ImageSource::Pixmap(pixmap.clone()),
        sampler: ImageSampler::default(),
    });
    match fit {
        ImageFit::Fit => {
            ctx.set_transform(image_to_pixels);
            ctx.fill_rect(&Rect::new(0.0, 0.0, iw, ih));
        }
        // A filling image covers the whole area, so the clip itself is filled: an edge of the
        // image on a fractional pixel would otherwise blend with the white slide beneath.
        ImageFit::Fill => {
            ctx.set_transform(Affine::IDENTITY);
            ctx.set_paint_transform(image_to_pixels);
            ctx.fill_rect(&clip);
            ctx.set_paint_transform(Affine::IDENTITY);
        }
    }
}
