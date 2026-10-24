---
kind: defect
---
# An IME composition abandoned in a stay in editing leaves an undo step that changes nothing

## Summary

A stay in editing whose only edits are a composition later abandoned pushes an undo step whose net
effect is none, so the next Ctrl+Z appears to do nothing.

## Details

### What

While a text is edited in place, `App::set_preedit` writes the composing text into the area's text
with a `text` command, inside the group the stay holds. Abandoning the composition, by an empty
`ImeEvent::Preedit` or by `App::stop_editing` (Esc, a click outside), sends a second `text` command
that restores the text. `Session::end_group` in `path@rootpoint-protocol@src/session.rs` skips a
group only when it holds no inverse, so the stay pushes a step of two inverses that cancel out.

Reproduction, with the `Driver` of `path@rootpoint-app@tests/editing.rs`: run `text n5 "Z"`; edit
n3; call `App::set_preedit("あ")`, then `App::set_preedit("")`, as the IME events do; call
`App::stop_editing`, which Esc calls; run `undo`. The undo replies `Ok("")`, and n5 is still `"Z"`:
the step undone was the abandoned composition. Assumption, not run: a second undo undoes
`text n5 "Z"`. Found by the self-consistency review of the text editing slice, with a probe test
that is not committed.

Not established: whether a stay that types a character and deletes it should leave a step. Text
editors commonly keep one, and this entry does not claim it is a defect.

### Why it matters

`design@rootpoint-app@gesture-groups` makes one stay in editing one undo step, so that undo takes
back what the stay did. A step that takes back nothing costs the user one press of Ctrl+Z that shows
no change, and reads as a broken undo, against `goal@rootpoint-app@direct-manipulation`. The empty
step also counts in `Session::unsaved`, so a window held in memory asks before closing, by
`design@rootpoint-app@no-discard-confirmation`, and its state label counts a change, for an edit
that changed nothing; and, in a workspace, the step costs one autosave write that changes nothing,
`design@rootpoint-app@workspace-autosave`; assumptions, not run.

### What would close it

A stay whose group leaves the text as it found it pushes no step: either `Session::end_group`
drops a group whose net effect is none, or the app skips the composition's two commands when it is
abandoned with nothing committed. A test in `path@rootpoint-app@tests/editing.rs` runs the
reproduction above and asserts that the first Ctrl+Z undoes `text n5 "Z"`.
