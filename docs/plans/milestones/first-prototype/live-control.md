# Slice: Driving the running GUI by text commands

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites, and those of this spec that cross slices. Where this spec and a design home disagree on a
point that "Decided design" below does not list as rewritten, the design home wins.

The design of how the window and the socket share the session was argued in a design session
held at this slice's design audit. It was assembled from the transcript of the session that ran the
audit, where the discussion begins at the owner's message that opens "Q33: here, I think it is
important to keep things simple". Its threads, arguments, criteria, premortem and acceptance
criterion are below, and the owner's messages in it, O20 to O26, are quoted verbatim in their own
section.

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
- The protocol's verbs `control` and `release`, parsed and printed by the protocol crate. A
  headless session has one controller, so `Session::run` replies `ok` to both and changes nothing.
  `thread@first-prototype@control-commands`.
- Control in the window: the app holds which controller has the authority to edit, the window or
  the socket, the window at start (`thread@first-prototype@control-at-start`). Control belongs to
  the socket as a whole, every connection together. A socket command is executed as follows:
  - `tree`, `view` and `render` run whoever holds control
    (`thread@first-prototype@read-without-control`);
  - `control` ends the window's gestures, as `App::end_gestures` does, gives control to the socket
    and replies `ok`, also when the socket holds it already
    (`thread@first-prototype@control-passing`);
  - `release` closes with `end` a group the socket opened and left open, gives control to the
    window and replies `ok`, also when the window holds it already;
  - every other command, the view commands included, runs while the socket holds control, and
    otherwise replies `err the window has control`.
- The window while the socket holds control (`thread@first-prototype@window-under-protocol-control`):
  - it shows each change as it is made;
  - the inputs that change only the view stay usable: a click on an area, which selects it and no
    more, a click on the selected text area included; Alt with an arrow; the breadcrumb; a click on
    a row of the slide list, which shows its slide; F5; and, in presenter mode, →, ←, Space, Page
    Up, Page Down, and Esc, which leaves presenter mode;
  - every input that changes the document is ignored: a split, by a button, by Shift and an arrow
    or at the pointer, and its preview line; a divider drag and a double-click on a divider; a grip
    drag; Delete; the slide list's drag and its buttons; the side panel's content, style and image
    controls; undo, redo, open, the sample and save; and entering text editing;
  - it draws no divider highlight, no resize cursor and no grip;
  - a banner over the toolbar reads "The text protocol has control" beside a button "Take
    control", and in presenter mode the same button sits in a corner, over the slide's texture and
    never in its image. The button closes with `end` a group the socket left open, and gives
    control to the window (`thread@first-prototype@interrupted-client-group`).

## Claims

1. **A command over the socket acts like one from the UI.** After `control`, a command sent with
   `rootpoint_protocol::control::send` changes the document and the view as the window's action for
   it does, and, once the window has taken control back, `undo` from the UI undoes it. Test: an
   integration test that starts the app's control loop with no window, sends commands, and reads
   the session.
2. **The GUI wakes for a command.** A command sent while the GUI is idle is applied within one frame
   of its arrival, through egui's `request_repaint` from the socket's thread. Test: the integration
   test of claim 1 times the reply.
3. **The scripts of the protocol run through the socket.** After `control`, every script of
   `path@rootpoint-protocol@tests/scripts/` gives the same replies through the socket, one
   connection per line by `rootpoint_protocol::control::send`, as through `rootpoint-headless`,
   each run from an empty session in a scratch directory of its own; `ids-readdress.txt` runs after
   `ids-build.txt` in the same directory, as it loads the file that one saves. Test: an app
   integration test whose `Files` resolve paths under that directory.
4. **`rootpoint-ctl` prints the reply and exits by it.** It prints the reply line, exits 0 on `ok`
   and on no reply, and non-zero on `err` or when it cannot connect. Test: a protocol integration
   test runs the binary against a listener the test holds, which answers `ok`, `err` and nothing.
5. **Control passes as "Builds" says.** The window holds control at start; a socket edit without
   control replies `err the window has control` and changes nothing; `tree`, `view` and `render`
   are answered without control; `control` during a stay in editing ends the stay, its edits one
   undo step, and during a divider drag ends the drag as at its release; `release` and the button
   close the socket's open group with `end`, so one undo reverts its edits. Test: an app
   integration test per case. Shown to fail against a wrong implementation by the case T5 watches:
   a stay, then `control` and a socket `text` on another area, then the window's take-over and one
   `undo`, which must revert the socket's edit alone; it fails when the socket's command runs
   inside the stay's group.
6. **While the socket holds control, no window input runs a command but a view command and the
   take-over.** Test: each action of `path@rootpoint-app@tests/actions.rs`, each gesture of
   `path@rootpoint-app@tests/gestures.rs` and the entry into editing of
   `path@rootpoint-app@tests/editing.rs`, driven through `App::frame` with the socket in control,
   leaves the document and the undo stack as they were; and each view input of "Builds" changes the
   view as it does with the window in control. Shown to fail by running it against the app before
   the freeze is built.
7. **`control` and `release` run headless.** Each replies `ok` from `rootpoint-headless`, and a
   script holding them gives the same document as the script without them. Test: a protocol
   integration test.
8. **Presenter mode under the socket's control shows the slide as the editor does.** The image
   presenter mode draws equals the editor's at the same size, as
   `path@rootpoint-app@tests/presenter.rs` compares it, with the button outside the image; Esc
   leaves presenter mode and the socket keeps control. Test: an addition to that file.

## Audit subjects

- `thread@first-prototype@protocol-transport`.
- egui's `Context::request_repaint`, callable from any thread (quoted in
  `argument@first-prototype@arg-26`).
- T5 of the milestone document's premortem: a command from the socket arriving while a group of the
  GUI is open, `begin` without its `end`, joins that group. The decided design below prevents its
  firing condition, `thread@first-prototype@one-controller-at-a-time`.
- `issue@rootpoint-protocol@save-mixes-relative-and-absolute`, which a protocol client other than
  the window can produce: the socket's `load` and `save` take the window's paths, per "Builds".
- `design@rootpoint-app@gesture-groups`, `design@rootpoint-protocol@view-state` and
  `design@rootpoint-app@editor-overlay`, which the decided design meets as "Decided design" says.

## Fails alone on

- A command that succeeds through `rootpoint-headless` and fails through the socket while the
  socket holds control.
- A window input that changes the document while the socket holds control.

## Premises that expire

- The socket is native only. Guard: the browser build has no transport, per
  `thread@first-prototype@protocol-transport`, and `spec@first-prototype@browser-build` does not
  build this listener for the wasm target, so the window holds control there always.
- Control belongs to the socket as a whole, which holds while one socket is the one other
  controller. A design that widens the lock, to several clients each with control of their own or
  to locks per slide or per node, ends it; the milestone document's "Later consequences" carries
  the owner's condition for that design.

## Implementation sequence

1. **An echo socket.** The GUI replies `ok` to every line without executing it, and
   `rootpoint-ctl` sends one line. Claim 4. Fails alone on: the socket and the wakeup.
2. **The verbs `control` and `release`**, in the protocol crate: parsed, printed and run headless.
   Claim 7. Fails alone on: the grammar.
3. **Execution, with control.** Socket commands run on the GUI's session, by the rules of control.
   Claims 1, 2, 3 and 5.
4. **The window under the socket's control.** The view inputs, the ignored inputs, the banner and
   the button. Claims 6 and 8.

The echo socket before the verbs: the verbs have nothing to carry without a transport. The verbs
before execution: the rules of control take them as input. Execution before the window's freeze:
the freeze is tested by socket commands that hold control.

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
(a) a socket command ends the window's gestures first, as each of the window's actions does; (b)
it closes the window's group and the gesture goes on in a new one; (c) one group per client, in the
session. The discriminating fact: a stay in editing keeps its group open from its first click to
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
Component, on the owner's word for the rule and on its argument for the rest, and the rewrite of
the last sentence of `design@rootpoint-app@gesture-groups`.

### The window takes control at once, and the protocol's `control` is granted at once, ending the window's gestures `##control-passing`

Proposed by the session in round 1 with three shapes for the protocol's request, (i) granted at
once, (ii) refused until the author gives control back, (iii) granted when no gesture of the window
is open, and the window's take-over immediate (`argument@first-prototype@arg-86`,
`argument@first-prototype@arg-87`). The owner answered Q34 in O23 with the workflow it serves,
"Manual adjustement (layout, wording...) is something that I would do when the agent has finished
a task and control was handed back to me with `release`, before giving further instructions. I
think (i) fits this workflow best." (`argument@first-prototype@arg-94`). Final state: approved, on
(i), by those words and the whole-table agreement of O23. A refused edit replies
`err the window has control` (`argument@first-prototype@arg-98`). Shape: "Decided design". Harvest:
part of the app's head of `thread@first-prototype@one-controller-at-a-time`, if the rules on one
decision per head join them; a head of its own otherwise. T6 guards it.

### The protocol asks for control with the verbs `control` and `release`, which run headless `##control-commands`

Proposed by the session in round 1 (`argument@first-prototype@arg-88`). Rivals: control taken
implicitly by the socket's first edit, and verbs the app's listener intercepts outside the grammar
(`argument@first-prototype@arg-89`). Final state: approved, by the whole-table agreement of O23.
The clause that control belongs to the socket as a whole, every connection together, was added by
the premortem of round 2, and closed on its argument (`argument@first-prototype@arg-97`): argued.
Shape: "Decided design". Harvest: a design head of the protocol crate's Component, since the verbs
are part of the grammar every program that drives rootpoint consumes, citing
`design@rootpoint-protocol@protocol-transport`.

### The protocol's queries need no control; its view commands do; the window's view inputs never do `##read-without-control`

Proposed by the session in round 1, for the protocol's queries (`argument@first-prototype@arg-90`);
reshaped in round 4 with the window's view inputs and the asymmetry
(`argument@first-prototype@arg-105`). Final state: approved, by the whole-table agreement of O23
and, reshaped, by O26: "I agree with the reshaped thread, proceed". Shape: "Decided design".
Harvest: part of the app's head of `thread@first-prototype@one-controller-at-a-time`.

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
O26: "I agree with the reshaped thread, proceed". Shape: "Decided design". Harvest: part of the
app's head of `thread@first-prototype@one-controller-at-a-time`, and the rewrite of the line of
`path@rootpoint-app@CLAUDE.md` that says `App::presenter` draws nothing over the slide.

### The window holds control when it starts `##control-at-start`

Proposed by the session in round 1 (`argument@first-prototype@arg-92`). Final state: approved, by
the whole-table agreement of O23. Harvest: part of the app's head of
`thread@first-prototype@one-controller-at-a-time`.

### Taking control closes the group the protocol left open with `end` `##interrupted-client-group`

Proposed by the session in round 1 (`argument@first-prototype@arg-93`); `release` does the same.
Final state: approved, by the whole-table agreement of O23. Harvest: part of the app's head of
`thread@first-prototype@one-controller-at-a-time`.

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

Round: round 1. Assistant. "closes that group with `end`, not `cancel`. Its edits stay visible as one
undo step, so one Ctrl+Z reverts them, while `cancel` would erase what you just watched happen."
Bears on `thread@first-prototype@interrupted-client-group`.

### The owner takes control to interrupt, and adjusts after `release` `##arg-94`

Round: round 1, the owner's answer. Owner, O23, Q34. "I expect taking control being a mean to
interrupt the agent to ask a question or add/change instructions I gave, or correct a mistake early
before it propagates. Manual adjustement (layout, wording...) is something that I would do when the
agent has finished a task and control was handed back to me with `release`, before giving further
instructions. I think (i) fits this workflow best." Bears on `thread@first-prototype@control-passing`.

### No prior art faces this case; a survey waits for a wider lock `##arg-95`

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

Round: round 4. Assistant. "When the agent sends `select` or `present`, your window follows it, which
is what a demo needs. When you click, the agent's `view` reply changes, which costs it nothing,
since its edits name ids." The rival "brings back the rival `design@rootpoint-protocol@view-state`
argued against, a view the GUI alone holds, which no headless test reaches. It also adds a second
view to reconcile at every hand-over." "An agent's `select` while you hold control would move what you are looking at. Worse, a
`select` of another node ends your stay in editing, since editing ends when its area is no longer
the selection (`App::editor` in `path@rootpoint-app@src/lib.rs`). That closes your group, an
effect on your edits. The window's view inputs act on nothing of the agent's." Bears on
`thread@first-prototype@window-under-protocol-control`,
`thread@first-prototype@read-without-control`.

## New names

```text
crates/rootpoint-protocol/  (crate rootpoint-protocol)
  Command::Control, Command::Release   the verbs `control` and `release`
  control::send                        Unix only: send one line to a socket, read the reply
  binary rootpoint-ctl                 src/bin/rootpoint-ctl.rs
crates/rootpoint-app/       (crate rootpoint-app)
  the controller holding control: the window or the socket, a field of App
  the listener of `--control <path>`, its thread and its channel
```

## Decided design

**The transport.** The socket's thread parses each line and sends the command to the GUI's thread
over a channel, then waits for the reply; the GUI drains the channel at the start of each frame, so
the session is only touched by the GUI's thread (`thread@first-prototype@protocol-transport`).
Nearest rival: JSON-RPC; see "Losing alternatives" of the milestone document.

**One controller at a time** (`thread@first-prototype@one-controller-at-a-time`). The app holds which
of two controllers, the window or the socket, has the authority to edit; the socket counts as one,
every connection together. When control passes, the side that loses it has its gestures or its
group closed, so no group is ever shared, which meets
`criterion@first-prototype@c17-no-shared-group` and prevents T5's firing condition; and one lock
over the whole document meets `criterion@first-prototype@c19-no-reconciling`. Nearest rival:
`thread@first-prototype@command-during-gesture`'s (a), the socket's commands ending the window's
gestures one by one, with no notion of control; defeated by the owner's intent in O20 and O23: the
author interrupts the agent, which (a) cannot express, since the agent's next command runs anyway.

**Passing control** (`thread@first-prototype@control-passing`,
`thread@first-prototype@interrupted-client-group`, `thread@first-prototype@control-at-start`). The
window holds control at start. The button takes control at once, closing with `end` any group the
socket left open. The socket's `control` is granted at once, ending the window's gestures as
`App::end_gestures` does: a stay ends with its edits one undo step, a drag as at its release.
`release` closes the socket's open group with `end` and gives control back. An edit refused for
want of control replies `err the window has control`. Nearest rival for the request: (ii), refused
until the author gives control back; defeated by the owner's workflow in O23, where the author
adjusts by hand after `release` and so is rarely in a gesture when the agent asks.

**The verbs** (`thread@first-prototype@control-commands`). `control` and `release` are verbs of the
protocol's grammar, `ok` headless, since a headless session has one controller; a script that
drives the window then runs headless unchanged. Nearest rival: implicit control, taken by the
socket's first edit; defeated because nothing would tell the window the agent is done, and an agent
could not ask whether it holds control.

**What needs control** (`thread@first-prototype@read-without-control`). `tree`, `view` and `render`
run whoever holds control, since they apply no operation and join no group. The protocol's view
commands need control; the window's view inputs never do. The asymmetry: a `select` of another node
by the socket ends a stay of the window, since editing ends when its area is no longer the
selection, while the window's view inputs act on nothing of the socket's. The view stays one, the
session's, `design@rootpoint-protocol@view-state`. Nearest rival: a view the window keeps apart
while the socket holds control; defeated because it brings back the rival that head argues
against, a view no headless test reaches, and adds a second view to reconcile at each hand-over.

**The window under the socket's control**
(`thread@first-prototype@window-under-protocol-control`). As "Builds" lists. The button sits in the
editor's banner and in a corner of presenter mode, over the texture, so the slide's image stays the
editor's, `design@rootpoint-app@editor-overlay`. Nearest rival: Esc taking control in presenter
mode; defeated by the owner's word in O24, who wants to leave presenter mode without taking control.

**What changes in the record.** The last sentence of `design@rootpoint-app@gesture-groups`, that a
command of another client could join a group left open while the window is away, becomes false: the
harvest rewrites it. The line of `path@rootpoint-app@CLAUDE.md` that says `App::presenter` draws
nothing over the slide becomes "nothing but the control button". T5,
`tripwire@rootpoint-protocol@client-joins-gui-group`, guards `design@rootpoint-protocol@gesture-undo`,
which stands; its condition is prevented by the app, so the harvest puts it to the owner, to keep or
to remove.

## Losing alternatives

- **The socket's commands ending the window's gestures, with no control**, Q33's (a), lost to
  `thread@first-prototype@one-controller-at-a-time`: the author could not interrupt the agent, whose
  next command runs anyway (`argument@first-prototype@arg-83`, `argument@first-prototype@arg-94`).
- **The window's gesture going on in a new group after a socket command**, Q33's (b), lost to the
  same thread: a gesture an agent interrupts is two undo steps, and the app must place the caret
  again on a text the agent changed (`argument@first-prototype@arg-85`).
- **One group per client in the session**, Q33's (c), lost to the same thread: per-connection undo
  state, and an undo that writes the author's earlier text over the agent's or is refused
  (`argument@first-prototype@arg-85`).
- **Holding a socket command until the window's group closes**, ruled out at the audit: the agent
  waits with no reply as long as a stay lasts, against `goal@rootpoint-app@live-agent-edits`
  (`argument@first-prototype@arg-81`).
- **The protocol's request refused until the author gives control back**, (ii), and **granted only
  when no gesture is open**, (iii), lost to (i) of `thread@first-prototype@control-passing`: a click
  before each task, or refusals for no visible reason (`argument@first-prototype@arg-87`,
  `argument@first-prototype@arg-94`). T6 brings (iii) back if (i) ends the owner's gestures in real
  use.
- **Implicit control, and verbs the listener intercepts**, lost to
  `thread@first-prototype@control-commands` (`argument@first-prototype@arg-89`).
- **Control held by one connection**, lost to the same thread: `rootpoint-ctl` opens one connection
  per command (`argument@first-prototype@arg-97`).
- **Every window input ignored**, lost to `thread@first-prototype@window-under-protocol-control`:
  the author inspects the slides while the agent edits, and editing commands read no view
  (`argument@first-prototype@arg-103`, `argument@first-prototype@arg-104`).
- **Esc taking control**, in presenter mode and in the editor, lost to the same thread, by the
  owner's word in O24 (`argument@first-prototype@arg-100`, `argument@first-prototype@arg-101`).
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
"Keep T6 and AC7."

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
round 3; O26 round 4.

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

## Harvest

Judged by the entry tests of `primer@design-heads`, under
`skill@knowledge-architect-decision-recording`:

- The socket listener and its threads, into the design home of the app crate's Component, citing
  `design@rootpoint-protocol@protocol-transport`.
- `thread@first-prototype@one-controller-at-a-time`, with the threads that join it by the rules on
  one decision per head (`thread@first-prototype@control-passing`,
  `thread@first-prototype@read-without-control`,
  `thread@first-prototype@window-under-protocol-control`, `thread@first-prototype@control-at-start`,
  `thread@first-prototype@interrupted-client-group`), into the design home of the app crate's
  Component; the rule on the owner's word, the rest on its argument; citing the `rejected` entries
  the losing alternatives earn, if any.
- `thread@first-prototype@control-commands`, into the design home of the protocol crate's
  Component, citing `design@rootpoint-protocol@protocol-transport`.
- The rewrite of the last sentence of `design@rootpoint-app@gesture-groups`, and of the line of
  `path@rootpoint-app@CLAUDE.md` on `App::presenter`.
- T6, in the tripwires home of the app crate's Component, on the owner's word, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- T5, `tripwire@rootpoint-protocol@client-joins-gui-group`, put to the owner: its condition is
  prevented by the app.
- The report on AC7, in the landing commits.
