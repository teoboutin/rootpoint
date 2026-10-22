---
kind: todo
---
# Text is styled per area, with no style per span

## Summary

A text area has one style, `design@rootpoint-model@text-style-per-area`. Bold, colour or size
cannot change inside a text.

## Details

### What

Rich text replaces `TextStyle` per area with styled spans, in the model, the renderer and the
in-place editor, `design@rootpoint-app@edit-in-place`, which takes one style per area as given.

### Why it matters

`goal@rootpoint@minimal-content-set` names bold and colour among the options of text; per span they
are what a talk's text often needs, a word in bold or a term in colour.

### What would close it

A design session on rich text, held before any work builds on the in-place editor for it, and its
work built.
