# rootpoint-render

The slide renderer of rootpoint: it draws one slide of a document into an RGBA image of any size,
scaled uniformly from slide units, so the editor, presenter mode and a headless PNG show the same
slide. It reads no file and no system font, and it builds for the browser.

Its public API: `render_slide` takes a document, a slide, a pixel size, a `FontSet`, an `ImageSet`
and a `Precision`, and returns a `Rendered`, the image with a `Report` of what it could not draw as
asked; `Rendered::png` encodes the image as PNG bytes. `FontSet` holds the built-in Noto Sans and
the fonts the caller registers from a document's font assets; `ImageSet` holds the images the
caller decodes from the image assets' bytes. `slide_to_pixels` gives the scale and the offset that
carry slide units to the pixels of an image of a given size, where `render_slide` draws, so a
caller places its own marks over the image, or tests a pointer against the slide, at the drawn
place. `area_text` gives the text of one area as `render_slide` lays it out, a Parley `Layout`
in slide units whose runs carry their colour as `TextBrush`, the point it is drawn at, and the
`TextMap` from each caret position of the text's tree to its byte index in the layout, so an editor
places a caret or a selection on the drawn glyphs with Parley's `Selection` and `Cursor`.
