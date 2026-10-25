//! `cargo x math-commands`: the table of commands of the math crate, generated from W3C
//! `unicode.xml` and the math font, with the W3C notice beside it.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

/// LaTeX's aliases that no column of `unicode.xml` gives, each with the command it stands for:
/// the `\let`s of `path@elsewhere@base/fontdef.dtx`, and `\neq` and `\iff`, which its macros
/// define as `\not=` and as `\Longleftrightarrow` between spaces.
const ALIASES: [(&str, &str); 10] = [
    ("ge", "geq"),
    ("gets", "leftarrow"),
    ("iff", "Longleftrightarrow"),
    ("land", "wedge"),
    ("le", "leq"),
    ("lnot", "neg"),
    ("lor", "vee"),
    ("neq", "ne"),
    ("owns", "ni"),
    ("to", "rightarrow"),
];

/// LaTeX's commands of one character that `unicode.xml` names otherwise, each with its code point:
/// `\emptyset`, a symbol of `path@elsewhere@base/fontdef.dtx`, which the file names `\varnothing`.
const SYMBOLS: [(&str, u32); 1] = [("emptyset", 0x2205)];

/// The delimiters LaTeX declares an open or a close that `unicode.xml` types otherwise, each with
/// its class, as `path@elsewhere@base/fontdef.dtx` declares `\langle` and `\rangle`, `\lgroup` and
/// `\rgroup`, `\lmoustache` and `\rmoustache`.
const FENCES: [(u32, &str); 6] = [
    (0x27E8, "Open"),
    (0x27E9, "Close"),
    (0x27EE, "Open"),
    (0x27EF, "Close"),
    (0x23B0, "Open"),
    (0x23B1, "Close"),
];

/// The prefix of unicode-math's names of an upright letter, as `\mupalpha`, which the math crate
/// does not take: its letters are set italic, as LaTeX sets them, and `\mathrm` sets one upright.
const UPRIGHT_PREFIX: &str = "mup";

/// The names the math crate reads as structures, which are not symbols of the table.
const STRUCTURES: &[&str] = &[
    "frac",
    "sqrt",
    "left",
    "right",
    "begin",
    "end",
    "text",
    "acute",
    "bar",
    "breve",
    "check",
    "ddot",
    "dot",
    "grave",
    "hat",
    "overline",
    "tilde",
    "underline",
    "vec",
    "mathbb",
    "mathbf",
    "mathcal",
    "mathit",
    "mathrm",
    "quad",
    "qquad",
];

/// A character of the file: its code point, its `type`, its mode and its names by column.
#[derive(Default)]
struct Character {
    code: Option<u32>,
    kind: Option<String>,
    text_mode: bool,
    names: Vec<(usize, String)>,
}

/// Write `path@rootpoint-math@src/commands.rs` and `path@rootpoint-math@W3C-NOTICE.txt`, under
/// `root`, from the `unicode.xml` at `source` and the font at `font`: the commands of the
/// characters the font has, and apart those of the characters it lacks. `revision` and `date` are written in the notice, which the W3C licence asks of a
/// derived file.
pub fn generate(
    root: &Path,
    source: &Path,
    font: &Path,
    revision: &str,
    date: &str,
) -> Result<(usize, usize, usize), String> {
    let xml = std::fs::read_to_string(source).map_err(|e| format!("{}: {e}", source.display()))?;
    let bytes = std::fs::read(font).map_err(|e| format!("{}: {e}", font.display()))?;
    let face =
        ttf_parser::Face::parse(&bytes, 0).map_err(|e| format!("{}: {e}", font.display()))?;
    let has = |cp: u32| {
        char::from_u32(cp)
            .and_then(|c| face.glyph_index(c))
            .is_some()
    };

    let (header, characters) = read_characters(&xml)?;
    // Each name goes to the character of its first column; among characters of one column, to
    // the lowest code point.
    let mut best: BTreeMap<String, (usize, u32)> = BTreeMap::new();
    let mut classes: BTreeMap<u32, &'static str> = BTreeMap::new();
    for c in &characters {
        let Some(code) = c.code else { continue };
        let class = FENCES
            .iter()
            .find(|(f, _)| *f == code)
            .map_or_else(|| class_of(c.kind.as_deref()), |(_, class)| *class);
        if class != "Ord" {
            classes.insert(code, class);
        }
        for (column, name) in &c.names {
            if *column == 4 && c.text_mode {
                continue;
            }
            if STRUCTURES.contains(&name.as_str())
                || OPERATOR_NAMES.contains(&name.as_str())
                || name.starts_with(UPRIGHT_PREFIX)
            {
                continue;
            }
            let entry = best.entry(name.clone()).or_insert((*column, code));
            if (*column, code) < *entry {
                *entry = (*column, code);
            }
        }
    }
    let mut table: BTreeMap<String, u32> = best.into_iter().map(|(n, (_, c))| (n, c)).collect();
    for (alias, target) in ALIASES {
        let code = *table
            .get(target)
            .ok_or_else(|| format!("the alias \\{alias} names \\{target}, not in the table"))?;
        table.insert(alias.to_owned(), code);
    }
    for (name, code) in SYMBOLS {
        table.insert(name.to_owned(), code);
    }
    // The commands of a character the font lacks are read, and drawn as the font's missing glyph,
    // but the table the palette lists holds none of them.
    let (present, absent): (BTreeMap<String, u32>, BTreeMap<String, u32>) =
        table.into_iter().partition(|(_, code)| has(*code));
    classes.retain(|code, _| has(*code) || absent.values().any(|c| c == code));
    let class_by_code =
        |code: u32| -> &'static str { classes.get(&code).copied().unwrap_or("Ord") };

    let mut out = String::new();
    out.push_str(
        "//! The table of commands, generated by `cargo x math-commands` from W3C `unicode.xml` and\n\
         //! the math font, with the notice of `path@rootpoint-math@W3C-NOTICE.txt`. Not edited by hand.\n\n\
         use crate::{Class, Command};\n\n\
         /// Every command name, without its backslash, with its character and its class, sorted by\n\
         /// name: only the characters the math font has.\n\
         #[rustfmt::skip]\n\
         pub(crate) const COMMANDS: &[Command] = &[\n",
    );
    let write = |out: &mut String, table: &BTreeMap<String, u32>| {
        for (name, code) in table {
            let class = class_by_code(*code);
            writeln!(
                out,
                "    Command {{ name: {name:?}, ch: '\\u{{{code:x}}}', class: Class::{class} }},"
            )
            .unwrap();
        }
    };
    write(&mut out, &present);
    out.push_str(
        "];\n\n\
         /// The commands of a character the math font lacks, sorted by name: read, and drawn as the\n\
         /// font's missing glyph.\n\
         #[rustfmt::skip]\n\
         pub(crate) const ABSENT: &[Command] = &[\n",
    );
    write(&mut out, &absent);
    out.push_str(
        "];\n\n\
         /// The class of each character the font has, or of a command of `ABSENT`, that `unicode.xml`,\n\
         /// or LaTeX for a fence, gives a class other than ord, sorted by character.\n\
         #[rustfmt::skip]\n\
         pub(crate) const CLASSES: &[(char, Class)] = &[\n",
    );
    for (code, class) in &classes {
        writeln!(out, "    ('\\u{{{code:x}}}', Class::{class}),").unwrap();
    }
    out.push_str("];\n");

    let crate_dir = root.join("crates/rootpoint-math");
    std::fs::write(crate_dir.join("src/commands.rs"), out).map_err(|e| e.to_string())?;
    let notice = format!(
        "The table of commands of this crate, `src/commands.rs`, is derived from `unicode.xml`,\n\
         {revision}, of the repository https://github.com/w3c/xml-entities.\n\
         On {date}, `cargo x math-commands` of rootpoint's xtask extracted from it the LaTeX names,\n\
         the code point and the type of each character, split them by whether the math font has the\n\
         character, and added a list of LaTeX's own aliases. The file's notice, verbatim:\n\n\
         {header}\n\n{license}",
        header = header.trim(),
        license = include_str!("w3c-software-license.txt"),
    );
    std::fs::write(crate_dir.join("W3C-NOTICE.txt"), notice).map_err(|e| e.to_string())?;
    Ok((present.len(), absent.len(), classes.len()))
}

/// The operator names, which the math crate reads as atoms of their own.
const OPERATOR_NAMES: &[&str] = &[
    "arccos", "arcsin", "arctan", "arg", "cos", "cosh", "cot", "coth", "csc", "deg", "det", "dim",
    "exp", "gcd", "hom", "inf", "ker", "lg", "lim", "liminf", "limsup", "ln", "log", "max", "min",
    "Pr", "sec", "sin", "sinh", "sup", "tan", "tanh",
];

/// The class of a character of `type`, by the milestone's mapping table of the types.
fn class_of(kind: Option<&str>) -> &'static str {
    match kind {
        Some("relation") => "Rel",
        Some("binaryop") => "Bin",
        Some("large") => "Op",
        Some("opening") => "Open",
        Some("closing") => "Close",
        Some("punctuation") => "Punct",
        _ => "Ord",
    }
}

/// The file's leading comment, and each `<character>` of one code point.
fn read_characters(xml: &str) -> Result<(String, Vec<Character>), String> {
    let mut reader = Reader::from_str(xml);
    let mut header = String::new();
    let mut characters = Vec::new();
    let mut current: Option<Character> = None;
    let mut column: Option<usize> = None;
    let mut text = String::new();
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Comment(c) if header.is_empty() && characters.is_empty() => {
                header = c.decode().map_err(|e| e.to_string())?.into_owned();
            }
            Event::Start(e) => {
                let name = e.local_name();
                if name.as_ref() == b"character" {
                    let mut c = Character::default();
                    for a in e.attributes().flatten() {
                        let value = a
                            .normalized_value(XmlVersion::default())
                            .map_err(|e| e.to_string())?;
                        match a.key.as_ref() {
                            b"id" => c.code = single_code_point(&value),
                            b"type" => c.kind = Some(value.into_owned()),
                            b"mode" => c.text_mode = value == "text",
                            _ => {}
                        }
                    }
                    current = Some(c);
                } else if current.is_some() {
                    // The columns that name a LaTeX command, numbered in the order that decides a
                    // name two characters claim: a name means what it means in LaTeX's math mode,
                    // which the AMS column gives for the Greek variants where `<latex>` gives the
                    // text-mode letter, as for `\epsilon`.
                    column = match name.as_ref() {
                        b"AMS" => Some(0),
                        b"IEEE" => Some(2),
                        b"latex" => Some(4),
                        b"mathlatex" => {
                            let set = e
                                .try_get_attribute("set")
                                .map_err(|e| e.to_string())?
                                .map(|a| {
                                    a.normalized_value(XmlVersion::default())
                                        .map(|v| v.into_owned())
                                })
                                .transpose()
                                .map_err(|e| e.to_string())?;
                            match set.as_deref() {
                                None => Some(1),
                                Some("unicode-math") => Some(3),
                                Some(_) => None,
                            }
                        }
                        _ => None,
                    };
                    text.clear();
                }
            }
            Event::Text(t) if column.is_some() => {
                text.push_str(&t.decode().map_err(|e| e.to_string())?);
            }
            Event::End(e) => {
                if e.local_name().as_ref() == b"character" {
                    characters.extend(current.take());
                } else if let (Some(col), Some(c)) = (column.take(), current.as_mut())
                    && let Some(name) = command_name(&text)
                {
                    c.names.push((col, name));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok((header, characters))
}

/// The code point of an id `U` followed by five hexadecimal digits; `None` for a sequence.
fn single_code_point(id: &str) -> Option<u32> {
    let hex = id.strip_prefix('U')?;
    if hex.len() != 5 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

/// The name of a column's text that is one control word, as `\leq ` gives `leq`.
fn command_name(text: &str) -> Option<String> {
    let name = text.trim().strip_prefix('\\')?;
    (!name.is_empty() && name.bytes().all(|b| b.is_ascii_alphabetic())).then(|| name.to_owned())
}
