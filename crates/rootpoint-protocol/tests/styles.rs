//! Named text styles: a text names a style of the document's table or holds its own values; a style
//! edit changes every text naming it; a rename and a removal carry their texts; a missing style is
//! drawn with the default values and reported; and no command names one.

mod common;

use common::*;
use rootpoint_model::*;
use rootpoint_protocol::{Command, Reply, Session, parse_command};

fn sample(files: &mut MemoryFiles) -> Session {
    let mut s = Session::new();
    ok(&mut s, files, "sample");
    s
}

fn text_of(s: &Session, id: u64) -> TextContent {
    match s.document().node(NodeId(id)) {
        Some(Node::Area(Area {
            content: Content::Text(t),
            ..
        })) => t.clone(),
        other => panic!("n{id} is {other:?}"),
    }
}

fn named(name: &str) -> StyleChoice {
    StyleChoice::Named(name.to_owned())
}

/// Run a line that must be refused, and check that it changed nothing.
fn refused(s: &mut Session, files: &mut MemoryFiles, line: &str) -> String {
    let doc = s.document().clone();
    let reply = s.execute(line, files);
    assert_eq!(s.document(), &doc, "`{line}` changed the document");
    match reply {
        Some(Reply::Err(reason)) => reason,
        other => panic!("`{line}` gave {other:?}"),
    }
}

/// The sample with its n2 naming "Heading", which the table lacks, as a file edited by hand.
fn with_heading(files: &mut MemoryFiles) -> Session {
    let mut s = sample(files);
    ok(&mut s, files, "save deck.ron");
    let text = String::from_utf8(files.files["deck.ron"].clone()).unwrap();
    let edited = text.replacen("Named(\"Title\")", "Named(\"Heading\")", 1);
    assert_ne!(edited, text, "the saved sample changed shape");
    files.files.insert("deck.ron".into(), edited.into_bytes());
    let mut s = Session::new();
    ok(&mut s, files, "load deck.ron");
    s
}

#[test]
fn the_sample_names_title_and_body() {
    let mut files = MemoryFiles::default();
    let s = sample(&mut files);
    assert_eq!(text_of(&s, 2).style, named("Title"));
    for id in [3, 5, 9] {
        assert_eq!(text_of(&s, id).style, named("Body"), "n{id}");
    }
    assert!(matches!(text_of(&s, 11).style, StyleChoice::Custom(_)));
    assert_eq!(
        s.document().styles.keys().collect::<Vec<_>>(),
        ["Body", "Legend", "Subtitle", "Title"]
    );
}

/// `style set` changes every text naming the style, and no other; its undo restores the values.
#[test]
fn a_style_edit_changes_every_area_naming_it() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let before = s.document().clone();
    let title = s.document().resolve_style(&text_of(&s, 2).style);
    ok(
        &mut s,
        &mut files,
        "style set \"Body\" size=50 colour=#204060",
    );
    for id in [3, 5, 9] {
        let drawn = s.document().resolve_style(&text_of(&s, id).style);
        assert_eq!(drawn.size, 50.0, "n{id}");
        assert_eq!(
            drawn.colour,
            Rgba {
                r: 0x20,
                g: 0x40,
                b: 0x60,
                a: 255
            },
            "n{id}"
        );
    }
    assert_eq!(s.document().resolve_style(&text_of(&s, 2).style), title);
    assert_eq!(
        s.document().slides,
        before.slides,
        "a style edit changes no area"
    );
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &before);
}

#[test]
fn a_rename_rewrites_the_references_and_its_undo_restores_them() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let before = s.document().clone();
    ok(&mut s, &mut files, "style rename \"Body\" \"Text\"");
    for id in [3, 5, 9] {
        assert_eq!(text_of(&s, id).style, named("Text"), "n{id}");
    }
    assert!(!s.document().styles.contains_key("Body"));
    assert_eq!(s.document().breaches(), vec![]);
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &before);
}

#[test]
fn a_removal_in_use_takes_a_replacement() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let before = s.document().clone();
    assert_eq!(
        refused(&mut s, &mut files, "style remove \"Body\""),
        "\"Body\" is used by n3 n5 n9"
    );
    ok(&mut s, &mut files, "style remove \"Body\" into=\"Title\"");
    for id in [3, 5, 9] {
        assert_eq!(text_of(&s, id).style, named("Title"), "n{id}");
    }
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &before);
    ok(&mut s, &mut files, "render s1 before.png");
    let body = before.styles["Body"].clone();
    ok(&mut s, &mut files, "style remove \"Body\" into=custom");
    for id in [3, 5, 9] {
        assert_eq!(
            text_of(&s, id).style,
            StyleChoice::Custom(body.clone()),
            "n{id}"
        );
    }
    assert_eq!(s.document().breaches(), vec![]);
    ok(&mut s, &mut files, "render s1 after.png");
    assert_eq!(
        files.files["before.png"], files.files["after.png"],
        "a custom copy changes no slide"
    );
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &before);
    // A replacement that is the style itself, or that the table lacks, would leave a breach.
    refused(&mut s, &mut files, "style remove \"Body\" into=\"Body\"");
    refused(&mut s, &mut files, "style remove \"Body\" into=\"Heading\"");
}

/// A text naming a style the table lacks loads, draws with the default values, and is named.
#[test]
fn a_missing_style_draws_with_the_default_and_is_reported() {
    let mut files = MemoryFiles::default();
    let mut s = with_heading(&mut files);
    assert_eq!(
        s.document().breaches(),
        vec![Breach::MissingStyle {
            area: NodeId(2),
            name: "Heading".to_owned()
        }]
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 heading.png 320 180"),
        "fallback=- missing=- glyphs=- styles=n2:Heading"
    );
    // The same text in the default values, as its own, draws the same image.
    ok(&mut s, &mut files, "text n2 \"rootpoint\" custom");
    assert_eq!(
        text_of(&s, 2).style,
        StyleChoice::Custom(TextStyle::default())
    );
    assert_eq!(
        ok(&mut s, &mut files, "render s1 default.png 320 180"),
        "fallback=- missing=- glyphs=- styles=-"
    );
    assert_eq!(files.files["heading.png"], files.files["default.png"]);
}

#[test]
fn no_command_names_a_missing_style() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    assert_eq!(
        refused(&mut s, &mut files, "text n3 \"x\" style=\"Heading\""),
        "no style \"Heading\""
    );
    // A style of the table whose font is an asset the table lacks, as a file edited by hand holds.
    ok(&mut s, &mut files, "asset font.ttf");
    let font = *s.document().assets.keys().max().unwrap();
    ok(
        &mut s,
        &mut files,
        &format!("style add \"Odd\" font={font}"),
    );
    ok(&mut s, &mut files, "save odd.ron");
    let text = String::from_utf8(files.files["odd.ron"].clone()).unwrap();
    let table = format!(
        "        {}: (\n            source: Path(\"font.ttf\"),\n        ),\n",
        font.0
    );
    let edited = text.replacen(&table, "", 1);
    assert_ne!(edited, text, "the saved file changed shape");
    files.files.insert("odd.ron".into(), edited.into_bytes());
    ok(&mut s, &mut files, "load odd.ron");
    assert_eq!(
        refused(&mut s, &mut files, "text n3 \"x\" style=\"Odd\""),
        format!("no asset {font}")
    );
    refused(
        &mut s,
        &mut files,
        &format!("style add \"Other\" font={font}"),
    );
    refused(
        &mut s,
        &mut files,
        &format!("style set \"Body\" font={font}"),
    );
}

#[test]
fn a_style_option_on_a_named_area_makes_it_custom() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let body = s.document().styles["Body"].clone();
    ok(&mut s, &mut files, "text n3 \"x\" size=64");
    assert_eq!(
        text_of(&s, 3).style,
        StyleChoice::Custom(TextStyle {
            size: 64.0,
            ..body.clone()
        })
    );
    assert_eq!(s.document().styles["Body"], body);
    // `style=` names a style again; `custom` copies the values drawn.
    ok(&mut s, &mut files, "text n3 \"x\" style=\"Legend\"");
    assert_eq!(text_of(&s, 3).style, named("Legend"));
    ok(&mut s, &mut files, "text n3 \"x\" custom");
    assert_eq!(
        text_of(&s, 3).style,
        StyleChoice::Custom(s.document().styles["Legend"].clone())
    );
}

#[test]
fn a_new_area_takes_the_split_area_s_style_or_body() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let new = |reply: String| -> u64 {
        reply.split_whitespace().next().unwrap()["new=n".len()..]
            .parse()
            .unwrap()
    };
    let a = new(ok(&mut s, &mut files, "split n2 beside 0.5"));
    assert_eq!(text_of(&s, a).style, named("Title"));
    let b = new(ok(&mut s, &mut files, "split n6 stacked 0.5"));
    assert_eq!(text_of(&s, b).style, named("Body"));
    // With no Body in the table, the default values.
    ok(&mut s, &mut files, "style remove \"Body\" into=custom");
    let reply = ok(&mut s, &mut files, "add-slide");
    let root = reply.split("root=n").nth(1).unwrap().parse().unwrap();
    assert_eq!(
        text_of(&s, root).style,
        StyleChoice::Custom(TextStyle::default())
    );
}

/// The undo of a style's addition removes it again, though texts loaded with it name it.
#[test]
fn an_undo_of_a_style_s_addition_is_not_refused() {
    let mut files = MemoryFiles::default();
    let mut s = with_heading(&mut files);
    let loaded = s.document().clone();
    ok(&mut s, &mut files, "style add \"Heading\" size=90");
    assert_eq!(s.document().breaches(), vec![]);
    ok(&mut s, &mut files, "undo");
    assert_eq!(s.document(), &loaded);
    assert_eq!(text_of(&s, 2).style, named("Heading"));
}

#[test]
fn a_rename_onto_a_name_an_area_holds_is_refused() {
    let mut files = MemoryFiles::default();
    let mut s = with_heading(&mut files);
    assert_eq!(
        refused(&mut s, &mut files, "style rename \"Body\" \"Heading\""),
        "n2 names \"Heading\""
    );
    refused(&mut s, &mut files, "style rename \"Body\" \"Title\"");
}

/// A condition awaiting the owner, `issue@rootpoint-model@new-area-style-awaits-rulings`.
#[test]
fn a_split_does_not_copy_a_missing_name() {
    let mut files = MemoryFiles::default();
    let mut s = with_heading(&mut files);
    let reply = ok(&mut s, &mut files, "split n2 beside 0.5");
    let new: u64 = reply.split_whitespace().next().unwrap()["new=n".len()..]
        .parse()
        .unwrap();
    assert_eq!(text_of(&s, new).style, named("Body"));
}

#[test]
fn a_name_with_a_space_reads_and_prints_back() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    for line in [
        "style add \"Style 1\" size=40",
        "text n3 \"x\" style=\"Style 1\"",
        "style rename \"Style 1\" \"Style \\\"2\\\"\"",
        "style remove \"Style \\\"2\\\"\" into=\"Body\"",
    ] {
        let command = parse_command(line).unwrap();
        assert_eq!(
            parse_command(&command.to_string()),
            Ok(command.clone()),
            "{line}"
        );
        assert!(matches!(s.run(command, &mut files), Reply::Ok(_)), "{line}");
    }
    assert_eq!(text_of(&s, 3).style, named("Body"));
    assert_eq!(
        parse_command("text n3 \"x\" style=\"Style 1\""),
        Ok(Command::Text {
            area: NodeId(3),
            text: "x".into(),
            style: vec![rootpoint_protocol::StyleOption::Named("Style 1".into())],
        })
    );
}

/// `styles` lists each style with every value, as the options `style add` takes, so each part reads
/// back as a `style add`.
#[test]
fn styles_lists_each_style_as_its_options() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    let listed = ok(&mut s, &mut files, "styles");
    assert!(
        listed.starts_with(
            "\"Body\" size=40 bold=off italic=off colour=#000000 align=left valign=top font=sans; \"Legend\""
        ),
        "{listed}"
    );
    let mut fresh = Session::new();
    for part in listed.split("; ") {
        ok(
            &mut fresh,
            &mut files,
            &format!("style remove {}", part.split(' ').next().unwrap()),
        );
        ok(&mut fresh, &mut files, &format!("style add {part}"));
    }
    assert_eq!(fresh.document().styles, s.document().styles);
}

/// A file's Body whose font the document lacks is no style a new area takes: it takes the default
/// values, so no command makes a breach in a document that has none; a condition awaiting the
/// owner, `issue@rootpoint-model@new-area-style-awaits-rulings`.
#[test]
fn a_new_area_takes_no_body_that_would_draw_with_a_stand_in() {
    let mut files = MemoryFiles::default();
    let mut s = Session::new();
    run(
        &mut s,
        &mut files,
        "asset font.ttf\nstyle set \"Body\" font=a1\nsave deck.ron",
    );
    let text = String::from_utf8(files.files["deck.ron"].clone()).unwrap();
    let table = "        1: (\n            source: Path(\"font.ttf\"),\n        ),\n";
    let edited = text.replacen(table, "", 1);
    assert_ne!(edited, text, "the saved file changed shape");
    files.files.insert("deck.ron".into(), edited.into_bytes());
    ok(&mut s, &mut files, "load deck.ron");
    assert_eq!(s.document().breaches(), vec![]);
    ok(&mut s, &mut files, "add-slide");
    assert_eq!(s.document().breaches(), vec![]);
    assert_eq!(
        text_of(&s, 3).style,
        StyleChoice::Custom(TextStyle::default())
    );
}

/// A style of a file whose built-in family the binary does not carry is no style a text may name.
#[test]
fn no_command_names_a_style_of_an_unknown_family() {
    let mut files = MemoryFiles::default();
    let mut s = sample(&mut files);
    ok(&mut s, &mut files, "save deck.ron");
    let text = String::from_utf8(files.files["deck.ron"].clone()).unwrap();
    let legend = "        \"Legend\": (\n            font: Builtin(\"sans\"),";
    let edited = text.replacen(
        legend,
        "        \"Legend\": (\n            font: Builtin(\"serif\"),",
        1,
    );
    assert_ne!(edited, text, "the saved sample changed shape");
    files.files.insert("deck.ron".into(), edited.into_bytes());
    ok(&mut s, &mut files, "load deck.ron");
    assert_eq!(
        refused(&mut s, &mut files, "text n3 \"x\" style=\"Legend\""),
        "\"Legend\" names no built-in family serif"
    );
    refused(&mut s, &mut files, "style remove \"Body\" into=\"Legend\"");
}
