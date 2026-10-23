//! The invariants every document keeps, checked on every loaded file and after every operation, and
//! the breaches a document may hold, as `design@rootpoint-model@validated-documents` and
//! `design@rootpoint-model@cosmetic-fallbacks` decide.
//!
//! A document comes from `apply` or from a file a person or an agent may have edited by hand, so
//! the invariants the types' doc comments state are checked, not assumed. A structural breach, one
//! that leaves the layout or the identity of the nodes undefined, is refused: no later operation
//! meets a broken tree. A cosmetic breach, a reference to a missing target that has a defined
//! stand-in inside one area's drawing, is accepted, and `Document::breaches` names it so that the
//! renderer's stand-in is reported.

use std::collections::BTreeSet;

use crate::document::*;

/// The first invariant a document breaks, described for a reader of the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Invalid(pub String);

/// A cosmetic breach of a document: a reference of one area to a target the document or the binary
/// lacks, which the renderer draws with a stand-in.
#[derive(Clone, Debug, PartialEq)]
pub enum Breach {
    /// An image, or a text's font, names an asset the table lacks. The image is drawn grey; the
    /// text in the built-in sans.
    MissingAsset { area: NodeId, asset: AssetId },
    /// A text's font names a built-in family the binary does not carry, which
    /// [`BUILTIN_FAMILIES`] lists; it is drawn in the built-in sans.
    UnknownFont { area: NodeId, name: String },
}

impl Document {
    /// Check every invariant the types state: the aspect ratio has no zero side; every id is
    /// unique and below the id counter; every split has two children or more, one divider fewer,
    /// finite, strictly increasing and strictly between 0 and 1, and no child split of its own
    /// axis; every asset an area names has an id below the counter. An asset the table lacks is
    /// a cosmetic breach, which [`Document::breaches`] names, and is not checked here.
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
            Node::Area(a) => self.validate_content(a),
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
        match asset_named(area) {
            Some(id) if id.0 >= self.next_id => Err(Invalid(format!(
                "area {} names asset {}, which is not below the id counter, next_id {}",
                area.id.0, id.0, self.next_id
            ))),
            _ => Ok(()),
        }
    }

    /// Every cosmetic breach of the document, area by area in the order of the slides and of
    /// their trees. A document the editor's commands built has none; a file edited by hand, or
    /// an undo back to such a file's content, can.
    pub fn breaches(&self) -> Vec<Breach> {
        let mut out = Vec::new();
        for area in self.slides.iter().flat_map(|s| s.root.areas()) {
            if let Some(asset) = asset_named(area)
                && !self.assets.contains_key(&asset)
            {
                out.push(Breach::MissingAsset {
                    area: area.id,
                    asset,
                });
            }
            if let Content::Text(TextContent {
                style:
                    TextStyle {
                        font: FontRef::Builtin(name),
                        ..
                    },
                ..
            }) = &area.content
                && !BUILTIN_FAMILIES.contains(&name.as_str())
            {
                out.push(Breach::UnknownFont {
                    area: area.id,
                    name: name.clone(),
                });
            }
        }
        out
    }
}

/// The asset an area names: its image, or its text's font file.
fn asset_named(area: &Area) -> Option<AssetId> {
    match &area.content {
        Content::Image(image) => Some(image.asset),
        Content::Text(TextContent {
            style:
                TextStyle {
                    font: FontRef::Asset(id),
                    ..
                },
            ..
        }) => Some(*id),
        Content::Text(_) => None,
    }
}
