# rootpoint

A minimal presentation editor with an interactive GUI, written in Rust. Its document model is built;
the renderer, the protocol and the GUI are not yet.

## Layout of the repository

- `path@rootpoint@docs/`: the goals, the design and the open issues of the project, and
  `path@rootpoint@docs/plans/` for work that is designed and not built.
- `path@rootpoint@crates/rootpoint-model/`: the document model, which runs headless and in the
  browser.
- `path@rootpoint@tools/xtask/`: the maintenance tool. `cargo x gates` runs every check a branch
  owes before it merges, and `cargo klarch` runs the document checker at the version the project
  pins.
