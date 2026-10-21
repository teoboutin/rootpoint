# Design of rootpoint-protocol

How rootpoint-protocol is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-protocol is for is in `register@rootpoint-protocol@goal`.

## Commands

### Commands and their replies are text lines, and the headless binary reads them from stdin `##protocol-transport`

A command is one line: a verb, then words separated by spaces. A text is a double-quoted string,
with `\"`, `\\` and `\n` escapes; a path is one too where it holds a space, a quote or a
backslash; a named option is `key=value`. A blank line and a line starting with `#` are no command, so a script carries
comments. Each command gets one reply line, `ok` or `err` followed by its result or its reason; a
command that creates ids names them, and `render` names what it could not draw. A printed
`Command` is a line that parses back to it, so a session can be written out as a script and run
again. The
binary `rootpoint-headless` runs a `Session` on its stdin, one reply per command on its stdout.

The commands serve what a readable save file cannot: an edit applied to a running editor, a query
of its state, and a script run headless, as `goal@rootpoint@text-command-protocol` and
`goal@rootpoint@headless-core` ask. An agent or a person types them at a terminal or in a script,
and reads one line per reply. The rival, JSON-RPC, `rejected@rootpoint-protocol@json-rpc-commands`,
is easier to parse and harder to type. The grammar and the replies are this crate's interface to
every program that drives rootpoint.

### Commands name slides, nodes and assets by their stored ids `##protocol-addressing`

Every argument that names a slide, a node or an asset is its id, `s<n>`, `n<n>` or `a<n>`, as the
model stores it and `tree` prints it, never a position. An id names the same slide, node or asset
after every later edit, and across a save and a load, since the model stores ids in the file and
never reuses one, `design@rootpoint-model@stable-ids`. A position shifts at every split or
removal, so a script's later command would name another area,
`rejected@rootpoint-model@addressing-by-position`.

## The session

### The session holds a view state, which only commands change and which undo leaves alone `##view-state`

Besides its document and its undo and redo stacks, a `Session` holds a `View`: the selected node,
if any, and the slide presenter mode shows, if it is on. The view commands, `select`, `deselect`,
`present`, `next`, `prev` and `stop`, change it and apply no operation, so `undo` and `redo` do not
step through them; `view` reports it. An operation that removes the selected node or the presented
slide drops it from the view, so the view never names what the document does not hold. Selection
and presenter mode then run headless, in a test or a script, as `goal@rootpoint@headless-core`
asks of every operation the GUI offers. The rival, a view state the GUI alone holds, would leave
both out of every headless test.

### The session opens no file: its host gives it a `Files` `##session-files`

`Session` reads and writes the presentation, its assets and rendered images only through the
`Files` trait its caller passes to each command, and touches no file system itself; the headless
binary gives it the native file system. The session is part of the browser build, per
`goal@rootpoint@runs-in-browser`, and there `std::fs` compiles and fails only when called, so the
wasm gate cannot catch a direct file access: a rule every file access of the session respects, and
every host that implements `Files`. It is the session's counterpart of
`design@rootpoint-render@caller-supplies-assets`.
