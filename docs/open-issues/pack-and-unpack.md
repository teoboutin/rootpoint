---
kind: todo
---
# A presentation cannot be packed into one file, nor a packed file opened

## Summary

`AssetSource::Embedded` exists in the format, `design@rootpoint-model@asset-source`, but nothing
writes it: no command packs a presentation and its assets into one file, unpacks one, or opens a
packed file in the browser build, which saves nothing until then.

## Details

### What

The first prototype kept packing out of its scope. Packing writes each asset of a presentation as
`Embedded` bytes into the RON file; unpacking writes them back as files beside it, as `Path`
assets; the browser build opens a packed file through its file picker, and could then save one by
a download, `design@rootpoint-app@browser-load-only`. The owner wants, later than the prototype,
assets that an author arranges in subdirectories of any depth, and a pack then an unpack elsewhere
to keep that tree; the save rule, `design@rootpoint-protocol@asset-save-rule`, already keeps an
asset inside the saved file's directory at its path.

### Why it matters

`goal@rootpoint@readable-save-file` is met only when a packed file opens to the same presentation
as its unpacked form, and `goal@rootpoint@runs-in-browser` only when a packed file opens in the
browser build. Without it, a talk cannot be carried as one file and presented from any computer.

### What would close it

A design session on packing, then its work built: commands that pack and unpack, by the protocol,
with round-trip tests, and the browser build opening a packed file.
