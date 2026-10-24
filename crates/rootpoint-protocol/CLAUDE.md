# rootpoint-protocol

## The session opens no file

The session reads and writes only through the `Files` its host gives it, or in its own map when
its store is in memory, per
`design@rootpoint-protocol@session-files`. Three parts of the crate touch the file system, and no
other: the binaries of `path@rootpoint-protocol@src/bin/`; the client of a window's control socket,
`path@rootpoint-protocol@src/control.rs`, which connects to a socket at a path and is built on Unix
only, outside the browser build; and the native host's workspace,
`path@rootpoint-protocol@src/workspace.rs`, its lock and its socket's place, outside the browser
build, which the session never calls.

## The build is tied to its checkout

`path@rootpoint-protocol@src/lib.rs` and the headless binary read `ROOTPOINT_CHECKOUT`. So does
every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
