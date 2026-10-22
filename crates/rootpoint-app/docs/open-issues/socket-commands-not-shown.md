---
kind: todo
---
# The window does not show the commands its control socket runs

## Summary

While an agent drives the window through its control socket, the author sees each change but
not the command that made it. The owner, trying the socket with an agent, judged that a display
printing each command the agent sends, as it comes, would be nice, for the future and not in the
slice that built the socket.

## Details

### What

`App::serve_socket` in `path@rootpoint-app@src/control.rs` runs each line of the socket and sends
its reply back to the client; nothing of the line or its reply is shown in the window. The only
trace in the window is the status line, which `App::run` sets to the reason of the last command
that failed, the socket's included, and which a later command clears. The owner proposed, during
the trial of the slice that built the socket, a display that prints the commands the agent sends
as they come, and said it is for the future, not for that slice.

### Why it matters

`goal@rootpoint-app@live-agent-edits` asks that the author watch each change an agent makes and
interrupt it at any moment to correct a mistake before it spreads. A change seen with no command
beside it says what moved, not what the agent asked for, so the author reads the agent's intent
from the slide alone, and can miss an edit on a slide that is not shown.

### What would close it

A view in the window, beside the slide list or under the slide view, that lists the commands the
socket ran, newest last, each with its reply, kept for the session; the window's own commands are
not listed, and the slide is not drawn over. A test in `path@rootpoint-app@tests/control.rs` sends
commands through the socket and finds them listed, in order, with their replies.
