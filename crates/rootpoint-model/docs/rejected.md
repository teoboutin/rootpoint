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

### Naming a node by its position in the tree `##addressing-by-position`

`live`. A position, such as the second child of the first child of a slide's root, shifts at every
split or removal, so a command written after one edit names another node after the next.

### Naming a split's axis by its dividers' direction, `Vertical` and `Horizontal` `##divider-direction-names`

`live`. Both readings are common: the direction the children are stacked in, or the direction of
the divider line, as with vim's `:vsplit`. A fresh agent given only a saved 2×2 slide and asked to
change its bottom-left area changed the area the other reading puts there.
