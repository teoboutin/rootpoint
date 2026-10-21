//! Fonts and text: the fonts a render may use, and the layout of a text in slide units.

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use parley::fontique::{Blob, CollectionOptions, GenericFamily};
use parley::{
    Alignment, AlignmentOptions, FontContext, FontStyle, FontWeight, Layout, LayoutContext,
    PositionedLayoutItem, StyleProperty,
};
use rootpoint_model::{AssetId, FontRef, HAlign, TextContent};

/// The family of the built-in sans, which `FontRef::Builtin("sans")` names.
const BUILTIN_SANS: &str = "Noto Sans";

const BUILTIN_FONTS: [&[u8]; 4] = [
    include_bytes!("../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../fonts/NotoSans-Bold.ttf"),
    include_bytes!("../fonts/NotoSans-Italic.ttf"),
    include_bytes!("../fonts/NotoSans-BoldItalic.ttf"),
];

/// Why an asset's bytes gave no font.
#[derive(Clone, Debug, PartialEq)]
pub struct FontError(pub String);

/// The fonts a render may use: the built-in Noto Sans and the font files of the document's asset
/// table, and nothing else. No font of the system is ever discovered.
pub struct FontSet {
    fonts: FontContext,
    layouts: LayoutContext<()>,
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
        let mut sans = Vec::new();
        for bytes in BUILTIN_FONTS {
            let families = fonts
                .collection
                .register_fonts(Blob::new(Arc::new(bytes)), None);
            sans.extend(families.into_iter().map(|(id, _)| id));
        }
        sans.dedup();
        // Characters a chosen font lacks fall back to the built-in sans.
        fonts
            .collection
            .set_generic_families(GenericFamily::SansSerif, sans.into_iter());
        FontSet {
            fonts,
            layouts: LayoutContext::new(),
            assets: BTreeMap::new(),
        }
    }

    /// Register the bytes of a font file, TrueType or OpenType, as the font of the asset `id`.
    pub fn insert_bytes(&mut self, id: AssetId, bytes: Vec<u8>) -> Result<(), FontError> {
        let families = self
            .fonts
            .collection
            .register_fonts(Blob::new(Arc::new(bytes)), None);
        let (family, _) = families
            .first()
            .ok_or_else(|| FontError(format!("asset {id} holds no font")))?;
        let name = self
            .fonts
            .collection
            .family_name(*family)
            .ok_or_else(|| FontError(format!("asset {id} holds a font with no family name")))?
            .to_owned();
        self.assets.insert(id, name);
        Ok(())
    }

    /// The family a font reference resolves to, or `None` when no font of the set has it, and the
    /// built-in sans stands in.
    fn family(&mut self, font: &FontRef) -> Option<String> {
        let name = match font {
            FontRef::Builtin(name) if name == "sans" => BUILTIN_SANS.to_owned(),
            FontRef::Builtin(name) => name.clone(),
            FontRef::Asset(id) => self.assets.get(id)?.clone(),
        };
        self.fonts.collection.family_id(&name).map(|_| name)
    }
}

/// A text laid out in slide units, with the lines' ranges in the text.
pub(crate) struct TextLayout {
    pub layout: Layout<()>,
    /// The byte range of the text each line holds.
    pub lines: Vec<Range<usize>>,
    /// Whether the text's font was missing, and the built-in sans stood in.
    pub fell_back: bool,
}

/// Lay a text out in slide units, wrapped to `width` slide units. The layout depends on no pixel
/// size: every display draws this one layout through its own scale.
pub(crate) fn layout_text(fonts: &mut FontSet, text: &TextContent, width: f32) -> TextLayout {
    let resolved = fonts.family(&text.style.font);
    let fell_back = resolved.is_none();
    let family = resolved.unwrap_or_else(|| BUILTIN_SANS.to_owned());
    let style = &text.style;
    let mut builder = fonts
        .layouts
        .ranged_builder(&mut fonts.fonts, &text.text, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(family.as_str().into()));
    builder.push_default(StyleProperty::FontSize(style.size as f32));
    if style.bold {
        builder.push_default(StyleProperty::FontWeight(FontWeight::BOLD));
    }
    if style.italic {
        builder.push_default(StyleProperty::FontStyle(FontStyle::Italic));
    }
    let mut layout: Layout<()> = builder.build(&text.text);
    layout.break_all_lines(Some(width));
    let alignment = match style.h_align {
        HAlign::Left => Alignment::Left,
        HAlign::Center => Alignment::Center,
        HAlign::Right => Alignment::Right,
    };
    layout.align(alignment, AlignmentOptions::default());
    let lines = layout.lines().map(|line| line.text_range()).collect();
    TextLayout {
        layout,
        lines,
        fell_back,
    }
}

/// Draw a laid-out text with its top-left corner at the origin of the context's transform.
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
            let glyphs = glyph_run.positioned_glyphs().map(|g| vello_cpu::Glyph {
                id: g.id,
                x: g.x,
                y: g.y,
            });
            drawn &= ctx
                .glyph_run(resources, run.font())
                .font_size(run.font_size())
                // Hinting snaps outlines to the pixel grid, which would make the slide differ
                // between sizes.
                .hint(false)
                // No variation coordinates are passed: a variable font is drawn at its default
                // instance, which the built-in static fonts do not need.
                .fill_glyphs(glyphs)
                .is_ok();
        }
    }
    drawn
}
