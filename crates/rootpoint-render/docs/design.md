# Design of rootpoint-render

How rootpoint-render is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-render is for is in `register@rootpoint-render@goal`.

## Drawing a slide

### Slides are drawn by the project's own renderer, from one layout in slide units `##slide-renderer`

`render_slide` lays each text out with parley in slide units, the model's
`design@rootpoint-model@slide-units`, and draws the slide with vello_cpu through one uniform
transform from slide units to pixels, glyphs unhinted. Every display of a slide draws through it,
so all share one layout by construction, which `goal@rootpoint-render@same-slide-everywhere` asks;
hinting would snap outlines to each size's pixel grid.

The rival, drawing slides with a GUI toolkit's own painter and text engine,
`rejected@rootpoint-render@toolkit-painter`, lays text out in pixels, so line breaks can move
between window sizes (assumed by the survey of the first prototype, not measured), and its headless
output is only as good as the toolkit's offscreen path. On the owner's word, too: the owner chose
this renderer because, as the owner thinks, precise control of the renderer is important for the
project. The function is the crate's public interface.

### The renderer reads no file: the caller gives it every asset's decoded image and font bytes `##caller-supplies-assets`

`render_slide` takes an `ImageSet` and a `FontSet` the caller fills from the assets' bytes; it
opens no file. The renderer is part of the browser build,
`goal@rootpoint-render@browser-buildable`, where `std::fs` compiles and fails only at run time, as
`path@rootpoint-model@CLAUDE.md` records, and an asset's path is relative to a presentation file
only the caller knows. The two sets are part of the crate's public interface.

### A text is inset from its area by 24 slide units `##text-padding`

Every text is laid out within its area less 24 slide units on each side. The padding sets the
wrap width of every text, so changing it moves the line breaks of every saved slide. The value is
the session's choice at the design audit of the renderer; the owner judged it right, and may
adjust it later.

### A render is limited to 16384 pixels a side `##size-limit`

`render_slide` refuses a side of zero or above `MAX_SIDE`, 16384 pixels, with
`RenderError::BadSize`. vello_cpu rounds a size up to its tiles in 16 bits and panics above 65408
pixels wide or 65532 high, measured at the review of the renderer; the limit keeps well below. The
limit and its error are part of the crate's public interface.

### Golden images are drawn with the scalar SIMD level `##portable-precision`

`Precision::Portable` runs vello_cpu's `Level::fallback()`, the scalar code, through fearless_simd's
feature `force_support_fallback`, so a golden image does not depend on the machine's SIMD:
`Level::baseline()` is SSE2 on the owner's machine and Neon on ARM, read in fearless_simd 0.7 and
printed at the review of the renderer. `Precision::Native` runs the fastest level the machine has,
for display. `Precision` is a parameter of the crate's public interface.

## Fonts

### Fonts come only from the binary and the presentation's assets, never from the system `##font-sources`

A `FontSet` holds the built-in fonts and the font files of the document's asset table, and nothing
else. Two guards keep system fonts out: parley's feature `system` is off, the default that turns on
system font discovery, as parley 0.12's `Cargo.toml` reads; and the collection is created without
system fonts, which fontique's collection honours whatever the feature. A unit test pins a new set
to the built-in family alone. A text whose font no font of the set has falls back to the built-in
sans, and the render reports it.

The rival, system fonts with a fallback, `rejected@rootpoint-render@system-fonts`, lays a slide
out differently on another computer, whose fonts differ, and in the browser, which has none, so it
fails `goal@rootpoint-render@same-slide-everywhere`. Whether a needed glyph is missing from every
available font is watched by `tripwire@rootpoint-render@glyph-missing-everywhere`.

### The built-in sans is Noto Sans, in its four static styles `##builtin-noto-sans`

`FontRef::Builtin("sans")` names Noto Sans, regular, bold, italic and bold italic, embedded in the
binary under the SIL Open Font License 1.1, whose text sits beside the files. Noto's glyph coverage
serves the concern `tripwire@rootpoint-render@glyph-missing-everywhere` watches, and changing the
built-in font changes every saved slide that names it. The four files weigh 2.1 MB, which every
build carries. The family is the session's choice at the design audit of the renderer, among the
fonts under the Open Font License on the owner's machine.
