//! The commands of a text tree and of the span style table: each runs its operation, names the
//! spans it makes, refuses what would leave a breach, and `tree` prints a text in the grammar
//! `text` takes.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Command, Reply, Session, SpanOption, SpanStyleCommand};

/// A session holding one slide whose area n2 holds `text`'s tree, in the style Body.
fn one_text(tree: &str) -> (Session, MemoryFiles) {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    ok(&mut s, &mut files, "add-slide");
    ok(&mut s, &mut files, &format!("text n2 {tree}"));
    (s, files)
}

fn refused(s: &mut Session, files: &mut MemoryFiles, line: &str) -> String {
    let doc = s.document().clone();
    match s.execute(line, files) {
        Some(Reply::Err(reason)) => {
            assert_eq!(s.document(), &doc, "`{line}` changed the document");
            reason
        }
        other => panic!("`{line}` gave {other:?}"),
    }
}

fn root_of(s: &Session, area: NodeId) -> TextNode {
    match s.document().node(area) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => t.root.clone(),
        other => panic!("{area} holds {other:?}"),
    }
}

/// A text node with every span's id set to 0, so that two trees compare by their characters and
/// styles alone.
fn without_ids(node: &TextNode) -> TextNode {
    TextNode {
        items: node
            .items
            .iter()
            .map(|item| match item {
                TextItem::Chars(_) | TextItem::Math(_) | TextItem::Ref(_) => item.clone(),
                TextItem::Span(span) => TextItem::Span(Span {
                    id: SpanId(0),
                    style: span.style.clone(),
                    text: without_ids(&span.text),
                }),
            })
            .collect(),
    }
}

fn custom(style: SpanStyle) -> SpanChoice {
    SpanChoice::Custom(style)
}

fn bold() -> SpanStyle {
    SpanStyle {
        bold: Some(true),
        ..SpanStyle::default()
    }
}

/// The script of every new command gives its recorded replies, `tree` printing each text as a
/// tree with its spans' ids.
#[test]
fn a_script_of_the_span_commands_gives_its_replies() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    let replies: Vec<String> = run(&mut s, &mut files, &script("spans.txt"))
        .iter()
        .map(|r| format!("{r}\n"))
        .collect();
    assert_eq!(replies.concat(), script("spans.replies"));
}

/// Each command applies one operation, the one listed beside it: the document after the command
/// equals the document after `apply` of that operation.
#[test]
fn each_span_command_applies_its_operation() {
    let setup = "span-style add \"Term\" italic";
    let tree = "[\"ab \", {bold \"cd\"}, \" ef\"]";
    let t3 = SpanId(3);
    let rows: Vec<(&str, Op)> = vec![
        (
            "insert n2 1 \"X\"",
            Op::InsertText {
                area: NodeId(2),
                node: TextNodeRef::Root,
                offset: 1,
                text: "X".into(),
            },
        ),
        (
            "insert t3 2 \"Y\"",
            Op::InsertText {
                area: NodeId(2),
                node: TextNodeRef::Span(t3),
                offset: 2,
                text: "Y".into(),
            },
        ),
        (
            "delete n2 1..4",
            Op::DeleteText {
                area: NodeId(2),
                node: TextNodeRef::Root,
                range: 1..4,
            },
        ),
        (
            "wrap t3 0..1 italic",
            Op::WrapSpan {
                area: NodeId(2),
                node: TextNodeRef::Span(t3),
                range: 0..1,
                style: custom(SpanStyle {
                    italic: Some(true),
                    ..SpanStyle::default()
                }),
            },
        ),
        (
            "wrap n2 0..2 style=\"Term\"",
            Op::WrapSpan {
                area: NodeId(2),
                node: TextNodeRef::Root,
                range: 0..2,
                style: SpanChoice::Named("Term".into()),
            },
        ),
        (
            "unwrap t3",
            Op::UnwrapSpan {
                area: NodeId(2),
                span: t3,
            },
        ),
        (
            "span t3 size=20",
            Op::SetSpanStyle {
                area: NodeId(2),
                span: t3,
                style: custom(SpanStyle {
                    size: Some(20.0),
                    bold: Some(true),
                    ..SpanStyle::default()
                }),
            },
        ),
        (
            "split-span t3 1",
            Op::SplitSpan {
                area: NodeId(2),
                span: t3,
                offset: 1,
            },
        ),
        (
            "span-style add \"Key\" bold",
            Op::AddSpanStyle {
                name: "Key".into(),
                style: bold(),
            },
        ),
        (
            "span-style set \"Term\" bold",
            Op::SetSpanStyleValues {
                name: "Term".into(),
                style: SpanStyle {
                    bold: Some(true),
                    italic: Some(true),
                    ..SpanStyle::default()
                },
            },
        ),
        (
            "span-style rename \"Term\" \"Key\"",
            Op::RenameSpanStyle {
                from: "Term".into(),
                to: "Key".into(),
            },
        ),
        (
            "span-style remove \"Term\"",
            Op::RemoveSpanStyle {
                name: "Term".into(),
                into: None,
            },
        ),
    ];
    for (line, op) in rows {
        let (mut s, mut files) = one_text(tree);
        ok(&mut s, &mut files, setup);
        let mut expected = s.document().clone();
        assert!(
            matches!(s.execute(line, &mut files), Some(Reply::Ok(_))),
            "`{line}`"
        );
        apply(&mut expected, op).unwrap();
        assert!(s.document().same_content(&expected), "`{line}`");
    }
}

/// `tree`'s line of a text, given back to `text`, gives the same text, every span with a new id.
#[test]
fn a_tree_printed_by_tree_and_given_to_text_gives_the_same_text() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    // A styled text with nested and named spans, built by commands, and a quote, a backslash, a
    // new line and the tree's own punctuation in its characters.
    let built = [
        "add-slide",
        "split n2 beside 0.5",
        "span-style add \"Term \\\"x\\\"\" italic colour=#336699",
        "text n2 [\"a \\\"q\\\" \\\\ [b], {c}\\n\", {bold size=30 \"bold \", {style=\"Term \\\"x\\\"\" \"term\"}}, \" end\"] style=\"Title\"",
        "wrap n2 0..1 colour=#ff000080 font=sans",
    ];
    for line in built {
        ok(&mut s, &mut files, line);
    }
    let line = ok(&mut s, &mut files, "tree s1");
    let start = line.find("n2 text ").expect("n2 is printed") + "n2 text ".len();
    let end = line
        .rfind(" style=\"Title\"")
        .expect("n2's style is printed");
    let tree = &line[start..end];
    assert!(tree.contains("{t"), "the spans' ids are printed: {tree}");
    let before = root_of(&s, NodeId(2));
    let reply = ok(&mut s, &mut files, &format!("text n3 {tree}"));
    let after = root_of(&s, NodeId(3));
    assert_eq!(without_ids(&after), without_ids(&before), "{tree}");
    let old: Vec<SpanId> = before.spans().iter().map(|t| t.id).collect();
    let new: Vec<SpanId> = after.spans().iter().map(|t| t.id).collect();
    assert!(new.iter().all(|id| !old.contains(id)), "{old:?} {new:?}");
    let named: Vec<String> = new.iter().map(|t| t.to_string()).collect();
    assert_eq!(reply, named.join(" "));
}

/// A span keeps its id across an edit before it, so a script styles it again by its id.
#[test]
fn a_span_keeps_its_id_across_edits_before_it() {
    let (mut s, mut files) = one_text("[\"a \", {bold \"word\"}]");
    ok(&mut s, &mut files, "insert n2 0 \"a longer \"");
    ok(&mut s, &mut files, "delete n2 0..2");
    ok(&mut s, &mut files, "span t3 italic");
    let span = root_of(&s, NodeId(2)).spans()[0].clone();
    assert_eq!(span.id, SpanId(3));
    assert_eq!(
        span.style,
        custom(SpanStyle {
            bold: Some(true),
            italic: Some(true),
            ..SpanStyle::default()
        })
    );
}

/// `split-span` at either edge of a span is refused, and inside it gives the second half's id.
#[test]
fn split_span_at_an_edge_is_refused() {
    let (mut s, mut files) = one_text("[{bold \"word\"}]");
    refused(&mut s, &mut files, "split-span t3 0");
    refused(&mut s, &mut files, "split-span t3 4");
    assert_eq!(ok(&mut s, &mut files, "split-span t3 2"), "t4");
}

/// A command that names a span style the table lacks, or a font asset it lacks, is refused, so no
/// command makes a breach, `design@rootpoint-protocol@no-new-breach`; and so is one that would
/// leave a custom span setting nothing.
#[test]
fn a_span_command_naming_a_missing_target_is_refused() {
    let (mut s, mut files) = one_text("[\"ab\", {bold \"cd\"}]");
    for line in [
        "wrap n2 0..1 style=\"Nope\"",
        "span t3 style=\"Nope\"",
        "text n2 [{style=\"Nope\" \"x\"}]",
        "wrap n2 0..1 font=a99",
        "span t3 font=a99",
        "span-style add \"Key\" font=a99",
        "span-style remove \"Key\"",
        "span-style set \"Key\" bold",
        "unwrap t99",
        "insert t99 0 \"x\"",
    ] {
        refused(&mut s, &mut files, line);
    }
    assert!(refused(&mut s, &mut files, "span t3 bold=inherit").contains("unwrap"));
    refused(&mut s, &mut files, "wrap n2 0..1");
    refused(&mut s, &mut files, "text n2 [{\"x\"}]");
    // A span style whose font the table lacks, as only a file can hold, is refused as a span's
    // style, as a text style is; the slide's id, 1, is below the counter and names no asset.
    ok(&mut s, &mut files, "span-style add \"Key\" bold");
    let mut doc = s.document().clone();
    doc.span_styles.get_mut("Key").unwrap().font = Some(FontRef::Asset(AssetId(1)));
    files
        .files
        .insert("deck/presentation.ron".into(), save_ron(&doc).into_bytes());
    ok(&mut s, &mut files, "open-copy deck");
    assert_eq!(
        refused(&mut s, &mut files, "span t3 style=\"Key\""),
        "no asset a1"
    );
}

/// A span style in use is refused a removal with no replacement, and a rename to a name a span
/// names; the replies name the spans.
#[test]
fn a_span_style_in_use_names_its_spans() {
    let (mut s, mut files) = one_text("[\"ab\"]");
    ok(&mut s, &mut files, "span-style add \"Term\" italic");
    assert_eq!(ok(&mut s, &mut files, "wrap n2 0..1 style=\"Term\""), "t3");
    assert_eq!(
        refused(&mut s, &mut files, "span-style remove \"Term\""),
        "\"Term\" is used by t3"
    );
    ok(&mut s, &mut files, "span-style remove \"Term\" into=custom");
    assert_eq!(
        root_of(&s, NodeId(2)).spans()[0].style,
        custom(SpanStyle {
            italic: Some(true),
            ..SpanStyle::default()
        })
    );
    ok(&mut s, &mut files, "undo");
    ok(&mut s, &mut files, "span-style add \"Key\" bold");
    ok(&mut s, &mut files, "wrap n2 1..2 style=\"Key\"");
    assert_eq!(
        refused(&mut s, &mut files, "span-style rename \"Term\" \"Key\""),
        "t4 names \"Key\""
    );
}

/// A value set on a named span makes it custom with the named values and that change; `custom`
/// copies them alone; `<key>=inherit` clears a value, which the span then takes from its parent.
#[test]
fn a_span_s_options_apply_over_its_current_style() {
    let (mut s, mut files) = one_text("[\"ab\"]");
    ok(
        &mut s,
        &mut files,
        "span-style add \"Term\" italic colour=#336699",
    );
    ok(&mut s, &mut files, "wrap n2 0..2 style=\"Term\"");
    ok(&mut s, &mut files, "span t3 size=30 colour=inherit");
    assert_eq!(
        root_of(&s, NodeId(2)).spans()[0].style,
        custom(SpanStyle {
            size: Some(30.0),
            italic: Some(true),
            ..SpanStyle::default()
        })
    );
    ok(&mut s, &mut files, "span t3 style=\"Term\" custom");
    assert_eq!(
        root_of(&s, NodeId(2)).spans()[0].style,
        custom(s.document().span_styles["Term"].clone())
    );
    ok(
        &mut s,
        &mut files,
        "span-style set \"Term\" italic=inherit bold",
    );
    assert_eq!(
        ok(&mut s, &mut files, "span-styles"),
        "\"Term\" bold colour=#336699"
    );
}

/// `text` with options and no tree changes the area's style and keeps its tree with its ids, in
/// one undo step.
#[test]
fn text_with_no_tree_changes_the_style_and_keeps_the_spans() {
    let (mut s, mut files) = one_text("[\"ab\", {bold \"cd\"}]");
    let before = root_of(&s, NodeId(2));
    assert_eq!(ok(&mut s, &mut files, "text n2 style=\"Title\""), "");
    assert_eq!(root_of(&s, NodeId(2)), before);
    assert_eq!(
        ok(&mut s, &mut files, "tree"),
        "s1 n2 text [\"ab\", {t3 bold \"cd\"}] style=\"Title\""
    );
    ok(&mut s, &mut files, "undo");
    assert_eq!(
        ok(&mut s, &mut files, "tree"),
        "s1 n2 text [\"ab\", {t3 bold \"cd\"}] style=\"Body\""
    );
}

/// `text` with a tree on an image area makes it a text, in one undo step, which the undo takes
/// back whole.
#[test]
fn text_with_a_tree_on_an_image_is_one_undo_step() {
    let mut files = MemoryFiles::holding(&["cat.png"]);
    let mut s = Session::new();
    ok(&mut s, &mut files, "add-slide");
    ok(&mut s, &mut files, "import cat.png");
    ok(&mut s, &mut files, "image n2 a3 fit");
    let doc = s.document().clone();
    assert_eq!(
        ok(&mut s, &mut files, "text n2 [{italic \"x\"}] bold"),
        "t4"
    );
    ok(&mut s, &mut files, "undo");
    assert!(s.document().same_content(&doc));
}

/// The render's reply names each span naming a span style the table lacks, with the name, after
/// `span-styles=`.
#[test]
fn a_missing_span_style_is_named_by_the_render() {
    let (mut s, mut files) = one_text("[\"ab\"]");
    ok(&mut s, &mut files, "span-style add \"Term\" italic");
    ok(&mut s, &mut files, "wrap n2 0..1 style=\"Term\"");
    let mut doc = s.document().clone();
    doc.span_styles.clear();
    files
        .files
        .insert("deck/presentation.ron".into(), save_ron(&doc).into_bytes());
    ok(&mut s, &mut files, "open-copy deck");
    let reply = ok(&mut s, &mut files, "render s1 out.png 160 90");
    assert!(
        reply.ends_with(" span-styles=t3:Term commands=- refs=-"),
        "{reply}"
    );
}

/// `span-styles` writes a built-in family the binary does not carry, which only a file edited by
/// hand holds, by its name, as `styles` does, so the reply hides no value the table holds.
#[test]
fn span_styles_names_a_family_the_binary_does_not_carry() {
    let (mut s, mut files) = one_text("[\"ab\"]");
    ok(&mut s, &mut files, "span-style add \"Term\" bold");
    let mut doc = s.document().clone();
    doc.span_styles.get_mut("Term").unwrap().font = Some(FontRef::Builtin("serif".into()));
    files
        .files
        .insert("deck/presentation.ron".into(), save_ron(&doc).into_bytes());
    ok(&mut s, &mut files, "open-copy deck");
    assert_eq!(
        ok(&mut s, &mut files, "span-styles"),
        "\"Term\" bold font=serif"
    );
}

/// A command built through the API that the grammar would refuse is refused by the session too, so
/// that every command a session runs prints back as a line that parses.
#[test]
fn a_command_the_grammar_refuses_is_refused_when_built_through_the_api() {
    let (mut s, mut files) = one_text("[{bold \"ab\"}]");
    ok(&mut s, &mut files, "span-style add \"Term\" italic");
    let commands = [
        Command::Span {
            span: SpanId(3),
            style: Vec::new(),
        },
        Command::SpanStyle(SpanStyleCommand::Add {
            name: "Key".into(),
            options: vec![SpanOption::Custom],
        }),
        Command::SpanStyle(SpanStyleCommand::Set {
            name: "Term".into(),
            options: vec![SpanOption::Named("Term".into())],
        }),
    ];
    for command in commands {
        let doc = s.document().clone();
        let line = command.to_string();
        assert!(rootpoint_protocol::parse_command(&line).is_err(), "{line}");
        let reply = s.run(command, &mut files);
        assert!(matches!(reply, Reply::Err(_)), "`{line}` gave {reply:?}");
        assert_eq!(s.document(), &doc, "`{line}`");
    }
}

/// `text` with options and no tree is refused on an image area, which has no text style to change.
#[test]
fn text_with_no_tree_is_refused_on_an_image() {
    let mut files = MemoryFiles::holding(&["cat.png"]);
    let mut s = Session::new();
    ok(&mut s, &mut files, "add-slide");
    ok(&mut s, &mut files, "import cat.png");
    ok(&mut s, &mut files, "image n2 a3 fit");
    assert_eq!(
        refused(&mut s, &mut files, "text n2 bold"),
        "n2 holds an image: `text` with a tree makes it a text"
    );
}
