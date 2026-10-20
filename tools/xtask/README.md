# xtask

The maintenance tool of the project. It is never published.

- `cargo x gates` runs every check a branch must pass before it merges: formatting, the document
  check, the commit messages, clippy and the tests. It runs them all when one fails, and its exit
  code is the verdict. The complete output of each gate is written under `target/gates/`.
- `cargo klarch <command>` runs the knowledge-architect checker at the version pinned in
  `path@xtask@Cargo.toml` and in `knowledge-architect.toml`. `cargo klarch check` checks the
  documents.

Both aliases are defined in `path@rootpoint@.cargo/config.toml`.
