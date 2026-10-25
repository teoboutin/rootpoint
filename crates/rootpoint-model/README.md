# rootpoint-model

The document model of rootpoint: a presentation, its slides, the tree of splits that divides each
slide into areas, the content of each area, and the operations that change them. It opens no
window and needs no GUI, so every editing operation runs headless. It also builds for the browser.

Its public API: the types of `Document`, `apply` with the operations of `Op`, `area_rects` and
`node_rects`, the rectangles of a slide's areas and of all its nodes in slide units, and `save_ron`
and `load_ron` for the RON save file. A text is a tree, `TextContent::root`, of runs of characters
and spans, each span with an id from the document's counter and its own values over its parent's;
an offset in a text node counts characters, a span counting as one, and `TextContent::plain_text`
gives the characters in order. An area holds a text, an image or an equation, `EquationContent`: a
formula of `rootpoint-math`, written in the file as its LaTeX, whether it is numbered, its size
mode and its colour; `Document::equation_numbers` numbers the numbered ones. A text's style names a style of the document's table, which a new
document holds four of, Title, Subtitle, Body and Legend, or holds its own values;
`Document::resolve_style` gives the values it draws with. A span's style names a style of the span
style table, `Document::span_styles`, empty in a new document, or holds its own values, each
optional; `Document::resolve_span` gives the values it sets. A file or an operation that would
break a structural invariant of the document is refused, which `Document::validate` checks, an
area, a span or a style naming an asset id at or above the id counter included. A reference to an
asset the table lacks, below the counter, to a built-in family the binary does not carry, to a
style the table lacks, or to a span style the span style table lacks, is a cosmetic breach: the file loads and the operation applies, the renderer draws a
stand-in, and `Document::breaches` names each area that draws one, and each span naming a missing
span style; a style of the table whose font
the table lacks is named once a text names it. An asset is the path of its file, never its bytes;
`asset_path_is_confined` tells a path below its store's root, and an asset whose path fails it is a
cosmetic breach too, named by its id and drawn as a missing one.