---
kind: observation
---
# A command of the control socket may wait while the window draws no frame

## Summary

The window runs the socket's commands only at the start of a frame. Whether a window minimized or
hidden by the desktop still draws a frame when the socket wakes it is not established; if it does
not, an agent's command waits until the window is shown again, and the client has no timeout.

## Details

### What

`App::frame` in `path@rootpoint-app@src/lib.rs` drains the socket's lines at its start, as
`design@rootpoint-app@control-socket` has it, and the listener of
`path@rootpoint-app@src/control.rs` wakes the window by `Context::request_repaint` for
each line. `rootpoint_protocol::control::Connection::send` then waits for the reply with no timeout.
Seen by the review of the slice that built the socket, under Xvfb: with the window idle and shown,
every command got its reply, so the wake works there. Not reproduced: a window minimized, or
covered on a desktop that stops drawing covered windows, was not tested; whether eframe 0.37.0 or
the desktop skips frames then is not established. What would make it reproducible: start
`rootpoint <dir>`, minimize the window, and time a `rootpoint-ctl <dir> view`.

### Why it matters

An agent that drives a window the author has minimized would hang on its next command, with no
reply and no error, against `goal@rootpoint-app@live-agent-edits`, which asks that an agent drive
the open window while the author watches; and against `goal@rootpoint@text-command-protocol`.

### What would close it

The measurement above on the owner's desktop. If the command waits, the commands are drained also
when no frame is drawn, or the client is given a timeout that names the cause; if it does not, the
entry is deleted.
