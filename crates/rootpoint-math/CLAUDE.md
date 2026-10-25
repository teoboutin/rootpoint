# rootpoint-math

## The crate builds for the browser

The gate `wasm` of `path@xtask@src/main.rs` builds this crate for `wasm32-unknown-unknown`, per
`design@rootpoint@wasm-from-the-start`, as it does the model crate, whose
`path@rootpoint-model@CLAUDE.md` states what fails that build and what does not.

## The build is tied to its checkout

`path@rootpoint-math@src/lib.rs` reads `ROOTPOINT_CHECKOUT`. So does every binary target and build
script added to this crate, as `path@xtask@CLAUDE.md` says.

## The table of commands is generated

`path@rootpoint-math@src/commands.rs` and `path@rootpoint-math@W3C-NOTICE.txt` are written by
`cargo x math-commands`, from W3C `unicode.xml` and the built-in math font; a change goes into the
generator, `path@xtask@src/math_commands.rs`, and the files are generated again. The table holds
only the characters the font has, so a change of the font regenerates it, and the test
`the_missing_set_is_pinned_both_ways` of `path@rootpoint-math@tests/font.rs` fails until the
missing set is read again.

## A layout scales exactly

Every value of `path@rootpoint-math@src/layout.rs` is a value of the font in design units times
the size of a style; nothing is rounded and no device table is read, so the layout at `k` times a
size is the layout scaled by `k`, which `path@rootpoint-math@tests/layout.rs` checks to 0.001.
An equation fitted to its area relies on it.
