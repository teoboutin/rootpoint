# Design of rootpoint-model

How rootpoint-model is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-model is for is in `register@rootpoint-model@goal`.

## The tree of a slide

### A split divides its extent along one axis into any number of children, and never holds a child split of its own axis `##split-tree-arity`

A slide's root is a node: an area, or a split with an axis and k ≥ 2 children separated by k−1
dividers, its axis named by `design@rootpoint-model@axis-names`. A split along the axis of the split that holds it is merged into it,
and so is a child left alone by a removal when its axis is its new parent's, so each layout has
exactly one tree.

The nearest rival, binary splits, `rejected@rootpoint-model@binary-splits`, gives one layout
several trees, by the order the splits were made in: removing an area or dragging a divider then
behaves differently on layouts that look the same, and the file shows a structure the author never
chose. With the merge, dragging a divider moves only its two neighbours. The tree is the structure
of every saved file and of the crate's public types. It serves `goal@rootpoint@free-slide-division`.
Whether merging removes a grouping an author wants is watched by
`tripwire@rootpoint-model@grouping-lost-to-merge`.

### A split's axis is named by where its children go `##axis-names`

A `LeftToRight` split lays its children side by side, left to right, with vertical dividers; a
`TopToBottom` split stacks them, top to bottom, with horizontal dividers. The names are written in
every saved file, which a person or an agent reads and edits by hand, per
`goal@rootpoint@readable-save-file`, and they are part of the crate's public types. The names of
the dividers' direction, `Vertical` and `Horizontal`,
`rejected@rootpoint-model@divider-direction-names`, were read both ways by a fresh agent. Among
names that say where the children go, the owner chose `LeftToRight` and `TopToBottom` over
`SideBySide` and `Stacked`, as reading better in the owner's opinion.

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

An area holds one content: a text, its tree of characters and spans with its style,
`design@rootpoint-model@text-node-tree`, or an image, `design@rootpoint-model@no-empty-kind`. Image data and font files live once in the document's asset
table, keyed by asset id, so a slide's text is read in the file where it is shown, as
`goal@rootpoint@readable-save-file` asks, and an image placed on many slides is stored once. A
text's style may name an entry of the document's style table, `design@rootpoint-model@named-styles`,
and a span's an entry of its span style table, `design@rootpoint-model@named-span-styles`: the parts
of a content shared by name.

The rival, every content stored apart and referenced by areas,
`rejected@rootpoint-model@content-by-reference`, costs garbage collection or a library of orphans,
editing that changes every slide sharing a content, a file read by following ids, and content ids
beside node ids; the owner weighed those costs above the reuse it gives. A shared style answers
each: a style no text names is an entry of a library, not an orphan; changing every text that names
it is its purpose, done only by `style set`; the file reads the name in place; and the name is no
id. Reuse of text across slides is left to templates, which `issue@rootpoint-model@slide-templates`
keeps open.

### An area with nothing in it holds an empty text, and no kind of content of its own `##no-empty-kind`

A text may hold no characters, and a new area, of `Op::AddSlide` or of a split, holds one, in the
style `design@rootpoint-model@new-area-style` gives. An empty text draws nothing, so a kind for an
area with nothing in it adds nothing an empty text lacks: in what the renderer draws, it differs
from an empty text only by an arm that draws nothing, `rejected@rootpoint-model@empty-content-kind`.
Outside the renderer, the editor has one kind to enter for typing, the text, per
`design@rootpoint-app@edit-entry`. On the owner's word: the owner proposed that no kind stand for an
area with nothing in it, a kind that in the owner's view is noise in both the GUI and the
implementation.

### A workspace is a directory with `presentation.ron` at its root, its assets as files below it, and `.rootpoint/` for the machine `##workspace-layout`

A workspace, the store of `design@rootpoint-protocol@editing-modes` on disk, is a directory holding
`presentation.ron` at its root and each asset as a file at its path below it, so the text stays
readable and the assets stay ordinary files; `.rootpoint/` holds what is the machine's and not the
presentation's, a `.gitignore` that ignores all of it, the lock of
`design@rootpoint-protocol@workspace-lock`, the pointer to the window's socket and the trash of
`design@rootpoint-protocol@asset-trash`. Neither
`presentation.ron` nor `.rootpoint/`, nor a pack's `mimetype`, is an asset path,
`design@rootpoint-model@asset-path-confinement`, and `.rootpoint/` is never packed. The fixed name
lets a script or an agent find the presentation without listing the directory, where a directory of
several `.ron` files is ambiguous; its cost is one presentation per directory, so two talks that
share an image each keep a copy.

On the owner's word: the owner proposed the text file of a fixed name at the workspace's root and
the `.rootpoint/` beside it, and judged two talks that share images a rare case, in the owner's
opinion; the owner chose this form for prototyping, and, as the owner thinks, it stays the
preferred editing method in the long term, and packing would be the final goal, by commands that
pack a presentation and its assets into one file and unpack it again, so that one file can be
carried and presented from any computer with a web browser and an internet connection, through the
web app served from a static site, `design@rootpoint-protocol@pack-container`. It serves
`goal@rootpoint@readable-save-file` and `goal@rootpoint@runs-in-browser`.

### An asset is the path of its file, never its bytes in the presentation's file `##asset-paths-only`

`Asset { path }`: the path of the asset's file, `/`-separated, relative to a workspace's root, or a
key of the map of a store in memory, `design@rootpoint-protocol@editing-modes`. The presentation's
file is text only, as `goal@rootpoint@readable-save-file` and
`goal@rootpoint-model@faithful-readable-file` ask: bytes held in it, the rival
`rejected@rootpoint-model@bytes-in-the-file`, grow it about 20 times their size.

### A folder of assets is a prefix of their paths, and the file stores no folder `##folders-from-paths`

The asset table holds each asset's path, and nothing else names a folder: a folder is a prefix of
the paths, and a folder with no asset is no folder. The rival, folder nodes
stored in the presentation's file, `rejected@rootpoint-model@folders-in-the-file`, stores structure
that holds no content, which a move of a file must then keep in step with the paths.

### An asset path is relative, below its root, outside `.rootpoint/`, and never a file of the store `##asset-path-confinement`

`asset_path_is_confined` holds for a path with no empty, `.` or `..` component, no leading `/` and
no `\`, whose first component is not `.rootpoint`, `presentation.ron` or `mimetype`, the files of a
workspace and of a pack that are no asset: an asset named `mimetype` would give a pack two entries
of that name, `design@rootpoint-protocol@pack-container`, and one below a folder `mimetype/` or
`presentation.ron/` would be a file and a folder of one name, which no directory holds and
`open-pack` refuses. It is the test of the confinement the store keeps at each access,
`design@rootpoint-protocol@store-confines-access`, and of the commands that refuse to name a path
that fails it, `design@rootpoint-protocol@no-new-breach`. The model checks nothing more: whether a
file is at the path, and where a symlink leads, are the store's to check. A file that names another
path loads, a cosmetic breach, `design@rootpoint-model@cosmetic-fallbacks`.

### An area's text is a tree of characters and spans, each span with its own values over its parent's `##text-node-tree`

A text is a root `TextNode`, a sequence of runs of characters and spans; a span has an id, a span's
style and a `TextNode` of its own. The root draws in the area's style,
`design@rootpoint-model@named-styles`, and each span in the values it sets over its parent's,
`design@rootpoint-model@span-style-overrides`. An offset in a text node counts characters, Unicode
scalar values, and a span counts as one item, so that a span's edges are offsets of their own: the
position before a span and the position at its start are two, and typing at each goes outside or
inside the span. The tree is part of the file, `design@rootpoint-model@save-format`, and of the
document's public type.

The nearest rival, flat runs of styles over one string, `rejected@rootpoint-model@flat-style-runs`,
gives a style's edge one position, so the style typed there is guessed, and it cannot hold a
fraction's two cells, which maths will need. Both at once, as LyX keeps them,
`rejected@rootpoint-model@runs-beside-spans`, puts a seam between two styling models. One style
for a whole area, `rejected@rootpoint-model@one-style-per-area`, styles no word apart from its
text. Parley lays the tree out in one layout,
`design@rootpoint-render@area-text-layout`, which `goal@rootpoint-render@same-slide-everywhere`
asks of every display. On the owner's word: the owner proposed the text as a tree, a span's edge
being a position of its own, as LyX's editor moves through its insets.

### No span is dissolved, a custom span that sets nothing is refused, and runs of characters are kept merged `##text-tree-normal-form`

`apply` and `load_ron` keep each text node's runs of characters merged, none empty and no two
adjacent, so that a document and its round trip through an operation and its inverse compare equal.
No operation dissolves a span: a named span is kept whatever its values, and a span that sets a
value equal to its parent's is kept, so a bold word inside a bold span stays bold when the outer
span changes. An operation that would make a custom span that sets no field, by a wrap, a change of
style, a whole tree or a removal of a span style into a copy, is refused, and its caller unwraps
the span instead. A file that holds such a span loads, drawn with its parent's values. An empty span
is accepted, so that a style chosen with nothing selected opens one at the caret. Adjacent spans of
one style stay apart: they are what the author made, and either can be restyled alone, the
grouping `tripwire@rootpoint-model@grouping-lost-to-merge` watches in a slide's tree. The rival,
dissolving every span whose values change nothing of its parent's, dissolves a named span whose
style is missing or equal to its parent's, which loses the name a later change of the style would
reach, and a bold word inside a bold span, which loses its bold when the outer span changes.

### A span sets each text option it changes, each optional, and the area keeps one full style `##span-style-overrides`

A span's `SpanStyle` holds `font`, `size`, `bold`, `italic` and `colour`, each an `Option`; a span
draws with its own value where it sets one and its parent's elsewhere, the root's being the area's
style, `Document::resolve_span`. Alignment is the area's alone. A span sets several values at
once, so one span changes bold and colour together. The rival, a full style per span,
`rejected@rootpoint-model@full-style-per-span`, copies the area's values into the span, which keep
their old value when the area's style changes later. It serves
`goal@rootpoint@minimal-content-set`, a span offering the options of text and no others. The file
writes only the values a span sets.

### A span names a style of a table of span styles, each of whose fields is optional `##named-span-styles`

`Document::span_styles` is a table of span styles, by name, separate from the text styles; a span's
style is `Named(name)`, a style of that table, or `Custom(values)`, its own. A new document holds
none. The rules of the text styles hold for it: a rename rewrites every span naming the style,
`design@rootpoint-model@style-key`; a removal gives its users another span style or a copy of the
removed values each, or leaves them naming it, `design@rootpoint-model@style-removal`; a span naming
a style the table lacks draws with its parent's values, `design@rootpoint-model@cosmetic-fallbacks`.
A span style changes only what it sets, so a term in a title stays at the title's size. The rivals:
a span naming a full text style, `rejected@rootpoint-model@span-names-text-style`, sets every field
of it, alignment included; one table of styles whose fields are all optional, for areas and spans,
`rejected@rootpoint-model@one-table-of-optional-styles`, makes every area style a cascade with a
fallback, which `design@rootpoint-model@named-styles` keeps out of the area. The table is part of
the file and of the document's public type. On the owner's word: the owner asked that a span can
name a style.

### A text's items are kinds of an enumeration, which leaves room for maths beside the span `##text-node-kinds`

`TextItem` is `Chars` or `Span`, and the file, the operations and a caret position name an item by
its kind, so a kind added later, maths, or a reference to an equation, joins the enumeration with
no change of what a text is. No kind but these two exists, and an equation's design is
`issue@rootpoint@more-content-kinds`'s. It serves `goal@rootpoint@real-presentations`, whose talks
need maths, written inside a sentence.

### A text names a style of the document's table, or holds its own values `##named-styles`

A document holds a table of named text styles, and a text's style is `Named(name)`, a style of the
table, or `Custom(values)`, its own. A new document holds four, Title, Subtitle, Body and Legend.
Editing a named style changes every text that names it, which is its purpose: the nearest rival, a
preset whose values are copied into the text when it is picked, changes nothing placed when the
preset changes later. The other rival, overrides per field over a named style, makes every field
of an area's style optional with a fallback for each; a custom style covers the text that differs,
and a part of a text that differs is a span, whose values are overrides over the area's,
`design@rootpoint-model@span-style-overrides`. A value given to a named text alone makes it custom, with the named values and that change,
so that only `style set` changes a shared style. The table is part of the file and of the
document's public type. On the owner's word: the owner proposed predefined styles for titles,
subtitles and legends, with an editor of them, each text choosing one or a custom style of its own.

### Styles are named by their name, in the file and in commands `##style-key`

A text names a style by its name, `Named("Title")` in the file and `style="Title"` in a command,
and a rename is one operation that rewrites every text naming the style, its inverse renaming them
back. The name reads in place in the file, where an id would send the reader to the table, as
`goal@rootpoint@readable-save-file` asks of the file. The rival, an id from the document's counter,
with which a rename touches nothing else, `rejected@rootpoint-model@style-ids`, meets readability
too, so the choice was a weighing. On the owner's word: the owner chose names over ids, judging
that referring by name is important.

### Named styles are for text only, until another kind of content needs its own `##style-kinds`

The style tables hold text styles, for areas, and span styles, for spans of a text,
`design@rootpoint-model@named-span-styles`; an image's one option, how it fits its area, is chosen
per area with no name. A kind of content added later, `issue@rootpoint@more-content-kinds`, decides then
whether its options earn named styles. The rival, a style object per kind from the start, splits
styles by kind before any kind but text has options worth sharing: an image has one option, so a
named image style would save one click.

### Removing a style that texts name takes a replacement for them `##style-removal`

`Op::RemoveStyle` gives the texts naming the removed style another style, which the command requires
to be one a text may name, `design@rootpoint-protocol@no-new-breach`, or each a custom copy of the
removed values, which changes no slide; with no replacement they keep naming the removed style, a
cosmetic breach, `design@rootpoint-model@cosmetic-fallbacks`, so that the inverse of `Op::AddStyle`
removes a style that texts loaded from a file name. The command refuses a removal with no
replacement while a text names the style, and names those texts,
`design@rootpoint-protocol@no-new-breach`. Its inverse puts the style back and makes its former
users name it again. The rival, a removal that leaves its users drawn with the default values,
leaves a warning to clear after a deliberate act. On the owner's word: the owner suggested, as a
"maybe", that the author name the style that replaces a removed one; the custom copy is the
session's, the one replacement that changes no slide.

### A new area takes the style of the text area it was split from, or Body `##new-area-style`

A split of a text area gives its new area that area's style; every other new area, of a split of a
split or of an image, of `Op::AddSlide`, or of an image area emptied or given a text with no style,
takes Body. Splitting a body text for a second column gives a body text with no further step, which
`goal@rootpoint-app@direct-manipulation` asks of common tasks. The rival, a new area that always
takes Body, costs a style change after a split of a styled text; whether inheriting gets in the way
instead is watched by `tripwire@rootpoint-model@inherited-style-in-the-way`.

Body is taken only while the table holds it and it draws with no stand-in, its font an asset of the
table or a family of `BUILTIN_FAMILIES`, and otherwise the default values: Body taken always would
let `add-slide` make a breach in a document that has none, after `style remove "Body"` or from a
file edited by hand, which `design@rootpoint-protocol@no-new-breach` refuses. A split does not copy
a style name the table lacks, its new area taking the style any other new area takes, so that no
split spreads a breach a file brought, where `duplicate-slide` copies one. On the owner's word: the
owner chose that a split does not copy the missing name, over copying it as `duplicate-slide` does.

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

### A document is validated when it is loaded and after every operation, and a structural breach is refused `##validated-documents`

`Document::validate` checks every structural invariant: unique ids below the counter, a span's
included, every asset an area, a span or a style of a table names below the counter too, splits of two children or more with
ordered dividers strictly inside (0, 1) and no child split of their own axis, no zero side in the
aspect ratio. `load_ron` refuses a file that breaks one, and `apply` refuses an operation whose
result breaks one, leaving the document unchanged. A breach is structural when, with it, the layout
or the identity of the nodes is undefined: no area rectangle can be computed, an operation could
panic or reuse an id, or a repair would have to guess the author's layout. An asset id at or above
the counter is one, since a later `Op::AddAsset` would give it to a new asset, which the area would
then show. What is not structural is `design@rootpoint-model@cosmetic-fallbacks`.

The rival, trusting a loaded file and the result of every operation,
`rejected@rootpoint-model@unvalidated-documents`, was refuted by counterexamples built at review: a
hand-edited file with a missing divider made `area_rects` and `apply` panic, a lowered id counter
made a later operation reuse ids, and rounding in a split or a merge made dividers equal, 0 or 1.
Each is structural. A file edited by hand is a purpose,
`goal@rootpoint-model@faithful-readable-file`, so a breach is reachable.

### A cosmetic breach of a document is accepted, drawn with a stand-in, and named `##cosmetic-fallbacks`

A breach is cosmetic when it is a reference to a missing target that has a defined stand-in, whose
effect stays inside the drawing of each area that names it: an image or a text font naming an asset
the table lacks, drawn grey or in the built-in sans, a `FontRef::Builtin` naming a family outside
`BUILTIN_FAMILIES`, drawn in the built-in sans, a text naming a style the table lacks, drawn
with `TextStyle::default()`, and a span naming a span style the table lacks, drawn with its
parent's values; a span's font counts as a text's; or an asset of the table whose path fails `asset_path_is_confined`,
which no store reads, so that every area showing it draws the stand-in of a missing asset.
`Document::breaches` names each, as `Breach`, and the renderer's report, the window's status line
and the reply of `render` name what was drawn with a stand-in. `load_ron` loads such a file, and
`apply` accepts an operation whose result holds a breach, so the inverse of an edit of such an area,
which brings back what a file was loaded with, is never refused. Two operations refuse what no undo
meets. `Op::RemoveAsset` refuses an asset an area, a span or a style of a table uses: it refuses a
removal, not a reference, and the inverse of `Op::AddAsset` removes an id the counter has just
given, which no area can name. `Op::RenameStyle` refuses a rename while a text names the new name, a
style the table lacks, since its inverse would rename that text back with the others; after a rename
from a name, no text names it; `Op::RenameSpanStyle` refuses one while a span does, for the same
reason. No command creates a breach in a document that has none,
`design@rootpoint-protocol@no-new-breach`. Which files load is part of the save format.

A file whose asset path fails `asset_path_is_confined` loads too. The store refuses each access to such a
path, `design@rootpoint-protocol@store-confines-access`, and that check is needed whatever the file
names, since a symlink out of the root is the disk's state, met only at a read; the file's layout
and the identity of its nodes are defined. The rival, a refusal at load,
`rejected@rootpoint-model@unconfined-path-refused-at-load`, adds a second mechanism and refuses a
file the structural class does not cover.

The renderer draws each stand-in already, an image it was not given grey, which
`Report::missing_images` names, and a font it does not have in the built-in sans,
`design@rootpoint-render@font-sources`, so refusing such a file refuses one the renderer could
draw, and a file edited by hand is a purpose,
`goal@rootpoint-model@faithful-readable-file`. The rival, refusing every breach and relying on tests
to keep the editor from making one, `rejected@rootpoint-model@every-breach-refused`, leaves a file
edited by hand with no way to open it. On the owner's word: the owner set the principle that a
document's inconsistencies are told apart as structural or cosmetic, that a cosmetic one falls back
wherever a fallback can be used, in the owner's opinion, and that a file whose structural change
makes it impossible to render properly, or not obvious for a user to recover from, may be refused.
The test above, which tells the two apart, stands on its argument.

### Ids are stored in the file, come from one counter, and are never reused `##stable-ids`

Every slide, node, asset and span has an id from one counter the document stores and only grows;
ids are written in the file and print as `s<n>`, `n<n>`, `a<n>` and `t<n>`. A duplicated slide's
nodes and spans get new ids, and so does every span of a tree an operation sets whole. A command or a script can then name a node across edits, saves and loads, as
`goal@rootpoint@text-command-protocol` needs, and so could a link between dividers,
`issue@rootpoint-model@linked-dividers`. The rival, naming a node by its position in the tree,
`rejected@rootpoint-model@addressing-by-position`, shifts at every split or removal. An asset keeps
its id when its file moves or its bytes change, so content names it by its id and a move is one
`Op::SetAssetPath` that changes no slide; content naming an asset by its path,
`rejected@rootpoint-model@asset-named-by-path`, would rewrite every image and style that shows it
at each move. Ids are part of every saved file and of the crate's public types.

## The file

### A presentation is saved as RON text, with the version of its format first `##save-format`

`save_ron` writes the document as RON, its first field the format version, and `load_ron` refuses
another version. Ids are plain numbers, and the file enables ron's `unwrap_variant_newtypes`, so a
node reads `Split(id: 4, ...)`, and `implicit_some`, so a value a span sets reads `bold: true`. A
text's item is tagged, `Chars("...")` or `Span(id: ..., style: ..., text: [...])`: an untagged
item, a bare string beside a span, writes a span with no name, and a hand-written `Span(...)` then
fails to load, as measured with ron 0.12.2, which a file edited by hand,
`goal@rootpoint-model@faithful-readable-file`, rules out.

RON writes enum variants as they read and allows comments and trailing commas, which helps editing
by hand; the rival, JSON, `rejected@rootpoint-model@json-save-format`, is universal and slightly
better for tools other than rootpoint. The choice is held weakly. The version comes first so a
file of another version is refused, not misread; no migration reads an older version while the
project keeps no compatibility, `design@rootpoint@no-compat-in-initial-dev`, and a migration of each
earlier version is `rejected@rootpoint-model@format-migration`. It serves
`goal@rootpoint@readable-save-file`, and the file is an interface every saved presentation
consumes.
