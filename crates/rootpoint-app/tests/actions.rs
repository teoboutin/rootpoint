//! Every UI action goes through the session: each action of the app, called with no window, leaves
//! the document and the view that its commands, run on a second session, leave; and one `undo`
//! after it leaves them equal too, so the action groups its commands as they do, as
//! `design@rootpoint-app@actions-are-commands` says. The same tests run in a browser, for the
//! browser build, as `path@rootpoint-app@tests/web.rs` says.

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as test;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use eframe::egui::{Pos2, Rect, Vec2};
use rootpoint_app::App;
use rootpoint_app::hit::{Edge, Layout, Preview, Zone};
use rootpoint_app::selection::Move;
use rootpoint_model::{AssetId, Axis, Content, ImageFit, Node, NodeId, Side, SlideId};
use rootpoint_protocol::{
    Command, Files, Reply, Session, StyleCommand, StyleOption, parse_command,
};

/// Files held in memory, by path, shared so that the test reads what a save wrote.
#[derive(Clone, Default)]
struct MemoryFiles(Rc<RefCell<BTreeMap<String, Vec<u8>>>>);

impl Files for MemoryFiles {
    fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        self.0
            .borrow()
            .get(path)
            .cloned()
            .ok_or(format!("{path}: no such file"))
    }

    fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.0.borrow_mut().insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn exists(&mut self, path: &str) -> Result<bool, String> {
        Ok(self.0.borrow().contains_key(path))
    }
}

fn c(line: &str) -> Command {
    parse_command(line).unwrap_or_else(|e| panic!("{line}: {e}"))
}

/// The text of an area of the sample, as a command must repeat it.
fn text_of(session: &Session, area: NodeId) -> String {
    match session.document().node(area) {
        Some(Node::Area(a)) => match &a.content {
            Content::Text(t) => t.text.clone(),
            _ => panic!("{area} holds no text"),
        },
        _ => panic!("{area} is no area"),
    }
}

/// Run `setup` on an app and on a session, both on the sample; then the action on the app and
/// `expected` on the session, and compare; then one `undo` on each, and compare again.
fn check(
    name: &str,
    setup: &[&str],
    action: impl FnOnce(&mut App),
    expected: impl FnOnce(&Session) -> Vec<Command>,
) {
    let (files_a, mut files_b) = (MemoryFiles::default(), MemoryFiles::default());
    let mut app = App::new(Box::new(files_a.clone()));
    let mut session = Session::new();
    for line in ["sample"].iter().chain(setup) {
        assert!(matches!(app.run(c(line)), Reply::Ok(_)), "{name}: {line}");
        assert!(matches!(session.run(c(line), &mut files_b), Reply::Ok(_)));
    }
    action(&mut app);
    for command in expected(&session) {
        let line = command.to_string();
        let reply = session.run(command, &mut files_b);
        assert!(matches!(reply, Reply::Ok(_)), "{name}: {line}: {reply}");
    }
    assert_eq!(app.session().document(), session.document(), "{name}");
    assert_eq!(app.session().view(), session.view(), "{name}");
    assert_eq!(
        *files_a.0.borrow(),
        *files_b.0.borrow(),
        "{name}: the files"
    );
    let a = app.run(Command::Undo);
    let b = session.run(Command::Undo, &mut files_b);
    assert_eq!(a, b, "{name}: undo");
    assert_eq!(
        app.session().document(),
        session.document(),
        "{name}: after undo"
    );
    assert_eq!(app.session().view(), session.view(), "{name}: after undo");
}

#[test]
fn the_selection_actions_issue_their_commands() {
    check(
        "click an area",
        &[],
        |a| a.select_node(NodeId(3)),
        |_| vec![c("select n3")],
    );
    check(
        "click a slide",
        &["select n3"],
        |a| a.show_slide(SlideId(7)),
        |_| vec![c("select s7")],
    );
    check(
        "Alt+Left",
        &["select n3"],
        |a| a.move_selection(Move::Parent),
        |_| vec![c("select n6")],
    );
    check(
        "Alt+Down",
        &["select n3"],
        |a| a.move_selection(Move::Next),
        |_| vec![c("select n5")],
    );
    check(
        "Alt+Up",
        &["select n5"],
        |a| a.move_selection(Move::Previous),
        |_| vec![c("select n3")],
    );
    // Alt+Right goes to the child selected last in the split, n5, not to its first.
    check(
        "Alt+Right",
        &["select n5", "select n6"],
        |a| a.move_selection(Move::Child),
        |_| vec![c("select n5")],
    );
}

#[test]
fn the_slide_list_actions_issue_their_commands() {
    check("add slide", &["select s7"], App::add_slide, |_| {
        vec![c("add-slide 2")]
    });
    check("remove slide", &["select s7"], App::remove_slide, |_| {
        vec![c("remove-slide s7")]
    });
    check(
        "duplicate slide",
        &["select s7"],
        App::duplicate_slide,
        |_| vec![c("duplicate-slide s7")],
    );
    check(
        "drag a slide",
        &[],
        |a| a.move_slide(SlideId(1), 2),
        |_| vec![c("move-slide s1 2")],
    );
}

#[test]
fn the_split_actions_issue_their_commands() {
    for (edge, line) in [
        (Edge::Left, "split n3 beside 0.5 before"),
        (Edge::Right, "split n3 beside 0.5 after"),
        (Edge::Up, "split n3 stacked 0.5 before"),
        (Edge::Down, "split n3 stacked 0.5 after"),
    ] {
        check(
            line,
            &["select n3"],
            |a| a.split_middle(edge),
            |_| vec![c(line)],
        );
    }
    check(
        "Ctrl click",
        &["select n3"],
        |a| {
            a.split_at(Preview {
                node: NodeId(3),
                axis: Axis::LeftToRight,
                at: 0.25,
                side: Side::Before,
                from: Default::default(),
                to: Default::default(),
            })
        },
        |_| vec![c("split n3 beside 0.25 before")],
    );
    check("Delete", &["select n3"], App::remove_selection, |_| {
        vec![c("remove n3")]
    });
}

#[test]
fn the_divider_actions_issue_their_commands() {
    check(
        "drag a divider",
        &[],
        |a| {
            a.grab_divider(NodeId(6), 0);
            a.drag_divider(0.4);
            a.drag_divider(0.45);
            a.release_drag();
        },
        |_| {
            vec![
                c("begin"),
                c("divider n6 0 0.4"),
                c("divider n6 0 0.45"),
                c("end"),
            ]
        },
    );
    check(
        "Esc during a divider drag",
        &[],
        |a| {
            a.grab_divider(NodeId(6), 0);
            a.drag_divider(0.4);
            a.cancel_drag();
            a.release_drag();
        },
        |_| vec![c("begin"), c("divider n6 0 0.4"), c("cancel")],
    );
    check(
        "double-click a divider",
        &["divider n6 0 0.3"],
        |a| a.reset_divider(NodeId(6), 0),
        |_| vec![c("divider n6 0 0.5")],
    );
}

#[test]
fn the_grip_actions_issue_their_commands() {
    check(
        "drop on a swap zone",
        &["select n3"],
        |a| {
            a.grab_grip(NodeId(3));
            a.drop_grip(Some((NodeId(5), Zone::Swap)));
        },
        |_| vec![c("begin"), c("swap n3 n5"), c("end")],
    );
    check(
        "drop on a move zone",
        &["select n3"],
        |a| {
            a.grab_grip(NodeId(3));
            a.drop_grip(Some((NodeId(2), Zone::Move(Edge::Down))));
        },
        // The sample's next id is 20, so the split's new area is n20.
        |_| {
            vec![
                c("begin"),
                c("split n2 stacked 0.5 after"),
                c("swap n3 n20"),
                c("remove n3"),
                c("end"),
            ]
        },
    );
    check(
        "drop outside any area",
        &["select n3"],
        |a| {
            a.grab_grip(NodeId(3));
            a.drop_grip(None);
        },
        |_| vec![c("begin"), c("cancel")],
    );
}

#[test]
fn the_content_actions_issue_their_commands() {
    check(
        "set the content to text",
        &["select n3", "image n3 a19 fit"],
        App::set_text_content,
        |_| vec![c("text n3 \"\"")],
    );
    check(
        "type in place",
        &["select n3"],
        |a| {
            let doc = a.session().document();
            let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
            let layout = Layout::new(doc, SlideId(1), view, [960, 540]).unwrap();
            let p = layout.rect(NodeId(3)).unwrap().center();
            a.edit_at(NodeId(3), p, &layout);
            a.set_caret(0);
            a.type_str("A");
            a.type_str("b");
            a.delete_backward();
            a.stop_editing();
        },
        |s| {
            let t = text_of(s, NodeId(3));
            let text = |prefix: &str| Command::Text {
                area: NodeId(3),
                text: format!("{prefix}{t}"),
                style: Vec::new(),
            };
            vec![c("begin"), text("A"), text("Ab"), text("A"), c("end")]
        },
    );
    check(
        "cut, paste and an IME composition in place",
        &["select n3"],
        |a| {
            let doc = a.session().document();
            let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
            let layout = Layout::new(doc, SlideId(1), view, [960, 540]).unwrap();
            let p = layout.rect(NodeId(3)).unwrap().center();
            a.edit_at(NodeId(3), p, &layout);
            a.set_caret(0);
            a.move_caret(rootpoint_app::editing::CaretMove::Right, true);
            a.cut();
            a.type_str("xy");
            a.set_preedit("´");
            a.commit_ime("é");
            a.delete_forward();
            a.stop_editing();
        },
        |s| {
            let t = text_of(s, NodeId(3));
            let text = |s: String| Command::Text {
                area: NodeId(3),
                text: s,
                style: Vec::new(),
            };
            let rest = &t[1..];
            vec![
                c("begin"),
                text(rest.to_owned()),
                text(format!("xy{rest}")),
                text(format!("xy´{rest}")),
                text(format!("xyé{rest}")),
                text(format!("xyé{}", &rest[1..])),
                c("end"),
            ]
        },
    );
    // A style option repeats the area's whole text: `text n3 "<its text>" bold`.
    for word in ["bold", "size=80", "align=right"] {
        check(
            word,
            &["select n3"],
            |a| {
                let Command::Text { mut style, .. } = c(&format!("text n3 \"\" {word}")) else {
                    panic!("a text command");
                };
                a.set_style(style.remove(0));
            },
            |s| vec![c(&format!("text n3 {:?} {word}", text_of(s, NodeId(3))))],
        );
    }
    check(
        "set the content to an image",
        &["select n3"],
        |a| a.set_image(AssetId(19), ImageFit::Fill),
        |_| vec![c("image n3 a19 fill")],
    );
    check(
        "add an image asset",
        &[],
        |a| a.add_asset("cat.png".to_owned()),
        |_| vec![c("asset cat.png")],
    );
}

#[test]
fn the_style_actions_issue_their_commands() {
    check(
        "name a style",
        &["select n3"],
        |a| a.name_style("Title".to_owned()),
        |s| {
            vec![c(&format!(
                "text n3 {:?} style=\"Title\"",
                text_of(s, NodeId(3))
            ))]
        },
    );
    check(
        "make a style custom",
        &["select n3"],
        App::make_custom,
        |s| vec![c(&format!("text n3 {:?} custom", text_of(s, NodeId(3))))],
    );
    check("add a style", &[], App::add_style, |_| {
        vec![c("style add \"Style 1\"")]
    });
    check(
        "duplicate a style",
        &[],
        |a| {
            a.select_style("Title");
            a.duplicate_style();
        },
        |s| {
            vec![Command::Style(StyleCommand::Add {
                name: "Style 1".to_owned(),
                options: StyleOption::values_of(&s.document().styles["Title"]),
            })]
        },
    );
    check(
        "rename a style",
        &[],
        |a| {
            a.select_style("Legend");
            a.rename_style("Caption".to_owned());
        },
        |_| vec![c("style rename \"Legend\" \"Caption\"")],
    );
    check(
        "change a value of a style",
        &[],
        |a| {
            a.select_style("Body");
            a.set_selected_style(StyleOption::Size(64.0));
        },
        |_| vec![c("style set \"Body\" size=64")],
    );
    check(
        "remove a style no text names",
        &[],
        |a| {
            a.select_style("Legend");
            a.remove_style();
        },
        |_| vec![c("style remove \"Legend\"")],
    );
    check(
        "remove a style texts name, into a custom copy",
        &[],
        |a| {
            a.select_style("Body");
            a.remove_style();
            a.confirm_removal();
        },
        |_| vec![c("style remove \"Body\" into=custom")],
    );
}

/// A control of the selection never changes a named style: a size on a text naming Body makes the
/// text custom at that size, and Body keeps its own.
#[test]
fn a_control_of_the_selection_never_changes_a_named_style() {
    let mut app = App::new(Box::new(MemoryFiles::default()));
    assert!(matches!(app.run(c("sample")), Reply::Ok(_)));
    let body = app.session().document().styles["Body"].clone();
    app.select_node(NodeId(3));
    app.set_style(StyleOption::Size(64.0));
    let doc = app.session().document();
    assert_eq!(doc.styles["Body"], body);
    match doc.node(NodeId(3)) {
        Some(Node::Area(a)) => match &a.content {
            Content::Text(t) => assert_eq!(
                t.style,
                rootpoint_model::StyleChoice::Custom(rootpoint_model::TextStyle {
                    size: 64.0,
                    ..body
                })
            ),
            other => panic!("n3 holds {other:?}"),
        },
        other => panic!("n3 is {other:?}"),
    }
}

#[test]
fn the_menu_actions_issue_their_commands() {
    check("undo", &["select n3", "clear n3"], App::undo, |_| {
        vec![c("undo")]
    });
    check(
        "redo",
        &["select n3", "clear n3", "undo"],
        App::redo,
        |_| vec![c("redo")],
    );
    check(
        "save",
        &[],
        |a| a.save_to("out/deck.ron".to_owned()),
        |_| vec![c("save out/deck.ron")],
    );
    check(
        "open",
        &["save out/deck.ron", "clear n3"],
        |a| a.open_file("out/deck.ron".to_owned()),
        |_| vec![c("load out/deck.ron")],
    );
    check(
        "open the built-in sample",
        &["clear n3"],
        App::open_sample,
        |_| vec![c("sample")],
    );
}

#[test]
fn the_presenter_actions_issue_their_commands() {
    check("present", &["select s7"], App::present, |_| {
        vec![c("present s7")]
    });
    check("next", &["present s1"], App::next_slide, |_| {
        vec![c("next")]
    });
    check("previous", &["present s7"], App::previous_slide, |_| {
        vec![c("prev")]
    });
    check("leave", &["present s7"], App::stop_presenting, |_| {
        vec![c("stop")]
    });
}

/// The actions a browser picker's files go through, on an app that holds its files in memory: a
/// picked directory loads its one presentation, whose files the app holds under their paths below
/// the directory, and a picked image is added under its name, or a numbered name where another
/// file with other bytes has it.
#[test]
fn the_picker_actions_issue_their_commands() {
    let ron = include_bytes!("data/mixed.ron").to_vec();
    let png = include_bytes!("data/two-colours.png").to_vec();
    let mut app = App::in_memory();
    let mut files = MemoryFiles::default();
    for (path, bytes) in [("mixed.ron", &ron), ("two-colours.png", &png)] {
        files.0.borrow_mut().insert(path.to_owned(), bytes.clone());
    }
    let mut session = Session::new();
    let same = |app: &App, session: &Session, name: &str| {
        assert_eq!(app.session().document(), session.document(), "{name}");
        assert_eq!(app.session().view(), session.view(), "{name}");
    };
    let picked = vec![
        ("talk/mixed.ron".to_owned(), ron.clone()),
        ("talk/two-colours.png".to_owned(), png.clone()),
    ];
    app.open_directory(picked);
    assert!(matches!(
        session.run(c("load mixed.ron"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "open a directory");
    // A load leaves no step to undo, in the app as in the session.
    assert_eq!(
        app.run(Command::Undo),
        session.run(Command::Undo, &mut files)
    );
    app.add_picked_image("two-colours.png".to_owned(), png);
    assert!(matches!(
        session.run(c("asset two-colours.png"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "add an image held under its name");
    app.add_picked_image("two-colours.png".to_owned(), vec![0, 1, 2]);
    files
        .0
        .borrow_mut()
        .insert("two-colours-2.png".to_owned(), vec![0, 1, 2]);
    assert!(matches!(
        session.run(c("asset two-colours-2.png"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "add an image under a numbered name");
    assert_eq!(
        app.run(Command::Undo),
        session.run(Command::Undo, &mut files)
    );
    same(&app, &session, "after undo");
    // The browser build saves nothing.
    assert!(matches!(app.run(c("save x.ron")), Reply::Err(_)));
}
