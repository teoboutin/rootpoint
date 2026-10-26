---
kind: todo
---
# A distributed build carries no notice of the crates and the toolkit's fonts it links

## Summary

The browser build that GitHub Pages serves, `path@rootpoint@.github/workflows/pages.yml`, is a copy
of the editor's binary. `cargo x web` puts beside it, in `licences/`, the project's licences and the
notices of what the project itself builds in: the W3C notice of the table of commands and the SIL
Open Font License of Open Sans and of Libertinus Math. It puts no notice of the crates the binary
links, nor of the fonts eframe builds in.

## Details

### What

The binary built for the wasm target links 153 packages, the workspace's own crates included: the
distinct names and versions that `cargo tree -p rootpoint-app --target wasm32-unknown-unknown -e
normal --prefix none` prints, its mark of a repeated subtree, `(*)`, removed, less the 10 marked
`(proc-macro)`, which run at the build and are not linked.
Most are under MIT or Apache-2.0, whose terms ask that a copy carry the copyright notice and the
licence text. eframe's feature `default_fonts` builds in the fonts of the crate
`epaint_default_fonts` 0.37.0, licensed `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`:
Hack, Noto Emoji, Ubuntu Light, the egui icons and the emoji icon font, with their licence texts in
that crate's `fonts/` directory. No generated list of these notices exists in the project.

The five font files the project builds in, the four faces of Open Sans and Libertinus Math, carry
their notice inside the file as well: name IDs 0, 13 and 14 of their `name` table hold the
copyright and the OFL notice. Not established for the fonts of `epaint_default_fonts`. The page
links no notice: `licences/` is served beside it, and GitHub Pages lists no directory, so a user
reaches a notice only by its address. The W3C notice asks for its text "in a location viewable to
users".

### Why it matters

`design@rootpoint@permissive-licence` argues that every dependency carries a licence that lets the
whole be distributed under MIT or Apache-2.0. A distribution that leaves out the notices those
licences ask for keeps the project out of their terms, which the publication on Pages, and the
owner's plan to make the project open source, make concrete. No native binary is distributed yet.

### What would close it

`cargo x web`, and any later packaging of a native binary, writes the notices of every linked
crate and of the toolkit's fonts beside the build, generated from the dependency tree so that a
dependency added later is covered, for example with `cargo about`; or the owner rules which notices
a distributed build carries.
