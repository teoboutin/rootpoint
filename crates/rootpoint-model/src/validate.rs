//! The invariants every document keeps, checked on every loaded file and after every operation.
//!
//! A document comes from `apply` or from a file a person or an agent may have edited by hand, so
//! the invariants the types' doc comments state are checked, not assumed: a file or an operation
//! that would break one is refused, and no later operation meets a broken tree.

use std::collections::BTreeSet;

use crate::document::*;

/// The first invariant a document breaks, described for a reader of the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Invalid(pub String);

impl Document {
    /// Check every invariant the types state: the aspect ratio has no zero side; every id is
    /// unique and below the id counter; every split has two children or more, one divider fewer,
    /// finite, strictly increasing and strictly between 0 and 1, and no child split of its own
    /// axis; every asset an area names exists.
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

    fn validate_content(&self, area: &Area) -> Result<(), Invalid> {
        let named = match &area.content {
            Content::Image(image) => Some(image.asset),
            Content::Text(TextContent {
                style:
                    TextStyle {
                        font: FontRef::Asset(id),
                        ..
                    },
                ..
            }) => Some(*id),
            _ => None,
        };
        match named {
            Some(id) if !self.assets.contains_key(&id) => Err(Invalid(format!(
                "area {} names asset {}, which does not exist",
                area.id.0, id.0
            ))),
            _ => Ok(()),
        }
    }
}
