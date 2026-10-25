//! Libertinus Math 7.051 read through `MathFont`, and the table of commands generated against it.

mod common;

use std::collections::BTreeSet;

use rootpoint_math::{MathFont, VerticalConstruction, command, commands};

/// The 44 math commands of KaTeX v0.19.1 of one character whose character Libertinus Math 7.051
/// lacks, as section 4 of `path@rootpoint@docs/grounding/math-typesetting.md` lists them.
const MISSING: [&str; 44] = [
    "lessapprox",
    "gtrapprox",
    "lnapprox",
    "gnapprox",
    "eqslantless",
    "eqslantgtr",
    "lesseqqgtr",
    "gtreqqless",
    "precapprox",
    "succapprox",
    "precnapprox",
    "succnapprox",
    "subseteqq",
    "supseteqq",
    "subsetneqq",
    "supsetneqq",
    "Subset",
    "Supset",
    "bumpeq",
    "Bumpeq",
    "pitchfork",
    "multimap",
    "Cap",
    "Cup",
    "doublecap",
    "doublecup",
    "barwedge",
    "veebar",
    "doublebarwedge",
    "divideontimes",
    "leftthreetimes",
    "rightthreetimes",
    "intercal",
    "diamond",
    "triangleleft",
    "triangleright",
    "bigcirc",
    "blacklozenge",
    "checkmark",
    "maltese",
    "diagup",
    "diagdown",
    "Finv",
    "Game",
];

#[test]
fn the_math_table_reads_as_the_survey_measured() {
    let font = common::font();
    assert_eq!(font.script_percent_scale_downs(), (80, 60));
    assert_eq!(
        font.vertical_construction('('),
        Some(VerticalConstruction {
            variants: 13,
            assembly_parts: Some(3)
        })
    );
}

#[test]
fn a_font_with_no_math_table_is_refused() {
    let sans = include_bytes!("../../rootpoint-render/fonts/open-sans/OpenSans-Regular.ttf");
    assert!(MathFont::new(&sans[..]).is_err());
    assert!(MathFont::new(&b"not a font"[..]).is_err());
}

#[test]
fn a_command_means_what_it_means_in_latex() {
    for (name, ch) in [
        ("alpha", 'α'),
        ("epsilon", 'ϵ'),
        ("varepsilon", 'ε'),
        ("phi", 'ϕ'),
        ("varphi", 'φ'),
        ("le", '≤'),
        ("neq", '≠'),
        ("to", '→'),
        ("hbar", 'ℏ'),
    ] {
        assert_eq!(command(name).map(|c| c.ch), Some(ch), "\\{name}");
    }
}

/// The KaTeX commands of one character, from the fixture.
fn katex() -> Vec<(String, char)> {
    include_str!("data/katex-symbols.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let (name, code) = l.split_once(" U+").unwrap();
            let ch = char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap();
            (name.to_owned(), ch)
        })
        .collect()
}

/// The commands whose character the font lacks are exactly the 44, so a change of the font or of
/// its version that changes its coverage of these commands fails here; and the table holds none of
/// their characters.
#[test]
fn the_missing_set_is_pinned_both_ways() {
    let font = common::font();
    let all = katex();
    assert_eq!(all.len(), 457);
    let missing: BTreeSet<&str> = all
        .iter()
        .filter(|(_, ch)| font.glyph(*ch).is_none())
        .map(|(n, _)| n.as_str())
        .collect();
    let expected: BTreeSet<&str> = MISSING.into_iter().collect();
    assert_eq!(missing, expected);
    for (name, ch) in all.iter().filter(|(n, _)| expected.contains(n.as_str())) {
        assert!(
            commands().iter().all(|c| c.ch != *ch),
            "\\{name}'s character is in the table"
        );
    }
}

#[test]
fn the_table_holds_only_characters_the_font_has() {
    let font = common::font();
    for c in commands() {
        assert!(font.glyph(c.ch).is_some(), "\\{}", c.name);
    }
}
