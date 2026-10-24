# Rejected designs of rootpoint-protocol

The designs not kept for rootpoint-protocol, and why: one level-three heading per design, each cited
by the design head whose argument covers it, in `register@rootpoint-protocol@design`.

### Commands as JSON-RPC requests `##json-rpc-commands`

`live`. Each command as a JSON-RPC request, an object with a method, its parameters and an id. It
is easier for a program to parse, and harder for a person or an agent to type by hand at a terminal
or in a script: `{"jsonrpc":"2.0","method":"split","params":{"node":"n2","axis":"beside","at":0.5},"id":1}`
against `split n2 beside 0.5`.

### A gesture previewed on a copy of the document and committed once at its end `##preview-on-copy`

`live`. During the drag the session's document and the image on screen differ, so an agent
watching live sees the old divider until the release, and the survey of split editing,
`path@rootpoint@docs/grounding/split-editing.md`, assumed, without checking, that a preview then a
commit loses the live re-layout of the slide during the drag. It keeps the protocol unchanged, which is its strength.

### The session merging consecutive commands into one undo step `##merged-commands`

`live`. Two `divider` lines in a script, then `undo`, would revert both, which a reader of the
script cannot see. The survey of split editing found that merging by key needs a break rule, or
two separate drags of one divider merge, and that merging by time conflicts with replaying text
commands deterministically.

### An undo group per client of the session `##group-per-client`

`live`. Each client, the window and every connection of the control socket, would open its own
group in the session. A group of the window closed after a client's step lands above it, so its
undo reverts edits made before the client's: where the client changed the same text, that undo
writes the author's earlier text over the client's; where the client removed the area, the step
fails and the undo is refused. It reopens `design@rootpoint-protocol@gesture-undo`, an interface
of the protocol; `design@rootpoint-app@one-controller-at-a-time` keeps one group, held by one
controller at a time.

### The authority to edit taken by a client's first edit, with no verb `##implicit-control`

`live`. Nothing would tell the window that the agent is done, so the window would stay under the
agent's authority until the author clicks, and the grammar would have no command for it, which
every program that drives rootpoint consumes. `design@rootpoint-protocol@control-commands` makes
the authority a pair of verbs.

### Verbs of the authority read by the window's listener, outside the protocol's grammar `##listener-verbs`

`live`. They would make a second grammar beside the one
`design@rootpoint-protocol@protocol-transport` calls the crate's interface to every program that
drives rootpoint, and a script holding them would not run headless.
`design@rootpoint-protocol@control-commands` puts them in the grammar.

### A `release` that closes the client's open group, as `end` does `##release-closes-group`

`live`. It adds a closer to the list of `design@rootpoint-protocol@gesture-undo`, which the
protocol's clients consume, and when only the window closed the group and a headless session
changed nothing, a script's replies differed between the two hosts.
`design@rootpoint-protocol@control-commands` refuses a `release` while a group is open.

### A click in the preview that does nothing `##click-does-nothing-in-preview`

`live`. The preview would stay on until its button or Esc ends it, at the cost of one more click
each time the author leaves it to change something seen. The owner proposed the click that leaves
it and selects while not fully certain, so doubt remains that the rule kept serves the author
better, `design@rootpoint-protocol@final-preview`; its tripwire brings this design back.

### A save to any path, copying the assets beside the saved file and rewriting their paths `##free-save-path`

`live`. Every asset path was relative to a directory that each save and load changed, so a save
compared paths of two bases: a relative and an absolute form of one directory never matched, and
a save copied an asset already inside its target beside itself, a defect reproduced in the issue
that recorded it, whose text is read by
`git show 5742b00:crates/rootpoint-protocol/docs/open-issues/save-mixes-relative-and-absolute.md`.
The paths the undo and redo stacks held were rewritten at each save, with the asset's bytes where no
path from the new directory reached its file. The command `save <path>` was its interface.

### An asset table that mirrors the files on disk `##table-mirrors-the-disk`

`live`. Every file under the root would be an asset: `.git/`, a `README.md` and an editor's backup
files would get ids, each file an editor writes would be an edit nobody made, and a pack would
carry them all. Untracked files are the doubt the table of added assets leaves open.

### A pack in a binary format of rootpoint's own `##own-pack-format`

`live`. It does what a zip does, an index of named entries, and only rootpoint opens it, where any
unzip tool opens a zip into the layout of a workspace.

### A pack as a tar file `##tar-pack`

`live`. A tar has no index, so a reader reads the whole file to find one asset, where a zip's central
index names the place of each entry.
