//! The commands of the protocol: their grammar, parsed from a line and printed back to one.

use std::fmt;

use rootpoint_model::{AssetId, Axis, HAlign, ImageFit, NodeId, Rgba, Side, SlideId, VAlign};

/// One command of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    New,
    Save {
        path: String,
    },
    Load {
        path: String,
    },
    Tree {
        slide: Option<SlideId>,
    },
    /// Draw a slide to a PNG file, 1920×1080 unless a size is given.
    Render {
        slide: SlideId,
        path: String,
        size: Option<(u16, u16)>,
    },
    AddSlide {
        at: Option<usize>,
    },
    RemoveSlide {
        slide: SlideId,
    },
    MoveSlide {
        slide: SlideId,
        to: usize,
    },
    DuplicateSlide {
        slide: SlideId,
    },
    Split {
        node: NodeId,
        axis: Axis,
        at: f64,
        side: Side,
    },
    Remove {
        node: NodeId,
    },
    Divider {
        split: NodeId,
        index: usize,
        pos: f64,
    },
    Swap {
        a: NodeId,
        b: NodeId,
    },
    /// Set an area's text; the options change the area's current style, or the default style
    /// when the area holds no text.
    Text {
        area: NodeId,
        text: String,
        style: Vec<StyleOption>,
    },
    Image {
        area: NodeId,
        asset: AssetId,
        fit: ImageFit,
    },
    Clear {
        area: NodeId,
    },
    Asset {
        path: String,
    },
    RemoveAsset {
        asset: AssetId,
    },
    /// Replace the document by the built-in sample.
    Sample,
    Undo,
    Redo,
    /// Open a group: the edits until its `end` are one undo step.
    Begin,
    End,
    /// Revert the edits of the open group, and close it with no step.
    Cancel,
    /// Ask for the authority to edit a running editor, which its window or its control socket
    /// holds; a headless session has one controller, which always holds it.
    Control,
    /// Give the authority to edit back to the window; refused while a group is open.
    Release,
    /// Select a node, and show its slide.
    Select {
        node: NodeId,
    },
    /// Show a slide, and select no node.
    SelectSlide {
        slide: SlideId,
    },
    Deselect,
    /// Enter presenter mode on a slide, the first when none is named.
    Present {
        slide: Option<SlideId>,
    },
    Next,
    Prev,
    Stop,
    View,
}

/// One option of a text's style.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleOption {
    Size(f64),
    /// `bold` or `bold=off`.
    Bold(bool),
    /// `italic` or `italic=off`.
    Italic(bool),
    Colour(Rgba),
    Align(HAlign),
    VAlign(VAlign),
    /// `sans`, the built-in font, or an asset id.
    Font(Option<AssetId>),
}

/// Why a line is not a command.
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Parse one line, which is neither blank nor a comment, into a command.
pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    let words = split_words(line)?;
    let mut args = Args::new(&words);
    let verb = args.word("a verb")?;
    let command = match verb.as_str() {
        "new" => Command::New,
        "save" => Command::Save { path: args.path()? },
        "load" => Command::Load { path: args.path()? },
        "tree" => Command::Tree {
            slide: args.optional(parse_slide)?,
        },
        "render" => {
            let slide = parse_slide(&args.word("a slide")?)?;
            let path = args.path()?;
            let size = match args.optional(parse_pixels)? {
                None => None,
                Some(w) => Some((w, parse_pixels(&args.word("a height")?)?)),
            };
            Command::Render { slide, path, size }
        }
        "add-slide" => Command::AddSlide {
            at: args.optional(parse_index)?,
        },
        "remove-slide" => Command::RemoveSlide {
            slide: parse_slide(&args.word("a slide")?)?,
        },
        "move-slide" => Command::MoveSlide {
            slide: parse_slide(&args.word("a slide")?)?,
            to: parse_index(&args.word("an index")?)?,
        },
        "duplicate-slide" => Command::DuplicateSlide {
            slide: parse_slide(&args.word("a slide")?)?,
        },
        "split" => Command::Split {
            node: parse_node(&args.word("a node")?)?,
            axis: parse_axis(&args.word("`beside` or `stacked`")?)?,
            at: parse_number(&args.word("a position")?)?,
            side: args.optional(parse_side)?.unwrap_or(Side::After),
        },
        "remove" => Command::Remove {
            node: parse_node(&args.word("a node")?)?,
        },
        "divider" => Command::Divider {
            split: parse_node(&args.word("a split")?)?,
            index: parse_index(&args.word("an index")?)?,
            pos: parse_number(&args.word("a position")?)?,
        },
        "swap" => Command::Swap {
            a: parse_node(&args.word("an area")?)?,
            b: parse_node(&args.word("an area")?)?,
        },
        "text" => {
            let area = parse_node(&args.word("an area")?)?;
            let text = args.quoted("a quoted text")?;
            let mut style = Vec::new();
            while let Some(option) = args.optional(parse_style_option)? {
                style.push(option);
            }
            Command::Text { area, text, style }
        }
        "image" => Command::Image {
            area: parse_node(&args.word("an area")?)?,
            asset: parse_asset(&args.word("an asset")?)?,
            fit: args.optional(parse_fit)?.unwrap_or(ImageFit::Fit),
        },
        "clear" => Command::Clear {
            area: parse_node(&args.word("an area")?)?,
        },
        "asset" => Command::Asset { path: args.path()? },
        "remove-asset" => Command::RemoveAsset {
            asset: parse_asset(&args.word("an asset")?)?,
        },
        "sample" => Command::Sample,
        "undo" => Command::Undo,
        "redo" => Command::Redo,
        "begin" => Command::Begin,
        "end" => Command::End,
        "cancel" => Command::Cancel,
        "control" => Command::Control,
        "release" => Command::Release,
        "select" => {
            let target = args.word("a node or a slide")?;
            if target.starts_with('s') {
                Command::SelectSlide {
                    slide: parse_slide(&target)?,
                }
            } else {
                Command::Select {
                    node: parse_node(&target)?,
                }
            }
        }
        "deselect" => Command::Deselect,
        "present" => Command::Present {
            slide: args.optional(parse_slide)?,
        },
        "next" => Command::Next,
        "prev" => Command::Prev,
        "stop" => Command::Stop,
        "view" => Command::View,
        other => return Err(ParseError(format!("unknown command `{other}`"))),
    };
    args.finish()?;
    Ok(command)
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::New => f.write_str("new"),
            Command::Save { path } => write!(f, "save {}", quote(path)),
            Command::Load { path } => write!(f, "load {}", quote(path)),
            Command::Tree { slide: None } => f.write_str("tree"),
            Command::Tree { slide: Some(s) } => write!(f, "tree {s}"),
            Command::Render { slide, path, size } => {
                write!(f, "render {slide} {}", quote(path))?;
                if let Some((w, h)) = size {
                    write!(f, " {w} {h}")?;
                }
                Ok(())
            }
            Command::AddSlide { at: None } => f.write_str("add-slide"),
            Command::AddSlide { at: Some(at) } => write!(f, "add-slide {at}"),
            Command::RemoveSlide { slide } => write!(f, "remove-slide {slide}"),
            Command::MoveSlide { slide, to } => write!(f, "move-slide {slide} {to}"),
            Command::DuplicateSlide { slide } => write!(f, "duplicate-slide {slide}"),
            Command::Split {
                node,
                axis,
                at,
                side,
            } => {
                let axis = match axis {
                    Axis::LeftToRight => "beside",
                    Axis::TopToBottom => "stacked",
                };
                let side = match side {
                    Side::Before => "before",
                    Side::After => "after",
                };
                write!(f, "split {node} {axis} {at} {side}")
            }
            Command::Remove { node } => write!(f, "remove {node}"),
            Command::Divider { split, index, pos } => write!(f, "divider {split} {index} {pos}"),
            Command::Swap { a, b } => write!(f, "swap {a} {b}"),
            Command::Text { area, text, style } => {
                write!(f, "text {area} {}", quote_always(text))?;
                for option in style {
                    write!(f, " {option}")?;
                }
                Ok(())
            }
            Command::Image { area, asset, fit } => {
                let fit = match fit {
                    ImageFit::Fit => "fit",
                    ImageFit::Fill => "fill",
                };
                write!(f, "image {area} {asset} {fit}")
            }
            Command::Clear { area } => write!(f, "clear {area}"),
            Command::Asset { path } => write!(f, "asset {}", quote(path)),
            Command::RemoveAsset { asset } => write!(f, "remove-asset {asset}"),
            Command::Sample => f.write_str("sample"),
            Command::Undo => f.write_str("undo"),
            Command::Redo => f.write_str("redo"),
            Command::Begin => f.write_str("begin"),
            Command::End => f.write_str("end"),
            Command::Cancel => f.write_str("cancel"),
            Command::Control => f.write_str("control"),
            Command::Release => f.write_str("release"),
            Command::Select { node } => write!(f, "select {node}"),
            Command::SelectSlide { slide } => write!(f, "select {slide}"),
            Command::Deselect => f.write_str("deselect"),
            Command::Present { slide: None } => f.write_str("present"),
            Command::Present { slide: Some(s) } => write!(f, "present {s}"),
            Command::Next => f.write_str("next"),
            Command::Prev => f.write_str("prev"),
            Command::Stop => f.write_str("stop"),
            Command::View => f.write_str("view"),
        }
    }
}

impl fmt::Display for StyleOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StyleOption::Size(size) => write!(f, "size={size}"),
            StyleOption::Bold(true) => f.write_str("bold"),
            StyleOption::Bold(false) => f.write_str("bold=off"),
            StyleOption::Italic(true) => f.write_str("italic"),
            StyleOption::Italic(false) => f.write_str("italic=off"),
            StyleOption::Colour(c) => {
                write!(f, "colour=#{:02x}{:02x}{:02x}", c.r, c.g, c.b)?;
                if c.a != 255 {
                    write!(f, "{:02x}", c.a)?;
                }
                Ok(())
            }
            StyleOption::Align(a) => f.write_str(match a {
                HAlign::Left => "align=left",
                HAlign::Center => "align=center",
                HAlign::Right => "align=right",
            }),
            StyleOption::VAlign(v) => f.write_str(match v {
                VAlign::Top => "valign=top",
                VAlign::Middle => "valign=middle",
                VAlign::Bottom => "valign=bottom",
            }),
            StyleOption::Font(None) => f.write_str("font=sans"),
            StyleOption::Font(Some(a)) => write!(f, "font={a}"),
        }
    }
}

/// A word of a line: a run of characters without spaces, or a double-quoted string with `\"`,
/// `\\` and `\n` escapes. A quoted word is kept apart, so `"bold"` is a text and not a flag.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Word {
    Bare(String),
    Quoted(String),
}

impl Word {
    pub(crate) fn text(&self) -> &str {
        match self {
            Word::Bare(s) | Word::Quoted(s) => s,
        }
    }
}

pub(crate) fn split_words(line: &str) -> Result<Vec<Word>, ParseError> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' {
            chars.next();
            let mut text = String::new();
            loop {
                match chars.next() {
                    None => return Err(ParseError("an unclosed quote".to_owned())),
                    Some('"') => break,
                    Some('\\') => match chars.next() {
                        Some('"') => text.push('"'),
                        Some('\\') => text.push('\\'),
                        Some('n') => text.push('\n'),
                        Some(other) => {
                            return Err(ParseError(format!("an unknown escape `\\{other}`")));
                        }
                        None => return Err(ParseError("an unclosed quote".to_owned())),
                    },
                    Some(other) => text.push(other),
                }
            }
            words.push(Word::Quoted(text));
        } else {
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                text.push(c);
                chars.next();
            }
            words.push(Word::Bare(text));
        }
    }
    Ok(words)
}

/// A word as the printer writes it: bare when it can be read back bare, quoted otherwise.
pub(crate) fn quote(text: &str) -> String {
    let bare =
        !text.is_empty() && !text.contains(|c: char| c.is_whitespace() || c == '"' || c == '\\');
    if bare {
        return text.to_owned();
    }
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The words of a command after its verb, consumed in order.
pub(crate) struct Args<'a> {
    words: &'a [Word],
    next: usize,
}

impl<'a> Args<'a> {
    pub(crate) fn new(words: &'a [Word]) -> Self {
        Args { words, next: 0 }
    }

    /// The next word, which must be bare: a verb, an id, a number or a keyword.
    pub(crate) fn word(&mut self, what: &str) -> Result<String, ParseError> {
        match self.words.get(self.next) {
            Some(Word::Bare(text)) => {
                self.next += 1;
                Ok(text.clone())
            }
            Some(Word::Quoted(_)) => Err(ParseError(format!("a quoted word instead of {what}"))),
            None => Err(ParseError(format!("missing {what}"))),
        }
    }

    /// The next word, bare or quoted, as a path.
    pub(crate) fn path(&mut self) -> Result<String, ParseError> {
        let word = self
            .words
            .get(self.next)
            .ok_or_else(|| ParseError("missing a path".to_owned()))?;
        self.next += 1;
        Ok(word.text().to_owned())
    }

    /// The next word parsed by `parse`, or `None` when the line has no more bare words. A quoted
    /// word is left for `finish` to refuse.
    pub(crate) fn optional<T>(
        &mut self,
        parse: impl Fn(&str) -> Result<T, ParseError>,
    ) -> Result<Option<T>, ParseError> {
        match self.words.get(self.next) {
            Some(Word::Bare(text)) => {
                self.next += 1;
                parse(text).map(Some)
            }
            _ => Ok(None),
        }
    }

    /// The next word, which must be a quoted string.
    pub(crate) fn quoted(&mut self, what: &str) -> Result<String, ParseError> {
        match self.words.get(self.next) {
            Some(Word::Quoted(text)) => {
                self.next += 1;
                Ok(text.clone())
            }
            _ => Err(ParseError(format!("missing {what}"))),
        }
    }

    /// Fail when words are left over.
    pub(crate) fn finish(&self) -> Result<(), ParseError> {
        match self.words.get(self.next) {
            None => Ok(()),
            Some(extra) => Err(ParseError(format!("unexpected `{}`", extra.text()))),
        }
    }
}

/// A number written as the printer writes it: decimal digits, with no sign and no leading zero.
fn parse_digits<T: std::str::FromStr>(text: &str) -> Option<T> {
    let canonical = !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    canonical.then(|| text.parse().ok()).flatten()
}

fn parse_id(text: &str, prefix: char) -> Result<u64, ParseError> {
    text.strip_prefix(prefix)
        .and_then(parse_digits)
        .ok_or_else(|| ParseError(format!("`{text}` is not an id `{prefix}<n>`")))
}

pub(crate) fn parse_slide(text: &str) -> Result<SlideId, ParseError> {
    parse_id(text, 's').map(SlideId)
}

pub(crate) fn parse_node(text: &str) -> Result<NodeId, ParseError> {
    parse_id(text, 'n').map(NodeId)
}

pub(crate) fn parse_asset(text: &str) -> Result<AssetId, ParseError> {
    parse_id(text, 'a').map(AssetId)
}

/// A text always quoted, so that a text such as `bold` is never read as an option.
pub(crate) fn quote_always(text: &str) -> String {
    let q = quote(text);
    if q.starts_with('"') {
        q
    } else {
        format!("\"{q}\"")
    }
}

fn parse_index(text: &str) -> Result<usize, ParseError> {
    parse_digits(text).ok_or_else(|| ParseError(format!("`{text}` is not an index")))
}

fn parse_pixels(text: &str) -> Result<u16, ParseError> {
    parse_digits(text).ok_or_else(|| ParseError(format!("`{text}` is not a size in pixels")))
}

fn parse_number(text: &str) -> Result<f64, ParseError> {
    text.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| ParseError(format!("`{text}` is not a number")))
}

// The words are `design@rootpoint-protocol@protocol-axis-words`.
fn parse_axis(text: &str) -> Result<Axis, ParseError> {
    match text {
        "beside" => Ok(Axis::LeftToRight),
        "stacked" => Ok(Axis::TopToBottom),
        other => Err(ParseError(format!(
            "`{other}` is not `beside` or `stacked`"
        ))),
    }
}

fn parse_side(text: &str) -> Result<Side, ParseError> {
    match text {
        "before" => Ok(Side::Before),
        "after" => Ok(Side::After),
        other => Err(ParseError(format!("`{other}` is not `before` or `after`"))),
    }
}

fn parse_fit(text: &str) -> Result<ImageFit, ParseError> {
    match text {
        "fit" => Ok(ImageFit::Fit),
        "fill" => Ok(ImageFit::Fill),
        other => Err(ParseError(format!("`{other}` is not `fit` or `fill`"))),
    }
}

fn parse_style_option(text: &str) -> Result<StyleOption, ParseError> {
    let bad = || ParseError(format!("`{text}` is not a style option"));
    match text.split_once('=') {
        None => match text {
            "bold" => Ok(StyleOption::Bold(true)),
            "italic" => Ok(StyleOption::Italic(true)),
            _ => Err(bad()),
        },
        Some(("size", v)) => parse_number(v)
            .ok()
            .filter(|n| *n > 0.0)
            .map(StyleOption::Size)
            .ok_or_else(bad),
        Some(("colour", v)) => parse_colour(v).map(StyleOption::Colour).ok_or_else(bad),
        Some(("align", v)) => match v {
            "left" => Ok(StyleOption::Align(HAlign::Left)),
            "center" => Ok(StyleOption::Align(HAlign::Center)),
            "right" => Ok(StyleOption::Align(HAlign::Right)),
            _ => Err(bad()),
        },
        Some(("valign", v)) => match v {
            "top" => Ok(StyleOption::VAlign(VAlign::Top)),
            "middle" => Ok(StyleOption::VAlign(VAlign::Middle)),
            "bottom" => Ok(StyleOption::VAlign(VAlign::Bottom)),
            _ => Err(bad()),
        },
        Some(("bold", "off")) => Ok(StyleOption::Bold(false)),
        Some(("italic", "off")) => Ok(StyleOption::Italic(false)),
        Some(("font", "sans")) => Ok(StyleOption::Font(None)),
        Some(("font", v)) => parse_asset(v).map(|a| StyleOption::Font(Some(a))),
        _ => Err(bad()),
    }
}

/// `#rrggbb` or `#rrggbbaa`.
fn parse_colour(text: &str) -> Option<Rgba> {
    let hex = text.strip_prefix('#')?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Rgba {
        r: byte(0)?,
        g: byte(2)?,
        b: byte(4)?,
        a: if hex.len() == 8 { byte(6)? } else { 255 },
    })
}
