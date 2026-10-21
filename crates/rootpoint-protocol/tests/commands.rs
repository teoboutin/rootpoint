//! The grammar prints every command back as it parses it; each editing command applies the
//! operation it names; a refused command changes nothing.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Command, Reply, Session, StyleOption, parse_command};

#[test]
fn every_command_prints_back_as_it_parses() {
    let commands = vec![
        Command::New,
        Command::Save {
            path: "a talk/deck.ron".into(),
        },
        Command::Load {
            path: "deck.ron".into(),
        },
        Command::Tree { slide: None },
        Command::Tree {
            slide: Some(SlideId(3)),
        },
        Command::AddSlide { at: None },
        Command::AddSlide { at: Some(2) },
        Command::RemoveSlide { slide: SlideId(1) },
        Command::MoveSlide {
            slide: SlideId(1),
            to: 0,
        },
        Command::DuplicateSlide { slide: SlideId(1) },
        Command::Split {
            node: NodeId(2),
            axis: Axis::SideBySide,
            at: 0.5,
            side: Side::After,
        },
        Command::Split {
            node: NodeId(2),
            axis: Axis::Stacked,
            at: 0.25,
            side: Side::Before,
        },
        Command::Remove { node: NodeId(4) },
        Command::Divider {
            split: NodeId(3),
            index: 0,
            pos: 0.4,
        },
        Command::Swap {
            a: NodeId(2),
            b: NodeId(4),
        },
        Command::Text {
            area: NodeId(2),
            text: "A \"quoted\" back\\slash\nand a new line".into(),
            style: vec![
                StyleOption::Size(64.0),
                StyleOption::Bold(true),
                StyleOption::Italic(false),
                StyleOption::Colour(Rgba {
                    r: 1,
                    g: 2,
                    b: 255,
                    a: 128,
                }),
                StyleOption::Align(HAlign::Center),
                StyleOption::VAlign(VAlign::Bottom),
                StyleOption::Font(Some(AssetId(7))),
                StyleOption::Font(None),
            ],
        },
        Command::Text {
            area: NodeId(2),
            text: "bold".into(),
            style: vec![],
        },
        Command::Image {
            area: NodeId(4),
            asset: AssetId(5),
            fit: ImageFit::Fill,
        },
        Command::Clear { area: NodeId(4) },
        Command::Asset {
            path: "images/cat.png".into(),
        },
        Command::RemoveAsset { asset: AssetId(5) },
        Command::Undo,
        Command::Redo,
    ];
    for command in commands {
        let line = command.to_string();
        assert_eq!(parse_command(&line), Ok(command.clone()), "`{line}`");
    }
}

/// Each editing command applies the operation the mapping table gives: the document after the
/// command equals the document after `apply` of that operation.
#[test]
fn each_editing_command_applies_its_operation() {
    let mut files = MemoryFiles::default();
    let mut session = Session::new();
    let setup = "add-slide\nsplit n2 beside 0.5\nasset cat.png\ntext n2 \"Hi\" size=64\nimage n3 a5 fit\nadd-slide";
    for reply in run(&mut session, &mut files, setup) {
        assert!(matches!(reply, Reply::Ok(_)), "{reply:?}");
    }
    let base = session.document().clone();
    let style = TextStyle {
        size: 64.0,
        bold: true,
        ..TextStyle::default()
    };
    let rows: Vec<(&str, Op)> = vec![
        ("add-slide 0", Op::AddSlide { at: 0 }),
        ("remove-slide s6", Op::RemoveSlide { slide: SlideId(6) }),
        (
            "move-slide s1 1",
            Op::MoveSlide {
                slide: SlideId(1),
                to: 1,
            },
        ),
        (
            "duplicate-slide s1",
            Op::DuplicateSlide { slide: SlideId(1) },
        ),
        (
            "split n2 stacked 0.3 before",
            Op::Split {
                node: NodeId(2),
                axis: Axis::Stacked,
                at: 0.3,
                side: Side::Before,
            },
        ),
        (
            "split n4 beside 0.5",
            Op::Split {
                node: NodeId(4),
                axis: Axis::SideBySide,
                at: 0.5,
                side: Side::After,
            },
        ),
        ("remove n3", Op::Remove { node: NodeId(3) }),
        (
            "divider n4 0 0.4",
            Op::MoveDivider {
                split: NodeId(4),
                index: 0,
                pos: 0.4,
            },
        ),
        (
            "swap n2 n3",
            Op::Swap {
                a: NodeId(2),
                b: NodeId(3),
            },
        ),
        (
            "text n2 \"Hi\" bold",
            Op::SetContent {
                area: NodeId(2),
                content: Content::Text(TextContent {
                    text: "Hi".into(),
                    style,
                }),
            },
        ),
        (
            "image n2 a5 fill",
            Op::SetContent {
                area: NodeId(2),
                content: Content::Image(ImageContent {
                    asset: AssetId(5),
                    fit: ImageFit::Fill,
                }),
            },
        ),
        (
            "clear n3",
            Op::SetContent {
                area: NodeId(3),
                content: Content::Empty,
            },
        ),
        (
            "asset dog.png",
            Op::AddAsset {
                asset: Asset {
                    source: AssetSource::Path("dog.png".into()),
                },
            },
        ),
    ];
    for (line, op) in rows {
        let mut s = Session::new();
        let mut f = MemoryFiles::default();
        run(&mut s, &mut f, setup);
        assert!(
            matches!(s.execute(line, &mut f), Some(Reply::Ok(_))),
            "`{line}`"
        );
        let mut expected = base.clone();
        apply(&mut expected, op).unwrap();
        assert!(s.document().same_content(&expected), "`{line}`");
    }
    // remove-asset needs an asset no area uses.
    let mut s = Session::new();
    let mut f = MemoryFiles::default();
    run(&mut s, &mut f, "asset dog.png");
    let mut expected = s.document().clone();
    apply(&mut expected, Op::RemoveAsset { id: AssetId(1) }).unwrap();
    assert!(matches!(
        s.execute("remove-asset a1", &mut f),
        Some(Reply::Ok(_))
    ));
    assert!(s.document().same_content(&expected));
}

#[test]
fn the_replies_name_the_ids_a_command_created() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    assert_eq!(ok(&mut s, &mut files, "add-slide"), "s1 root=n2");
    assert_eq!(
        ok(&mut s, &mut files, "split n2 beside 0.5"),
        "new=n3 split=n4"
    );
    assert_eq!(ok(&mut s, &mut files, "split n3 beside 0.5"), "new=n5");
    assert_eq!(ok(&mut s, &mut files, "asset cat.png"), "a7");
    assert_eq!(ok(&mut s, &mut files, "duplicate-slide s1"), "s12 root=n8");
}

#[test]
fn undo_and_redo_walk_the_history() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\ntext n2 \"A\"",
    );
    let after = s.document().clone();
    ok(&mut s, &mut files, "undo");
    ok(&mut s, &mut files, "undo");
    ok(&mut s, &mut files, "redo");
    ok(&mut s, &mut files, "redo");
    assert!(s.document().same_content(&after));
    assert!(matches!(s.execute("redo", &mut files), Some(Reply::Err(_))));
    ok(&mut s, &mut files, "undo");
    ok(&mut s, &mut files, "clear n3");
    assert!(
        matches!(s.execute("redo", &mut files), Some(Reply::Err(_))),
        "a new edit clears redo"
    );
}

#[test]
fn a_refused_command_changes_nothing() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nasset cat.png\nimage n3 a5 fit",
    );
    let refused = [
        "nonsense",
        "split n2 diagonal 0.5",
        "split n9 beside 0.5",
        "split n2 beside 1.5",
        "remove n4",
        "remove-slide s9",
        "move-slide s1 4",
        "divider n4 3 0.5",
        "divider n4 0 2",
        "swap n2 n4",
        "text n2 bold",
        "text n2 \"x\" size=-1",
        "image n2 a9 fit",
        "remove-asset a5",
        "tree s9",
        "load missing.ron",
        "text n2 \"unclosed",
    ];
    for line in refused {
        let doc = s.document().clone();
        let view = s.view().clone();
        let reply = s.execute(line, &mut files);
        assert!(
            matches!(reply, Some(Reply::Err(_))),
            "`{line}` gave {reply:?}"
        );
        assert_eq!(s.document(), &doc, "`{line}` changed the document");
        assert_eq!(s.view(), &view, "`{line}` changed the view");
    }
}

#[test]
fn blank_lines_and_comments_give_no_reply() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    assert_eq!(
        run(&mut s, &mut files, "\n   \n# a comment\nadd-slide"),
        vec![Reply::Ok("s1 root=n2".into())]
    );
}

#[test]
fn save_load_and_tree() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\ntext n2 \"Hello\"\nsave talk/deck.ron",
    );
    assert!(files.files.contains_key("talk/deck.ron"));
    let mut t = Session::new();
    ok(&mut t, &mut files, "load talk/deck.ron");
    assert_eq!(t.document(), s.document());
    assert_eq!(
        ok(&mut t, &mut files, "tree s1"),
        "s1 n4:beside(0.5)[n2 text \"Hello\" | n3 empty]"
    );
}
