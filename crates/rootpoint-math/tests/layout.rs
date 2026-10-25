//! The layout of a formula from Libertinus Math's MATH table.

mod common;

use rootpoint_math::{Atom, Formula, FormulaLayout, MAX_DEPTH, Style, lay_out};

/// The reference formulas of the milestone of math, which AC1 and AC2 lay out: the first two are
/// equations (61) and (62) of page 60 of the owner's thesis.
const REFERENCE: [&str; 5] = [
    r"\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)",
    r"\int_{-\infty}^{\infty} P_n(x)\,\bar{\omega}(x)\,\mathrm{d}x = \sum_{i=1}^{N} w_i P_n(x_i)",
    r"f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}",
    r"R(\theta) = \begin{pmatrix} \cos\theta & -\sin\theta \\ \sin\theta & \cos\theta \end{pmatrix}",
    r"|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}",
];

fn laid(text: &str, style: Style, size: f64) -> FormulaLayout {
    lay_out(&Formula::parse(text).unwrap(), &common::font(), style, size)
}

/// Every value of `b` within 0.001 of `a`'s scaled by `k`.
fn assert_scaled(a: &FormulaLayout, b: &FormulaLayout, k: f64, what: &str) {
    let near = |x: f64, y: f64, name: &str| {
        assert!((x * k - y).abs() <= 0.001, "{what}: {name} {x} × {k} ≠ {y}");
    };
    near(a.width, b.width, "width");
    near(a.ascent, b.ascent, "ascent");
    near(a.descent, b.descent, "descent");
    near(a.axis, b.axis, "axis");
    assert_eq!(a.atoms.len(), b.atoms.len());
    for (p, q) in a.atoms.iter().zip(&b.atoms) {
        assert_eq!(p.path, q.path);
        for (x, y, n) in [
            (p.x, q.x, "atom x"),
            (p.y, q.y, "atom y"),
            (p.width, q.width, "atom width"),
            (p.ascent, q.ascent, "atom ascent"),
            (p.descent, q.descent, "atom descent"),
        ] {
            near(x, y, n);
        }
    }
    assert_eq!(a.cells.len(), b.cells.len());
    for (p, q) in a.cells.iter().zip(&b.cells) {
        assert_eq!(p.path, q.path);
        for (x, y, n) in [
            (p.x, q.x, "cell x"),
            (p.y, q.y, "cell y"),
            (p.width, q.width, "cell width"),
            (p.ascent, q.ascent, "cell ascent"),
            (p.descent, q.descent, "cell descent"),
        ] {
            near(x, y, n);
        }
    }
    assert_eq!(a.glyphs.len(), b.glyphs.len());
    for (p, q) in a.glyphs.iter().zip(&b.glyphs) {
        assert_eq!(p.glyph, q.glyph);
        near(p.x, q.x, "glyph x");
        near(p.y, q.y, "glyph y");
        near(p.size, q.size, "glyph size");
    }
    assert_eq!(a.rules.len(), b.rules.len());
    for (p, q) in a.rules.iter().zip(&b.rules) {
        near(p.x, q.x, "rule x");
        near(p.y, q.y, "rule y");
        near(p.width, q.width, "rule width");
        near(p.height, q.height, "rule height");
    }
}

/// AC2: a formula laid out at `k` times a size is the formula at the size scaled by `k`.
#[test]
fn a_formula_fitted_by_scaling_equals_the_formula_laid_out_at_the_scaled_size() {
    for text in REFERENCE {
        for style in [Style::Display, Style::Text] {
            let at_s = laid(text, style, 48.0);
            for k in [0.25, 0.5, 1.5, 2.0, 3.7] {
                assert_scaled(&at_s, &laid(text, style, 48.0 * k), k, text);
            }
        }
    }
}

/// AC4: a script's glyph is the font's `ssty` form where the font has one, as for the prime, and
/// its base glyph where it has none, as for the italic a.
#[test]
fn a_script_is_drawn_with_the_font_s_ssty_glyph() {
    let font = common::font();
    let prime = font.glyph('′').unwrap();
    let script_prime = font.script_glyph(prime);
    assert_ne!(script_prime, prime, "the font substitutes the prime");
    let l = laid(r"x^{\prime}", Style::Display, 48.0);
    assert!(l.glyphs.iter().any(|g| g.glyph == script_prime));
    assert!(l.glyphs.iter().all(|g| g.glyph != prime));
    let italic_a = font.glyph('\u{1D44E}').unwrap();
    assert_eq!(font.script_glyph(italic_a), italic_a);
    let l = laid("x^{a}", Style::Display, 48.0);
    assert!(
        l.glyphs
            .iter()
            .any(|g| g.glyph == italic_a && g.size < 48.0)
    );
}

#[test]
fn an_empty_cell_is_a_placeholder_half_an_em_wide() {
    let l = laid(r"\frac{}{b}", Style::Display, 48.0);
    let empty: Vec<_> = l.cells.iter().filter(|c| c.empty).collect();
    assert_eq!(empty.len(), 1);
    // The numerator of a display fraction is in text style, at the full size.
    assert!((empty[0].width - 24.0).abs() < 1e-9);
    let l = laid("", Style::Display, 48.0);
    assert_eq!(l.cells.len(), 1);
    assert!(l.cells[0].empty);
}

/// Every atom of a formula, at every depth.
fn count_atoms(list: &[Atom]) -> usize {
    list.iter()
        .map(|a| match a {
            Atom::Matrix { rows, .. } => {
                1 + rows.iter().flatten().map(|c| count_atoms(c)).sum::<usize>()
            }
            other => 1 + other.cells().iter().map(|c| count_atoms(c)).sum::<usize>(),
        })
        .sum()
}

#[test]
fn every_atom_has_a_box_and_glyphs_are_found() {
    for text in REFERENCE.iter().chain([&r"a\,b \quad c"]) {
        let l = laid(text, Style::Display, 48.0);
        assert!(!l.missing_glyph, "{text}");
        assert!(!l.unknown_command, "{text}");
        assert!(l.width > 0.0 && l.ascent > 0.0, "{text}");
        let f = Formula::parse(text).unwrap();
        let atoms: usize = f.rows.iter().flatten().map(|c| count_atoms(c)).sum();
        assert_eq!(l.atoms.len(), atoms, "{text}");
    }
    let l = laid(r"\foo", Style::Text, 48.0);
    assert!(l.unknown_command);
    assert!(l.glyphs.iter().all(|g| g.unknown));
}

#[test]
fn an_italic_letter_is_drawn_from_the_math_italic_block() {
    let font = common::font();
    let l = laid("x", Style::Text, 10.0);
    assert_eq!(l.glyphs[0].glyph, font.glyph('\u{1D465}').unwrap());
    let l = laid(r"\mathrm{x}", Style::Text, 10.0);
    assert_eq!(l.glyphs[0].glyph, font.glyph('x').unwrap());
}

/// A command whose character the font lacks is read, drawn as the font's missing glyph and
/// reported, by #math-glyph-fallback, though the table the palette lists holds none.
#[test]
fn a_command_the_font_lacks_is_drawn_missing() {
    let f = Formula::parse(r"\Game").unwrap();
    assert!(!f.has_unknown());
    let l = lay_out(&f, &common::font(), Style::Text, 10.0);
    assert!(l.missing_glyph);
    assert!(!l.unknown_command);
    assert_eq!(l.glyphs[0].glyph, 0);
}

/// An empty group takes no width, as TeX gives it none, while an empty cell of a structure is a
/// placeholder.
#[test]
fn an_empty_group_takes_no_width() {
    let with = laid(r"{}^{14}\mathrm{C}", Style::Text, 48.0);
    let without = laid(r"^{14}\mathrm{C}", Style::Text, 48.0);
    assert!((with.width - without.width).abs() < 1e-9);
    assert!(with.cells.iter().any(|c| c.empty && c.width == 0.0));
}

/// A delimiter built from parts is centred on the axis, its ink in its box.
#[test]
fn an_assembled_bar_is_centred_on_the_axis() {
    let text = r"\left| \begin{matrix} a \\ b \\ c \\ d \\ e \\ f \\ g \\ h \end{matrix} \right.";
    let l = laid(text, Style::Display, 100.0);
    let atom = &l.atoms[0];
    let centre = (atom.ascent - atom.descent) / 2.0;
    assert!(
        (centre - l.axis).abs() < 2.0,
        "centre {centre} against the axis {}",
        l.axis
    );
}

/// A formula deeper than `MAX_DEPTH` is refused when read, and cut when laid out, rather than
/// overflowing the stack.
#[test]
fn deep_nesting_is_bounded() {
    let deep = |n: usize| format!("{}x{}", "{".repeat(n), "}".repeat(n));
    // `n` groups around `x` nest `n + 1` atoms.
    assert!(Formula::parse(&deep(MAX_DEPTH - 1)).is_ok());
    assert!(Formula::parse(&deep(MAX_DEPTH)).is_err());
    assert!(Formula::parse(&deep(5000)).is_err());
    // A script's argument nests as a group does, and `\hat` with one atom too.
    let scripts = |n: usize| format!("{}x{}", "x^{".repeat(n), "}".repeat(n));
    assert!(Formula::parse(&scripts(MAX_DEPTH - 1)).is_ok());
    assert!(Formula::parse(&scripts(MAX_DEPTH)).is_err());
    assert!(Formula::parse(&scripts(5000)).is_err());
    assert!(Formula::parse(&format!("{}x", r"\hat ".repeat(5000))).is_err());
    // The deepest formula `parse` gives is laid out whole, with no cell cut.
    let l = laid(&scripts(MAX_DEPTH - 1), Style::Text, 10.0);
    assert!(l.cells.iter().all(|c| !c.empty));
    let mut list = vec![Atom::Unknown("x".to_owned())];
    for _ in 0..5000 {
        list = vec![Atom::Group(list)];
    }
    let l = lay_out(
        &Formula {
            rows: vec![vec![list]],
        },
        &common::font(),
        Style::Text,
        10.0,
    );
    assert!(l.width > 0.0);
}
