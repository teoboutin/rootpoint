# Slice: Driving the running GUI by text commands

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- `rootpoint --control <path>`: the native GUI listens on a Unix domain socket at that path, reads
  one command per line from each connection, executes it on the GUI's session between frames, and
  writes one reply line.
- The binary `rootpoint-ctl <path> <command>`, in the protocol crate: sends one line, prints the
  reply, and exits non-zero on `err`.

## Claims

1. **A command over the socket acts like one from the UI.** A command sent with `rootpoint-ctl`
   changes the document and the view, and `undo` from the UI undoes it. Test: an integration test
   that starts the app's control loop with no window, sends commands, and reads the session.
2. **The GUI wakes for a command.** A command sent while the GUI is idle is applied within one frame
   of its arrival, through eframe's `request_repaint` from the socket's thread. Test: the
   integration test of claim 1 times the reply.

## Audit subjects

- `thread@first-prototype@protocol-transport`.
- eframe's `request_repaint` from another thread (V, per `argument@first-prototype@arg-26`).

## Fails alone on

- A command that succeeds through `rootpoint-headless` and fails through the socket.

## Premises that expire

- The socket is native and Unix only. The browser build has no protocol, per
  `thread@first-prototype@protocol-transport`.

## Implementation sequence

1. **An echo socket.** The GUI replies `ok` to every line without executing it. Fails alone on: the
   socket and the wakeup.
2. **Execution.** Commands run on the GUI's session. Claims 1, 2.

## Decided design

The socket's thread parses each line and sends the command to the GUI's thread over a channel,
then waits for the reply; the GUI drains the channel at the start of each frame, so the session is
only touched by the GUI's thread (`thread@first-prototype@protocol-transport`).

## Harvest

Judged by the entry tests of `primer@design-heads`: the live transport, into the design home of the
protocol crate's Component with `thread@first-prototype@protocol-transport`.
