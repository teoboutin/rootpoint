//! The grammar prints every command back as it parses it; each editing command applies the
//! operation it names; a refused command changes nothing.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Command, Reply, Session, StyleCommand, StyleOption, parse_command};

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
        Command::Render {
            slide: SlideId(1),
            path: "out.png".into(),
            size: None,
        },
        Command::Render {
            slide: SlideId(1),
            path: "slides/one.png".into(),
            size: Some((384, 216)),
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
            axis: Axis::LeftToRight,
            at: 0.5,
            side: Side::After,
        },
        Command::Split {
            node: NodeId(2),
            axis: Axis::TopToBottom,
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
        Command::Sample,
        Command::Undo,
        Command::Redo,
        Command::Begin,
        Command::End,
        Command::Cancel,
        Command::Control,
        Command::Release,
        Command::Select { node: NodeId(2) },
        Command::SelectSlide { slide: SlideId(3) },
        Command::Deselect,
        Command::Preview { on: true },
        Command::Preview { on: false },
        Command::Present { slide: None },
        Command::Present {
            slide: Some(SlideId(4)),
        },
        Command::Next,
        Command::Prev,
        Command::Stop,
        Command::View,
        Command::Text {
            area: NodeId(2),
            text: "x".into(),
            style: vec![
                StyleOption::Named("Style \"1\" \\ b".into()),
                StyleOption::Custom,
                StyleOption::Named("Title".into()),
            ],
        },
        Command::Styles,
        Command::Style(StyleCommand::Add {
            name: "Pull quote".into(),
            options: vec![StyleOption::Size(30.0), StyleOption::Italic(true)],
        }),
        Command::Style(StyleCommand::Set {
            name: "Body".into(),
            options: vec![StyleOption::Font(None)],
        }),
        Command::Style(StyleCommand::Rename {
            from: "Body".into(),
            to: "Body text".into(),
        }),
        Command::Style(StyleCommand::Remove {
            name: "Body".into(),
            into: None,
        }),
        Command::Style(StyleCommand::Remove {
            name: "Body".into(),
            into: Some(Replacement::Custom),
        }),
        Command::Style(StyleCommand::Remove {
            name: "Body".into(),
            into: Some(Replacement::Named("Title \"2\"".into())),
        }),
        // A style named "custom" stays apart from the copy `into=custom` asks for.
        Command::Style(StyleCommand::Remove {
            name: "Body".into(),
            into: Some(Replacement::Named("custom".into())),
        }),
        Command::Text {
            area: NodeId(2),
            text: "x".into(),
            style: vec![StyleOption::Named("custom".into())],
        },
    ];
    for command in commands {
        let line = command.to_string();
        assert_eq!(parse_command(&line), Ok(command.clone()), "`{line}`");
    }
}

#[test]
fn every_scripted_line_prints_back_as_a_line_that_parses_to_it() {
    // The test above starts from commands; this one starts from lines written by hand, with the
    // spacing and quoting a script uses.
    let mut parsed = 0;
    for name in [
        "golden.txt",
        "groups.txt",
        "ids-build.txt",
        "ids-readdress.txt",
        "view.txt",
    ] {
        for line in script(name).lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Ok(command) = parse_command(line) else {
                continue;
            };
            let printed = command.to_string();
            assert_eq!(
                parse_command(&printed),
                Ok(command),
                "`{line}` printed `{printed}`"
            );
            parsed += 1;
        }
    }
    assert!(parsed > 0);
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
                axis: Axis::TopToBottom,
                at: 0.3,
                side: Side::Before,
            },
        ),
        (
            "split n4 beside 0.5",
            Op::Split {
                node: NodeId(4),
                axis: Axis::LeftToRight,
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
                    style: StyleChoice::Custom(style),
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
                content: Content::Text(TextContent {
                    text: String::new(),
                    style: StyleChoice::Named(BODY.to_owned()),
                }),
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
    // A split of the other axis wraps an area that is not the root; `before` puts the new area
    // first.
    assert_eq!(
        ok(&mut s, &mut files, "split n5 stacked 0.5 before"),
        "new=n7 split=n8"
    );
    assert_eq!(
        ok(&mut s, &mut files, "tree"),
        "s1 n4:beside(0.5,0.75)[n2 text \"\" style=\"Body\" | n3 text \"\" style=\"Body\" | n8:stacked(0.5)[n7 text \"\" style=\"Body\" | n5 text \"\" style=\"Body\"]]"
    );
    assert_eq!(ok(&mut s, &mut files, "asset cat.png"), "a9");
    assert_eq!(ok(&mut s, &mut files, "duplicate-slide s1"), "s16 root=n10");
}

/// `clear` empties an area's text: a text keeps its style, and an image area becomes a text in the
/// style a new area takes.
#[test]
fn clear_keeps_a_text_s_style_and_gives_an_image_area_the_default_one() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nasset cat.png\ntext n2 \"Hi\" size=120\nimage n3 a5 fit",
    );
    ok(&mut s, &mut files, "clear n2");
    ok(&mut s, &mut files, "clear n3");
    let content = |id| match s.document().node(NodeId(id)) {
        Some(Node::Area(a)) => a.content.clone(),
        other => panic!("n{id} is {other:?}"),
    };
    let text = |style| {
        Content::Text(TextContent {
            text: String::new(),
            style,
        })
    };
    // n2 named Body, which `size=120` made custom with Body's other values; n3, an image, takes the
    // style of a new area.
    let body = s.document().styles[BODY].clone();
    assert_eq!(
        content(2),
        text(StyleChoice::Custom(TextStyle {
            size: 120.0,
            ..body
        }))
    );
    assert_eq!(content(3), text(StyleChoice::Named(BODY.to_owned())));
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
    // `load` and `new` clear both stacks.
    run(&mut s, &mut files, "save deck.ron\nundo\nload deck.ron");
    assert!(matches!(s.execute("undo", &mut files), Some(Reply::Err(_))));
    assert!(matches!(s.execute("redo", &mut files), Some(Reply::Err(_))));
    // The model refuses an operation whose ids its document has not allocated, so the new
    // document is given as many ids as the old one first: a kept `remove-slide` would then undo.
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nremove-slide s1\nnew\nadd-slide\nundo",
    );
    assert!(matches!(s.execute("undo", &mut files), Some(Reply::Err(_))));
}

#[test]
fn a_refused_command_changes_nothing() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.5\nasset cat.png\nimage n3 a5 fit\nselect n3\npresent s1",
    );
    // Every write fails, so that `save` and `render` are refused.
    files.read_only = true;
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
        "add-slide 5",
        "duplicate-slide s9",
        "clear n9",
        "select n9",
        "select s9",
        "select a3",
        "present s9",
        "next",
        "prev",
        "preview",
        "preview maybe",
        "preview on now",
        "render s9 out.png",
        "render s1 out.png",
        "save deck.ron",
        // A quoted word is a text or a path, never an id, an option or a verb.
        "text n2 \"x\" \"bold\"",
        "select \"n2\"",
        "\"view\"",
        // Ids and indices are read only as the printer writes them.
        "split n+2 beside 0.5",
        "clear n02",
        "add-slide +0",
        // Words left over.
        "new now",
        "deselect n3",
        "view all",
        "asset",
        "sample s1",
        "begin now",
        // The style commands: a style of the table takes values, never a choice of style.
        "style add \"X\" custom",
        "style add \"X\" style=\"Title\"",
        "style add X",
        "style frob \"X\"",
        "style remove \"Body\" into=",
        "styles all",
        "text n2 \"x\" style=",
        "style set \"Nope\" bold",
        "style add \"\"",
        "style add \"Body\"",
        "style rename \"Nope\" \"X\"",
        "style remove \"Nope\"",
    ];
    assert_refused(&mut s, &mut files, &refused);
    // With no history and presenter mode off.
    let mut fresh = Session::new();
    assert_refused(
        &mut fresh,
        &mut files,
        &["undo", "redo", "stop", "next", "prev", "end", "cancel"],
    );
}

fn assert_refused(s: &mut Session, files: &mut MemoryFiles, refused: &[&str]) {
    for &line in refused {
        let doc = s.document().clone();
        let view = s.view().clone();
        let reply = s.execute(line, files);
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
        "s1 n4:beside(0.5)[n2 text \"Hello\" style=\"Body\" | n3 text \"\" style=\"Body\"]"
    );
}

#[test]
fn every_reply_is_one_line() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    // Each error repeats a word or a path that holds a new line.
    for line in ["add-slide \"a\\nb\"", "\"x\\ny\"", "load \"a\\nb\""] {
        let reply = s.execute(line, &mut files).unwrap();
        assert!(matches!(reply, Reply::Err(_)), "`{line}` gave {reply:?}");
        assert!(!reply.to_string().contains('\n'), "`{line}` gave {reply}");
    }
}

#[test]
fn a_command_built_through_the_api_keeps_the_grammars_sizes() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "add-slide");
    for size in [f64::NAN, f64::INFINITY, 0.0, -5.0] {
        let reply = s.run(
            Command::Text {
                area: NodeId(2),
                text: "x".into(),
                style: vec![StyleOption::Size(size)],
            },
            &mut files,
        );
        assert!(matches!(reply, Reply::Err(_)), "size {size} gave {reply:?}");
    }
}

#[test]
fn one_gesture_is_one_undo_step() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    let replies: Vec<String> = run(&mut s, &mut files, &script("groups.txt"))
        .iter()
        .map(|r| format!("{r}\n"))
        .collect();
    assert_eq!(replies.concat(), script("groups.replies"));
}

#[test]
fn a_cancelled_group_restores_the_document_and_both_stacks() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsplit n2 beside 0.4\ntext n2 \"A\"\nundo",
    );
    let doc = s.document().clone();
    run(
        &mut s,
        &mut files,
        "begin\nsplit n3 stacked 0.5\ndivider n4 0 0.7\nremove n3\ncancel",
    );
    assert!(s.document().same_content(&doc));
    // The redo stack is back: the text the undo above reverted is typed again.
    ok(&mut s, &mut files, "redo");
    assert_eq!(
        ok(&mut s, &mut files, "tree s1"),
        "s1 n4:beside(0.4)[n2 text \"A\" style=\"Body\" | n3 text \"\" style=\"Body\"]"
    );
    // And the undo stack: two undos reach the empty slide, a third removes the slide, and a
    // fourth has nothing, so the cancelled group left no step.
    run(&mut s, &mut files, "undo\nundo");
    assert_eq!(
        ok(&mut s, &mut files, "tree s1"),
        "s1 n2 text \"\" style=\"Body\""
    );
    ok(&mut s, &mut files, "undo");
    assert!(s.document().slides.is_empty());
    assert!(matches!(s.execute("undo", &mut files), Some(Reply::Err(_))));
}

#[test]
fn a_load_inside_a_group_closes_it_and_leaves_no_history() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "add-slide\nsave d.ron\nbegin\ntext n2 \"x\"",
    );
    ok(&mut s, &mut files, "load d.ron");
    assert_eq!(
        ok(&mut s, &mut files, "tree s1"),
        "s1 n2 text \"\" style=\"Body\""
    );
    // The group was closed by the load: `end` finds none open, and nothing is left to undo.
    assert!(matches!(s.execute("end", &mut files), Some(Reply::Err(_))));
    assert!(matches!(s.execute("undo", &mut files), Some(Reply::Err(_))));
}

#[test]
fn the_sample_opens_with_its_images_and_no_history() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(&mut s, &mut files, "add-slide\nbegin\ntext n2 \"x\"");
    ok(&mut s, &mut files, "sample");
    let doc = s.document().clone();
    assert!(doc.slides.len() >= 3);
    assert!(
        doc.assets
            .values()
            .all(|a| matches!(a.source, AssetSource::Embedded(_)))
    );
    assert!(matches!(s.execute("undo", &mut files), Some(Reply::Err(_))));
    assert!(matches!(s.execute("end", &mut files), Some(Reply::Err(_))));
    for slide in doc.slides {
        assert_eq!(
            ok(
                &mut s,
                &mut files,
                &format!("render {} out.png 192 108", slide.id)
            ),
            "fallback=- missing=- glyphs=- styles=-"
        );
    }
}
