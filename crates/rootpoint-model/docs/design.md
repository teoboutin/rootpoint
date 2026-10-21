# Design of rootpoint-model

How rootpoint-model is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-model is for is in `register@rootpoint-model@goal`.

## The tree of a slide

### A split divides its extent along one axis into any number of children, and never holds a child split of its own axis `##split-tree-arity`

A slide's root is a node: an area, or a split with an axis and k ≥ 2 children separated by k−1
dividers. A `LeftToRight` split lays its children side by side, left to right; a `TopToBottom`
split stacks them, top to bottom. The names say where the children go, in the file and in the
protocol's `beside` and `stacked`: the names of the dividers' direction, `Vertical` and
`Horizontal`, `rejected@rootpoint-model@divider-direction-names`, were read both ways by a fresh
agent. Among names that say where the children go, the owner chose `LeftToRight` and `TopToBottom`
over `SideBySide` and `Stacked`, as reading better. A split along the axis of the split that holds it is merged into it,
and so is a child left alone by a removal when its axis is its new parent's, so each layout has
exactly one tree.

The nearest rival, binary splits, `rejected@rootpoint-model@binary-splits`, gives one layout
several trees, by the order the splits were made in: removing an area or dragging a divider then
behaves differently on layouts that look the same, and the file shows a structure the author never
chose. With the merge, dragging a divider moves only its two neighbours. The tree is the structure
of every saved file and of the crate's public types. It serves `goal@rootpoint@free-slide-division`.
Whether merging removes a grouping an author wants is watched by
`tripwire@rootpoint-model@grouping-lost-to-merge`.

### Any node can be split, so an area can span the areas beside it `##split-any-node`

`Split` applies to an area or to a split: a split along another axis than its parent's wraps the
node in a new split, so a new area can span the areas the node holds, as a title spans two
columns. This reaches every guillotine layout, a layout made by cuts that each run across the area
being cut. It does not reach a layout no such cut makes, such as four rectangles around a fifth,
nor rows aligned across two columns, which `issue@rootpoint-model@linked-dividers` keeps open.

The rival, splitting leaves only, makes a spanning area impossible once the areas beside it exist:
the author would have to rebuild the slide. It serves `goal@rootpoint@free-slide-division`.

## Coordinates

### Sizes are in slide units of a slide 1080 units high `##slide-units`

A document has one aspect ratio, 16:9 by default; a slide is 1080 slide units high and as wide as
the ratio makes it, 1920 for 16:9, so a font size of 48 means what it would on a 1080p screen.
`area_rects` gives each area's rectangle in slide units. No size is in pixels.

The rivals fail `goal@rootpoint@presenter-mode` with `goal@rootpoint@real-presentations`, which
need the slide shown in the editor to be the slide shown in presenter mode at any window size:
pixel units, `rejected@rootpoint-model@pixel-units`, wrap the same text differently on a 1080p
laptop and a 4K projector, and a free aspect ratio that reflows text,
`rejected@rootpoint-model@reflowing-slides`, shows another slide in each window. The file stores
these units, and the crate's API gives them.

### A divider's position is a fraction of the extent of its split `##divider-fractions`

Each divider is stored as a fraction of its split's extent, strictly between 0 and 1, not as an
absolute position on the slide. Moving a divider of a split then rescales every split nested in
the parts it resizes, with no change to their own dividers, so a nested layout keeps its
proportions; with absolute positions, one drag would rewrite every divider below it. The fractions
are part of every saved file.

## Content and assets

### An area holds its content, and heavy data is an asset the content refers to by id `##inline-content-assets`

An area holds one content: empty, a text with its style, or an image. Image data and font files
live once in the document's asset table, keyed by asset id, so a slide's text is read in the file
where it is shown, as `goal@rootpoint@readable-save-file` asks, and an image placed on many slides
is stored once.

The rival, every content stored apart and referenced by areas,
`rejected@rootpoint-model@content-by-reference`, costs garbage collection or a library of orphans,
editing that changes every slide sharing a content, a file read by following ids, and content ids
beside node ids; the owner weighed those costs above the reuse it gives. Reuse of text across
slides is left to templates, which `issue@rootpoint-model@slide-templates` keeps open.

### Images and other assets are files beside the presentation file `##image-storage`

The editing form of a presentation is its text file with its assets as files beside it, each named
by a path relative to the presentation file, so the text stays readable and the assets stay
ordinary files.

On the owner's word: the owner chose this form for prototyping, and, as the owner thinks, it stays
the preferred editing method in the long term, and packing would be the final goal, by commands that pack a
presentation and its assets into one file and unpack it again, so that one file can be carried and
presented from any computer with a web browser and an internet connection, through the web app
served from a static site. It serves `goal@rootpoint@readable-save-file` and
`goal@rootpoint@runs-in-browser`.

### An asset's source is a path or embedded bytes, both in the format's first version `##asset-source`

`AssetSource` is `Path`, a path relative to the presentation file, or `Embedded`, the asset's
bytes, which packing writes. Both are in the format from its first version, so the round-trip
tests cover embedded assets before any code writes one, and packing adds no change of format.
Adding the variant later would not have broken old files: the committed fixture,
`path@rootpoint-model@tests/data/fixture.ron`, names no embedded asset and loads with the enum
that has both. It would have left the format untested for embedded assets until packing.

### A text area has one style for all its text `##text-style-per-area`

A text's font, size, weight, slant, colour and alignment are set for the whole area. Styled spans
inside a text need an editor of rich text, the hardest part of a text editor to build, which this
design avoids; the owner judged one style per area enough for the first prototype. The style is part
of the file.

## Changes

### Every change of a document is an operation, applied by one function that returns its inverse `##edit-operation-core`

`apply` takes an `Op`, changes the document, and returns the operation that undoes it; a
structural edit of a slide's tree returns a snapshot of the slide's former tree. Code outside the
crate changes a document only through `apply`, which gives undo to every caller and keeps the
invariants. The document's fields are public for reading, so this rule is delivered by the doc
comment of `Document`, not enforced by the types.

The rival, a GUI changing the document directly, `rejected@rootpoint-model@gui-mutation`, makes a
text protocol re-implement every edit, so `goal@rootpoint-model@headless-changes` would cost two
implementations that drift apart. The operations are the crate's public interface for changing a
document.

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
A file edited by hand is a purpose, `goal@rootpoint-model@faithful-readable-file`, so a breach is
reachable.

### Ids are stored in the file, come from one counter, and are never reused `##stable-ids`

Every slide, node and asset has an id from one counter the document stores and only grows; ids are
written in the file and print as `s<n>`, `n<n>` and `a<n>`. A duplicated slide's nodes get new
ids. A command or a script can then name a node across edits, saves and loads, as
`goal@rootpoint@text-command-protocol` needs, and so could a link between dividers,
`issue@rootpoint-model@linked-dividers`. The rival, naming a node by its position in the tree,
`rejected@rootpoint-model@addressing-by-position`, shifts at every split or removal. Ids are part
of every saved file and of the crate's public types.

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
