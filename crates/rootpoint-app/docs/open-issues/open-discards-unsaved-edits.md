---
kind: question
---
# An open that replaces a presentation held in memory loses its unsaved edits without asking

## Summary

Closing a window held in memory with unsaved edits asks first; opening another store from it, by
Open, Open pack, Sample or a line of the socket, replaces the presentation at once, and no undo
reaches the edits it held. Whether such an open should ask too is not decided.

## Details

### What

`design@rootpoint-app@no-discard-confirmation` makes the close of a window held in memory with
`Session::unsaved` above 0 the one action that asks, since no undo reaches a closed window; D6 of
`milestone@plans@asset-storage`, awaiting the owner, adds a workspace whose writes stopped or
failed. The commands `open`, `open-copy`, `open-pack`, `new` and `sample`
replace the session whole, `*self = Session::new()` in `Session::run_inner` of
`path@rootpoint-protocol@src/session.rs`, so the undo stack of the presentation they replace goes
with it: its edits are lost the same way, and nothing asks. The trash in memory goes with it too,
`design@rootpoint-protocol@asset-trash`, so a removed asset's bytes are lost with the undo step
that would bring them back. Found by the self-consistency review of the slice of the hosts of a workspace, by
reading the code; not run in the window.

### Why it matters

An author who opens a pack or the sample after editing an unsaved presentation in memory loses
the edits with no warning, the loss `design@rootpoint-app@no-discard-confirmation` argues a question
for, against `goal@rootpoint-app@direct-manipulation` as the close's question guards it. The
owner accepted the question on close, a case the owner's ruling names; an open is a member that
ruling does not name.

### What would close it

The owner's ruling on whether an open from a presentation held in memory with unsaved edits asks
as a close does; then the question built for those actions in
`path@rootpoint-app@src/actions.rs` with a test of `App::frame`, or the head's body stating that an
open does not ask, and why.
