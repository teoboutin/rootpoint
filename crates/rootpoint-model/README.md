# rootpoint-model

The document model of rootpoint: a presentation, its slides, the tree of splits that divides each
slide into areas, the content of each area, and the operations that change them. It opens no
window and needs no GUI, so every editing operation runs headless. It also builds for the browser.

Its public API: the types of `Document`, `apply` with the operations of `Op`, `area_rects` and
`node_rects`, the rectangles of a slide's areas and of all its nodes in slide units, and `save_ron`
and `load_ron` for the RON save file. A text's style names a style of the document's table, which
a new document holds four of, Title, Subtitle, Body and Legend, or holds its own values;
`Document::resolve_style` gives the values it draws with. A file or an operation that would break
a structural invariant of the document is refused, which `Document::validate` checks, an area or a
style naming an asset id at or above the id counter included. A reference to an asset the table
lacks, below the counter, to a built-in family the binary does not carry, or to a style the table
lacks, is a cosmetic breach: the file loads and the operation applies, the renderer draws a
stand-in, and `Document::breaches` names it.
