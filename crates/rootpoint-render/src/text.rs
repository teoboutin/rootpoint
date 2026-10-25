//! Fonts and text: the fonts a render may use, and the layout of a text in slide units.

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use parley::fontique::{Blob, CollectionOptions, FontInfoOverride};
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, FontFamilyName, FontStyle, FontWeight,
    Layout, LayoutContext, PositionedLayoutItem, StyleProperty,
};
use rootpoint_model::{AssetId, BUILTIN_FAMILIES, FontRef, HAlign, Rgba, SpanStyle, TextStyle};
use vello_cpu::kurbo::{Affine, Diagonal2};

/// The family of the built-in sans, which `FontRef::Builtin("sans")` names.
const BUILTIN_SANS: &str = "Noto Sans";

const BUILTIN_FONTS: [&[u8]; 4] = [
    include_bytes!("../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../fonts/NotoSans-Bold.ttf"),
    include_bytes!("../fonts/NotoSans-Italic.ttf"),
    include_bytes!("../fonts/NotoSans-BoldItalic.ttf"),
];

/// The synthetic bold of a font with no bold face: the outline grows by this fraction of the
/// font size on each side.
const EMBOLDEN: f64 = 0.02;

/// The colour of a run of a laid-out text: the brush Parley carries for each run, so that a span's
/// colour is drawn with its glyphs. Its default, which every text replaces by its style's colour,
/// is opaque black.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextBrush(pub Rgba);

impl Default for TextBrush {
    fn default() -> Self {
        TextBrush(Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        })
    }
}

/// Why an asset's bytes gave no font.
#[derive(Clone, Debug, PartialEq)]
pub struct FontError(pub String);

/// The fonts a render may use: the built-in Noto Sans and the font files of the document's asset
/// table, and nothing else. No font of the system is ever discovered: parley's feature `system`
/// is off, and the collection is created without system fonts.
pub struct FontSet {
    fonts: FontContext,
    layouts: LayoutContext<TextBrush>,
    assets: BTreeMap<AssetId, String>,
}

impl Default for FontSet {
    fn default() -> Self {
        FontSet::new()
    }
}

impl FontSet {
    /// The built-in fonts alone.
    pub fn new() -> Self {
        let mut fonts = FontContext {
            collection: parley::fontique::Collection::new(CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            source_cache: Default::default(),
        };
        for bytes in BUILTIN_FONTS {
            fonts
                .collection
                .register_fonts(Blob::new(Arc::new(bytes)), None);
        }
        FontSet {
            fonts,
            layouts: LayoutContext::new(),
            assets: BTreeMap::new(),
        }
    }

    /// Register the bytes of a font file, TrueType or OpenType, as the font of the asset `id`.
    /// It is registered under a family of its own, so that two assets of one family, as a
    /// regular and a bold file, stay two fonts that `FontRef::Asset` tells apart.
    pub fn insert_bytes(&mut self, id: AssetId, bytes: Vec<u8>) -> Result<(), FontError> {
        let family = format!("rootpoint asset {}", id.0);
        let families = self.fonts.collection.register_fonts(
            Blob::new(Arc::new(bytes)),
            Some(FontInfoOverride {
                family_name: Some(&family),
                ..Default::default()
            }),
        );
        if families.is_empty() {
            return Err(FontError(format!("asset {id} holds no font")));
        }
        self.assets.insert(id, family);
        Ok(())
    }

    /// The family a font reference resolves to, or `None` when no font of the set has it and the
    /// built-in sans stands in. A built-in name is resolved by the model's `BUILTIN_FAMILIES`, so
    /// that a family the model reports as unknown is one drawn here in the built-in sans. The
    /// binary carries one family, which `Builtin("sans")` names.
    fn family(&self, font: &FontRef) -> Option<String> {
        match font {
            FontRef::Builtin(name) if BUILTIN_FAMILIES.contains(&name.as_str()) => {
                Some(BUILTIN_SANS.to_owned())
            }
            FontRef::Builtin(_) => None,
            FontRef::Asset(id) => self.assets.get(id).cloned(),
        }
    }
}

/// A text laid out in slide units, with the lines' ranges in the text.
pub(crate) struct TextLayout {
    pub layout: Layout<TextBrush>,
    /// The byte range of the text each line holds, within the text's length.
    pub lines: Vec<Range<usize>>,
    /// Whether the text's font, or a font a span sets, was missing, and the built-in sans stood
    /// in.
    pub fell_back: bool,
    /// Whether a character has no glyph in any font of the stack, and is drawn as a box.
    pub missing_glyph: bool,
}

/// Lay a text out in slide units in the style `style`, each span's values in `spans` set over its
/// byte range, wrapped to `width` slide units. `spans` lists the spans outer before inner, as a
/// later range's value wins where two overlap. The layout depends on no pixel size: every display
/// draws this one layout through its own scale.
pub(crate) fn layout_text(
    fonts: &mut FontSet,
    text: &str,
    style: &TextStyle,
    spans: &[(Range<usize>, SpanStyle)],
    width: f32,
) -> TextLayout {
    // The built-in sans ends each stack, so a character the chosen font lacks is drawn in it.
    let mut fell_back = false;
    let mut stack = |fonts: &FontSet, font: &FontRef| {
        let resolved = fonts.family(font);
        fell_back |= resolved.is_none();
        let family = resolved.unwrap_or_else(|| BUILTIN_SANS.to_owned());
        vec![family, BUILTIN_SANS.to_owned()]
    };
    let area_stack = stack(fonts, &style.font);
    let span_stacks: Vec<Option<Vec<String>>> = spans
        .iter()
        .map(|(_, s)| s.font.as_ref().map(|f| stack(fonts, f)))
        .collect();
    let area_names = family_names(&area_stack);
    let span_names: Vec<Option<Vec<FontFamilyName<'_>>>> = span_stacks
        .iter()
        .map(|s| s.as_deref().map(family_names))
        .collect();
    let mut builder = fonts
        .layouts
        .ranged_builder(&mut fonts.fonts, text, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::List(
        area_names.as_slice().into(),
    )));
    builder.push_default(StyleProperty::FontSize(style.size as f32));
    builder.push_default(StyleProperty::FontWeight(weight(style.bold)));
    builder.push_default(StyleProperty::FontStyle(slant(style.italic)));
    builder.push_default(StyleProperty::Brush(TextBrush(style.colour)));
    for ((range, span), names) in spans.iter().zip(&span_names) {
        let range = range.clone();
        if let Some(names) = names {
            builder.push(
                StyleProperty::FontFamily(FontFamily::List(names.as_slice().into())),
                range.clone(),
            );
        }
        if let Some(size) = span.size {
            builder.push(StyleProperty::FontSize(size as f32), range.clone());
        }
        if let Some(bold) = span.bold {
            builder.push(StyleProperty::FontWeight(weight(bold)), range.clone());
        }
        if let Some(italic) = span.italic {
            builder.push(StyleProperty::FontStyle(slant(italic)), range.clone());
        }
        if let Some(colour) = span.colour {
            builder.push(StyleProperty::Brush(TextBrush(colour)), range.clone());
        }
    }
    let mut layout: Layout<TextBrush> = builder.build(text);
    layout.break_all_lines(Some(width));
    let alignment = match style.h_align {
        HAlign::Left => Alignment::Left,
        HAlign::Center => Alignment::Center,
        HAlign::Right => Alignment::Right,
    };
    layout.align(alignment, AlignmentOptions::default());
    let len = text.len();
    let lines = layout
        .lines()
        .map(|line| {
            let r = line.text_range();
            r.start.min(len)..r.end.min(len)
        })
        .collect();
    // Glyph 0 is the font's box for a character it lacks.
    let missing_glyph = layout.lines().any(|line| {
        line.items().any(|item| match item {
            PositionedLayoutItem::GlyphRun(run) => run.glyphs().any(|g| g.id == 0),
            PositionedLayoutItem::InlineBox(_) => false,
        })
    });
    TextLayout {
        layout,
        lines,
        fell_back,
        missing_glyph,
    }
}

/// A stack of families as Parley names them.
fn family_names(stack: &[String]) -> Vec<FontFamilyName<'_>> {
    stack.iter().map(|f| FontFamilyName::named(f)).collect()
}

fn weight(bold: bool) -> FontWeight {
    if bold {
        FontWeight::BOLD
    } else {
        FontWeight::NORMAL
    }
}

fn slant(italic: bool) -> FontStyle {
    if italic {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    }
}

/// Draw a laid-out text with its top-left corner at the origin of the context's transform, each
/// glyph run in its run's colour.
/// Returns false when a glyph run could not be drawn, as from a damaged font file.
pub(crate) fn draw_text(
    ctx: &mut vello_cpu::RenderContext,
    resources: &mut vello_cpu::Resources,
    text: &TextLayout,
) -> bool {
    let mut drawn = true;
    for line in text.layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let TextBrush(c) = glyph_run.style().brush;
            ctx.set_paint(
                vello_cpu::color::AlphaColor::<vello_cpu::color::Srgb>::from_rgba8(
                    c.r, c.g, c.b, c.a,
                ),
            );
            let size = run.font_size();
            let synthesis = run.synthesis();
            let glyphs = glyph_run.positioned_glyphs().map(|g| vello_cpu::Glyph {
                id: g.id,
                x: g.x,
                y: g.y,
            });
            let mut builder = ctx
                .glyph_run(resources, run.font())
                .font_size(size)
                // Hinting snaps outlines to the pixel grid, which would make the slide differ
                // between sizes.
                .hint(false);
            // A font with no bold or italic face is emboldened or slanted, as parley asks.
            if synthesis.embolden() {
                let amount = EMBOLDEN * f64::from(size);
                builder =
                    builder.font_embolden(glifo::FontEmbolden::new(Diagonal2::new(amount, amount)));
            }
            if let Some(degrees) = synthesis.skew() {
                let slant = f64::from(degrees).to_radians().tan();
                builder = builder.glyph_transform(Affine::skew(-slant, 0.0));
            }
            // No variation coordinates are passed: a variable font is drawn at its default
            // instance, which the built-in static fonts do not need.
            drawn &= builder.fill_glyphs(glyphs).is_ok();
        }
    }
    drawn
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The set's collection holds the built-in family alone: a font of the system, loaded by
    /// either guard failing, would add a family.
    #[test]
    fn a_new_set_holds_the_built_in_family_alone() {
        let mut set = FontSet::new();
        let names: Vec<String> = set
            .fonts
            .collection
            .family_names()
            .map(str::to_owned)
            .collect();
        assert_eq!(names, vec![BUILTIN_SANS.to_owned()]);
    }

    /// The binary carries one family, so `family` resolves every built-in name to it: a second
    /// name in the model's list would need a family of its own here, and this test then fails.
    #[test]
    fn the_model_lists_the_one_built_in_family_the_set_carries() {
        assert_eq!(BUILTIN_FAMILIES, ["sans"]);
        let set = FontSet::new();
        assert_eq!(
            set.family(&FontRef::Builtin("sans".to_owned())),
            Some(BUILTIN_SANS.to_owned())
        );
        assert_eq!(set.family(&FontRef::Builtin("serif".to_owned())), None);
    }
}
