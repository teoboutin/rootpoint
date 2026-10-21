# rootpoint-protocol

## The session opens no file

The session reads and writes only through the `Files` its host gives it, and only the binary
`path@rootpoint-protocol@src/bin/rootpoint-headless.rs` touches the file system, per
`design@rootpoint-protocol@session-files`.

## The build is tied to its checkout

`path@rootpoint-protocol@src/lib.rs` and the headless binary read `ROOTPOINT_CHECKOUT`. So does
every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
