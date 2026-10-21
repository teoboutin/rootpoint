# Design of rootpoint-model

How rootpoint-model is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-model is for is in `register@rootpoint-model@goal`.

## The tree of a slide

### A split divides its extent along one axis into any number of children, and never holds a child split of its own axis `##split-tree-arity`

A slide's root is a node: an area, or a split with an axis and k ≥ 2 children separated by k−1
dividers. A `Vertical` split lays its children side by side, left to right; a `Horizontal` split
stacks them, top to bottom. A split along the axis of the split that holds it is merged into it,
and so is a child left alone by a removal when its axis is its new parent's, so each layout has
exactly one tree.

The nearest rival, binary splits, `rejected@rootpoint-model@binary-splits`, gives one layout
several trees, by the order the splits were made in: removing an area or dragging a divider then
behaves differently on layouts that look the same, and the file and the protocol's listings show
a structure the author never chose. With the merge, dragging a divider moves only its two
neighbours. The tree is the file's structure, so it is an interface the renderer, the protocol and
every saved file consume. It serves `goal@rootpoint@free-slide-division`. Whether merging removes a
grouping an author wants is watched by `tripwire@rootpoint-model@grouping-lost-to-merge`.

### Any node can be split, so an area can span the areas beside it `##split-any-node`

`Split` applies to an area or to a split: a split along another axis than its parent's wraps the
node in a new split, so a new area can span the areas the node holds, as a title spans two
columns. This reaches every guillotine layout, a layout made by cuts that each run across the area
being cut. It does not reach a layout no such cut makes, such as four rectangles around a fifth;
only constraints between dividers, which no head records, would.

The rival, splitting leaves only, makes a spanning area impossible once the areas beside it exist:
the author would have to rebuild the slide. It serves `goal@rootpoint@free-slide-division`.

## Coordinates

### Sizes are in slide units of a slide 1080 units high, and divider positions are fractions of their split `##slide-units`

A document has one aspect ratio, 16:9 by default; a slide is 1080 slide units high and as wide as
the ratio makes it, 1920 for 16:9, so a font size of 48 means what it would on a 1080p screen.
`area_rects` gives each area's rectangle in slide units. No size is in pixels.

The rivals fail `goal@rootpoint@presenter-mode` with `goal@rootpoint@real-presentations`, which
need the slide shown in the editor to be the slide shown in presenter mode at any window size:
pixel units, `rejected@rootpoint-model@pixel-units`, wrap the same text differently on a 1080p
laptop and a 4K projector, and a free aspect ratio that reflows text,
`rejected@rootpoint-model@reflowing-slides`, shows another slide in each window. The renderer
draws in these units, and the file stores them.

## Content and assets

### An area holds its content, and heavy data is an asset the content refers to by id `##inline-content-assets`

An area holds one content: empty, a text with its style, or an image. Image data and font files
live once in the document's asset table, keyed by asset id, so a slide's text is read where it is
shown, and an image placed on many slides is stored once.

The rival, every content stored apart and referenced by areas,
`rejected@rootpoint-model@content-by-reference`, costs garbage collection or a library of orphans,
editing that changes every slide sharing a content, a file read by following ids, and content ids
in the protocol beside node ids; the owner weighed those costs above the reuse it gives. Reuse of
text across slides is left to templates, which are not built.

### An asset is a file beside the presentation, or its bytes when the presentation is packed `##asset-source`

An asset's source is a path relative to the presentation file or the file's bytes embedded in the
document, both in version 1 of the format. The editing form of a presentation is its text file
with its assets as files beside it; packing a presentation into one file to carry it embeds them.
The model writes paths only, since no packing is built yet.

On the owner's word: the owner chose files beside the presentation for prototyping, and, as the
owner thinks, that form stays the preferred editing method in the long term, while a packed single
file, presented from a static web app on any computer, is the final goal. Both sources are in the
format from its first version because adding the embedded one later would force a migration of
every saved file. It serves `goal@rootpoint@readable-save-file` and
`goal@rootpoint@runs-in-browser`.

### A text area has one style for all its text `##text-style-per-area`

A text's font, size, weight, slant, colour and alignment are set for the whole area. Styled spans
inside a text need an editor of rich text, the hardest part of a text editor to build, which the
first prototype leaves out; the owner judged one style per area enough for the owner's needs at
first. The style is part of the file.

## Changes

### Every change of a document is an operation, applied by one function that returns its inverse `##edit-operation-core`

`apply` takes an `Op`, changes the document, and returns the operation that undoes it; a
structural edit of a slide's tree returns a snapshot of the slide's former tree. The GUI, the
protocol and the tests change a document through nothing else, which gives undo to all of them.

The rival, the GUI changing the document directly,
`rejected@rootpoint-model@gui-mutation`, makes the protocol re-implement every edit, so
`goal@rootpoint@headless-core` and `goal@rootpoint@text-command-protocol` would cost two
implementations that drift apart. The operations are an interface the protocol and the app crate
consume.

### A document is validated when it is loaded and after every operation, and a breach is refused `##validated-documents`

`Document::validate` checks every invariant the types state: unique ids below the counter, splits
of two children or more with ordered dividers strictly inside (0, 1) and no child split of their
own axis, assets present for every area that names one, no zero side in the aspect ratio. `load_ron`
refuses a file that breaks one, and `apply` refuses an operation whose result breaks one, leaving
the document unchanged.

The rival, trusting a loaded file and the result of every operation,
`rejected@rootpoint-model@unvalidated-documents`, was refuted by counterexamples built at review: a
hand-edited file with a missing divider made `area_rects` and `apply` panic, a lowered id counter
made a later operation reuse ids, and rounding in a split or a merge made dividers equal, 0 or 1.
A file edited by hand is a purpose, `goal@rootpoint@readable-save-file`, so a breach is reachable.

### Ids are stored in the file, come from one counter, and are never reused `##stable-ids`

Every slide, node and asset has an id from one counter the document stores and only grows; ids are
written in the file and printed `s<n>`, `n<n>` and `a<n>`. A duplicated slide's nodes get new ids.
A command, a script or a link between dividers can then name a node across edits, saves and loads,
where a position in the tree shifts at every split or removal. The protocol addresses by these
ids, so they are an interface it consumes.

## The file

### A presentation is saved as RON text, with the version of its format first `##save-format`

`save_ron` writes the document as RON, its first field the format version, and `load_ron` refuses
another version. Ids are plain numbers, and the file enables ron's `unwrap_variant_newtypes`, so a
node reads `Split(id: 4, ...)`.

RON writes enum variants as they read and allows comments and trailing commas, which helps editing
by hand; the rival, JSON, `rejected@rootpoint-model@json-save-format`, is universal and slightly
better for tools other than rootpoint. The choice is held weakly. The version comes first so a
later change of format is met by a migration, not by a misreading. It serves
`goal@rootpoint@readable-save-file`, and the file is an interface every saved presentation
consumes.
