# xtask

## The pin of the checker is written in three places

The version of the checker is pinned in `path@xtask@Cargo.toml`, for `knowledge-architect` and
`knowledge-architect-gates`, and in `[project] checker-version` of `knowledge-architect.toml`. A
move edits all three in one change, as `skill@knowledge-architect-setup@moving-the-pin` says: the
binary refuses every command when the manifest's version differs from the one it links, and
nothing compares the gates library's version.

## The build is tied to its checkout

`main.rs` reads `ROOTPOINT_CHECKOUT`, which `path@rootpoint@.cargo/config.toml` sets to the
checkout's root. Without that read, two checkouts that share a target directory run each other's build of this
crate, since cargo does not track `CARGO_MANIFEST_DIR`. Every library root, binary target and
build script added to the workspace reads the variable too, as
`skill@knowledge-architect-setup@rust-project` says.
