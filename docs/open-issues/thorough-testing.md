---
kind: todo
---
# The project has no plan for testing its invariants beyond the model's property tests

## Summary

Property tests exist in rootpoint-model only. No test drives the protocol's commands at random,
and nothing is fuzzed. A systematic approach to testing, chosen for the whole project, is not set
up.

## Details

### What

What exists, read in the tree:

- `path@rootpoint-model@tests/ops.rs` holds a proptest over random sequences of operations: each
  accepted operation keeps the invariants and is undone by its inverse, and a refused one changes
  nothing;
- `path@rootpoint-model@tests/file.rs` holds a proptest that a random document saved then loaded
  gives the document back;
- `path@rootpoint-model@tests/geometry.rs` holds a proptest of the geometry;
- every other test of the project is written by hand, with fixed inputs. A search of every
  `Cargo.toml` and `.rs` file for `proptest`, `quickcheck`, `arbitrary` and `libfuzzer` finds only
  the model's three test files and its dev-dependency.

Approaches to weigh, as open leads, none chosen:

- **widen the property tests** to the protocol: random sequences of commands, `undo`, `redo`,
  `begin`, `end` and `cancel` included, run on a headless `Session`, checking that the document
  stays valid, that an undo restores the document before, and that a save loads back equal. The
  text protocol makes a command sequence easy to generate and to print as a reproducing script;
- **fuzz the protocol's parser** with arbitrary bytes, checking that no line panics and that each
  line either parses or is refused with a reason;
- **fuzz command sequences guided by coverage**, with cargo-fuzz and libFuzzer, which need a
  nightly toolchain, or a stable alternative;
- **fuzz the loader** with mutated RON files, checking that `load_ron` refuses or loads, and never
  panics;
- **drive the window's actions at random** through `App::frame` in tests, checking that each action
  equals its commands on a second session, as `path@rootpoint-app@tests/actions.rs` does by hand.

The milestone that adds named styles adds one such test, a property test over protocol commands
that no command creates a reference to a missing style.

### Why it matters

The owner wants to be sure no edit of a user can lead to an invalid document. The model's tests
cover operations, but the protocol and the window add their own checks and their own state, such as
groups, the view and a command's refusal of an unknown style. A breach found by a user instead of a
test costs a talk, against `goal@rootpoint@real-presentations`, and every operation the GUI offers
is to be checked headless, `goal@rootpoint@headless-core`.

### What would close it

A design session that chooses which of these approaches the project runs, where each runs (the
gates of `path@xtask@src/main.rs` or a separate command), and with what budget, and the chosen
tests built.
