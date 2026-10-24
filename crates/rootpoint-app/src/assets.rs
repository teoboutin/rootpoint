//! The decoded images and fonts of the document's assets, which the app hands `render_slide`, read
//! from the bytes `Session::asset_bytes` gives.

use std::collections::BTreeMap;

use rootpoint_model::{AssetId, Content, FontRef};
use rootpoint_protocol::{Files, Session};
use rootpoint_render::{FontSet, ImageSet};

/// How the document uses an asset, and the path of its bytes in the store.
#[derive(Clone, PartialEq)]
struct Use {
    image: bool,
    font: bool,
    path: String,
}

/// The sets of decoded assets, and what they were decoded from.
#[derive(Default)]
pub(crate) struct Assets {
    decoded: Option<(Option<String>, BTreeMap<AssetId, Use>)>,
    fonts: FontSet,
    images: ImageSet,
    /// How many times the sets were built, for the tests.
    decodings: u64,
}

impl Assets {
    /// Decode the assets again where the document's assets, their use, their paths or the root
    /// their paths are read against changed since the last decoding. A store in memory has no root,
    /// so the caller drops the decoded assets when a command replaces it. A `FontSet` cannot drop a
    /// font, so both sets are built anew.
    pub(crate) fn refresh(&mut self, session: &Session, files: &mut dyn Files) {
        let doc = session.document();
        let mut uses: BTreeMap<AssetId, Use> = BTreeMap::new();
        for slide in &doc.slides {
            for area in slide.root.areas() {
                let (id, image) = match &area.content {
                    Content::Image(i) => (i.asset, true),
                    Content::Text(t) => match doc.resolve_style(&t.style).font {
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
                    path: asset.path.clone(),
                });
                if image {
                    entry.image = true;
                } else {
                    entry.font = true;
                }
            }
        }
        let key = (session.root().map(str::to_owned), uses);
        if self.decoded.as_ref() == Some(&key) {
            return;
        }
        self.fonts = FontSet::new();
        self.images = ImageSet::new();
        self.decodings += 1;
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

    /// Drop what was decoded, so that the next refresh decodes every asset again: the files may
    /// hold other bytes at the same paths.
    pub(crate) fn invalidate(&mut self) {
        self.decoded = None;
    }

    pub(crate) fn decodings(&self) -> u64 {
        self.decodings
    }

    pub(crate) fn sets(&mut self) -> (&mut FontSet, &ImageSet) {
        (&mut self.fonts, &self.images)
    }
}
