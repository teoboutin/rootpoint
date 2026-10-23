---
kind: question
---
# The new area's style is built with two conditions its head does not state, awaiting the owner

## Summary

`design@rootpoint-model@new-area-style` states the rule the owner approved: a split gives its new
area the split area's style, and every other new area takes Body. The code adds two conditions, so
that no new area holds a breach, which narrow that rule and await the owner's word as D3 and D5 of
`milestone@plans@editor-conveniences`. Until the owner rules, the head and the code differ wherever one of
those two conditions applies.

## Details

### What

- `Document::new_area_style` of `path@rootpoint-model@src/document.rs` gives Body only while the
  table holds it and it draws with no stand-in, `Document::draws_as_asked`, its font an asset of
  the table or a family of `BUILTIN_FAMILIES`; otherwise the default values. D3.
- `split_style` of `path@rootpoint-model@src/ops.rs` does not copy a style name the table lacks; the
  new area takes the style `Document::new_area_style` gives. D5.

Three cases reach them:

- a file edited by hand whose Body names a font the file lacks: with the head's rule as written,
  `add-slide` makes a breach in a document that has none, against
  `design@rootpoint-protocol@no-new-breach`, which is why the default builds D3;
- a command that leaves the table without Body, `style remove "Body" into="Title"`: with the head's
  rule as written, `add-slide` then gives its area the name Body, which the table lacks, a breach a
  command makes in a document that has none; D3 again;
- a file edited by hand whose text names a style the file lacks: with the head's rule as written, a
  split of that text copies the breach the file brought; D5.

### Why it matters

A design home is built intent: a session that checks the code against
`design@rootpoint-model@new-area-style` meets two divergences, and must not close them by following
the code. They close by the owner's rulings.

### What would close it

The owner's ruling on D3 and on D5. A condition the owner keeps is written into the head; one the
owner declines leaves the code, which then follows the head.
