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

// The printed form of an id, `s<n>`, `n<n>` or `a<n>`, as the protocol shows it; the file writes
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
        Content::Text(TextContent {
            text: String::new(),
            style: StyleChoice::Custom(TextStyle::default()),
        })
    }
}

/// Wrapped text, with one style for the whole area.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub style: StyleChoice,
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
/// not `.rootpoint` and a path that is not `presentation.ron` or `mimetype`, the files of a
/// workspace and of a pack that are no asset. A file edited by hand can
/// name another path: it loads, and the store never reads or writes it, a cosmetic breach,
/// `design@rootpoint-model@cosmetic-fallbacks`.
pub fn asset_path_is_confined(path: &str) -> bool {
    !path.contains('\\')
        && path != "presentation.ron"
        && path != "mimetype"
        && path.split('/').next() != Some(".rootpoint")
        && path
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..")
}
