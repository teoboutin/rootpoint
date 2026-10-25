---
kind: todo
---
# The project has no plan for testing its invariants beyond a few property tests

## Summary

Property tests exist in rootpoint-model, one in rootpoint-protocol drives the commands at random,
one in rootpoint-app walks the caret, and one in rootpoint-math writes and reads formulas. Nothing
is fuzzed, and the window's actions are tested only with fixed inputs written by
hand. A systematic approach to testing, chosen for the whole project, is not set up.

## Details

### What

What exists, read in the tree:

- `path@rootpoint-model@tests/ops.rs` holds a proptest over random sequences of operations: each
  accepted operation keeps the invariants and is undone by its inverse, and a refused one changes
  nothing;
- `path@rootpoint-model@tests/file.rs` holds a proptest that a random document saved then loaded
  gives the document back;
- `path@rootpoint-model@tests/geometry.rs` holds a proptest of the geometry;
- `path@rootpoint-protocol@tests/properties.rs` holds a proptest over random sequences of
  commands, `undo`, `redo`, `begin`, `end` and `cancel` and the commands of named styles included,
  on a headless `Session`: after each command the document is valid, holds no cosmetic breach, a
  missing style among them, and saves to a text that loads back equal. It does not check that an
  undo restores the document as it was before;
- `path@rootpoint-app@tests/editing.rs` holds a proptest of the caret over random text trees, each
  position visited once each way and drawn on its glyph;
- `path@rootpoint-math@tests/latex.rs` holds a proptest that a random formula written as LaTeX
  reads back equal;
- every other test of the project is written by hand, with fixed inputs. A search of every
  `Cargo.toml` and `.rs` file for `proptest`, `quickcheck`, `arbitrary` and `libfuzzer` finds only
  these six test files, and the dev-dependencies of rootpoint-app, rootpoint-math, rootpoint-model
  and rootpoint-protocol.

Approaches to weigh, as open leads, none chosen:

- **check in the protocol's property test that an undo restores the document** as it was before
  the step it reverts, which the test above does not check yet;
- **start the property sequences from documents loaded from files edited by hand**, as one whose
  unused Body names a missing font asset, where the test above starts from an empty document only:
  the breach a review of the milestone's named styles found, `add-slide` on such a file, was out of
  its reach;
- **fuzz the protocol's parser** with arbitrary bytes, checking that no line panics and that each
  line either parses or is refused with a reason;
- **fuzz command sequences guided by coverage**, with cargo-fuzz and libFuzzer, which need a
  nightly toolchain, or a stable alternative;
- **fuzz the loader** with mutated RON files, checking that `load_ron` refuses or loads, and never
  panics;
- **drive the window's actions at random** through `App::frame` in tests, checking that each action
  equals its commands on a second session, as `path@rootpoint-app@tests/actions.rs` does by hand.

### Why it matters

The owner wants to be sure that no edit of a user can lead to an invalid document, and asked for the
testing of the whole project to be set up, not fuzzing alone. The model's tests cover operations,
and the protocol's cover its commands and groups from an empty document, but not an undo's result,
nor a document loaded from a file edited by hand, nor the window's own state. A breach found by a
user instead of a test costs a talk, against `goal@rootpoint@real-presentations`, and every
operation the GUI offers is to be checked headless, `goal@rootpoint@headless-core`.

### What would close it

A design session that chooses which of these approaches the project runs, where each runs (the
gates of `path@xtask@src/main.rs` or a separate command), and with what budget, and the chosen
tests built.
