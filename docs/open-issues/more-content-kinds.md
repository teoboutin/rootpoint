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

### Why it matters

`goal@rootpoint@real-presentations` asks for talks that need mathematics, which the prototype draws
only as images prepared elsewhere, the owner's own fallback; bullet lists and code snippets are
among the kinds the owner named as needed.

### What would close it

A design session per kind, or one for several, and its work built. Each new kind is read against
`tripwire@rootpoint-render@render-within-bounds` at its design audit.
