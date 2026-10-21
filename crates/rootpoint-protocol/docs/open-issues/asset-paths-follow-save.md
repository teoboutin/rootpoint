---
kind: defect
---
# An asset's path points at another file once the presentation is saved into another directory

## Summary

`Session` reads an asset's path against the directory of the presentation last saved or loaded,
or against the working directory before either, and keeps the path as it was typed. A `save` into
another directory changes the directory the path is read against, so the same path then names
another file, or none.

## Details

### What

The session keeps the directory of the last `save` or `load` in its private field `dir`; `save`
and `load` both set it to the parent of their path. `Session::asset_bytes` reads an asset whose
source is a path at `dir` joined with that path. Neither `save` nor `asset` rewrites the paths
stored in the document's assets.

Reproduced with `rootpoint-headless` in a directory that holds `pic.png` and an empty directory
`talk/` (the native `Files` creates no directory, so a `save` into a missing one replies `err`):

```text
add-slide
asset pic.png
image n2 a3 fit
render s1 out.png       -> ok fallback=- missing=- glyphs=-   (reads ./pic.png)
save talk/deck.ron
render s1 out.png       -> ok fallback=- missing=a3 glyphs=-  (reads talk/pic.png)
```

The reverse also holds: with `pic.png` only under `talk/`, the render after the save finds it.
`path@rootpoint-protocol@tests/files.rs`, in
`assets_are_read_beside_the_presentation`, relies on this second case: it adds an asset before a
first save into `talk/`, with the file only under `talk/`. A presentation saved into the same
directory again, or loaded and saved beside itself, keeps every asset. By the same mechanism, not
run separately: saving a copy into another directory breaks every asset whose file is not also
there.

Ruled out: the code is not at odds with the rule the protocol's slice stated, "relative to the
presentation file's directory, or to the working directory of a presentation not yet saved"; it
applies that rule literally. The rule says nothing about the moment a save changes the
presentation's directory, and that silence is the defect.

### Why it matters

`design@rootpoint-model@image-storage` names each asset by a path relative to the presentation
file. A `save` into another directory moves the presentation file and leaves its assets' paths
unchanged, so after it the paths name other files. The render reports the asset as missing
(`missing=a3`), so no image is lost from disk, but the presentation saved under the new name is
broken without any `err` reply. An agent that builds a presentation before choosing where to save
it meets this on its first save, against `goal@rootpoint@text-command-protocol`.

### What would close it

A decision on what a `save` into another directory does to the assets' paths, taken under
`skill@knowledge-architect-design`. The shapes seen so far: rewrite each relative path on `save`
so it names the same file from the new directory (lexically, which needs a rule for paths that
cannot be rewritten so, such as a `..` that leaves a known directory); read the path typed to
`asset` against the working directory and store it relative to the presentation; or refuse a
`save` into another directory while an asset is stored by path, with an `err` reply that names
it. Then the fix, and a test that saves into another directory and renders with `missing=-`. The
test `assets_are_read_beside_the_presentation` changes with it.
