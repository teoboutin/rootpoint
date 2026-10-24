//! The decoded images of a document's assets, which the caller builds from the assets' bytes: the
//! renderer reads no file, since the browser build has none to read.

use std::collections::BTreeMap;
use std::sync::Arc;

use image::ImageDecoder;
use rootpoint_model::AssetId;
use vello_cpu::PixelMetadata;
use vello_cpu::Pixmap;
use vello_cpu::peniko::ImageAlphaType;

/// Why an asset's bytes gave no image.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodeError(pub String);

/// The images the renderer can draw, by asset id.
#[derive(Clone, Default)]
pub struct ImageSet {
    images: BTreeMap<AssetId, Arc<Pixmap>>,
}

impl ImageSet {
    pub fn new() -> Self {
        ImageSet::default()
    }

    /// Decode the bytes of an image file, PNG or JPEG, as the image of the asset `id`.
    pub fn insert_bytes(&mut self, id: AssetId, bytes: &[u8]) -> Result<(), DecodeError> {
        let (width, height, rgba) = decode_rgba(bytes)?;
        let pixmap = Pixmap::from_parts(
            rgba,
            width,
            height,
            PixelMetadata {
                may_have_transparency: true,
                alpha_type: ImageAlphaType::Alpha,
            },
        );
        self.images.insert(id, Arc::new(pixmap));
        Ok(())
    }

    pub(crate) fn get(&self, id: AssetId) -> Option<&Arc<Pixmap>> {
        self.images.get(&id)
    }
}

/// Decode the bytes of an image file, PNG or JPEG, as its width, its height and its pixels, RGBA
/// with straight alpha, row by row: what the renderer draws, which a caller showing the image
/// elsewhere, as the editor's thumbnail, decodes the same way.
pub fn decode_rgba(bytes: &[u8]) -> Result<(u16, u16, Vec<u8>), DecodeError> {
    let error = |e: image::ImageError| DecodeError(e.to_string());
    let mut decoder = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| DecodeError(e.to_string()))?
        .into_decoder()
        .map_err(error)?;
    // A photograph's EXIF orientation is applied, so that it is drawn upright.
    let orientation = decoder.orientation().map_err(error)?;
    let mut decoded = image::DynamicImage::from_decoder(decoder).map_err(error)?;
    decoded.apply_orientation(orientation);
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    let too_big = |side: u32| DecodeError(format!("a side of {side} pixels exceeds 65535"));
    let width = u16::try_from(width).map_err(|_| too_big(width))?;
    let height = u16::try_from(height).map_err(|_| too_big(height))?;
    Ok((width, height, rgba.into_raw()))
}
