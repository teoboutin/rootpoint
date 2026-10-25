//! The commands of the protocol: their grammar, parsed from a line and printed back to one.

use std::fmt;

use std::ops::Range;

use rootpoint_math::Formula;
use rootpoint_model::{
    AssetId, Axis, EquationSize, FontRef, HAlign, ImageFit, NodeId, Replacement, Rgba, Side,
    SlideId, SpanId, SpanStyle, TextStyle, VAlign,
};

/// One command of the protocol.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    New,
    /// Write the presentation to its store: `presentation.ron` at a workspace's root.
    Save,
    /// Open a directory of the host as a workspace: its `presentation.ron`, or an empty
    /// presentation when it has none, and its assets read from under it.
    Open {
        dir: String,
    },
    /// Open a copy of a directory's presentation in memory: its `presentation.ron`, and the files
    /// of its assets copied into the session's map.
    OpenCopy {
        dir: String,
    },
    /// Add a file of the host as an asset: at its path below a workspace's root, or copied into
    /// the store, at `to` or under its file name at the root.
    Import {
        from: String,
        to: Option<String>,
    },
    /// The store, and the count of undo steps since the last open or save.
    Store,
    /// Write the presentation and its assets' files as a pack; the store is unchanged.
    Pack {
        file: String,
    },
    /// Open a pack in memory: its entries become the session's map.
    OpenPack {
        file: String,
    },
    /// Write a presentation held in memory as a workspace at an empty directory, which it then is.
    Unpack {
        dir: String,
    },
    /// Read `presentation.ron` at a workspace's root again, and make it the document, one undo
    /// step.
    Reload,
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
    /// Set an area's text tree, every span given a new id, or, with no tree, keep it and its ids
    /// and change the style alone, which an image or an equation area refuses. Its style is the
    /// area's current one, or the style a new area takes when the area holds no text; `style=` names a style, `custom` copies the values drawn
    /// into the text's own, and any other option changes the values drawn, which makes a named
    /// style custom.
    Text {
        area: NodeId,
        root: Option<Vec<TreeItem>>,
        style: Vec<StyleOption>,
    },
    /// Insert characters in a text node at an offset.
    Insert {
        node: TextTarget,
        offset: usize,
        text: String,
    },
    /// Remove a range of a text node's offsets; a span in the range goes whole.
    Delete {
        node: TextTarget,
        range: Range<usize>,
    },
    /// Put a range of a text node's offsets into a new span, whose style the options give over a
    /// custom style that sets nothing.
    Wrap {
        node: TextTarget,
        range: Range<usize>,
        style: Vec<SpanOption>,
    },
    /// Replace a span by its content.
    Unwrap {
        span: SpanId,
    },
    /// Change a span's style, the options applied over its current one.
    Span {
        span: SpanId,
        style: Vec<SpanOption>,
    },
    /// Cut a span at an offset inside it into two spans of its style.
    SplitSpan {
        span: SpanId,
        offset: usize,
    },
    Image {
        area: NodeId,
        asset: AssetId,
        fit: ImageFit,
    },
    /// Set an area's content to an equation of this formula, or change the equation it holds:
    /// each option given replaces the equation's own, an option not given keeping the current
    /// equation's, or a new equation's, numbered, fitted, in the Body style's colour.
    Equation {
        area: NodeId,
        formula: Formula,
        options: Vec<EquationOption>,
    },
    /// Empty an area's text. A text keeps its style, so that clearing it changes its characters
    /// only; an image area, which has no text style to keep, becomes a text in the style a new area
    /// takes.
    Clear {
        area: NodeId,
    },
    /// Move an asset's file to another asset path, which no file holds; the asset keeps its id.
    AssetMove {
        asset: AssetId,
        to: String,
    },
    /// Give an asset the path of a file already in the store, and move no file: the repair of a
    /// missing asset.
    AssetRelink {
        asset: AssetId,
        to: String,
    },
    /// Replace the bytes of an asset's file by those of a file of the host; the old ones go to the
    /// trash.
    AssetReplace {
        asset: AssetId,
        from: String,
    },
    /// Remove an asset, its file going to the trash. With `into`, what names it names that asset
    /// instead; with none, the removal is refused while anything names it.
    AssetRemove {
        asset: AssetId,
        into: Option<AssetId>,
    },
    /// Remove every asset nothing names, one undo step.
    AssetRemoveUnused,
    /// List the assets, or the store's files no asset names, a query.
    Assets {
        filter: AssetFilter,
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
    /// Give the authority to edit back to the window; refused while the client holding it has a
    /// group open.
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
    /// Show the final result in the editor, with no mark and no selection, or leave it.
    Preview {
        on: bool,
    },
    /// Enter presenter mode on a slide, the first when none is named.
    Present {
        slide: Option<SlideId>,
    },
    Next,
    Prev,
    Stop,
    View,
    /// Change the document's table of named styles.
    Style(StyleCommand),
    /// List the named styles, a query.
    Styles,
    /// Change the document's table of named span styles.
    SpanStyle(SpanStyleCommand),
    /// List the named span styles, a query.
    SpanStyles,
}

/// A text node a command names: the root of an area's text, by the area's id, or a span, by its
/// own, which is unique in the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextTarget {
    Root(NodeId),
    Span(SpanId),
}

/// An item of a text tree as `text` takes it and `tree` prints it.
#[derive(Clone, Debug, PartialEq)]
pub enum TreeItem {
    Chars(String),
    Span(TreeSpan),
}

/// A span of a text tree: its id, which `tree` prints and `text` ignores, since it gives every span
/// a new one; its style's options, applied over a custom style that sets nothing; and its items.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeSpan {
    pub id: Option<SpanId>,
    pub style: Vec<SpanOption>,
    pub items: Vec<TreeItem>,
}

/// One option of a span's style. Alignment is the area's alone, so a span has none.
#[derive(Clone, Debug, PartialEq)]
pub enum SpanOption {
    /// `style="<name>"`: name a span style of the table.
    Named(String),
    /// `custom`: copy the values the span's named style sets into its own, the others still taken
    /// from its parent.
    Custom,
    Size(f64),
    Bold(bool),
    Italic(bool),
    Colour(Rgba),
    /// `sans`, the built-in font, or an asset id.
    Font(Option<AssetId>),
    /// `<key>=inherit`: set no value, so the span draws with its parent's.
    Inherit(SpanField),
}

/// A value a span style sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanField {
    Font,
    Size,
    Bold,
    Italic,
    Colour,
}

impl SpanField {
    fn key(self) -> &'static str {
        match self {
            SpanField::Font => "font",
            SpanField::Size => "size",
            SpanField::Bold => "bold",
            SpanField::Italic => "italic",
            SpanField::Colour => "colour",
        }
    }
}

impl SpanOption {
    /// The options that set the values `style` sets, in a fixed order: size, bold, italic, colour,
    /// font. A built-in family the binary does not carry, which only a file edited by hand holds,
    /// has no option, so it is left out.
    pub fn values_of(style: &SpanStyle) -> Vec<SpanOption> {
        let mut options = Vec::new();
        options.extend(style.size.map(SpanOption::Size));
        options.extend(style.bold.map(SpanOption::Bold));
        options.extend(style.italic.map(SpanOption::Italic));
        options.extend(style.colour.map(SpanOption::Colour));
        match &style.font {
            Some(FontRef::Builtin(name)) if name == "sans" => options.push(SpanOption::Font(None)),
            Some(FontRef::Asset(id)) => options.push(SpanOption::Font(Some(*id))),
            Some(FontRef::Builtin(_)) | None => {}
        }
        options
    }
}

/// A change of the table of named span styles.
#[derive(Clone, Debug, PartialEq)]
pub enum SpanStyleCommand {
    /// Add a span style, which sets the values its options give.
    Add {
        name: String,
        options: Vec<SpanOption>,
    },
    /// Change a span style's values, the options applied over its current ones.
    Set {
        name: String,
        options: Vec<SpanOption>,
    },
    /// Rename a span style, and every span naming it with it.
    Rename { from: String, to: String },
    /// Remove a span style; the spans naming it take `into`, and with none the removal is refused
    /// while a span names it.
    Remove {
        name: String,
        into: Option<Replacement>,
    },
}

/// Which entries `assets` lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetFilter {
    All,
    /// The assets no area and no style names.
    Unused,
    /// The assets whose file the store cannot read.
    Missing,
    /// The files of the store no asset names, its own files left out.
    Untracked,
}

/// A change of the table of named styles.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleCommand {
    /// Add a style, whose options change `TextStyle::default()`.
    Add {
        name: String,
        options: Vec<StyleOption>,
    },
    /// Change a style's values, which every text naming it then draws with.
    Set {
        name: String,
        options: Vec<StyleOption>,
    },
    /// Rename a style, and every text naming it with it.
    Rename { from: String, to: String },
    /// Remove a style; the texts naming it take `into`, and with none the removal is refused while
    /// a text names it.
    Remove {
        name: String,
        into: Option<Replacement>,
    },
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
    /// `style="<name>"`: name a style of the table.
    Named(String),
    /// `custom`: copy the values drawn into the text's own.
    Custom,
}

impl StyleOption {
    /// The options that give `TextStyle::default()` the values of `style`, every value in a fixed
    /// order, as `style add` takes them. A built-in family the binary does not carry, which only a
    /// file edited by hand holds, has no option, so it is left out.
    pub fn values_of(style: &TextStyle) -> Vec<StyleOption> {
        let mut options = vec![
            StyleOption::Size(style.size),
            StyleOption::Bold(style.bold),
            StyleOption::Italic(style.italic),
            StyleOption::Colour(style.colour),
            StyleOption::Align(style.h_align),
            StyleOption::VAlign(style.v_align),
        ];
        match &style.font {
            FontRef::Builtin(name) if name == "sans" => options.push(StyleOption::Font(None)),
            FontRef::Builtin(_) => {}
            FontRef::Asset(id) => options.push(StyleOption::Font(Some(*id))),
        }
        options
    }
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
        "save" => Command::Save,
        "open" => Command::Open { dir: args.path()? },
        "open-copy" => Command::OpenCopy { dir: args.path()? },
        "import" => Command::Import {
            from: args.path()?,
            to: args.optional_path()?,
        },
        "store" => Command::Store,
        "pack" => Command::Pack { file: args.path()? },
        "open-pack" => Command::OpenPack { file: args.path()? },
        "unpack" => Command::Unpack { dir: args.path()? },
        "reload" => Command::Reload,
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
            let root = match args.peek() {
                Some(Word::Quoted(_)) => {
                    let text = args.quoted("a quoted text")?;
                    Some(vec![TreeItem::Chars(text)])
                }
                Some(Word::Punct('[')) => Some(parse_tree(&mut args)?),
                _ => None,
            };
            let mut style = Vec::new();
            while let Some(option) = args.optional(parse_style_option)? {
                style.push(option);
            }
            if root.is_none() && style.is_empty() {
                return Err(ParseError("missing a text or a style option".to_owned()));
            }
            Command::Text { area, root, style }
        }
        "insert" => Command::Insert {
            node: parse_target(&args.word("a text node")?)?,
            offset: parse_index(&args.word("an offset")?)?,
            text: args.quoted("a quoted text")?,
        },
        "delete" => Command::Delete {
            node: parse_target(&args.word("a text node")?)?,
            range: parse_range(&args.word("a range")?)?,
        },
        "wrap" => Command::Wrap {
            node: parse_target(&args.word("a text node")?)?,
            range: parse_range(&args.word("a range")?)?,
            style: span_options(&mut args)?,
        },
        "unwrap" => Command::Unwrap {
            span: parse_span(&args.word("a span")?)?,
        },
        "span" => {
            let span = parse_span(&args.word("a span")?)?;
            let style = span_options(&mut args)?;
            if style.is_empty() {
                return Err(ParseError("missing a span's style option".to_owned()));
            }
            Command::Span { span, style }
        }
        "split-span" => Command::SplitSpan {
            span: parse_span(&args.word("a span")?)?,
            offset: parse_index(&args.word("an offset")?)?,
        },
        "span-styles" => Command::SpanStyles,
        "span-style" => Command::SpanStyle(
            match args.word("`add`, `set`, `rename` or `remove`")?.as_str() {
                "add" => SpanStyleCommand::Add {
                    name: args.quoted("a quoted span style name")?,
                    options: span_value_options(&mut args)?,
                },
                "set" => SpanStyleCommand::Set {
                    name: args.quoted("a quoted span style name")?,
                    options: span_value_options(&mut args)?,
                },
                "rename" => SpanStyleCommand::Rename {
                    from: args.quoted("a quoted span style name")?,
                    to: args.quoted("a quoted new name")?,
                },
                "remove" => SpanStyleCommand::Remove {
                    name: args.quoted("a quoted span style name")?,
                    into: args.optional(parse_into)?,
                },
                other => {
                    return Err(ParseError(format!(
                        "`{other}` is not `add`, `set`, `rename` or `remove`"
                    )));
                }
            },
        ),
        "image" => Command::Image {
            area: parse_node(&args.word("an area")?)?,
            asset: parse_asset(&args.word("an asset")?)?,
            fit: args.optional(parse_fit)?.unwrap_or(ImageFit::Fit),
        },
        "equation" => Command::Equation {
            area: parse_node(&args.word("an area")?)?,
            formula: args.formula()?,
            options: {
                let mut options = Vec::new();
                while let Some(option) = args.optional(parse_equation_option)? {
                    options.push(option);
                }
                options
            },
        },
        "clear" => Command::Clear {
            area: parse_node(&args.word("an area")?)?,
        },
        "asset-move" => Command::AssetMove {
            asset: parse_asset(&args.word("an asset")?)?,
            to: args.path()?,
        },
        "asset-relink" => Command::AssetRelink {
            asset: parse_asset(&args.word("an asset")?)?,
            to: args.path()?,
        },
        "asset-replace" => Command::AssetReplace {
            asset: parse_asset(&args.word("an asset")?)?,
            from: args.path()?,
        },
        "asset-remove" => Command::AssetRemove {
            asset: parse_asset(&args.word("an asset")?)?,
            into: args.optional(parse_asset_into)?,
        },
        "asset-remove-unused" => Command::AssetRemoveUnused,
        "assets" => Command::Assets {
            filter: args.optional(parse_filter)?.unwrap_or(AssetFilter::All),
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
        "preview" => Command::Preview {
            on: match args.word("`on` or `off`")?.as_str() {
                "on" => true,
                "off" => false,
                other => return Err(ParseError(format!("`{other}` is not `on` or `off`"))),
            },
        },
        "present" => Command::Present {
            slide: args.optional(parse_slide)?,
        },
        "next" => Command::Next,
        "prev" => Command::Prev,
        "stop" => Command::Stop,
        "view" => Command::View,
        "styles" => Command::Styles,
        "style" => Command::Style(
            match args.word("`add`, `set`, `rename` or `remove`")?.as_str() {
                "add" => StyleCommand::Add {
                    name: args.quoted("a quoted style name")?,
                    options: value_options(&mut args)?,
                },
                "set" => StyleCommand::Set {
                    name: args.quoted("a quoted style name")?,
                    options: value_options(&mut args)?,
                },
                "rename" => StyleCommand::Rename {
                    from: args.quoted("a quoted style name")?,
                    to: args.quoted("a quoted new name")?,
                },
                "remove" => StyleCommand::Remove {
                    name: args.quoted("a quoted style name")?,
                    into: args.optional(parse_into)?,
                },
                other => {
                    return Err(ParseError(format!(
                        "`{other}` is not `add`, `set`, `rename` or `remove`"
                    )));
                }
            },
        ),
        other => return Err(ParseError(format!("unknown command `{other}`"))),
    };
    args.finish()?;
    Ok(command)
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::New => f.write_str("new"),
            Command::Save => f.write_str("save"),
            Command::Open { dir } => write!(f, "open {}", quote(dir)),
            Command::OpenCopy { dir } => write!(f, "open-copy {}", quote(dir)),
            Command::Import { from, to: None } => write!(f, "import {}", quote(from)),
            Command::Import { from, to: Some(to) } => {
                write!(f, "import {} {}", quote(from), quote(to))
            }
            Command::Store => f.write_str("store"),
            Command::Pack { file } => write!(f, "pack {}", quote(file)),
            Command::OpenPack { file } => write!(f, "open-pack {}", quote(file)),
            Command::Unpack { dir } => write!(f, "unpack {}", quote(dir)),
            Command::Reload => f.write_str("reload"),
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
            Command::Text { area, root, style } => {
                write!(f, "text {area}")?;
                if let Some(root) = root {
                    write!(f, " {}", tree_text(root))?;
                }
                for option in style {
                    write!(f, " {option}")?;
                }
                Ok(())
            }
            Command::Insert { node, offset, text } => {
                write!(f, "insert {node} {offset} {}", quote_always(text))
            }
            Command::Delete { node, range } => {
                write!(f, "delete {node} {}..{}", range.start, range.end)
            }
            Command::Wrap { node, range, style } => {
                write!(f, "wrap {node} {}..{}", range.start, range.end)?;
                style.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::Unwrap { span } => write!(f, "unwrap {span}"),
            Command::Span { span, style } => {
                write!(f, "span {span}")?;
                style.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::SplitSpan { span, offset } => write!(f, "split-span {span} {offset}"),
            Command::SpanStyles => f.write_str("span-styles"),
            Command::SpanStyle(SpanStyleCommand::Add { name, options }) => {
                write!(f, "span-style add {}", quote_always(name))?;
                options.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::SpanStyle(SpanStyleCommand::Set { name, options }) => {
                write!(f, "span-style set {}", quote_always(name))?;
                options.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::SpanStyle(SpanStyleCommand::Rename { from, to }) => write!(
                f,
                "span-style rename {} {}",
                quote_always(from),
                quote_always(to)
            ),
            Command::SpanStyle(SpanStyleCommand::Remove { name, into }) => {
                write!(f, "span-style remove {}", quote_always(name))?;
                match into {
                    None => Ok(()),
                    Some(Replacement::Custom) => f.write_str(" into=custom"),
                    Some(Replacement::Named(other)) => write!(f, " into={}", quote_always(other)),
                }
            }
            Command::Image { area, asset, fit } => {
                let fit = match fit {
                    ImageFit::Fit => "fit",
                    ImageFit::Fill => "fill",
                };
                write!(f, "image {area} {asset} {fit}")
            }
            Command::Equation {
                area,
                formula,
                options,
            } => {
                write!(f, "equation {area} ${}$", formula.to_latex())?;
                for option in options {
                    write!(f, " {option}")?;
                }
                Ok(())
            }
            Command::Clear { area } => write!(f, "clear {area}"),
            Command::AssetMove { asset, to } => write!(f, "asset-move {asset} {}", quote(to)),
            Command::AssetRelink { asset, to } => {
                write!(f, "asset-relink {asset} {}", quote(to))
            }
            Command::AssetReplace { asset, from } => {
                write!(f, "asset-replace {asset} {}", quote(from))
            }
            Command::AssetRemove { asset, into: None } => write!(f, "asset-remove {asset}"),
            Command::AssetRemove {
                asset,
                into: Some(into),
            } => write!(f, "asset-remove {asset} into={into}"),
            Command::AssetRemoveUnused => f.write_str("asset-remove-unused"),
            Command::Assets { filter } => f.write_str(match filter {
                AssetFilter::All => "assets",
                AssetFilter::Unused => "assets unused",
                AssetFilter::Missing => "assets missing",
                AssetFilter::Untracked => "assets untracked",
            }),
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
            Command::Preview { on } => f.write_str(if *on { "preview on" } else { "preview off" }),
            Command::Present { slide: None } => f.write_str("present"),
            Command::Present { slide: Some(s) } => write!(f, "present {s}"),
            Command::Next => f.write_str("next"),
            Command::Prev => f.write_str("prev"),
            Command::Stop => f.write_str("stop"),
            Command::View => f.write_str("view"),
            Command::Styles => f.write_str("styles"),
            Command::Style(StyleCommand::Add { name, options }) => {
                write!(f, "style add {}", quote_always(name))?;
                options.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::Style(StyleCommand::Set { name, options }) => {
                write!(f, "style set {}", quote_always(name))?;
                options.iter().try_for_each(|o| write!(f, " {o}"))
            }
            Command::Style(StyleCommand::Rename { from, to }) => {
                write!(
                    f,
                    "style rename {} {}",
                    quote_always(from),
                    quote_always(to)
                )
            }
            Command::Style(StyleCommand::Remove { name, into }) => {
                write!(f, "style remove {}", quote_always(name))?;
                match into {
                    None => Ok(()),
                    Some(Replacement::Custom) => f.write_str(" into=custom"),
                    Some(Replacement::Named(other)) => write!(f, " into={}", quote_always(other)),
                }
            }
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
            StyleOption::Named(name) => write!(f, "style={}", quote_always(name)),
            StyleOption::Custom => f.write_str("custom"),
        }
    }
}

impl fmt::Display for TextTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextTarget::Root(area) => write!(f, "{area}"),
            TextTarget::Span(span) => write!(f, "{span}"),
        }
    }
}

impl fmt::Display for SpanOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanOption::Named(name) => StyleOption::Named(name.clone()).fmt(f),
            SpanOption::Custom => f.write_str("custom"),
            SpanOption::Size(v) => StyleOption::Size(*v).fmt(f),
            SpanOption::Bold(v) => StyleOption::Bold(*v).fmt(f),
            SpanOption::Italic(v) => StyleOption::Italic(*v).fmt(f),
            SpanOption::Colour(v) => StyleOption::Colour(*v).fmt(f),
            SpanOption::Font(v) => StyleOption::Font(*v).fmt(f),
            SpanOption::Inherit(field) => write!(f, "{}=inherit", field.key()),
        }
    }
}

/// A text tree on one line: `[` its items separated by `, ` `]`, a span written `{`, its id if
/// it has one, its options, then its items, `}`, as `["a ", {t4 bold "b"}]`.
pub(crate) fn tree_text(items: &[TreeItem]) -> String {
    format!("[{}]", items_text(items))
}

fn items_text(items: &[TreeItem]) -> String {
    items
        .iter()
        .map(|item| match item {
            TreeItem::Chars(text) => quote_always(text),
            TreeItem::Span(span) => {
                let mut words: Vec<String> = span.id.iter().map(|id| id.to_string()).collect();
                words.extend(span.style.iter().map(|o| o.to_string()));
                if !span.items.is_empty() {
                    words.push(items_text(&span.items));
                }
                format!("{{{}}}", words.join(" "))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The characters that are words of their own outside a quoted string: the brackets, the braces
/// and the commas of a text tree.
const PUNCTUATION: [char; 5] = ['[', ']', '{', '}', ','];

/// A word of a line: a run of characters without spaces, a double-quoted string with `\"`, `\\`
/// and `\n` escapes, a formula between `$` signs, or one of the `PUNCTUATION` characters. A quoted
/// word is kept apart, so `"bold"` is a text and not a flag.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Word {
    Bare(String),
    Quoted(String),
    Punct(char),
    /// A formula's LaTeX, read raw between `$` signs: no escape is read, so `\frac` is written
    /// once; the formula ends at the first `$` preceded by an even number of `\`, so `\$` is a
    /// dollar of the formula and `\\$` a row break that ends it,
    /// `design@rootpoint-protocol@math-in-protocol-grammar`.
    Math(String),
}

impl Word {
    pub(crate) fn text(&self) -> &str {
        match self {
            Word::Bare(s) | Word::Quoted(s) | Word::Math(s) => s,
            Word::Punct('[') => "[",
            Word::Punct(']') => "]",
            Word::Punct('{') => "{",
            Word::Punct('}') => "}",
            Word::Punct(_) => ",",
        }
    }
}

pub(crate) fn split_words(line: &str) -> Result<Vec<Word>, ParseError> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if PUNCTUATION.contains(&c) {
            chars.next();
            words.push(Word::Punct(c));
        } else if c == '"' {
            chars.next();
            words.push(Word::Quoted(read_quoted(&mut chars)?));
        } else if c == '$' {
            chars.next();
            words.push(Word::Math(read_math(&mut chars)?));
        } else {
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || PUNCTUATION.contains(&c) {
                    break;
                }
                chars.next();
                // A quoted value after `key=`, as in `style="Style 1"`, is read with the escapes
                // of a quoted text, and stays part of the bare word, quoted again as the printer
                // writes it, so that `into="custom"`, a style's name, stays apart from
                // `into=custom`.
                if c == '"' && text.ends_with('=') {
                    text.push_str(&quote_always(&read_quoted(&mut chars)?));
                } else {
                    text.push(c);
                }
            }
            words.push(Word::Bare(text));
        }
    }
    Ok(words)
}

/// The rest of a formula, after its opening `$`, up to the first `$` preceded by an even number of
/// backslashes, which ends it and is not kept.
fn read_math(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, ParseError> {
    let mut text = String::new();
    let mut backslashes = 0usize;
    for c in chars.by_ref() {
        if c == '$' && backslashes.is_multiple_of(2) {
            return Ok(text);
        }
        backslashes = if c == '\\' { backslashes + 1 } else { 0 };
        text.push(c);
    }
    Err(ParseError("an unclosed `$`".to_owned()))
}

/// The rest of a quoted string, after its opening quote, with its escapes read.
fn read_quoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, ParseError> {
    let mut text = String::new();
    loop {
        match chars.next() {
            None => return Err(ParseError("an unclosed quote".to_owned())),
            Some('"') => return Ok(text),
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
}

/// A word as the printer writes it: bare when it can be read back bare, quoted otherwise. A word
/// that starts with `$` is quoted, since the lexer reads a bare one as a formula.
pub(crate) fn quote(text: &str) -> String {
    let bare = !text.is_empty()
        && !text.starts_with('$')
        && !text.contains(|c: char| {
            c.is_whitespace() || c == '"' || c == '\\' || PUNCTUATION.contains(&c)
        });
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
            Some(Word::Punct(c)) => Err(ParseError(format!("`{c}` instead of {what}"))),
            Some(Word::Math(_)) => Err(ParseError(format!("a formula instead of {what}"))),
            None => Err(ParseError(format!("missing {what}"))),
        }
    }

    /// The next word, left in place.
    pub(crate) fn peek(&self) -> Option<&Word> {
        self.words.get(self.next)
    }

    /// Take the next word when it is the punctuation `c`.
    fn punct(&mut self, c: char) -> bool {
        let found = self.peek() == Some(&Word::Punct(c));
        if found {
            self.next += 1;
        }
        found
    }

    /// The next word, bare or quoted, as a path. An empty path is refused: a host reads it as its
    /// working directory, where `unpack ""` would write over files.
    pub(crate) fn path(&mut self) -> Result<String, ParseError> {
        let word = self
            .words
            .get(self.next)
            .ok_or_else(|| ParseError("missing a path".to_owned()))?;
        if let Word::Punct(c) = word {
            return Err(ParseError(format!("`{c}` instead of a path")));
        }
        if let Word::Math(_) = word {
            return Err(ParseError("a formula instead of a path".to_owned()));
        }
        self.next += 1;
        if word.text().is_empty() {
            return Err(ParseError("an empty path".to_owned()));
        }
        Ok(word.text().to_owned())
    }

    /// The next word, bare or quoted, as a path, or `None` when the line has no more words; an
    /// empty path is refused, as `path` refuses it.
    pub(crate) fn optional_path(&mut self) -> Result<Option<String>, ParseError> {
        if self.next == self.words.len() {
            return Ok(None);
        }
        self.path().map(Some)
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

    /// The next word, which must be a formula between `$` signs, read by `Formula::parse`.
    pub(crate) fn formula(&mut self) -> Result<Formula, ParseError> {
        match self.words.get(self.next) {
            Some(Word::Math(text)) => {
                self.next += 1;
                Formula::parse(text).map_err(|e| ParseError(format!("the formula: {e}")))
            }
            _ => Err(ParseError("missing a formula between `$` signs".to_owned())),
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

pub(crate) fn parse_span(text: &str) -> Result<SpanId, ParseError> {
    parse_id(text, 't').map(SpanId)
}

/// `n<n>`, the root of an area's text, or `t<n>`, a span.
fn parse_target(text: &str) -> Result<TextTarget, ParseError> {
    if text.starts_with('t') {
        parse_span(text).map(TextTarget::Span)
    } else {
        parse_node(text).map(TextTarget::Root)
    }
}

/// `<a>..<b>`, a range of offsets.
fn parse_range(text: &str) -> Result<Range<usize>, ParseError> {
    let bad = || ParseError(format!("`{text}` is not a range `<a>..<b>`"));
    let (a, b) = text.split_once("..").ok_or_else(bad)?;
    Ok(parse_digits(a).ok_or_else(bad)?..parse_digits(b).ok_or_else(bad)?)
}

/// A text tree: `[` items separated by commas `]`.
fn parse_tree(args: &mut Args) -> Result<Vec<TreeItem>, ParseError> {
    if !args.punct('[') {
        return Err(ParseError("missing a text tree `[…]`".to_owned()));
    }
    parse_items(args, ']')
}

/// The items of a tree or of a span, separated by commas, up to and with the closing `end`.
fn parse_items(args: &mut Args, end: char) -> Result<Vec<TreeItem>, ParseError> {
    let mut items = Vec::new();
    if args.punct(end) {
        return Ok(items);
    }
    loop {
        items.push(parse_item(args)?);
        if args.punct(end) {
            return Ok(items);
        }
        if !args.punct(',') {
            return Err(ParseError(format!("missing `,` or `{end}` in a text tree")));
        }
    }
}

/// A quoted string, or a span `{[t<n>] <options> <items>}`.
fn parse_item(args: &mut Args) -> Result<TreeItem, ParseError> {
    if let Some(Word::Quoted(_)) = args.peek() {
        return Ok(TreeItem::Chars(args.quoted("a quoted text")?));
    }
    if !args.punct('{') {
        return Err(ParseError(
            "missing a quoted text or a span `{…}` in a text tree".to_owned(),
        ));
    }
    let id = match args.peek() {
        Some(Word::Bare(word)) if word.starts_with('t') => {
            let id = parse_span(word)?;
            args.next += 1;
            Some(id)
        }
        _ => None,
    };
    let mut style = Vec::new();
    while let Some(option) = args.optional(parse_span_option)? {
        style.push(option);
    }
    let items = parse_items(args, '}')?;
    Ok(TreeItem::Span(TreeSpan { id, style, items }))
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

/// The options of `style add` and `style set`: values, never `style=` or `custom`, which choose a
/// text's style and mean nothing for a style of the table.
fn value_options(args: &mut Args) -> Result<Vec<StyleOption>, ParseError> {
    let mut options = Vec::new();
    while let Some(option) = args.optional(parse_style_option)? {
        if matches!(option, StyleOption::Named(_) | StyleOption::Custom) {
            return Err(ParseError(format!(
                "`{option}` chooses a text's style, not a value of a style"
            )));
        }
        options.push(option);
    }
    Ok(options)
}

/// The options of a span's style, `wrap`'s and `span`'s.
fn span_options(args: &mut Args) -> Result<Vec<SpanOption>, ParseError> {
    let mut options = Vec::new();
    while let Some(option) = args.optional(parse_span_option)? {
        options.push(option);
    }
    Ok(options)
}

/// The options of `span-style add` and `span-style set`: values, never `style=` or `custom`, which
/// choose a span's style and mean nothing for a style of the table.
fn span_value_options(args: &mut Args) -> Result<Vec<SpanOption>, ParseError> {
    let options = span_options(args)?;
    if let Some(option) = options
        .iter()
        .find(|o| matches!(o, SpanOption::Named(_) | SpanOption::Custom))
    {
        return Err(ParseError(format!(
            "`{option}` chooses a span's style, not a value of a span style"
        )));
    }
    Ok(options)
}

/// A span's option: one of `text`'s but alignment, which is the area's alone, or `<key>=inherit`.
fn parse_span_option(text: &str) -> Result<SpanOption, ParseError> {
    let field = match text.split_once('=') {
        Some((key, "inherit")) => [
            SpanField::Font,
            SpanField::Size,
            SpanField::Bold,
            SpanField::Italic,
            SpanField::Colour,
        ]
        .into_iter()
        .find(|f| f.key() == key),
        _ => None,
    };
    if let Some(field) = field {
        return Ok(SpanOption::Inherit(field));
    }
    Ok(match parse_style_option(text)? {
        StyleOption::Named(name) => SpanOption::Named(name),
        StyleOption::Custom => SpanOption::Custom,
        StyleOption::Size(v) => SpanOption::Size(v),
        StyleOption::Bold(v) => SpanOption::Bold(v),
        StyleOption::Italic(v) => SpanOption::Italic(v),
        StyleOption::Colour(v) => SpanOption::Colour(v),
        StyleOption::Font(v) => SpanOption::Font(v),
        StyleOption::Align(_) | StyleOption::VAlign(_) => {
            return Err(ParseError(format!(
                "`{text}` is the area's alignment, which a span does not set"
            )));
        }
    })
}

/// `into=a<n>`: the asset that names a removed one's uses.
fn parse_asset_into(text: &str) -> Result<AssetId, ParseError> {
    match text.strip_prefix("into=") {
        Some(id) => parse_asset(id),
        None => Err(ParseError(format!("`{text}` is not `into=a<n>`"))),
    }
}

fn parse_filter(text: &str) -> Result<AssetFilter, ParseError> {
    match text {
        "all" => Ok(AssetFilter::All),
        "unused" => Ok(AssetFilter::Unused),
        "missing" => Ok(AssetFilter::Missing),
        "untracked" => Ok(AssetFilter::Untracked),
        other => Err(ParseError(format!(
            "`{other}` is not `all`, `unused`, `missing` or `untracked`"
        ))),
    }
}

/// `into=custom`, or `into="<name>"`, a name bare or quoted: what the texts naming a removed style
/// name instead. Only a bare `custom` asks for a copy, so a style named "custom" is
/// `into="custom"`.
fn parse_into(text: &str) -> Result<Replacement, ParseError> {
    match text.strip_prefix("into=") {
        Some("custom") => Ok(Replacement::Custom),
        Some(value) if !value.is_empty() => Ok(Replacement::Named(value_text(value)?)),
        _ => Err(ParseError(format!(
            "`{text}` is not `into=custom` or `into=\"<name>\"`"
        ))),
    }
}

/// The text of a value after `key=`: a bare word as it is, or a quoted one, as `split_words` keeps
/// it, with its escapes read.
fn value_text(value: &str) -> Result<String, ParseError> {
    let Some(rest) = value.strip_prefix('"') else {
        return Ok(value.to_owned());
    };
    let mut chars = rest.chars().peekable();
    let text = read_quoted(&mut chars)?;
    match chars.next() {
        None => Ok(text),
        Some(_) => Err(ParseError(format!("`{value}` has words after its quote"))),
    }
}

/// An option of `equation`.
#[derive(Clone, Debug, PartialEq)]
pub enum EquationOption {
    Numbered(bool),
    Size(EquationSize),
    Colour(Rgba),
}

impl fmt::Display for EquationOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EquationOption::Numbered(true) => f.write_str("numbered=yes"),
            EquationOption::Numbered(false) => f.write_str("numbered=no"),
            EquationOption::Size(EquationSize::Fit) => f.write_str("size=fit"),
            EquationOption::Size(EquationSize::AtMost(size)) => write!(f, "size={size}"),
            EquationOption::Colour(c) => write!(f, "{}", StyleOption::Colour(*c)),
        }
    }
}

fn parse_equation_option(text: &str) -> Result<EquationOption, ParseError> {
    let bad = || ParseError(format!("`{text}` is not an option of `equation`"));
    match text.split_once('=') {
        Some(("numbered", "yes")) => Ok(EquationOption::Numbered(true)),
        Some(("numbered", "no")) => Ok(EquationOption::Numbered(false)),
        Some(("size", "fit")) => Ok(EquationOption::Size(EquationSize::Fit)),
        Some(("size", v)) => parse_number(v)
            .ok()
            .filter(|n| *n > 0.0)
            .map(|n| EquationOption::Size(EquationSize::AtMost(n)))
            .ok_or_else(bad),
        Some(("colour", v)) => parse_colour(v).map(EquationOption::Colour).ok_or_else(bad),
        _ => Err(bad()),
    }
}

fn parse_style_option(text: &str) -> Result<StyleOption, ParseError> {
    let bad = || ParseError(format!("`{text}` is not a style option"));
    match text.split_once('=') {
        None => match text {
            "bold" => Ok(StyleOption::Bold(true)),
            "italic" => Ok(StyleOption::Italic(true)),
            "custom" => Ok(StyleOption::Custom),
            _ => Err(bad()),
        },
        Some(("style", name)) if !name.is_empty() => value_text(name).map(StyleOption::Named),
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
