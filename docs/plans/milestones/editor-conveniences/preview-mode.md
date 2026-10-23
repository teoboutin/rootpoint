# Slice: The preview

The seventh and last slice of `milestone@plans@editor-conveniences`. Read the milestone document
whole before this spec. Its harvest commit also deletes the milestone document.

## Builds

- **The view state.** `View::preview`, a `bool`, in `path@rootpoint-protocol@src/session.rs`. The
  rules, in the session:
  - `preview on` sets it and deselects;
  - `preview off` clears it;
  - `select n<id>` clears it;
  - `select s<id>`, `present` and `deselect` leave it as it is;
  - `view` reports it, as `preview=on` or `preview=off` after the fields it prints today. Every test
    that asserts the exact reply of `view` gains the field: a search for `presenting=` in the tests
    and scripts of the protocol and the app lists them: 14 lines in 4 files at the time of writing,
    by `grep -rn "presenting=" crates/*/tests`.

  These are view commands: they apply no operation, and the socket needs the authority to send
  them, as for `select`, `design@rootpoint-app@one-controller-at-a-time`; `App::socket_command` of
  `path@rootpoint-app@src/control.rs` already sends every command but the queries, `control` and
  `release` through that rule, so it needs no change. The grammar of
  `path@rootpoint-protocol@src/command.rs` gains `Command::Preview { on: bool }`, read from
  `preview on` and `preview off` and written back so; any other word after `preview` is an error
  that names the two.
- **The button.** "Preview", in the toolbar, between "Sample" and "Present", toggles the preview.
  It is a view-only input of the window, usable while the socket holds the authority.
- **The slide view in preview.**
  - `App::overlay` draws no mark.
  - A press on the slide starts no gesture: no drag, no split at the pointer, no editing.
  - A click on an area sends `select` on it, which ends the preview and selects that area.
  - A secondary click on an area, or a click or a secondary click on a row of the outline, acts as
    outside the preview: its `select` ends the preview, and a secondary click then opens the node
    menu, `design@rootpoint-app@node-menu`. The menu describes the selection, which the preview has
    none of until that `select`.
  - Ctrl or Shift held shows no line of a split at the pointer, and the pointer over a divider
    asks for no resize cursor: both are marks of a gesture, and no gesture starts in preview.
  - Esc sends `preview off`. Esc's other meanings, hiding the line of a split at the pointer,
    cancelling a drag and leaving editing, do not arise in preview, where no gesture runs.
  - In the app, the new state and its functions are named for the final view, as `App::final_view`,
    since "preview" already names the line of a split at the pointer: `hit::Preview`,
    `Layout::preview`, `Press::Split` and `App::preview_hidden`. `App::final_view` returns whether
    the session's view is in preview, and `App::toggle_final_view` is the button's action, in
    `path@rootpoint-app@src/actions.rs`, per `design@rootpoint-app@actions-are-commands`. The protocol's command keeps the
    word `preview`, which names nothing else there.
  - The slide's image and its size are the editor's, since the side panel's width does not change.
- **Entering the preview ends the window's gestures**, as any action that does not join a gesture
  does, `design@rootpoint-app@gesture-groups`: the button's action ends a stay in editing, with its
  edits one undo step, before it sends `preview on`, and no drag is open when the button is
  clicked. The socket's `preview on` needs the authority, whose `control` already ended them.
- **A double-click on a divider** resets nothing in preview, since its press starts no gesture.
- **The node keys and Delete** act on the selection, and there is none in preview, so they do
  nothing until a click selects.

## Claims

- **The session's rules.** Test: a protocol test in `path@rootpoint-protocol@tests/view.rs` runs
  `select n3`, `preview on`, `view`, `select n5`, `view`. It finds `preview=on` with no selection,
  then `preview=off` with n5 selected. Shown to fail: against a `preview on` that keeps the
  selection, the first `view` names n3.
- **The preview's image has no mark.** Test: `path@rootpoint-app@tests/presenter.rs` compares the
  editor's image in preview with presenter mode's at the same size. Before this slice it compared
  the texture only, so it now also checks that the overlay painted no shape but the texture's own
  image in the slide's rectangle. Shown to fail: against
  an overlay that still draws the divider lines, a shape is painted.
- **A click ends the preview and selects.** Test: a test of `App::frame` presses Preview, drags a
  divider, then clicks area n3 with Ctrl held. The divider has not moved, the view has
  `preview=off` and n3 selected, and no split happened. Shown to fail: against a slide view that
  starts its gestures in preview, the divider moves. A Ctrl-click alone would not show it: the
  preview selects nothing, and a split at the pointer needs a selection.
- **The button works while the socket holds the authority.** Test: in
  `path@rootpoint-app@tests/control.rs`, with the socket holding control, Preview toggles the view.
  Shown to fail: against a button gated as an edit, the view is unchanged.

## Audit subjects

- `View`, the view commands and `view`'s reply in `path@rootpoint-protocol@src/session.rs`; the
  grammar of `path@rootpoint-protocol@src/command.rs`.
- `App::toolbar`, `App::slide_view` and `App::overlay`; `path@rootpoint-app@tests/presenter.rs`.
- `design@rootpoint-protocol@view-state`, `design@rootpoint-app@editor-overlay`,
  `design@rootpoint-app@one-controller-at-a-time`, `goal@rootpoint-app@faithful-slide-view`;
  `tripwire@rootpoint-render@render-within-bounds`, whose re-entry this audit is, though the slice
  draws through `render_slide` unchanged.

## Fails alone on

- A mark is painted in preview, or a click in preview starts a gesture.
- The session's `view` reports a preview and a selection at once.

## Premises that expire

None.

## Decided design

**A "Preview" button toggles a final-result view inside the editor, and a click on the slide ends
it**, `thread@editor-conveniences@final-preview`. The owner asked for the final result without the
editor's marks and without presenter mode (`argument@editor-conveniences@arg-7`).

- It is view state of the session, because `goal@rootpoint-protocol@commands-drive-everything`
  makes every view the editor shows a command (`argument@editor-conveniences@arg-14`). The rival, a
  flag only the window holds, fails that goal.
- Entering the preview deselects, and a click on the slide ends it and selects, as the owner
  proposed (`argument@editor-conveniences@arg-36`). The session made that a rule of the session, so
  an agent and a headless test meet the same behaviour (`argument@editor-conveniences@arg-42`).
- A press in the preview starts no other gesture, since the author did not see where it would act.
- The nearest rival, a preview in which a click does nothing, costs a click every time the author
  leaves it.
- What is given up: a selected area's own custom style cannot be tuned while previewing. The Styles
  tab still updates named styles live.

The owner was not fully certain of this shape, and T2 watches it.

## Implementation sequence

1. `View::preview` and its commands in the session. Fails alone on the protocol test.
2. The button, the overlay off, and the slide view's presses in preview. Fails alone on the app
   tests.

## Acceptance criteria

This slice judges no acceptance criterion. Its harvest commit reports once more on every acceptance
criterion of the milestone, as the milestone document leaves.

## Harvest

- `design@rootpoint-protocol@view-state` rewritten: `View` holds the preview, with its rules. The
  texts that cite it are re-read: `design@rootpoint-app@node-selection`,
  `design@rootpoint-app@one-controller-at-a-time`, `issue@rootpoint-protocol@history-browsing`, and
  the comment of `path@rootpoint-protocol@tests/scripts/view.txt`, which the checker does not read.
- `design@rootpoint-app@editor-overlay` rewritten: the overlay draws nothing in preview. The texts
  that cite it are re-read: `path@rootpoint-app@CLAUDE.md`, `design@rootpoint-app@edit-in-place`,
  `design@rootpoint-app@take-over-at-once` and the comment of `path@rootpoint-app@tests/presenter.rs`.
- The rules of the preview, in the rewritten `design@rootpoint-protocol@view-state`, attributed to the
  owner with the owner's hedge, "I'm not fully certain though"; the presses of the slide view in
  preview, in an app head if `primer@design-heads` admits one, or a comment at `App::slide_view`.
- `design@rootpoint-app@one-controller-at-a-time`: the Preview button joins the view-only inputs.
- T2, recorded on the owner's word in `register@rootpoint-protocol@tripwire`, guarding
  `design@rootpoint-protocol@view-state`, with its label in its text.
- The milestone document leaves, as its "Harvest" says.
