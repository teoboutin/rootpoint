# Slice: Driving the running GUI by text commands

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- `rootpoint --control <path>`: the native GUI listens on a local socket at that path, reads one
  command per line from each connection, executes it on the GUI's session between frames, and
  writes one reply line. On Linux, the platform built first, the socket is a Unix domain socket; a
  Windows build is not part of the prototype.
- The binary `rootpoint-ctl <path> <command>`, in the protocol crate: sends one line, prints the
  reply, and exits non-zero on `err`. Its target reads `ROOTPOINT_CHECKOUT`, as
  `path@xtask@CLAUDE.md` requires.

## Claims

1. **A command over the socket acts like one from the UI.** A command sent with `rootpoint-ctl`
   changes the document and the view, and `undo` from the UI undoes it. Test: an integration test
   that starts the app's control loop with no window, sends commands, and reads the session.
2. **The GUI wakes for a command.** A command sent while the GUI is idle is applied within one frame
   of its arrival, through egui's `request_repaint` from the socket's thread. Test: the integration
   test of claim 1 times the reply.
3. **The scripts of the protocol run through the socket.** Every script of
   `path@rootpoint-protocol@tests/scripts/` gives the same replies through `rootpoint-ctl`
   as through `rootpoint-headless`.

## Audit subjects

- `thread@first-prototype@protocol-transport`.
- egui's `Context::request_repaint`, callable from any thread (quoted in
  `argument@first-prototype@arg-26`).
- T5 of the milestone document's premortem: a command from the socket arriving while a group of the
  GUI is open, `begin` without its `end`, joins that group.

## Fails alone on

- A command that succeeds through `rootpoint-headless` and fails through the socket.

## Premises that expire

- The socket is native only. Guard: the browser build has no transport, per
  `thread@first-prototype@protocol-transport`, and `spec@first-prototype@browser-build` does not
  build this listener for the wasm target.

## Implementation sequence

1. **An echo socket.** The GUI replies `ok` to every line without executing it. Fails alone on: the
   socket and the wakeup.
2. **Execution.** Commands run on the GUI's session. Claims 1 to 3.

## Decided design

The socket's thread parses each line and sends the command to the GUI's thread over a channel,
then waits for the reply; the GUI drains the channel at the start of each frame, so the session is
only touched by the GUI's thread (`thread@first-prototype@protocol-transport`).

Nearest rival: JSON-RPC; see "Losing alternatives" of the milestone document.

## Harvest

Judged by the entry tests of `primer@design-heads`: the socket listener and its threads, into the
design home of the app crate's Component, citing the protocol crate's head on the line grammar.
