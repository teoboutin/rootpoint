---
kind: todo
---
# The browser build keeps no copy of its edits between two pages

## Summary

The browser build edits a presentation in memory only, and saves nothing. Once
`milestone@plans@asset-storage` lands, it keeps a presentation only by a download of its pack, and
a page closed or reloaded after its warning has been dismissed, or a tab the browser discards,
loses every edit since the last download.

## Details

### What

The milestone gives the browser build one mode, in memory, and a save that downloads a pack. The
owner observed that the browser build could save on its own as the native workspace does, into a
store of the page's origin that is not `localStorage`, whose size is limited to a few megabytes,
and suggested, as a "maybe", a todo issue for later. Two leads, from the browsers' documentation as recalled
in the discussion and not read for this entry: IndexedDB, a database of the origin whose quota is
a share of the disk; and the Origin Private File System, a tree of files of the origin, which
could implement `Files` as the native workspace's directory does, and so give the browser build a
workspace rather than only a copy.

### Why it matters

`goal@rootpoint-app@browser-window` asks the browser build to edit as the native window does, and
the native window's workspace loses no edit when it closes. In the browser, an author who forgets
to download loses the work of the session.

### What would close it

A design session on a store of the browser build, then its work built: the presentation and its
assets written to the store as the native workspace writes its directory, and opened again from it
on the next page, checked by a browser test that edits, reloads the page and finds the edit.
