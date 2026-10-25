---
kind: todo
---
# Bullet lists, code snippets, equations and video are not content kinds yet

## Summary

An area holds text or an image. The other kinds `goal@rootpoint@minimal-content-set` lists, bullet
lists, code snippets, equations and video, do not exist.

## Details

### What

Each kind adds a variant of `Content` in `path@rootpoint-model@src/document.rs`, its options, its
drawing in the render crate and its editor in the app. For equations, the survey of the first
prototype said, unchecked, that Rust has no small, mature TeX-math renderer; the owner asked how
hard a minimal one would be, and left the question for later.

The owner is considering maths in two forms: inline maths inside a text, and an equation kind of
its own that holds one full equation and is numbered and referenceable, which inline maths is not;
the owner gives this split of uses as an intuition only, expected to be simpler for authors and for
the implementation. A text's items are kinds of an enumeration that admits a maths kind and builds
none, `design@rootpoint-model@text-node-kinds`; a reference to an equation, written inside a text,
is another kind it admits.

Leads for inline maths, none decided, from the milestone on rich text:

- the renderer lays a text out with Parley 0.12.0's `RangedBuilder`,
  `design@rootpoint-render@area-text-layout`, whose `push_inline_box` places an `InlineBox`, of
  the `width`, `height` and `baseline` given, at the byte index `index` the caller gives, read in
  Parley's `path@elsewhere@src/builder.rs` and `path@elsewhere@src/inline_box.rs`; a maths item
  laid out by the project could stand in the text's one layout as such a box;
- the built-in Noto Sans has no `MATH` table: the table tags of each of its four files,
  `path@rootpoint-render@fonts/NotoSans-Regular.ttf` and its bold, italic and bold italic, read by a
  script, are DSIG, GDEF, GPOS, GSUB, OS/2, cmap, cvt, fpgm, gasp, glyf, head, hhea, hmtx, loca,
  maxp, name, post and prep; maths would need a font of its own.

### Why it matters

`goal@rootpoint@real-presentations` asks for talks that need mathematics, which the prototype draws
only as images prepared elsewhere, the owner's own fallback; bullet lists and code snippets are
among the kinds the owner named as needed.

### What would close it

A design session per kind, or one for several, and its work built. Each new kind is read against
`tripwire@rootpoint-render@render-within-bounds` at its design audit, and video, whose files the
browser build holds in memory, against `tripwire@rootpoint-app@browser-pack-too-large`, and decides whether its
options earn named styles, which the style tables hold for text alone, its areas and its spans,
`design@rootpoint-model@style-kinds`.
