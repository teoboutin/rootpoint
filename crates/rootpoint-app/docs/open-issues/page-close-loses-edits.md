---
kind: todo
---
# Closing or reloading the browser build's page loses its edits with no warning

## Summary

The browser build holds every edit in memory and saves nothing. F5 is kept from reloading the
page, but Ctrl+R, the browser's reload button and closing the tab still end the page at once, and
the edits are lost with no question asked.

## Details

### What

`App::in_memory`, the browser build's editor, holds the presentation and its files in memory,
`design@rootpoint-app@browser-load-only`. `keep_f5` of `path@rootpoint-app@src/web.rs` prevents
F5's default alone, per `design@rootpoint-app@editor-keys-kept-from-browser`; the page installs no
`beforeunload` listener, so the browser asks nothing before it reloads or closes the page. Found by
reading the code at the review before the merge of the browser build; the README of the app crate
says the changes are lost when the page is closed or reloaded.

### Why it matters

An author who edits a presentation in the browser and presses Ctrl+R, or closes the tab by mistake,
loses the work, which `goal@rootpoint-app@browser-window` asks the browser build to edit as the
native window does. While the build saves nothing the edits cannot be kept anyway, so the loss is
of the session's work, not of a file.

### What would close it

A `beforeunload` listener that asks the browser for its confirmation while the session holds an edit
made since the last open, checked in a headless browser by dispatching `beforeunload` and reading
the event's default prevented; or the owner's ruling that no warning is wanted until saving in the
browser comes with packing.
