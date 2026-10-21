//! The built-in sample: a presentation the crate holds, its RON text and its images included at
//! build time, so that `sample` opens it in every host, the browser's included, with no file.

use rootpoint_model::{AssetSource, Document, load_ron};

const TEXT: &str = include_str!("../sample/sample.ron");

/// The sample's images, by the path its RON text gives each.
const IMAGES: &[(&str, &[u8])] = &[("splits.png", include_bytes!("../sample/splits.png"))];

/// The sample, its images embedded, as no directory holds them.
pub(crate) fn sample() -> Result<Document, String> {
    let mut doc = load_ron(TEXT).map_err(|e| format!("the sample: {e:?}"))?;
    let mut ops = Vec::new();
    for (&id, asset) in &doc.assets {
        let AssetSource::Path(path) = &asset.source else {
            continue;
        };
        let (_, bytes) = IMAGES
            .iter()
            .find(|(name, _)| name == path)
            .ok_or(format!("the sample has no image {path}"))?;
        ops.push(rootpoint_model::Op::SetAssetSource {
            id,
            source: AssetSource::Embedded(bytes.to_vec()),
        });
    }
    for op in ops {
        rootpoint_model::apply(&mut doc, op).map_err(|e| format!("the sample: {e:?}"))?;
    }
    Ok(doc)
}
