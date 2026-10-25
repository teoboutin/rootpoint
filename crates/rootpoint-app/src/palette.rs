//! The palette of symbols and structures the Selection tab shows while a formula is edited: the
//! commands of the math crate's table by category, each inserted at the caret by a click, and the
//! structures, scripts, spaces and the controls of a matrix's rows and columns. The table holds the
//! characters the built-in math font has alone, so the palette offers no command the font draws as
//! a box.

use rootpoint_math::{Atom, Class, Command, Formula, MatrixEdit, commands};

/// A category of the palette, in the order the tab lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Relations,
    Arrows,
    Operators,
    Greek,
    Delimiters,
    Symbols,
    Letters,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Relations,
        Category::Arrows,
        Category::Operators,
        Category::Greek,
        Category::Delimiters,
        Category::Symbols,
        Category::Letters,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::Relations => "Relations",
            Category::Arrows => "Arrows",
            Category::Operators => "Operators",
            Category::Greek => "Greek",
            Category::Delimiters => "Delimiters",
            Category::Symbols => "Other symbols",
            Category::Letters => "Letters",
        }
    }
}

/// The category of a command of the table: an arrow by its block, whatever its class; a Greek
/// letter by its block; a styled letter of the block of mathematical alphanumerics, which
/// `\mathbf` and its kin also set; then by its class.
pub fn category(c: &Command) -> Category {
    let code = u32::from(c.ch);
    let arrow = (0x2190..=0x21FF).contains(&code)
        || (0x27F0..=0x27FF).contains(&code)
        || (0x2900..=0x297F).contains(&code)
        || (0x2B00..=0x2BFF).contains(&code);
    if arrow {
        Category::Arrows
    } else if (0x370..=0x3FF).contains(&code) {
        Category::Greek
    } else if (0x1D400..=0x1D7FF).contains(&code) {
        Category::Letters
    } else {
        match c.class {
            Class::Rel => Category::Relations,
            Class::Bin | Class::Op => Category::Operators,
            Class::Open | Class::Close => Category::Delimiters,
            Class::Ord | Class::Punct | Class::Inner => Category::Symbols,
        }
    }
}

/// The commands of a category, in the table's order, by name.
pub fn of_category(category: Category) -> impl Iterator<Item = &'static Command> {
    commands()
        .iter()
        .filter(move |c| self::category(c) == category)
}

/// What a button of the palette does at the caret.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// Insert the atom, the caret in its first cell when it has cells.
    Insert(Atom),
    /// Make the atom before the caret a script's nucleus: superscript (`true`) or subscript.
    Script(bool),
    /// Edit the rows (`true`) or the columns of the matrix that holds the caret.
    Matrix(bool, MatrixEdit),
}

/// The atom a structure's LaTeX reads as, alone.
fn structure(latex: &str) -> Atom {
    let f = Formula::parse(latex).expect("a structure of the palette parses");
    f.rows[0][0][0].clone()
}

/// The structures of the palette, by their label and the LaTeX of the atom each inserts, its cells
/// empty, with the scripts; LyX's math toolbar holds the same.
pub fn structures() -> Vec<(&'static str, Action)> {
    let mut out: Vec<(&'static str, Action)> = [
        ("a/b", r"\frac{}{}"),
        ("√", r"\sqrt{}"),
        ("ⁿ√", r"\sqrt[]{}"),
        ("( )", r"\left(\right)"),
        ("[ ]", r"\left[\right]"),
        ("{ }", r"\left\{\right\}"),
        ("| |", r"\left|\right|"),
        ("⟨ ⟩", r"\left\langle\right\rangle"),
        ("⟩ ⟨", r"\left\rangle\right\langle"),
        ("matrix", r"\begin{matrix} & \\ & \end{matrix}"),
        ("pmatrix", r"\begin{pmatrix} & \\ & \end{pmatrix}"),
        ("bmatrix", r"\begin{bmatrix} & \\ & \end{bmatrix}"),
        ("cases", r"\begin{cases} & \\ & \end{cases}"),
        ("â", r"\hat{}"),
        ("ā", r"\bar{}"),
        ("a⃗", r"\vec{}"),
        ("ȧ", r"\dot{}"),
        ("ä", r"\ddot{}"),
        ("ã", r"\tilde{}"),
        ("á", r"\acute{}"),
        ("à", r"\grave{}"),
        ("ǎ", r"\check{}"),
        ("ă", r"\breve{}"),
        ("overline", r"\overline{}"),
        ("underline", r"\underline{}"),
        ("text", r"\text{}"),
        ("mathrm", r"\mathrm{}"),
        ("mathbf", r"\mathbf{}"),
        ("mathit", r"\mathit{}"),
        ("mathbb", r"\mathbb{}"),
        ("mathcal", r"\mathcal{}"),
        ("thin space", r"\,"),
        ("quad", r"\quad"),
        ("qquad", r"\qquad"),
    ]
    .into_iter()
    .map(|(label, latex)| (label, Action::Insert(structure(latex))))
    .collect();
    out.insert(0, ("x_", Action::Script(false)));
    out.insert(0, ("x^", Action::Script(true)));
    out
}

/// The controls of a matrix's rows and columns, by label: append, delete, copy and swap, as LyX's
/// Alt+M W and Alt+M C sequences do.
pub fn matrix_controls() -> Vec<(&'static str, Action)> {
    let mut out = Vec::new();
    for (rows, edits) in [
        (
            true,
            [
                ("row: append", MatrixEdit::Append),
                ("row: delete", MatrixEdit::Delete),
                ("row: copy", MatrixEdit::Copy),
                ("row: swap", MatrixEdit::Swap),
            ],
        ),
        (
            false,
            [
                ("column: append", MatrixEdit::Append),
                ("column: delete", MatrixEdit::Delete),
                ("column: copy", MatrixEdit::Copy),
                ("column: swap", MatrixEdit::Swap),
            ],
        ),
    ] {
        for (label, edit) in edits {
            out.push((label, Action::Matrix(rows, edit)));
        }
    }
    out
}
