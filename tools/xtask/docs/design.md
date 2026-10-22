# Design of xtask

How xtask is built, and why: one level-three heading per decision, under level-two subjects. Design
that is decided and not built is in a plan document, not here. What xtask is for is in
`register@xtask@goal`.

## The browser build

### The browser build is bound, and its tests run, by wasm-bindgen-cli at the version of the `wasm-bindgen` crate in `Cargo.lock` `##one-tool-binds-and-tests`

`cargo x web` builds the editor's binary for the wasm target and binds it with the `wasm-bindgen`
program for a page that loads it as a module; the browser tests run through
`wasm-bindgen-test-runner`, set as the wasm target's runner in `path@rootpoint@.cargo/config.toml`.
Both programs come with wasm-bindgen-cli, which writes the module the `wasm-bindgen` crate of the
build describes, so its version is the crate's in `Cargo.lock`, and `cargo x web` refuses another.
The rival, trunk, would build the page with a tool of its own, while the tests' runner would still
need wasm-bindgen-cli: two tools to install where one serves both.

### Every test of the browser build runs in a headless browser, as a gate `##browser-tests-gated`

The gate `wasm-test` runs the app's browser tests, `path@rootpoint-app@tests/actions.rs` and
`path@rootpoint-app@tests/web.rs`, on the wasm target in a headless Firefox through geckodriver, so
`goal@xtask@one-command-runs-every-gate` covers what only a browser refutes: a key the page handles,
and the layout of a slide against the native golden image. A native run cannot refute them: a break
on the wasm target alone, `App::select_node` returning at once, or the renderer's font size scaled
by 1.01, under `cfg(target_arch = "wasm32")`, fails these tests in Firefox and passes them natively;
it is taken again by such an edit and `cargo test -p rootpoint-app --test actions --test web`, run
with and without `--target wasm32-unknown-unknown`. The cost is that the machine that runs the gates
needs Firefox and geckodriver, as `path@xtask@README.md` says.
