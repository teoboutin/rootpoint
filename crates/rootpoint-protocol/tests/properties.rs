//! Over random sequences of commands on a headless session, `undo`, `redo`, the groups, imports,
//! copies of a directory, the asset commands, the commands of a text tree and of the span style
//! table included, the document stays valid, holds no
//! cosmetic breach, an asset path that leads out of the root among them, and saves to a text that
//! loads back equal. A
//! breach reaches a document only by a file edited by hand, never by a command,
//! `design@rootpoint-protocol@no-new-breach`; and the commands that give an asset a new path,
//! `asset-move` and `asset-relink`, refuse one that fails confinement, so a path an undo or a redo
//! brings back stays confined too.

mod common;

use common::*;
use proptest::prelude::*;
use rootpoint_model::*;
use rootpoint_protocol::Session;

/// One step of a random sequence: a kind of command, and numbers that pick its targets among the
/// ids of the document at the time it runs.
#[derive(Clone, Debug)]
struct Step {
    kind: u8,
    pick: usize,
    other: usize,
    at: f64,
    flag: bool,
}

fn step() -> impl Strategy<Value = Step> {
    (
        0u8..50,
        any::<usize>(),
        any::<usize>(),
        0.05f64..0.95,
        any::<bool>(),
    )
        .prop_map(|(kind, pick, other, at, flag)| Step {
            kind,
            pick,
            other,
            at,
            flag,
        })
}

/// The names a style command picks from: two of the predefined styles, two new ones, and one with a
/// space.
const STYLE_NAMES: [&str; 5] = ["Title", "Body", "Quote", "Note", "Style 1"];

/// The names a span style command picks from: two the table may hold, and one with a space.
const SPAN_STYLE_NAMES: [&str; 3] = ["Term", "Key", "Key term"];

/// Every span of every text of the document.
fn spans(doc: &Document) -> Vec<SpanId> {
    doc.slides
        .iter()
        .flat_map(|s| s.root.areas())
        .flat_map(|a| match &a.content {
            Content::Text(t) => t.root.spans().iter().map(|s| s.id).collect(),
            Content::Image(_) | Content::Equation(_) => Vec::new(),
        })
        .collect()
}

/// Every id the document holds, of any kind.
fn ids(doc: &Document) -> Vec<u64> {
    fn walk(node: &Node, out: &mut Vec<u64>) {
        out.push(node.id().0);
        if let Node::Split(s) = node {
            s.children.iter().for_each(|c| walk(c, out));
        }
    }
    let mut out: Vec<u64> = doc.assets.keys().map(|a| a.0).collect();
    for slide in &doc.slides {
        out.push(slide.id.0);
        walk(&slide.root, &mut out);
    }
    out
}

/// The command line a step runs on this document. An asset is picked among every id up to two
/// past the largest the document holds, so that a command often names an id that is no asset:
/// a node's, a removed asset's, or one the counter has not reached.
fn line_for(doc: &Document, s: &Step) -> String {
    let all = ids(doc);
    let nodes: Vec<NodeId> = doc
        .slides
        .iter()
        .flat_map(|sl| {
            let mut v = Vec::new();
            fn walk(n: &Node, v: &mut Vec<NodeId>) {
                v.push(n.id());
                if let Node::Split(sp) = n {
                    sp.children.iter().for_each(|c| walk(c, v));
                }
            }
            walk(&sl.root, &mut v);
            v
        })
        .collect();
    let node = |k: usize| {
        nodes
            .get(k % nodes.len().max(1))
            .map_or("n1".to_owned(), |n| n.to_string())
    };
    let slide = |k: usize| {
        doc.slides
            .get(k % doc.slides.len().max(1))
            .map_or("s1".to_owned(), |sl| sl.id.to_string())
    };
    let asset = |k: usize| format!("a{}", 1 + k as u64 % (all.iter().max().unwrap_or(&0) + 2));
    let style = |k: usize| format!("\"{}\"", STYLE_NAMES[k % STYLE_NAMES.len()]);
    let span_style = |k: usize| format!("\"{}\"", SPAN_STYLE_NAMES[k % SPAN_STYLE_NAMES.len()]);
    // A span is picked among the document's, or an id that names none.
    let all_spans = spans(doc);
    let span = |k: usize| match all_spans.get(k % (all_spans.len() + 1)) {
        Some(t) => t.to_string(),
        None => format!("t{}", k % 50),
    };
    // A text node: an area's root, or a span.
    let target = |k: usize| {
        if k.is_multiple_of(2) {
            node(k / 2)
        } else {
            span(k / 2)
        }
    };
    // A span's options: values, a named span style the table may lack, a font asset it may lack,
    // and an inherited value, which leaves a new span setting nothing.
    let span_options = |k: usize| match k % 5 {
        0 => "bold".to_owned(),
        1 => format!("style={}", span_style(k / 5)),
        2 => format!("font={} size=20", asset(k / 5)),
        3 => "italic=inherit".to_owned(),
        _ => "colour=#ff000080 custom".to_owned(),
    };
    let axis = if s.flag { "beside" } else { "stacked" };
    match s.kind {
        0 => "add-slide".to_owned(),
        1 => format!("remove-slide {}", slide(s.pick)),
        2 => format!("move-slide {} {}", slide(s.pick), s.other % 3),
        3 => format!("duplicate-slide {}", slide(s.pick)),
        4 | 5 => format!("split {} {axis} {} after", node(s.pick), s.at),
        6 => format!("remove {}", node(s.pick)),
        7 => format!("divider {} 0 {}", node(s.pick), s.at),
        8 => format!("swap {} {}", node(s.pick), node(s.other)),
        9 => format!(
            "text {} \"t{}\" size={}",
            node(s.pick),
            s.other % 10,
            10 + s.other % 90
        ),
        10 => format!("text {} \"f\" font={}", node(s.pick), asset(s.other)),
        11 => format!("text {} \"s\" font=sans bold", node(s.pick)),
        12 => format!(
            "image {} {} {}",
            node(s.pick),
            asset(s.other),
            if s.flag { "fit" } else { "fill" }
        ),
        // An equation, set or changed, with an option or none, as often as a clear.
        13 if s.flag => format!(
            "equation {} $x^{{{}}}$ {}",
            node(s.pick),
            s.other % 10,
            ["", "numbered=no", "size=fit", "size=30", "colour=#ff0000"][s.other % 5]
        ),
        13 => format!("clear {}", node(s.pick)),
        // Five files of other bytes, imported at the root or under a folder; the folder `../` is
        // refused, and leaves nothing.
        14 => match s.other % 3 {
            0 => format!("import image{}.png", s.other % 5),
            1 => format!("import image{}.png img/i{}.png", s.other % 5, s.pick % 3),
            _ => format!("import image{}.png ../i.png", s.other % 5),
        },
        15 => match s.flag {
            true => format!("asset-remove {}", asset(s.other)),
            false => format!("asset-remove {} into={}", asset(s.other), asset(s.pick)),
        },
        16 => "undo".to_owned(),
        17 => "redo".to_owned(),
        18 => ["begin", "end", "cancel"][s.other % 3].to_owned(),
        19 => "open-copy deck".to_owned(),
        // A style is picked among names the table holds, names it lacks, and one with a space.
        20 => format!("text {} \"n\" style={}", node(s.pick), style(s.other)),
        21 => format!("text {} \"c\" custom", node(s.pick)),
        22 => format!(
            "style add {} size={} italic",
            style(s.pick),
            10 + s.other % 90
        ),
        23 => format!("style set {} bold font={}", style(s.pick), asset(s.other)),
        24 => format!("style rename {} {}", style(s.pick), style(s.other)),
        25 => match s.other % 3 {
            0 => format!("style remove {}", style(s.pick)),
            1 => format!("style remove {} into=custom", style(s.pick)),
            _ => format!("style remove {} into={}", style(s.pick), style(s.other / 3)),
        },
        // A new path among free ones, ones a file or an asset holds, and ones that fail
        // confinement.
        26 => format!("asset-move {} {}", asset(s.other), new_path(s.pick)),
        27 => format!("asset-relink {} {}", asset(s.other), new_path(s.pick)),
        28 => format!("asset-replace {} image{}.png", asset(s.other), s.pick % 5),
        29 => "asset-remove-unused".to_owned(),
        30 => [
            "assets",
            "assets unused",
            "assets missing",
            "assets untracked",
        ][s.other % 4]
            .to_owned(),
        31 => format!(
            "text {} [\"ab \", {{{} \"cd\", {{italic \"e\"}}}}, \" f\"]",
            node(s.pick),
            span_options(s.other)
        ),
        32 => format!("insert {} {} \"x\"", target(s.pick), s.other % 6),
        33 => format!(
            "delete {} {}..{}",
            target(s.pick),
            s.other % 4,
            s.other % 4 + s.pick % 3
        ),
        34 => format!(
            "wrap {} {}..{} {}",
            target(s.pick),
            s.other % 3,
            s.other % 3 + s.pick % 3,
            span_options(s.other / 3)
        ),
        35 => format!("unwrap {}", span(s.pick)),
        36 => format!("span {} {}", span(s.pick), span_options(s.other)),
        37 => format!("split-span {} {}", span(s.pick), s.other % 4),
        38 => format!(
            "span-style add {} {}",
            span_style(s.pick),
            span_options(s.other)
        ),
        39 => format!(
            "span-style set {} bold font={}",
            span_style(s.pick),
            asset(s.other)
        ),
        40 => format!(
            "span-style rename {} {}",
            span_style(s.pick),
            span_style(s.other)
        ),
        41 => match s.other % 3 {
            0 => format!("span-style remove {}", span_style(s.pick)),
            1 => format!("span-style remove {} into=custom", span_style(s.pick)),
            _ => format!(
                "span-style remove {} into={}",
                span_style(s.pick),
                span_style(s.other / 3)
            ),
        },
        42 => format!(
            "text {} {}",
            node(s.pick),
            ["bold", "style=\"Title\"", "custom"][s.other % 3]
        ),
        43 => "span-styles".to_owned(),
        // Formulas and references, to areas that hold an equation or not, which the refusals of
        // a reference to an area that holds no equation meet.
        44 => format!(
            "insert {} {} $x_{}$",
            target(s.pick),
            s.other % 4,
            s.other % 10
        ),
        45 => format!(
            "insert {} {} {{ref {}}}",
            target(s.pick),
            s.other % 4,
            node(s.other)
        ),
        46 => format!("math {} {} $y$", target(s.pick), s.other % 4),
        47 => format!("retarget {} {}", node(s.pick), node(s.other)),
        48 => format!(
            "text {} [\"a \", $z$, {{ref {}}}]",
            node(s.pick),
            node(s.other)
        ),
        _ => format!("select {}", node(s.pick)),
    }
}

/// A path an asset command gives an asset.
fn new_path(k: usize) -> String {
    match k % 6 {
        0 => format!("img/m{}.png", k % 4),
        1 => format!("image{}.png", k % 5),
        2 => format!("img/i{}.png", k % 3),
        3 => "../m.png".to_owned(),
        4 => ".rootpoint/m.png".to_owned(),
        5 if k % 12 < 6 => "mimetype".to_owned(),
        _ => "presentation.ron".to_owned(),
    }
}

proptest! {
    /// No sequence of commands leaves a breach, breaks an invariant, or saves a document that
    /// loads back otherwise.
    #[test]
    fn no_sequence_of_commands_creates_a_breach(steps in prop::collection::vec(step(), 1..60)) {
        let mut files = MemoryFiles::holding(&[
            "image0.png", "image1.png", "image2.png", "image3.png", "image4.png",
        ]);
        // The directory `open-copy` copies: a presentation of one image, its file beside it.
        let mut deck = Session::new();
        run(&mut deck, &mut files, "add-slide\nimport image0.png\nimage n2 a3 fit");
        write_presentation(&deck, &mut files, "deck");
        files.files.insert("deck/image0.png".into(), b"image0.png".to_vec());
        let mut session = Session::new();
        let mut script = Vec::new();
        for s in &steps {
            let line = line_for(session.document(), s);
            session.execute(&line, &mut files);
            script.push(line);
            let doc = session.document();
            prop_assert_eq!(doc.validate(), Ok(()), "script:\n{}", script.join("\n"));
            prop_assert_eq!(doc.breaches(), vec![], "script:\n{}", script.join("\n"));
            let loaded = load_ron(&save_ron(doc));
            prop_assert_eq!(
                loaded.as_ref(),
                Ok(doc),
                "script:\n{}",
                script.join("\n")
            );
        }
    }
}
