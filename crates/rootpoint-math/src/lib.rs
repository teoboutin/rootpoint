//! The math of rootpoint: a formula as a tree, its closed subset of LaTeX read and written, the
//! table of commands, and the layout of a formula from a font's OpenType MATH table into a box per
//! atom and per cell, with the glyphs to draw.

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("ROOTPOINT_CHECKOUT");

mod commands;
mod edit;
mod font;
mod latex;
mod layout;
mod table;

pub use edit::*;
pub use font::{FontError, MathFont, VerticalConstruction};
pub use latex::{MAX_DEPTH, ParseError};
pub use layout::{AtomBox, CellBox, FormulaLayout, PlacedGlyph, Rule, Style, lay_out};
pub use table::{OPERATOR_NAMES, command, command_of, commands};

/// The class of an atom, which sets the space between two neighbours, as TeX's classes do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

/// A command of the table: its name without the backslash, its character and its class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: &'static str,
    pub ch: char,
    pub class: Class,
}

/// A formula: a grid of rows, each row a list of cells, separated by `&` in LaTeX and the rows by
/// `\\`. A formula with no `&` and no `\\` is one row of one cell.
///
/// `parse` gives only formulas that keep these invariants, and `to_latex` writes a formula that
/// keeps them as a text that reads back equal; a caller that builds or edits a formula keeps them:
/// - a grid, the formula's or a matrix's, has a row, and each row a cell;
/// - a symbol of no name is not a space, `\`, `^` or `'`, and a radical's index holds no `]` at its
///   top level, which a group around it keeps;
/// - a `Text` holds balanced braces, and ends with no lone backslash;
/// - a script has a subscript or a superscript; no script follows a group, which `parse` reads as
///   its nucleus, and no script of an empty nucleus follows a script, which `parse` reads as one;
/// - atoms nest at most `MAX_DEPTH` deep.
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    pub rows: Vec<Vec<List>>,
}

/// The atoms of one cell, in order.
pub type List = Vec<Atom>;

/// One atom of a list.
#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    /// A character with its class: a letter, a digit, a character of the table, or a command of
    /// the table, whose name is kept so that it is written back as it was read.
    Symbol {
        ch: char,
        class: Class,
        name: Option<&'static str>,
    },
    Fraction {
        num: List,
        den: List,
    },
    Radical {
        index: Option<List>,
        body: List,
    },
    /// A nucleus with a subscript, a superscript or both. A nucleus that is one big operator is set
    /// with its scripts as limits, where TeX sets them so.
    Scripts {
        nucleus: List,
        sub: Option<List>,
        sup: Option<List>,
    },
    /// `\left` and `\right` around a list; `None` is the empty delimiter `.`.
    Delimited {
        open: Option<Delim>,
        close: Option<Delim>,
        body: List,
    },
    Accent {
        accent: Accent,
        body: List,
    },
    /// One of LaTeX's operator names, as `\sin` or `\lim`: upright letters of class op.
    OperatorName(&'static str),
    FontChange {
        font: MathVariant,
        body: List,
    },
    /// `\text{…}`: the characters between the braces, as written.
    Text(String),
    Matrix {
        env: Environment,
        rows: Vec<Vec<List>>,
    },
    Space(Space),
    /// `{…}`.
    Group(List),
    /// A command the subset does not know, kept by its name, without the backslash. It takes no
    /// argument, so the groups after it stay groups.
    Unknown(String),
}

/// A delimiter of `\left` or `\right`: its character, and the name it was written with, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delim {
    pub ch: char,
    pub name: Option<&'static str>,
}

/// The accents of the subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Accent {
    Hat,
    Bar,
    Vec,
    Dot,
    Ddot,
    Tilde,
    Check,
    Breve,
    Acute,
    Grave,
    Overline,
    Underline,
}

/// The font changes of the subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathVariant {
    Bold,
    Roman,
    Calligraphic,
    DoubleStruck,
    Italic,
}

/// The matrix environments of the subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Environment {
    Matrix,
    PMatrix,
    BMatrix,
    Cases,
}

/// The spaces of the subset: `\,`, `\:`, `\;`, `\!`, `\quad` and `\qquad`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Space {
    Thin,
    Medium,
    Thick,
    NegativeThin,
    Quad,
    Qquad,
}

impl Formula {
    /// The empty formula: one row of one empty cell.
    pub fn empty() -> Self {
        Formula {
            rows: vec![vec![Vec::new()]],
        }
    }

    /// Whether the formula holds no atom.
    pub fn is_empty(&self) -> bool {
        self.rows.iter().flatten().all(|cell| cell.is_empty())
    }

    /// Whether the formula, or an atom of it, holds a command the subset does not know.
    pub fn has_unknown(&self) -> bool {
        self.rows.iter().flatten().any(list_has_unknown)
    }
}

fn list_has_unknown(list: &List) -> bool {
    list.iter().any(|atom| match atom {
        Atom::Unknown(_) => true,
        Atom::Matrix { rows, .. } => rows.iter().flatten().any(list_has_unknown),
        other => other.cells().iter().any(|cell| list_has_unknown(cell)),
    })
}

impl Atom {
    /// The cells of the atom, mutable, in the order of [`Atom::cells`].
    pub fn cells_mut(&mut self) -> Vec<&mut List> {
        match self {
            Atom::Fraction { num, den } => vec![num, den],
            Atom::Radical { index, body } => std::iter::once(body).chain(index).collect(),
            Atom::Scripts { nucleus, sub, sup } => {
                std::iter::once(nucleus).chain(sub).chain(sup).collect()
            }
            Atom::Delimited { body, .. }
            | Atom::Accent { body, .. }
            | Atom::FontChange { body, .. }
            | Atom::Group(body) => vec![body],
            Atom::Matrix { rows, .. } => rows.iter_mut().flatten().collect(),
            Atom::Symbol { .. }
            | Atom::OperatorName(_)
            | Atom::Text(_)
            | Atom::Space(_)
            | Atom::Unknown(_) => Vec::new(),
        }
    }

    /// The cells of the atom, in the order a `CellPath` numbers them: a fraction's numerator then
    /// its denominator; a radical's body then its index; a script's nucleus, subscript and
    /// superscript, those present; a matrix's cells row by row; the body of any other atom with
    /// one.
    pub fn cells(&self) -> Vec<&List> {
        match self {
            Atom::Fraction { num, den } => vec![num, den],
            Atom::Radical { index, body } => std::iter::once(body).chain(index).collect(),
            Atom::Scripts { nucleus, sub, sup } => {
                std::iter::once(nucleus).chain(sub).chain(sup).collect()
            }
            Atom::Delimited { body, .. }
            | Atom::Accent { body, .. }
            | Atom::FontChange { body, .. }
            | Atom::Group(body) => vec![body],
            Atom::Matrix { rows, .. } => rows.iter().flatten().collect(),
            Atom::Symbol { .. }
            | Atom::OperatorName(_)
            | Atom::Text(_)
            | Atom::Space(_)
            | Atom::Unknown(_) => Vec::new(),
        }
    }
}

/// Where a caret stands in a formula, or where a box sits: the cell of the formula's grid, then,
/// from that cell down, the index of an atom in its list and of a cell of that atom, as
/// `Atom::cells` numbers them, then a position in the innermost list, from 0 to its length. The
/// path of an atom's box is that of the position before the atom.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CellPath {
    pub row: usize,
    pub column: usize,
    pub steps: Vec<(usize, usize)>,
    pub offset: usize,
}
