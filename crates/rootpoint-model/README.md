# rootpoint-model

The document model of rootpoint: a presentation, its slides, the tree of splits that divides each
slide into areas, the content of each area, and the operations that change them. It opens no
window and needs no GUI, so every editing operation runs headless. It also builds for the browser.

Its public API: the types of `Document`, `apply` with the operations of `Op`, `area_rects` and
`node_rects`, the rectangles of a slide's areas and of all its nodes in slide units, and `save_ron`
and `load_ron` for the RON save file. A file or an operation that would break an
invariant of the document is refused, which `Document::validate` checks.
