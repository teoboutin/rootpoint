# rootpoint-render

The slide renderer of rootpoint: it draws one slide of a document into an RGBA image of any size,
scaled uniformly from slide units, so the editor, presenter mode and a headless PNG show the same
slide. It reads no file and no system font, and it builds for the browser.

Its public API: `render_slide`, which takes a document, a slide and a pixel size, and returns the
image; `Rendered::png` writes it as a PNG file.
