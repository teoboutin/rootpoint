//! An equation area draws its formula fitted to the area, or held at a size, with its number at
//! the right edge; `area_formula` gives the layout `render_slide` draws.

mod common;

use common::*;
use rootpoint_math::Formula;
use rootpoint_model::*;
use rootpoint_render::{FontSet, ImageSet, Precision, area_formula, render_slide};

const FORMULA: &str = r"\int_{-\infty}^{\infty} e^{-x^2}\,\mathrm{d}x = \sqrt{\pi}";

fn equation(doc: &Document, text: &str, size: EquationSize) -> Content {
    Content::Equation(EquationContent {
        size,
        ..doc.new_equation(Formula::parse(text).unwrap())
    })
}

/// One slide, split beside at a third: the left area a small equation, the right a large one,
/// numbered (1) and (2), both fitted.
fn two_sizes() -> Document {
    let (mut doc, root) = one_slide();
    split(&mut doc, root, Axis::LeftToRight, 1.0 / 3.0);
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    for area in areas {
        let content = equation(&doc, FORMULA, EquationSize::Fit);
        set(&mut doc, area, content);
    }
    doc
}

fn render(doc: &Document, width: u16, height: u16) -> rootpoint_render::Rendered {
    render_slide(
        doc,
        doc.slides[0].id,
        width,
        height,
        &mut FontSet::new(),
        &ImageSet::new(),
        Precision::Portable,
    )
    .unwrap()
}

#[test]
fn an_equation_area_at_two_sizes_matches_its_golden_image() {
    let doc = two_sizes();
    let out = render(&doc, 960, 540);
    assert_eq!(out.report.missing_glyphs, vec![]);
    assert_eq!(out.report.unknown_commands, vec![]);
    assert_golden(out.pixmap, "equations.png");
}

/// `Fit` fills the room of its area on one axis, and `AtMost` holds its size while it fits.
#[test]
fn fit_fills_its_area_and_at_most_holds_its_size() {
    let doc = two_sizes();
    let slide = doc.slides[0].id;
    let areas: Vec<NodeId> = doc.slides[0].root.areas().iter().map(|a| a.id).collect();
    let rects = area_rects(&doc.slides[0], doc.aspect);
    let mut fonts = FontSet::new();
    let small = area_formula(&doc, slide, areas[0], &mut fonts).unwrap();
    let large = area_formula(&doc, slide, areas[1], &mut fonts).unwrap();
    // The right area is twice as wide as the left, and as high, so its scale is at most twice.
    assert!(
        large.scale > small.scale,
        "{} > {}",
        large.scale,
        small.scale
    );
    let inner_h = rects[1].1.h - 48.0;
    let height = large.scale * (large.layout.ascent + large.layout.descent);
    let width_fills = large.scale * large.layout.width > rects[1].1.w * 0.5;
    assert!(
        (height - inner_h).abs() < 1e-6 || width_fills,
        "fitted on one axis"
    );
    // Held at 20 slide units, which fits, the scale is 1.
    let mut doc = doc.clone();
    let content = equation(&doc, FORMULA, EquationSize::AtMost(20.0));
    set(&mut doc, areas[1], content);
    let held = area_formula(&doc, slide, areas[1], &mut fonts).unwrap();
    assert_eq!(held.scale, 1.0);
    // Held at 2000, which does not fit, it is scaled as `Fit` lays out at 2000.
    let content = equation(&doc, FORMULA, EquationSize::AtMost(2000.0));
    set(&mut doc, areas[1], content);
    let shrunk = area_formula(&doc, slide, areas[1], &mut fonts).unwrap();
    assert!(shrunk.scale < 1.0);
    assert!((shrunk.scale * shrunk.layout.width - large.scale * large.layout.width).abs() < 1e-6);
}

#[test]
fn an_unknown_command_and_a_missing_glyph_are_reported() {
    let (mut doc, root) = one_slide();
    let content = equation(&doc, r"\foo + \Game", EquationSize::Fit);
    set(&mut doc, root, content);
    let out = render(&doc, 384, 216);
    assert_eq!(out.report.unknown_commands, vec![root]);
    assert_eq!(out.report.missing_glyphs, vec![root]);
}

#[test]
fn area_formula_is_none_for_a_text() {
    let (doc, root) = one_slide();
    assert!(area_formula(&doc, doc.slides[0].id, root, &mut FontSet::new()).is_none());
}
