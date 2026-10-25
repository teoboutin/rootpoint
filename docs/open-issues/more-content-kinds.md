---
kind: todo
---
# Bullet lists, code snippets and video are not content kinds yet

## Summary

An area holds text or an image, and equations are scheduled by `milestone@plans@math-support`. The
other kinds `goal@rootpoint@minimal-content-set` lists, bullet lists, code snippets and video, do
not exist.

## Details

### What

Each kind adds a variant of `Content` in `path@rootpoint-model@src/document.rs`, its options, its
drawing in the render crate and its editor in the app. Equations, inline and displayed, left this
entry for the milestone that designs and builds them.

### Why it matters

`goal@rootpoint@real-presentations` asks that the owner give real talks with the tool. The owner
needs bullet lists before the first real talk, and needs no code snippet for the talk planned; video
was named among the kinds of the goal.

### What would close it

A design session per kind, or one for several, and its work built. Each new kind is read against
`tripwire@rootpoint-render@render-within-bounds` at its design audit, and video, whose files the
browser build holds in memory, against `tripwire@rootpoint-app@browser-pack-too-large`, and decides whether its
options earn named styles, which the style tables hold for text alone, its areas and its spans,
`design@rootpoint-model@style-kinds`.
