//! The names the subset reads: the generated table of commands, and the commands of the structures
//! and the operator names, written here.

use crate::commands::{ABSENT, CLASSES, COMMANDS};
use crate::{Accent, Class, Command, Environment, MathVariant, Space};

/// The operator names of LaTeX, as `path@elsewhere@base/ltmath.dtx` defines them, each with whether
/// it sets its scripts as limits in display style: those LaTeX defines without `\nolimits`.
pub const OPERATOR_NAMES: &[(&str, bool)] = &[
    ("arccos", false),
    ("arcsin", false),
    ("arctan", false),
    ("arg", false),
    ("cos", false),
    ("cosh", false),
    ("cot", false),
    ("coth", false),
    ("csc", false),
    ("deg", false),
    ("det", true),
    ("dim", false),
    ("exp", false),
    ("gcd", true),
    ("hom", false),
    ("inf", true),
    ("ker", false),
    ("lg", false),
    ("lim", true),
    ("liminf", true),
    ("limsup", true),
    ("ln", false),
    ("log", false),
    ("max", true),
    ("min", true),
    ("Pr", true),
    ("sec", false),
    ("sin", false),
    ("sinh", false),
    ("sup", true),
    ("tan", false),
    ("tanh", false),
];

/// Every command of the table, sorted by name: only the characters the math font has, which the
/// palette lists and an author types.
pub fn commands() -> &'static [Command] {
    COMMANDS
}

/// The command named `name`, without its backslash: one of the table, or one of a character the
/// math font lacks, which a formula may hold and which is drawn as the font's missing glyph.
pub fn command(name: &str) -> Option<Command> {
    [COMMANDS, ABSENT].into_iter().find_map(|table| {
        table
            .binary_search_by(|c| c.name.cmp(name))
            .ok()
            .map(|i| table[i])
    })
}

/// The first command of the table, by name, whose character is `ch`.
pub fn command_of(ch: char) -> Option<Command> {
    COMMANDS.iter().find(|c| c.ch == ch).copied()
}

/// The ASCII characters whose class in LaTeX's math mode differs from the type `unicode.xml` gives
/// them, as `path@elsewhere@base/fontdef.dtx` declares them, so that `3.14` takes no space after its
/// point and `a - b` is a subtraction.
const ASCII: &[(char, Class)] = &[
    ('!', Class::Close),
    ('*', Class::Bin),
    ('-', Class::Bin),
    ('.', Class::Ord),
    (':', Class::Rel),
    ('?', Class::Close),
];

/// The character LaTeX draws for an ASCII character of math mode: the minus sign for `-` and the
/// asterisk operator for `*`, as `path@elsewhere@base/fontdef.dtx` takes them from its symbols font.
pub(crate) fn drawn(ch: char) -> char {
    match ch {
        '-' => '\u{2212}',
        '*' => '\u{2217}',
        _ => ch,
    }
}

/// The class of a character written as itself: LaTeX's for the ASCII characters above, else the
/// class `unicode.xml` gives it, ord when it gives none.
pub(crate) fn class_of(ch: char) -> Class {
    if let Some((_, class)) = ASCII.iter().find(|(c, _)| *c == ch) {
        return *class;
    }
    CLASSES
        .binary_search_by(|(c, _)| c.cmp(&ch))
        .map(|i| CLASSES[i].1)
        .unwrap_or(Class::Ord)
}

/// The operator name `name`, as the table above spells it.
pub(crate) fn operator_name(name: &str) -> Option<&'static str> {
    OPERATOR_NAMES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(n, _)| *n)
}

/// Whether the operator name sets its scripts as limits in display style.
pub(crate) fn operator_limits(name: &str) -> bool {
    OPERATOR_NAMES
        .iter()
        .any(|(n, limits)| *n == name && *limits)
}

pub(crate) const ACCENTS: &[(&str, Accent)] = &[
    ("acute", Accent::Acute),
    ("bar", Accent::Bar),
    ("breve", Accent::Breve),
    ("check", Accent::Check),
    ("ddot", Accent::Ddot),
    ("dot", Accent::Dot),
    ("grave", Accent::Grave),
    ("hat", Accent::Hat),
    ("overline", Accent::Overline),
    ("tilde", Accent::Tilde),
    ("underline", Accent::Underline),
    ("vec", Accent::Vec),
];

impl Accent {
    pub(crate) fn name(self) -> &'static str {
        ACCENTS
            .iter()
            .find(|(_, a)| *a == self)
            .map(|(n, _)| *n)
            .unwrap()
    }

    /// The combining character drawn above the body; `None` for the two drawn as a rule.
    pub(crate) fn mark(self) -> Option<char> {
        Some(match self {
            Accent::Hat => '\u{0302}',
            Accent::Bar => '\u{0304}',
            Accent::Vec => '\u{20D7}',
            Accent::Dot => '\u{0307}',
            Accent::Ddot => '\u{0308}',
            Accent::Tilde => '\u{0303}',
            Accent::Check => '\u{030C}',
            Accent::Breve => '\u{0306}',
            Accent::Acute => '\u{0301}',
            Accent::Grave => '\u{0300}',
            Accent::Overline | Accent::Underline => return None,
        })
    }
}

pub(crate) const VARIANTS: &[(&str, MathVariant)] = &[
    ("mathbb", MathVariant::DoubleStruck),
    ("mathbf", MathVariant::Bold),
    ("mathcal", MathVariant::Calligraphic),
    ("mathit", MathVariant::Italic),
    ("mathrm", MathVariant::Roman),
];

impl MathVariant {
    pub(crate) fn name(self) -> &'static str {
        VARIANTS
            .iter()
            .find(|(_, v)| *v == self)
            .map(|(n, _)| *n)
            .unwrap()
    }
}

pub(crate) const ENVIRONMENTS: &[(&str, Environment)] = &[
    ("bmatrix", Environment::BMatrix),
    ("cases", Environment::Cases),
    ("matrix", Environment::Matrix),
    ("pmatrix", Environment::PMatrix),
];

impl Environment {
    pub(crate) fn name(self) -> &'static str {
        ENVIRONMENTS
            .iter()
            .find(|(_, e)| *e == self)
            .map(|(n, _)| *n)
            .unwrap()
    }
}

/// The spaces, by the name or the character after the backslash.
pub(crate) const SPACES: &[(&str, Space)] = &[
    (",", Space::Thin),
    (":", Space::Medium),
    (";", Space::Thick),
    ("!", Space::NegativeThin),
    ("quad", Space::Quad),
    ("qquad", Space::Qquad),
];

impl Space {
    pub(crate) fn name(self) -> &'static str {
        SPACES
            .iter()
            .find(|(_, s)| *s == self)
            .map(|(n, _)| *n)
            .unwrap()
    }

    /// Its width in mu, 18 to the em.
    pub(crate) fn mu(self) -> f64 {
        match self {
            Space::Thin => 3.0,
            Space::Medium => 4.0,
            Space::Thick => 5.0,
            Space::NegativeThin => -3.0,
            Space::Quad => 18.0,
            Space::Qquad => 36.0,
        }
    }
}

/// The characters a backslash makes a symbol of, with the character each stands for.
pub(crate) const ESCAPED: &[(char, char)] = &[
    ('{', '{'),
    ('}', '}'),
    ('|', '\u{2016}'),
    ('#', '#'),
    ('$', '$'),
    ('%', '%'),
    ('&', '&'),
    ('_', '_'),
];

/// The names of the structures, which the table's commands of the same name do not override.
pub(crate) fn is_structure(name: &str) -> bool {
    matches!(
        name,
        "frac" | "sqrt" | "left" | "right" | "begin" | "end" | "text"
    ) || ACCENTS.iter().any(|(n, _)| *n == name)
        || VARIANTS.iter().any(|(n, _)| *n == name)
        || SPACES.iter().any(|(n, _)| *n == name)
        || operator_name(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_are_sorted_for_their_searches() {
        assert!(COMMANDS.windows(2).all(|w| w[0].name < w[1].name));
        assert!(ABSENT.windows(2).all(|w| w[0].name < w[1].name));
        assert!(CLASSES.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
