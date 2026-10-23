---
kind: question
---
# A double-click begun in the final view enters no editing, where the head on entering editing says it does, awaiting the owner

## Summary

`design@rootpoint-app@edit-entry` states, on the owner's word, that a double-click on any text area
enters editing. The window builds an exception, awaiting the owner as D8 of
`milestone@plans@editor-conveniences`: a double-click whose first click left the final view acts as
that one click. Until the owner rules, the head and the code differ in that case.

## Details

### What

`App::pointer` in `path@rootpoint-app@src/view.rs` keeps the time of the click that left the final
view, and a press within egui's double-click delay of it starts nothing, so the second click of the
double-click neither enters editing on a text area nor sets a divider halfway. The test
`a_double_click_in_the_final_view_only_selects` of `path@rootpoint-app@tests/gestures.rs` pins it.

### Why it matters

A design home is built intent: a session that checks the code against
`design@rootpoint-app@edit-entry` meets the divergence, and must not close it by following the code.
It closes by the owner's ruling.

### What would close it

The owner's ruling on D8. If the owner keeps the default, the head states the case; if not, the
guard leaves the code and its test, and the code follows the head.
