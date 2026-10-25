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
`path@rootpoint-model@CLAUDE.md` records, and an asset's path names a file of a store only the caller
knows. The two sets are part of the crate's public interface.

### A text area's layout comes from one function, which drawing and `area_text` both call `##area-text-layout`

`render_slide` lays each text area out, and places it, through one private function, which
`area_text` also calls: it gives the area's Parley `Layout`, in slide units, each run carrying its
colour as the brush `TextBrush`, and the point of the slide where its origin is drawn. The text's
tree, `design@rootpoint-model@text-node-tree`, is laid out as one string with `ranged_builder`,
each span's values set over its byte range, and `AreaText` carries the `TextMap` from each caret
position of the tree to its byte index in that string, and each span's byte range; the ranged
builder collapses no whitespace, as the doc comment of `RangedBuilder` in Parley 0.12.0's
`builder.rs` says, so those byte indices are the stored characters'. A formula stands in that string
on a mark of its own, `design@rootpoint-render@formula-mark`, and a reference on the characters of
the number it draws, `(n)` or `(??)`; `AreaText` carries each formula's byte range, its layout and
the point its origin is drawn at. An editor that places a caret or a selection on that layout, with
Parley's `Selection` and `Cursor`, places it on the drawn glyphs, which
`goal@rootpoint-render@same-slide-everywhere` asks of every display. `area_text` and its `AreaText` are part
of the crate's public interface; the app's editing in place, `design@rootpoint-app@edit-in-place`,
consumes it. A layout the editor made itself, from the same styles set again, would be a second
layout that must agree with the drawn one, as `rejected@rootpoint-app@plain-editor` records of
Parley's own editor.

### A formula of a text is an inline box, on a word joiner of the laid-out string `##formula-mark`

Each formula of a text is laid out by the math crate, in text style, at the size and in the colour
its text node resolves to, and placed in the text's one Parley layout as an inline box of its width,
its height and its baseline, at the byte index of a U+2060, the word joiner, that the laid-out
string holds for it. The caret's positions before and after the formula are then two byte indices,
either side of the box. The nearest rival, the box at an index with no character of its own,
`rejected@rootpoint-render@inline-box-without-mark`, leaves no caret position between the box and
the character after it, since Parley 0.12 gives an inline box no byte of the string; measured by
`a_box_with_no_character_after_it_leaves_no_position_after_it` of
`path@rootpoint-render@src/text.rs`, `cargo test -p rootpoint-render --lib`, which also finds that
Open Sans draws U+2060 with a glyph of its own and no advance, so the mark is no missing glyph and
moves nothing.

### What an area sets in type is inset from it by 24 slide units, and an image is not `##typeset-padding`

Every text is laid out within its area less 24 slide units on each side, and an equation is fitted
within the same rectangle, so a formula's edge and its number line up with the texts beside it; an
image fits or fills its whole area. The
padding sets the wrap width of every text, so changing it moves the line breaks of every saved
slide. The value is the session's choice at the design audit of the renderer; the owner judged it
right, and may adjust it later.

### An equation area is drawn from one layout of its formula and a scale `##equation-one-layout`

`lay_out_equation` of `path@rootpoint-render@src/formula.rs` lays the formula out once, in display
style, at the Body style's size for `Fit` and at its size for `AtMost`, and draws it at the largest
scale at which it fits the area less the padding of `design@rootpoint-render@typeset-padding`, with
room kept for the number on both sides, a scale no greater than 1 for `AtMost`,
`design@rootpoint-model@equation-fit`. `area_formula`, a public function of the crate, gives that
layout, the point where its origin is drawn and the scale, as `render_slide` draws them. One layout
and one multiplication suffice, since a layout at `k` times a size is the layout at that size scaled
by `k`, `design@rootpoint-math@layout-scales-exactly`; the rival, a search over sizes, lays the
formula out once for each size it tries, several times on every frame of a divider's drag where one
suffices.

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
to the built-in families alone: the text's, `design@rootpoint-render@builtin-open-sans`, and the
math font's, `design@rootpoint-render@builtin-libertinus-math`. A text whose font no font of the set
has falls back to the built-in sans, and the render reports it; a character neither has is drawn
from the math font, `design@rootpoint-render@text-falls-back-to-math-font`.

The rival, system fonts with a fallback, `rejected@rootpoint-render@system-fonts`, lays a slide
out differently on another computer, whose fonts differ, and in the browser, which has none, so it
fails `goal@rootpoint-render@same-slide-everywhere`. Whether a needed glyph is missing from every
available font is watched by `tripwire@rootpoint-render@glyph-missing-everywhere`.

### The built-in sans is Open Sans, in the four static styles of its `noto-set` build `##builtin-open-sans`

`FontRef::Builtin("sans")` names Open Sans, regular, bold, italic and bold italic, of the `noto-set`
build of googlefonts/opensans, in `path@rootpoint-render@fonts/open-sans/` under the SIL Open Font
License 1.1, whose text sits beside the files. The four files weigh 2,159,352 bytes, which every
build carries. Changing the built-in font changes every saved slide that names it.

The `noto-set` build covers every block section 4 of
`path@rootpoint@docs/grounding/math-typesetting.md` measured as fully as Noto Sans does, and 45 of
the 112 code points of Hebrew and 13 mathematical operators besides, among them the minus sign, ≤,
≥, ≠, ≈ and ∞, which Noto Sans lacks; coverage is the concern
`tripwire@rootpoint-render@glyph-missing-everywhere` watches. Its rival, the plain build, is
`rejected@rootpoint-render@open-sans-plain-build`. A second built-in family is
`rejected@rootpoint-render@two-builtin-text-families`.

On the owner's word: the owner considered Libertinus for the text too, and, from rendered samples
of text fonts beside Libertinus Math, prefers the look of Noto Sans and of Open Sans and finds them
the easier to read, with a slight preference for Open Sans.

### A character the text's fonts lack is drawn from the math font `##text-falls-back-to-math-font`

The font stack of every text ends with Libertinus Math, after the text's font and the built-in
sans, so a character neither has, as an arrow, is drawn from the math font, character by
character, and the report's `missing_glyphs` names only what no built-in font has. Open Sans and
Noto Sans have no arrow among the 112 of their block, where Libertinus Math has all of them, as
measured in `path@rootpoint@docs/grounding/math-typesetting.md`, sections 2 and 4, so an author's `→`
would otherwise be a box. The test
`an_arrow_typed_in_a_text_is_drawn_from_the_math_font` of `path@rootpoint-render@src/text.rs`
checks it. The fallback adds no weight to a build, since the math font is built in for formulas,
`design@rootpoint-render@builtin-libertinus-math`.

### The built-in math font is Libertinus Math `##builtin-libertinus-math`

`path@rootpoint-render@fonts/libertinus/LibertinusMath-Regular.otf`, its version 7.051, under the SIL Open Font
License 1.1, whose text sits beside it, is registered in every `FontSet`, and every formula is laid
out from its MATH table and drawn with its glyphs. It weighs 588,848 bytes, which every build
carries, the browser build included. Changing it changes the look and
the layout of every saved formula, and the table of commands of
`path@rootpoint-math@src/commands.rs`, which splits its commands by whether it has their
character.

Of the math commands of KaTeX v0.19.1 of one character, it lacks 44, recorded here on the owner's
word: `\lessapprox`, `\gtrapprox`, `\lnapprox`, `\gnapprox`, `\eqslantless`, `\eqslantgtr`,
`\lesseqqgtr`, `\gtreqqless`, `\precapprox`, `\succapprox`, `\precnapprox`, `\succnapprox`,
`\subseteqq`, `\supseteqq`, `\subsetneqq`, `\supsetneqq`, `\Subset`, `\Supset`, `\bumpeq`,
`\Bumpeq`, `\pitchfork`, `\multimap`, `\Cap`, `\Cup`, `\doublecap`, `\doublecup`, `\barwedge`,
`\veebar`, `\doublebarwedge`, `\divideontimes`, `\leftthreetimes`, `\rightthreetimes`,
`\intercal`, `\diamond`, `\triangleleft`, `\triangleright`, `\bigcirc`, `\blacklozenge`,
`\checkmark`, `\maltese`, `\diagup`, `\diagdown`, `\Finv` and `\Game`. The test
`the_missing_set_is_pinned_both_ways` of `path@rootpoint-math@tests/font.rs` pins that set, so a
change of the font that changes its coverage of those commands fails it. Its `ssty` feature
substitutes the six primes alone, measured with ttf-parser 0.25.1, so its other glyphs are set in
scripts at their base form, scaled; its scripts are scaled to 80 and 60 per cent; it has no math
kerning.

On the owner's word: the owner chose Libertinus Math from rendered samples of Noto Sans Math, Latin
Modern Math, New Computer Modern Math, STIX Two Math and Fira Math, and found its missing set not
much of a problem. The nearest rival, Noto Sans Math, scales its scripts to 60 and 50
per cent, has no math kerning and one weight, as measured in
`path@rootpoint@docs/grounding/math-typesetting.md`, sections 2 and 4, beside the other fonts.

### No fallback math font is built in: a character the math font lacks is drawn as its missing glyph `##no-fallback-math-font`

A formula holding a character Libertinus Math has no glyph for draws the font's glyph 0, and
`draw_formula` returns it to its caller, as a text's missing glyph is returned. A command of
such a character is read as that character, `design@rootpoint-math@absent-commands-read`. The rival, a second math font for those
characters alone, is `rejected@rootpoint-render@stix-math-fallback`; the owner judged the absence of
the 44 commands not much of a problem, by `design@rootpoint-render@builtin-libertinus-math`.
Whether a real talk needs one of them is watched by
`tripwire@rootpoint-render@math-glyph-needed`.
