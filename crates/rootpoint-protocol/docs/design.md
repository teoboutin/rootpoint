# Design of rootpoint-protocol

How rootpoint-protocol is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-protocol is for is in `register@rootpoint-protocol@goal`.

## Commands

### Commands and their replies are text lines, and the headless binary reads them from stdin `##protocol-transport`

A command is one line: a verb, then words separated by spaces. A text is a double-quoted string,
with `\"`, `\\` and `\n` escapes; a path is one too where it holds a space, a quote, a backslash or
a word of a tree; a named option is `key=value`, its value bare or a quoted string, as a style's
name with a space is. A text tree is given on one line, in brackets, braces and commas, which are
words of their own outside a quoted string: `["Let x be a ", {t41 bold "prime"}, " number"]`, a
tree its items separated by commas, each a quoted text or a span, and a span `{` its id, its
options, then its items `}`. `text` takes a tree so and `tree` prints one so, so that an agent
writes and reads a styled text as one line; the nearest rival, a tree as a JSON payload,
`rejected@rootpoint-protocol@json-rpc-commands`, is easier to parse and harder to type.
`tripwire@rootpoint-protocol@one-line-tree-written-badly` watches agents writing the tree. A blank
line and a line starting with `#` are no command, so a script carries comments. Each command
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

### Commands name slides, nodes, assets and spans by their stored ids `##protocol-addressing`

Every argument that names a slide, a node, an asset or a span is its id, `s<n>`, `n<n>`, `a<n>` or
`t<n>`, as the model stores it and `tree` prints it, never a position. An id names the same slide,
node, asset or span after every later edit, and across a save and a load, since the model stores
ids in the file and never reuses one, `design@rootpoint-model@stable-ids`. A position shifts at
every split or removal, and an offset at every edit before it, so a script's later command would
name another area or another span, `rejected@rootpoint-model@addressing-by-position`. A command
that makes a span or changes characters names a text node, the root of an area's text by the
area's id or a span by its own, and takes an offset or a range in it, read at that moment and
valid at that moment only; a `t<n>` finds its area, since a span's id is unique in the document. A
style is named by its name instead, `design@rootpoint-model@style-key`.

### A text is changed by commands of one operation each, so its spans keep their ids `##text-edit-commands`

`insert <n|t> <offset> "<text>"`, `delete <n|t> <a>..<b>`, `wrap <n|t> <a>..<b> <options>`,
`unwrap t<n>`, `span t<n> <options>` and `split-span t<n> <offset>` each apply one operation of
the model's text tree, `design@rootpoint-model@edit-operation-core`, and change no span they do not
name, so every span keeps its id, `design@rootpoint-model@stable-ids`, and a script styles a span
again by its id after any edit before it. `wrap` and `split-span` reply the id of the span they
make. `split-span` is not `split`, which splits a node of a slide. `text <area> <tree> [options]`
sets a whole tree, every span with a new id, and replies the new ids. A span's options are
`text`'s but alignment, the area's alone, and apply in order over the span's current style,
`design@rootpoint-model@named-span-styles`. The nearest rival, a whole-text replacement at each
change, `rejected@rootpoint-protocol@whole-text-replacement`, gives every span a new id at each
key, since a tree set whole is renumbered. The commands
are the crate's interface to every program that edits a styled text, as
`goal@rootpoint-protocol@commands-drive-everything` asks.
`tripwire@rootpoint-protocol@one-line-tree-written-badly` watches agents writing `text`'s tree.

### `text` with options and no tree changes the area's style alone, its spans and their ids kept `##text-style-alone`

`text <area> <options>` changes the style of an area's text by `Op::SetContent` of its current
tree, so its spans keep their ids, `design@rootpoint-model@stable-ids`, where `text` with a tree
gives each span a new one; it is refused on an image area, which has no text style to change. The
window's controls of a text's style send that form. The rival, a verb of its own for the area's
style, `rejected@rootpoint-protocol@area-style-verb`, takes the options `text` already takes under
a second name.

### `<key>=inherit` clears a value a span's style sets, one option for each value `##span-value-inherit`

In every list of a span's options, `font=inherit`, `size=inherit`, `bold=inherit`,
`italic=inherit` and `colour=inherit` each leave a value unset, so the span draws with its
parent's, `design@rootpoint-model@span-style-overrides`: one option for each value, as every other
option sets one. The rival, an option naming the values to clear,
`rejected@rootpoint-protocol@clear-values-option`, holds a list in its value, a comma separating
its values, which outside a quoted string is a word of a text tree,
`design@rootpoint-protocol@protocol-transport`.

### A command refuses what would put a cosmetic breach in the document, so no command creates one `##no-new-breach`

A command that names a target the document lacks is refused before any operation: `image` and the
`font` option of `text`, `style add` and `style set` reply `err no asset a<n>` for an asset the
table does not hold, `text … style="<name>"` and `style remove … into="<name>"` refuse a style
the table lacks, or one whose font is a missing target, and so do the span commands for a span
style and a span's font, `wrap`, `span`, a span of `text`, `span-style add`, `span-style set` and
`span-style remove … into="<name>"`, and `import` refuses an asset path that
fails `asset_path_is_confined`, `design@rootpoint-protocol@store-confines-access`, as `asset-move`
and `asset-relink` refuse one, and `asset-remove … into=` an asset the table lacks. The refusal is
the command's, since the operations accept a reference to a missing target but for the exceptions
`design@rootpoint-model@cosmetic-fallbacks` names. So a breach reaches a document only through a
file: a command that copies or moves an area, as `duplicate-slide` and `swap` do, carries the breach
a file brought, and creates none in a document that has none. The property test of
`path@rootpoint-protocol@tests/properties.rs` runs random sequences of commands from an empty
document, `undo`, `redo`, the groups and the commands of a text tree and of the span style table
included, and checks that none leaves a breach. The nearest
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

### `rootpoint-ctl` sends one command from its arguments, or a batch from its stdin, to a window it finds by its workspace or its socket, and exits by the replies `##ctl-sends-a-line-or-a-batch`

`rootpoint-ctl <workspace-dir | socket> <words>…` joins the words with single spaces into one line;
with no words, it sends the lines of its stdin, as bytes, over one connection, a batch,
`design@rootpoint-app@socket-batch`. A directory names the socket in its pointer file,
`.rootpoint/socket`, which the window holding the workspace writes; `rootpoint-ctl list` prints the
file name of each socket of the runtime directory that accepts a connection. It prints each reply
as it comes, and exits 0 when every reply is `ok`, a line with no command getting none; 1 when one
is `err`, also when the connection then ends, as after the refusal of a take-over; and 2 when the
socket cannot be reached, or stdin or the connection fails before any `err`, as when the window is
closed. `path@rootpoint-protocol@README.md` states the contract for its users.
`rootpoint_protocol::control::Connection` is the client it is built on, Unix only. A command whose
connection ends with no reply is an error, never the empty answer of a comment, and a line holding a
newline is refused before it is sent: the client knows which of its lines are commands, by
`read_line`, so an agent never reads an edit the window never ran as done, nor one it ran as none.
A client finds the window from the workspace it works in, so where the socket is, the runtime
directory today, can change with no change to what a client types.

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
`deselect` and `present` leave it as it is. In the editor, a click on an area sends that `select`,
so a click on the slide leaves the preview and selects. The rules are the session's, part of
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
`Files` trait its caller passes to each command, or in its own map when its store is in memory,
`design@rootpoint-protocol@editing-modes`, and touches no file system itself; the headless binary
gives it the native file system. The session is part of the browser build, per
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
open. `undo` and `redo` inside a group close it first, as `end` does, then act; `new`, `sample`,
`open`, `open-copy` and `open-pack` close it as part of replacing the document and the stacks; `reload` joins
it as any edit does; view commands join nothing.
The stacks hold steps, each the revert of one group or of one command, applied atomically: a step
that fails leaves the document and the view as they were. The file moves a step holds beside its
operations are `design@rootpoint-protocol@step-holds-its-moves`'s.

A drag that sent one command a frame with no grouping would leave about sixty undo steps a second.
The groups keep the session the only state and keep the grouping visible in a script, and the
commands are the crate's interface, so a script replays a gesture as the GUI made it. `begin` and
`end` is the family Qt, Cocoa, GTK and Emacs ship, and `cancel` is Emacs's
`cancel-change-group`; without it, cancelling a gesture is `end` then `undo`, which leaves a redo
step. The rivals lost: a preview on a copy, `rejected@rootpoint-protocol@preview-on-copy`, which
keeps the protocol unchanged but leaves the session behind the screen, and the session merging
consecutive commands, `rejected@rootpoint-protocol@merged-commands`. On the owner's word: the
owner chose explicit groups over both.

### `reload` makes the file the document as one undo step, which brings the editor's own back `##reload-is-an-undo-step`

In a workspace, `reload` reads `presentation.ron` and applies it as `Op::ReplaceDocument`, whose
inverse holds the document replaced, so `undo` gives back what the editor held; the id counter
keeps the larger of the two documents', so no id the editor handed out is given again,
`design@rootpoint-model@stable-ids`. In memory it replies `err`, since no file is the document's.
The rival, a reload that empties the undo stacks as `open` does, loses the editor's own edits at
the one moment the author chose between two versions. The window's banner runs it,
`design@rootpoint-app@ron-overwrite-guard`, and so can a script.

## The store

### The session reads its presentation from a store: its own map in memory, or the host's files below a workspace's root `##editing-modes`

`Store::Memory` holds the presentation's files in the session, keyed by asset path, and no file of
it is on disk; `Store::Workspace` names a directory of the host's files, its root, stored as
`Files::canonical` gives it, with `presentation.ron` at the root and each asset at its path below
it. `new`, `sample` and `open-copy <dir>`, which copies a directory's presentation and the files of
its assets into the map, open in memory; `open <dir>` opens a workspace, an empty presentation when
the directory holds none. `open-pack <file>` opens a pack in memory, its entries the map, and
`unpack <dir>` writes a presentation held in memory into an empty or absent directory, a workspace
from then on. `save` writes `presentation.ron` at a workspace's root; in memory it writes the pack last opened
or written, `Session::pack`, and replies `err` with none. A host path, the argument of `open`, `open-copy`, `open-pack`, `pack`, `unpack`, `import`,
`render`, is read by the host's files as the host reads it.

An asset path is a key of the map in memory, and relative to the root in a workspace, whose root
never changes while it is open, so no save and no open rewrites a path, in the document or in the
stacks; only `asset-move` and `asset-relink` give one asset a new path,
`design@rootpoint-protocol@asset-commands`. A
save to any directory, the rival `rejected@rootpoint-protocol@free-save-path`, rewrites the paths
against a base that each save and open changes, the cause of every defect of that rule, and
compares paths of two bases, which a relative and an absolute form of one directory defeat; a
canonical root leaves no two forms of one directory
to compare. Its cost: a `.ron` file outside a workspace is not opened by itself, but as the
directory that holds it. The map is the session's, not a host's, so a presentation in memory
behaves the same headless and in the window, as `goal@rootpoint@headless-core` asks: a map of the
host would need every host to build the same map, and a way for a command to switch the host's
store, which the session cannot ask of `Files`. On the owner's word: the native build keeps the
store in memory beside the workspace.

### `store` replies the store and the undo steps since its open or its save `##store-query`

`store` replies `ok memory unsaved=<n>`, with ` pack=<file>` once a pack is the one `save` writes,
or `ok workspace <root> unsaved=<n>`, where `n` is
`Session::unsaved`, the undo steps pushed, undone or redone since the store was opened or saved, or,
in memory, written as a pack. It
is the query of what the window's state shows, so an agent reads that view as the author sees it, as
`goal@rootpoint-protocol@commands-drive-everything` asks of every view the window shows. With no
query, an agent would have to infer the store from the commands it sent, which miss those the window
ran.

### The store refuses every access to an asset path that is not confined below its root `##store-confines-access`

Every read of an asset, every write of an import and every move of an asset command goes through one
check of the store, in `path@rootpoint-protocol@src/store.rs`: a path that fails `asset_path_is_confined` is refused, and in
a workspace so is one whose file, every symlink resolved by `Files::canonical`, lies outside the
canonical root, the nearest existing directory's path taken for a file not written yet; the trash
has its own check, `design@rootpoint-protocol@asset-trash`. `open-copy <dir>` reads through the same check against `<dir>`, and leaves out of the map an asset
it refuses. So no command reads or writes outside the root, whatever the file names: `render` draws
such an asset as missing, and `import` to such a path replies `err`. A check of the paths at load
alone would miss a symlink, which is the disk's state, `design@rootpoint-model@cosmetic-fallbacks`.
On the owner's word: the assets stay confined inside the workspace; packs are meant to come from
trusted sources, as with the files of any presentation software, and security features can be
designed and added later.

### An import adds a file below a workspace's root at its path, and copies any other file into the store under a name no other bytes hold `##import-copies-into-the-store`

`import <host-path> [<asset-path>]` reads a file of the host's files, anywhere. In a workspace, a
file whose canonical path lies below the root is added at its path relative to the root, with no
copy, and an `<asset-path>` given must be that path. Any other file is written into the store, the
map or the root, at `<asset-path>`, or at the root under its file name; a name that a file of other
bytes, or a folder, holds takes a suffix, `-2`, `-3` and on, before the extension of its last
component, a name below a file is refused, and a name whose file holds the same bytes is used as it
is, so an import overwrites no file. An import
whose path an asset already has adds no asset and replies that asset's id: one asset per file, and
the import of a missing asset's file gives that asset its bytes.

On the owner's word: a file from outside the workspace is imported by a copy into it; the owner
chose copies at the root, beside the presentation, over a subfolder for them, for now, and wants
the author free, later than the first prototype, to arrange assets in subdirectories of any depth,
a tree that packing, then
unpacking elsewhere, keeps, which the `<asset-path>` of an import gives.

### A workspace is held by one process, by `File::try_lock` on its `.rootpoint/lock` `##workspace-lock`

`WorkspaceLock`, in `path@rootpoint-protocol@src/workspace.rs`, creates the workspace's
`.rootpoint/` with a `.gitignore` that ignores everything in it, takes `File::try_lock` on
`.rootpoint/lock`, writes its pid there, and holds the file until dropped. A native host runs every
command through `workspace::run`: before `open <dir>` of a directory that exists, and before
`unpack <dir>` of a presentation held in memory to a path that is no file, it takes the lock of `<dir>`, or replies
`err <dir> is open in another process, pid <pid>` and runs nothing; after an `ok` that changed the
session's root, it holds the new lock and drops the old one. The lock of the workspace held is kept
when it is opened again, since `File::try_lock` is `flock` on Linux, which refuses a second
descriptor of one file in one process as in another, as the test `the_socket_follows_the_store` of
`path@rootpoint-app@tests/socket_move.rs` shows, opening the workspace held again. `File::try_lock`
is stable from Rust 1.89.0. The headless binary and the window both run
it, and the session never does, `design@rootpoint-protocol@session-files`; it is the protocol's,
since `design@rootpoint@gui-only-in-app` keeps the headless binary out of the app.

The kernel releases the lock when the process ends, so a crash leaves no stale lock, where a file
holding a pid, the rival, stays held after a crash until someone removes it by hand. Only the
holder binds the workspace's socket, `design@rootpoint-app@socket-place`, so two windows cannot
both bind it. The lock is between processes: inside one window, the authority to
edit stays `design@rootpoint-app@one-controller-at-a-time`, and an agent drives the window through
its socket and never opens the workspace itself, so the lock keeps no agent out. `tripwire@rootpoint-protocol@workspace-on-two-machines`
watches a workspace a sync tool shares. On the owner's word: a client locks the workspace, through
a lock in `.rootpoint/`, so that another client cannot take it, at least as long as no finer lock,
per slide or per node, exists.

### The asset table and its files change only by commands, each one undo step at most `##asset-commands`

The commands are `asset-move`, `asset-relink`, `asset-replace`, `asset-remove` and
`asset-remove-unused`, beside `import`, which adds one. `asset-move <a> <asset-path>` moves the
asset's file to a path that holds no file and that no other asset names, and gives the asset that
path; it replies `err` for an asset whose file is missing,
naming `asset-relink`. `asset-relink <a> <asset-path>` gives the asset the path of a file the store
holds and no other asset names, and moves no file: the repair of a missing asset.
`asset-replace <a> <host-path>` gives the asset's file the bytes of a host file. `asset-remove <a>`
removes an asset nothing names, and `asset-remove <a> into=a<n>` first makes every image, text,
span, named style and named span style that names `a` name `a<n>`, as `style remove … into=` does for a style,
`design@rootpoint-model@style-removal`. `asset-remove-unused` removes every asset nothing names,
one undo step, and none when no asset is unused, since an empty step would count as a change to
save. Each other command is one undo step, its file moves in it,
`design@rootpoint-protocol@asset-trash`. The query `assets [all|unused|missing|untracked]` replies
one line, as every reply does, its entries separated by `; `, each `a<id> <path> <bytes|missing>
uses=<uses|->`, its uses the areas, by an image or a font of their text or of its spans, the named
styles and the named span styles that name it, a span style written `span:"<name>"` apart from a
text style of one name, `missing` an asset whose file
the store cannot read, one whose path fails confinement included; `assets untracked` replies the
paths of the store's files no asset names, `design@rootpoint-protocol@table-of-added-assets`.

An asset keeps its id through each command, `design@rootpoint-model@stable-ids`, so no slide
changes at a move or a replace. A path another asset names is refused, as the target of a move or
a relink, and as the file a move or a replace changes, since a change of one would change the
other; only a file edited by hand names one path twice. On an asset whose path fails confinement, `asset-move` and
`asset-replace` reply `err`, `asset-remove` removes the entry and moves no file, and
`asset-relink` to a confined path repairs it; an asset whose file a symlink places outside the
root is removed the same way, its file left where it is. So such an asset can always leave the
table and no command reaches outside the root, `design@rootpoint-protocol@store-confines-access`;
the tests of `path@rootpoint-protocol@tests/asset_commands.rs` give `RecordingFiles` each command.
A change of an asset is a command, so a script or an agent makes every change the author can, one
line each with a one-line reply, as `goal@rootpoint-protocol@commands-drive-everything` asks.

### Every change of the store's files is a move, and a file a change discards stays in the trash while a step reaches it `##asset-trash`

`asset-remove` moves the asset's file to the trash; `asset-replace` writes the new bytes into the
trash, then moves the old file there and the new one to the asset's path; and `import` writes the
file it adds into the trash, then moves it to its path. Every change a command makes to the store's
files is then a move, which its undo step holds and runs back,
`design@rootpoint-protocol@step-holds-its-moves`.

The trash is `.rootpoint/trash/<id>-<n>` in a workspace, `import-<n>` for an import's file,
`design@rootpoint-model@workspace-layout`, and `Session::trash`, by the same names, in memory; `<n>`
counts the session's entries, so an asset replaced twice keeps both versions, and an entry an open
failed to remove is never named again. A trash whose `.rootpoint/` or `trash/` resolves to another
path than its own, as a symlink makes it, is refused, and an open empties none, so that no access
reaches outside the root, `design@rootpoint-protocol@store-confines-access`. `unpack` writes the trash in memory into the
workspace's, so the undo steps it keeps find their files. The trash holds a file only while a step
reaches it: an edit that clears the redo stack, and a `cancel`, remove the entries no step reaches
any more; an `open` of a workspace empties its trash, since no undo step of the new session reaches
it; and a session in memory takes its trash with it. The test
`a_removal_goes_to_the_trash_and_its_undo_brings_the_file_back` of
`path@rootpoint-protocol@tests/asset_commands.rs` shows it across two sessions, and
`the_trash_holds_no_entry_no_step_reaches` after a replace undone.

A removal that deleted the file at once would leave the undo of `asset-remove` an asset with no
file, against `design@rootpoint-app@no-discard-confirmation`, whose undo restores what an action
discarded. A trash kept across sessions would keep every removed file, a video included, with no
undo left to reach it.

### An undo step holds its file moves beside its operations, and applies whole or not at all `##step-holds-its-moves`

A step holds the moves of the store's files its command made, `design@rootpoint-protocol@asset-trash`,
beside its operations, the moves first, and its revert runs them back in reverse order; a group of
`design@rootpoint-protocol@gesture-undo` gathers both. A step applies whole or not at all: one whose
move or operation fails moves back the files it moved and leaves the document and the view as they
were, so an undo whose move finds a file in the way replies `err` and keeps its step; only a move
back that fails in its turn leaves a file where it went, which the reply names. The rival, moves
made apart from the operations, would let a step that moved a file with no operation, or an
operation with no file, leave the document naming a file the store holds elsewhere.

### The asset table holds the assets added, and a file no asset names stays untracked `##table-of-added-assets`

An asset enters the table by a command, and by no change on disk: `import` adds one; `open`,
`open-copy`, `open-pack`, `sample` and `reload` bring the table of a file; and an undo brings back
one removed. A file under the root that no asset names is untracked, which `pack` leaves out,
`design@rootpoint-protocol@pack-container`, and which `assets untracked` lists, but for
`presentation.ron`, a pack's `mimetype`, and every file whose path has a component that starts with
`.`, as `.rootpoint/`, `.git/` and a `.gitignore`, the files of tools that no presentation shows;
such a path is still an asset path, so an asset may name one. The rival, a table that mirrors the files on disk,
`rejected@rootpoint-protocol@table-mirrors-the-disk`, gives an id to `.git/`, to a `README.md` and
to an editor's backup files, and makes each change on disk an edit nobody made.
`tripwire@rootpoint-protocol@untracked-file-lost-in-a-pack` watches an author who expected an
untracked file packed.

## Packs

### A pack is a zip whose first entry names its type, then the presentation and its assets' files `##pack-container`

A pack, of extension `.rpk`, is a zip file: first the entry `mimetype`, stored with no compression
and no extra field, holding `application/x-rootpoint`; then `presentation.ron`, deflated; then
each asset's file at its asset path, stored, a path two assets name written once. `pack <file>`
writes one from any store, reading each asset through
`design@rootpoint-protocol@store-confines-access`, so an asset it cannot read is left out and named
after `missing=`. `open-pack <file>` opens one in memory, `design@rootpoint-protocol@pack-refused-whole`.
The pack carries no version of its own: the RON in it names its format's version,
`design@rootpoint-model@save-format`.

A pack is the one file that carries a presentation and its assets, every font included, and opens
to the same presentation as its unpacked form, as `goal@rootpoint@readable-save-file` and
`goal@rootpoint-render@same-slide-everywhere` ask; and it opens and saves in the browser, as
`goal@rootpoint@runs-in-browser` asks, the zip crate built for the wasm target at the gate `wasm`
and a pack opened there by the browser test `a_picked_pack_opens_as_open_pack_does`. Any unzip
tool extracts it into the layout of a workspace, a `mimetype` file aside, and the zip's central
index names the place of each entry, so a reader can take one asset without the rest, though
`open-pack` reads them all today. On the owner's word: the session's
criterion that a pack unpacks without rootpoint is one the owner believed important. The first entry is EPUB's convention, read in EPUB 3.3, section 4.3.3: the
`mimetype` file is the container's first, neither compressed nor encrypted, with no extra field in
its header and no padding around the media type, so a reader tells a pack by its first bytes.

A pack adds only the zip's headers to its contents: at most 22 bytes, and, for each entry, 100
bytes and twice its name's length, beyond its assets' bytes, its deflated RON and its media type,
a bound the owner may reset. The test `a_pack_adds_only_the_zip_s_headers` of
`path@rootpoint-protocol@tests/pack.rs` measures 318 bytes for the sample, against 390. A RON
holding the bytes, the rival `rejected@rootpoint-model@bytes-in-the-file`, adds 20.3 bytes of text
per byte of asset, by that entry's measurement. The other rivals:
`rejected@rootpoint-protocol@own-pack-format` and `rejected@rootpoint-protocol@tar-pack`.

### `open-pack` refuses whole a file that is no pack, or a pack with one entry that cannot be placed inside the root `##pack-refused-whole`

`open-pack <file>` refuses a file whose first entry is not that `mimetype`, a pack with no
`presentation.ron`, and one with a symlink entry, an entry whose name, a directory's included,
fails `asset_path_is_confined`, or a file whose name is the folder of another, and leaves the store
as it was. A refusal tells the author the pack is broken or crafted: the rival, leaving such an
entry out of the map as `open-copy` leaves out a file it refuses, would open a presentation that
silently lacks an asset its pack carried, while a file that names a path out of the root, which
loads, `design@rootpoint-model@cosmetic-fallbacks`, shows the breach in its own text. A file whose
first entry is not the `mimetype` is no pack, `design@rootpoint-protocol@pack-container`. The
refusal costs nothing to a pack from a trusted source, which the confinement of
`design@rootpoint-protocol@store-confines-access` takes packs to be.

## The built-in sample

### Every build carries a sample presentation that `sample` opens `##builtin-sample`

The protocol crate holds a sample presentation, a RON text and its image files, included at build
time from `path@rootpoint-protocol@sample/`. `sample` replaces the session's document by it, in
memory, its images in the session's map at the paths its text names, with empty stacks, as
`open-copy` does, `design@rootpoint-protocol@editing-modes`. So the sample opens in the GUI and headless by one
command, and the command is the interface every client of the protocol uses. Its slides say how
to split, select, drag and swap, and its images stay under 50 KB in all.

On the owner's word: the owner wants the sample kept as a permanent feature of every build, a
tutorial and a demonstrator anywhere, not only in the browser. The rival, a file the app's `Files`
alone serves, would open in the GUI and in no headless session, against
`goal@rootpoint@headless-core`.
