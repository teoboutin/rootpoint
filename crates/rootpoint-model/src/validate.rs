//! The invariants every document keeps, checked on every loaded file and after every operation, and
//! the breaches a document may hold, as `design@rootpoint-model@validated-documents` and
//! `design@rootpoint-model@cosmetic-fallbacks` decide.
//!
//! A document comes from `apply` or from a file a person or an agent may have edited by hand, so
//! the invariants the types' doc comments state are checked, not assumed. A structural breach, one
//! that leaves the layout or the identity of the nodes undefined, is refused: no later operation
//! meets a broken tree. A cosmetic breach, a reference to a missing target that has a defined
//! stand-in inside one area's drawing, or an asset whose path fails `asset_path_is_confined`, which
//! no store reads and which is drawn as a missing one, is accepted, and `Document::breaches` names it so that
//! the renderer's stand-in is reported.

use std::collections::BTreeSet;

use crate::document::*;

/// The first invariant a document breaks, described for a reader of the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Invalid(pub String);

/// A cosmetic breach of a document: a reference of one area to a target the document or the binary
/// lacks, or an asset of the table whose path fails `asset_path_is_confined`; the renderer draws
/// each with a stand-in.
#[derive(Clone, Debug, PartialEq)]
pub enum Breach {
    /// An image, or a text's font, names an asset the table lacks. The image is drawn grey; the
    /// text in the built-in sans.
    MissingAsset { area: NodeId, asset: AssetId },
    /// A text's font names a built-in family the binary does not carry, which
    /// [`BUILTIN_FAMILIES`] lists; it is drawn in the built-in sans.
    UnknownFont { area: NodeId, name: String },
    /// A text names a style the table lacks; it is drawn with `TextStyle::default()`.
    MissingStyle { area: NodeId, name: String },
    /// A span of a text names a span style the table lacks; it is drawn with its parent's values.
    MissingSpanStyle {
        area: NodeId,
        span: SpanId,
        name: String,
    },
    /// An asset's path fails `asset_path_is_confined`: no store reads or writes it, so an image
    /// that shows it is drawn grey, and a text in its font in the built-in sans.
    UnconfinedAsset { asset: AssetId },
}

impl Document {
    /// Check every invariant the types state: the aspect ratio has no zero side; every id is
    /// unique and below the id counter; every split has two children or more, one divider fewer,
    /// finite, strictly increasing and strictly between 0 and 1, and no child split of its own
    /// axis; every asset an area, a span or a style of a table names has an id below the counter. An
    /// asset the table lacks is a cosmetic breach, which [`Document::breaches`] names, and is not
    /// checked here.
    pub fn validate(&self) -> Result<(), Invalid> {
        if self.aspect.width == 0 || self.aspect.height == 0 {
            return Err(Invalid(format!(
                "the aspect ratio {}:{} has a zero side",
                self.aspect.width, self.aspect.height
            )));
        }
        let mut ids = BTreeSet::new();
        let mut take = |id: u64| {
            if id >= self.next_id {
                Err(Invalid(format!(
                    "id {id} is not below the id counter, next_id {}",
                    self.next_id
                )))
            } else if !ids.insert(id) {
                Err(Invalid(format!("id {id} is used twice")))
            } else {
                Ok(())
            }
        };
        for id in self.assets.keys() {
            take(id.0)?;
        }
        let span_fonts = self
            .span_styles
            .iter()
            .filter_map(|(name, s)| Some((name, s.font.as_ref()?)));
        for (name, font) in self
            .styles
            .iter()
            .map(|(n, s)| (n, &s.font))
            .chain(span_fonts)
        {
            if let FontRef::Asset(id) = font
                && id.0 >= self.next_id
            {
                return Err(Invalid(format!(
                    "style {name:?} names asset {}, which is not below the id counter, next_id {}",
                    id.0, self.next_id
                )));
            }
        }
        for slide in &self.slides {
            take(slide.id.0)?;
            self.validate_node(&slide.root, &mut take)?;
        }
        Ok(())
    }

    fn validate_node(
        &self,
        node: &Node,
        take: &mut impl FnMut(u64) -> Result<(), Invalid>,
    ) -> Result<(), Invalid> {
        take(node.id().0)?;
        match node {
            Node::Area(a) => {
                if let Content::Text(text) = &a.content {
                    for span in text.root.spans() {
                        take(span.id.0)?;
                    }
                }
                self.validate_content(a)
            }
            Node::Split(s) => {
                let at = |what: &str| Invalid(format!("split {}: {what}", s.id.0));
                if s.children.len() < 2 {
                    return Err(at("fewer than two children"));
                }
                if s.dividers.len() + 1 != s.children.len() {
                    return Err(at(&format!(
                        "{} dividers for {} children",
                        s.dividers.len(),
                        s.children.len()
                    )));
                }
                let mut last = 0.0;
                for &d in &s.dividers {
                    // Also refuses NaN, for which every comparison is false.
                    if !(d > last && d < 1.0) {
                        return Err(at(&format!(
                            "dividers {:?} are not strictly increasing in (0, 1)",
                            s.dividers
                        )));
                    }
                    last = d;
                }
                for child in &s.children {
                    if let Node::Split(c) = child
                        && c.axis == s.axis
                    {
                        return Err(at(&format!("child split {} has its own axis", c.id.0)));
                    }
                    self.validate_node(child, take)?;
                }
                Ok(())
            }
        }
    }

    /// An area's reference to an asset at or above the id counter is structural: a later
    /// `Op::AddAsset` would give that id to a new asset, which the area would then show,
    /// `design@rootpoint-model@validated-documents`.
    fn validate_content(&self, area: &Area) -> Result<(), Invalid> {
        match assets_named(area)
            .into_iter()
            .find(|id| id.0 >= self.next_id)
        {
            Some(id) => Err(Invalid(format!(
                "area {} names asset {}, which is not below the id counter, next_id {}",
                area.id.0, id.0, self.next_id
            ))),
            None => Ok(()),
        }
    }

    /// Every cosmetic breach of the document, area by area in the order of the slides and of
    /// their trees, then each asset of the table whose path fails `asset_path_is_confined`, used
    /// or not, in the order of the ids. A document the editor's commands built has none; a file
    /// edited by hand, or an undo back to such a file's content, can.
    pub fn breaches(&self) -> Vec<Breach> {
        let mut out = Vec::new();
        for area in self.slides.iter().flat_map(|s| s.root.areas()) {
            let font = match &area.content {
                Content::Image(image) => {
                    if !self.assets.contains_key(&image.asset) {
                        out.push(Breach::MissingAsset {
                            area: area.id,
                            asset: image.asset,
                        });
                    }
                    continue;
                }
                Content::Text(TextContent {
                    style: StyleChoice::Named(name),
                    ..
                }) if !self.styles.contains_key(name) => {
                    out.push(Breach::MissingStyle {
                        area: area.id,
                        name: name.clone(),
                    });
                    // The spans are drawn over the stand-in, and named as the renderer names them.
                    if let Content::Text(text) = &area.content {
                        self.span_breaches(area.id, &text.root, &mut out);
                    }
                    continue;
                }
                Content::Text(text) => {
                    self.span_breaches(area.id, &text.root, &mut out);
                    self.resolve_style(&text.style).font
                }
            };
            match font {
                FontRef::Asset(asset) if !self.assets.contains_key(&asset) => {
                    out.push(Breach::MissingAsset {
                        area: area.id,
                        asset,
                    })
                }
                FontRef::Builtin(name) if !BUILTIN_FAMILIES.contains(&name.as_str()) => {
                    out.push(Breach::UnknownFont {
                        area: area.id,
                        name,
                    })
                }
                _ => {}
            }
        }
        for (&asset, entry) in &self.assets {
            if !asset_path_is_confined(&entry.path) {
                out.push(Breach::UnconfinedAsset { asset });
            }
        }
        out
    }

    /// The breaches of a text's spans: a span style the table lacks, and a font the document or
    /// the binary lacks, in the order of the text.
    fn span_breaches(&self, area: NodeId, root: &TextNode, out: &mut Vec<Breach>) {
        for span in root.spans() {
            let font = match &span.style {
                SpanChoice::Named(name) => match self.span_styles.get(name) {
                    Some(style) => style.font.clone(),
                    None => {
                        out.push(Breach::MissingSpanStyle {
                            area,
                            span: span.id,
                            name: name.clone(),
                        });
                        continue;
                    }
                },
                SpanChoice::Custom(style) => style.font.clone(),
            };
            match font {
                Some(FontRef::Asset(asset)) if !self.assets.contains_key(&asset) => {
                    out.push(Breach::MissingAsset { area, asset })
                }
                Some(FontRef::Builtin(name)) if !BUILTIN_FAMILIES.contains(&name.as_str()) => {
                    out.push(Breach::UnknownFont { area, name })
                }
                _ => {}
            }
        }
    }
}

/// The assets an area names itself: its image, or the font files of its own text values and of its
/// spans' own values. A named style's font is its table's, which `Document::validate` checks once.
fn assets_named(area: &Area) -> Vec<AssetId> {
    match &area.content {
        Content::Image(image) => vec![image.asset],
        Content::Text(text) => {
            let own = match &text.style {
                StyleChoice::Custom(TextStyle {
                    font: FontRef::Asset(id),
                    ..
                }) => Some(*id),
                _ => None,
            };
            let spans = text
                .root
                .spans()
                .into_iter()
                .filter_map(|span| match &span.style {
                    SpanChoice::Custom(SpanStyle {
                        font: Some(FontRef::Asset(id)),
                        ..
                    }) => Some(*id),
                    _ => None,
                });
            own.into_iter().chain(spans).collect()
        }
    }
}
