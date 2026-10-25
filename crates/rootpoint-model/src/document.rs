//! The types of a presentation: slides, their split trees, the content of their areas, and the
//! asset table.

use std::collections::BTreeMap;

use std::fmt;

use serde::{Deserialize, Serialize};

/// The id of a slide, unique in its document and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlideId(pub u64);

/// The id of a split or an area, unique in its document and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

/// The id of an asset, unique in its document and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AssetId(pub u64);

/// The id of a span of a text, unique in its document and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpanId(pub u64);

// The printed form of an id, `s<n>`, `n<n>`, `a<n>` or `t<n>`, as the protocol shows it; the file writes
// the plain number.
impl fmt::Display for SlideId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s{}", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}", self.0)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a{}", self.0)
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// A presentation: its aspect ratio, its slides in order, its assets, and its named text styles.
///
/// Every id of the document comes from one counter, which only grows, so a removed slide, node or
/// asset never lends its id to another (`design@rootpoint-model@stable-ids`). Change a document only
/// through [`crate::apply`], which keeps the invariants [`Document::validate`] checks, per
/// `design@rootpoint-model@edit-operation-core`; a loaded file is checked by it too.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub aspect: AspectRatio,
    pub slides: Vec<Slide>,
    pub assets: BTreeMap<AssetId, Asset>,
    /// The named text styles, by name. A text names one, or holds its own values.
    pub styles: BTreeMap<String, TextStyle>,
    /// The named span styles, by name. A span names one, or holds its own values.
    pub span_styles: BTreeMap<String, SpanStyle>,
    pub(crate) next_id: u64,
}

/// A new document holds the predefined styles, Title, Subtitle, Body and Legend.
impl Default for Document {
    fn default() -> Self {
        Document {
            aspect: AspectRatio::default(),
            slides: Vec::new(),
            assets: BTreeMap::new(),
            styles: predefined_styles(),
            span_styles: BTreeMap::new(),
            next_id: 1,
        }
    }
}

/// The name of the predefined style a new area takes when it copies none, as
/// `Document::new_area_style` says.
pub const BODY: &str = "Body";

/// The styles a new document holds: Title, Subtitle, Body and Legend, each a change of
/// `TextStyle::default()`.
pub fn predefined_styles() -> BTreeMap<String, TextStyle> {
    let base = TextStyle::default;
    BTreeMap::from([
        (
            "Title".to_owned(),
            TextStyle {
                size: 120.0,
                bold: true,
                h_align: HAlign::Center,
                v_align: VAlign::Middle,
                ..base()
            },
        ),
        (
            "Subtitle".to_owned(),
            TextStyle {
                size: 64.0,
                h_align: HAlign::Center,
                ..base()
            },
        ),
        (
            BODY.to_owned(),
            TextStyle {
                size: 40.0,
                h_align: HAlign::Left,
                v_align: VAlign::Top,
                ..base()
            },
        ),
        (
            "Legend".to_owned(),
            TextStyle {
                size: 28.0,
                italic: true,
                h_align: HAlign::Center,
                ..base()
            },
        ),
    ])
}

impl Document {
    /// Whether two documents are equal in everything but their id counter. An operation followed
    /// by its inverse gives a document equal to the original in this sense: the counter has grown,
    /// so that no id is reused.
    pub fn same_content(&self, other: &Document) -> bool {
        self.aspect == other.aspect
            && self.slides == other.slides
            && self.assets == other.assets
            && self.styles == other.styles
            && self.span_styles == other.span_styles
    }

    /// The values a text's style draws with: the table's entry for a name, the text's own values,
    /// or `TextStyle::default()` for a name the table lacks, a cosmetic breach.
    pub fn resolve_style(&self, choice: &StyleChoice) -> TextStyle {
        match choice {
            StyleChoice::Named(name) => self.styles.get(name).cloned().unwrap_or_default(),
            StyleChoice::Custom(style) => style.clone(),
        }
    }

    /// The style of a new area that copies no other: Body while the table holds it and it draws
    /// with no stand-in, the default values otherwise, so that no new area holds a breach,
    /// `design@rootpoint-model@new-area-style`.
    pub fn new_area_style(&self) -> StyleChoice {
        match self.styles.get(BODY) {
            Some(body) if self.draws_as_asked(body) => StyleChoice::Named(BODY.to_owned()),
            _ => StyleChoice::Custom(TextStyle::default()),
        }
    }

    /// Whether a style's font is one the document or the binary has: an asset of the table, or a
    /// built-in family of `BUILTIN_FAMILIES`. A text drawn in one that is not stands in the built-in
    /// sans, a cosmetic breach.
    pub fn draws_as_asked(&self, style: &TextStyle) -> bool {
        match &style.font {
            FontRef::Asset(id) => self.assets.contains_key(id),
            FontRef::Builtin(name) => BUILTIN_FAMILIES.contains(&name.as_str()),
        }
    }

    pub(crate) fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// The slide with this id, if any.
    pub fn slide(&self, id: SlideId) -> Option<&Slide> {
        self.slides.iter().find(|s| s.id == id)
    }

    /// The node with this id, in any slide, if any.
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.slides.iter().find_map(|s| s.root.find(id))
    }
}

/// The shape of every slide of a document, 16:9 by default. A slide is 1080 slide units high and
/// `1080 * width / height` wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}

impl Default for AspectRatio {
    fn default() -> Self {
        AspectRatio {
            width: 16,
            height: 9,
        }
    }
}

/// One slide: its id and the root of its split tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slide {
    pub id: SlideId,
    pub root: Node,
}

/// A node of a slide's tree: a split of its extent, or an area holding one content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Split(Split),
    Area(Area),
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Node::Split(s) => s.id,
            Node::Area(a) => a.id,
        }
    }

    /// The node with this id in this subtree, if any.
    pub fn find(&self, id: NodeId) -> Option<&Node> {
        if self.id() == id {
            return Some(self);
        }
        match self {
            Node::Split(s) => s.children.iter().find_map(|c| c.find(id)),
            Node::Area(_) => None,
        }
    }

    /// Every area of this subtree, in tree order.
    pub fn areas(&self) -> Vec<&Area> {
        let mut out = Vec::new();
        self.collect_areas(&mut out);
        out
    }

    fn collect_areas<'a>(&'a self, out: &mut Vec<&'a Area>) {
        match self {
            Node::Split(s) => s.children.iter().for_each(|c| c.collect_areas(out)),
            Node::Area(a) => out.push(a),
        }
    }
}

/// Where a split puts its children. A `LeftToRight` split lays them side by side, left to right,
/// with vertical dividers between them; a `TopToBottom` split stacks them, top to bottom, with
/// horizontal dividers. The names say where the children go, which a reader of the file needs:
/// `Vertical` and `Horizontal` were read both ways (`design@rootpoint-model@axis-names`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    LeftToRight,
    TopToBottom,
}

/// A split of its extent along one axis into k ≥ 2 children.
///
/// `dividers` holds the k−1 positions between consecutive children, as fractions of the split's
/// extent, strictly increasing and strictly between 0 and 1. No child is a split of the same axis:
/// such a child is merged into its parent, so each layout has one tree
/// (`design@rootpoint-model@split-tree-arity`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub id: NodeId,
    pub axis: Axis,
    pub children: Vec<Node>,
    pub dividers: Vec<f64>,
}

impl Split {
    /// The extent of child `i`, as fractions of the split's extent.
    pub fn child_extent(&self, i: usize) -> (f64, f64) {
        let start = if i == 0 { 0.0 } else { self.dividers[i - 1] };
        let end = if i + 1 == self.children.len() {
            1.0
        } else {
            self.dividers[i]
        };
        (start, end)
    }
}

/// A leaf of a slide's tree, holding one content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub id: NodeId,
    pub content: Content,
}

/// The content of an area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    Text(TextContent),
    Image(ImageContent),
}

/// An empty text, in the default values. It reads no style table: the operations give a new area
/// the style `Document::new_area_style` or the area it was split from gives.
impl Default for Content {
    fn default() -> Self {
        Content::Text(TextContent::plain(
            "",
            StyleChoice::Custom(TextStyle::default()),
        ))
    }
}

/// Wrapped text: a tree of characters and spans, the root drawn in the area's style and each span
/// in its own values over its parent's.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub root: TextNode,
    pub style: StyleChoice,
}

impl TextContent {
    /// A text of one run of characters, with no span.
    pub fn plain(text: impl Into<String>, style: StyleChoice) -> Self {
        TextContent {
            root: TextNode::plain(text),
            style,
        }
    }

    /// The text's characters, the spans' included, in order.
    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        self.root.push_plain(&mut out);
        out
    }
}

/// A sequence of characters and spans: the root of a text, or a span's content. Kept in one
/// canonical form, no run of characters empty and no two adjacent, which [`TextNode::normalise`]
/// restores, so that two texts that read the same compare equal.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextNode {
    pub items: Vec<TextItem>,
}

impl TextNode {
    /// One run of characters, or no item for an empty text.
    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        TextNode {
            items: if text.is_empty() {
                Vec::new()
            } else {
                vec![TextItem::Chars(text)]
            },
        }
    }

    /// Merge adjacent runs of characters and drop empty ones, in this node and every span below.
    pub fn normalise(&mut self) {
        let mut items: Vec<TextItem> = Vec::with_capacity(self.items.len());
        for mut item in std::mem::take(&mut self.items) {
            match &mut item {
                TextItem::Chars(s) if s.is_empty() => continue,
                TextItem::Chars(s) => {
                    if let Some(TextItem::Chars(last)) = items.last_mut() {
                        last.push_str(s);
                        continue;
                    }
                }
                TextItem::Span(span) => span.text.normalise(),
            }
            items.push(item);
        }
        self.items = items;
    }

    fn push_plain(&self, out: &mut String) {
        for item in &self.items {
            match item {
                TextItem::Chars(s) => out.push_str(s),
                TextItem::Span(span) => span.text.push_plain(out),
            }
        }
    }

    /// Every span of this node and below, in the order of the text.
    pub fn spans(&self) -> Vec<&Span> {
        let mut out = Vec::new();
        self.collect_spans(&mut out);
        out
    }

    fn collect_spans<'a>(&'a self, out: &mut Vec<&'a Span>) {
        for item in &self.items {
            if let TextItem::Span(span) = item {
                out.push(span);
                span.text.collect_spans(out);
            }
        }
    }
}

/// An item of a text node. Room is left for other kinds, as maths, beside the span.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TextItem {
    Chars(String),
    Span(Span),
}

/// A span of a text: its id, its style, and its own sequence of characters and spans.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub id: SpanId,
    pub style: SpanChoice,
    pub text: TextNode,
}

/// A span's style: a span style of the document's table, by its name, or the span's own values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpanChoice {
    Named(String),
    Custom(SpanStyle),
}

/// The values a span sets over its parent's, each optional; a field left unset is the parent's.
/// Sizes are in slide units. Alignment is the area's alone.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpanStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<FontRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<Rgba>,
}

/// A text node of an area's text: its root, or one of its spans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextNodeRef {
    Root,
    Span(SpanId),
}

/// A caret position in an area's text: a text node and an offset in its sequence, counted in
/// characters, a span counting as one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CaretPos {
    pub node: TextNodeRef,
    pub offset: usize,
}

/// A text's style: a style of the document's table, by its name, or the text's own values. Editing
/// a named style changes every text that names it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StyleChoice {
    Named(String),
    Custom(TextStyle),
}

/// The style of a text area. Sizes are in slide units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font: FontRef,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub colour: Rgba,
    pub h_align: HAlign,
    pub v_align: VAlign,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: FontRef::Builtin("sans".to_owned()),
            size: 48.0,
            bold: false,
            italic: false,
            colour: Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            h_align: HAlign::Left,
            v_align: VAlign::Top,
        }
    }
}

/// The built-in families a `FontRef::Builtin` may name, the ones the binary carries. The renderer
/// draws each, and a text naming another is drawn in the built-in sans, a cosmetic breach.
pub const BUILTIN_FAMILIES: &[&str] = &["sans"];

/// A font: one the binary carries, by name, or a font file of the asset table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FontRef {
    Builtin(String),
    Asset(AssetId),
}

/// A colour, 8 bits per channel, alpha last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

/// An image area: the asset it shows and how the image is scaled into the area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageContent {
    pub asset: AssetId,
    pub fit: ImageFit,
}

/// How an image is scaled into its area. `Fit` shows the whole image, scaled to fit inside the area
/// and centred; `Fill` covers the whole area, scaled to cover it, centred and cropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFit {
    Fit,
    Fill,
}

/// An entry of the asset table: heavy data that areas refer to by id. Its bytes are a file at its
/// path, relative to the root of the presentation's store, which only the caller knows
/// (`design@rootpoint-model@asset-paths-only`); the document holds no bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub path: String,
}

/// Whether an asset path stays inside its root, `design@rootpoint-model@asset-path-confinement`:
/// `/`-separated components, none of them empty, `.` or `..`, no `\`, and a first component that is
/// not `.rootpoint`, `presentation.ron` or `mimetype`, the files of a workspace and of a pack that
/// are no asset, which no asset is, nor holds as its folder. A file edited by hand can
/// name another path: it loads, and the store never reads or writes it, a cosmetic breach,
/// `design@rootpoint-model@cosmetic-fallbacks`.
pub fn asset_path_is_confined(path: &str) -> bool {
    !path.contains('\\')
        && !matches!(
            path.split('/').next(),
            Some(".rootpoint" | "presentation.ron" | "mimetype")
        )
        && path
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..")
}
