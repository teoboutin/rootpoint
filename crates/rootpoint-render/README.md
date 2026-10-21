# rootpoint-render

The slide renderer of rootpoint: it draws one slide of a document into an RGBA image of any size,
scaled uniformly from slide units, so the editor, presenter mode and a headless PNG show the same
slide. It reads no file and no system font, and it builds for the browser.

Its public API: `render_slide` takes a document, a slide, a pixel size, a `FontSet`, an `ImageSet`
and a `Precision`, and returns a `Rendered`, the image with a `Report` of what it could not draw as
asked; `Rendered::png` encodes the image as PNG bytes. `FontSet` holds the built-in Noto Sans and
the fonts the caller registers from a document's font assets; `ImageSet` holds the images the
caller decodes from the image assets' bytes.
