//! A math font: a font file's bytes, read with ttf-parser for its cmap, its glyphs' advances and
//! bounds, its MATH table and the `ssty` lookups of its GSUB.

use std::fmt;
use std::sync::Arc;

use ttf_parser::gsub::{SingleSubstitution, SubstitutionSubtable};
use ttf_parser::math::{Constants, GlyphConstruction, GlyphInfo, Variants};
use ttf_parser::{Face, GlyphId, Tag};

/// Why a font file's bytes are not a math font.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontError(pub String);

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FontError {}

/// The bytes of a font file with a MATH table, checked once when it is made.
#[derive(Clone)]
pub struct MathFont {
    bytes: Arc<[u8]>,
}

impl fmt::Debug for MathFont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MathFont")
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// How a font makes a glyph taller: its larger variants, and whether it builds one of any height
/// from parts, with how many.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerticalConstruction {
    pub variants: usize,
    pub assembly_parts: Option<usize>,
}

impl MathFont {
    /// A math font from a font file's bytes: refused unless the face parses and has a MATH table
    /// with its constants, its glyph info and its variants.
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Result<MathFont, FontError> {
        let font = MathFont {
            bytes: bytes.into(),
        };
        font.reader()?;
        Ok(font)
    }

    /// The font file's bytes, which a renderer draws the glyphs of a layout from.
    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }

    /// The glyph of a character in the font's cmap.
    pub fn glyph(&self, ch: char) -> Option<u16> {
        let reader = self.reader().expect("checked when the font was made");
        reader.face.glyph_index(ch).map(|g| g.0)
    }

    /// The glyph the font's `ssty` substitutes for `glyph` in a script, or `glyph` itself.
    pub fn script_glyph(&self, glyph: u16) -> u16 {
        let reader = self.reader().expect("checked when the font was made");
        reader.script_glyph(GlyphId(glyph)).0
    }

    /// The scale of a script and of a script of a script, in per cent of the size.
    pub fn script_percent_scale_downs(&self) -> (i16, i16) {
        let reader = self.reader().expect("checked when the font was made");
        (
            reader.constants.script_percent_scale_down(),
            reader.constants.script_script_percent_scale_down(),
        )
    }

    /// The vertical construction of a character's glyph, if the font has one.
    pub fn vertical_construction(&self, ch: char) -> Option<VerticalConstruction> {
        let reader = self.reader().expect("checked when the font was made");
        let glyph = reader.face.glyph_index(ch)?;
        let c = reader.variants.vertical_constructions.get(glyph)?;
        Some(VerticalConstruction {
            variants: usize::from(c.variants.len()),
            assembly_parts: c.assembly.map(|a| usize::from(a.parts.len())),
        })
    }

    pub(crate) fn reader(&self) -> Result<Reader<'_>, FontError> {
        let face =
            Face::parse(&self.bytes, 0).map_err(|e| FontError(format!("not a font: {e}")))?;
        let math = face
            .tables()
            .math
            .ok_or_else(|| FontError("the font has no MATH table".to_owned()))?;
        let (Some(constants), Some(glyph_info), Some(variants)) =
            (math.constants, math.glyph_info, math.variants)
        else {
            return Err(FontError(
                "the font's MATH table lacks its constants, glyph info or variants".to_owned(),
            ));
        };
        let mut ssty = Vec::new();
        if let Some(gsub) = face.tables().gsub {
            for feature in gsub.features {
                if feature.tag != Tag::from_bytes(b"ssty") {
                    continue;
                }
                for index in feature.lookup_indices {
                    let Some(lookup) = gsub.lookups.get(index) else {
                        continue;
                    };
                    for sub in lookup.subtables.into_iter::<SubstitutionSubtable>() {
                        if let SubstitutionSubtable::Single(single) = sub {
                            ssty.push(single);
                        }
                    }
                }
            }
        }
        Ok(Reader {
            upem: f64::from(face.units_per_em()),
            face,
            constants,
            glyph_info,
            variants,
            ssty,
        })
    }
}

/// A parsed face and its MATH table, for the time of one layout.
pub(crate) struct Reader<'a> {
    pub face: Face<'a>,
    pub upem: f64,
    pub constants: Constants<'a>,
    pub glyph_info: GlyphInfo<'a>,
    pub variants: Variants<'a>,
    /// The single substitutions of the `ssty` lookups; a font whose `ssty` lookups are of another
    /// type has its scripts set at their base glyphs.
    ssty: Vec<SingleSubstitution<'a>>,
}

impl<'a> Reader<'a> {
    /// The `ssty` form of a glyph, the first substitution of the feature's lookups that covers it.
    pub fn script_glyph(&self, glyph: GlyphId) -> GlyphId {
        for single in &self.ssty {
            match *single {
                SingleSubstitution::Format1 { coverage, delta } => {
                    if coverage.get(glyph).is_some() {
                        return GlyphId((i32::from(glyph.0) + i32::from(delta)) as u16);
                    }
                }
                SingleSubstitution::Format2 {
                    coverage,
                    substitutes,
                } => {
                    if let Some(sub) = coverage.get(glyph).and_then(|i| substitutes.get(i)) {
                        return sub;
                    }
                }
            }
        }
        glyph
    }

    pub fn italic_correction(&self, glyph: GlyphId) -> f64 {
        self.glyph_info
            .italic_corrections
            .and_then(|v| v.get(glyph))
            .map_or(0.0, |v| f64::from(v.value))
    }

    pub fn top_accent(&self, glyph: GlyphId) -> Option<f64> {
        self.glyph_info
            .top_accent_attachments
            .and_then(|v| v.get(glyph))
            .map(|v| f64::from(v.value))
    }

    pub fn vertical(&self, glyph: GlyphId) -> Option<GlyphConstruction<'a>> {
        self.variants.vertical_constructions.get(glyph)
    }

    pub fn advance(&self, glyph: GlyphId) -> f64 {
        f64::from(self.face.glyph_hor_advance(glyph).unwrap_or(0))
    }

    /// The glyph's ink above and below the baseline, in design units, from its bounding box; zero
    /// for a glyph with no outline.
    pub fn ink(&self, glyph: GlyphId) -> (f64, f64) {
        match self.face.glyph_bounding_box(glyph) {
            Some(b) => (f64::from(b.y_max), -f64::from(b.y_min)),
            None => (0.0, 0.0),
        }
    }
}
