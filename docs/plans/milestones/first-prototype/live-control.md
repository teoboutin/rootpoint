# Slice: Driving the running GUI by text commands

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites, and those of this spec that cross slices. Where this spec and a design home disagree on a
point that "What changes in the record", under "Decided design", does not list, the design home
wins.

The slice's design audit is done: its record is the message of the commit "Apply the design audit
of the sixth slice, driving the running GUI" (`git log --grep='design audit'`). Its load-bearing
gap opened a design session, held at the audit, which converged; the plan reviews of the converged
design read its shapes against the code, and their repairs are in this spec. The slice resumes at
point 3 of `skill@knowledge-architect-planning@working-a-slice`; D6 of the milestone document,
put to the owner after the reviews, is ruled in O27. The session was assembled from the transcript
of the session that ran the audit, where the discussion begins at the owner's answer to Q33. Its
threads,
arguments, criteria, premortem and acceptance criterion are below, and the owner's messages in it,
O20 to O26, are quoted verbatim in their own section.

## Names

- **round 1 to round 4**: the rounds of the design session held at this slice's audit, each a
  message of the session and the owner's answer to it: round 1 the grounding, the criteria and the
  first threads, answered by O23; round 2 the finding on presenter mode, the premortem and the goal
  drafts, answered by O24; round 3 the corner button and the redrafted goal, answered by O25; round
  4 the window's inspection, answered by O26. "Round: the audit" names the message that put Q33.
  They are not the rounds of the milestone's own session, which the milestone's "Names" defines.
- **Q33**: what a socket command does to a gesture of the window still open, put to the owner at
  the audit. **Q34**: which rule grants the protocol's request for control, (i), (ii) or (iii) of
  `argument@first-prototype@arg-87`. **Q35**: whether to run a survey of prior art on sharing an
  application between a person and an automated controller.
- **the socket**: the listener of `--control <path>` and every connection to it, counted as one
  controller.
- **control**: the authority to edit, held by the window or by the socket.
- **the window under the socket's control**: the window while the socket holds control, as
  "Builds" describes it.

## Builds

- `rootpoint [--control <path>] [presentation.ron]`: with `--control`, the native GUI listens on a
  Unix domain socket at that path; a Windows build is not part of the prototype. At start, a path
  that exists and accepts a connection is another window's, and the GUI exits with an error that
  names it; a path that exists and refuses one is left from a run that ended, and is removed before
  the bind. The socket's file is removed at exit. The usage line of `path@rootpoint-app@src/main.rs`
  and of `path@rootpoint-app@README.md` say so.
- The listener, in the app's library, native only (`cfg(not(target_arch = "wasm32"))`, since the
  app crate joins the wasm gate in `spec@first-prototype@browser-build`):
  `rootpoint_app::control::Listener::bind(path, wake)`, where `wake` is a function the listener
  calls once for each line it receives. An accept thread serves each connection on a thread of its
  own; a connection thread reads one line at a time, sends it over a channel to the GUI's thread,
  and waits for the reply before it reads the next. `App::listen(listener)` gives the listener to
  the app, which keeps its receiver; `App::frame` drains the channel at its start, before anything
  else, so the session is only touched by the GUI's thread. `main` binds the listener in the
  closure that `eframe::run_native` calls with the `CreationContext`, with `wake` set to that
  context's `request_repaint`, which wakes the window from another thread, per
  `design@rootpoint-app@gui-toolkit`.
- The binary `rootpoint-ctl <path> <words>…`, in the protocol crate: joins the words after the path
  with single spaces into one line, sends it, prints the reply, and exits 0 on `ok` and on no reply,
  1 on `err`, and 2 when it cannot connect. A line holding a quoted text is given as one word, as
  `rootpoint-ctl /tmp/rp 'text n3 "two words"'`, and its doc comment says so. Its target reads
  `ROOTPOINT_CHECKOUT`, as `path@xtask@CLAUDE.md` requires. It is a thin `main` over a function of
  the protocol crate's library,
  `rootpoint_protocol::control::send(path: &Path, line: &str) -> io::Result<Option<String>>`:
  `Ok(Some(reply))`, `Ok(None)` when the listener closed the
  connection with no reply, `Err` when it could not connect. It is built on Unix only, so the wasm
  gate never sees it. The app's tests drive the listener through that function, since cargo sets
  `CARGO_BIN_EXE_<name>` for a test only for a binary of the test's own package: measured at the
  plan's review, a test that reads the variable of a dependency's binary fails to compile.
- A batch over one connection (`thread@first-prototype@socket-batch`): `rootpoint-ctl <socket>`
  with no words reads lines from its stdin until its end and sends them over one connection,
  printing each reply as it comes. The window runs them one by one, so each edit shows as it is
  made and a take-over falls between two lines. When the window refuses a line because the author
  took control, it writes that refusal once and closes the connection, so the batch's other lines
  are dropped; `rootpoint-ctl` prints the refusal and exits 1. `rootpoint-ctl` exits 1 too when a
  batch had any `err` reply.
- One rule for a line, in the protocol crate's library, shared by `rootpoint-headless` and the
  listener: `rootpoint_protocol::read_line(bytes: &[u8]) -> Option<Result<Command, Reply>>` gives
  `None` for a blank line or a comment, the `err` reply `rootpoint-headless` gives today for a line
  that is not UTF-8 or does not parse, and the command otherwise. `Session::execute` and
  `rootpoint-headless` use it in place of their own rules, so the replies stay as they are. A line
  with no command gets no reply: the listener writes nothing, and the client, which shuts down its
  side of the connection after its line, reads the end of the connection.
- A socket command runs as the window's action for it would: through `App::run`, per
  `design@rootpoint-app@actions-are-commands`, so the status line shows the reason of the last
  command that failed, the socket's included. The file Save writes, the field `App::path`, follows
  the socket's commands as it follows the window's: a `load` or a `save` that succeeds sets it, as
  `App::open_file` and `App::save_to` do, its path first made relative to the working directory
  when it is absolute, by `App::relative_if_absolute`; a `sample` or a `new` that succeeds clears
  it, as `App::open_sample` does, so that Save never writes the new document over the file last
  opened. The paths of `save`, `load` and `render` are relative to the window's working directory,
  not to that of `rootpoint-ctl`; an asset's path is relative to the directory of the presentation
  last saved or loaded, as `path@rootpoint-protocol@README.md` says; the doc comment of
  `rootpoint-ctl` says both. The queries `tree`, `view` and `render` run on the session without
  `App::run`, since they change nothing, so they neither change the status line nor make the next
  frame render the slide again; `render` writes its image into the window's working directory.
- The protocol's verbs `control` and `release`, parsed and printed by the protocol crate, each with
  its own arm in `Session::run`: a headless session has one controller, so `control` replies `ok`;
  `release` replies `err a group is open` while a group is open, in every host, and `ok` otherwise.
  `thread@first-prototype@control-commands`. A new public accessor,
  `Session::group_open() -> bool`, tells a host whether a group is open.
- Control in the window: the field `App::controller`, of the type
  `Controller { Window, Socket, TakenFromSocket }`,
  with the accessor `App::controller()`; `Window` at start
  (`thread@first-prototype@control-at-start`). Control belongs to the socket as a whole, every
  connection together. A socket command is executed as follows:
  - `tree`, `view` and `render` run whoever holds control
    (`thread@first-prototype@read-without-control`);
  - `control` gives control to the socket and replies `ok`, also when the socket holds it already,
    after ending the window's gestures as `App::end_gestures` does and clearing the press in
    progress (`App::press`, `App::edit_on_click`, `App::text_drag`), so a press begun before it
    completes nothing at its release: a stay in editing ends with its edits one undo step, a
    divider drag ends as at its release, and a grip drag is cancelled, as at a loss of the window's
    focus, since its edits are made only at its drop (`thread@first-prototype@control-passing`).
    A `control` after a take-over by the button is granted at once too, by D6 of the milestone
    document, ruled in O27;
  - `release` replies as `Session::run` does, and on `ok` gives control to the window;
  - every other command, the view commands included, runs while the socket holds control, and
    otherwise replies `err the window has control` while the window holds control from the start
    or from a `release`, and
    `err the author took control: stop, and wait for the author's instructions` after a take-over
    by the button, until the socket's next `control`. The app keeps which of the two holds, in
    `Controller`.
- The window under the socket's control (`thread@first-prototype@window-under-protocol-control`).
  It shows each change as it is made. Every action of `path@rootpoint-app@src/actions.rs` that
  changes the document returns at once, changing nothing, while the socket holds control; every
  view action runs. Each input of the window, by the table below; Esc in the editor, which ends a
  gesture, has none to end. The window draws no divider highlight, no grab, grabbing or resize
  cursor, no grip, no preview line and no drop hint. A banner over the toolbar reads "The text
  protocol has control" beside a button "Take control"; in presenter mode the same button sits in
  a corner, over the slide's texture and never in its image. The button closes with `end` a group
  the socket left open, when `Session::group_open` says one is, and gives control to the window
  (`thread@first-prototype@interrupted-client-group`).

| input of the window | while the socket holds control |
| --- | --- |
| a click on an area, the selected text area included | selects it (`select`), and never enters editing |
| Alt with an arrow; a link of the breadcrumb | selects, as now |
| a click on a row of the slide list | shows its slide |
| F5; the toolbar's "Present" | presents |
| in presenter mode: →, Space, Page Down, ←, Page Up, Esc | as now |
| the button "Take control" | takes control |
| the toolbar's "Open…", "Save", "Save as…", "Sample", "Undo", "Redo"; Ctrl+O, Ctrl+S, Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y | ignored |
| the slide list's "Add", "Remove", "Duplicate", and its drag of a row | ignored |
| the side panel's split buttons, "Remove", content, size, bold, italic, colour, alignments, font, image, fit and "Add an image…" | ignored |
| Shift with an arrow; Delete | ignored |
| Ctrl or Shift over the selection, and the click that splits; a press on a divider, its drag and its double-click; a press on the grip and its drag | ignored |
| typing, the IME, copy, cut, paste: none reach a text, since editing cannot start | ignored |

## Claims

Each claim's test is shown to fail against a wrong implementation as its last sentence says.

1. **A command over the socket acts like one from the UI.** After `control`, a command sent with
   `rootpoint_protocol::control::send` changes the document and the view as the window's action for
   it does, and, once the window has taken control back, `undo` from the UI undoes it. Test: an app
   integration test that binds a `Listener` in a scratch directory, gives it to an `App`, and runs
   `App::frame` through `Context::run_ui` in a loop on the test's thread while a client thread
   sends commands and the test reads the session. Fails when the drain is removed from
   `App::frame`: the client never gets its reply, and the test times out after 30 s, a bound the
   session chose, since a debug build renders the slide after each command.
2. **The GUI wakes for a command.** The listener calls `wake` once for each line, before the GUI's
   thread has read it, and the next `App::frame` applies it. Test: the integration test of claim 1,
   with a `wake` that counts its calls: after each command, the count rose by one before the reply
   came. Fails when `wake` is not called: the count stays at 0.
3. **The scripts of the protocol run through the socket.** After `control`, every script of
   `path@rootpoint-protocol@tests/scripts/` gives through the socket, one connection per line by
   `rootpoint_protocol::control::send`, the replies a `Session` gives it by `Session::execute` in
   the test's own process, the function `rootpoint-headless` runs. Each script runs from an empty
   session in a scratch directory of its own, with the window and the reference session each given
   a `Files` that resolves paths under its directory; `ids-readdress.txt` runs after `ids-build.txt`
   in the same directory, as it loads the file that one saves; `golden.txt` finds
   `talk/two-colours.png` written there first, as `path@rootpoint-protocol@tests/files.rs` writes
   it. Test: an app integration test, with `image` as a dev-dependency of the app crate for the
   fixture. Fails when the socket drops a line, or a line with no command gets a reply: the reply
   lists differ.
4. **`rootpoint-ctl` prints the reply and exits by it.** It prints the reply line, exits 0 on `ok`
   and on no reply, 1 on `err`, 2 when it cannot connect. Test: a protocol integration test runs the
   binary against a listener the test holds, which answers `ok`, `err` and nothing, and once with no
   listener. Fails when the exit codes are swapped: the assertion on each code fails.
5. **Control passes as "Builds" says.** The window holds control at start; a socket edit without
   control replies `err the window has control` and changes nothing, and after a take-over by the
   button `err the author took control: stop, and wait for the author's instructions`, until the
   socket's next `control`, which is granted; `tree`, `view` and `render`
   are answered without control; `control` during a stay in editing ends the stay, its edits one
   undo step, during a divider drag ends the drag as at its release, and during a grip drag cancels
   it; `release` with a group open replies `err a group is open`; the button closes the socket's
   open group with `end`, so one undo reverts its edits. A stay whose only edits are an IME
   composition is outside the claim: it leaves an undo step that changes nothing, the defect
   `issue@rootpoint-app@abandoned-composition-undo-step`. Test: an app integration test per case.
   Fails, for the case T5 watches, when the socket's command runs inside the window's group: a stay,
   then `control` and a socket `text` on another area, then the take-over and one `undo`, which
   must revert the socket's edit alone.
6. **While the socket holds control, no window input runs a command but a view command and the
   take-over.** Test, two parts. Every action of `path@rootpoint-app@src/actions.rs` that changes
   the document, called as `path@rootpoint-app@tests/actions.rs` calls them, leaves the document and
   the undo stack as they were while the socket holds control, and each view action changes the view
   as with the window in control. Each pointer gesture of `path@rootpoint-app@tests/gestures.rs`,
   and the entry into editing of `path@rootpoint-app@tests/editing.rs`, driven through `App::frame`
   with the socket in control, changes nothing, and the frame draws no gesture mark. Fails against
   the app before the guard is built: every edit applies.
7. **`control` and `release` run headless.** `control` replies `ok` from `rootpoint-headless`;
   `release` replies `ok`, and `err a group is open` between a `begin` and its `end`; a script
   holding them gives the same document as the script without them. Test: a protocol integration
   test, and both verbs join the round-trip list of `path@rootpoint-protocol@tests/commands.rs`.
   Fails when either verb falls to the catch-all arm of `Session::run`: the reply is "applies no
   operation".
8. **The take-over button works in presenter mode, and keeps the slide as the editor's.** With the
   socket in control, presenter mode places the button where the last frame recorded it,
   `App::take_control_rect()`; a synthetic click there gives control to the window; the image
   presenter mode draws equals the editor's at the same size, as
   `path@rootpoint-app@tests/presenter.rs` compares them; and Esc leaves presenter mode with the
   socket keeping control. Test: additions to that file. Fails when the button is not drawn: the
   click leaves control with the socket.
10. **A batch runs over one connection, and a take-over ends it with one refusal.** Lines piped
   into `rootpoint-ctl <socket>` each get their reply; after a take-over between two lines, the next
   line gets the refusal that says the author took control, the connection closes, and the lines
   after it reach no frame. Test: an app integration test sends a batch over one connection, takes
   control after its first reply, and finds one refusal and the document with the first line's edit
   alone; and a protocol integration test runs the binary with a batch on stdin against a listener
   the test holds. Fails when the connection stays open after the refusal: the next line gets a
   second refusal.
9. **An idle listener costs a divider drag no time beyond its recorded bound.** With a `Listener`
   given to the app and no line sent, the largest frame of the divider drag of
   `design@rootpoint-app@gui-toolkit` stays within 33 ms, AC2's bound, the recorded runs being 5.9
   ms and 6.05 ms. Instrument:
   `cargo test --release -p rootpoint-app --test presenter -- --ignored --nocapture`, run with and without the listener; both times are reported in the step's commit.

## Audit subjects

- `thread@first-prototype@protocol-transport`.
- egui's `Context::request_repaint`, callable from any thread: egui 0.37.0's `context.rs` says
  the UI thread wakes "provided the egui integration has set that up via
  [`Self::set_request_repaint_callback`] (this will work on `eframe`)".
- T5 of the milestone document's premortem: a command from the socket arriving while a group of the
  GUI is open, `begin` without its `end`, joins that group. The decided design below prevents its
  firing condition, `thread@first-prototype@one-controller-at-a-time`.
- `issue@rootpoint-protocol@save-mixes-relative-and-absolute`: a protocol client other than the
  window can produce the mix, and the socket does not, since its `load` and `save` take the
  window's paths, per "Builds". The issue stays open for headless scripts.
- `issue@rootpoint-app@abandoned-composition-undo-step`: `control` ends a stay through
  `App::stop_editing`, the site the issue names; claim 5 leaves the case out, and the issue stays
  open.
- `issue@rootpoint-app@alt-arrows-follow-the-tree`, whose re-entry is the design audit of the next
  slice that touches the app's keys: this slice decides which keys act while the socket holds
  control, and changes what none of them does; on Q37 the owner ruled in O27 that this does not
  meet the re-entry.
- `design@rootpoint-app@gesture-groups`, `design@rootpoint-protocol@view-state`,
  `design@rootpoint-app@editor-overlay`, `design@rootpoint-app@edit-entry`,
  `design@rootpoint-app@selection-actions`, `design@rootpoint-app@divider-drag` and
  `design@rootpoint-app@swap-gesture`, which the decided design meets or rewrites as "What changes
  in the record" says.

## Fails alone on

- A command that succeeds through `rootpoint-headless` and fails through the socket while the
  socket holds control.
- A window input that changes the document while the socket holds control.

## Premises that expire

- The socket is native only. Guard: the listener is built under
  `cfg(not(target_arch = "wasm32"))`, so the wasm gate, which builds the app crate from
  `spec@first-prototype@browser-build`, fails if a part of it reaches the browser build; the
  browser build has no socket, and its window holds control always. That spec's audit subjects
  name it.
- Control belongs to the socket as a whole, which holds while one socket is the one other
  controller. A design that widens the lock, to several clients each with control of their own or
  to locks per slide or per node, ends it. Guard: the app's design head that records
  `thread@first-prototype@one-controller-at-a-time` states the clause, so a design that widens the
  lock reverses that head under `skill@knowledge-architect-decision-recording`; the milestone
  document's "Later consequences" carries the owner's view on a survey for that design.

## Implementation sequence

1. **An echo socket.** The listener, its wake-up and `rootpoint_protocol::control::send`; the GUI
   replies `ok` to every line without executing it. Claims 2 and 9, on echoed lines. Fails alone
   on: the socket and the wake-up.
2. **The line rule and the verbs `control` and `release`**, in the protocol crate: `read_line`,
   used by `Session::execute` and `rootpoint-headless`; the verbs parsed, printed and run, with
   their arms in `Session::run`, `Session::group_open`, and `rootpoint-ctl`. Claims 4 and 7, and the
   protocol's existing tests unchanged. Fails alone on: the grammar. The commit that adds
   `control::send` brings the line of `path@rootpoint-protocol@CLAUDE.md` that says only
   `rootpoint-headless` touches the file system to what is then true: `control::send` and
   `rootpoint-ctl` open a socket at a path.
3. **Execution, with control.** Socket commands run on the GUI's session, by the rules of control,
   the command line of `rootpoint` included. Claims 1, 2, 3 and 5.
4. **The window under the socket's control.** The guard on the actions, the table of inputs, the
   marks, the banner and the button. Claims 6 and 8.

5. **A batch over one connection**, decided during the owner's trial of the slice: `rootpoint-ctl`
   reading its stdin, and the connection closed after a take-over's refusal. Claim 10.

The echo socket before the verbs: the verbs have nothing to carry without a transport. The verbs
before execution: the rules of control take them as input. Execution before the window under the
socket's control: its tests send socket commands that hold control.

## Criteria

Labelled K1 to K9 in the discussion.

### The protocol can always reach control `##c13-protocol-reaches-control`

K1. Binding. Source: `goal@rootpoint-protocol@commands-drive-everything`, every change reachable by
command in the running editor too. Met by `thread@first-prototype@control-commands` and
`thread@first-prototype@control-passing`, whose `control` is granted at once.

### The window shows the protocol's changes as they are made `##c14-changes-shown-live`

K2. Binding. Source: `goal@rootpoint-app@live-agent-edits`. Met by
`thread@first-prototype@window-under-protocol-control` and
`thread@first-prototype@read-without-control`; the author can also look at any slide meanwhile.

### The author takes control, and does everything, without typing a command `##c15-take-control-by-hand`

K3. Binding. Source: `goal@rootpoint-app@direct-manipulation`. Met by the button of
`thread@first-prototype@window-under-protocol-control`, in the editor and in presenter mode.

### Every control command runs headless `##c16-control-headless`

K4. Binding. Source: `goal@rootpoint@headless-core`. Met by
`thread@first-prototype@control-commands`: both verbs reply `ok` headless.

### No command of one controller joins the other's undo group `##c17-no-shared-group`

K5. Binding, as a presumption. Source: `design@rootpoint-protocol@gesture-undo` and T5. Met by
`thread@first-prototype@one-controller-at-a-time`, `thread@first-prototype@control-passing` and
`thread@first-prototype@interrupted-client-group`; view commands join no group, and the protocol's
view commands need control, so its `select` cannot end a stay of the window.

### One reply line per command, and no line the client did not ask for `##c18-one-reply-line`

K6. Binding, as a presumption. Source: `design@rootpoint-protocol@protocol-transport`. Met: control
is asked for by commands, and nothing is pushed.

### No reconciling of two changes made at once to the same part `##c19-no-reconciling`

K7. Binding. Source: `goal@rootpoint@minimal-content-set`, as reworded in the session. Met by
`thread@first-prototype@one-controller-at-a-time`, one lock over the whole document.

### The browser build behaves as it does now `##c20-browser-unchanged`

K8. Binding. Source: `goal@rootpoint-app@browser-window`. Met: the browser build has no socket, so
the window holds control there always.

### Simple: few new verbs, and few new states in the app `##c21-simple`

K9. Weighed. Source: the owner's word in O20, "it is important to keep things simple", measured
as the count of new verbs and of new states. Met: two verbs, one state of the app, which
controller holds control, and one banner with its button.

## Threads

### What a command of the socket does to a gesture of the window still open `##command-during-gesture`

Proposed by the session at the audit and put to the owner as Q33, with three shapes and a default:

- (a), the default: a socket command ends the window's gestures first, as each of the window's
  edits does, `tree`, `view` and `render` excepted. Its cost, in the audit's words: "an agent's
  edit ends the author's stay in editing, and the author clicks the area again to go on".
- (b): it closes the window's group, and the gesture goes on in a new group opened after the
  command. Its cost: "a stay or a drag that an agent's edit interrupted is two undo steps, before
  and after it, where `design@rootpoint-app@gesture-groups` makes it one; and the app must place
  the caret again when the agent changed the edited text, and end a drag whose split the agent
  removed".
- (c): one group per client, in the session, the response T5 names, a reopening of
  `design@rootpoint-protocol@gesture-undo`. Its cost: "the session keeps a group per connection,
  and the window's group, closed after the agent's step, is pushed above it, so its undo reverts
  edits made before the agent's: where the agent changed the same text, that undo writes the
  author's earlier text over the agent's; where the agent removed the area, the step fails and the
  undo is refused".

The discriminating fact: a stay in editing keeps its group open from its first click to
its end, also while the window has lost the focus, per `design@rootpoint-app@gesture-groups` and
`App::frame` in `path@rootpoint-app@src/lib.rs`, so a socket command sent during a stay joins it,
which is T5's firing condition (`argument@first-prototype@arg-81`). The owner answered Q33 in O20
with another shape, `thread@first-prototype@one-controller-at-a-time`, which absorbs this thread
(`argument@first-prototype@arg-85`). Final state: superseded, closed with the thread that absorbed
it, by the owner's word in O23: "Overall I agree with your positions over the whole table". Harvest:
none of its own; its shapes are under "Losing alternatives".

### The window and the protocol are two controllers, and only one has the authority to edit at a time `##one-controller-at-a-time`

Proposed by the owner in O20: "text protocol and gui should be considered as two different
controllers, with only one of them being able to have control/authority to edit at a time, and the
ability for each one to "request" control from the other when it does not have it." Argued by the
session in round 1 (`argument@first-prototype@arg-85`). Final state: approved, by the owner's word
in O23: "Overall I agree with your positions over the whole table, I think they would make for
quite a convenient workflow." Shape: "Decided design". Harvest: a design head of the app crate's
Component, its rule on the owner's word, and on its argument the clause that control belongs to the
socket as a whole (`argument@first-prototype@arg-97`), which the premortem of round 2 added and
which closed on its argument; with the rewrites "What changes in the record" lists.

### The window takes control at once, and the protocol's `control` is granted at once, ending the window's gestures `##control-passing`

Proposed by the session in round 1 with three shapes for the protocol's request, (i) granted at
once, (ii) refused until the author gives control back, (iii) granted when no gesture of the window
is open, and the window's take-over immediate (`argument@first-prototype@arg-86`,
`argument@first-prototype@arg-87`). The owner answered Q34 in O23 with the workflow it serves,
"Manual adjustement (layout, wording...) is something that I would do when the agent has finished
a task and control was handed back to me with `release`, before giving further instructions. I
think (i) fits this workflow best." (`argument@first-prototype@arg-94`). Final state: approved, on
(i), by those words and the whole-table agreement of O23. A refused edit replies
`err the window has control` (`argument@first-prototype@arg-98`). Shape: "Decided design". After the
reviews, the
case of an agent asking again right after a take-over (`argument@first-prototype@arg-107`) was put
to the owner as D6; the owner chose in O27 to keep (i) in that case too, with a refusal that
tells the agent the author took control and to stop and wait for the author's instructions, over a
"Give control" button the author would click to hand back, and to record T7, which reopens that
button if agents misbehave too often. Harvest: judged on its own by the entry tests of
`primer@design-heads`, a head of its own where one passes, a comment at its code where none does.
Its rule (i) is the owner's choice in O23, on the owner's workflow, and its case after a take-over
the owner's choice in O27; the take-over's immediacy stands on `argument@first-prototype@arg-86`.
T6 and T7 guard it.

### The protocol asks for control with the verbs `control` and `release`, which run headless `##control-commands`

Proposed by the session in round 1 (`argument@first-prototype@arg-88`). Rivals: control taken
implicitly by the socket's first edit, and verbs the app's listener intercepts outside the grammar
(`argument@first-prototype@arg-89`). Final state: approved, by the whole-table agreement of O23.
The clause that control belongs to the socket as a whole is the app's, in
`thread@first-prototype@one-controller-at-a-time`. At the plan's review the session decided that
`release` replies `err a group is open` while a group is open, in every host, so that a script
gives the same replies in the window and headless (`argument@first-prototype@arg-106`): argued.
Shape: "Decided design". Harvest: a design head of the protocol crate's Component, since the verbs
are part of the grammar every program that drives rootpoint consumes, citing
`design@rootpoint-protocol@protocol-transport`; `rootpoint-ctl`, its command line and its exit
codes, and `rootpoint_protocol::control::send`, {J}.

### The protocol's queries need no control; its view commands do; the window's view inputs never do `##read-without-control`

Proposed by the session in round 1, for the protocol's queries (`argument@first-prototype@arg-90`);
reshaped in round 4 with the window's view inputs and the asymmetry
(`argument@first-prototype@arg-105`). Final state: approved, by the whole-table agreement of O23
and, reshaped, by O26: "I agree with the reshaped thread, proceed". Shape: "Decided design".
Harvest: judged on its own by the entry tests of `primer@design-heads`, a head of its own where one
passes, a comment at its code where none does; its ground is its argument.

### While the protocol holds control, the window takes only the inputs that change the view, and a button takes control back `##window-under-protocol-control`

Proposed by the session in round 1 as a window whose every input is ignored, after the owner's game
in O20 (`argument@first-prototype@arg-91`). Approved by O23, then reopened by the session in round
2 by a material finding: presenter mode has no toolbar for the banner's button, so the author could
not take control there without typing a command, against
`criterion@first-prototype@c15-take-control-by-hand` (`argument@first-prototype@arg-96`); its
default, Esc taking control, was replaced by the owner's shape in O24: "I'd propose a small
button/disclaimer in a corner to take it. I might want to exit presenter mode without taking
control. Also, a real presentation would not happen with control from an agent anyway, so adding a
discrete overlay in a corner is not a problem." (`argument@first-prototype@arg-100`,
`argument@first-prototype@arg-101`). The owner reopened it in O25: "is it needed to freeze all
window input ? ... any input that only changes the view but not the content/layout of the
presentation can stay useable." (`argument@first-prototype@arg-103`), and the session reshaped it
(`argument@first-prototype@arg-104`, `argument@first-prototype@arg-105`). Final state: approved, by
O26: "I agree with the reshaped thread, proceed". Shape: "Decided design". Harvest: judged on its
own by the entry tests of `primer@design-heads`, a head of its own where one passes, a comment at
its code where none does; the
corner button and the inspection of the slides are the owner's proposals in O24 and O25, and Esc
taking no control in presenter mode is the owner's word in O24; the rest stands on its argument.
With the rewrites "What changes in the record" lists.

### The window holds control when it starts `##control-at-start`

Proposed by the session in round 1 (`argument@first-prototype@arg-92`). Final state: approved, by
the whole-table agreement of O23. Harvest: judged on its own by the entry tests of
`primer@design-heads`, a head of its own where one passes, a comment at its code where none does.

### Taking control closes the group the protocol left open with `end` `##interrupted-client-group`

Proposed by the session in round 1 (`argument@first-prototype@arg-93`), for the take-over by the
window. Final state: approved, by the whole-table agreement of O23. At the assembly the session
applied the same rule to `release`; at the plan's review it replaced that clause by a `release`
refused while a group is open (`argument@first-prototype@arg-106`), argued. Harvest: judged on its
own by the entry tests of `primer@design-heads`, a head of its own where one passes, a comment at
its code where none does.

### A batch of lines goes over one connection, and a take-over ends it with one refusal `##socket-batch`

Proposed by the owner during the trial of the built slice, in O30: "maybe the "Take control"
button should just consume all pending commands at once, instead of processing them one by one and
sending the message multiple times. Restricting agents to one edit at a time seem like a less
efficient method." Shaped by the session in its answer (`argument@first-prototype@arg-110`): the
batch over one connection, the refusal written once and the connection closed, the window still
running the lines one by one. Final state: approved, by the owner's word in O31: "Build the batch in
this slice". Shape: "Builds" and "Decided design". Harvest: judged on its own by the entry tests of
`primer@design-heads`; it changes the command line of `rootpoint-ctl` and what a connection to the
socket does after a take-over, which other programs consume.

## Arguments

### A socket command during a stay joins its group; holding the command fails the goal `##arg-81`

Round: the audit. Assistant, the message that put Q33. "I ruled out holding the agent's command
until your group closes. The agent would get no reply for as long as you keep editing, and its edit
would not show as it happens, which goes against `goal@rootpoint-app@live-agent-edits`." Bears on
`thread@first-prototype@command-during-gesture`.

### Ending the window's gestures first is the window's own rule `##arg-82`

Round: the audit. Assistant, the message that put Q33. "I recommend (a). It is the rule the window
already applies to its own actions, so the slice's claim that a socket command acts like one from
the UI holds as the code stands, and it adds the least code." Bears on
`thread@first-prototype@command-during-gesture`.

### The window is shared with an agent the owner directs, not edited together `##arg-83`

Round: the owner's answer to Q33. Owner, O20. "Collab editing is not a goal, and neither is
collaborative editing between me and an agent or other user of the text protocol. My goal with that
feature (outside of testing and making demos) is sharing the open executable with an agent that is
performing changes that I ask and me watching the result and being able to interrupt or
occasionally adjust the look myself." "activating the text protocol freezes all gui input (and it
also enables a synthetic cursor, as the text protocol allows scripted cursor control). But I had no
need of "sharing" it with myself". Bears on `thread@first-prototype@one-controller-at-a-time`,
`thread@first-prototype@window-under-protocol-control`, the goals
`goal@rootpoint-app@live-agent-edits` and `goal@rootpoint@minimal-content-set`.

### The owner's addition is design and goal, to be discussed before the goal is written `##arg-84`

Round: before round 1. Owner, O21 and O22: "my words here were a significant design addition to
the previously recorded aims. It might need adding the goal I told to be recorded in project goals
too." "the grounding and reflection on design will help us give a better shape to the goal I
stated". Bears on the order of the session: the discussion first, the goals after.

### One controller at a time shares no group, and costs less than a group per client `##arg-85`

Round: round 1. Assistant. "It meets K5 by construction. When control passes, the side that loses
it has its open gestures or groups closed, so no group is ever shared." "It meets K7 by
construction." "It costs less than shape (c), one group per client, which needed per-connection undo
state in the session." "Under it, "a command during a gesture" happens only at a hand-over, and the
hand-over says what happens." Bears on `thread@first-prototype@one-controller-at-a-time`,
`thread@first-prototype@command-during-gesture`.

### An interrupt that needs the agent's consent is not one `##arg-86`

Round: round 1. Assistant. "Taking control back in the window is immediate, one click on a "Take
control" button in a banner. "Being able to interrupt" requires it, and an interrupt that needed the
agent's consent would not be one." Bears on `thread@first-prototype@control-passing`.

### The costs of the three rules for the protocol's request `##arg-87`

Round: round 1. Assistant. (i) "Cost: if you are mid-adjust, the agent's request ends your stay."
(ii) "Cost: every task you ask of the agent needs a click from you first, or the agent stalls and
has to tell you." (iii) "Cost: a stay can stay open for minutes while you are in the terminal, so
the agent sees refusals for no visible reason." "I default to (i). ... Nothing is lost, because your
ended stay is one undo step, kept." Bears on `thread@first-prototype@control-passing`.

### `release` frees the window with no click, and both verbs run headless `##arg-88`

Round: round 1. Assistant. "`release` gives it back, so the window unfreezes when the agent is done,
with no click from you." "Headless, there is one controller, and both reply `ok`. That meets K4, and
a script that drives the window also runs headless unchanged." Bears on
`thread@first-prototype@control-commands`.

### Implicit control and intercepted verbs lose `##arg-89`

Round: round 1. Assistant. Implicit control: "It saves two verbs (K9), but it loses on two points:
The window would stay frozen after the agent finishes until you click, since nothing says it is
done. An agent cannot tell, by command, whether it holds control." Intercepted verbs: "Ruled out:
it makes a second grammar beside the one `design@rootpoint-protocol@protocol-transport` calls the
crate's interface to every program that drives rootpoint." Bears on
`thread@first-prototype@control-commands`.

### The protocol's queries join no group `##arg-90`

Round: round 1. Assistant. "They apply no operation and join no group, so they cannot break K5. An
agent can then look at what you adjusted before it asks for control." "View commands, such as
`select` and `present`, count as edits here, since they change the session's view, which the window
shows." Bears on `thread@first-prototype@read-without-control`.

### No synthetic cursor, and the banner is a mark of the editor `##arg-91`

Round: round 1. Assistant. "No synthetic cursor. The protocol has no pointer commands, and what the
agent changes already shows: the selection outline follows its `select`." "The banner is a mark of
the editor, never drawn into the slide, per `design@rootpoint-app@editor-overlay`." Bears on
`thread@first-prototype@window-under-protocol-control`.

### The protocol in control at start would freeze a window just opened `##arg-92`

Round: round 1. Assistant. "The rival, the protocol holding control at start, would freeze a window
you just opened." Bears on `thread@first-prototype@control-at-start`.

### `end` keeps what the author watched, `cancel` erases it `##arg-93`

Round: round 1. Assistant. "closes that group with `end`, not `cancel`. Its edits stay visible as
one
undo step, so one Ctrl+Z reverts them, while `cancel` would erase what you just watched happen."
Bears on `thread@first-prototype@interrupted-client-group`.

### The owner takes control to interrupt, and adjusts after `release` `##arg-94`

Round: round 1, the owner's answer. Owner, O23, Q34. "I expect taking control being a mean to
interrupt the agent to ask a question or add/change instructions I gave, or correct a mistake early
before it propagates. Manual adjustement (layout, wording...) is something that I would do when the
agent has finished a task and control was handed back to me with `release`, before giving further
instructions. I think (i) fits this workflow best." Bears on
`thread@first-prototype@control-passing`.

### In the owner's view, no similar use case exists; a survey would be worth it for a wider lock `##arg-95`

Round: round 1, the owner's answer. Owner, O23, Q35. "There's not really a similar use case IMO: it
is a bit similar to collaborative editing, except that one of the collaborators has authority to
interrupt the other unilaterally, and that the editing is fully locked by one controller rather
than simultaneous and merged "in the background". I think it would become worth to run one if we
start expanding the scope of this locking feature later (collaborative editing being reopened
later, per slide locking, or other proposals that would widen the scope)." Bears on the survey,
declined, and on "Premises that expire".

### Presenter mode has no toolbar for the button `##arg-96`

Round: round 2. Assistant. "In presenter mode there is no toolbar, and nothing may be drawn over the
slide (`design@rootpoint-app@editor-overlay`, `goal@rootpoint-app@faithful-slide-view`). So while an
agent presents, as in a demo, you could not take control without typing a command, which fails
K3." Default: "Esc takes control". Bears on `thread@first-prototype@window-under-protocol-control`.

### Control by connection would end at every command of `rootpoint-ctl` `##arg-97`

Round: round 2, the premortem. Assistant. "`rootpoint-ctl` opens one connection per command, so if
control belonged to a connection, it would end at every command." "control belongs to the socket as
a whole, every connection together". Bears on `thread@first-prototype@control-commands`.

### Every refused edit says why `##arg-98`

Round: round 2, the premortem. Assistant. "every refused edit replies `err the window has control`,
so the agent can stop and tell you." Bears on `thread@first-prototype@control-passing`.

### The premortem's remaining causes and their verdicts `##arg-99`

Round: round 2, the premortem. Assistant. A crashed agent: "Esc or the button always takes control
back (K3), and closes the group with `end`." A frozen widget that still issues a command: a claim
of the slice. T6: "*Fires when:* you report that an agent's `control` ended a stay in editing or a
drag you had not finished, more than once in real use. *Response:* reopen #control-passing toward
(iii)". AC7: "Only the built code can show this". Bears on
`thread@first-prototype@control-passing`, `thread@first-prototype@one-controller-at-a-time`,
`thread@first-prototype@interrupted-client-group`,
`thread@first-prototype@window-under-protocol-control`.

### A corner button in presenter mode, and Esc that leaves without taking control `##arg-100`

Round: round 2, the owner's answer. Owner, O24. "I don't like Esc taking control back when in
presenter mode. Instead, I'd propose a small button/disclaimer in a corner to take it. I might want
to exit presenter mode without taking control. Also, a real presentation would not happen with
control from an agent anyway, so adding a discrete overlay in a corner is not a problem." Bears on
`thread@first-prototype@window-under-protocol-control`.

### The corner button keeps the slide's image as the editor's, and Esc keeps one meaning `##arg-101`

Round: round 3. Assistant. "It never goes into the slide's image, so presenter mode's image still
equals the editor's, the measure of `goal@rootpoint-app@faithful-slide-view`." "it forbids editor
marks *in* the slide image, and the editor already draws its marks over the texture." "the button
is the one way to take control, in both views, and Esc keeps its one meaning, "leave"." Bears on
`thread@first-prototype@window-under-protocol-control`.

### What stays out of scope is reconciling two changes at the same place `##arg-102`

Round: round 2 and 3. Owner, O24: "What I want to keep out of scope is having to resolve the
potential conflict of two edits at the same place (which is the hard thing AFAIK, but maybe it is
easier than I believe). Under this, collaborative editing using per slide or per node locks is not
fully out of scope. The advantage here is that our slide layout method gives clear hierarchy of
what content and changes can be applied without any risk of conflicts." "it is a bit weird to ship
it while stating outright that collaborative editing is out of scope." Assistant, round 3: "Locks
per slide or per node are then outside the exclusion. They need no reconciling, since one editor
holds each part." "#one-controller-at-a-time meets it with one lock over the whole document." Bears
on `criterion@first-prototype@c19-no-reconciling` and the goal
`goal@rootpoint@minimal-content-set`.

### The author inspects the slides while the agent edits `##arg-103`

Round: round 3, the owner's answer. Owner, O25. "for me to observe changes an agent is doing, maybe
I need to be able to "inspect" as much as I want the slides, no ? After all, the text protocol is
also able to inspect without control. Under this condition, any input that only changes the view
but not the content/layout of the presentation can stay useable. Is it needed from the text
protocol to have a specific slide selected and rendered on gui to apply edit commands to it ?"
Bears on `thread@first-prototype@window-under-protocol-control`,
`thread@first-prototype@read-without-control`.

### Editing commands never read the view `##arg-104`

Round: round 4. Assistant, read in `path@rootpoint-protocol@src/session.rs`. "Editing commands never
read the view. Every editing command names its slide, node or asset by id, per
`design@rootpoint-protocol@protocol-addressing`" "After an edit, the session prunes its view
(`Session::apply_op`)." "Save also writes the document's asset paths through `apply`
(`design@rootpoint-protocol@save-updates-sources`), so it counts as an edit" "While frozen, the
window shows no divider highlight, no resize cursor and no grip. They would offer gestures that do
nothing." "View commands join no group, per `design@rootpoint-protocol@gesture-undo`." Bears on
`thread@first-prototype@window-under-protocol-control`.

### One shared view, and the asymmetry of view commands `##arg-105`

Round: round 4. Assistant. "When the agent sends `select` or `present`, your window follows it,
which
is what a demo needs. When you click, the agent's `view` reply changes, which costs it nothing,
since its edits name ids." The rival "brings back the rival `design@rootpoint-protocol@view-state`
argued against, a view the GUI alone holds, which no headless test reaches. It also adds a second
view to reconcile at every hand-over." "An agent's `select` while you hold control would move what
you are looking at. Worse, a
`select` of another node ends your stay in editing, since editing ends when its area is no longer
the selection (`App::editor` in `path@rootpoint-app@src/lib.rs`). That closes your group, an
effect on your edits. The window's view inputs act on nothing of the agent's." Bears on
`thread@first-prototype@window-under-protocol-control`,
`thread@first-prototype@read-without-control`.

### `release` refused while a group is open keeps the two hosts alike `##arg-106`

Round: the plan's review. Assistant, on the design-conformance review's finding that `release`
closing the socket's group in the window, and changing nothing headless, gives a script `begin`, an
edit, `release`, `control`, `begin` the reply `ok` in the window and `err` headless, against the
replay argument of `design@rootpoint-protocol@gesture-undo` and the plan's "runs headless
unchanged". `release` closing the group in every host adds a closer to that head's list; a
`release` refused while a group is open, in every host, adds none, keeps the replies alike, and
costs the agent an `end` before its `release`. Bears on `thread@first-prototype@control-commands`,
`thread@first-prototype@interrupted-client-group`.

### Under (i), the agent's next `control` undoes the author's take-over `##arg-107`

Round: the plan's review. Assistant, on the design-conformance review's finding: under (i), an
agent that answers `err the window has control` with `control` has control back at once, so the
author's take-over holds only while the agent does not ask, which is the agent's consent that
`argument@first-prototype@arg-86` rules out, and `goal@rootpoint-app@live-agent-edits` asks that
the author "takes the work over". The owner chose (i) in O23 on a workflow where the agent asks for
control at the start of a task the author gave; the case of an agent asking again right after an
interrupt was not put to the owner. Default, D6: after a take-over by the button, `control` replies
`err the author took control` until the author gives control back with the banner's "Give
control"; at start and after `release`, `control` is granted at once, as (i). Its cost: one click
to hand back after an interrupt. Bears on `thread@first-prototype@control-passing`.

### The refusal tells the agent to stop and wait; a tripwire watches misbehaving agents `##arg-108`

Round: the owner's answer after the reviews. Owner, O27, D6. "when I take control from an agent,
the agent is supposed to receive the message that I took it (via replies to edit tentatives), and
the message should carry an instruction for the agent to stop and wait for further instructions.
In principle this should be enough, agents rarely misbehave like that nowadays." "I think we can
try out this shape first, with a tripwire if agents misbehave too often, that would reopen the
"Give control" button proposal." Bears on `thread@first-prototype@control-passing`.

### Sending one command at a time costs an agent a round trip per edit `##arg-109`

Round: the trial of the built slice. Owner, O30. "maybe the "Take control" button should just
consume all pending commands at once, instead of processing them one by one and sending the message
multiple times. Restricting agents to one edit at a time seem like a less efficient method." Bears
on `thread@first-prototype@socket-batch`.

### Nothing is queued in the window today; a batch over one connection, ended by the take-over `##arg-110`

Round: the trial of the built slice. Assistant. "Each `rootpoint-ctl` call is one connection
carrying one command, and it waits for its reply before the next call. So nothing was queued in the
window when you clicked. The repeated refusals came from my shell loop, which kept starting new
calls after the first refusal." The shape: one connection for a batch read from stdin; "When an edit
is refused because you took control, the window replies with the refusal once, closes that
connection, and drops the batch's remaining lines"; "A batch still runs line by line in the window,
so you see each edit as it happens, and the take-over cuts in between two lines." Bears on
`thread@first-prototype@socket-batch`.

## New names

```text
crates/rootpoint-protocol/  (crate rootpoint-protocol)
  Command::Control, Command::Release   the verbs `control` and `release`
  read_line                            the rule for one line: none, a command, or an `err` reply
  Session::group_open                  whether a group is open
  control::send                        Unix only: send one line to a socket, read the reply
  binary rootpoint-ctl                 src/bin/rootpoint-ctl.rs
crates/rootpoint-app/       (crate rootpoint-app)
  control::Listener, Listener::bind    native only: the socket's listener and its threads
  App::listen                          gives the app a listener
  Controller { Window, Socket,         who holds control, TakenFromSocket after a take-over;
    TakenFromSocket }                   the field App::controller,
                                       the accessor App::controller()
  App::take_control                    the button's action: the window takes control back
  App::take_control_rect               where the last frame placed the button "Take control"
```

## Decided design

**The transport.** Each connection's thread reads a line and sends it to the GUI's thread over a
channel, then waits for the reply; the GUI drains the channel at the start of each frame, so the
session is only touched by the GUI's thread (`thread@first-prototype@protocol-transport`).
Nearest rival: JSON-RPC; see "Losing alternatives" of the milestone document.

**One controller at a time** (`thread@first-prototype@one-controller-at-a-time`). The app holds
which
of two controllers, the window or the socket, has the authority to edit; the socket counts as one,
every connection together. When control passes, the side that loses it has its gestures or its
group closed, so no group is ever shared, which meets
`criterion@first-prototype@c17-no-shared-group` and prevents T5's firing condition; and one lock
over the whole document meets `criterion@first-prototype@c19-no-reconciling`. The rule is the
owner's, in O20. Nearest rival: `thread@first-prototype@command-during-gesture`'s (a), the socket's
commands ending the window's gestures one by one, with no notion of control. The owner answered Q33
with this rule in its place, and the session argued it, `argument@first-prototype@arg-85`. The
session's reading, at the assembly, of what (a) lacks: with no control, the agent's next command
runs whatever the author does, so the author cannot interrupt the agent, which the owner's intent
in O20 asks.

**Passing control** (`thread@first-prototype@control-passing`,
`thread@first-prototype@interrupted-client-group`, `thread@first-prototype@control-at-start`). The
window holds control at start. The button takes control at once, closing with `end` any group the
socket left open. The socket's `control` is granted at once, ending the window's gestures as
`App::end_gestures` does: a stay ends with its edits one undo step, a divider drag as at its
release, a grip drag cancelled, as at a loss of the window's focus. `release` gives control back,
and is refused while a group is open. An edit refused for want of control replies
`err the window has control`. Nearest rival for the request: (ii), refused until the author gives
control back; defeated by the owner's workflow in O23, where the author adjusts by hand after
`release` and so is rarely in a gesture when the agent asks. After a take-over by the button,
the socket's refused edits reply
`err the author took control: stop, and wait for the author's instructions`, so a well-behaved agent stops before it asks for control again; on the owner's
word in O27, that refusal over a "Give control" button the author clicks to hand back, which T7
reopens (`argument@first-prototype@arg-107`).

**The verbs** (`thread@first-prototype@control-commands`). `control` and `release` are verbs of the
protocol's grammar. Headless, `control` replies `ok`, since a headless session has one controller,
and `release` replies as it does in the window, so a script gives the same replies in both hosts.
Nearest rival: implicit control, taken by the socket's first edit; defeated because nothing would
tell the window the agent is done (`argument@first-prototype@arg-89`). The second point that
argument made, that an agent could not ask whether it holds control, is true of the chosen verbs
too, since `control` replies `ok` whether or not the socket held control, and decides nothing.

**What needs control** (`thread@first-prototype@read-without-control`). `tree`, `view` and `render`
run whoever holds control, since they apply no operation and join no group. The protocol's view
commands need control; the window's view inputs never do. The asymmetry: a `select` of another node
by the socket ends a stay of the window, since editing ends when its area is no longer the
selection, while the window's view inputs act on nothing of the socket's. The view stays one, the
session's, `design@rootpoint-protocol@view-state`. Nearest rival: a view the window keeps apart
while the socket holds control; defeated because it brings back the rival that head argues
against, a view no headless test reaches, and adds a second view to reconcile at each hand-over.

**The window under the socket's control**
(`thread@first-prototype@window-under-protocol-control`). As "Builds" and its table say. The guard
sits in the actions of `path@rootpoint-app@src/actions.rs`, which every input calls, so the input
layer needs no list of its own and a test calls the actions directly. The button sits in the
editor's banner and in a corner of presenter mode, over the texture, so the slide's image stays the
editor's, `design@rootpoint-app@editor-overlay`. Nearest rival: Esc taking control in presenter
mode; defeated by the owner's word in O24, since the owner wants to leave presenter mode without
taking control.

**A batch over one connection** (`thread@first-prototype@socket-batch`). A connection carries
any number of lines, and a take-over's refusal closes it, so an agent sends a task's edits in one
batch and a take-over stops the batch with one message. The window still runs one line at a time,
between its frames, so the author sees each edit and can cut in between two. Nearest rival: one
connection per command, as `rootpoint-ctl` had; defeated by the owner's weighing in O30, that one
edit at a time is less efficient for an agent.

**What changes in the record.** Each text below states a rule the decided design changes. The
harvest rewrites each, unless a step's commit says otherwise:

- `design@rootpoint-app@gesture-groups`: its last sentence, that a command of another client could
  join a group left open while the window is away, becomes false.
- `design@rootpoint-app@edit-entry`, `design@rootpoint-app@selection-actions`,
  `design@rootpoint-app@divider-drag` and `design@rootpoint-app@swap-gesture`: a click on the
  selected text area enters editing, Delete removes, the cursor is a resize cursor over a divider,
  and the selected area shows a grip, only while the window holds control. The owner approved the
  window under the socket's control in O26, which includes these limits;
  `design@rootpoint-app@edit-entry` rests on the
  owner's word, and the limit is within that approval.
- `design@rootpoint-app@editor-overlay`: its list of marks gains the banner and the button, and its
  marks of a gesture are not drawn while the socket holds control.
- `path@rootpoint-app@CLAUDE.md`: the line that `App::presenter` draws nothing over the slide
  becomes "nothing but the button 'Take control'"; and the line that an action not part of a
  gesture closes any group still open, which says more than `App::end_gestures` does (it ends the
  window's own gestures), is brought to the code and to the socket's group.
- The doc comment of `App::presenter` in `path@rootpoint-app@src/lib.rs` and the description of
  presenter mode in `path@rootpoint-app@README.md`, which say "with no mark over it".
- `path@rootpoint-app@README.md` and the doc comment of `path@rootpoint-app@src/main.rs`: the
  command line gains `--control`.
- `path@rootpoint-protocol@CLAUDE.md`: brought to `control::send` in step 2.
- The other texts that cite `design@rootpoint-app@gesture-groups`, which stay true:
  `design@rootpoint-app@swap-gesture`,
  `issue@rootpoint-app@abandoned-composition-undo-step`, and the comments of
  `path@rootpoint-app@src/editing.rs` and `path@rootpoint-app@src/lib.rs`; each is read again at
  the harvest.
- T5, `tripwire@rootpoint-protocol@client-joins-gui-group`, guards
  `design@rootpoint-protocol@gesture-undo`, which stands; its condition is prevented by the app, so
  the harvest puts it to the owner, to keep or to remove. It cites this spec twice, and
  `path@rootpoint-protocol@tests/common/mod.rs` cites it once; the commit that deletes this spec
  rewrites each, the tripwire's as the owner rules.

## Losing alternatives

- **The socket's commands ending the window's gestures, with no control**, Q33's (a), lost to
  `thread@first-prototype@one-controller-at-a-time`, on the owner's answer to Q33 and the session's
  argument for that thread (`argument@first-prototype@arg-83`, `argument@first-prototype@arg-85`);
  the session's reading of what it lacks is under "Decided design".
- **The window's gesture going on in a new group after a socket command**, Q33's (b), lost to the
  same thread: a gesture an agent interrupts is two undo steps, and the app must place the caret
  again on a text the agent changed, as the thread `thread@first-prototype@command-during-gesture`
  quotes the audit.
- **One group per client in the session**, Q33's (c), lost to the same thread: per-connection undo
  state (`argument@first-prototype@arg-85`), and an undo that writes the author's earlier text over
  the agent's or is refused, as the same thread quotes the audit. It reopens
  `design@rootpoint-protocol@gesture-undo`, and T5's response names it.
- **Holding a socket command until the window's group closes**, ruled out at the audit: the agent
  waits with no reply as long as a stay lasts, against `goal@rootpoint-app@live-agent-edits`
  (`argument@first-prototype@arg-81`).
- **The protocol's request refused until the author gives control back**, (ii), and **granted only
  when no gesture is open**, (iii), lost to (i) of `thread@first-prototype@control-passing`: a click
  before each task, or refusals for no visible reason (`argument@first-prototype@arg-87`,
  `argument@first-prototype@arg-94`). T6 brings (iii) back if (i) ends the owner's gestures in real
  use.
- **A "Give control" button, the author's, after a take-over**, D6's default, the socket's
  `control` refused until the author clicks it, lost to the refusal that tells the agent to stop
  and wait, on the owner's word in O27 (`argument@first-prototype@arg-107`). T7 brings it back if
  agents misbehave too often.
- **Implicit control, and verbs the listener intercepts**, lost to
  `thread@first-prototype@control-commands` (`argument@first-prototype@arg-89`).
- **Control held by one connection**, lost to `thread@first-prototype@one-controller-at-a-time`:
  `rootpoint-ctl` opens one connection per command (`argument@first-prototype@arg-97`).
- **`release` closing the socket's open group with `end`**, the session's clause at the assembly,
  lost to the refused `release`: headless, with no window, the group would close in one host only
  (`argument@first-prototype@arg-106`).
- **Every window input ignored**, lost to `thread@first-prototype@window-under-protocol-control`:
  the author inspects the slides while the agent edits, and editing commands read no view
  (`argument@first-prototype@arg-103`, `argument@first-prototype@arg-104`).
- **Esc taking control**, lost to the same thread: in presenter mode by the owner's word in O24
  (`argument@first-prototype@arg-100`); in the editor by the session's default of round 3, that
  Esc keeps one meaning, "leave" (`argument@first-prototype@arg-101`), within the thread the owner
  approved in O26.
- **A synthetic cursor**, as in the owner's game, lost to the same thread: the protocol has no
  pointer commands (`argument@first-prototype@arg-91`).
- **A view the window keeps apart while the socket holds control**, lost to
  `thread@first-prototype@read-without-control` (`argument@first-prototype@arg-105`).
- **The protocol holding control at start**, lost to `thread@first-prototype@control-at-start`
  (`argument@first-prototype@arg-92`).
- **`cancel` of the socket's open group at a take-over**, lost to
  `thread@first-prototype@interrupted-client-group` (`argument@first-prototype@arg-93`).

## Premortem

Each cause assumes the slice shipped and the feature failed. The owner ruled on T6 and AC7 in O25:
"Keep T6 and AC7." T7 is the owner's, proposed and ruled in O27, the answer to D6.

- **`rootpoint-ctl` opens one connection per command.** Stresses
  `thread@first-prototype@control-commands`. Converted into a clause of it: control belongs to the
  socket as a whole.
- **An agent loses control mid-task and does not understand why.** Stresses
  `thread@first-prototype@control-passing`. Survives into a clause: a refused edit replies
  `err the window has control`.
- **An agent crashes while holding control, with a group open.** Stresses
  `thread@first-prototype@interrupted-client-group`. Survives: the button always takes control
  back and closes the group with `end`.
- **A window input still runs a command while frozen.** Stresses
  `thread@first-prototype@window-under-protocol-control`. Survives into claim 6.
- **T6. An agent's `control` ends a stay or a drag the owner had not finished.** Stresses
  `thread@first-prototype@control-passing`. Recorded on the owner's word, at this slice's harvest,
  in the app crate's Component's tripwires home, guarding the head that records
  `thread@first-prototype@control-passing`. Fires when: the owner reports that an agent's
  `control` ended a stay in editing or a drag the owner had not finished, more than once in real
  use. Response: reopen the rule of the protocol's request toward (iii), granted only when no
  gesture of the window is open. Re-entry: the first real talk prepared with an agent.
- **T7. An agent takes control back after the author took it, without the author's word.**
  Stresses `thread@first-prototype@control-passing`, its case after a take-over, ruled in O27.
  Recorded on the owner's word in O27, at this slice's harvest, in the app crate's Component's
  tripwires home, guarding the same head as T6. Fires when: in real use, an agent answers a
  refusal that says the author took control with `control`, before the author gave it new
  instructions, more than once; the count is a default, the owner's to reset. Response: reopen the
  case after a take-over, toward a "Give control" button the author clicks to hand back. Re-entry:
  the first real talk prepared with an agent.
- **AC7. The hand-over is in the way of the workflow.** Stresses
  `thread@first-prototype@one-controller-at-a-time` and `thread@first-prototype@control-passing`.
  Becomes `acceptance@first-prototype@hand-over-convenient`.

## Acceptance criteria

### AC7. The owner's hand-over workflow with an agent works as the owner expects `##hand-over-convenient`

Guards `thread@first-prototype@one-controller-at-a-time` and
`thread@first-prototype@control-passing`. Judged at the end of this slice, before its merge, by the
owner, in a session where an agent builds and changes a presentation in the running window. Fires
when, in that session, the owner interrupts the agent with the button, gives new instructions, the
agent resumes with `control` and hands back with `release`, and the owner then finds the hand-over
in the way of that workflow. Response: reopen both decisions. Kept on the owner's word in O25.

## The owner's messages in the design session

Verbatim, in order. O20 answers Q33; O23 answers Q34, Q35 and round 1; O24 answers round 2; O25
round 3; O26 round 4; O27 answers D6, Q36 and Q37, put after the reviews.

**O20**

> Q33: here, I think it is important to keep things simple. Collab editing is not a goal, and neither is collaborative editing between me and an agent or other user of the text protocol. My goal with that feature (outside of testing and making demos) is sharing the open executable with an agent that is performing changes that I ask and me watching the result and being able to interrupt or occasionally adjust the look myself.
> My idea for this is that text protocol and gui should be considered as two different controllers, with only one of them being able to have control/authority to edit at a time, and the ability for each one to "request" control from the other when it does not have it. In one of my other projects with a similar feature (it is a small, mostly menu driven game), activating the text protocol freezes all gui input (and it also enables a synthetic cursor, as the text protocol allows scripted cursor control). But I had no need of "sharing" it with myself, it was used only for beta testing and demos, so I needed to act only as a spectator there.

**O21**

> wait a bit on that. I think we should hold a proper design discussion on the subject, since my words here were a significant design addition to the previously recorded aims. It might need adding the goal I told to be recorded in project goals too.

**O22**

> I think we should hold the design discussion first, because I believe the grounding and reflection on design will help us give a better shape to the goal I stated. We can carry it in this session, since you already hold quite a bit of context about the existing text protocol design and plans.

**O23**

> Q34: I expect taking control being a mean to interrupt the agent to ask a question or add/change instructions I gave, or correct a mistake early before it propagates. Manual adjustement (layout, wording...) is something that I would do when the agent has finished a task and control was handed back to me with `release`, before giving further instructions. I think (i) fits this workflow best.
> Q35: I don't think this is quite necessary. There's not really a similar use case IMO: it is a bit similar to collaborative editing, except that one of the collaborators has authority to interrupt the other unilaterally, and that the editing is fully locked by one controller rather than simultaneous and merged "in the background". I think it would become worth to run one if we start expanding the scope of this locking feature later (collaborative editing being reopened later, per slide locking, or other proposals that would widen the scope).
>
> Overall I agree with your positions over the whole table, I think they would make for quite a convenient workflow.

**O24**

> window-under-protocol-control: I don't like Esc taking control back when in presenter mode. Instead, I'd propose a small button/disclaimer in a corner to take it. I might want to exit presenter mode without taking control. Also, a real presentation would not happen with control from an agent anyway, so adding a discrete overlay in a corner is not a problem.
>
> live-agent-edits: goal wording approved.
> minimal-content-set: to make the goal clear, I think it is important to define collaborative editing. What I want to keep out of scope is having to resolve the potential conflict of two edits at the same place (which is the hard thing AFAIK, but maybe it is easier than I believe). Under this, collaborative editing using per slide or per node locks is not fully out of scope. The advantage here is that our slide layout method gives clear hierarchy of what content and changes can be applied without any risk of conflicts.
> The agent editing workflow I described is pretty much collaborative editing after all, so it is a bit weird to ship it while stating outright that collaborative editing is out of scope.

**O25**

> window-under-protocol-control: actually, is it needed to freeze all window input ? It's the example I mentionned earlier, but for me to observe changes an agent is doing, maybe I need to be able to "inspect" as much as I want the slides, no ? After all, the text protocol is also able to inspect without control. Under this condition, any input that only changes the view but not the content/layout of the presentation can stay useable. Is it needed from the text protocol to have a specific slide selected and rendered on gui to apply edit commands to it ? Or is it possible for me to keep the ability to freely inspect any slide while the agent sends editing commands to a specific one?
>
> minimal-content-set: wording approved.
>
> Keep T6 and AC7.
>
> The discussion is not fully closed though, I still want to rediscuss window-under-protocol-control.

**O26**

> I agree with the reshaped thread, proceed

**O27**, the owner's answer to D6, Q36 and Q37, put after the reviews of the plan.

> D6: IMO, when I take control from an agent, the agent is supposed to receive the message that I took it (via replies to edit tentatives), and the message should carry an instruction for the agent to stop and wait for further instructions. In principle this should be enough, agents rarely misbehave like that nowadays.
> I think we can try out this shape first, with a tripwire if agents misbehave too often, that would reopen the "Give control" button proposal.
>
> Q36: There is nothing asked of me here, right ?  Your defaults look right.
> Q37: no reentry here.

**O28 to O31**, the owner's messages during the trial of the built slice, AC7's.

**O28**

> I started rootpoint with the control socket as you asked, start creating anything, just to try it out.

**O29**

> Go on with the edits, write more slides with made up content, I'm now gonna try out inspecting slides while you are editing others, to see whether the live edits allow this without constantly bringing my view back to were you are editing

**O30**

> Inspection went perfectly fine. Maybe (for the future, not today), a notification system that tells me what the agent is doing would be nice. Simply printing the command the agent send, as they come.
> About sending commands one at a time: maybe the "Take control" button should just consume all pending commands at once, instead of processing them one by one and sending the message multiple times. Restricting agents to one edit at a time seem like a less efficient method.
> Is there anything else needed to live test ?

**O31**

> Build the batch in this slice, and go finish s15

## Harvest

Judged by the entry tests of `primer@design-heads`, under
`skill@knowledge-architect-decision-recording`. Each thread is judged on its own, as its item's
harvest line says: two statements are one head only where they answer one question and lose to the
same nearest rival, and each part states its own ground, the owner's where the item says so.

- `thread@first-prototype@socket-batch`, its command line in the protocol crate's design home and
  its connection's end at a take-over in the app's, by that item's harvest line.
- The socket listener and its threads, into the design home of the app crate's Component, citing
  `design@rootpoint-protocol@protocol-transport`.
- `thread@first-prototype@one-controller-at-a-time`, with its clause that control belongs to the
  socket as a whole, into the design home of the app crate's Component.
- `thread@first-prototype@control-passing`, `thread@first-prototype@read-without-control`,
  `thread@first-prototype@window-under-protocol-control`, `thread@first-prototype@control-at-start`
  and `thread@first-prototype@interrupted-client-group`, each by its item's harvest line, into the
  design home of the app crate's Component or a comment at its code.
- `thread@first-prototype@control-commands`, into the design home of the protocol crate's
  Component, citing `design@rootpoint-protocol@protocol-transport`; `rootpoint-ctl` and
  `rootpoint_protocol::control::send` by that item's harvest line.
- The losing alternatives, each judged by the tests of
  `skill@knowledge-architect-decision-recording@losing-alternatives`; those the session expects to
  meet a test: Q33's (c), which reopens `design@rootpoint-protocol@gesture-undo` and which T5's
  response names; (iii), toward which T6 reopens `thread@first-prototype@control-passing`; implicit
  control and verbs the listener intercepts, which would change the grammar; control held by one
  connection, which would change the socket's contract.
- Every text "What changes in the record" lists, under "Decided design".
- T6 and T7, in the tripwires home of the app crate's Component, on the owner's word, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- T5, `tripwire@rootpoint-protocol@client-joins-gui-group`, put to the owner: its condition is
  prevented by the app.
- The report on AC7, in the landing commits.
