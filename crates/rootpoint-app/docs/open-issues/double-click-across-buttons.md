---
kind: defect
---
# A secondary click during a divider drag, then the release, resets the divider as a double-click

## Summary

egui counts a double-click across the mouse buttons. A secondary click during a divider drag,
followed within 0.3 s by the release of the primary button, is read as a double-click on the
divider, which sets it halfway as an undo step of its own after the drag's.

## Details

### What

`App::pointer` in `path@rootpoint-app@src/view.rs` reads
`button_double_clicked(PointerButton::Primary)` at the release, and a release of a press on a
divider with that flag runs `App::reset_divider`. egui 0.37.0 keeps one `last_click_time` for every
button in its input state, so a secondary click shortens the time to the primary release that
follows it. Seen by the spec-conformity review of the work of the node menu of
`milestone@plans@editor-conveniences`, on the sample: a divider drag, a secondary click during it,
and the primary release within 0.3 s left two undo steps, the drag and a reset to 0.5. The same
result came with the node menu's call removed from the view, so it predates the menu. The node
menu ignores a secondary click during a drag, which does not stop egui from counting it.

### Why it matters

A drag the author ends where they meant to ends with the divider halfway, against
`goal@rootpoint-app@direct-manipulation`; undo restores it, at one more step.

### What would close it

The reset reads a double-click only when both clicks were of the primary button, for example by
the time of the last primary click kept by the window; and a test of `App::frame` that makes the
sequence above and finds one undo step.
