---
kind: defect
---
# A reader of a saved file infers what `Vertical` and `Horizontal` mean

## Summary

An agent editing a saved presentation by hand found the right areas, but only by assuming what
the `axis` names mean: the file does not say whether `Vertical` puts children side by side or
stacked.

## Details

### What

In the model's save format, a split carries `axis: Vertical` or `axis: Horizontal`. In the code,
`Vertical` has vertical dividers and lays its children side by side, left to right, as the doc
comment of `Axis` in `path@rootpoint-model@src/document.rs` says. Asked to change "the top-right
area" of `path@rootpoint-model@tests/data/fixture.ron`, given only that file, a fresh agent made
the right edit and reported that it rested on two assumptions: that `Vertical` means a vertical
divider line, and that the first child is the left or top one. It ruled out the other reading only
because that reading left no top-right text to change. The edited file is
`path@rootpoint-model@tests/data/fixture-edited-by-agent.ron`.

The observation was made first on one file with one split per axis. The experiment below then
reproduced a misreading.

### Reproduction

A slide whose root is a `Vertical` split of two `Horizontal` splits, each of two text areas, in
the order the model writes them: Kiwi and Plum in the first child, Fig and Lime in the second. In
the code, the first child is the left column, so the bottom-left area is Plum. A fresh agent, given
only the saved file and asked to change the bottom-left area's text, changed Fig: it read
`axis` as the direction the children are stacked in, and named the other reading, a vertical
divider line as with vim's `:vsplit`, as equally common. The file was generated with the model's
own `save_ron`; nothing else was given to the agent.

### Why it matters

`goal@rootpoint@readable-save-file` asks that a person or an agent can read and edit the file. The
names are stated by `design@rootpoint-model@split-tree-arity` and written by
`design@rootpoint-model@save-format`, both of which a renaming changes. A
name whose meaning the reader must guess can lead to an edit of the wrong area, and the
protocol's planned `split <node> <h|v>` of `spec@first-prototype@command-protocol`, which serves `goal@rootpoint@text-command-protocol`,
carries the same ambiguity.

### What would close it

An experiment: give a fresh agent a saved file where both readings fit, such as a slide of four
areas in a two-by-two layout, and ask for an edit by position. A wrong edit makes this a defect,
answered by a design session on the names (such as `Columns` and `Rows`, or a comment the file
writes beside the axis); a right one, with no assumption reported, closes it.
