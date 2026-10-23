//! Over random sequences of commands on a headless session, `undo`, `redo` and the groups
//! included, the document stays valid, holds no cosmetic breach, and saves to a text that loads
//! back equal. A breach reaches a document only by a file edited by hand, never by a command,
//! `design@rootpoint-protocol@no-new-breach`.

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
        0u8..26,
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
        13 => format!("clear {}", node(s.pick)),
        14 => format!("asset image{}.png", s.other % 5),
        15 => format!("remove-asset {}", asset(s.other)),
        16 => "undo".to_owned(),
        17 => "redo".to_owned(),
        18 => ["begin", "end", "cancel"][s.other % 3].to_owned(),
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
        _ => format!("select {}", node(s.pick)),
    }
}

proptest! {
    /// No sequence of commands leaves a breach, breaks an invariant, or saves a document that
    /// loads back otherwise.
    #[test]
    fn no_sequence_of_commands_creates_a_breach(steps in prop::collection::vec(step(), 1..60)) {
        let mut files = MemoryFiles::default();
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
