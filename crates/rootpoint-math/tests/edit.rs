//! The editing of a formula at a caret: the moves visit every position once each way, the edits
//! keep the formula one its LaTeX reads back to, and every position has a caret box on the layout.

mod common;

use proptest::prelude::*;
use rootpoint_math::*;

fn parse(text: &str) -> Formula {
    Formula::parse(text).unwrap()
}

/// Every position of a formula, in the order Right visits them from the start.
fn by_right(f: &Formula) -> Vec<CellPath> {
    let mut out = vec![start()];
    while let Some(next) = right(f, out.last().unwrap()) {
        assert!(out.len() < 10_000, "Right does not reach the end");
        out.push(next);
    }
    out
}

/// Right visits each position once, from the start to the end, and Left the same positions in the
/// reverse order: a fraction's numerator, then its denominator, a script's nucleus then its
/// scripts, a matrix's cells row by row, the grid's cells.
#[test]
fn right_and_left_visit_every_position_once_each_way() {
    for text in [
        r"x",
        r"\frac{a}{b} + c",
        r"x_i^2 = \sqrt{y}",
        r"f(x) &= (x+1)^2 \\ &= x^2 + 1",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
        r"\left( \frac{1}{2} \right) \hat{x}",
        r"",
    ] {
        let f = parse(text);
        let forward = by_right(&f);
        assert_eq!(forward.last(), Some(&end(&f)), "{text}");
        let mut backward = vec![end(&f)];
        while let Some(previous) = left(&f, backward.last().unwrap()) {
            backward.push(previous);
        }
        backward.reverse();
        assert_eq!(forward, backward, "{text}");
        let unique: std::collections::HashSet<_> = forward.iter().collect();
        assert_eq!(
            unique.len(),
            forward.len(),
            "{text}: a position visited twice"
        );
    }
}

/// Up and Down move between a fraction's numerator and denominator, a script and its nucleus, the
/// rows of a matrix and of the formula.
#[test]
fn up_and_down_move_between_the_cells_above_and_below() {
    let f = parse(r"\frac{ab}{c}");
    let in_num = by_right(&f)[2].clone();
    let down = vertical_move(&f, &in_num, false).unwrap();
    assert_eq!(down.steps, vec![(0, 1)]);
    assert_eq!(down.offset, 1);
    assert_eq!(vertical_move(&f, &down, true).unwrap().steps, vec![(0, 0)]);
    assert_eq!(vertical_move(&f, &down, false), None);
    let f = parse(r"x_i^2");
    let nucleus = by_right(&f)[1].clone();
    assert_eq!(nucleus.steps, vec![(0, 0)]);
    assert_eq!(
        vertical_move(&f, &nucleus, true).unwrap().steps,
        vec![(0, 2)]
    );
    assert_eq!(
        vertical_move(&f, &nucleus, false).unwrap().steps,
        vec![(0, 1)]
    );
    let f = parse(r"a \\ b");
    assert_eq!(vertical_move(&f, &start(), false).unwrap().row, 1);
}

/// `^` and `_` make the atom before the caret a nucleus, a group giving its list, as LaTeX reads
/// `{ab}^2`; a second `^` enters the superscript there is.
#[test]
fn a_script_takes_the_atom_before_the_caret() {
    let mut f = parse("x");
    let caret = script(&mut f, &with(1), true).unwrap();
    assert_eq!(f.to_latex(), "x^{}");
    let caret = insert(&mut f, &caret, typed_atom('2').unwrap()).unwrap();
    assert_eq!(f.to_latex(), "x^2");
    let out = leave_cell(&caret).unwrap();
    let caret = script(&mut f, &out, false).unwrap();
    assert_eq!(f.to_latex(), "x_{}^2");
    assert_eq!(caret.steps, vec![(0, 1)]);
    let mut f = parse("{ab}");
    script(&mut f, &with(1), true).unwrap();
    assert_eq!(f.to_latex(), "{ab}^{}");
    assert_eq!(Formula::parse(&f.to_latex()).unwrap(), f);
}

fn with(offset: usize) -> CellPath {
    CellPath {
        offset,
        ..CellPath::default()
    }
}

/// A command typed after `\` names its atom: a symbol of the table, an operator name, or a
/// structure with empty cells, the caret then in its first cell.
#[test]
fn a_typed_command_names_its_atom() {
    let mut f = Formula::empty();
    let frac = command_atom("frac").unwrap();
    let caret = insert(&mut f, &start(), frac).unwrap();
    assert_eq!(f.to_latex(), r"\frac{}{}");
    assert_eq!(caret.steps, vec![(0, 0)]);
    assert_eq!(
        command_atom("leq"),
        Some(Atom::Symbol {
            ch: '≤',
            class: Class::Rel,
            name: Some("leq")
        })
    );
    assert_eq!(command_atom("sin"), Some(Atom::OperatorName("sin")));
    assert!(matches!(command_atom("hat"), Some(Atom::Accent { .. })));
    assert_eq!(command_atom("foo"), Some(Atom::Unknown("foo".to_owned())));
    assert_eq!(command_atom("left"), None);
    assert_eq!(complete("lefta"), "leftarrow");
    assert!(completions("fra").contains(&"frac"));
}

/// Backspace deletes a symbol, selects an atom with cells or the atom at whose cell's start it
/// stands, joins two cells of the grid, and at the formula's start does nothing.
#[test]
fn backspace_deletes_a_symbol_and_selects_an_atom_with_cells() {
    let mut f = parse(r"a\frac{b}{c}d");
    assert_eq!(
        delete(&mut f, &with(3), false),
        Some(Deletion::Deleted(with(2)))
    );
    assert_eq!(
        delete(&mut f, &with(2), false),
        Some(Deletion::Select(with(1)))
    );
    assert_eq!(delete_atom(&mut f, &with(1)), Some(with(1)));
    assert_eq!(f.to_latex(), "a");
    assert_eq!(delete(&mut f, &with(0), false), Some(Deletion::Nothing));
    let mut f = parse(r"a & b");
    let second = CellPath {
        column: 1,
        ..CellPath::default()
    };
    assert_eq!(
        delete(&mut f, &second, false),
        Some(Deletion::Deleted(with(1)))
    );
    assert_eq!(f.to_latex(), "ab");
}

/// `&` cuts a cell of the grid in two, Ctrl+Return starts a new row with what follows the caret,
/// and in a matrix they move to the next cell and add a row.
#[test]
fn ampersand_and_new_row_shape_the_grid() {
    let mut f = parse("ab");
    let caret = ampersand(&mut f, &with(1)).unwrap();
    assert_eq!(f.to_latex(), "a & b");
    assert_eq!((caret.row, caret.column, caret.offset), (0, 1, 0));
    let caret = new_row(&mut f, &caret).unwrap();
    assert_eq!(f.to_latex(), r"a & \\ b");
    assert_eq!((caret.row, caret.column), (1, 0));
    let mut f = parse(r"\begin{matrix} a & b \end{matrix}");
    let in_a = by_right(&f)[1].clone();
    let next = ampersand(&mut f, &in_a).unwrap();
    assert_eq!(next.steps, vec![(0, 1)]);
    new_row(&mut f, &next).unwrap();
    assert_eq!(f.to_latex(), r"\begin{matrix} a & b \\ & \end{matrix}");
    edit_matrix(&mut f, &in_a, false, MatrixEdit::Delete).unwrap();
    assert_eq!(f.to_latex(), r"\begin{matrix} b \\ \end{matrix}");
}

/// Every position of a laid-out formula has a caret box, inside the box of the cell it stands in.
#[test]
fn every_position_has_a_caret_box_in_its_cell() {
    let font = common::font();
    for text in [
        r"\frac{\partial f_i}{\partial t} + \mathbf{c}_i \cdot \nabla_{\mathbf{x}} f_i = \Omega(f_i)",
        r"f(x) &= (x+1)^2 \\ &= x^2 + 2x + 1 \\ &\leq \sqrt{\frac{a^2+b^2}{2}}",
        r"|x| = \begin{cases} x & \text{if } x \geq 0 \\ -x & \text{otherwise} \end{cases}",
        r"\frac{}{} + {}^{}",
    ] {
        let f = parse(text);
        let layout = lay_out(&f, &font, Style::Display, 40.0);
        for p in by_right(&f) {
            let b = caret_box(&layout, &p).unwrap_or_else(|| panic!("{text}: {p:?}"));
            let cell = layout
                .cells
                .iter()
                .find(|c| {
                    c.path.steps == p.steps && (c.path.row, c.path.column) == (p.row, p.column)
                })
                .unwrap();
            assert!(
                b.x >= cell.x - 1e-9 && b.x <= cell.x + cell.width + 1e-9,
                "{text}: {p:?}"
            );
            // A click on the caret's place gives the position back, where no other position
            // stands at the same place.
            let back = path_at(&layout, b.x, b.y - b.ascent / 2.0).unwrap();
            let same: Vec<_> = by_right(&f)
                .into_iter()
                .filter(|q| {
                    caret_box(&layout, q)
                        .is_some_and(|c| (c.x - b.x).abs() < 1e-9 && (c.y - b.y).abs() < 1e-9)
                })
                .collect();
            if same.len() == 1 {
                assert_eq!(back, p, "{text}");
            }
        }
    }
}

/// An edit at a random position of a random formula keeps a formula whose LaTeX reads back to it,
/// or, where the parser reads its LaTeX otherwise, `clamp` keeps the caret inside the formula read
/// back.
fn kind() -> impl Strategy<Value = u8> {
    0u8..9
}

proptest! {
    #[test]
    fn random_edits_keep_the_formula_and_the_caret(
        steps in prop::collection::vec((kind(), 0usize..64, any::<char>()), 1..30)
    ) {
        let mut f = parse(r"\frac{a}{b} + x_i^2 \\ \begin{pmatrix} 1 & 2 \end{pmatrix}");
        let mut caret = start();
        for (k, pick, ch) in steps {
            let positions = by_right(&f);
            if k == 0 {
                caret = positions[pick % positions.len()].clone();
                continue;
            }
            let next = match k {
                1 => typed_atom(if ch.is_ascii_alphanumeric() { ch } else { 'y' })
                    .and_then(|a| insert(&mut f, &caret, a)),
                2 => script(&mut f, &caret, pick % 2 == 0),
                3 => match delete(&mut f, &caret, pick % 2 == 0) {
                    Some(Deletion::Deleted(p)) => Some(p),
                    Some(Deletion::Select(at)) => delete_atom(&mut f, &at),
                    _ => None,
                },
                4 => ampersand(&mut f, &caret),
                5 => new_row(&mut f, &caret),
                6 => command_atom(["frac", "sqrt", "hat", "alpha", "sin"][pick % 5])
                    .and_then(|a| insert(&mut f, &caret, a)),
                7 => edit_matrix(&mut f, &caret, pick % 2 == 0, [MatrixEdit::Append, MatrixEdit::Delete, MatrixEdit::Copy, MatrixEdit::Swap][pick % 4]),
                _ => right(&f, &caret),
            };
            if let Some(p) = next {
                caret = p;
            }
            let back = Formula::parse(&f.to_latex()).unwrap();
            if back != f {
                f = back;
            }
            caret = clamp(&f, &caret);
            prop_assert!(cell(&f, &caret).is_some_and(|l| caret.offset <= l.len()), "{caret:?} in {f:?}");
        }
    }
}
