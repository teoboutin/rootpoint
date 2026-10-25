//! Formulas: the built-in math font, and the drawing of a formula the math crate laid out.

use std::sync::Arc;

use rootpoint_math::{FormulaLayout, MathFont};
use rootpoint_model::Rgba;
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
    use rootpoint_math::{Formula, Style, lay_out};
    use vello_cpu::kurbo::Affine;
    use vello_cpu::{Pixmap, RasterizerSettings, RenderContext, RenderSettings, Resources};

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

    /// The reference formulas drawn into PNG files, for the owner's comparison of AC1 with the
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
