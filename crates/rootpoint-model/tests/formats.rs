//! Every version of the save format from the oldest that loads keeps loading, to the same
//! presentation, per `design@rootpoint@saved-presentations-stay-loadable`.
//!
//! In `path@rootpoint-model@tests/data/formats/`, `v<N>.ron` is a file of version N, frozen: it is
//! written once, when version N is the current one, and never rewritten, since it stands for the
//! presentations saved at that version. `v<N>.saved.ron` is what the current version saves of it,
//! which `ROOTPOINT_BLESS=1` rewrites after a deliberate change of the format, for every version at
//! once. A breaking change adds both files of its new version, and a frozen pack of it,
//! `path@rootpoint-protocol@tests/formats.rs`.

use std::path::PathBuf;

use rootpoint_model::*;

fn formats(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/formats")
        .join(name)
}

fn frozen(version: u32) -> String {
    let path = formats(&format!("v{version}.ron"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}; a new version of the format freezes a file of itself here",
            path.display()
        )
    })
}

/// AC1 of the premortem on compatibility: a file of each version from `OLDEST_VERSION` loads, so a
/// change of the format with no migration of the version before fails here.
#[test]
fn a_frozen_file_of_every_version_loads() {
    for version in OLDEST_VERSION..=FORMAT_VERSION {
        if let Err(e) = load_ron(&frozen(version)) {
            panic!("the frozen file of version {version} does not load: {e:?}");
        }
    }
}

/// AC2 of the premortem on compatibility: a frozen file loads to the presentation it held, so a
/// change that loads an old file to another meaning shows here, as a change of the saved text the
/// change's commit has to bless.
#[test]
fn a_frozen_file_saves_as_its_committed_text() {
    for version in OLDEST_VERSION..=FORMAT_VERSION {
        let saved = save_ron(&load_ron(&frozen(version)).unwrap());
        let path = formats(&format!("v{version}.saved.ron"));
        if std::env::var_os("ROOTPOINT_BLESS").is_some() {
            std::fs::write(&path, &saved).unwrap();
        }
        let committed = std::fs::read_to_string(&path).expect("the committed saved text");
        assert_eq!(
            committed,
            saved,
            "the frozen file of version {version} now saves as another text; check the migration, then bless {}",
            path.display()
        );
    }
}

/// AC3 of the premortem on compatibility: the frozen file of the current version holds every part
/// of the format, so a change of any part meets a file that holds it.
#[test]
fn the_newest_frozen_file_holds_every_part_of_the_format() {
    let doc = load_ron(&frozen(FORMAT_VERSION)).unwrap();
    let mut items = Vec::new();
    let mut contents = Vec::new();
    let mut nested_split = false;
    for slide in &doc.slides {
        if let Node::Split(split) = &slide.root {
            nested_split |= split.children.iter().any(|c| matches!(c, Node::Split(_)));
        }
        for area in slide.root.areas() {
            contents.push(&area.content);
            if let Content::Text(text) = &area.content {
                collect(&text.root, &mut items);
            }
        }
    }
    let texts = || {
        contents.iter().filter_map(|c| match c {
            Content::Text(t) => Some(t),
            _ => None,
        })
    };
    let has = |name: &str, found: bool| assert!(found, "the frozen file holds no {name}");
    has("nested split", nested_split);
    has(
        "named text style",
        texts().any(|t| matches!(t.style, StyleChoice::Named(_))),
    );
    has(
        "custom text style",
        texts().any(|t| matches!(t.style, StyleChoice::Custom(_))),
    );
    has(
        "image fitted",
        contents
            .iter()
            .any(|c| matches!(c, Content::Image(i) if i.fit == ImageFit::Fit)),
    );
    has(
        "image filling",
        contents
            .iter()
            .any(|c| matches!(c, Content::Image(i) if i.fit == ImageFit::Fill)),
    );
    has(
        "numbered equation",
        contents
            .iter()
            .any(|c| matches!(c, Content::Equation(e) if e.numbered)),
    );
    has(
        "unnumbered equation",
        contents
            .iter()
            .any(|c| matches!(c, Content::Equation(e) if !e.numbered)),
    );
    has(
        "equation fitted",
        contents
            .iter()
            .any(|c| matches!(c, Content::Equation(e) if e.size == EquationSize::Fit)),
    );
    has(
        "equation at most a size",
        contents.iter().any(
            |c| matches!(c, Content::Equation(e) if matches!(e.size, EquationSize::AtMost(_))),
        ),
    );
    has(
        "named span",
        items
            .iter()
            .any(|i| matches!(i, TextItem::Span(s) if matches!(s.style, SpanChoice::Named(_)))),
    );
    has(
        "custom span",
        items
            .iter()
            .any(|i| matches!(i, TextItem::Span(s) if matches!(s.style, SpanChoice::Custom(_)))),
    );
    has("span inside a span", items.iter().any(|i| matches!(i, TextItem::Span(s) if s.text.items.iter().any(|j| matches!(j, TextItem::Span(_))))));
    has(
        "inline formula",
        items.iter().any(|i| matches!(i, TextItem::Math(_))),
    );
    has(
        "reference",
        items.iter().any(|i| matches!(i, TextItem::Ref(_))),
    );
    has("span style", !doc.span_styles.is_empty());
    has(
        "font asset",
        doc.styles
            .values()
            .any(|s| matches!(s.font, FontRef::Asset(_))),
    );
}

fn collect<'a>(node: &'a TextNode, out: &mut Vec<&'a TextItem>) {
    for item in &node.items {
        out.push(item);
        if let TextItem::Span(span) = item {
            collect(&span.text, out);
        }
    }
}
