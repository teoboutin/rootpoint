---
kind: todo
---
# The sample presentation does not teach the editing of a formula in the window

## Summary

The sample's slide on equations says that the protocol's `equation` command sets a formula, and says
nothing of the window's editing of it, its keys or its palette, which the window now has.

## Details

### What

The sample, `design@rootpoint-protocol@builtin-sample`, holds a slide "Equations", slide s33 in
`path@rootpoint-protocol@sample/`: an equation area and the text "An equation area draws a formula
written in LaTeX, numbered at its right edge in the order of the slides. Its options are in the
side panel; the protocol's equation command sets its formula." Since the window edits a formula in
place, `design@rootpoint-app@math-editing-model`, with LyX's keys, `design@rootpoint-app@math-keys`,
and a palette in the Selection tab, `design@rootpoint-app@math-symbol-palette`, the slide names the
protocol alone where the window does the same, and no slide shows a formula inside a text.

Leads, none decided: the slide's text names a second click on the equation to edit it, Alt+M F for a
fraction, `\` for a command and the palette; a text of the slide, or of another, holds an inline
formula and a reference to the equation. A change of slide s33 changes the golden image
`path@rootpoint-app@tests/data/sample-s33.png`, which `ROOTPOINT_BLESS=1` rewrites from a native
run, as `path@rootpoint-app@tests/web.rs` says.

### Why it matters

The sample is the tutorial of every build, on the owner's word recorded in
`design@rootpoint-protocol@builtin-sample`; a reader of it learns that a formula is set by a command
of the protocol, and not that the window edits it, nor its keys, so the editing the milestone on math
built for an author with no LaTeX to know, `goal@rootpoint@no-language-no-build`, goes unseen.

### What would close it

The sample's slides teach the editing of a formula in the window and show a formula inside a text,
the golden image of slide s33 rewritten.
