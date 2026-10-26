//! The built-in table of the math keys, `design@rootpoint-app@keymap-table`: each entry is a
//! sequence of keys, a chord then the characters typed after it, and the action it runs. The keys are
//! LyX's, copied with their oddities, `design@rootpoint-app@math-keys`, from
//! `path@elsewhere@lib/bind/math.bind` as section 1 of
//! `path@rootpoint@docs/grounding/math-typesetting.md` reads it. The editor's other shortcuts stay
//! where they are, until `issue@rootpoint-app@configurable-keys`.

use eframe::egui::{Key, Modifiers};
use rootpoint_math::MatrixEdit;

/// One key of a sequence: a chord, a key with Ctrl or Alt, or a character typed after one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stroke {
    Chord(Modifiers, Key),
    Char(char),
}

/// When an action acts: while a text or a formula is edited, or while an area is selected and not
/// edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Editing,
    Selected,
}

/// What a sequence runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Insert an inline formula at the caret, the selection its content where it reads as one.
    InsertFormula,
    /// Make the selected area an equation, numbered or not.
    MakeEquation { numbered: bool },
    /// Insert the atom this LaTeX reads as, alone.
    Insert(&'static str),
    /// Make the atom before the caret a script's nucleus: superscript (`true`) or subscript.
    Script(bool),
    /// Toggle the number of the equation being edited.
    ToggleNumber,
    /// Edit the rows (`true`) or the columns of the matrix that holds the caret.
    Matrix(bool, MatrixEdit),
}

impl Action {
    pub fn scope(self) -> Scope {
        match self {
            Action::MakeEquation { .. } => Scope::Selected,
            _ => Scope::Editing,
        }
    }
}

/// An entry of the table: its sequence and its action.
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub keys: Vec<Stroke>,
    pub action: Action,
}

/// The prefix of LyX's math table, Alt+M.
pub const ALT_M: Stroke = Stroke::Chord(Modifiers::ALT, Key::M);

/// The characters typed after Alt+M, and what each runs: LyX's `math.bind`, as section 1 of
/// `path@rootpoint@docs/grounding/math-typesetting.md` reads it, `design@rootpoint-app@math-keys`.
const AFTER_ALT_M: &[(&str, Action)] = &[
    // LyX's `math-mode`, which Ctrl+M runs too.
    ("m", Action::InsertFormula),
    ("f", Action::Insert(r"\frac{}{}")),
    ("s", Action::Insert(r"\sqrt{}")),
    // LyX's `\root`.
    ("r", Action::Insert(r"\sqrt[]{}")),
    ("x", Action::Script(false)),
    ("e", Action::Script(true)),
    ("u", Action::Insert(r"\sum")),
    ("i", Action::Insert(r"\int")),
    ("y", Action::Insert(r"\oint")),
    ("p", Action::Insert(r"\partial")),
    ("8", Action::Insert(r"\infty")),
    ("+", Action::Insert(r"\pm")),
    ("=", Action::Insert(r"\neq")),
    ("n", Action::ToggleNumber),
    ("\"", Action::Insert(r"\ddot{}")),
    ("h", Action::Insert(r"\hat{}")),
    ("\\", Action::Insert(r"\grave{}")),
    ("/", Action::Insert(r"\acute{}")),
    ("&", Action::Insert(r"\tilde{}")),
    ("-", Action::Insert(r"\bar{}")),
    (".", Action::Insert(r"\dot{}")),
    ("V", Action::Insert(r"\check{}")),
    ("U", Action::Insert(r"\breve{}")),
    ("v", Action::Insert(r"\vec{}")),
    ("b", Action::Insert(r"\overline{}")),
    // LyX's `\underbar`.
    ("_", Action::Insert(r"\underline{}")),
    // LyX's `\prime`, a symbol, as the subset reads it.
    ("'", Action::Insert(r"\prime")),
    ("(", Action::Insert(r"\left(\right)")),
    ("[", Action::Insert(r"\left[\right]")),
    ("{", Action::Insert(r"\left\{\right\}")),
    ("<", Action::Insert(r"\left\langle\right\rangle")),
    (">", Action::Insert(r"\left\rangle\right\langle")),
    ("|", Action::Insert(r"\left|\right|")),
    // LyX's `math-space`, which inserts its `defaultSpace`, the thin space `\,`, by
    // `InsetMathSpace` of its source.
    (" ", Action::Insert(r"\,")),
    ("ci", Action::Matrix(false, MatrixEdit::Append)),
    ("cd", Action::Matrix(false, MatrixEdit::Delete)),
    ("cc", Action::Matrix(false, MatrixEdit::Copy)),
    ("cs", Action::Matrix(false, MatrixEdit::Swap)),
    ("wi", Action::Matrix(true, MatrixEdit::Append)),
    ("wd", Action::Matrix(true, MatrixEdit::Delete)),
    ("wc", Action::Matrix(true, MatrixEdit::Copy)),
    ("ws", Action::Matrix(true, MatrixEdit::Swap)),
];

/// Alt+M G, then a letter: LyX's Greek, o and w both ω, v θ and q ϑ, and six Shift keys giving a
/// variant in lowercase.
const GREEK: &[(char, &str)] = &[
    ('a', r"\alpha"),
    ('b', r"\beta"),
    ('c', r"\chi"),
    ('d', r"\delta"),
    ('e', r"\epsilon"),
    ('f', r"\phi"),
    ('g', r"\gamma"),
    ('h', r"\eta"),
    ('i', r"\iota"),
    ('j', r"\varphi"),
    ('k', r"\kappa"),
    ('l', r"\lambda"),
    ('m', r"\mu"),
    ('n', r"\nu"),
    ('o', r"\omega"),
    ('p', r"\pi"),
    ('q', r"\vartheta"),
    ('r', r"\rho"),
    ('s', r"\sigma"),
    ('t', r"\tau"),
    ('u', r"\upsilon"),
    ('v', r"\theta"),
    ('w', r"\omega"),
    ('x', r"\xi"),
    ('y', r"\psi"),
    ('z', r"\zeta"),
    ('D', r"\Delta"),
    ('E', r"\varepsilon"),
    ('F', r"\Phi"),
    ('G', r"\Gamma"),
    ('I', r"\iota"),
    ('J', r"\epsilon"),
    ('L', r"\Lambda"),
    ('O', r"\Omega"),
    ('P', r"\Pi"),
    ('Q', r"\vartheta"),
    ('R', r"\varrho"),
    ('S', r"\Sigma"),
    ('T', r"\varsigma"),
    ('U', r"\Upsilon"),
    ('V', r"\Theta"),
    ('W', r"\Omega"),
    ('X', r"\Xi"),
    ('Y', r"\Psi"),
];

/// The whole table.
pub fn bindings() -> Vec<Binding> {
    let chord = |modifiers, key| vec![Stroke::Chord(modifiers, key)];
    let mut out = vec![
        Binding {
            keys: chord(Modifiers::COMMAND, Key::M),
            action: Action::InsertFormula,
        },
        Binding {
            keys: chord(Modifiers::COMMAND | Modifiers::SHIFT, Key::M),
            action: Action::MakeEquation { numbered: false },
        },
        Binding {
            keys: chord(Modifiers::COMMAND | Modifiers::ALT, Key::N),
            action: Action::MakeEquation { numbered: true },
        },
    ];
    for (typed, action) in AFTER_ALT_M {
        let mut keys = vec![ALT_M];
        keys.extend(typed.chars().map(Stroke::Char));
        out.push(Binding {
            keys,
            action: *action,
        });
    }
    for (letter, latex) in GREEK {
        out.push(Binding {
            keys: vec![ALT_M, Stroke::Char('g'), Stroke::Char(*letter)],
            action: Action::Insert(latex),
        });
    }
    out
}

/// A sequence as the status line names it while it waits, as `Alt+M G`.
pub fn name(keys: &[Stroke]) -> String {
    keys.iter()
        .map(|k| match k {
            Stroke::Chord(m, key) => {
                let mut out = String::new();
                if m.ctrl || m.command {
                    out.push_str("Ctrl+");
                }
                if m.alt {
                    out.push_str("Alt+");
                }
                if m.shift {
                    out.push_str("Shift+");
                }
                out.push_str(key.name());
                out
            }
            Stroke::Char(c) => c.to_uppercase().to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether a key types no text, so that it drops a sequence that waits: egui gives a text event
/// for every other key.
pub fn types_no_text(key: Key) -> bool {
    matches!(
        key,
        Key::Escape
            | Key::Backspace
            | Key::Delete
            | Key::Enter
            | Key::Tab
            | Key::ArrowLeft
            | Key::ArrowRight
            | Key::ArrowUp
            | Key::ArrowDown
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
            | Key::Insert
    )
}

/// What a sequence of keys typed so far is in the table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lookup {
    /// A whole sequence, which runs its action.
    Action(Action),
    /// The start of one or more sequences, which waits for its next key.
    Prefix,
    /// No sequence of the table.
    None,
}

/// The sequence `keys` in the table.
pub fn lookup(keys: &[Stroke]) -> Lookup {
    let mut prefix = false;
    for b in bindings() {
        if b.keys.len() >= keys.len() && b.keys.iter().zip(keys).all(|(a, k)| same(a, k)) {
            if b.keys.len() == keys.len() {
                return Lookup::Action(b.action);
            }
            prefix = true;
        }
    }
    if prefix { Lookup::Prefix } else { Lookup::None }
}

/// Whether two strokes are one key: chords by their modifiers exactly, Ctrl and the command key
/// being one, as egui reads them.
fn same(a: &Stroke, b: &Stroke) -> bool {
    match (a, b) {
        (Stroke::Chord(ma, ka), Stroke::Chord(mb, kb)) => ka == kb && mb.matches_exact(*ma),
        (Stroke::Char(a), Stroke::Char(b)) => a == b,
        _ => false,
    }
}

impl crate::App {
    /// A key with Ctrl or Alt while a text or a formula is edited, read against the table: one
    /// that runs an action runs it, one that starts a sequence waits for its next key; whether
    /// the table took it.
    pub(crate) fn math_chord(&mut self, key: Key, modifiers: Modifiers) -> bool {
        if !(modifiers.command || modifiers.ctrl || modifiers.alt) {
            return false;
        }
        let stroke = Stroke::Chord(modifiers, key);
        match lookup(&[stroke]) {
            Lookup::Action(action) if action.scope() == Scope::Editing => {
                self.key_sequence.clear();
                self.run_key_action(action);
                true
            }
            Lookup::Prefix => {
                self.key_sequence = vec![stroke];
                self.chord_text = modifiers.alt;
                true
            }
            Lookup::Action(_) | Lookup::None => false,
        }
    }

    /// A character typed while a sequence waits: the sequence runs its action, waits again, or,
    /// matching none, is dropped with the character.
    pub(crate) fn sequence_char(&mut self, ch: char) {
        // egui can give Alt+M's own text, `m` or a character of the layout, as `µ`.
        if std::mem::take(&mut self.chord_text) && (ch.eq_ignore_ascii_case(&'m') || !ch.is_ascii())
        {
            return;
        }
        self.key_sequence.push(Stroke::Char(ch));
        match lookup(&self.key_sequence) {
            Lookup::Action(action) => {
                self.key_sequence.clear();
                self.run_key_action(action);
            }
            Lookup::Prefix => {}
            Lookup::None => self.key_sequence.clear(),
        }
    }

    /// Run an action of the table. In a text whose caret is in no formula, an action that edits a
    /// formula first inserts one at the caret, as LyX's math keys do in its text.
    pub fn run_key_action(&mut self, action: Action) {
        match action {
            Action::InsertFormula => self.insert_formula(),
            Action::MakeEquation { numbered } => self.make_equation(numbered),
            Action::ToggleNumber => self.toggle_equation_number(),
            Action::Insert(_) | Action::Script(_) | Action::Matrix(..) => {
                if self.math_caret().is_none() {
                    self.insert_formula();
                }
                if self.math_caret().is_none() {
                    return;
                }
                match action {
                    Action::Insert(latex) => {
                        if let Some(atom) = rootpoint_math::Formula::parse(latex)
                            .ok()
                            .and_then(|f| f.rows.into_iter().next()?.into_iter().next()?.pop())
                        {
                            self.insert_math_atom(atom);
                        }
                    }
                    Action::Script(sup) => self.math_script(sup),
                    Action::Matrix(rows, edit) => self.edit_math_matrix(rows, edit),
                    _ => {}
                }
            }
        }
    }
}
