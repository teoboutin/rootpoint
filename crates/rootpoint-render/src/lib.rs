//! The slide renderer of rootpoint: draws one slide of a document into an RGBA image, at any size,
//! from the same layout in slide units.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod images;

use std::sync::Arc;

pub use images::{DecodeError, ImageSet};
use rootpoint_model::area_rects;
use rootpoint_model::{AssetId, Content, Document, ImageFit, Node, Rect as SlideRect, SlideId};
use vello_cpu::color::AlphaColor;
use vello_cpu::color::palette::css::{BLACK, WHITE};
use vello_cpu::kurbo::{Affine, Rect, Shape};
use vello_cpu::peniko::ImageSampler;
use vello_cpu::{
    Image, ImageSource, Level, Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources,
};

/// Which SIMD code the rasterizer runs. `Native` is the fastest the machine has; `Portable` gives
/// the same pixels on every machine, for golden images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    Native,
    Portable,
}

/// Why a slide was not drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderError {
    UnknownSlide(SlideId),
    /// A side of zero pixels.
    EmptySize,
}

/// What a render could not draw as the document asks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// The image assets with no decoded image in the `ImageSet`; their areas are drawn empty.
    pub missing_images: Vec<AssetId>,
}

/// A drawn slide: premultiplied RGBA pixels, row by row, and what the render could not draw.
pub struct Rendered {
    pub pixmap: Pixmap,
    pub report: Report,
}

impl Rendered {
    /// The image as a PNG file.
    pub fn png(self) -> Vec<u8> {
        self.pixmap
            .into_png()
            .expect("an RGBA pixmap always encodes")
    }
}

/// The grey of an image area whose image is missing.
const MISSING: AlphaColor<vello_cpu::color::Srgb> = AlphaColor::from_rgba8(224, 224, 224, 255);

/// Draw the slide `slide` of `doc` into a `width`×`height` image. The slide is scaled uniformly to
/// the largest size that fits, centred, and the rest is black.
pub fn render_slide(
    doc: &Document,
    slide: SlideId,
    width: u16,
    height: u16,
    images: &ImageSet,
    precision: Precision,
) -> Result<Rendered, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::EmptySize);
    }
    let the_slide = doc.slide(slide).ok_or(RenderError::UnknownSlide(slide))?;
    let level = match precision {
        Precision::Native => Level::try_detect().unwrap_or(Level::baseline()),
        Precision::Portable => Level::baseline(),
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
    ctx.set_paint(BLACK);
    ctx.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    let to_pixels = slide_transform(doc, width, height);
    ctx.set_transform(to_pixels);
    let whole = doc.aspect.slide_rect();
    ctx.set_paint(WHITE);
    ctx.fill_rect(&Rect::new(0.0, 0.0, whole.w, whole.h));
    for (id, area) in area_rects(the_slide, doc.aspect) {
        let Some(Node::Area(a)) = the_slide.root.find(id) else {
            continue;
        };
        let bounds = Rect::new(area.x, area.y, area.x + area.w, area.y + area.h);
        ctx.set_transform(to_pixels);
        ctx.push_clip_layer(&bounds.to_path(0.1));
        match &a.content {
            Content::Empty | Content::Text(_) => {}
            Content::Image(image) => match images.get(image.asset) {
                Some(pixmap) => draw_image(&mut ctx, to_pixels, area, pixmap, image.fit),
                None => {
                    report.missing_images.push(image.asset);
                    ctx.set_paint(MISSING);
                    ctx.fill_rect(&bounds);
                }
            },
        }
        ctx.pop_layer();
    }
    ctx.flush();
    let mut pixmap = Pixmap::new(width, height);
    ctx.render_with(
        &mut pixmap,
        &mut Resources::new(),
        RasterizerSettings::default(),
    );
    Ok(Rendered { pixmap, report })
}

/// The transform from slide units to pixels: one uniform scale, and a translation that centres
/// the slide.
fn slide_transform(doc: &Document, width: u16, height: u16) -> Affine {
    let whole = doc.aspect.slide_rect();
    let (w, h) = (f64::from(width), f64::from(height));
    let scale = (w / whole.w).min(h / whole.h);
    let x = (w - whole.w * scale) / 2.0;
    let y = (h - whole.h * scale) / 2.0;
    Affine::translate((x, y)) * Affine::scale(scale)
}

/// Draw an image into its area: `Fit` scales it to fit inside and centres it; `Fill` scales it to
/// cover the area and centres it, the clip of the area cropping the rest.
fn draw_image(
    ctx: &mut RenderContext,
    to_pixels: Affine,
    area: SlideRect,
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
    ctx.set_transform(to_pixels * Affine::translate((x, y)) * Affine::scale(scale));
    ctx.set_paint(Image {
        image: ImageSource::Pixmap(pixmap.clone()),
        sampler: ImageSampler::default(),
    });
    ctx.fill_rect(&Rect::new(0.0, 0.0, iw, ih));
}
