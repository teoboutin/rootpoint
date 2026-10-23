//! The decoded images and fonts of the document's assets, which the app hands `render_slide`, read
//! from the bytes `Session::asset_bytes` gives.

use std::collections::BTreeMap;
use std::path::PathBuf;

use rootpoint_model::{AssetId, AssetSource, Content, FontRef};
use rootpoint_protocol::{Files, Session};
use rootpoint_render::{FontSet, ImageSet};

/// How the document uses an asset, and where its bytes are.
#[derive(Clone, PartialEq)]
struct Use {
    image: bool,
    font: bool,
    source: AssetSource,
}

/// The sets of decoded assets, and what they were decoded from.
#[derive(Default)]
pub(crate) struct Assets {
    decoded: Option<(PathBuf, BTreeMap<AssetId, Use>)>,
    fonts: FontSet,
    images: ImageSet,
}

impl Assets {
    /// Decode the assets again where the document's assets, their use or the directory their
    /// paths are read against changed since the last decoding. A `FontSet` cannot drop a font,
    /// so both sets are built anew.
    pub(crate) fn refresh(&mut self, session: &Session, files: &mut dyn Files) {
        let doc = session.document();
        let mut uses: BTreeMap<AssetId, Use> = BTreeMap::new();
        for slide in &doc.slides {
            for area in slide.root.areas() {
                let (id, image) = match &area.content {
                    Content::Image(i) => (i.asset, true),
                    Content::Text(t) => match t.style.font {
                        FontRef::Asset(id) => (id, false),
                        FontRef::Builtin(_) => continue,
                    },
                };
                let Some(asset) = doc.assets.get(&id) else {
                    continue;
                };
                let entry = uses.entry(id).or_insert(Use {
                    image: false,
                    font: false,
                    source: asset.source.clone(),
                });
                if image {
                    entry.image = true;
                } else {
                    entry.font = true;
                }
            }
        }
        let key = (session.directory().to_path_buf(), uses);
        if self.decoded.as_ref() == Some(&key) {
            return;
        }
        self.fonts = FontSet::new();
        self.images = ImageSet::new();
        for (id, used) in &key.1 {
            // An asset that cannot be read or decoded is left out: the render draws it grey, or
            // in the built-in sans, and its report names it.
            let Ok(bytes) = session.asset_bytes(*id, files) else {
                continue;
            };
            if used.image {
                let _ = self.images.insert_bytes(*id, &bytes);
            }
            if used.font {
                let _ = self.fonts.insert_bytes(*id, bytes);
            }
        }
        self.decoded = Some(key);
    }

    pub(crate) fn sets(&mut self) -> (&mut FontSet, &ImageSet) {
        (&mut self.fonts, &self.images)
    }
}
