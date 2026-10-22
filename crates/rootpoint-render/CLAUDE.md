# rootpoint-render

## The crate builds for the browser

The gate `wasm` of `path@xtask@src/main.rs` builds this crate for `wasm32-unknown-unknown`, per
`design@rootpoint@wasm-from-the-start`, as it does the model crate, whose
`path@rootpoint-model@CLAUDE.md` states what fails that build and what does not. The renderer reads
no file: the caller gives it every asset's bytes, per
`design@rootpoint-render@caller-supplies-assets`.

## The build is tied to its checkout

`path@rootpoint-render@src/lib.rs` reads `ROOTPOINT_CHECKOUT`. So does every binary target and
build script added to this crate, as `path@xtask@CLAUDE.md` says.
