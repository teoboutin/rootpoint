# Design of rootpoint-protocol

How rootpoint-protocol is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-protocol is for is in `register@rootpoint-protocol@goal`.

## Commands

### Commands and their replies are text lines, and the headless binary reads them from stdin `##protocol-transport`

A command is one line: a verb, then words separated by spaces. A text is a double-quoted string,
with `\"`, `\\` and `\n` escapes; a path is one too where it holds a space, a quote or a backslash;
a named option is `key=value`, its value bare or a quoted string, as a style's name with a space is.
A blank line and a line starting with `#` are no command, so a script carries comments. Each command
gets one reply line, `ok` or `err` followed by its result or its reason; a command that creates ids
names them, and `render` names what it could not draw. A printed `Command` is a line that parses
back to it, so a session can be written out as a script and run again. The binary
`rootpoint-headless` runs a `Session` on its stdin, one reply per command on its stdout.

The commands serve what a readable save file cannot: an edit applied to a running editor, a query
of its state, and a script run headless, as `goal@rootpoint-protocol@commands-drive-everything`
and `goal@rootpoint@headless-core` ask. An agent or a person types them at a terminal or in a script,
and reads one line per reply. The rival, JSON-RPC, `rejected@rootpoint-protocol@json-rpc-commands`,
is easier to parse and harder to type. The grammar and the replies are this crate's interface to
every program that drives rootpoint.

### Commands name slides, nodes and assets by their stored ids `##protocol-addressing`

Every argument that names a slide, a node or an asset is its id, `s<n>`, `n<n>` or `a<n>`, as the
model stores it and `tree` prints it, never a position. An id names the same slide, node or asset
after every later edit, and across a save and a load, since the model stores ids in the file and
never reuses one, `design@rootpoint-model@stable-ids`. A position shifts at every split or
removal, so a script's later command would name another area,
`rejected@rootpoint-model@addressing-by-position`. A style is named by its name instead,
`design@rootpoint-model@style-key`.

### A command refuses a target the document lacks, so no command creates a cosmetic breach `##no-new-breach`

A command that names a target the document lacks is refused before any operation: `image` and the
`font` option of `text`, `style add` and `style set` reply `err no asset a<n>` for an asset the
table does not hold, and `text … style="<name>"` and `style remove … into="<name>"` refuse a style
the table lacks, or one whose font is a missing target. The refusal is the command's, since the
operations accept a reference to a missing target but for the exceptions
`design@rootpoint-model@cosmetic-fallbacks` names. So a breach reaches a document only through a
file: a command that copies or moves an area, as `duplicate-slide` and `swap` do, carries the breach
a file brought, and creates none in a document that has none. The property test of
`path@rootpoint-protocol@tests/properties.rs` runs random sequences of commands from an empty
document, `undo`, `redo` and the groups included, and checks that none leaves a breach. The nearest
rival, a command that passes the target through and leaves the breach to the fallback, would hide an
editor's bug behind a warning; the refusal makes that bug a test's failure. It is a reason respected
at every command that names a target.

### A split's axis is typed as a short word for where its children go `##protocol-axis-words`

`split` and `tree` write a `LeftToRight` split as `beside` and a `TopToBottom` split as `stacked`.
Each word says where the children go, as the model's names do, `design@rootpoint-model@axis-names`,
and not the direction of the dividers, which a fresh agent read both ways,
`rejected@rootpoint-model@divider-direction-names`. Between these words and the model's own names
written as `left-to-right` and `top-to-bottom`, the owner chose the shorter words, which are typed
in every `split` command.

### The authority to edit a running editor is asked for with `control` and given back with `release` `##control-commands`

`control` asks for the authority to edit a running editor, which its window or its control socket
holds, `design@rootpoint-app@one-controller-at-a-time`; `release` gives it back, and replies
`err a group is open` while a group is open, so the authority never passes with a client's group
open, which `design@rootpoint-protocol@gesture-undo` would otherwise see joined by the other
controller's edits. A headless session has one controller, which always holds the authority, so
`Session::run` replies `ok` to `control` and changes nothing; a script that holds the authority in
a window gives the same replies headless, as `goal@rootpoint@headless-core` asks. The cost: a
client ends its group before it releases. The rival, a `release` that closes the open group as
`end` does, `rejected@rootpoint-protocol@release-closes-group`, adds a closer to that head's list.
The verbs are part of the grammar, the crate's interface to every program that drives rootpoint,
and not of the window's listener alone, `rejected@rootpoint-protocol@listener-verbs`; an authority
taken by a client's first edit would tell the window nothing when the agent is done,
`rejected@rootpoint-protocol@implicit-control`.

### `rootpoint-ctl` sends one command from its arguments, or a batch from its stdin, and exits by the replies `##ctl-sends-a-line-or-a-batch`

`rootpoint-ctl <socket> <words>…` joins the words with single spaces into one line; with no words,
it sends the lines of its stdin, as bytes, over one connection, a batch,
`design@rootpoint-app@socket-batch`. It prints each reply as it comes, and exits 0 when every reply
is `ok`, a line with no command getting none; 1 when one is `err`, also when the connection then
ends, as after the refusal of a take-over; and 2 when the socket cannot be reached, or stdin or the
connection fails before any `err`, as when the window is closed. `path@rootpoint-protocol@README.md`
states the contract for its users. `rootpoint_protocol::control::Connection` is the client it is
built on, Unix only. A command whose connection ends with no reply is an error, never the empty
answer of a comment, and a line holding a newline is refused before it is sent: the client knows
which of its lines are commands, by `read_line`, so an agent never reads an edit the window never
ran as done, nor one it ran as none.

## The session

### The view state lives in the session, outside the document's operations `##view-state`

Besides its document and its undo and redo stacks, a `Session` holds a `View`: the selected node,
if any, the slide the editor shows, if one is named, the slide presenter mode shows, if it is on,
and the preview, in which the editor shows the final result, with no mark, under the rules of
`design@rootpoint-protocol@final-preview`. `select n<id>` selects a node and shows its slide;
`select s<id>` shows a slide and selects no node; with no slide named the editor shows the first.
The view commands, `select`, `deselect`, `preview`,
`present`, `next`, `prev` and `stop`, change it and apply no operation; `view` reports it. Since
they apply no operation, `undo` and `redo` do not step through them: an `undo` after a `select`
reverts the last edit, not the selection. An operation that removes the selected node, the
shown slide or the presented slide drops it from the view, so the view never names what the
document does not hold. Selection, the slide shown, presenter mode and the preview then run
headless, in a test or a script, as `goal@rootpoint@headless-core` asks of every operation the GUI
offers, and `goal@rootpoint-protocol@commands-drive-everything` asks of every view of the
presentation the editor shows. The rival, a view state the GUI alone holds, would leave them out of
every headless test, and an agent could not show the author the final result.

### Entering the preview deselects, and a selection of a node ends it `##final-preview`

`preview on` sets the preview and deselects, and `select n<id>` ends it and selects, so the view
never holds a preview and a selection at once; `preview off` ends it too, and `select s<id>`,
`deselect` and `present` leave it as it is. The rules are the session's, part of
`design@rootpoint-protocol@view-state`, so an agent and a headless test meet what the window does.

The nearest rival, a click in the preview that does nothing,
`rejected@rootpoint-protocol@click-does-nothing-in-preview`, costs a click each time the author
leaves the preview. What the rules give up: a text's own custom style cannot be tuned while the
preview shows its result, since tuning it needs it selected, which ends the preview; a named style
is still tuned live in the window's Styles tab.

On the owner's word: the owner proposed that entering the preview clears the selection and that a
click on the slide leaves it and selects, intuitively the way the owner would use it, while not
fully certain. `tripwire@rootpoint-protocol@click-ends-the-preview` watches the choice.

### The session opens no file: its host gives it a `Files` `##session-files`

`Session` reads and writes the presentation, its assets and rendered images only through the
`Files` trait its caller passes to each command, and touches no file system itself; the headless
binary gives it the native file system. The session is part of the browser build, per
`goal@rootpoint-protocol@browser-buildable`, and there `std::fs` compiles and fails only when
called, so the wasm gate cannot catch a direct file access, as `path@rootpoint-model@CLAUDE.md`
records: read in Rust 1.98's `std`, `wasm32-unknown-unknown` takes the module
`sys::fs::unsupported`, whose `File::open` returns an `Unsupported` error. It is a rule every
file access of the session respects, and every host that implements `Files`. It is the session's counterpart of
`design@rootpoint-render@caller-supplies-assets`.

## Undo

### One gesture is one undo step: the commands `begin`, `end` and `cancel` group the edits between them `##gesture-undo`

`begin` opens a group, and replies `err` while one is open, since groups do not nest. `end` closes
it: every operation applied inside is one undo step, undone and redone together, and an empty
group leaves no step; `err` with no group open. `cancel` reverts every operation applied inside
the group and restores the undo and redo stacks as they were at its `begin`; `err` with no group
open. `undo` and `redo` inside a group close it first, as `end` does, then act; `new`, `load` and
`sample` close it as part of replacing the document and the stacks; view commands join nothing.
The stacks hold steps, each the inverses of one group or of one command, applied atomically: a
step that fails leaves the document and the view as they were.

A drag that sent one command a frame with no grouping would leave about sixty undo steps a second.
The groups keep the session the only state and keep the grouping visible in a script, and the
commands are the crate's interface, so a script replays a gesture as the GUI made it. `begin` and
`end` is the family Qt, Cocoa, GTK and Emacs ship, and `cancel` is Emacs's
`cancel-change-group`; without it, cancelling a gesture is `end` then `undo`, which leaves a redo
step. The rivals lost: a preview on a copy, `rejected@rootpoint-protocol@preview-on-copy`, which
keeps the protocol unchanged but leaves the session behind the screen, and the session merging
consecutive commands, `rejected@rootpoint-protocol@merged-commands`. On the owner's word: the
owner chose explicit groups over both.

## Saving

### A save keeps the assets inside its directory and copies every other asset, and every embedded one, beside the saved file `##asset-save-rule`

Every `Path` asset of a session is relative to `Session::directory`. At a `save` to a file whose
directory is D, paths compared after joining them to their directory and removing `.` and `..`:
a `Path` asset whose file lies inside D, at any depth, keeps its file, its path written relative
to D; any other asset, one outside D or an embedded one, is written into D under its file name, or
`asset-<id>` with the extension its bytes name, and its path becomes that name. A name taken by a
file of other bytes takes a suffix, `-2`, `-3` and on, before the extension, and one with the same
bytes is used as it is, so a save overwrites no file of other bytes. Every asset to write is read
before anything is written, and one that cannot be read makes the save reply `err`, naming it, and
write nothing; an asset that keeps its file is not read. A saved file therefore holds no embedded bytes, which stay what packing writes, per
`design@rootpoint-model@asset-source`.

It applies `design@rootpoint-model@image-storage`: assets are files beside the presentation, and
the browser build opens one presentation directory, `design@rootpoint-app@browser-load-only`, so it
cannot read a file outside it, which rules out rewriting paths in place with `..`. Refusing a save
into another directory would refuse the first save of every presentation built in the GUI with an
image, against `goal@rootpoint@no-language-no-build`. On the owner's word: the owner chose copies
beside the presentation file over a subfolder for them, for now, and wants the author free, later
than the first prototype, to arrange assets in subdirectories of any depth, a tree that packing,
then unpacking elsewhere, keeps; the rule keeps such a tree already.

### A save changes the open document's asset paths to the saved file's, through `apply` and with no undo step `##save-updates-sources`

After a save writes its files, the paths that changed enter the document through `apply`, one
`Op::SetAssetSource` per asset, as `design@rootpoint-model@edit-operation-core` asks of every
change, so the open document is the file just saved. They are no undo step: the session rewrites
the same asset's source in the operations its undo and redo stacks hold, so an undo after the save
restores an asset with the path the saved file uses, and an asset that only the stacks hold gets
its path from the new directory, with `..` where needed, or its bytes where no such path exists.

The rivals: the new paths as an undo step, whose undo would name another file once the directory
changed; and a session that keeps its old paths, whose document would then differ from the file it
just saved, against `goal@rootpoint@readable-save-file`.

## The built-in sample

### Every build carries a sample presentation that `sample` opens `##builtin-sample`

The protocol crate holds a sample presentation, a RON text and its image files, included at build
time from `path@rootpoint-protocol@sample/`. `sample` replaces the session's document by it, its
images as embedded assets, with empty stacks and the working directory as its directory, as
`load` does; a save writes its images beside the saved file, by
`design@rootpoint-protocol@asset-save-rule`. So the sample opens in the GUI and headless by one
command, and the command is the interface every client of the protocol uses. Its slides say how
to split, select, drag and swap, and its images stay under 50 KB in all.

On the owner's word: the owner wants the sample kept as a permanent feature of every build, a
tutorial and a demonstrator anywhere, not only in the browser. The rival, a file the app's `Files`
alone serves, would open in the GUI and in no headless session, against
`goal@rootpoint@headless-core`.
