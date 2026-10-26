---
kind: deferred
---
# How a step that migrates a save file to the next version is written is not decided

## Summary

`design@rootpoint@saved-presentations-stay-loadable` has every change of the save format add a
step that migrates a file of the version before. No step exists yet, since version 7 is both the
oldest that loads and the current one, and the shape of a step is open.

## Details

### What

A step reads a file of version N and gives one of version N+1, before `load_ron` of
`path@rootpoint-model@src/file.rs` reads it. Two shapes stand:

- a frozen copy of the file's types for each older version, deserialized and converted in Rust to
  the next version's types: checked by the compiler, at the cost of a copy of the types a change
  touches;
- a reader of RON that keeps every name, a variant's included, whose steps rewrite the tree, then
  write it as text for the current reader: one reader for every version, at the cost of writing it.

A third is ruled out by measurement: ron's own untyped `ron::Value` drops the name of every enum
variant, so a step over it cannot tell a `Split(...)` from an `Area(...)`. Parsed with ron 0.12.2,
`(root: Split(id: 4, axis: TopToBottom, children: [Area(id: 2, ...)]))` reads back as
`{"root":{"axis":(),"children":[{...,"id":2}],"id":4}}`, by
`ron::from_str::<ron::Value>` then `ron::to_string`.

### Why it matters

A step written in haste, at the change that needs it, picks a shape for every later version. The
choice also meets `design@rootpoint@crate-existence-rule`, which keeps serialization in the model
crate "until it needs migrations of its own", and the window's overwrite guard,
`design@rootpoint-app@ron-overwrite-guard`, which compares `presentation.ron` with the bytes read at
the open: a step that rewrote the file at the open, rather than migrating it in memory until the
next save, would show the guard's banner with no outside writer, which
`tripwire@rootpoint-app@guard-shows-with-no-outside-writer` counts a defect of the window.

### Trigger

The first change of the save format that raises `FORMAT_VERSION` above `OLDEST_VERSION`: its
author must write the first step, so it weighs the two shapes, under
`skill@knowledge-architect-design`, and the crate rule with them. The test
`a_frozen_file_of_every_version_loads` of `path@rootpoint-model@tests/formats.rs` fails at that
change until a step exists.
