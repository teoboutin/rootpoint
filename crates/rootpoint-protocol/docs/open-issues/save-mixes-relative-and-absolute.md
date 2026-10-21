---
kind: defect
---
# A save by an absolute path from a session whose directory is relative copies the assets already beside it

## Summary

`Session::save` decides whether an asset's file lies inside the saved file's directory by
comparing two paths as text. When one is relative to the working directory and the other is
absolute, they never match, so the save copies every such asset beside the saved file under its
bare name instead of keeping its path. No file is lost or overwritten; the presentation gains
duplicate files, and an asset in a subdirectory is flattened.

## Details

### What

In `path@rootpoint-protocol@src/session.rs`, `Session::save` computes the target directory as
`normalize(parent(path))` and each `Path` asset's file as `normalize(self.dir.join(p))`, then calls
`inside` of `path@rootpoint-protocol@src/save.rs`, which uses `Path::strip_prefix`. The session's
directory, `self.dir`, is the parent of the path last given to `load` or `save`, as given: relative
when that path was relative. The session has no working directory of its own to make either path
absolute, since it opens no file and runs in the browser too.

Reproduction with the headless binary, in a directory `D` holding `img/cat.png`, with `D` as the
working directory:

```text
add-slide
asset img/cat.png
image n2 a3 fit
save deck.ron
load deck.ron
save D/deck2.ron        (D written as an absolute path)
```

Every command replies `ok`. `deck.ron` holds `source: Path("img/cat.png")`; `deck2.ron` holds
`source: Path("cat.png")`, and a new file `D/cat.png`, a copy of `D/img/cat.png`, is written. The
mix the other way round copies in the same way, measured by a review of the native GUI slice with
a protocol test over `MemoryFiles`: `load /D/deck.ron`, whose asset is `img/cat.png`, then
`save deck2.ron`, writes a `cat.png` and a `deck2.ron` holding `source: Path("cat.png")`.

Ruled out: the overwrite clause holds, so no file of other bytes is replaced. The GUI no longer
produces the mix: every path it gives `load` and `save`, from a dialog or the command line, is
made relative to the working directory when it is absolute, in `App::relative_if_absolute` of
`path@rootpoint-app@src/actions.rs`. A headless script, or a later client of the protocol, still
can.

### Why it matters

The save rule promises that an asset whose file lies inside the saved file's directory, at any
depth, keeps its file. Here it does not: a presentation organized with an image folder is
flattened and its images duplicated. It strains `goal@rootpoint@readable-save-file`, whose saved
file and its files beside it should be the author's own layout, and it rests on
`design@rootpoint-protocol@session-files`, which keeps the session from asking the file system.

### What would close it

The comparison sees both paths in one form. Either `Files` gains a way to make a path absolute,
which the native host answers from its working directory and the browser's from its own root, and
the save compares the absolute forms; or a `save` or `load` whose path is absolute while the
session's directory is relative, or the reverse, is refused with an error naming the mix. Then the
reproduction above keeps `Path("img/cat.png")` in `deck2.ron` and writes no `D/cat.png`, or
replies `err`; a protocol test over `MemoryFiles` checks it.
