# xtask

The maintenance tool of the project. It is never published.

- `cargo x gates` runs every check a branch must pass before it merges: formatting, the document
  check, the commit messages, clippy and the tests. It runs them all when one fails, and its exit
  code is the verdict. The complete output of each gate is written under `target/gates/`.
  Two gates build and test the browser build: `wasm` builds every crate it links for the wasm
  target, and `wasm-test` runs the app's browser tests in a headless Firefox. They need the
  target, `rustup target add wasm32-unknown-unknown`; `wasm-bindgen-test-runner` from
  `wasm-bindgen-cli` at the version of the `wasm-bindgen` crate in `Cargo.lock`,
  `cargo install wasm-bindgen-cli --version <version> --locked`; and Firefox and geckodriver on the
  `PATH`.
- `cargo x wasm-test` runs the gate `wasm-test` alone. It first looks for the programs the gate
  needs, and fails with each one missing and how to install it. It hands the runner the
  geckodriver it found, the one `GECKODRIVER` names or the first on the `PATH`, so the tests run in
  Firefox even where another driver is on the `PATH`; with `GECKODRIVER_REMOTE` set, it looks for
  neither geckodriver nor Firefox.
- `cargo x web [--out <dir>]` builds the browser build of the editor into a static directory,
  `web/` under cargo's target directory by default, `target/web/` unless cargo is told another,
  emptied first: the page of `path@rootpoint-app@web/index.html`, the editor built for the wasm
  target in release, and the loader `wasm-bindgen` writes. It needs the
  `wasm-bindgen` program at the version the gates need, and says how to install it otherwise.
- `cargo klarch <command>` runs the knowledge-architect checker at the version pinned in
  `path@xtask@Cargo.toml` and in `knowledge-architect.toml`. `cargo klarch check` checks the
  documents.

Both aliases are defined in `path@rootpoint@.cargo/config.toml`.
