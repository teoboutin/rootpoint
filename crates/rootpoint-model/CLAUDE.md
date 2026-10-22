# rootpoint-model

## The crate builds for the browser

The gate `wasm` of `path@xtask@src/main.rs` builds this crate for `wasm32-unknown-unknown`, per
`design@rootpoint@wasm-from-the-start`. A
dependency or a `std` module that exists only on a desktop target, such as `std::os::unix`, fails
it; `std::fs` compiles there and fails only at run time, so the gate does not catch it.

## The build is tied to its checkout

`path@rootpoint-model@src/lib.rs` reads `ROOTPOINT_CHECKOUT`. So does every binary target and build
script added to this crate, as `path@xtask@CLAUDE.md` says.
