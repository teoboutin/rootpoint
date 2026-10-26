---
kind: todo
---
# The window offers no interactive tour of its parts

## Summary

A tour would draw callouts over the real window, step by step, naming each part and what its
buttons do. The owner did not rule it out, and left it for later, for completeness and interactive
help. The pictures planned for the built-in sample cover a few parts of the window, and the owner
asked that they not be fully exhaustive.

## Details

### What

The window has no help of its own beyond its labels and the status line. The redesign of the
built-in sample, `spec@plans@sample-redesign`, plans pictures the window draws of itself with
numbered callouts: one of the whole window, then one of each of a few panels or tabs. The owner
asked that these pictures not be fully exhaustive, and left an interactive tour for later.

Leads, none decided:

- the spec plans to anchor the pictures' callouts on the rectangles a frame records by label, as
  `App::button_rect` returns a button's; a tour could anchor on the same rectangles, drawn live over
  the window instead of into a picture;
- a tour is no part of a presentation: a pack or a slide cannot hold it, so it lives in the app,
  not in the sample;
- `design@rootpoint-app@editor-overlay` keeps every editor mark out of the slide's image, which a
  tour's marks would respect as the overlay does.

### Why it matters

A new user meets every button with no explanation but its label. The planned sample explains a
chosen few, and the rest of the window, the node menu, most of the Styles tab's controls and the
dialogs among them, is learnt by trying.

### What would close it

A tour the user starts from the window, which steps through its parts with callouts over the live
window, natively and in the browser build.
