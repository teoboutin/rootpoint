# rootpoint

A minimal presentation editor with an interactive GUI, written in Rust: a document model, a slide
renderer, a text protocol, and the editor, in a window and in a web browser.

## Layout of the repository

- `path@rootpoint@docs/`: the goals, the design and the open issues of the project, and
  `path@rootpoint@docs/plans/` for work that is designed and not built.
- `path@rootpoint@crates/rootpoint-app/`: the editor, the binary `rootpoint`, natively and in a
  web browser.
- `path@rootpoint@crates/rootpoint-math/`: formulas, their LaTeX form and their layout from a math
  font.
- `path@rootpoint@crates/rootpoint-model/`: the document model, which runs headless and in the
  browser.
- `path@rootpoint@crates/rootpoint-protocol/`: the text protocol, and the headless binary that
  runs it.
- `path@rootpoint@crates/rootpoint-render/`: the slide renderer, which draws a slide at any size.
- `path@rootpoint@tools/xtask/`: the maintenance tool. `cargo x gates` runs every check a branch
  owes before it merges, and `cargo klarch` runs the document checker at the version the project
  pins.

## Licence

rootpoint is licensed under either of `path@rootpoint@LICENSE-APACHE` (Apache License 2.0) or
`path@rootpoint@LICENSE-MIT` (MIT licence), at your option. The fonts and the data built into the
binary keep their own licence or notice, whose text sits beside each file.
