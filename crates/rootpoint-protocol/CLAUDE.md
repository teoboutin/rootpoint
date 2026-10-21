# rootpoint-protocol

## The session opens no file

The library is part of the browser build: the gate `wasm` of `path@xtask@src/main.rs` builds it,
and `std::fs` compiles there and fails only at run time, as `path@rootpoint-model@CLAUDE.md` says.
The session reads and writes through the `Files` its host gives it; only the binary
`path@rootpoint-protocol@src/bin/rootpoint-headless.rs` touches the file system.

## The build is tied to its checkout

`path@rootpoint-protocol@src/lib.rs` and the headless binary read `ROOTPOINT_CHECKOUT`. So does
every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
