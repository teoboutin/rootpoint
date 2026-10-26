---
kind: todo
---
# The window offers no interactive tour of its parts

## Summary

A tour would draw callouts over the real window, step by step, naming each part and what its
buttons do. The owner wants one later, for completeness and interactive help; the built-in sample's
pictures cover a few parts of the window and are not exhaustive on purpose.

## Details

### What

The window has no help of its own beyond its labels and the status line. The built-in sample,
redesigned by `spec@plans@sample-redesign`, shows pictures the window draws of itself with numbered
callouts, one of the whole window and one per panel or tab for a few of them. The owner asked that
these pictures not be exhaustive, and left an interactive tour for later.

Leads, none decided:

- the callouts of the sample's pictures anchor on the rectangles a frame records by label,
  `App::button_rect` and the panels' rectangles; a tour could anchor on the same rectangles, drawn
  live over the window instead of into a picture;
- a tour is no part of a presentation: a pack or a slide cannot hold it, so it lives in the app,
  not in the sample;
- `design@rootpoint-app@editor-overlay` keeps every editor mark out of the slide's image, which a
  tour's marks would respect as the overlay does.

### Why it matters

A new user meets every button with no explanation but its label. The sample explains a chosen few,
and the rest of the window, the node menu, the Styles tab's controls and the dialogs among them, is
learnt by trying.

### What would close it

A tour the user starts from the window, which steps through its parts with callouts over the live
window, natively and in the browser build.
