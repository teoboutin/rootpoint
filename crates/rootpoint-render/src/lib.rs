//! The slide renderer of rootpoint: draws one slide of a document into an RGBA image, at any size,
//! from the same layout in slide units.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod formula;
mod images;
mod map;
mod text;

use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::sync::Arc;

pub use formula::AreaFormula;

/// The bytes of the built-in math font, Libertinus Math, which the editor's own controls draw a
/// symbol of the palette with.
pub fn builtin_math_font() -> &'static [u8] {
    formula::BUILTIN_MATH
}
pub use images::{DecodeError, ImageSet, decode_rgba};
pub use map::TextMap;
use parley::Layout;
use rootpoint_math::{FormulaLayout, Style};
use rootpoint_model::{
    AssetId, Content, Document, ImageFit, Node, NodeId, Rect as SlideRect, SlideId, SpanChoice,
    SpanId, SpanStyle, StyleChoice, TextContent, TextStyle, VAlign, area_rects,
};
pub use text::{FontError, FontSet, TextBrush};
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
    /// The text areas whose font, or a font one of its spans sets, no font of the `FontSet` has;
    /// the built-in sans stands in.
    pub font_fallbacks: Vec<NodeId>,
    /// The text areas holding a character that no font of its font stack has, drawn as a box: the
    /// text's or its span's font, then the built-in fonts; a font asset outside the stack is not
    /// searched. Also the equation areas whose formula, or number, holds a character its font
    /// lacks.
    pub missing_glyphs: Vec<NodeId>,
    /// The text areas naming a style the document's table lacks, with the name; each is drawn with
    /// `TextStyle::default()`.
    pub missing_styles: Vec<(NodeId, String)>,
    /// The spans naming a span style the document's table lacks, each with its area and the name;
    /// each is drawn with its parent's values.
    pub missing_span_styles: Vec<(NodeId, SpanId, String)>,
    /// The equation areas, and the text areas holding a formula, whose formula holds a command the
    /// math subset does not know, drawn red.
    pub unknown_commands: Vec<NodeId>,
    /// The references of a text to an area that holds no equation, each with its area and the area
    /// it names; each is drawn `(??)`.
    pub dangling_references: Vec<(NodeId, NodeId)>,
    /// The text and equation areas whose glyphs could not all be drawn, as from a damaged font file.
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
pub(crate) const TEXT_PADDING: f64 = 24.0;

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
    let numbers = doc.equation_numbers();
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
                if let StyleChoice::Named(name) = &content.style
                    && !doc.styles.contains_key(name)
                {
                    report.missing_styles.push((id, name.clone()));
                }
                for span in content.root.spans() {
                    if let SpanChoice::Named(name) = &span.style
                        && !doc.span_styles.contains_key(name)
                    {
                        report.missing_span_styles.push((id, span.id, name.clone()));
                    }
                }
                for target in content.root.refs() {
                    if !doc.holds_equation(target) {
                        report.dangling_references.push((id, target));
                    }
                }
                let style = doc.resolve_style(&content.style);
                let (laid, origin, _) = lay_out_area(doc, fonts, content, &style, area, &numbers);
                if laid.fell_back {
                    report.font_fallbacks.push(id);
                }
                if laid.missing_glyph {
                    report.missing_glyphs.push(id);
                }
                if laid.unknown_command {
                    report.unknown_commands.push(id);
                }
                let base = to_pixels * Affine::translate(origin);
                if !text::draw_text(&mut ctx, &mut resources, &laid, &fonts.math.data, base) {
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
            Content::Equation(equation) => {
                let laid = formula::lay_out_equation(
                    doc,
                    fonts,
                    equation,
                    numbers.get(&id).copied(),
                    area,
                );
                ctx.set_transform(
                    to_pixels
                        * Affine::translate(laid.formula.origin)
                        * Affine::scale(laid.formula.scale),
                );
                let drawn = formula::draw_formula(
                    &mut ctx,
                    &mut resources,
                    &fonts.math.data,
                    &laid.formula.layout,
                    equation.colour,
                );
                let mut all = drawn.all;
                let mut missing = drawn.missing_glyph;
                if let Some((number, origin)) = &laid.number {
                    let base = to_pixels * Affine::translate(*origin);
                    all &=
                        text::draw_text(&mut ctx, &mut resources, number, &fonts.math.data, base);
                    missing |= number.missing_glyph;
                }
                if missing {
                    report.missing_glyphs.push(id);
                }
                if drawn.unknown_command {
                    report.unknown_commands.push(id);
                }
                if !all {
                    report.undrawn_text.push(id);
                }
            }
        }
        ctx.pop_layer();
    }
    ctx.flush();
    let mut pixmap = Pixmap::new(width, height);
    ctx.render_with(&mut pixmap, &mut resources, RasterizerSettings::default());
    Ok(Rendered { pixmap, report })
}

/// The text of one area as `render_slide` draws it: its layout, in slide units, the point of the
/// slide, in slide units, where the layout's origin is drawn, the map from the text's caret
/// positions to the layout's byte indices, and each formula of the text. An editor that places a
/// caret or a selection on this layout, with Parley's `Selection` and `Cursor`, places it on the
/// drawn glyphs.
pub struct AreaText {
    pub layout: Layout<TextBrush>,
    pub origin: (f64, f64),
    pub map: TextMap,
    pub formulas: Vec<TextFormula>,
}

/// A formula of a text as `render_slide` draws it: the byte range of the laid-out string it covers,
/// its layout, and the point of the text's layout, in slide units, where the origin of its first
/// row's baseline is drawn.
#[derive(Clone, Debug)]
pub struct TextFormula {
    pub range: Range<usize>,
    pub layout: FormulaLayout,
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
    let style = doc.resolve_style(&content.style);
    let numbers = doc.equation_numbers();
    let (laid, origin, map) = lay_out_area(doc, fonts, content, &style, rect, &numbers);
    let mut placed = vec![(0.0, 0.0); laid.formulas.len()];
    for line in laid.layout.lines() {
        for item in line.items() {
            if let parley::PositionedLayoutItem::InlineBox(b) = item {
                let ascent = laid.formulas[b.id as usize].0.ascent;
                placed[b.id as usize] = (f64::from(b.x), f64::from(b.y) + ascent);
            }
        }
    }
    let formulas = map
        .formulas()
        .iter()
        .zip(laid.formulas)
        .zip(placed)
        .map(|((f, (layout, _)), origin)| TextFormula {
            range: f.range.clone(),
            layout,
            origin,
        })
        .collect();
    Some(AreaText {
        layout: laid.layout,
        origin,
        map,
        formulas,
    })
}

/// The formula of the equation area `area` of the slide `slide`, laid out as `render_slide` lays
/// it out; `None` when the slide or the area is not there, or the area holds no equation.
pub fn area_formula(
    doc: &Document,
    slide: SlideId,
    area: NodeId,
    fonts: &mut FontSet,
) -> Option<AreaFormula> {
    let the_slide = doc.slide(slide)?;
    let rect = area_rects(the_slide, doc.aspect)
        .into_iter()
        .find(|(id, _)| *id == area)?
        .1;
    let Some(Node::Area(a)) = the_slide.root.find(area) else {
        return None;
    };
    let Content::Equation(equation) = &a.content else {
        return None;
    };
    let number = doc.equation_numbers().get(&area).copied();
    Some(formula::lay_out_equation(doc, fonts, equation, number, rect).formula)
}

/// A text area's layout in its resolved style, wrapped to the area's width less its padding, the
/// point its origin is drawn at, inset by the padding and moved down by the vertical alignment's
/// share of the room left, and the map of its caret positions. `render_slide` and `area_text` both
/// lay a text out here, so they give one layout.
fn lay_out_area(
    doc: &Document,
    fonts: &mut FontSet,
    content: &TextContent,
    style: &TextStyle,
    area: SlideRect,
    numbers: &BTreeMap<NodeId, u32>,
) -> (text::TextLayout, (f64, f64), TextMap) {
    let inner = (area.w - 2.0 * TEXT_PADDING).max(0.0);
    // Each span's values over its byte range, outer spans first, as `TextNode::spans` lists them:
    // a span naming a style the table lacks sets nothing, and draws with its parent's values.
    let map = TextMap::new(content, numbers);
    let values: HashMap<SpanId, SpanStyle> = content
        .root
        .spans()
        .into_iter()
        .filter_map(|span| Some((span.id, doc.resolve_span(&span.style)?)))
        .collect();
    let spans: Vec<(Range<usize>, SpanStyle)> = content
        .root
        .spans()
        .into_iter()
        .filter_map(|span| Some((map.span_range(span.id)?, values.get(&span.id)?.clone())))
        .collect();
    // A formula takes the size and the colour its text node resolves to, the spans that hold it set
    // over the text's style, outer first; bold and italic do not apply to it.
    let formulas = map
        .formulas()
        .iter()
        .map(|f| {
            let (mut size, mut colour) = (style.size, style.colour);
            for v in f.spans.iter().filter_map(|id| values.get(id)) {
                size = v.size.unwrap_or(size);
                colour = v.colour.unwrap_or(colour);
            }
            text::InlineFormula {
                index: f.range.start,
                layout: rootpoint_math::lay_out(&f.formula, &fonts.math.font, Style::Text, size),
                colour,
            }
        })
        .collect();
    let laid = text::layout_text(fonts, map.text(), style, &spans, formulas, inner as f32);
    let free = area.h - 2.0 * TEXT_PADDING - f64::from(laid.layout.height());
    let dy = match style.v_align {
        VAlign::Top => 0.0,
        VAlign::Middle => free / 2.0,
        VAlign::Bottom => free,
    };
    (
        laid,
        (area.x + TEXT_PADDING, area.y + TEXT_PADDING + dy),
        map,
    )
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
