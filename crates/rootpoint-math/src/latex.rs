//! The closed subset of LaTeX a formula is written in, read by `Formula::parse` and written by
//! `Formula::to_latex`, `design@rootpoint-math@math-storage-form`. `to_latex` of `parse` of a text `to_latex` wrote gives the same text.

use std::fmt;

use crate::table::{
    ACCENTS, ENVIRONMENTS, ESCAPED, SPACES, VARIANTS, class_of, command, is_structure,
    operator_name,
};
use crate::{Atom, Class, Delim, Formula, List};

/// Why a text is not a formula of the subset: what is wrong, and the byte offset where it was met.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub at: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.at)
    }
}

impl std::error::Error for ParseError {}

impl Formula {
    /// Read a formula from its LaTeX. A text that does not parse is refused: an unbalanced brace, a
    /// second superscript or subscript on one nucleus, an `\end` that closes no `\begin`, a
    /// `\right` with no `\left`, a missing argument, an `&` or a `\\` outside the formula's grid
    /// or a matrix, an environment the subset does not know, a delimiter that is no open, close,
    /// bar or slash, a `$` in a text, or atoms nested deeper than `MAX_DEPTH`. An unknown command
    /// is an atom of its own.
    pub fn parse(text: &str) -> Result<Formula, ParseError> {
        let mut p = Parser {
            text,
            tokens: tokenize(text),
            pos: 0,
            depth: 0,
        };
        let (rows, stop) = p.grid()?;
        match stop {
            Stop::Eof => Ok(Formula { rows }),
            other => Err(p.error_at_last(&format!("{} with nothing open", other.describe()))),
        }
    }

    /// Write the formula as LaTeX of the subset: cells joined by ` & `, rows by ` \\ `, a space
    /// on each side of a binary operator or a relation.
    pub fn to_latex(&self) -> String {
        write_grid(&self.rows)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    /// `\` and letters, the name without the backslash.
    Word(String),
    /// `\` and one character that is not a letter.
    Symbol(char),
    Char(char),
    Space,
}

/// Each token with the byte offset where it starts.
fn tokenize(text: &str) -> Vec<(usize, Token)> {
    let mut out = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let token = if c == '\\' {
            match chars.peek().copied() {
                Some((_, l)) if l.is_ascii_alphabetic() => {
                    let mut name = String::new();
                    while let Some(&(_, l)) = chars.peek() {
                        if !l.is_ascii_alphabetic() {
                            break;
                        }
                        name.push(l);
                        chars.next();
                    }
                    Token::Word(name)
                }
                Some((_, s)) => {
                    chars.next();
                    Token::Symbol(s)
                }
                // A backslash at the end stands for itself, and is refused by the parser.
                None => Token::Symbol('\0'),
            }
        } else if c.is_whitespace() {
            Token::Space
        } else {
            Token::Char(c)
        };
        out.push((at, token));
    }
    out
}

/// What ended a list.
#[derive(Clone, Debug, PartialEq)]
enum Stop {
    Eof,
    /// `}`.
    Close,
    /// `&`.
    Align,
    /// `\\`.
    Row,
    /// `\right`, with the delimiter after it.
    Right(Option<Delim>),
    /// `\end{…}`, with the environment's name.
    End(String),
    /// `]`, where an index of a radical is read.
    Bracket,
}

impl Stop {
    fn describe(&self) -> String {
        match self {
            Stop::Eof => "the end".to_owned(),
            Stop::Close => "a `}`".to_owned(),
            Stop::Align => "an `&`".to_owned(),
            Stop::Row => "a `\\\\`".to_owned(),
            Stop::Right(_) => "a `\\right`".to_owned(),
            Stop::End(name) => format!("an `\\end{{{name}}}`"),
            Stop::Bracket => "a `]`".to_owned(),
        }
    }
}

/// How deep atoms may nest, one in the cell of another. Reading and laying out recurse once per
/// level, so a bound keeps a formula of a few kilobytes from overflowing the stack, which aborts
/// the program: in a debug build, a thread of 1 MB, the stack Rust's wasm32 target gives, overflowed
/// at a nesting of 200 `\left(`, measured at the review of the math crate, and held the deepest
/// formula this bound admits. No formula of a talk nests near it.
pub const MAX_DEPTH: usize = 64;

struct Parser<'a> {
    text: &'a str,
    tokens: Vec<(usize, Token)>,
    pos: usize,
    /// How deep the atom being read is nested.
    depth: usize,
}

impl Parser<'_> {
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_owned(),
            at: self
                .tokens
                .get(self.pos)
                .map(|(at, _)| *at)
                .unwrap_or(self.text.len()),
        }
    }

    /// An error at the token just read.
    fn error_at_last(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_owned(),
            at: self
                .tokens
                .get(self.pos.saturating_sub(1))
                .map(|(at, _)| *at)
                .unwrap_or(self.text.len()),
        }
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).map(|(_, t)| t.clone());
        self.pos += 1;
        t
    }

    fn skip_spaces(&mut self) {
        while matches!(self.tokens.get(self.pos), Some((_, Token::Space))) {
            self.pos += 1;
        }
    }

    /// Rows of cells, up to a stop that is neither `&` nor `\\`.
    fn grid(&mut self) -> Result<(Vec<Vec<List>>, Stop), ParseError> {
        let mut rows = vec![Vec::new()];
        loop {
            let (cell, stop) = self.list(false)?;
            rows.last_mut().unwrap().push(cell);
            match stop {
                Stop::Align => {}
                Stop::Row => rows.push(Vec::new()),
                other => return Ok((rows, other)),
            }
        }
    }

    /// A list of atoms, up to the stop that ends it. `in_index` makes a `]` a stop.
    fn list(&mut self, in_index: bool) -> Result<(List, Stop), ParseError> {
        self.enter()?;
        let list = self.list_at_depth(in_index);
        self.leave();
        list
    }

    fn list_at_depth(&mut self, in_index: bool) -> Result<(List, Stop), ParseError> {
        let mut list = List::new();
        loop {
            let Some(token) = self.next() else {
                return Ok((list, Stop::Eof));
            };
            match token {
                Token::Space => {}
                Token::Char('}') => return Ok((list, Stop::Close)),
                Token::Char('&') => return Ok((list, Stop::Align)),
                Token::Char(']') if in_index => return Ok((list, Stop::Bracket)),
                Token::Symbol('\\') => return Ok((list, Stop::Row)),
                Token::Char(c @ ('^' | '_')) => self.script(&mut list, c == '^')?,
                Token::Char('\'') => self.prime(&mut list)?,
                Token::Word(name) if name == "right" => {
                    let delim = self.delimiter()?;
                    return Ok((list, Stop::Right(delim)));
                }
                Token::Word(name) if name == "end" => {
                    let env = self.raw_group()?;
                    return Ok((list, Stop::End(env)));
                }
                other => list.push(self.atom(other)?),
            }
        }
    }

    /// A list closed by `}`, after its `{` was read.
    fn group(&mut self) -> Result<List, ParseError> {
        match self.list(false)? {
            (list, Stop::Close) => Ok(list),
            (_, Stop::Eof) => Err(self.error("a `{` is not closed")),
            (_, other) => Err(self.error_at_last(&format!("{} inside a group", other.describe()))),
        }
    }

    /// The argument of a command or a script: a braced group, or one atom.
    fn argument(&mut self) -> Result<List, ParseError> {
        self.skip_spaces();
        match self.next() {
            Some(Token::Char('{')) => self.group(),
            Some(Token::Char(c @ ('}' | '&' | '^' | '_' | ']'))) => {
                Err(self.error_at_last(&format!("an argument is missing before `{c}`")))
            }
            Some(Token::Symbol('\\')) | None => Err(self.error_at_last("an argument is missing")),
            Some(Token::Word(name)) if name == "right" || name == "end" => {
                Err(self.error_at_last("an argument is missing"))
            }
            // A one-atom argument is a list of the tree, as a braced one is.
            Some(token) => {
                self.enter()?;
                let atom = self.atom(token);
                self.leave();
                Ok(vec![atom?])
            }
        }
    }

    /// The text between braces, kept as written, braces nested inside it balanced.
    fn raw_group(&mut self) -> Result<String, ParseError> {
        self.skip_spaces();
        let Some((start, Token::Char('{'))) = self.tokens.get(self.pos).cloned() else {
            return Err(self.error("a `{` is expected"));
        };
        self.pos += 1;
        let mut depth = 0usize;
        while let Some((at, token)) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            match token {
                Token::Char('{') => depth += 1,
                Token::Char('}') if depth == 0 => {
                    return Ok(self.text[start + 1..at].to_owned());
                }
                Token::Char('}') => depth -= 1,
                // A `$` would switch LaTeX to math, which a group read as written does not read;
                // it would also end the `$…$` of the protocol that holds the formula. `\$` writes
                // a dollar.
                Token::Char('$') => {
                    return Err(ParseError {
                        message: "a `$` in a text, where `\\$` writes a dollar".to_owned(),
                        at,
                    });
                }
                _ => {}
            }
        }
        Err(self.error("a `{` is not closed"))
    }

    /// A `^` or a `_` and its argument, on the atom before it.
    fn script(&mut self, list: &mut List, sup: bool) -> Result<(), ParseError> {
        let at = self.pos - 1;
        let arg = self.argument()?;
        let double = || ParseError {
            message: format!(
                "a second {} on one nucleus",
                if sup { "superscript" } else { "subscript" }
            ),
            at: self.tokens[at].0,
        };
        if let Some(Atom::Scripts {
            sub, sup: sup_slot, ..
        }) = list.last_mut()
        {
            // A superscript after primes joins them, as TeX reads `f'^2` as `f^{\prime 2}`.
            if sup && let Some(primes) = sup_slot.as_mut().filter(|s| all_primes(s)) {
                primes.extend(arg);
                return Ok(());
            }
            let slot = if sup { sup_slot } else { sub };
            if slot.is_some() {
                return Err(double());
            }
            *slot = Some(arg);
            return Ok(());
        }
        let nucleus = match list.pop() {
            Some(Atom::Group(inner)) => inner,
            Some(atom) => vec![atom],
            None => Vec::new(),
        };
        let (sub, sup_list) = if sup {
            (None, Some(arg))
        } else {
            (Some(arg), None)
        };
        list.push(Atom::Scripts {
            nucleus,
            sub,
            sup: sup_list,
        });
        Ok(())
    }

    /// A `'` after an atom: a prime in its superscript, as LaTeX reads `f'` as `f^{\prime}`, a second
    /// `'` adding a second prime; after another superscript, as in `f^2'`, a second superscript,
    /// refused as TeX refuses it.
    fn prime(&mut self, list: &mut List) -> Result<(), ParseError> {
        if let Some(Atom::Scripts { sup, .. }) = list.last_mut() {
            match sup {
                None => {
                    *sup = Some(vec![PRIME]);
                    return Ok(());
                }
                Some(primes) if all_primes(primes) => {
                    primes.push(PRIME);
                    return Ok(());
                }
                Some(_) => return Err(self.error_at_last("a second superscript on one nucleus")),
            }
        }
        let nucleus = match list.pop() {
            Some(Atom::Group(inner)) => inner,
            Some(atom) => vec![atom],
            None => Vec::new(),
        };
        list.push(Atom::Scripts {
            nucleus,
            sub: None,
            sup: Some(vec![PRIME]),
        });
        Ok(())
    }

    /// The delimiter after `\left` or `\right`.
    fn delimiter(&mut self) -> Result<Option<Delim>, ParseError> {
        self.skip_spaces();
        match self.next() {
            Some(Token::Char('.')) => Ok(None),
            // `<` and `>` stand for the angle brackets after `\left` and `\right`, as plain.tex's
            // `\delcode` makes them.
            Some(Token::Char('<')) => Ok(Some(Delim {
                ch: '\u{27E8}',
                name: Some("langle"),
            })),
            Some(Token::Char('>')) => Ok(Some(Delim {
                ch: '\u{27E9}',
                name: Some("rangle"),
            })),
            // A bare brace opens or closes a group, so `\left{` is refused, as TeX refuses it.
            Some(Token::Char(c)) if is_delimiter(c) && c != '{' && c != '}' => {
                Ok(Some(Delim { ch: c, name: None }))
            }
            Some(Token::Symbol(s @ ('{' | '}' | '|'))) => {
                let ch = ESCAPED.iter().find(|(e, _)| *e == s).unwrap().1;
                Ok(Some(Delim {
                    ch,
                    name: Some(escaped_name(s)),
                }))
            }
            Some(Token::Word(name)) => match command(&name).filter(|c| is_delimiter(c.ch)) {
                Some(c) => Ok(Some(Delim {
                    ch: c.ch,
                    name: Some(c.name),
                })),
                None => Err(self.error_at_last(&format!("`\\{name}` is not a delimiter"))),
            },
            _ => Err(self.error_at_last("a delimiter is expected")),
        }
    }

    /// Enter a list, refused when lists would nest deeper than `MAX_DEPTH`; `leave` ends it.
    fn enter(&mut self) -> Result<(), ParseError> {
        if self.depth >= MAX_DEPTH {
            return Err(self.error_at_last(&format!("atoms nest deeper than {MAX_DEPTH} levels")));
        }
        self.depth += 1;
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }

    /// The atom a token begins.
    fn atom(&mut self, token: Token) -> Result<Atom, ParseError> {
        Ok(match token {
            Token::Char('{') => Atom::Group(self.group()?),
            Token::Char(c) => Atom::Symbol {
                ch: c,
                class: class_of(c),
                name: None,
            },
            Token::Symbol(s) => {
                let mut one = [0u8; 4];
                let one = &*s.encode_utf8(&mut one);
                if let Some((_, space)) = SPACES.iter().find(|(n, _)| *n == one) {
                    Atom::Space(*space)
                } else if let Some((_, ch)) = ESCAPED.iter().find(|(e, _)| *e == s) {
                    // A character the subset reads otherwise is a symbol of no name, written back
                    // with its backslash; `\|` alone names another character, ‖.
                    Atom::Symbol {
                        ch: *ch,
                        class: if s == '{' {
                            Class::Open
                        } else if s == '}' {
                            Class::Close
                        } else {
                            class_of(*ch)
                        },
                        name: (s == '|').then_some("|"),
                    }
                } else if s == '\0' {
                    return Err(self.error_at_last("a backslash ends the formula"));
                } else {
                    Atom::Unknown(s.to_string())
                }
            }
            Token::Space => unreachable!("spaces are skipped"),
            Token::Word(name) => self.word(name)?,
        })
    }

    /// The atom a control word begins.
    fn word(&mut self, name: String) -> Result<Atom, ParseError> {
        Ok(match name.as_str() {
            "frac" => {
                let num = self.argument()?;
                let den = self.argument()?;
                Atom::Fraction { num, den }
            }
            "sqrt" => {
                self.skip_spaces();
                let index = if matches!(self.tokens.get(self.pos), Some((_, Token::Char('[')))) {
                    self.pos += 1;
                    match self.list(true)? {
                        (index, Stop::Bracket) => Some(index),
                        _ => return Err(self.error("a `[` is not closed")),
                    }
                } else {
                    None
                };
                let body = self.argument()?;
                Atom::Radical { index, body }
            }
            "left" => {
                let open = self.delimiter()?;
                match self.list(false)? {
                    (body, Stop::Right(close)) => Atom::Delimited { open, close, body },
                    (_, Stop::Eof) => return Err(self.error("a `\\left` is not closed")),
                    (_, other) => {
                        return Err(
                            self.error_at_last(&format!("{} inside `\\left`", other.describe()))
                        );
                    }
                }
            }
            "right" => return Err(self.error_at_last("a `\\right` with no `\\left`")),
            "begin" => {
                let env_name = self.raw_group()?;
                let Some((_, env)) = ENVIRONMENTS.iter().find(|(n, _)| *n == env_name) else {
                    return Err(
                        self.error_at_last(&format!("the environment `{env_name}` is not known"))
                    );
                };
                match self.grid()? {
                    (rows, Stop::End(end)) if end == env_name => Atom::Matrix { env: *env, rows },
                    (_, Stop::End(end)) => {
                        return Err(self.error_at_last(&format!(
                            "`\\end{{{end}}}` closes `\\begin{{{env_name}}}`"
                        )));
                    }
                    (_, Stop::Eof) => {
                        return Err(self.error(&format!("`\\begin{{{env_name}}}` is not closed")));
                    }
                    (_, other) => {
                        return Err(self.error_at_last(&format!(
                            "{} inside `\\begin{{{env_name}}}`",
                            other.describe()
                        )));
                    }
                }
            }
            "end" => return Err(self.error_at_last("an `\\end` closes no `\\begin`")),
            "text" => Atom::Text(self.raw_group()?),
            _ => {
                if let Some((_, accent)) = ACCENTS.iter().find(|(n, _)| *n == name) {
                    Atom::Accent {
                        accent: *accent,
                        body: self.argument()?,
                    }
                } else if let Some((_, font)) = VARIANTS.iter().find(|(n, _)| *n == name) {
                    Atom::FontChange {
                        font: *font,
                        body: self.argument()?,
                    }
                } else if let Some((_, space)) = SPACES.iter().find(|(n, _)| *n == name) {
                    Atom::Space(*space)
                } else if let Some(op) = operator_name(&name) {
                    Atom::OperatorName(op)
                } else if let Some(c) = command(&name).filter(|_| !is_structure(&name)) {
                    Atom::Symbol {
                        ch: c.ch,
                        class: c.class,
                        name: Some(c.name),
                    }
                } else {
                    Atom::Unknown(name)
                }
            }
        })
    }
}

/// A character that may stand after `\left` or `\right`: an open, a close, a bar or a slash.
fn is_delimiter(ch: char) -> bool {
    matches!(class_of(ch), Class::Open | Class::Close)
        || matches!(
            ch,
            '|' | '/' | '\\' | '\u{2016}' | '\u{2215}' | '\u{2223}' | '\u{2225}'
        )
}

/// The prime a `'` stands for.
const PRIME: Atom = Atom::Symbol {
    ch: '\u{2032}',
    class: Class::Ord,
    name: Some("prime"),
};

/// Whether a superscript is primes alone, as `'`s give it.
fn all_primes(list: &List) -> bool {
    !list.is_empty() && list.iter().all(|a| *a == PRIME)
}

/// The name a symbol written with a backslash and one character keeps.
fn escaped_name(s: char) -> &'static str {
    match s {
        '{' => "{",
        '}' => "}",
        '|' => "|",
        '#' => "#",
        '$' => "$",
        '%' => "%",
        '&' => "&",
        _ => "_",
    }
}

/// A piece of written LaTeX, and whether it is set apart by a space on each side.
struct Piece {
    text: String,
    spaced: bool,
}

/// Join pieces: a space where either neighbour is spaced, where a control word would run into the
/// letter after it, and, for a file read by people, after a piece of more than one character
/// before a letter or a command, as in `w_i P_n`.
fn join(pieces: Vec<Piece>) -> String {
    let mut out = String::new();
    let mut last_spaced = false;
    let mut last_long = false;
    for (i, p) in pieces.into_iter().enumerate() {
        let next_word = starts_with_letter(&p.text) || p.text.starts_with('\\');
        if i > 0
            && (p.spaced
                || last_spaced
                || (ends_with_word(&out) && starts_with_letter(&p.text))
                || (last_long && next_word))
        {
            out.push(' ');
        }
        last_long = p.text.chars().nth(1).is_some();
        out.push_str(&p.text);
        last_spaced = p.spaced;
    }
    out
}

fn ends_with_word(s: &str) -> bool {
    match s.rfind('\\') {
        Some(i) => {
            let tail = &s[i + 1..];
            !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_alphabetic())
        }
        None => false,
    }
}

fn starts_with_letter(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

/// Rows joined by `\\` and cells by `&`, each set apart by one space, an empty cell writing
/// nothing.
fn write_grid(rows: &[Vec<List>]) -> String {
    let mut tokens = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        if r > 0 {
            tokens.push("\\\\".to_owned());
        }
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                tokens.push("&".to_owned());
            }
            let text = write_list(cell);
            if !text.is_empty() {
                tokens.push(text);
            }
        }
    }
    tokens.join(" ")
}

fn write_list(list: &List) -> String {
    join(list.iter().map(write_atom).collect())
}

/// An argument: bare when it is one symbol written as one character or one command, braced
/// otherwise.
fn write_argument(list: &List) -> String {
    match list.as_slice() {
        [Atom::Symbol { ch, name: None, .. }] if ch.is_ascii_alphanumeric() => ch.to_string(),
        [Atom::Symbol { name: Some(n), .. }] if n.bytes().all(|b| b.is_ascii_alphabetic()) => {
            format!("\\{n}")
        }
        _ => format!("{{{}}}", write_list(list)),
    }
}

fn braced(list: &List) -> String {
    format!("{{{}}}", write_list(list))
}

fn write_atom(atom: &Atom) -> Piece {
    let plain = |text: String| Piece {
        text,
        spaced: false,
    };
    match atom {
        Atom::Symbol { ch, class, name } => Piece {
            text: match name {
                Some(n) => format!("\\{n}"),
                None => write_char(*ch),
            },
            spaced: matches!(class, Class::Bin | Class::Rel),
        },
        Atom::Fraction { num, den } => plain(format!("\\frac{}{}", braced(num), braced(den))),
        Atom::Radical { index, body } => plain(match index {
            Some(index) => format!("\\sqrt[{}]{}", write_list(index), braced(body)),
            None => format!("\\sqrt{}", braced(body)),
        }),
        Atom::Scripts { nucleus, sub, sup } => {
            let mut text = match nucleus.as_slice() {
                // A lone `]` is braced, so that it does not close a radical's index.
                [one]
                    if !matches!(one, Atom::Scripts { .. } | Atom::Group(_))
                        && !matches!(
                            one,
                            Atom::Symbol {
                                ch: ']',
                                name: None,
                                ..
                            }
                        ) =>
                {
                    write_atom(one).text
                }
                _ => braced(nucleus),
            };
            if let Some(sub) = sub {
                text.push('_');
                text.push_str(&write_argument(sub));
            }
            if let Some(sup) = sup {
                text.push('^');
                text.push_str(&write_argument(sup));
            }
            plain(text)
        }
        Atom::Delimited { open, close, body } => {
            let pieces = vec![
                plain(format!("\\left{}", write_delim(*open))),
                plain(write_list(body)),
                plain(format!("\\right{}", write_delim(*close))),
            ];
            plain(join(
                pieces.into_iter().filter(|p| !p.text.is_empty()).collect(),
            ))
        }
        Atom::Accent { accent, body } => plain(format!("\\{}{}", accent.name(), braced(body))),
        Atom::OperatorName(name) => plain(format!("\\{name}")),
        Atom::FontChange { font, body } => plain(format!("\\{}{}", font.name(), braced(body))),
        Atom::Text(text) => plain(format!("\\text{{{text}}}")),
        Atom::Matrix { env, rows } => plain(format!(
            "\\begin{{{0}}} {1} \\end{{{0}}}",
            env.name(),
            write_grid(rows)
        )),
        Atom::Space(space) => plain(format!("\\{}", space.name())),
        Atom::Group(list) => plain(braced(list)),
        Atom::Unknown(name) => plain(format!("\\{name}")),
    }
}

fn write_delim(delim: Option<Delim>) -> String {
    match delim {
        None => ".".to_owned(),
        Some(Delim { name: Some(n), .. }) => format!("\\{n}"),
        Some(Delim { ch, name: None }) => ch.to_string(),
    }
}

/// A character written as itself, or with a backslash where the subset reads it otherwise.
fn write_char(ch: char) -> String {
    match ch {
        '{' | '}' | '$' | '#' | '%' | '&' | '_' => format!("\\{ch}"),
        _ => ch.to_string(),
    }
}
