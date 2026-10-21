# Rejected designs of rootpoint-app

The designs not kept for rootpoint-app, and why: one level-three heading per design, each cited by
the design head whose argument covers it, in `register@rootpoint-app@design`.

### iced, Slint, xilem or gpui as the window's host `##other-gui-hosts`

`live`. The toolkit survey of the first prototype, `path@rootpoint@docs/grounding/toolkits.md`,
found: under the project's own renderer, iced's `pane_grid` does not help, since the dividers are
inside the slide's image, and its releases are slow, 0.13 in 2024-09 and 0.14 in 2025-12; Slint
makes a free-form canvas from declarative items, which is friction; xilem and gpui failed the
first prototype's criterion that the browser build edit and present as the native one does:
xilem's web backend lays text out in the browser's DOM, so no canvas is shared, and gpui's web
support was unpublished and needs nightly Rust and threaded wasm. iced stays the host to weigh
again if slides are ever drawn with a toolkit's own painter, where its one text engine and its
`iced_test` screenshots count.
