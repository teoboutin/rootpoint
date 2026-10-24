//! The built-in sample: a presentation the crate holds, its RON text and its images included at
//! build time, so that `sample` opens it in every host, the browser's included, with no file.

use std::collections::BTreeMap;

use rootpoint_model::{Document, load_ron};

const TEXT: &str = include_str!("../sample/sample.ron");

/// The sample's images, by the path its RON text gives each.
const IMAGES: &[(&str, &[u8])] = &[("splits.png", include_bytes!("../sample/splits.png"))];

/// The sample, and its images by asset path: the map of a store in memory, as no directory holds
/// them.
pub(crate) fn sample() -> Result<(Document, BTreeMap<String, Vec<u8>>), String> {
    let doc = load_ron(TEXT).map_err(|e| format!("the sample: {e:?}"))?;
    let mut images = BTreeMap::new();
    for asset in doc.assets.values() {
        let (_, bytes) = IMAGES
            .iter()
            .find(|(name, _)| *name == asset.path)
            .ok_or(format!("the sample has no image {}", asset.path))?;
        images.insert(asset.path.clone(), bytes.to_vec());
    }
    Ok((doc, images))
}
