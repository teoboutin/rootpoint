# Rejected designs of rootpoint-app

The designs not kept for rootpoint-app, and why: one level-three heading per design, each cited by
the design head whose argument covers it, in `register@rootpoint-app@design`.

### iced, Slint, xilem or gpui as the window's host `##other-gui-hosts`

`live`. The toolkit survey of the first prototype, `path@rootpoint@docs/grounding/toolkits.md`,
found: under the project's own renderer, iced's `pane_grid` does not help, since the dividers are
inside the slide's image, and its releases are slow, 0.13 in 2024-09 and 0.14 in 2025-12; Slint
would make a free-form canvas from declarative items, which is friction, an assumption of the survey
it did not check; xilem and gpui failed what `goal@rootpoint@runs-in-browser` asks, that the browser
build edit and present as the native one does: xilem's web backend lays text out in the browser's
DOM, so no canvas is shared, and gpui's web support was unpublished and, by the survey's assumption,
needs nightly Rust and threaded wasm. iced stays the host to weigh again if slides are ever drawn
with a toolkit's own painter, where its one text engine and its `iced_test` screenshots count.

### Parley's `PlainEditor` as the editor of a text in place `##plain-editor`

`live`. In Parley 0.12.0 a `PlainEditor` owns its own `Layout`, built from its own buffer and
`StyleSet` (the field `layout` of the struct in `parley/src/editing/editor.rs`), so it cannot work on
the layout `render_slide` draws: its caret would sit on a second layout that has to agree with the
drawn one, from styles set again by hand, which `design@rootpoint-render@area-text-layout` exists
to avoid. Its `Selection` and `Cursor` take any layout.

### A protocol command that places or reads the caret and the selection of an edited text `##caret-command`

`live`. On the owner's word, the caret and the selection of a text being edited are the editor's
state, not a view `goal@rootpoint-protocol@commands-drive-everything` asks a command for, so the
protocol carries no command for them, `design@rootpoint-app@caret-editor-state`. A script changes a
text with `text`, and needs no caret to do so.

### The authority to edit held by one connection of the control socket `##control-per-connection`

`live`. An authority held by a connection would end with it: at every command a client sends on a
connection of its own, as `rootpoint-ctl <socket> <words>…` does, and between two batches, so an
agent would ask for it again before each edit or each batch, and the window would show the author
an authority that comes and goes. It would change what the socket's clients consume;
`design@rootpoint-app@one-controller-at-a-time` counts the socket as one controller.

### The socket holding the authority to edit when the window starts `##socket-holds-control-at-start`

`live`. A window just opened with `--control` would refuse its author's every edit until the
agent released it, and a client would not need `control` before its first edit, which changes what
the socket's clients consume. `design@rootpoint-app@one-controller-at-a-time` gives the window the
authority at start.

### The socket's request for the authority refused until the author gives it back `##request-refused-until-handed-back`

`live`. Every task the author gives the agent would need a click from the author first, or the
agent stalls and must tell the author, and `control` would reply otherwise, which the socket's
clients consume. The owner chose the request granted at once, `design@rootpoint-app@control-passing`,
on the owner's workflow.

### The socket's request for the authority granted only while the window has no gesture open `##control-when-idle`

`live`. A stay in editing keeps its gesture open until the author leaves its area, also while the
author is in a terminal giving the agent its task, so the agent would meet refusals with no reason
it can see. The owner chose the request granted at once, `design@rootpoint-app@control-passing`;
`tripwire@rootpoint-app@control-ends-a-gesture` brings this design back if the request granted at
once ends the author's gestures in real use.

### A button "Give control" the author clicks before the agent may take the authority again `##give-control-button`

`live`. After a take-over, the socket's `control` would be refused until the author clicks it, at
the cost of one click to hand back after each interrupt. The owner chose instead the refusal that
tells the agent to stop and wait for the author's instructions, judging that in principle it should
be enough, `design@rootpoint-app@control-passing`; `tripwire@rootpoint-app@agent-takes-control-back`
brings this design back if agents misbehave too often.

### One connection to the control socket for each command `##connection-per-command`

`live`. An agent would pay a round trip for each edit, and a take-over would be answered with as
many refusals as the agent's script has lines left, since each line opens a connection of its own.
It would change the socket's contract and the command line of `rootpoint-ctl`, which other programs
consume. The owner judged that restricting an agent to one edit at a time seems a less efficient
method; `design@rootpoint-app@socket-batch` lets a connection carry a batch.
