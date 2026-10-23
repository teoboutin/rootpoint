---
kind: defect
---
# The actions of editing in place are functions of editing.rs, where the head says actions.rs

## Summary

`design@rootpoint-app@actions-are-commands` says each action of the editor is a function of
`path@rootpoint-app@src/actions.rs`. The actions of a stay in editing, which issue `begin`, `text`
and `end`, are functions of `path@rootpoint-app@src/editing.rs`.

## Details

### What

`App::edit_at`, `App::stop_editing`, `App::type_str`, `App::delete_backward`,
`App::delete_forward`, `App::cut`, `App::set_preedit` and `App::commit_ime` are in
`path@rootpoint-app@src/editing.rs`, beside the caret's state, which no command places,
`design@rootpoint-app@caret-editor-state`. Each issues its commands through `App::run`, and
`path@rootpoint-app@tests/actions.rs` checks them against a second session, so the head's rule on
commands holds; only its statement of where the functions are is false of them. No comment or
commit message gives a reason for the place. Found by the self-consistency review of the work of
the node menu of `milestone@plans@editor-conveniences`, whose own actions were moved to
`path@rootpoint-app@src/actions.rs` to meet the head.

### Why it matters

A design home is built intent: a session that reads the head looks for every action in one file,
and misses these; a session that checks the code against the head finds a divergence each time.

### What would close it

Either the editing actions move to `path@rootpoint-app@src/actions.rs`, leaving the caret's state
and its geometry in `path@rootpoint-app@src/editing.rs`, or the head is reworded to name both
files, under `skill@knowledge-architect-decision-recording`.
