---
kind: defect
---
# A stay in editing that changes nothing in the end, as an abandoned composition, leaves an undo step

## Summary

A stay in editing whose only edits are a composition later abandoned, or a span opened by a style
action and left empty, pushes an undo step whose net effect is none, so the next Ctrl+Z appears to
do nothing.

## Details

### What

While a text is edited in place, `App::set_preedit` writes the composing text into the caret's
text node with an `insert` command, inside the group the stay holds. Abandoning the composition, by
an empty `ImeEvent::Preedit` or by `App::stop_editing` (Esc, a click outside), sends a `delete`
command that removes it. `Session::end_group` in `path@rootpoint-protocol@src/session.rs` skips a
group only when it holds no inverse, so the stay pushes a step of two inverses that cancel out.

Reproduction, with the `Driver` of `path@rootpoint-app@tests/editing.rs`: run `text n5 "Z"`; edit
n3; call `App::set_preedit("あ")`, then `App::set_preedit("")`, as the IME events do; call
`App::stop_editing`, which Esc calls; run `undo`. The undo replies `Ok("")`, and n5 is still `"Z"`:
the step undone was the abandoned composition. Assumption, not run: a second undo undoes
`text n5 "Z"`. Found by the self-consistency review of the text editing slice, with a probe test
that is not committed.

A style action with no selection reaches the same defect by another path. Ctrl+B, Ctrl+I or a
control of the Selection tab, `App::style_action`, sends `wrap` of the empty range at the caret,
and `App::stop_editing` sends `unwrap` of the span, still empty, inside the same group, so the stay
pushes a step of two inverses that cancel out. Reproduction, a probe test that is not committed:
run `text n5 "Z"`; select n3 and edit it, `App::edit_at`; call `App::bold`, then
`App::stop_editing`; run `undo`. The undo replies `Ok("")`, and n5 is still `"Z"`.

A formula inserted and left empty reaches it too. Ctrl+M, `App::insert_formula`, sends `insert` of
an empty math item at the caret, and leaving the formula empty, `App::leave_formula`, sends
`delete` of the item, inside the same group. The check "a formula left empty is removed" of
`path@rootpoint-app@tests/actions.rs` runs the two commands. Reproduction, a probe test of the
review of the slice that built it, not committed: run `text n5 "Z"`; edit n3; call
`App::insert_formula`, then `App::stop_editing`; run `undo`. The undo replies `Ok("")`, and n5 is
still `"Z"`.

Not established: whether a stay that types a character and deletes it should leave a step. Text
editors commonly keep one, and this entry does not claim it is a defect.

### Why it matters

`design@rootpoint-app@gesture-groups` makes one stay in editing one undo step, so that undo takes
back what the stay did. A step that takes back nothing costs the user one press of Ctrl+Z that shows
no change, and reads as a broken undo, against `goal@rootpoint-app@direct-manipulation`. The empty
step also counts in `Session::unsaved`, so a window held in memory asks before closing, and the
browser's page before it unloads, by `design@rootpoint-app@no-discard-confirmation`, and its state label counts a change, for an edit
that changed nothing; and, in a workspace, the step costs one autosave write that changes nothing,
`design@rootpoint-app@workspace-autosave`; assumptions, not run.

### What would close it

A stay whose group leaves the text as it found it pushes no step: either `Session::end_group`
drops a group whose net effect is none, or the app skips the commands of a composition abandoned
with nothing committed, of a span opened and left empty and of a formula inserted and left empty. A test in
`path@rootpoint-app@tests/editing.rs` runs each reproduction above and asserts that the first
Ctrl+Z undoes `text n5 "Z"`.
