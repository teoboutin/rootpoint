//! Formulas: the built-in math font, and the drawing of a formula the math crate laid out.

use std::sync::Arc;

use rootpoint_math::{FormulaLayout, MathFont, Style, lay_out};
use rootpoint_model::{
    BODY, Document, EquationContent, EquationSize, FontRef, Rect as SlideRect, Rgba, StyleChoice,
    TextStyle,
};

use crate::FontSet;
use crate::TEXT_PADDING;
use crate::text::TextLayout;
use vello_cpu::kurbo::Rect;
use vello_cpu::peniko::{Blob, FontData};

/// Libertinus Math 7.051, under the SIL Open Font License 1.1, whose text sits beside it.
pub(crate) const BUILTIN_MATH: &[u8] =
    include_bytes!("../fonts/libertinus/LibertinusMath-Regular.otf");

/// The red an unknown command is drawn in.
const UNKNOWN: Rgba = Rgba {
    r: 204,
    g: 0,
    b: 0,
    a: 255,
};

/// The built-in math font, as the math crate reads it and as the rasterizer draws it.
pub(crate) struct BuiltinMath {
    pub font: MathFont,
    pub data: FontData,
}

impl BuiltinMath {
    pub fn new() -> Self {
        BuiltinMath {
            font: MathFont::new(BUILTIN_MATH).expect("the built-in math font has a MATH table"),
            data: FontData::new(Blob::new(Arc::new(BUILTIN_MATH)), 0),
        }
    }
}

/// An equation's formula as `render_slide` draws it: its layout, at the base size its size mode
/// lays it out at, the point of the slide, in slide units, where the origin of its first row's
/// baseline is drawn, and the scale the layout is drawn at.
#[derive(Clone, Debug)]
pub struct AreaFormula {
    pub layout: FormulaLayout,
    pub origin: (f64, f64),
    pub scale: f64,
}

/// An equation laid out in its area: its formula, and its number's text with the point its origin
/// is drawn at, when it is numbered.
pub(crate) struct EquationLayout {
    pub formula: AreaFormula,
    pub number: Option<(TextLayout, (f64, f64))>,
}

/// The space between the formula and its number, in ems of the number's size, kept free on both
/// sides of the formula so that it stays centred in its area.
const NUMBER_GAP: f64 = 1.0;

/// Lay out an equation in its area: its formula once, in display style, at the Body style's size
/// for `Fit` and at its size for `AtMost`, then scaled by the largest factor at which it fits the
/// area less the padding of a text, with room for its number on both sides, a factor no greater
/// than 1 for `AtMost`; the formula centred, the number `(n)` at the right edge, in the built-in
/// sans at the Body style's size, centred on the formula's axis, or on its rows when it has
/// several, `design@rootpoint-render@equation-one-layout`.
/// One layout and one multiplication suffice, since a layout scales exactly,
/// `design@rootpoint-math@layout-scales-exactly`.
pub(crate) fn lay_out_equation(
    doc: &Document,
    fonts: &mut FontSet,
    equation: &EquationContent,
    number: Option<u32>,
    area: SlideRect,
) -> EquationLayout {
    let body = doc.resolve_style(&StyleChoice::Named(BODY.to_owned()));
    let inner = SlideRect {
        x: area.x + TEXT_PADDING,
        y: area.y + TEXT_PADDING,
        w: (area.w - 2.0 * TEXT_PADDING).max(0.0),
        h: (area.h - 2.0 * TEXT_PADDING).max(0.0),
    };
    let base = match equation.size {
        EquationSize::Fit => body.size,
        EquationSize::AtMost(size) => size,
    };
    let layout = lay_out(&equation.formula, &fonts.math.font, Style::Display, base);
    let number = number.map(|n| {
        // Upright and regular whatever the Body style's weight and slant, as an equation number
        // is set beside its formula.
        let style = TextStyle {
            font: FontRef::Builtin("sans".to_owned()),
            colour: equation.colour,
            bold: false,
            italic: false,
            ..body.clone()
        };
        crate::text::layout_text(fonts, &format!("({n})"), &style, &[], f32::MAX)
    });
    let reserved = number.as_ref().map_or(0.0, |n| {
        f64::from(n.layout.width()) + NUMBER_GAP * body.size
    });
    let room_w = (inner.w - 2.0 * reserved).max(0.0);
    let height = layout.ascent + layout.descent;
    let fit = (room_w / layout.width).min(inner.h / height);
    let fit = if fit.is_finite() { fit.max(0.0) } else { 0.0 };
    let scale = match equation.size {
        EquationSize::Fit => fit,
        EquationSize::AtMost(_) => fit.min(1.0),
    };
    let top = inner.y + (inner.h - scale * height) / 2.0;
    let origin = (
        inner.x + (inner.w - scale * layout.width) / 2.0,
        top + scale * layout.ascent,
    );
    // A number stands beside the formula as a whole, so it is centred on the rows when there are
    // several, and on the one row's axis otherwise.
    let middle = if equation.formula.rows.len() > 1 {
        top + scale * height / 2.0
    } else {
        origin.1 - scale * layout.axis
    };
    let number = number.map(|n| {
        let w = f64::from(n.layout.width());
        let h = f64::from(n.layout.height());
        (n, (inner.x + inner.w - w, middle - h / 2.0))
    });
    EquationLayout {
        formula: AreaFormula {
            layout,
            origin,
            scale,
        },
        number,
    }
}

/// What drawing a formula met.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Drawn {
    /// Whether every glyph run was drawn.
    pub all: bool,
    /// Whether a character has no glyph in the font, drawn as its box.
    pub missing_glyph: bool,
    /// Whether an unknown command is drawn, in red.
    pub unknown_command: bool,
}

/// Draw a laid-out formula with the origin of its first row's baseline at the origin of the
/// context's transform, in `colour`, glyphs unhinted; an unknown command in red.
pub(crate) fn draw_formula(
    ctx: &mut vello_cpu::RenderContext,
    resources: &mut vello_cpu::Resources,
    font: &FontData,
    layout: &FormulaLayout,
    colour: Rgba,
) -> Drawn {
    let paint = |c: Rgba| {
        vello_cpu::color::AlphaColor::<vello_cpu::color::Srgb>::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let mut all = true;
    // One run per font size and colour, as vello_cpu draws a run at one size.
    let mut runs: Vec<(f64, bool, Vec<vello_cpu::Glyph>)> = Vec::new();
    for g in &layout.glyphs {
        let glyph = vello_cpu::Glyph {
            id: u32::from(g.glyph),
            x: g.x as f32,
            y: g.y as f32,
        };
        match runs
            .iter_mut()
            .find(|(size, unknown, _)| *size == g.size && *unknown == g.unknown)
        {
            Some(run) => run.2.push(glyph),
            None => runs.push((g.size, g.unknown, vec![glyph])),
        }
    }
    for (size, unknown, glyphs) in runs {
        ctx.set_paint(paint(if unknown { UNKNOWN } else { colour }));
        // Hinting snaps outlines to the pixel grid, which would make the slide differ between
        // sizes.
        all &= ctx
            .glyph_run(resources, font)
            .font_size(size as f32)
            .hint(false)
            .fill_glyphs(glyphs.into_iter())
            .is_ok();
    }
    ctx.set_paint(paint(colour));
    for r in &layout.rules {
        ctx.fill_rect(&Rect::new(r.x, r.y, r.x + r.width, r.y + r.height));
    }
    Drawn {
        all,
        missing_glyph: layout.missing_glyph,
        unknown_command: layout.unknown_command,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rootpoint_math::Formula;
    use vello_cpu::kurbo::Affine;
    use vello_cpu::{Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources};

    /// The number of an equation of one row is centred on its axis, and that of several rows on
    /// the rows.
    #[test]
    fn the_number_stands_on_the_axis_of_one_row_and_in_the_middle_of_several() {
        let doc = Document::default();
        let mut fonts = FontSet::new();
        let area = SlideRect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        };
        for text in ["x = 1", r"x &= 1 \\ y &= 2 \\ z &= 3"] {
            let equation = doc.new_equation(Formula::parse(text).unwrap());
            let laid = lay_out_equation(&doc, &mut fonts, &equation, Some(1), area);
            let f = &laid.formula;
            let (n, at) = laid.number.unwrap();
            let centre = at.1 + f64::from(n.layout.height()) / 2.0;
            let expected = if equation.formula.rows.len() > 1 {
                let top = f.origin.1 - f.scale * f.layout.ascent;
                top + f.scale * (f.layout.ascent + f.layout.descent) / 2.0
            } else {
                f.origin.1 - f.scale * f.layout.axis
            };
            assert!(
                (centre - expected).abs() < 1e-6,
                "{text}: {centre} ≠ {expected}"
            );
        }
    }

    /// The reference formulas of the slice #math-layout.
    const REFERENCE: [&str; 5] = [
        r"\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)",
        r"\int_{-\infty}^{\infty} P_n(x)\,\bar{\omega}(x)\,\mathrm{d}x = \sum_{i=1}^{N} w_i P_n(x_i)",
        r"f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}",
        r"R(\theta) = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix}",
        r"|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}",
    ];

    /// A formula drawn alone on white at `size` pixels, with a margin of a quarter of the size.
    fn draw(text: &str, style: Style, size: f64) -> Pixmap {
        let math = BuiltinMath::new();
        let layout = lay_out(&Formula::parse(text).unwrap(), &math.font, style, size);
        let margin = size / 4.0;
        let w = (layout.width + 2.0 * margin).ceil() as u16;
        let h = (layout.ascent + layout.descent + 2.0 * margin).ceil() as u16;
        let mut ctx = RenderContext::new_with(
            w,
            h,
            RenderSettings {
                level: vello_cpu::Level::fallback(),
                num_threads: 0,
            },
        );
        let mut resources = Resources::new();
        ctx.set_paint(vello_cpu::color::palette::css::WHITE);
        ctx.fill_rect(&Rect::new(0.0, 0.0, f64::from(w), f64::from(h)));
        ctx.set_transform(Affine::translate((margin, margin + layout.ascent)));
        let drawn = draw_formula(
            &mut ctx,
            &mut resources,
            &math.data,
            &layout,
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        );
        assert!(
            drawn.all && !drawn.missing_glyph && !drawn.unknown_command,
            "{text}"
        );
        ctx.flush();
        let mut pixmap = Pixmap::new(w, h);
        ctx.render_with(&mut pixmap, &mut resources, RasterizerSettings::default());
        pixmap
    }

    /// The reference formulas drawn into PNG files, for the owner's comparison with the
    /// browser's rendering of the same formulas: in display style, `reference-<n>.png`, and in
    /// text style, `reference-<n>-text.png`, in the directory `ROOTPOINT_MATH_IMAGES` names.
    #[test]
    #[ignore = "writes files for the owner's eye; run with ROOTPOINT_MATH_IMAGES set"]
    fn draw_the_reference_formulas() {
        let dir = std::env::var("ROOTPOINT_MATH_IMAGES")
            .expect("ROOTPOINT_MATH_IMAGES names a directory");
        for (i, text) in REFERENCE.iter().enumerate() {
            for (style, suffix) in [(Style::Display, ""), (Style::Text, "-text")] {
                let png = draw(text, style, 64.0).into_png().unwrap();
                std::fs::write(format!("{dir}/reference-{}{suffix}.png", i + 1), png).unwrap();
            }
        }
    }

    /// Every reference formula draws, with every glyph found.
    #[test]
    fn the_reference_formulas_draw() {
        for text in REFERENCE {
            let pixmap = draw(text, Style::Display, 24.0);
            assert!(pixmap.width() > 0);
        }
    }
}
