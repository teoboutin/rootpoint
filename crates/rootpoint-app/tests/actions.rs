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
use rootpoint_app::hit::{Edge, Layout, Preview, Zone};
use rootpoint_app::selection::Move;
use rootpoint_app::{App, MenuEntry};
use rootpoint_model::{
    AssetId, Axis, CaretPos, Content, ImageFit, Node, NodeId, Side, SlideId, SpanId, TextNodeRef,
};
use rootpoint_protocol::{
    Command, Files, Reply, Session, SpanField, SpanOption, StyleCommand, StyleOption, parse_command,
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

    fn canonical(&mut self, path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }
}

impl MemoryFiles {
    /// Files at these paths, each holding its own path's bytes, for a test that imports them.
    fn holding(paths: &[&str]) -> Self {
        let files = MemoryFiles::default();
        for p in paths {
            files
                .0
                .borrow_mut()
                .insert(p.to_string(), p.as_bytes().to_vec());
        }
        files
    }
}

/// The ids of n3's spans, in the order of its text.
fn spans_of(session: &Session) -> Vec<SpanId> {
    match session.document().node(NodeId(3)) {
        Some(Node::Area(a)) => match &a.content {
            Content::Text(t) => t.root.spans().iter().map(|s| s.id).collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// Enter editing n3, the selected area, with no window.
fn edit_n3(a: &mut App) {
    let doc = a.session().document();
    let view = Rect::from_min_size(Pos2::ZERO, Vec2::new(960.0, 540.0));
    let layout = Layout::new(doc, SlideId(1), view, [960, 540]).unwrap();
    let p = layout.rect(NodeId(3)).unwrap().center();
    a.edit_at(NodeId(3), p, &layout);
}

fn c(line: &str) -> Command {
    parse_command(line).unwrap_or_else(|e| panic!("{line}: {e}"))
}

/// The text of an area of the sample, as a command must repeat it.
fn text_of(session: &Session, area: NodeId) -> String {
    match session.document().node(area) {
        Some(Node::Area(a)) => match &a.content {
            Content::Text(t) => t.plain_text(),
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
    let images = ["cat.png", "other.png"];
    let (files_a, mut files_b) = (MemoryFiles::holding(&images), MemoryFiles::holding(&images));
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
    assert_eq!(app.session().store(), session.store(), "{name}: the store");
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
        // The sample's next id is 33, so the split's new area is n33.
        |_| {
            vec![
                c("begin"),
                c("split n2 stacked 0.5 after"),
                c("swap n3 n33"),
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
        |_| {
            vec![
                c("begin"),
                c("insert n3 0 \"A\""),
                c("insert n3 1 \"b\""),
                c("delete n3 1..2"),
                c("end"),
            ]
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
        |_| {
            vec![
                c("begin"),
                c("delete n3 0..1"),
                c("insert n3 0 \"xy\""),
                c("insert n3 2 \"´\""),
                c("delete n3 2..3"),
                c("insert n3 2 \"é\""),
                c("delete n3 3..4"),
                c("end"),
            ]
        },
    );
    // At a span's edge, Backspace selects the span, then deletes it, and unwraps the span the
    // caret stands at the start of; the caret leaving an emptied span unwraps it, and so does the
    // stay's end for every empty span left.
    let tree = r#"text n3 ["ab", {bold "cd"}, "e", {italic "f"}, {bold "g"}, {bold ""}]"#;
    check(
        "delete and unwrap at a span's edges",
        &["select n3", tree],
        |a| {
            let spans = spans_of(a.session());
            let span = |i: usize| TextNodeRef::Span(spans[i]);
            let at = |node, offset| CaretPos { node, offset };
            edit_n3(a);
            a.set_caret_pos(at(TextNodeRef::Root, 3));
            a.delete_backward();
            a.delete_backward();
            a.set_caret_pos(at(span(1), 0));
            a.delete_backward();
            a.set_caret_pos(at(span(2), 1));
            a.delete_backward();
            a.move_caret(rootpoint_app::editing::CaretMove::Right, false);
            a.stop_editing();
        },
        |s| {
            let t = spans_of(s);
            vec![
                c("begin"),
                c("delete n3 2..3"),
                c(&format!("unwrap {}", t[1])),
                c(&format!("delete {} 0..1", t[2])),
                c(&format!("unwrap {}", t[2])),
                c(&format!("unwrap {}", t[3])),
                c("end"),
            ]
        },
    );
    // A style option changes the style alone, the text's tree and its ids kept: `text n3 bold`.
    for word in ["bold", "size=80", "align=right"] {
        check(
            word,
            &["select n3"],
            |a| {
                let Command::Text { mut style, .. } = c(&format!("text n3 {word}")) else {
                    panic!("a text command");
                };
                a.set_style(style.remove(0));
            },
            |_| vec![c(&format!("text n3 {word}"))],
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
        |_| vec![c("import cat.png")],
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
        "rename a style texts name",
        &[],
        |a| {
            a.select_style("Body");
            a.rename_style("Text".to_owned());
        },
        |_| vec![c("style rename \"Body\" \"Text\"")],
    );
    check(
        "remove a style texts name, into another",
        &[],
        |a| {
            a.select_style("Body");
            a.remove_style();
            a.choose_replacement(Some("Title".to_owned()));
            a.confirm_removal();
        },
        |_| vec![c("style remove \"Body\" into=\"Title\"")],
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

/// The span style actions of the Styles tab, on the sample's span style "Key", which the span
/// `t30` names.
#[test]
fn the_span_style_table_actions_issue_their_commands() {
    check("add a span style", &[], App::add_span_style, |_| {
        vec![c("span-style add \"Span style 1\"")]
    });
    check(
        "duplicate a span style",
        &[],
        |a| {
            a.select_span_style("Key");
            a.duplicate_span_style();
        },
        |_| vec![c("span-style add \"Span style 1\" bold colour=#2a9d8f")],
    );
    check(
        "rename a span style a span names",
        &[],
        |a| {
            a.select_span_style("Key");
            a.rename_span_style("Shortcut".to_owned());
        },
        |_| vec![c("span-style rename \"Key\" \"Shortcut\"")],
    );
    check(
        "set a value of a span style, and set none",
        &[],
        |a| {
            a.select_span_style("Key");
            a.set_selected_span_style(SpanOption::Size(64.0));
            a.set_selected_span_style(SpanOption::Inherit(SpanField::Bold));
        },
        |_| {
            vec![
                c("span-style set \"Key\" size=64"),
                c("span-style set \"Key\" bold=inherit"),
            ]
        },
    );
    check(
        "remove a span style no span names",
        &["span-style add \"Term\" italic"],
        |a| {
            a.select_span_style("Term");
            a.remove_span_style();
        },
        |_| vec![c("span-style remove \"Term\"")],
    );
    check(
        "remove a span style a span names, into another",
        &["span-style add \"Term\" italic"],
        |a| {
            a.select_span_style("Key");
            a.remove_span_style();
            assert!(a.removal().is_some_and(|r| r.span), "the dialog opens");
            a.choose_replacement(Some("Term".to_owned()));
            a.confirm_removal();
        },
        |_| vec![c("span-style remove \"Key\" into=\"Term\"")],
    );
    check(
        "remove a span style a span names, into a custom copy",
        &[],
        |a| {
            a.select_span_style("Key");
            a.remove_span_style();
            a.confirm_removal();
        },
        |_| vec![c("span-style remove \"Key\" into=custom")],
    );
}

/// The Assets tab's actions, on the sample's image `a19`, shown by `n14` and `n17`, and `cat.png`
/// imported as `a33`, which nothing names.
#[test]
fn the_asset_actions_issue_their_commands() {
    let imported = &["import cat.png"];
    check(
        "move an asset",
        imported,
        |a| a.move_asset(AssetId(19), "img/splits.png".to_owned()),
        |_| vec![c("asset-move a19 img/splits.png")],
    );
    check(
        "drop an asset on a folder",
        &["import cat.png", "asset-move a19 img/splits.png"],
        |a| a.move_asset_to_folder(AssetId(33), "img"),
        |_| vec![c("asset-move a33 img/cat.png")],
    );
    check(
        "drop an asset on the root",
        &["import cat.png", "asset-move a33 img/cat.png"],
        |a| a.move_asset_to_folder(AssetId(33), ""),
        |_| vec![c("asset-move a33 cat.png")],
    );
    check(
        "replace an asset",
        imported,
        |a| a.replace_asset(AssetId(33), "other.png".to_owned()),
        |_| vec![c("asset-replace a33 other.png")],
    );
    check(
        "remove an asset",
        imported,
        |a| a.remove_asset(AssetId(33)),
        |_| vec![c("asset-remove a33")],
    );
    check(
        "remove an asset into another",
        imported,
        |a| a.remove_asset_into(AssetId(19), AssetId(33)),
        |_| vec![c("asset-remove a19 into=a33")],
    );
    check(
        "remove the unused assets",
        imported,
        App::remove_unused_assets,
        |_| vec![c("asset-remove-unused")],
    );
    check(
        "a use's button",
        &[],
        |a| a.show_use(NodeId(17)),
        |_| vec![c("select n17")],
    );
    // A store in memory holds no file no asset names, so the relink here names the asset's own.
    check(
        "relink an asset",
        imported,
        |a| a.relink_asset(AssetId(33), "cat.png".to_owned()),
        |_| vec![c("asset-relink a33 cat.png")],
    );
}

/// The ids of n3's spans the action made: those of `after` that `before` lacks, in the order of
/// the text.
fn made(before: &[SpanId], after: &[SpanId]) -> Vec<SpanId> {
    after
        .iter()
        .filter(|t| !before.contains(t))
        .copied()
        .collect()
}

/// The style actions while a text is edited: Ctrl+B and Ctrl+I, the Selection tab's controls,
/// "split here" and a span of the ancestor row, each the commands of the stay's one group.
#[test]
fn the_span_style_actions_issue_their_commands() {
    let tree = r#"text n3 ["abc", {bold "de"}, "f"]"#;
    let at = |node, offset| CaretPos { node, offset };
    let root = TextNodeRef::Root;
    // With no selection, a style action opens an empty span at the caret, which typing goes into.
    let new = Rc::new(RefCell::new(Vec::new()));
    let seen = new.clone();
    check(
        "Ctrl+B at the caret, then typing",
        &["select n3", tree],
        move |a| {
            let before = spans_of(a.session());
            edit_n3(a);
            a.set_caret_pos(at(root, 1));
            a.bold();
            a.type_str("X");
            *seen.borrow_mut() = made(&before, &spans_of(a.session()));
            a.stop_editing();
        },
        move |_| {
            let t = new.borrow()[0];
            vec![
                c("begin"),
                c("wrap n3 1..1 bold"),
                c(&format!("insert {t} 0 \"X\"")),
                c("end"),
            ]
        },
    );
    // With a selection, a style action wraps it and selects the new span, so that a second one
    // changes that span's style.
    let new = Rc::new(RefCell::new(Vec::new()));
    let seen = new.clone();
    check(
        "Ctrl+I over a selection, then Ctrl+B on its span",
        &["select n3", tree],
        move |a| {
            let before = spans_of(a.session());
            edit_n3(a);
            a.set_caret_pos(at(root, 0));
            a.move_caret(rootpoint_app::editing::CaretMove::Right, true);
            a.italic();
            a.bold();
            *seen.borrow_mut() = made(&before, &spans_of(a.session()));
            a.stop_editing();
        },
        move |_| {
            let t = new.borrow()[0];
            vec![
                c("begin"),
                c("wrap n3 0..1 italic"),
                c(&format!("span {t} bold")),
                c("end"),
            ]
        },
    );
    // Inside a span, "split here" cuts it, the caret between the halves; at its start it sends
    // nothing and moves the caret before it.
    check(
        "split here, inside a span and at its start",
        &["select n3", tree],
        |a| {
            let span = TextNodeRef::Span(spans_of(a.session())[0]);
            edit_n3(a);
            a.set_caret_pos(at(span, 1));
            a.split_here();
            a.type_str("Y");
            a.set_caret_pos(at(span, 0));
            a.split_here();
            a.type_str("Z");
            a.stop_editing();
        },
        |s| {
            let t = spans_of(s)[0];
            vec![
                c("begin"),
                c(&format!("split-span {t} 1")),
                c("insert n3 4 \"Y\""),
                c("insert n3 3 \"Z\""),
                c("end"),
            ]
        },
    );
    // A span of the ancestor row selected, a control changes its style; an alignment changes the
    // area's; at the caret, a control opens a span, which the stay's end unwraps while empty.
    let new = Rc::new(RefCell::new(Vec::new()));
    let seen = new.clone();
    check(
        "the Selection tab's controls while editing",
        &["select n3", tree],
        move |a| {
            let before = spans_of(a.session());
            edit_n3(a);
            a.select_span(before[0]);
            a.edit_control(StyleOption::Size(80.0));
            a.edit_control(StyleOption::Align(rootpoint_model::HAlign::Right));
            a.set_caret_pos(at(root, 0));
            a.edit_control(StyleOption::Italic(true));
            *seen.borrow_mut() = made(&before, &spans_of(a.session()));
            a.stop_editing();
        },
        move |s| {
            let (old, t) = (spans_of(s)[0], new.borrow()[0]);
            vec![
                c("begin"),
                c(&format!("span {old} size=80")),
                c("text n3 align=right"),
                c("wrap n3 0..0 italic"),
                c(&format!("unwrap {t}")),
                c("end"),
            ]
        },
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
    // The browser build's Save downloads a pack, `design@rootpoint-app@browser-pack-save`, which
    // `path@rootpoint-app@tests/web.rs` checks.
    #[cfg(not(target_arch = "wasm32"))]
    check("save", &["open out", "add-slide"], App::save, |_| {
        vec![c("save")]
    });
    check(
        "open",
        &["open out", "add-slide", "save", "sample"],
        |a| a.open_workspace("out".to_owned()),
        |_| vec![c("open out")],
    );
    check(
        "pack",
        &["add-slide"],
        |a| a.pack_to("s.rpk".to_owned()),
        |_| vec![c("pack s.rpk")],
    );
    check(
        "open a pack",
        &["pack s.rpk", "add-slide"],
        |a| a.open_pack("s.rpk".to_owned()),
        |_| vec![c("open-pack s.rpk")],
    );
    check(
        "unpack",
        &["add-slide"],
        |a| a.unpack_to("out".to_owned()),
        |_| vec![c("unpack out")],
    );
    check(
        "open the built-in sample",
        &["clear n3"],
        App::open_sample,
        |_| vec![c("sample")],
    );
}

/// Open the node menu on `node`, as a secondary click does, then click `entry`.
fn menu(node: u64, entry: MenuEntry) -> impl FnOnce(&mut App) {
    move |a| {
        a.open_node_menu(NodeId(node), Pos2::ZERO);
        a.menu_entry(entry);
    }
}

/// Each entry of the node menu runs the side panel's action on the node the menu was opened on,
/// which the secondary click selected first.
#[test]
fn the_node_menu_entries_issue_their_commands() {
    check(
        "menu: open",
        &["select n5"],
        menu(3, MenuEntry::Select(NodeId(3))),
        |_| vec![c("select n3")],
    );
    check(
        "menu: select an ancestor",
        &[],
        menu(3, MenuEntry::Select(NodeId(4))),
        |_| vec![c("select n3"), c("select n4")],
    );
    check("menu: remove", &[], menu(3, MenuEntry::Remove), |_| {
        vec![c("select n3"), c("remove n3")]
    });
    check(
        "menu: content text",
        &["image n3 a19 fill"],
        menu(3, MenuEntry::Text),
        |_| vec![c("select n3"), c("text n3 \"\"")],
    );
    check(
        "menu: content image",
        &[],
        menu(3, MenuEntry::Image(AssetId(19))),
        |_| vec![c("select n3"), c("image n3 a19 fit")],
    );
    check(
        "menu: another image keeps the fit",
        &["import other.png", "image n3 a19 fill"],
        menu(3, MenuEntry::Image(AssetId(33))),
        |_| vec![c("select n3"), c("image n3 a33 fill")],
    );
    check(
        "menu: name a style",
        &[],
        menu(3, MenuEntry::Style(Some("Title".to_owned()))),
        |s| {
            vec![
                c("select n3"),
                c(&format!(
                    "text n3 {:?} style=\"Title\"",
                    text_of(s, NodeId(3))
                )),
            ]
        },
    );
    check(
        "menu: make a style custom",
        &[],
        menu(3, MenuEntry::Style(None)),
        |s| {
            vec![
                c("select n3"),
                c(&format!("text n3 {:?} custom", text_of(s, NodeId(3)))),
            ]
        },
    );
    // The entry "image" on an image area names the image it holds, and sends nothing, as the side
    // panel's button does; the menu's own entries are read, not built by the test.
    check(
        "menu: image on an image area",
        &["import other.png", "image n3 a33 fill"],
        |a| {
            a.open_node_menu(NodeId(3), Pos2::ZERO);
            let item = a.menu_items().into_iter().find(|i| i.label == "image");
            a.menu_entry(item.unwrap().entry);
        },
        |_| vec![c("select n3")],
    );
    // An entry that names what the node holds sends nothing, as the side panel's does.
    check(
        "menu: the style it names",
        &[],
        menu(3, MenuEntry::Style(Some("Body".to_owned()))),
        |_| vec![c("select n3")],
    );
}

#[test]
fn the_final_view_actions_issue_their_commands() {
    check(
        "enter the final view",
        &["select n3"],
        App::toggle_final_view,
        |_| vec![c("preview on")],
    );
    check(
        "leave the final view",
        &["preview on"],
        App::toggle_final_view,
        |_| vec![c("preview off")],
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
/// picked directory opens a copy of its presentation, whose files the app holds under their paths,
/// and a picked image is imported under its name, or a numbered name where the map holds other
/// bytes under it.
#[test]
fn the_picker_actions_issue_their_commands() {
    let ron = include_bytes!("data/mixed.ron").to_vec();
    let png = include_bytes!("data/two-colours.png").to_vec();
    let mut app = App::in_memory();
    let mut files = MemoryFiles::default();
    let picked = vec![
        ("talk/presentation.ron".to_owned(), ron.clone()),
        ("talk/two-colours.png".to_owned(), png.clone()),
    ];
    for (path, bytes) in &picked {
        files.0.borrow_mut().insert(path.clone(), bytes.clone());
    }
    let mut session = Session::new();
    let same = |app: &App, session: &Session, name: &str| {
        assert_eq!(app.session().document(), session.document(), "{name}");
        assert_eq!(app.session().view(), session.view(), "{name}");
        assert_eq!(app.session().store(), session.store(), "{name}");
    };
    app.open_directory(picked);
    assert!(matches!(
        session.run(c("open-copy talk"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "open a directory");
    // An open leaves no step to undo, in the app as in the session.
    assert_eq!(
        app.run(Command::Undo),
        session.run(Command::Undo, &mut files)
    );
    app.add_picked_image("two-colours.png".to_owned(), png.clone());
    files
        .0
        .borrow_mut()
        .insert("two-colours.png".to_owned(), png);
    assert!(matches!(
        session.run(c("import two-colours.png"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "add an image the map holds under its name");
    app.add_picked_image("two-colours.png".to_owned(), vec![0, 1, 2]);
    files
        .0
        .borrow_mut()
        .insert("two-colours.png".to_owned(), vec![0, 1, 2]);
    assert!(matches!(
        session.run(c("import two-colours.png"), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "add an image under a numbered name");
    assert_eq!(
        app.session()
            .document()
            .assets
            .values()
            .last()
            .unwrap()
            .path,
        "two-colours-2.png"
    );
    assert_eq!(
        app.run(Command::Undo),
        session.run(Command::Undo, &mut files)
    );
    same(&app, &session, "after undo");
    // A file picked to replace an asset's bytes is held under its name, then `asset-replace`.
    let last = *app.session().document().assets.keys().last().unwrap();
    app.replace_with_picked(last, "new.png".to_owned(), vec![7, 7]);
    files
        .0
        .borrow_mut()
        .insert("new.png".to_owned(), vec![7, 7]);
    assert!(matches!(
        session.run(c(&format!("asset-replace {last} new.png")), &mut files),
        Reply::Ok(_)
    ));
    same(&app, &session, "replace an asset by a picked file");
    // A presentation held in memory, with no pack its target, has no file the `save` command
    // writes: the browser build's Save then packs it, `App::pack_for_download`.
    assert!(matches!(app.run(c("save")), Reply::Err(_)));
}

/// A pack the author picked opens through the files held in memory as `open-pack` opens it in a
/// session, in the browser as natively; a file that is no pack is refused and changes nothing.
#[test]
fn a_picked_pack_opens_as_open_pack_does() {
    let mut files = MemoryFiles::default();
    let mut session = Session::new();
    for line in ["sample", "add-slide", "pack s.rpk"] {
        assert!(
            matches!(session.run(c(line), &mut files), Reply::Ok(_)),
            "{line}"
        );
    }
    let pack = files.0.borrow()["s.rpk"].clone();
    let mut expected = Session::new();
    assert!(matches!(
        expected.run(c("open-pack s.rpk"), &mut files),
        Reply::Ok(_)
    ));
    let mut app = App::in_memory();
    app.open_sample();
    app.open_picked_pack("s.rpk".to_owned(), pack);
    assert_eq!(app.status(), Vec::<String>::new());
    assert_eq!(app.session().document(), expected.document());
    assert_eq!(app.session().store(), expected.store());
    assert_eq!(app.session().pack(), Some("s.rpk"));
    let before = app.session().document().clone();
    // Picked under the same name, a file that is no pack leaves the pack held there before.
    app.open_picked_pack("s.rpk".to_owned(), b"no zip".to_vec());
    assert_eq!(app.status().len(), 1, "{:?}", app.status());
    assert_eq!(app.session().document(), &before);
    assert!(matches!(app.run(c("open-pack s.rpk")), Reply::Ok(_)));
}
