//! The editing of a formula at a caret: the moves visit every position once each way, the edits
//! keep the formula one its LaTeX reads back to, and every position has a caret box on the layout.

mod common;

use proptest::prelude::*;
use rootpoint_math::*;

fn parse(text: &str) -> Formula {
    Formula::parse(text).unwrap()
}

/// The positions Right visits from the start, in order.
fn by_right(f: &Formula) -> Vec<CellPath> {
    let mut out = vec![start()];
    while let Some(next) = right(f, out.last().unwrap()) {
        assert!(out.len() < 10_000, "Right does not reach the end");
        out.push(next);
    }
    out
}

/// The positions Left visits from the end, in order.
fn by_left(f: &Formula) -> Vec<CellPath> {
    let mut out = vec![end(f)];
    while let Some(previous) = left(f, out.last().unwrap()) {
        assert!(out.len() < 10_000, "Left does not reach the start");
        out.push(previous);
    }
    out
}

fn at(steps: &[(usize, usize)], offset: usize) -> CellPath {
    CellPath {
        steps: steps.to_vec(),
        offset,
        ..CellPath::default()
    }
}

/// Right and Left move as LyX's caret: into an atom at its first cell, Right, or its last, Left,
/// and out of it at that cell's edge, but across a matrix's columns and from a root's index to its
/// body; so a fraction's denominator is reached by Left and its numerator by Right, and a script by
/// Left alone. Each visits a position once, and neither leaves the row of the formula's grid it
/// starts in.
#[test]
fn right_and_left_move_as_lyx_s_caret() {
    let f = parse(r"\frac{a}{b}c");
    assert_eq!(
        by_right(&f),
        vec![
            at(&[], 0),
            at(&[(0, 0)], 0),
            at(&[(0, 0)], 1),
            at(&[], 1),
            at(&[], 2)
        ]
    );
    assert_eq!(
        by_left(&f),
        vec![
            at(&[], 2),
            at(&[], 1),
            at(&[(0, 1)], 1),
            at(&[(0, 1)], 0),
            at(&[], 0)
        ]
    );
    // A script's nucleus by Right; its last script by Left.
    let f = parse(r"x_i^2");
    assert_eq!(
        by_right(&f),
        vec![at(&[], 0), at(&[(0, 0)], 0), at(&[(0, 0)], 1), at(&[], 1)]
    );
    assert_eq!(
        by_left(&f),
        vec![at(&[], 1), at(&[(0, 2)], 1), at(&[(0, 2)], 0), at(&[], 0)]
    );
    // A root's index, then its body.
    let f = parse(r"\sqrt[n]{y}");
    assert_eq!(
        by_right(&f),
        vec![
            at(&[], 0),
            at(&[(0, 1)], 0),
            at(&[(0, 1)], 1),
            at(&[(0, 0)], 0),
            at(&[(0, 0)], 1),
            at(&[], 1)
        ]
    );
    // A matrix's columns, in the caret's row.
    let f = parse(r"\begin{matrix} a & b \\ c & d \end{matrix}");
    let right_path = by_right(&f);
    assert_eq!(right_path[1], at(&[(0, 0)], 0));
    assert_eq!(right_path[3], at(&[(0, 1)], 0));
    assert_eq!(right_path.last(), Some(&at(&[], 1)));
    // The grid: Right goes along a row's columns and stops at its end.
    let f = parse(r"a & b \\ c");
    assert_eq!(by_right(&f).len(), 4);
    assert_eq!(
        right(
            &f,
            &CellPath {
                column: 1,
                offset: 1,
                ..CellPath::default()
            }
        ),
        None
    );
    for text in [
        r"x",
        r"\frac{a}{b} + c",
        r"x_i^2 = \sqrt{y}",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
        r"\left( \frac{1}{2} \right) \hat{x}",
        r"",
    ] {
        let f = parse(text);
        for path in [by_right(&f), by_left(&f)] {
            let unique: std::collections::HashSet<_> = path.iter().collect();
            assert_eq!(unique.len(), path.len(), "{text}: a position visited twice");
            assert!(path.iter().all(|p| positions(&f).contains(p)), "{text}");
        }
    }
}

/// `positions` gives each position of a formula once, the start and the end among them.
#[test]
fn positions_gives_every_position_once() {
    let f = parse(r"\frac{a}{b_1} + \begin{matrix} x & \\ y & z \end{matrix} \\ q");
    let all = positions(&f);
    let unique: std::collections::HashSet<_> = all.iter().collect();
    assert_eq!(unique.len(), all.len());
    assert!(all.contains(&start()) && all.contains(&end(&f)));
    for p in &all {
        assert!(cell(&f, p).is_some_and(|l| p.offset <= l.len()), "{p:?}");
    }
    // A cell of n atoms has n + 1 positions: the grid's first cell 4; the numerator 2; the
    // denominator, one script, 2, and its nucleus and subscript 2 each; the matrix's cells 2, 1, 2
    // and 2; the grid's second row 2.
    assert_eq!(all.len(), 4 + 2 + 2 + 2 + 2 + 2 + 1 + 2 + 2 + 2);
}

/// Up and Down move between a fraction's numerator and denominator, a script and its nucleus, the
/// rows of a matrix and of the formula.
#[test]
fn up_and_down_move_between_the_cells_above_and_below() {
    let f = parse(r"\frac{ab}{c}");
    let in_num = at(&[(0, 0)], 1);
    let down = vertical_move(&f, &in_num, false).unwrap();
    assert_eq!(down.steps, vec![(0, 1)]);
    assert_eq!(down.offset, 1);
    assert_eq!(vertical_move(&f, &down, true).unwrap().steps, vec![(0, 0)]);
    assert_eq!(vertical_move(&f, &down, false), None);
    let f = parse(r"x_i^2");
    // At the nucleus's end; from its start, Up and Down go nowhere, as in LyX.
    let nucleus = by_right(&f)[2].clone();
    assert_eq!(nucleus.steps, vec![(0, 0)]);
    assert_eq!(vertical_move(&f, &by_right(&f)[1], true), None);
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
    let in_a = at(&[(0, 0)], 0);
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
        for p in positions(&f) {
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
            let same: Vec<_> = positions(&f)
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
            let all = positions(&f);
            if k == 0 {
                caret = all[pick % all.len()].clone();
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

/// A matrix whose rows have unequal lengths, which LaTeX reads and the parser takes: Up and Down
/// keep to the nearest column of a shorter row, and every edit of its rows and columns pads the
/// rows to the longest first, at every position, so none indexes past a row.
#[test]
fn a_matrix_with_rows_of_unequal_length_is_edited_whole() {
    let ragged = r"\begin{matrix} a & b \\ c \end{matrix}";
    let f = parse(ragged);
    let b = at(&[(0, 1)], 0);
    assert_eq!(
        vertical_move(&f, &b, false),
        Some(at(&[(0, 2)], 0)),
        "b, then c"
    );
    let d = parse(r"a \\ c & d");
    let in_d = CellPath {
        row: 1,
        column: 1,
        ..CellPath::default()
    };
    assert_eq!(
        vertical_move(&d, &in_d, true).map(|p| (p.row, p.column)),
        Some((0, 0))
    );
    for p in positions(&f) {
        let _ = vertical_move(&f, &p, true);
        let _ = vertical_move(&f, &p, false);
        for rows in [true, false] {
            for edit in [
                MatrixEdit::Append,
                MatrixEdit::Delete,
                MatrixEdit::Copy,
                MatrixEdit::Swap,
            ] {
                let mut g = f.clone();
                if let Some(q) = edit_matrix(&mut g, &p, rows, edit) {
                    assert!(
                        cell(&g, &q).is_some_and(|l| q.offset <= l.len()),
                        "{p:?} {edit:?}"
                    );
                    assert_eq!(Formula::parse(&g.to_latex()).unwrap(), g);
                }
            }
        }
        let mut g = f.clone();
        if let Some(q) = new_row(&mut g, &p) {
            assert!(cell(&g, &q).is_some());
        }
    }
    let mut g = f.clone();
    edit_matrix(&mut g, &b, false, MatrixEdit::Append).unwrap();
    assert_eq!(
        g.to_latex(),
        r"\begin{matrix} a & b & \\ c & & \end{matrix}"
    );
}
