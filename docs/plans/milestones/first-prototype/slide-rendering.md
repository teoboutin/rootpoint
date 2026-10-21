# Slice: The slide renderer and a headless PNG

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-render` at `planned@rootpoint@crates/rootpoint-render/`: `FontSet`,
  `render_slide`, which draws one slide of a document at a given pixel size into an RGBA image, and
  `RenderError`. Its library root reads `ROOTPOINT_CHECKOUT`, as `path@xtask@CLAUDE.md` requires.
- The dependency on parley with `default-features = false` and the feature `std`: parley's default
  feature `system` turns on system font discovery (read in parley 0.12.0's `Cargo.toml`), which
  `thread@first-prototype@font-sources` excludes.
- The built-in fonts: Noto Sans, under the SIL Open Font License 1.1, in regular, bold, italic and
  bold italic, embedded in the binary, with its licence beside the files. `FontRef::Builtin("sans")`
  names it. The files are those of the Debian package `fonts-noto-core` on the owner's machine,
  about 0.5 MB each; Noto's wide glyph coverage serves T2.
- `ImageSet`, the decoded images of a document's assets, which the caller builds from the bytes of
  each asset: the renderer reads no file, since `std::fs` fails at run time in the browser, as
  `path@rootpoint-model@CLAUDE.md` says. Images are decoded with the `image` crate, PNG and JPEG.
- `render_slide` returns the image with a report: the fonts that fell back to the built-in sans,
  and the assets that had no decoded image, drawn as an empty area.
- The crate added to the `wasm` gate.
- Per D2, the crate's Component.

## Claims

1. **One layout at every size.** For a fixture slide, the line breaks of every text area are the
   same at 960×540, 1920×1080 and 3840×2160 pixels. Text is laid out by one function in slide
   units, whatever the pixel size, and `render_slide` draws that layout through its scale. Test:
   compare the break positions of the layouts `render_slide` uses at each size. Shown to fail:
   laying text out at the pixel size in place of slide units moves a break, which the test
   reports.
2. **A slide renders the same twice.** Two renders of the same slide at the same size give
   identical bytes. Test: render twice, compare. Shown to fail: a render that depends on a
   hash-map iteration order differs.
3. **Golden images.** Each fixture renders to pixels equal to those of a committed golden PNG,
   compared pixel by pixel, with vello_cpu's portable `Level::fallback`, since its SIMD levels may
   differ by a pixel between machines. `ROOTPOINT_BLESS=1` rewrites the golden files after a
   deliberate change, as the model's fixture does. Shown to fail: changing a colour of the fixture
   breaks the comparison.
4. **No system font is ever loaded.** `FontSet` is built from the embedded fonts and the document's
   font assets only. Test: a text whose font names `DejaVu Sans`, a font installed on the owner's
   machine and absent from both, renders with the built-in sans, and the report names it as a
   fallback. Shown to fail: restoring parley's default features loads the system's fonts, and the
   test sees `DejaVu Sans` resolve.
5. **The render crate builds for the wasm target.** The `wasm` gate passes. Measured at the audit:
   a crate depending on parley 0.12 without `system`, vello_cpu 0.3 and image 0.25 with PNG and
   JPEG builds for the wasm target.

## Fixtures

Three slides built in code with the model crate's names: a text area in each `HAlign` and `VAlign`;
an image area in `Fit` and in `Fill`; a split tree of five areas mixing both. The image is a
committed PNG of the test directory.

## Audit subjects

- The crate documentation of `parley` 0.12.0 and `vello_cpu` 0.3.0, the latest releases on
  crates.io at the planning review, for drawing glyph runs and writing PNGs.
- `thread@first-prototype@slide-renderer`, `thread@first-prototype@font-sources`,
  `thread@first-prototype@slide-coordinate-space`.
- The model crate's `area_rects`, `TextStyle` and `ImageFit`, as landed.

## Fails alone on

- A wrong pixel in a golden image with `area_rects` correct, as the model's tests show.
- The `wasm` gate failing on `rootpoint-render` with `rootpoint-model` passing.

## Premises that expire

- egui 0.37.0 depends, through epaint, on `vello_cpu` 0.2 while this crate uses 0.3 (V, quoted in
  `argument@first-prototype@arg-26`), so the workspace builds two versions once the app crate
  exists. Guard: the audit of `spec@first-prototype@editor-shell` measures the build time and the
  binary size with both, and judges whether that costs anything.

## Implementation sequence

1. **An empty render.** `render_slide` fills the slide with white at the given size; the PNG written
   by a test. Measures the time of a 3840×2160 empty render as the baseline for AC2.
2. **Areas and images.** Draw images in `Fit` and `Fill` into their rectangles. Claims 2, 3.
3. **Text.** Parley layout in slide units, drawn through one scale transform, with hinting off.
   Claims 1, 3, 4.
4. **The wasm gate and AC2's 4K bound.** Claim 5; the measurement of AC2.

## Decided design

### What a slide's drawing holds

The slide is filled white, then each area is drawn inside its rectangle, clipped to it. A text is
laid out within its area less a padding of 24 slide units on every side, wrapped to that width,
placed by its vertical alignment, and clipped where it overflows. An image is scaled into the area
by its fit. The padding is the session's choice, which the owner may change.

### Our own renderer

Text is laid out with Parley in slide units, once per slide and style, and drawn with `vello_cpu`
through one scale transform from slide units to pixels. The editor, the presenter and a headless
PNG call the same `render_slide`, so they share one layout by construction
(`thread@first-prototype@slide-renderer`). Hinting is off, since it snaps glyphs to the pixel grid
and breaks the scale invariance of claim 1 (`argument@first-prototype@arg-26`).

Nearest rival: the toolkit's own painter; see "Losing alternatives" of the milestone document.

### Fonts

A `FontSet` holds the built-in fonts and the font files of the document's asset table, and nothing
else: system font discovery is off. A text whose font is absent falls back to the built-in sans and
the render reports the fallback (`thread@first-prototype@font-sources`).

Nearest rival: system fonts with a fallback; see "Losing alternatives" of the milestone document.

## Acceptance criteria

This slice judges the 4K bound of AC2, `acceptance@first-prototype@render-fast-enough`, at step 4:
10 renders of one 3840×2160 slide of the third fixture in a release build on the owner's machine,
each timed with `std::time::Instant`; the bound holds when every one of them is within it, and the
largest is reported.

## Harvest

Judged by the entry tests of `primer@design-heads`, into the design home of the render crate's
Component: `thread@first-prototype@slide-renderer`, with the owner's stated reason attributed to the
owner, and `thread@first-prototype@font-sources`; the losing alternatives, shape (a) and system
fonts, judged for a `rejected` entry; T2 into that Component's tripwires home.
