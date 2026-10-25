//! `equation` sets an area's content to an equation or changes one, its formula read raw between
//! `$` signs; `tree` prints an equation with its options and its number.

mod common;

use common::*;
use rootpoint_model::{Content, EquationContent, EquationSize, Node, NodeId};
use rootpoint_protocol::{Command, EquationOption, Reply, Session, parse_command};

fn equation_of(s: &Session, area: u64) -> EquationContent {
    match s.document().node(NodeId(area)) {
        Some(Node::Area(a)) => match &a.content {
            Content::Equation(e) => e.clone(),
            other => panic!("n{area} holds {other:?}"),
        },
        other => panic!("n{area} is {other:?}"),
    }
}

/// A session whose one slide has two areas, n2 and n3.
fn two_areas() -> (Session, MemoryFiles) {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\nsplit n2 beside 0.5");
    (s, files)
}

/// The formula between `$` signs is read with no escape, so `\frac` is written once, and the
/// command prints back as typed.
#[test]
fn the_literal_is_raw() {
    let line = r"equation n7 $\frac{a}{b}$";
    let command = parse_command(line).unwrap();
    let Command::Equation { formula, .. } = &command else {
        panic!("{command:?}");
    };
    assert_eq!(formula.to_latex(), r"\frac{a}{b}");
    assert_eq!(command.to_string(), line);
}

/// A `$` of the formula is written `\$`, and `\\` before the closing `$` is a row break, not an
/// escape of it.
#[test]
fn a_dollar_is_escaped_and_a_row_break_ends_the_literal() {
    let command = parse_command(r"equation n7 $a \$ \\$ numbered=no").unwrap();
    let Command::Equation {
        formula, options, ..
    } = &command
    else {
        panic!("{command:?}");
    };
    assert_eq!(formula.rows.len(), 2);
    assert_eq!(options, &vec![EquationOption::Numbered(false)]);
    assert_eq!(parse_command(&command.to_string()), Ok(command.clone()));
    assert!(parse_command("equation n7 $x").is_err());
    assert!(parse_command(r"equation n7 $x^a^b$").is_err());
    assert!(parse_command("equation n7 $x$ size=0").is_err());
}

#[test]
fn every_option_prints_back() {
    for line in [
        "equation n7 $x$",
        "equation n7 $x$ numbered=yes",
        "equation n7 $x$ numbered=no size=fit",
        "equation n7 $x$ size=32.5 colour=#ff0000",
        "equation n7 $x$ colour=#00ff0080",
    ] {
        let command = parse_command(line).unwrap();
        assert_eq!(command.to_string(), line);
        assert_eq!(parse_command(&command.to_string()), Ok(command));
    }
}

/// A new equation is numbered and fitted; a change keeps the options it does not give.
#[test]
fn a_change_keeps_the_options_it_does_not_give() {
    let (mut s, mut files) = two_areas();
    ok(&mut s, &mut files, r"equation n3 $\alpha$");
    let e = equation_of(&s, 3);
    assert!(e.numbered);
    assert_eq!(e.size, EquationSize::Fit);
    ok(&mut s, &mut files, "equation n3 $x$ size=40 numbered=no");
    ok(&mut s, &mut files, r"equation n3 $\beta$");
    let e = equation_of(&s, 3);
    assert_eq!(e.formula.to_latex(), r"\beta");
    assert!(!e.numbered);
    assert_eq!(e.size, EquationSize::AtMost(40.0));
}

/// `tree` prints an equation as `equation` takes it, with all its options, then its number.
#[test]
fn tree_prints_an_equation_with_its_number() {
    let (mut s, mut files) = two_areas();
    ok(&mut s, &mut files, r"equation n3 $\frac{a}{b}$");
    let tree = ok(&mut s, &mut files, "tree");
    assert!(
        tree.contains(r"n3 equation $\frac{a}{b}$ numbered=yes size=fit colour=#000000 (1)"),
        "{tree}"
    );
}

/// The numbered equations of the document, by area, in the order of their numbers.
fn numbered(s: &Session) -> Vec<NodeId> {
    let numbers = s.document().equation_numbers();
    let mut areas: Vec<(u32, NodeId)> = numbers.into_iter().map(|(a, n)| (n, a)).collect();
    areas.sort();
    areas.into_iter().map(|(_, a)| a).collect()
}

/// Three slides: an equation on the first and on the last; on the second, a split beside of two
/// splits stacked, an equation in each of its four areas, A and B on the left, C and D on the
/// right, B unnumbered. The numbers follow the slides, then the depth-first order of each tree,
/// whatever the ids and the places: a move of a slide, and a swap, which exchanges the contents of
/// two areas, change them; a drag of the divider between C and D does not, though it moves C's
/// middle above A's.
#[test]
fn numbers_follow_the_slides_and_the_trees() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\nadd-slide\nadd-slide");
    let roots: Vec<NodeId> = s.document().slides.iter().map(|sl| sl.root.id()).collect();
    let slides: Vec<String> = s
        .document()
        .slides
        .iter()
        .map(|sl| sl.id.to_string())
        .collect();
    let [first, middle, last] = roots[..] else {
        panic!("three slides");
    };
    ok(&mut s, &mut files, &format!("split {middle} beside 0.5"));
    let right = s.document().slides[1].root.areas()[1].id;
    ok(&mut s, &mut files, &format!("split {middle} stacked 0.5"));
    ok(&mut s, &mut files, &format!("split {right} stacked 0.5"));
    let quad: Vec<NodeId> = s.document().slides[1]
        .root
        .areas()
        .iter()
        .map(|a| a.id)
        .collect();
    let [a, b, c, d] = quad[..] else {
        panic!("four areas");
    };
    for area in [first, a, b, c, d, last] {
        ok(&mut s, &mut files, &format!("equation {area} $x$"));
    }
    ok(&mut s, &mut files, &format!("equation {b} $y$ numbered=no"));
    assert_eq!(numbered(&s), [first, a, c, d, last]);
    let tree = ok(&mut s, &mut files, &format!("tree {}", slides[1]));
    assert!(tree.contains(&format!(
        "{c} equation $x$ numbered=yes size=fit colour=#000000 (3)"
    )));
    // The last slide moved first.
    ok(&mut s, &mut files, &format!("move-slide {} 0", slides[2]));
    assert_eq!(numbered(&s), [last, first, a, c, d]);
    // The divider of the right split moved up, so C's middle is above A's.
    let Node::Split(top) = &s.document().slides[2].root else {
        panic!("the middle slide, now third, is split");
    };
    let right_split = top.children[1].id();
    ok(&mut s, &mut files, &format!("divider {right_split} 0 0.1"));
    assert_eq!(numbered(&s), [last, first, a, c, d]);
    // The contents of A and B swapped.
    ok(&mut s, &mut files, &format!("swap {a} {b}"));
    assert_eq!(numbered(&s), [last, first, b, c, d]);
}

/// An `equation` built through the API with what the grammar refuses, a size not above 0 or a
/// formula whose LaTeX does not read back to it, is refused, so that every command the session
/// runs prints as a line that parses.
#[test]
fn a_command_the_grammar_refuses_is_refused_through_the_api() {
    let (mut s, mut files) = two_areas();
    let before = s.document().clone();
    let x = rootpoint_math::Formula::parse("x").unwrap();
    for size in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let command = Command::Equation {
            area: NodeId(3),
            formula: x.clone(),
            options: vec![EquationOption::Size(EquationSize::AtMost(size))],
        };
        assert!(parse_command(&command.to_string()).is_err());
        assert!(
            matches!(s.run(command, &mut files), Reply::Err(_)),
            "{size}"
        );
    }
    let dollar = rootpoint_math::Formula {
        rows: vec![vec![vec![rootpoint_math::Atom::Text("$".to_owned())]]],
    };
    let command = Command::Equation {
        area: NodeId(3),
        formula: dollar,
        options: Vec::new(),
    };
    assert!(parse_command(&command.to_string()).is_err());
    assert!(matches!(s.run(command, &mut files), Reply::Err(_)));
    assert_eq!(s.document(), &before);
}
