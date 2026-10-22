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
  `path@xtask@CLAUDE.md` requires. It is a thin `main` over a function of the protocol crate's
  library, `rootpoint_protocol::control::send`, built on Unix only, so the wasm gate never sees it.
  The app's tests drive the listener through that function, since cargo gives a test the path of a
  binary of its own package only (`CARGO_BIN_EXE_<name>`).
- A line that is no command, blank or a comment, gets no reply, as in `rootpoint-headless`. The
  client shuts down its side of the connection after its line, and the listener closes the
  connection at its end, so `rootpoint-ctl` given such a line reads the end of the connection,
  prints nothing and exits 0. A line that is not UTF-8 gets the reply `rootpoint-headless` gives it.
- A socket command runs as the window's action for it would: through `App::run`, per
  `design@rootpoint-app@actions-are-commands`; and the file Save writes, `App::path`, follows the
  socket's commands as it follows the window's: a `load` or a `save` that succeeds sets it, as
  `App::open_file` and `App::save_to` do, its path first made relative to the working directory
  when it is absolute, by `App::relative_if_absolute`; a `sample` or a `new` that succeeds clears
  it, as `App::open_sample` does, so that Save never writes the new document over the file last
  opened. Paths are relative to the window's working directory, not to that of
  `rootpoint-ctl`, and its doc comment says so.
- What a socket command does to a gesture of the window still open, a drag or a stay in editing:
  open at the audit, `thread@first-prototype@command-during-gesture`; the slice waits for its
  ruling.

## Claims

1. **A command over the socket acts like one from the UI.** A command sent with `rootpoint-ctl`
   changes the document and the view, and `undo` from the UI undoes it. Test: an integration test
   that starts the app's control loop with no window, sends commands, and reads the session.
2. **The GUI wakes for a command.** A command sent while the GUI is idle is applied within one frame
   of its arrival, through egui's `request_repaint` from the socket's thread. Test: the integration
   test of claim 1 times the reply.
3. **The scripts of the protocol run through the socket.** Every script of
   `path@rootpoint-protocol@tests/scripts/` gives the same replies through the socket, one
   connection per line by `rootpoint_protocol::control::send`, as through `rootpoint-headless`,
   each run from an empty session in a scratch directory of its own; `ids-readdress.txt` runs after
   `ids-build.txt` in the same directory, as it loads the file that one saves. Test: an app
   integration test whose `Files` resolve paths under that directory.
4. **`rootpoint-ctl` prints the reply and exits by it.** It prints the reply line, exits 0 on `ok`
   and on no reply, and non-zero on `err` or when it cannot connect. Test: a protocol integration
   test runs the binary against a listener the test holds, which answers `ok`, `err` and nothing.

## Audit subjects

- `thread@first-prototype@protocol-transport`.
- egui's `Context::request_repaint`, callable from any thread (quoted in
  `argument@first-prototype@arg-26`).
- T5 of the milestone document's premortem: a command from the socket arriving while a group of the
  GUI is open, `begin` without its `end`, joins that group. The decided design below meets its
  firing condition, `thread@first-prototype@command-during-gesture`.
- `issue@rootpoint-protocol@save-mixes-relative-and-absolute`, which a protocol client other than
  the window can produce: the socket's `load` and `save` take the window's paths, per "Builds".

## Fails alone on

- A command that succeeds through `rootpoint-headless` and fails through the socket.

## Premises that expire

- The socket is native only. Guard: the browser build has no transport, per
  `thread@first-prototype@protocol-transport`, and `spec@first-prototype@browser-build` does not
  build this listener for the wasm target.

## Implementation sequence

1. **An echo socket.** The GUI replies `ok` to every line without executing it. Fails alone on: the
   socket and the wakeup.
2. **Execution.** Commands run on the GUI's session, with what they do to a gesture still open as
   `thread@first-prototype@command-during-gesture` closes. Claims 1 to 4.

## Threads

### What a command of the socket does to a gesture of the window still open `##command-during-gesture`

Open at the audit, put to the owner as Q33 with a default. Proposed by the session at the audit.
The discriminating fact: the decided design runs each socket command on the window's one session,
and a stay in editing keeps its group open from its first click to its end, also while the window
has lost the focus, per `design@rootpoint-app@gesture-groups` and `App::frame` in
`path@rootpoint-app@src/lib.rs`. So an agent's command sent while the author types in an area, or
after the author left the window for a terminal in the middle of a stay, joins the stay's group,
which is the firing condition of `tripwire@rootpoint-protocol@client-joins-gui-group`, T5: one
`undo` would revert the agent's edit with the author's typing. The window's own actions do not
meet it: each one that is no part of a gesture first ends every gesture, by `App::end_gestures` of
`path@rootpoint-app@src/actions.rs`. Three shapes meet every binding constraint:

- **(a), the default.** A socket command ends the window's gestures first, as each of the window's
  own actions does: a drag ends as at its release, its group closed with `end`; a stay ends as by
  Esc, its group closed. The commands that change neither the document nor the view, `tree`,
  `view` and `render`, run without ending them, since they join nothing. The agent's edit is then
  an undo step of its own, and T5's condition never occurs. Cost: an agent's edit ends the author's
  stay in editing, and the author clicks the area again to go on.
- **(b).** A socket command closes the window's group, and the gesture goes on in a new group
  opened after the command: the stay keeps its caret, the drag goes on. Cost: a stay or a drag
  that an agent's edit interrupted is two undo steps, before and after it, where
  `design@rootpoint-app@gesture-groups` makes it one; and the app must place the caret again when
  the agent changed the edited text, and end a drag whose split the agent removed.
- **(c).** One group per client, in the session: the response T5 names, a reopening of
  `design@rootpoint-protocol@gesture-undo`, whose ground is the owner's word. Cost: the session
  keeps a group per connection, and the window's group, closed after the agent's step, is pushed
  above it, so its undo reverts edits made before the agent's: where the agent changed the same
  text, that undo writes the author's earlier text over the agent's; where the agent removed the
  area, the step fails and the undo is refused.

Ruled out: holding a socket command until the window's group closes. A stay lasts as long as the
author edits, so the agent waits with no reply for that time, and its edit does not show as it is
made, against `goal@rootpoint-app@live-agent-edits`.

Why (a) is the default: it is the rule the window already applies to every action of its own, so
claim 1, a socket command acts like one from the UI, holds as the window's code stands, and it adds
the least code. Either (a) or (b) makes the last sentence of
`design@rootpoint-app@gesture-groups` false, that a command of another client could join a group
left open while the window is away, so the harvest rewrites that head, and puts T5 to the owner,
whose condition the app then prevents.

## Decided design

The socket's thread parses each line and sends the command to the GUI's thread over a channel,
then waits for the reply; the GUI drains the channel at the start of each frame, so the session is
only touched by the GUI's thread (`thread@first-prototype@protocol-transport`).

Nearest rival: JSON-RPC; see "Losing alternatives" of the milestone document.

## Harvest

Judged by the entry tests of `primer@design-heads`: the socket listener and its threads, into the
design home of the app crate's Component, citing the protocol crate's head on the line grammar.
