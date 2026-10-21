# Design of rootpoint-render

How rootpoint-render is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-render is for is in `register@rootpoint-render@goal`.

## Drawing a slide

### Slides are drawn by the project's own renderer, from one layout in slide units `##slide-renderer`

`render_slide` lays each text out with parley in slide units, the model's
`design@rootpoint-model@slide-units`, and draws the slide with vello_cpu through one uniform
transform from slide units to pixels, glyphs unhinted. Every display of a slide, the editor,
presenter mode and a saved image, calls it, so they share one layout by construction, which
`goal@rootpoint-render@same-slide-everywhere` asks; hinting would snap outlines to each size's
pixel grid.

The rival, drawing slides with a GUI toolkit's own painter and text engine,
`rejected@rootpoint-render@toolkit-painter`, lays text out in pixels, so line breaks can move
between window sizes, and its headless output is only as good as the toolkit's offscreen path. On
the owner's word, too: the owner chose this renderer because, as the owner thinks, precise control
of the renderer is
important for the project. The function is the interface the editor, presenter mode and the
protocol's headless render consume.

### The renderer reads no file: the caller gives it every asset's decoded image and font bytes `##caller-supplies-assets`

`render_slide` takes an `ImageSet` and a `FontSet` the caller fills from the assets' bytes; it
opens no file. The renderer is part of the browser build,
`goal@rootpoint-render@browser-buildable`, where `std::fs` compiles and fails only at run time, and
an asset's path is relative to a presentation file only the caller knows. The two sets are the
interface the caller consumes.

## Fonts

### Fonts come only from the binary and the presentation's assets, never from the system `##font-sources`

A `FontSet` holds the built-in fonts and the font files of the document's asset table, and nothing
else: parley's feature `system` is off and the collection is created without system fonts, each
of which alone keeps system fonts out. A text whose font no font of the set has falls back to the
built-in sans, and the render reports it.

The rival, system fonts with a fallback, `rejected@rootpoint-render@system-fonts`, lays a slide
out differently on another computer, whose fonts differ, and in the browser, which has none, so it
fails `goal@rootpoint-render@same-slide-everywhere`. Whether a needed glyph is missing from every
available font is watched by `tripwire@rootpoint-render@glyph-missing-everywhere`.

### The built-in sans is Noto Sans, in its four static styles `##builtin-noto-sans`

`FontRef::Builtin("sans")` names Noto Sans, regular, bold, italic and bold italic, embedded in the
binary under the SIL Open Font License 1.1, whose text sits beside the files; it also ends every
text's font stack, so a character a chosen font lacks is drawn in it. Noto's glyph coverage serves
`tripwire@rootpoint-render@glyph-missing-everywhere`'s concern, and changing the built-in font
changes every saved slide that names it. The four files weigh 2.1 MB, which every build carries.
