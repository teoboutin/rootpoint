# Rejected designs of rootpoint-model

The designs not kept for rootpoint-model, and why: one level-three heading per design, each cited
by the design head whose argument covers it, in `register@rootpoint-model@design`. An entry
describes a design that is not built.

### Binary splits, each with exactly two children `##binary-splits`

`live`. One layout has several binary trees, by the order its splits were made in: three equal
columns are a third and a half-of-the-rest, or two thirds and a half, and dragging the first
divider of the first tree shrinks both other columns. Removals and drags then behave differently on
layouts that look the same, and the file shows a structure the author never chose. Kept open by the
doubt that merging same-axis splits removes a grouping an author wants, which
`tripwire@rootpoint-model@grouping-lost-to-merge` watches.

### Content stored apart from the areas, each area holding a reference `##content-by-reference`

`live`. Removing an area leaves its content orphaned, which needs garbage collection or a library
to manage; editing a shared content changes every slide that shows it, which the GUI must mark and
offer to undo by copying; reading a slide in the file means following ids; the protocol needs
content ids beside node ids. The owner, who proposed it, withdrew it as more trouble, for building
and for using the editor, than convenience.

### Sizes in pixels `##pixel-units`

`live`. The same slide wraps its text differently at different window sizes, so a talk prepared on
a 1080p laptop and shown on a 4K projector shows another slide.

### A free aspect ratio, with text reflowed to each window `##reflowing-slides`

`live`. The slide shown in presenter mode differs from the slide edited, by the shape of the window.

### The GUI changing a document directly, without operations `##gui-mutation`

`live`. The text protocol would then implement every edit a second time, and the two
implementations would drift apart; undo would need a third.

### JSON as the save format `##json-save-format`

`live`, held weakly. JSON is universal and slightly better for tools other than rootpoint, but it
writes an enum variant as an object keyed by its name and allows no comment, which makes editing
the file by hand harder.

### Trusting loaded files and operation results without validation `##unvalidated-documents`

`live`. Counterexamples built at review showed it: a hand-edited file with a missing divider loaded,
and `area_rects` and `Remove` then panicked; a lowered id counter loaded, and the next operation
reused ids; splits and merges at extreme positions made dividers equal, or equal to 0 or 1. They
are kept as tests in `path@rootpoint-model@tests/invalid.rs`.

### Refusing every breach of a document, and relying on tests to keep the editor from making one `##every-breach-refused`

`live`. A file edited by hand, which `goal@rootpoint-model@faithful-readable-file` makes a purpose,
is reached by no test of the editor, so a reference to a missing asset or font in it would make
the file impossible to open, although the renderer draws a stand-in for each. Tests show that the
editor makes no breach; they cannot repair a file the editor did not make.

### Naming a style by an id from the document's counter `##style-ids`

`live`. A rename would then touch nothing but the table, and the name would change no reference.
The file would read `Named(21)` where a name reads in place, and a command would name a style by an
id a reader looks up; the owner chose names, judging that referring by name is important. It would
change the save format and every command that names a style.

### Naming a node by its position in the tree `##addressing-by-position`

`live`. A position, such as the second child of the first child of a slide's root, shifts at every
split or removal, so a command written after one edit names another node after the next; a span's
offset in its text shifts likewise at every edit before it.

### Naming a split's axis by its dividers' direction, `Vertical` and `Horizontal` `##divider-direction-names`

`live`. Both readings are common: the direction the children are stacked in, or the direction of
the divider line, as with vim's `:vsplit`. A fresh agent given only a saved 2×2 slide and asked to
change its bottom-left area changed the area the other reading puts there.

### A migration of each earlier version of the save format `##format-migration`

`live`. A loader of each earlier version costs code and a test per version, and no presentation
exists to read: the owner gives no real presentation with the project while it is in initial
development, and keeps no compatibility until the owner rules that real use has begun.

### A kind of content for an area with nothing in it `##empty-content-kind`

`live`. An empty text draws nothing, as this kind does: the renderer's arm for it draws nothing,
and is the only place the drawing tells the two apart. The kind costs a variant of the save format,
a reply of `tree` and a button of the side panel, and an area holding it is made a text before the
author can type into it.

### Asset bytes held in the presentation's file `##bytes-in-the-file`

`live`. RON writes a `Vec<u8>` as one decimal number per line: one asset of 580 bytes, the sample's
`splits.png`, added 11,768 bytes to the file, 20.3 bytes of text per byte, measured by setting the
fixture's first asset to those bytes and comparing the lengths of `save_ron`. A photo of 5 MB would
be about 100 MB of text, and the file would no longer be a text a person reads or a diff shows.

### Content that names an asset by its path `##asset-named-by-path`

`live`. A move or a rename of an asset's file would rewrite every image and every style that names
it, each one an operation, where an id makes it one operation on the asset alone. The file's
grammar would change with it.

### Folders stored in the presentation's file `##folders-in-the-file`

`live`. A folder node holds no content: it is structure the paths already carry, which every move
of a file would have to keep in step, and the file's grammar would gain a kind of entry.

### A file whose asset path is not confined, refused at load `##unconfined-path-refused-at-load`

`live`. The store must refuse such a path at each access anyway, since a symlink out of the root is
the disk's state and is met only at a read, so a refusal at load is a second mechanism for the paths
the file names. It would refuse a file whose layout and node ids are defined, which the structural
class of `design@rootpoint-model@validated-documents` does not cover; and a repair at load, copying
the outside file in, would read outside the root.

### One style for the whole of an area's text `##one-style-per-area`

`live`. No word of a text can be bold or coloured apart from the rest, which the owner asked for
when asking for rich text. It was the first prototype's design, which avoided an editor of rich
text, the hardest part of a text editor to build.

### Flat runs of styles over one string `##flat-style-runs`

`live`. A style's edge has one caret position, so the style of what is typed there is chosen by a
rule, as LyX's `Cursor::setCurrentFont` does for its flat fonts, read in
`path@rootpoint@docs/grounding/lyx-nested-text.md`; and a list of runs cannot hold a fraction's two
cells, which maths needs.

### Flat runs of styles beside nested spans, as LyX keeps both `##runs-beside-spans`

`live`. Two styling models meet at a seam: LyX's tickets 1973 and 6919 are a change of font that
did not reach the text inside an inset, read in `path@rootpoint@docs/grounding/lyx-nested-text.md`.

### A full text style in every span `##full-style-per-span`

`live`. A span holding every value copies its area's, so a bold word in a Body text keeps the old
size when Body's size changes later, which is the change named styles exist to spread.

### A span naming a full text style of the text style table `##span-names-text-style`

`live`. The span takes every field of the style, its font, size and alignment included, so a term
named inside a title leaves the title's size, and alignment means nothing inside a line.

### One table of styles whose fields are all optional, for areas and spans alike `##one-table-of-optional-styles`

`live`. Every area style becomes a set of optional fields with a fallback for each, the cascade
`design@rootpoint-model@named-styles` keeps out of the area. Kept open by the doubt that two tables
confuse authors.

### A text's items untagged, a bare string beside a span, in the file `##untagged-text-items`

`live`. ron 0.12.2 writes an untagged span with no name, and a hand-written `Span(...)` then fails
to load, "data did not match any variant of untagged enum", as measured in the design audit of the
slice that built the text tree, recorded in its commit "Record the design audit of the slice of
the text tree".
