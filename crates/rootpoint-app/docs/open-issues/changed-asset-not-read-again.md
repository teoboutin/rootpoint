---
kind: todo
---
# An image or a font file changed on disk while the editor runs is not read again

## Summary

The editor decodes each asset once, and decodes it again only when the document's assets, their
use, or the session's directory change. A file edited or replaced outside the editor keeps showing
as it was until one of those changes, as on reopening the presentation.

## Details

### What

`Assets::refresh` of `path@rootpoint-app@src/assets.rs` keys the decoded `FontSet` and `ImageSet`
on the session's directory and, for each asset the slides use, on its source and its use as an
image or a font. A `Path` source is the path's text, not the file's bytes or its modification
time, so a change of the file leaves the key equal and nothing is read. Read from the code; it is
`not established` by a run.

### Why it matters

An author who edits a picture in another program and comes back to the editor sees the old one,
and could present it. It bears on `goal@rootpoint-app@faithful-slide-view`, which asks the window
to show the slide the renderer would draw from the presentation's files.

### What would close it

The editor reads an asset's file again when it changes: on a modification time newer than the
decoding's, checked when the slide is drawn again, or on a file-system notification, or at least on
an explicit "reload assets" action. A test changes a file in its `Files` between two frames and
sees the new image in the texture.
