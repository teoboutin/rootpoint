---
kind: deferred
---
# Rows cannot be aligned across two columns of a slide

## Summary

Dividers in different branches of a slide's tree cannot be linked, so rows of two columns line up
only by hand. The first prototype leaves linking out; T3 of its premortem, recorded on the
owner's word, watches for the need.

## Details

### What

A slide of two columns, each split into rows, has two independent sets of row dividers,
`design@rootpoint-model@split-tree-arity`. To keep the rows aligned, the author moves both
dividers to the same position. The owner proposed linking two dividers so each follows the other,
a constraint "divider X = divider Y" in slide coordinates, with a rule for what dragging one does
to the other. Every node carries a stable id, `design@rootpoint-model@stable-ids`, so a link can
name its dividers without a change of the file format.

### Why it matters

Aligned rows across columns are a layout `goal@rootpoint@free-slide-division` asks to reach
conveniently; aligning them by hand defeats it if real slides need it often.

### Trigger

A real slide the owner prepares with the prototype needs rows aligned across two columns. The
owner's work on that slide meets the need, and the response is a design session on linked
dividers, which this entry opens with the shape above.
