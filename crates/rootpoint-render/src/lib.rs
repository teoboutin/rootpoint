//! The slide renderer of rootpoint: draws one slide of a document into an RGBA image, at any size,
//! from the same layout in slide units.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

use rootpoint_model::{Document, SlideId};
use vello_cpu::color::palette::css::{BLACK, WHITE};
use vello_cpu::kurbo::{Affine, Rect};
use vello_cpu::{Level, Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources};

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

/// A drawn slide: premultiplied RGBA pixels, row by row.
pub struct Rendered {
    pub pixmap: Pixmap,
}

impl Rendered {
    /// The image as a PNG file.
    pub fn png(self) -> Vec<u8> {
        self.pixmap
            .into_png()
            .expect("an RGBA pixmap always encodes")
    }
}

/// Draw the slide `slide` of `doc` into a `width`×`height` image. The slide is scaled uniformly to
/// the largest size that fits, centred, and the rest is black.
pub fn render_slide(
    doc: &Document,
    slide: SlideId,
    width: u16,
    height: u16,
    precision: Precision,
) -> Result<Rendered, RenderError> {
    if width == 0 || height == 0 {
        return Err(RenderError::EmptySize);
    }
    doc.slide(slide).ok_or(RenderError::UnknownSlide(slide))?;
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
    ctx.set_paint(BLACK);
    ctx.fill_rect(&Rect::new(0.0, 0.0, f64::from(width), f64::from(height)));
    ctx.set_transform(slide_transform(doc, width, height));
    let whole = doc.aspect.slide_rect();
    ctx.set_paint(WHITE);
    ctx.fill_rect(&Rect::new(0.0, 0.0, whole.w, whole.h));
    ctx.flush();
    let mut pixmap = Pixmap::new(width, height);
    ctx.render_with(
        &mut pixmap,
        &mut Resources::new(),
        RasterizerSettings::default(),
    );
    Ok(Rendered { pixmap })
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
