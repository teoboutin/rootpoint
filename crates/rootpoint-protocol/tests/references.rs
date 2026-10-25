//! Formulas and references in a text through the commands: the tree's `$…$` and `{ref n<id>}`
//! items, `insert`, `math` and `retarget`, and the refusals that keep a command from leaving a
//! reference to an area that holds no equation.

mod common;

use common::*;
use rootpoint_protocol::{Reply, Session, parse_command};

/// One slide, split stacked then beside: n2, a text, above n3 and n5, each an equation, so
/// numbered (1) and (2).
fn session() -> (Session, MemoryFiles) {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 stacked 0.5\nsplit n3 beside 0.5\nequation n3 $a$\nequation n5 $b$",
    );
    (s, files)
}

fn refused(s: &mut Session, files: &mut MemoryFiles, line: &str) -> String {
    match s.execute(line, files) {
        Some(Reply::Err(reason)) => reason,
        other => panic!("`{line}` gave {other:?}"),
    }
}

/// `text` takes a tree holding formulas and references, `tree` prints it back as `text` takes it,
/// and a reference draws its equation's number.
#[test]
fn a_tree_of_formulas_and_references_prints_back() {
    let (mut s, mut files) = session();
    let tree = r#"["Let ", $x^2$, " as in ", {ref n5}, {bold "and ", {ref n3}}]"#;
    ok(&mut s, &mut files, &format!("text n2 {tree}"));
    let printed = ok(&mut s, &mut files, "tree");
    assert!(
        printed.contains(r#"n2 text ["Let ", $x^2$, " as in ", {ref n5}, {t"#),
        "{printed}"
    );
    assert!(printed.contains(r#" bold "and ", {ref n3}}]"#), "{printed}");
    let command = parse_command(&format!("text n2 {tree}")).unwrap();
    assert_eq!(parse_command(&command.to_string()), Ok(command));
}

/// `insert` puts a formula or a reference at an offset, `math` replaces a formula, and each
/// prints back as typed.
#[test]
fn insert_and_math_edit_the_items_of_a_text() {
    let (mut s, mut files) = session();
    ok(&mut s, &mut files, r#"text n2 "ab""#);
    for line in [
        r"insert n2 1 $\frac{1}{2}$",
        "insert n2 3 {ref n3}",
        r"math n2 1 $\sqrt{2}$",
    ] {
        ok(&mut s, &mut files, line);
        let command = parse_command(line).unwrap();
        assert_eq!(command.to_string(), line);
    }
    let printed = ok(&mut s, &mut files, "tree");
    assert!(
        printed.contains(r#"n2 text ["a", $\sqrt{2}$, "b", {ref n3}]"#),
        "{printed}"
    );
    // `math` at an offset that holds no formula is refused.
    refused(&mut s, &mut files, r"math n2 0 $y$");
}

/// `retarget` moves every reference to one equation onto another and replies the count; its undo
/// moves back only those.
#[test]
fn retarget_replies_the_count_and_undoes_what_it_moved() {
    let (mut s, mut files) = session();
    ok(
        &mut s,
        &mut files,
        r#"text n2 [{ref n3}, {ref n5}, {ref n3}]"#,
    );
    assert_eq!(ok(&mut s, &mut files, "retarget n3 n5"), "2");
    assert!(ok(&mut s, &mut files, "tree").contains("[{ref n5}, {ref n5}, {ref n5}]"));
    ok(&mut s, &mut files, "undo");
    assert!(ok(&mut s, &mut files, "tree").contains("[{ref n3}, {ref n5}, {ref n3}]"));
    assert_eq!(
        refused(&mut s, &mut files, "retarget n3 n2"),
        "n2 holds no equation"
    );
}

/// A command that would leave a text referencing an area that holds no equation is refused, naming
/// each referencing text and its count: the removal of the equation, of a split or a slide holding
/// it, and its change into another kind; and a reference given to an area that holds no equation.
/// A reference inside the slide removed goes with it.
#[test]
fn a_removal_is_refused_while_referenced() {
    let (mut s, mut files) = session();
    ok(
        &mut s,
        &mut files,
        r#"text n2 [{ref n3}, " and ", {ref n3}]"#,
    );
    let reason = "n3 is referenced by n2 (2): `retarget` its references first";
    let before = s.document().clone();
    let split = match &before.slides[0].root {
        rootpoint_model::Node::Split(top) => top.children[1].id(),
        other => panic!("{other:?}"),
    };
    for line in [
        "remove n3".to_owned(),
        format!("remove {split}"),
        r#"text n3 "x""#.to_owned(),
        "clear n3".to_owned(),
    ] {
        assert_eq!(refused(&mut s, &mut files, &line), reason, "{line}");
        assert_eq!(s.document(), &before, "{line}");
    }
    assert_eq!(
        refused(&mut s, &mut files, "insert n2 0 {ref n2}"),
        "n2 holds no equation"
    );
    assert_eq!(
        refused(&mut s, &mut files, r#"text n2 [{ref n2}]"#),
        "n2 holds no equation"
    );
    // A text on another slide referencing n3 makes the slide's removal refused; the slide's own
    // reference would go with it.
    ok(&mut s, &mut files, "add-slide");
    let other = s.document().slides[1].root.id();
    ok(&mut s, &mut files, &format!("text {other} [{{ref n3}}]"));
    assert_eq!(
        refused(&mut s, &mut files, "remove-slide s1"),
        format!("n3 is referenced by {other} (1): `retarget` its references first")
    );
    ok(&mut s, &mut files, &format!("text {other} \"\""));
    ok(&mut s, &mut files, "remove-slide s1");
}

/// A file holding a reference to an area that holds no equation loads, and `render` names it after
/// `refs=`.
#[test]
fn a_dangling_reference_of_a_file_is_named_by_the_render() {
    let (mut s, mut files) = session();
    ok(&mut s, &mut files, "text n2 [{ref n3}]");
    let text = rootpoint_model::save_ron(s.document()).replace("Ref(3)", "Ref(5)");
    let mut doc = rootpoint_model::load_ron(&text).unwrap();
    // n5 made a text, so the reference to it dangles.
    let content = rootpoint_model::Content::default();
    rootpoint_model::apply(
        &mut doc,
        rootpoint_model::Op::SetContent {
            area: rootpoint_model::NodeId(5),
            content,
        },
    )
    .unwrap();
    use rootpoint_protocol::Files;
    files
        .write(
            "deck/presentation.ron",
            rootpoint_model::save_ron(&doc).as_bytes(),
        )
        .unwrap();
    ok(&mut s, &mut files, "open deck");
    let reply = ok(&mut s, &mut files, "render s1 out.png 160 90");
    assert!(reply.ends_with("refs=n2:n5"), "{reply}");
}
