# Milestone: asset storage and presentation packing

## Status and audience

This is the milestone document of how a presentation and its assets are stored, opened, saved,
packed and shown:
- two editing modes, a presentation held in memory and a workspace, a directory on disk;
- a workspace's fixed layout, `presentation.ron` at its root and `.rootpoint/` beside it, with a
  lock that keeps a workspace to one process and a control socket that starts with the window;
- assets named by a path inside the presentation's root, with no bytes in the RON;
- autosave of a workspace, guarded against overwriting a change made outside rootpoint, and assets
  read again when their files change;
- the commands and the window's tab that manage the asset tree;
- a pack, a zip file that holds a presentation and its assets, opened and saved in the native and
  the browser builds.

It is written for the sessions that implement its slices, which did not witness the discussion that
produced it. Each slice's spec leaves when that slice lands; this document leaves with the last
slice, in the commit that completes its harvest.

- Where this document, or a slice's spec, and a design home disagree on a point that "What is
  already decided" does not list as reversed or rewritten, the design home wins.
- Every name it uses is defined in it, in "Names" or "New names, in one place", or exists in the
  code.
- Where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says what
  waits. The defaults awaiting the owner are listed in their own section.
- The milestone was assembled from the transcript of one session. The discussion begins in it at
  the owner's message that opens "I want to discuss and converge on the mechanism of asset storage
  and presentation packing."

**The owner's messages of the discussion**, verbatim, in order, up to its convergence. Round n is the
owner's message n and the session's reply to it.

**Round 1**

> I want to discuss and converge on the mechanism of asset storage and presentation packing. Here are my ideas, to challenge or refine:
>
> I want to have two editing modes available:
> - a fully "in-memory" mode, where the file and content is only stored in the process memory, which would be the only mode available for the web build
> - a "workspace" mode: a directory is opened as a workspace, the content is synchronized with the render (cached and watching for changes), the slide layout/text/styles are saved in a text-only ron file (with a fixed name at root of workspace), images and other asset files are automatically kept in sync with the filesystem, and the text protocol is automatically started under a .rootpoint/ subdir of the workspace directory. A client should also lock the workspace to avoid another client taking it, through a lock in .rootpoint/, at least as long as we are not supporting more fine grained locks (per slide or node).
>
> - assets stored in a filesystem like tree, embedded by packing, unpacking gives layout with a pure text ron file and the full filesystem tree. To go with an asset manager screen. Would allow editing the file tree, search, rename, filter unused assets, any other meaningfull feature that comes to your mind.
>
>
> To go with that, I want to separate serialization of the slides in two different modes.
> Basically, the assets and the .ron that describes layout/style/text should be serializable alone, as a text only file. The standard way to serialize assets is writing the asset tree to the filesystem beside the ron file (the workspace shape), while the .ron file only describes layout and assumes asset content is right (with proper missing content handling when opened as workspace mode).
> Then, the other mode is "packed" mode: a binary file that holds both the assets and the layout description together, easier to transport and share, and which can be unpacked.

**Round 2**

> editing-modes accepted. i belive this shape is the most convenient and with the least drawbacks. it will be important to properly convey which state we are in and what each action do from the ui. also, maybe an explanation slide in the sample (which serves as a tutorial) could be useful.
>
> workspace-layout: accepted. two talks can no longer share images, i had not seen that comming, but this is a rare case anyway imo. cost accepted.
>
> workspace-lock: accepted
>
> socket-place: i did not know the lenght limit. maybe instead use a socket under XDG_RUNTIME_DIR, and give it the name of the directory containing the presentation.ron (the name of the workspace, and also of the talk), with a lock to avoid multiple clients using the same. .rootpoint would contain a file describing status of the socket (active or not, path  to it). Or maybe the text protocol mcan move away from using a socket, to remove the limitation. any other shape would allow what we do ?
>
> asset-paths-only: accepted
> asset-tree-shape: accepted
>
> asset-disk-changes: shape A. with autosave time measured (including scalability)
>
> asset-watch: i agree on watching assets ( i wanted it), but i see no reason to watch presentation.ron. the correct way to edit is either text protocol or gui IMO. do you have a strong reason to pushback here?
>
>
> asset-manager-scope: agreed on proposed features
>
> pack-container: approved
>
>
> browser-pack-save: accepted. this was part of my goal, closing design@rootpoint-app@browser-load-only, which was only a temporary measure while i was putting more thought on the asset management mechanism
>
>
> Q1 yes, autosave, probably as a background thread.
> Q2 yes, keep the in memory mode avail in native.
> Q3: yes, it should save
> Q4: i belive this is important. there does not seem to be too much of a cost using zip anyway, apart from needing it on wasm

**Round 3**

> ron-overwrite-guard: i accept this shape
>
> socket-place: shape approved, good refinements
>
> sample-explains-modes and unsaved-close-warning accepted. though, the web build could actually autosave, using db storage (not sure of exact name, but not local storage which has low size limits). maybe a todo issue for later
>
> C9 accepted
>
> ac1 accepted

**Round 4**

> agreed on tripwires and AC, keep all.
>
> about the malicious pack: powerpoint or anything else has the same problems anyway, packs are meant to be shared with and from trusted sources, security features can be designed and added later.
> one detail though: the project should not accept relative path, and especially not one sitting outside the workspace directory. the assets have to stay confined inside the workspace. importing from outside copies the file in the workspace.

The Q1 to Q4 the owner answers in round 2 are the questions of the session's reply of round 1,
numbered within that reply; the threads below cite them as "Q1 of round 1's reply". The "C9" and
the "ac1" the owner rules in round 3 are #c-visible-mode, whose form the session's reply of round 2
put as its Q1, and #ac-autosave-gui-cost, that reply's AC1. The session's replies named the criteria
C1 to C9, in the order of the section "Criteria" below; "Names" maps each.

## How the work is done

Before each slice, read `skill@knowledge-architect-planning@working-a-slice` whole, and follow it.
The slices are built in the order of "Implementation sequence". On the owner's word, given when the
implementation started, they are built on one branch, named after this milestone, which merges as
`instructions@git-workflow` says after the last slice: each slice takes its audit, its work, its
harvest and then one review, as the review before a merge would be, with the transcript axis run
alone after the others; the repairs of a slice's review are reviewed again only at the end of the
milestone. This replaces, for the branches, the one branch per slice of D1 and of
`skill@knowledge-architect-planning@cutting-steps-and-slices`. On the same word, the work pauses for the
owner only where the owner's word is needed to fill a gap, and otherwise goes on with sensible
defaults, each reported to the owner. The relocation axis of the first slice's review could not
read the diffs of `path@rootpoint-app@tests/actions.rs` and `path@rootpoint-app@tests/control.rs`;
the review at the end of the milestone reads them.

- **Gates.** `cargo x gates` runs every check a branch owes before it merges,
  `instructions@merge-gates`. Every slice keeps them passing, the gate `wasm` and the browser tests
  of the gate `wasm-test` included: a test added to `path@rootpoint-app@tests/actions.rs` or
  `path@rootpoint-app@tests/web.rs` builds and passes on the wasm target, with no file system and
  no thread, as `path@rootpoint-app@CLAUDE.md` says.
- **Actions are commands.** Every new action of the window is a function of
  `path@rootpoint-app@src/actions.rs` that issues protocol commands through `App::run`, checked
  against the same commands on a second session in `path@rootpoint-app@tests/actions.rs`, per
  `design@rootpoint-app@actions-are-commands`. Every new command parses back from its printed form,
  per `design@rootpoint-protocol@protocol-transport`.
- **No compatibility.** A slice that changes the save format bumps `FORMAT_VERSION` in
  `path@rootpoint-model@src/file.rs`, 3 at the time of writing, and writes no migration, per
  `design@rootpoint@no-compat-in-initial-dev`. The committed `.ron` files are rewritten in the new
  format in the same slice: the four `find crates -name '*.ron'` lists at the time of writing,
  `path@rootpoint-protocol@sample/sample.ron`, `path@rootpoint-app@tests/data/mixed.ron`,
  `path@rootpoint-model@tests/data/fixture.ron` and
  `path@rootpoint-model@tests/data/fixture-edited-by-agent.ron`. The fixture is written again by
  running its test `the_fixture_round_trips_and_matches_its_committed_file` with
  `ROOTPOINT_BLESS=1`; the others are edited by hand. A change of the protocol's grammar keeps no
  old command either.
- **This milestone's branch** carries this document and its slice specs, the issue the discussion
  asked for, `issue@rootpoint-app@browser-autosave`, and the closing of the five issues the
  milestone schedules, listed in "What is already decided". It merges once the owner has ruled on
  every default awaiting the owner that blocks it, before the first slice, #rooted-assets, begins.

## Names

- **The session**: `Session` of `path@rootpoint-protocol@src/session.rs`, which holds the document,
  the undo and redo stacks and the view.
- **The host's files**: the `Files` value a host passes to each command, trait `Files` of
  `path@rootpoint-protocol@src/files.rs`: `NativeFiles` in `path@rootpoint-app@src/main.rs` and in
  the headless binary, `MemoryFiles` of `path@rootpoint-app@src/memory.rs` in the browser build.
- **The store**: where the session finds the files of its presentation, `Store` in "New names, in
  one place": a map the session holds, in memory, or a root on the host's files, a workspace.
- **In memory**, **in-memory mode**: the session's store is its own map of paths to bytes. No file of
  the presentation exists on disk until a save.
- **A workspace**, **workspace mode**: the session's store is a directory of the host's files, its
  root; the presentation is `presentation.ron` at the root, its assets are files under it.
- **The root**: the directory a store's paths are relative to: the workspace's directory, stored
  canonical. A map has no root: `Session::root` gives none in memory, and the map's keys are the
  asset paths themselves.
- **An asset path**: the path of an asset relative to the root, in `/`-separated components, for
  which `asset_path_is_confined` holds, "Decided design" of #asset-paths-only,
  `thread@asset-storage@asset-paths-only`. No command makes another one; a file edited by hand can
  name another one, a cosmetic breach no access reaches, by D4.
- **Confinement**: the rule that no access to an asset reaches outside the root, on the owner's
  word of round 4 and D4.
- **A pack**: a zip file of extension `.rpk`, `thread@asset-storage@pack-container`.
- **`.rootpoint/`**: the directory of a workspace that holds what is the machine's, not the
  presentation's, `thread@asset-storage@workspace-layout`; it is never packed.
- **The workspace lock**: `File::try_lock` held on `.rootpoint/lock`,
  `thread@asset-storage@workspace-lock`.
- **The runtime directory**: `$XDG_RUNTIME_DIR/rootpoint/`, `/run/user/1000/rootpoint/` on the
  owner's machine, where the control sockets are, `thread@asset-storage@socket-place`.
- **The trash**: where the bytes of a removed asset wait for an undo,
  `thread@asset-storage@asset-disk-changes`: `.rootpoint/trash/` in a workspace, `Session::trash` in
  memory.
- **The guard**: the comparison before each autosave of `presentation.ron`'s bytes with those the
  editor last wrote, `thread@asset-storage@ron-overwrite-guard`.
- **The asset tab**: the tab of the side panel that manages the asset tree,
  `thread@asset-storage@asset-manager-scope`, beside the Styles tab of
  `design@rootpoint-app@styles-tab-edits-styles`.
- **The state label**: the text of the window that names the editing mode and whether the edits are
  saved, `criterion@asset-storage@c-visible-mode`, given by `App::state_label`, at the right end of
  the toolbar, `Panel::top("toolbar")` in `App::editor`, `path@rootpoint-app@src/lib.rs`.
- **`DiskStore`**: the discussion's word for a workspace with what its host adds, the lock, the
  socket, the autosave and the watcher. No type has the name; "Decided design" of #editing-modes.
- **`MemoryFiles`**: the app's, of `path@rootpoint-app@src/memory.rs`. The protocol's tests have a
  `MemoryFiles` of their own, in `path@rootpoint-protocol@tests/common/mod.rs`, which a protocol
  test uses; this document names that one as "the protocol tests' `MemoryFiles`".
- **The save-mixes defect**: the closed issue `save-mixes-relative-and-absolute`, whose text is read
  by `git show 5742b00:crates/rootpoint-protocol/docs/open-issues/save-mixes-relative-and-absolute.md`.
  The texts of the four other closed issues are read the same way, at their paths on that commit.
- **C1 to C9**: the session's labels of the criteria, in order: #c-readable-pack,
  #c-browser-pack, #c-headless-commands, #c-no-host-in-core, #c-undoable-assets, #c-fonts-in-pack,
  #c-pack-overhead, #c-unpack-anywhere, #c-visible-mode. "Measurement 1" of round 1 is #arg-1.
- **T1 to T4, AC1 to AC5**: the labels under which the premortem's tripwires and acceptance
  criteria were put to the owner, "Premortem" below.
- **D1 to D5**: the labels of the defaults put to the owner, "Defaults awaiting the owner" below.

## What the work is

**What exists today**, read in the tree at the commit this document is added on:

- An asset is `Asset { source: AssetSource }`, and `AssetSource` is `Path(String)`, a path
  relative to the presentation file, or `Embedded(Vec<u8>)`, in
  `path@rootpoint-model@src/document.rs`. RON writes `Embedded` as a list of decimal numbers, one
  per line: one asset of 580 bytes added 11,768 bytes to the RON, 20.3 bytes per byte (measured in
  round 1, on the sample's `splits.png`).
- The session has a directory, `Session::directory`: the parent of the path last given to `load`,
  as given, or to `save`, with `.` and `..` removed; the empty path after `new` and `sample`. `load <path>` and `save <path>` read and write one `.ron` file through the
  host's files; `Session::save` copies every asset outside the target's directory, and every
  embedded one, beside the saved file, per `asset-save-rule`, its shape now `rejected@rootpoint-protocol@free-save-path`, and
  rewrites the paths in the undo and redo stacks, per
  `save-updates-sources`, now `rejected@rootpoint-protocol@free-save-path`, the one place outside the sample that writes
  `Embedded`.
- `sample` opens the protocol crate's sample, `path@rootpoint-protocol@sample/`, its images as
  `Embedded` assets, per `design@rootpoint-protocol@builtin-sample`.
- `asset <path>`, `Command::Asset` of `path@rootpoint-protocol@src/command.rs`, adds an asset by a
  path relative to the session's directory, in `Session::run`; `remove-asset` removes one that
  nothing uses, `Op::RemoveAsset` of `path@rootpoint-model@src/ops.rs` refusing one in use.
- The window, `path@rootpoint-app@src/main.rs`, is `rootpoint [--control <socket>]
  [presentation.ron]`; with `--control` it listens on a Unix socket at that path,
  `rootpoint_app::control::Listener` of `path@rootpoint-app@src/control.rs`, per
  `design@rootpoint-app@control-socket`. `rootpoint-ctl <socket> [<words>…]`,
  `path@rootpoint-protocol@src/bin/rootpoint-ctl.rs`, sends commands to it.
- `App::relative_if_absolute` of `path@rootpoint-app@src/actions.rs` makes every path the window
  gives `load` and `save` relative to the working directory, a guard against the defect the
  milestone closes, `save-mixes-relative-and-absolute`.
- The browser build, `App::in_memory`, opens the sample, or a directory the author picks, whose
  files `App::open_directory` holds in `MemoryFiles`; it saves nothing, per
  `design@rootpoint-app@browser-load-only`.
- `Assets::refresh` of `path@rootpoint-app@src/assets.rs` decodes the assets again only when the
  session's directory, or an asset's source or use, changes.

**What is outside the work**, each with what owns it:

- A store of the browser build that keeps its edits between two pages:
  `issue@rootpoint-app@browser-autosave`, opened on this milestone's branch, on the owner's word of
  round 3.
- A lock finer than the whole document, per slide or per node: `issue@rootpoint-app@wider-lock`,
  which stays open. The workspace lock is between processes; the authority to edit inside one
  window stays `design@rootpoint-app@one-controller-at-a-time`.
- Protection of a pack from a source that is not trusted, beyond confinement: on the owner's word
  of round 4, "security features can be designed and added later". Confinement is in the work.
- Video, and every other content kind: `issue@rootpoint@more-content-kinds`. Nothing of the work
  assumes that an asset is an image or a font.
- Compatibility of files once the project is in real use:
  `issue@rootpoint@compatibility-after-real-use`.
- Windows: the control socket is built on Unix only, per `design@rootpoint-app@control-socket`, and
  stays so.

## What is already decided

**The decisions the design rests on**, which it does not argue again:
`design@rootpoint@no-compat-in-initial-dev`, `design@rootpoint@wasm-from-the-start`,
`design@rootpoint@gui-only-in-app`, `design@rootpoint-model@edit-operation-core`,
`design@rootpoint-model@stable-ids`, `design@rootpoint-model@inline-content-assets`,
`design@rootpoint-model@save-format`, `design@rootpoint-protocol@protocol-transport`,
`design@rootpoint-protocol@protocol-addressing`, `design@rootpoint-protocol@no-new-breach`,
`design@rootpoint-protocol@control-commands`,
`design@rootpoint-app@actions-are-commands`, `design@rootpoint-app@one-controller-at-a-time`,
`design@rootpoint-app@styles-tab-edits-styles`,
`design@rootpoint-app@gui-toolkit`, `design@rootpoint-render@caller-supplies-assets`,
`design@rootpoint-render@font-sources`.

**The decisions the work reverses or rewrites**, with every text `cargo klarch show` lists as
referencing each, and the slice that updates it:

| decision | outcome | referencing texts, and the slice that updates each |
| --- | --- | --- |
| `asset-source`, now `design@rootpoint-model@asset-paths-only` | reversed by #asset-paths-only: an asset is a path, `Embedded` leaves | `register@rootpoint-protocol@design` line 178 (#rooted-assets); `pack-and-unpack`, closed on this branch |
| `design@rootpoint-model@image-storage` | rewritten by #editing-modes and #workspace-layout; its owner's ground, assets as files beside a readable text, and packing as the final goal, stands | `register@rootpoint-app@design` line 523 (#workspace-host, and #browser-packs where it names the browser) |
| `asset-save-rule`, now `rejected@rootpoint-protocol@free-save-path`, and `design@rootpoint-protocol@import-copies-into-the-store` for the import | reversed: no save copies assets; an import copies a file from outside into the store | `register@rootpoint-protocol@design` line 210 (#rooted-assets); `pack-and-unpack`, closed on this branch |
| `save-updates-sources`, now `rejected@rootpoint-protocol@free-save-path` | reversed: the root never changes during a session, so no save rewrites a path | none |
| `design@rootpoint-protocol@builtin-sample` | rewritten: `sample` opens in memory, its images in the map; its owner's ground stands | none |
| `design@rootpoint-protocol@session-files` | rewritten: the session reads through the host's files under a workspace's root, or from its map | `register@rootpoint-app@design` line 537, `path@rootpoint-protocol@CLAUDE.md` line 6, `path@rootpoint-protocol@src/files.rs` line 3, `register@rootpoint@design` line 43 (#rooted-assets); `save-mixes-relative-and-absolute`, closed on this branch |
| `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch` | rewritten by #socket-place: `rootpoint-ctl` takes a workspace or a socket, and `list` | `register@rootpoint-app@design` line 508, `path@rootpoint-protocol@tests/ctl.rs` line 3 (#workspace-host) |
| `design@rootpoint-app@browser-load-only` | reversed by #browser-pack-save, on the owner's word of round 2 | `register@rootpoint-app@design` lines 541 and 555, `path@rootpoint-app@src/actions.rs` line 748, `path@rootpoint-app@src/memory.rs` line 4, `path@rootpoint-app@src/panels.rs` line 43, `path@rootpoint-app@tests/actions.rs` line 528 (#browser-packs); `page-close-loses-edits` and `pack-and-unpack`, closed on this branch |
| `design@rootpoint-app@picked-files-by-path` | rewritten: a picked directory opens in memory, its files copied into the map | `path@rootpoint-app@src/actions.rs` line 741, `path@rootpoint-app@src/memory.rs` line 3 (#rooted-assets); `changed-asset-not-read-again`, closed on this branch |
| `design@rootpoint-app@control-socket` | rewritten by #socket-place: the socket starts with the window, in the runtime directory; `--control` leaves | `path@rootpoint-app@CLAUDE.md` line 56, `issue@rootpoint-app@socket-waits-for-a-frame`, `path@rootpoint-app@src/control.rs` line 1, `path@rootpoint-app@tests/control.rs` line 1, `path@rootpoint-app@tests/listener_drop.rs` line 1, `path@rootpoint-app@tests/presenter.rs` line 233 (#workspace-host) |
| `design@rootpoint-app@only-a-stale-socket-is-replaced` | rewritten by #workspace-lock: a workspace's socket is bound only under its lock, so a second window is refused before its bind; the refusal of a file that is no socket stands | its own last sentence, which cites `spec@asset-storage@workspace-host` (#workspace-host); `two-windows-bind-one-path`, closed on this branch |
| `design@rootpoint-app@no-discard-confirmation` | its title contradicted by #unsaved-close-warning; its ground is the owner's, so D3 puts the rewrite to the owner | its own line 272, which cites `issue@rootpoint-app@browser-autosave`, and `register@rootpoint-app@rejected` line 93 (#workspace-host, on D3) |
| `design@rootpoint-protocol@gesture-undo` | rewritten by #asset-disk-changes: a step holds its file moves, and `load` leaves its list of the commands that close a group | none (#asset-commands; `load` left its list at #rooted-assets) |
| `design@rootpoint-model@cosmetic-fallbacks` | its test widened by #asset-paths-only, on D4: a cosmetic breach is a reference to a missing target, or an asset whose path fails confinement, each with its stand-in; the owner's principle, its ground, stands | `register@rootpoint-model@design` lines 165 and 216, `path@rootpoint-model@src/ops.rs` lines 295 and 311, `path@rootpoint-model@src/validate.rs` line 3 and its module and `Breach` docs, `register@rootpoint-protocol@design` line 44 (#rooted-assets) |
| `rejected@rootpoint-app@socket-holds-control-at-start` | names `--control`, which leaves; its argument stands | its own text (#workspace-host) |
| `design@rootpoint-render@caller-supplies-assets` | reworded: an asset's path is relative to a root only the caller knows | `register@rootpoint-protocol@design` line 140, `path@rootpoint-render@CLAUDE.md` line 9 (#rooted-assets) |

**The issues this milestone schedules**, closed in the commit that adds this document, with each
site that cited them re-pointed to this milestone, to the slice that carries their work, or to the
issue that now records what stays outside it:

| issue | what of it the milestone carries |
| --- | --- |
| `pack-and-unpack` | #pack-commands and #browser-packs |
| `save-mixes-relative-and-absolute` | #rooted-assets; its reproduction is a claim of that slice |
| `changed-asset-not-read-again` | #autosave-and-watch |
| `two-windows-bind-one-path` | #workspace-host; its measured race is a claim of that slice |
| `page-close-loses-edits` | #workspace-host (native) and #browser-packs (browser), #unsaved-close-warning |

**The standing entries the discussion's search returned**, with their outcome:
`issue@rootpoint@compatibility-after-real-use` bears on nothing in the work: the project is in
initial development, and the format changes owe no migration.
`issue@rootpoint-app@wider-lock` stays open, as "What is outside the work" says.
`issue@rootpoint@more-content-kinds` bears as a constraint: no slice assumes an asset is an image or
a font. `issue@rootpoint@thorough-testing` bears on #pack-commands: the reader of a pack is a second loader,
covered by that issue's open question on fuzzing loaders. The five issues above are scheduled.
Every other issue and tripwire it read bears on nothing.

**The design and `rejected` entries the design-entry search graded `bears`**, with their outcome:
the goals that bind are the criteria below. `design@rootpoint@crate-existence-rule` bears: no slice
adds a crate; the pack's code goes in the protocol crate, beside the session that reads and writes
it; the workspace's lock and the socket's paths in a native-only module of the protocol crate,
which the headless binary, `rootpoint-ctl` and the window share; the watcher and the autosave in
the app. `design@rootpoint-model@save-format`,
`design@rootpoint-model@inline-content-assets`, `design@rootpoint-model@stable-ids`,
`design@rootpoint-model@validated-documents`,
`design@rootpoint-model@edit-operation-core`, `design@rootpoint-protocol@protocol-addressing`,
`design@rootpoint-protocol@no-new-breach` and `design@rootpoint-app@no-discard-confirmation` bear
as constraints the work keeps. `design@rootpoint-app@tree-outline` bears on #asset-manager: its row
of an image shows its file's name, which becomes its asset path; `design@rootpoint-app@node-menu`
lists each image asset, which the code labels by its file's name, `App::asset_name` of
`path@rootpoint-app@src/panels.rs`. The entries this table lists as reversed or rewritten bear as it says.

## Criteria

### A presentation is readable text with its assets beside it, and a pack opens to the same presentation `##c-readable-pack`

Binding, from `goal@rootpoint@readable-save-file` and `goal@rootpoint-model@faithful-readable-file`.
Met by #asset-paths-only, #pack-container and #workspace-layout.

### A pack opens and saves in the browser build, with the same operations as the native build `##c-browser-pack`

Binding, from `goal@rootpoint@runs-in-browser` and `goal@rootpoint-app@browser-window`. Met by
#editing-modes, #pack-container and #browser-pack-save.

### Every asset action, pack, unpack and open is a command that runs headless `##c-headless-commands`

Binding, from `goal@rootpoint@headless-core` and
`goal@rootpoint-protocol@commands-drive-everything`. Met by #asset-manager-scope and #editing-modes.

### No watcher, lock, socket or `std::fs` in the model, the renderer or the session `##c-no-host-in-core`

Binding as a presumption, from `design@rootpoint-protocol@session-files`,
`design@rootpoint-render@caller-supplies-assets`, `goal@rootpoint-model@browser-buildable`,
`goal@rootpoint-protocol@browser-buildable` and `goal@rootpoint-render@browser-buildable`. Met by
#editing-modes: the lock, the socket, the watcher and the autosave are the app's.

### Every asset change is an operation with its inverse, and undo restores a removed asset `##c-undoable-assets`

Binding as a presumption, from `design@rootpoint-model@edit-operation-core` and
`design@rootpoint-app@no-discard-confirmation`. Met by #asset-disk-changes, its trash, and
#asset-tree-shape.

### A pack carries every font its slides use `##c-fonts-in-pack`

Binding, from `goal@rootpoint-render@same-slide-everywhere`. Met by #pack-container: a pack holds
every asset of the table, fonts included.

### A pack adds to its assets' bytes and its deflated RON only the zip's headers `##c-pack-overhead`

Weighed, named by the session in round 1 as "within 1%", a form a small presentation cannot meet:
the sample's pack, its one image of 580 bytes, is 2,129 bytes, measured with Python's `zipfile`
writing the three entries as #pack-container orders them, on the owner's AMD Ryzen 9 9950X. Its
checkable form: the pack's size, less its assets' bytes, the bytes of its deflated
`presentation.ron` and the 23 bytes of `PACK_MIMETYPE`, is at most 22 bytes plus, for each entry,
`mimetype` included, 100 bytes and twice the length of its name; the same measurement gives 318
bytes for the sample against a bound of 390; a zip's local and central headers take 76
bytes and the name twice, and 16 bytes allow a data descriptor. The bound is the session's default,
the owner's to reset. Met by #pack-container. A RON with embedded bytes fails it, at 20.3 times the
assets' bytes.

### A pack can be unpacked without rootpoint `##c-unpack-anywhere`

Weighed, named by the session in round 1 as C8, and weighed by the owner in round 2: "i belive this
is important". Met by #pack-container.

### The window shows which state it is in, and where each action writes `##c-visible-mode`

Binding, the owner's requirement of round 2: "it will be important to properly convey which state
we are in and what each action do from the ui". Its checkable form, put in round 2's reply and
ruled "C9 accepted" in round 3:

1. the window always shows one of three states: "In memory, not saved"; "In memory, saved to
   `<file>.rpk`, n changes since"; or "Workspace `<path>`, saved";
2. every action that writes names where it writes: "Save as pack…", "Save as workspace…",
   "Pack to…", "Unpack to…";
3. a test of `App::frame` reads the state label in each of the three states.

Met by #editing-modes and #unsaved-close-warning; built at #workspace-host and #browser-packs.

## Threads

### The editing mode is the store the session reads its presentation from: its own map, or a workspace's root `##editing-modes`

Proposed by the session in round 1, as one session over two stores, a map in memory and a
directory on disk, every asset path relative to the store's root. Approved by the owner in round 2:
"editing-modes accepted. i belive this shape is the most convenient and with the least drawbacks.
it will be important to properly convey which state we are in and what each action do from the
ui." Q2 of round 1's reply, "Q2 yes, keep the in memory mode avail in native.", keeps in-memory mode in the
native build. The owner's condition is #c-visible-mode; the owner's suggestion of a slide in the
sample is #sample-explains-modes. Arguments: #arg-2, #arg-3, #arg-4. Shape: "Decided design" below
and #rooted-assets. Harvest: a head of the protocol, rewriting
`design@rootpoint-protocol@session-files`, at #rooted-assets.

### A workspace is a directory with `presentation.ron` at its root and `.rootpoint/` for the machine `##workspace-layout`

Proposed by the owner in round 1 ("a text-only ron file (with a fixed name at root of workspace)",
"under a .rootpoint/ subdir"); the session named the file `presentation.ron` and listed what
`.rootpoint/` holds, its `.gitignore` included. Approved in round 2: "workspace-layout: accepted.
two talks can no longer share images, i had not seen that comming, but this is a rare case anyway
imo. cost accepted." Arguments: #arg-5. Shape: "Decided design". Harvest: a head of the model,
rewriting `design@rootpoint-model@image-storage`, at #workspace-host.

### A workspace is held by one process, by `File::try_lock` on `.rootpoint/lock` `##workspace-lock`

Proposed by the owner in round 1 ("A client should also lock the workspace ... through a lock in
.rootpoint/"); the session proposed the mechanism, an OS lock the kernel releases at the process's
end. Approved in round 2: "workspace-lock: accepted". Arguments: #arg-6. Shape: "Decided design"
and #workspace-host. Harvest: a head of the protocol, at #workspace-host. Guarded by T1.

### The control socket starts with the window, in the runtime directory, named by the workspace and a hash of its path `##socket-place`

Proposed by the owner in round 1, under `.rootpoint/`. The session's reply of round 1 measured the
limit of a socket's path and proposed `.rootpoint/control.sock` refused when too long, with the
runtime directory as rival. The owner proposed, in round 2: "maybe instead use a socket under
XDG_RUNTIME_DIR, and give it the name of the directory containing the presentation.ron (the name
of the workspace, and also of the talk), with a lock to avoid multiple clients using the same.
.rootpoint would contain a file describing status of the socket (active or not, path  to it). Or
maybe the text protocol mcan move away from using a socket, to remove the limitation. any other
shape would allow what we do ?" The session refined it in round 2's reply: a hash of the path in
the name, no flag of state in the pointer file, no lock of its own, an in-memory window's socket,
`rootpoint-ctl list`; and compared five shapes. Approved in round 3: "socket-place: shape approved,
good refinements". Arguments: #arg-7, #arg-18, #arg-19. Shape: "Decided design" and
#workspace-host. Harvest: rewrites of `design@rootpoint-app@control-socket` and
`design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`, and `rejected` entries for the socket
inside `.rootpoint/` and for the abstract namespace and TCP, at #workspace-host, each by the tests
of `skill@knowledge-architect-decision-recording`.

### An asset is a path inside the root, and the RON holds no bytes `##asset-paths-only`

Proposed by the session in round 1, from the owner's "a text-only ron file" and the measurement of
#arg-1. Approved in round 2: "asset-paths-only: accepted". The owner's word of round 4 adds
confinement: "the project should not accept relative path, and especially not one sitting outside
the workspace directory. the assets have to stay confined inside the workspace. importing from
outside copies the file in the workspace." Read, in the session's reply of round 4, as: an asset
path is never absolute, has no `..`, and leads out of the root by no symlink; a file from outside
is imported by a copy into the store. Arguments: #arg-1, #arg-8, #arg-23. Shape: "Decided design"
and #rooted-assets. Harvest: reversal of `asset-source`, now `design@rootpoint-model@asset-paths-only`, with a `rejected`
entry for the bytes embedded in the RON, and the widened test of
`design@rootpoint-model@cosmetic-fallbacks`, at #rooted-assets.

### Assets keep their ids, folders come from their paths, and the table holds only what was added `##asset-tree-shape`

Proposed by the session in round 1. Approved in round 2: "asset-tree-shape: accepted". Arguments:
#arg-9. Shape: "Decided design". Harvest, at #asset-commands: three decisions, each judged by the
tests of `primer@design-heads` on its own, since each lost to its own rival: assets keep their ids,
first read as a member of `design@rootpoint-model@stable-ids`; folders come from the paths; the
table holds only what was added. `rejected` entries for content naming an asset by its path, for
folders stored in the RON, a change of the file's format, and for a table that mirrors the disk.

### An action on an asset changes the disk at once, and a workspace saves itself after every undo step `##asset-disk-changes`

Proposed by the session in round 1, as shape A against shape B, an explicit save. Q1 of round 1's reply,
"Q1 yes, autosave, probably as a background thread.", and in round 2: "asset-disk-changes: shape A.
with autosave time measured (including scalability)". The measurement is #arg-16; AC1 holds the
owner's condition. Arguments: #arg-16, #arg-10. Shape: "Decided design", #autosave-and-watch and
#asset-commands. Harvest: heads of the app (autosave) at #autosave-and-watch and of the protocol
(the trash) at #asset-commands, with a `rejected` entry for shape B.

### The window reads an asset again when its file changes, and does not watch `presentation.ron` `##asset-watch`

Proposed by the session in round 1, for the assets and for `presentation.ron`. Round 2: "asset-watch:
i agree on watching assets ( i wanted it), but i see no reason to watch presentation.ron. the
correct way to edit is either text protocol or gui IMO. do you have a strong reason to pushback
here?" The asset part closed approved; the session withdrew the reload of `presentation.ron` on a
change and answered with #ron-overwrite-guard. Arguments: #arg-11, #arg-15. Shape: #autosave-and-watch.
Harvest: a head of the app at #autosave-and-watch.

### An autosave writes nothing over a `presentation.ron` changed outside rootpoint, and asks `##ron-overwrite-guard`

Proposed by the session in round 2's reply, in answer to the owner's question in #asset-watch.
Approved in round 3: "ron-overwrite-guard: i accept this shape". Arguments: #arg-17. Shape: "Decided
design" and #autosave-and-watch. Harvest: a head of the app at #autosave-and-watch. Guarded by T2.

### The asset tab and the asset commands `##asset-manager-scope`

Proposed by the owner in round 1 ("an asset manager screen. Would allow editing the file tree,
search, rename, filter unused assets, any other meaningfull feature that comes to your mind."); the
session listed the features. Approved in round 2: "asset-manager-scope: agreed on proposed
features". Arguments: #arg-12. Shape: "Decided design", #asset-commands and #asset-manager.
Harvest: heads of the protocol at #asset-commands and of the app at #asset-manager.

### A pack is a zip whose first entry names its type, and whose other entries are the workspace's files `##pack-container`

The pack is the owner's proposal of round 1 ("a binary file that holds both the assets and the
layout description together, easier to transport and share, and which can be unpacked"); the zip
container is the session's. Approved in round 2: "pack-container: approved", and Q4: "i belive this
is important. there does not seem to be too much of a cost using zip anyway, apart from needing it
on wasm". Arguments: #arg-1, #arg-13. Shape: "Decided design" and #pack-commands. Harvest: a head of the
protocol at #pack-commands, with `rejected` entries for a binary format of rootpoint's own and for
tar, and one for the RON with embedded bytes, written at #rooted-assets. The causes T3 and T4 stress #asset-tree-shape, #editing-modes and
#browser-pack-save, which they guard.

### The browser build saves by downloading a pack `##browser-pack-save`

Proposed by the session in round 1. Approved in round 2: "browser-pack-save: accepted. this was part
of my goal, closing design@rootpoint-app@browser-load-only, which was only a temporary measure while
i was putting more thought on the asset management mechanism", and Q3: "yes, it should save".
Arguments: #arg-14. Shape: #browser-packs. Harvest: the reversal of
`design@rootpoint-app@browser-load-only` at #browser-packs.

### The sample has a slide that explains the two modes and the pack `##sample-explains-modes`

Proposed by the owner in round 2: "also, maybe an explanation slide in the sample (which serves as a
tutorial) could be useful." Argued by the session in round 2's reply, and accepted in round 3:
"sample-explains-modes and unsaved-close-warning accepted." Arguments: #arg-20. Shape:
#browser-packs. Harvest: none; the sample's content is no decision, and
`design@rootpoint-protocol@builtin-sample` keeps its bound of 50 KB of images.

### Closing a window held in memory with unsaved edits asks first `##unsaved-close-warning`

Proposed by the session in round 2's reply, extending the `beforeunload` warning of round 1, which
the owner accepted with #browser-pack-save, to the native window held in memory. Accepted in round
3: "sample-explains-modes and unsaved-close-warning accepted." Arguments: #arg-21. Shape: "Decided
design", #workspace-host and #browser-packs. Harvest: a head of the app, or the rewrite of
`design@rootpoint-app@no-discard-confirmation` that D3 proposes, at #workspace-host, which builds
the native question; its browser half joins it at #browser-packs.

### The browser build could save on its own, to a store of the page's origin `##web-autosave`

Proposed by the owner in round 3: "though, the web build could actually autosave, using db storage
(not sure of exact name, but not local storage which has low size limits). maybe a todo issue for
later". Parked on the owner's word, as a todo issue: `issue@rootpoint-app@browser-autosave`,
opened on this milestone's branch. Its re-entry is that issue, picked up from the issues; no work
of the milestone builds it. Arguments: #arg-22.

## Arguments

### Bytes embedded in the RON grow it 20 times `##arg-1`

Round 1, the session; on #asset-paths-only and #pack-container. "One `Embedded` asset of 580 bytes
added 11,768 bytes to the RON, about 20.3 bytes per byte. RON writes `Vec<u8>` as a list of decimal
numbers, one per line. At that rate, a 5 MB photo becomes about 100 MB of text." Measured by a
scratch program over the model crate that sets the fixture's first asset to the sample's
`splits.png` as `Embedded` and compares the lengths of `save_ron`; the program is not committed.

### The mode is a property of the store, and both stores exist in the code today `##arg-2`

Round 1, the session; on #editing-modes. "Your two modes already exist in the code, unnamed. The
browser uses `MemoryFiles`, files held in memory. The native build uses `NativeFiles`, files on
disk. I propose making that split explicit: one session, two stores." "The watcher, the lock and
the socket live in the app's `DiskStore`, so C4 holds."

### A root that never changes removes the directory arithmetic and its defect `##arg-3`

Round 1, the session; on #editing-modes. "Every asset path is relative to the root, and the root
never changes during a session. So a save or an unpack never rewrites a path. ... This removes all
the directory arithmetic in `Session::save`, which closes
`save-mixes-relative-and-absolute` by deleting its cause."

### The rival keeps the free save path whose changing base made every bug of the save rule `##arg-4`

Round 1, the session; on #editing-modes. "Nearest rival. Keep a free choice of save path and
directory, as today. It loses because every bug in the save rule, and the open defect, comes from
paths whose base changes." The costs of the shape: "`Files` gains `list`, `rename` and `remove`."
and "A loose `deck.ron` outside a workspace can no longer be opened by itself. Opening it would
mean opening its directory."

### A fixed name lets a script find the presentation, and makes a directory one talk `##arg-5`

Round 1, the session; on #workspace-layout. "A script or an agent finds the presentation without
listing the directory. The browser build today loads 'the one `.ron` file' in a directory, and a
stray `old.ron` makes that ambiguous." Cost: "One presentation per directory. Two talks that share
images each keep a copy." Accepted by the owner in round 2: "cost accepted."

### The kernel releases the lock at the process's end, and only the holder binds the socket `##arg-6`

Round 1, the session; on #workspace-lock. Measured: `std::fs::File::try_lock` is in the installed
1.98.0 toolchain, at `library/std/src/fs.rs:981`, stable since 1.89.0; the tree pins no toolchain,
and the default one, 1.99.1, has it at line 976. "The kernel releases the lock when the process dies, so
a crash leaves no stale lock. ... only the lock holder binds the socket. That makes the bind race of
`two-windows-bind-one-path` impossible." Unverified: "`flock` is not reliable
on some network file systems", proposed as a tripwire, T1. "This lock is between processes.
`design@rootpoint-app@one-controller-at-a-time` stays the lock between a window and the clients of
its socket. An agent drives the window through the socket and never opens the workspace itself."

### A socket's path is limited to 108 bytes on Linux `##arg-7`

Round 1, the session; on #socket-place. "The path of a Unix socket is limited to 108 bytes on Linux
and 104 on macOS (`sun_path` in `unix(7)`). `/.rootpoint/control.sock` takes 24 of them, so a
workspace path longer than about 83 bytes cannot hold its socket." The 108 is read in `man 7 unix`
on the owner's AMD Ryzen 9 9950X, `char sun_path[108]`; the 104 of macOS is not read. "Since `rootpoint-ctl`
resolves the socket from the workspace, the rival can replace the default later without changing
what any client types." The owner, in round 2: "i did not know the lenght limit."

### `Embedded` has three writers, and the design removes each `##arg-8`

Round 1, the session; on #asset-paths-only. "`AssetSource::Embedded` has three writers today: the
packing that the head `asset-source` planned; the sample; the stack rewrite of
`save-updates-sources`. Measurement 1 defeats writer 1. #editing-modes removes writers 2 and 3".
Two writers are in the code, `path@rootpoint-protocol@src/sample.rs` and `Session::save`; the first
is planned by the head, not built. Tests also write `Embedded`.

### An id makes a rename one operation; a mirror of the disk gives ids to every file `##arg-9`

Round 1, the session; on #asset-tree-shape. "A rename or a move is one `Op::SetAssetSource` on one
asset, plus a move of its file, and no slide changes. The rival, content that names an asset by its
path, would rewrite every image and style at each rename." "The rival, folder nodes stored in the
RON, stores structure that holds no content." "The rival, a table that mirrors every
file on disk, would give an id to `.git/`, to `README.md` and to an editor's backup files. Each
change on disk would then become an edit nobody made."

### Under shape B, the disk disagrees with the window between two saves `##arg-10`

Round 1, the session; on #asset-disk-changes. "Under B, between two saves, the files on disk
disagree with what the manager shows. The watcher, git and an agent reading the directory all see
the old tree. Under A, the disk and the RON disagree for one gesture at most."

### A changed asset is decoded again, and a changed RON would be one undo step `##arg-11`

Round 1, the session; on #asset-watch. "A changed asset file is decoded again. This closes
`changed-asset-not-read-again`. The cache key gains the file's modification
time. The watcher wakes the window through `request_repaint`". For the RON: "the new file is loaded
as a new `Op::ReplaceDocument`. Its inverse is the old document, so undo brings back what was
there." Its rival: "a reload that empties the undo stacks. It loses the history without warning,
against `no-discard-confirmation`." The reload on a change was withdrawn after #arg-15;
`Op::ReplaceDocument` stays, as the "Load the file" action of #ron-overwrite-guard.

### A shared thing is changed only from the view that lists its uses `##arg-12`

Round 1, the session; on #asset-manager-scope. "The screen is a tab of the side panel, per
`styles-tab-edits-styles`: shared things change only from the view that lists their uses." "No
conflict with `goal@rootpoint@minimal-content-set`. That goal counts only features of slide
content, and this is not one."

### A zip unpacks with any tool, and indexes its entries `##arg-13`

Round 1, the session; on #pack-container. "This is the convention of ODF and EPUB: a zip whose first
entry names its type. Unpacking, in rootpoint or with any unzip tool, gives exactly the workspace
layout ... Zip has a central index, so the browser can read one asset without reading the whole
file." Against a format of rootpoint's own: "It does what zip does, and it can be opened only by
rootpoint." Against tar: "it has no index, and Windows file managers did not open it until
recently." Unverified: "A pure-Rust zip reader and writer builds for the wasm target." "The pack is
not a version of its own: the RON inside it names the format's version, per `save-format`."

### The browser's goal asks a pack to open there, and saving it follows `##arg-14`

Round 1, the session; on #browser-pack-save. "This answers C2 beyond 'opens a pack'. The browser
opens a `.rpk` file or a directory, and saves by downloading a `.rpk`." The owner, in round 2: "this
was part of my goal".

### The owner edits by the window and the protocol, so the RON needs no reload `##arg-15`

Round 2, the owner; on #asset-watch. "i see no reason to watch presentation.ron. the correct way to
edit is either text protocol or gui IMO."

### Autosave costs the GUI thread 0.11 ms at 1000 slides `##arg-16`

Round 2's reply, the session; on #asset-disk-changes, as the owner asked. Medians of 21 runs on the
AMD Ryzen 9 9950X, ext4 on LVM, of a clone of the document, `save_ron` on the clone, and a write to
a temporary file with `fsync` and a rename, the sample's slides duplicated to each size:

| slides | RON bytes | clone | `save_ron` | write, fsync, rename |
| --- | --- | --- | --- | --- |
| 3 | 19,864 | 0.000 ms | 0.025 ms | 0.292 ms |
| 50 | 102,599 | 0.008 ms | 0.178 ms | 0.344 ms |
| 200 | 367,102 | 0.025 ms | 0.673 ms | 0.442 ms |
| 1000 | 1,781,502 | 0.111 ms | 3.319 ms | 0.777 ms |

"The cost grows linearly, at about 3.3 µs of `save_ron` per slide of three areas." "One frame at
60 Hz is 16.7 ms. Even all on the GUI thread, a 1000-slide talk costs 4.2 ms per undo step." "With
it [the background thread], the GUI thread pays only the clone, 0.11 ms at 1000 slides. The thread
also absorbs a slow `fsync`, as on a USB stick, which this disk does not show." The shape: "If a
write is already running, only the newest clone waits; older ones are dropped. The browser build
has no thread and no workspace, so it has no autosave." The sizes include the sample's embedded
images. The program is not committed; AC1 makes a benchmark of it.

### Autosave makes the window a writer that can overwrite a checkout `##arg-17`

Round 2's reply, the session; on #ron-overwrite-guard. "autosave makes the editor a writer that can
destroy someone else's change without anyone seeing. The workspace lock does not help: git, text
editors and an agent's file tools do not know about it." The failure: "You run
`git checkout other-branch`, or `git stash`, or an agent edits `presentation.ron` with its file
tool ... Your next gesture triggers an autosave, which overwrites the file with the window's
document." The shape: "Before each autosave write, the thread compares the file's current bytes with
what it last wrote. If they differ, it writes nothing, and autosave stops." "This is not the dialog
of `rejected@rootpoint-app@removal-guard`. That one guarded actions that undo can reverse. This
guards content the window never held." In round 3's reply, of the gap between the comparison and
the rename: "I accept that window, since closing it would need a lock that git does not take."

### A bare name collides, and a flag of state goes stale at a crash `##arg-18`

Round 2's reply, the session; on #socket-place. "With the bare name, `~/2025/talk` and `~/2026/talk`
would collide, and the lock you proposed would refuse the second, though they are different
workspaces." "A flag goes stale when the window crashes, for the same reason as a pid file under
#workspace-lock. A client learns whether the window is live by connecting." "The workspace lock
already makes one window per workspace, so the socket needs no lock of its own." "An in-memory
window gets a socket too, `memory-<pid>.sock`. Per your answer to Q2, a new presentation starts in
memory, and `goal@rootpoint-app@live-agent-edits` asks that an agent can build one from the start."
"`--control <path>` goes, per `design@rootpoint@no-compat-in-initial-dev`."

### Of five shapes, the abstract namespace and TCP lose the file permissions a socket file has `##arg-19`

Round 2's reply, the session; on #socket-place, in answer to the owner's "any other shape would
allow what we do ?". Of the chosen shape: "macOS has no `XDG_RUNTIME_DIR`: fall back to
`$TMPDIR`, which is per user there". An abstract-namespace socket "has no file permissions: any user on the machine
can connect and drive the editor"; a bind through `/proc/self/fd/<dir>/control.sock` is "Linux
only, and obscure"; TCP on localhost means "any local process can connect, so the protocol needs a
token handshake; macOS firewall prompts"; named pipes have "no connections: one pipe pair per
client, set up by hand". "TCP stays the candidate if Windows ever matters, and `rootpoint-ctl`
resolving the socket keeps that change invisible to clients."

### The sample is the tutorial, and the modes are what a user meets at the first save `##arg-20`

Round 2's reply, the session; on #sample-explains-modes. "The sample is the tutorial, per your word
in `design@rootpoint-protocol@builtin-sample`, and the modes are the first new idea a user meets,
at the first save. It costs one slide and no image, so it stays inside the head's 50 KB."

### Undo cannot reach a closed window, so the warning guards no content undo guards `##arg-21`

Round 2's reply, the session; on #unsaved-close-warning. "The same loss exists in a native
in-memory window, so it gets the same question. A workspace autosaves, so it never asks. Undo
cannot recover a closed window, the case `rejected@rootpoint-app@removal-guard` names, so this is no
conflict with that entry."

### The browser's origin has stores larger than `localStorage` `##arg-22`

Round 3's reply, the session; on #web-autosave. "`localStorage` holds only a few MB per origin.
IndexedDB, and the Origin Private File System (OPFS) built beside it, get a share of the disk
instead. OPFS is the better lead: it is a file tree per origin, so it could implement `Files` the
way `DiskStore` does, and give the browser a workspace mode rather than only autosave." Labelled in
the reply as "assumptions from the browsers' documentation as I recall it, not read today".

### Packs come from trusted sources, and confinement is the one rule kept now `##arg-23`

Round 4, the owner; on #asset-paths-only and #pack-container. "powerpoint or anything else has the
same problems anyway, packs are meant to be shared with and from trusted sources, security features
can be designed and added later." "the assets have to stay confined inside the workspace." The
session's reading, in its reply: "An asset path is always a path inside the workspace root. It is
never absolute, has no `..`, and has no symlink that leads out of the root. A store refuses any
other path, at load and at every command." and "This makes AC2 the test of a rule you set, not a
security feature. It stays, and further protection of shared packs waits, as you said." The reading
is D2. Its "at load" is replaced by D4: a file loads, and no access reaches outside the root.

## New names, in one place

Each is introduced by the slice named beside it. A name that exists is marked so.

```text
crates/rootpoint-model/src/document.rs     Asset { path: String }                     (#rooted-assets; AssetSource leaves)
crates/rootpoint-model/src/document.rs     fn asset_path_is_confined(&str) -> bool     (#rooted-assets)
crates/rootpoint-model/src/validate.rs     Breach::UnconfinedAsset { asset: AssetId } (Breach exists, with MissingAsset)   (#rooted-assets)
crates/rootpoint-protocol/tests/common/mod.rs   RecordingFiles, native files that record every path given them   (#rooted-assets)
crates/rootpoint-model/src/ops.rs          Op::SetAssetPath { id, path }               (#rooted-assets; Op::SetAssetSource leaves)
crates/rootpoint-model/src/ops.rs          Op::ReplaceDocument { document }            (#autosave-and-watch)
crates/rootpoint-protocol/src/files.rs     Files::canonical                            (#rooted-assets)
crates/rootpoint-protocol/src/files.rs     Files::list                                 (#pack-commands)
crates/rootpoint-protocol/src/files.rs     Files::rename, Files::remove                (#asset-commands)
crates/rootpoint-protocol/src/session.rs   Session::pack: Option<String>, the pack last opened or written in memory   (#pack-commands)
crates/rootpoint-app/src/actions.rs        App::open_picked_pack(name, bytes)          (#pack-commands)
crates/rootpoint-protocol/src/session.rs   enum Store { Memory(BTreeMap<String, Vec<u8>>), Workspace(String) }   (#rooted-assets)
crates/rootpoint-protocol/src/session.rs   Session::store, Session::root               (#rooted-assets; Session::directory leaves)
crates/rootpoint-protocol/src/session.rs   Session::unsaved: u64, the steps applied, undone or redone since the last open or save, or a pack in memory   (#rooted-assets)
crates/rootpoint-protocol/src/session.rs   Session::trash: BTreeMap<AssetId, Vec<u8>>, the trash in memory   (#asset-commands)
crates/rootpoint-protocol/src/pack.rs      fn write_pack(ron: &str, assets: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String>   (#pack-commands)
crates/rootpoint-protocol/src/pack.rs      fn read_pack(&[u8]) -> Result<(String, BTreeMap<String, Vec<u8>>), String>   (#pack-commands)
crates/rootpoint-protocol/src/pack.rs      const PACK_MIMETYPE: &str = "application/x-rootpoint"   (#pack-commands)
crates/rootpoint-protocol/src/workspace.rs WorkspaceLock, fn socket_path(root: Option<&str>) -> PathBuf, fn runtime_dir() -> PathBuf,
                                           the pointer file; native only, cfg(not(target_arch = "wasm32"))   (#workspace-host)
crates/rootpoint-app/src/autosave.rs       the autosave thread and the guard           (#autosave-and-watch)
crates/rootpoint-app/src/watch.rs          the asset watcher                           (#autosave-and-watch)
crates/rootpoint-app/src/asset_tab.rs      the asset tab                               (#asset-manager)
crates/rootpoint-app/src/lib.rs            SideTab::Assets (SideTab exists, with Selection and Styles)   (#asset-manager)
crates/rootpoint-app/src/lib.rs            App::state_label(&self) -> String, the state label's text, read by tests   (#workspace-host)
crates/rootpoint-app/src/lib.rs            App::native(files), the window that takes the locks and moves the socket   (#workspace-host)
crates/rootpoint-protocol/src/workspace.rs fn socket_name(root: Option<&str>) -> String, fn run(…, lock: &mut Option<WorkspaceLock>) -> Reply   (#workspace-host)
crates/rootpoint-app/src/control.rs        Listener::rename_to(path)                   (#workspace-host)
crates/rootpoint-app/tests/autosave.rs     autosave_gui_cost, an #[ignore] test, the AC1 benchmark:
                                           cargo test -p rootpoint-app --release --test autosave -- --ignored   (#autosave-and-watch)

commands (#rooted-assets): open <dir>, open-copy <dir>, import <host-path> [<asset-path>], save, store; new and sample exist
commands (#pack-commands): open-pack <file>, pack <file>, unpack <dir>; save in memory writes the pack last opened or written
commands (#autosave-and-watch): reload
commands (#asset-commands): asset-move <a> <asset-path>, asset-relink <a> <asset-path>, asset-replace <a> <host-path>,
                            asset-remove <a> [into=<a>], asset-remove-unused, assets [all|unused|missing|untracked]
removed: load <path>, save <path>, asset <path>, remove-asset (replaced by asset-remove)
binary options: rootpoint [<dir> | <file>.rpk]; rootpoint-ctl <workspace-dir | socket | list> [<words>…]
removed: rootpoint --control <socket>
files: <root>/presentation.ron; <root>/.rootpoint/{.gitignore, lock, socket, trash/}
       <runtime directory>/<name>-<8 hex>.sock, memory-<pid>.sock
```

The names of the commands, and the numbers of #socket-place (40 bytes, 8 hexadecimal digits), are
this document's defaults. A slice's audit that finds a better name renames it in place, as a gap
with one answer.

## Decided design

### #editing-modes

The session holds a store, `Store::Memory` or `Store::Workspace(root)`. In memory, the presentation's
files are the session's own map, keyed by asset path; in a workspace, they are the host's files
under the root. Every command reads an asset through the store: the map, or the host's files at the
root joined with the asset path. The map is in the session, not in a host, so `new`, `sample`,
`open-copy` and `open-pack` behave the same headless and in the window, as `goal@rootpoint@headless-core`
asks; the rival, a map of the host, would need every host to build the same map and a way for a
command to switch the host's store, which the session cannot ask of `Files`. The session still opens
no file itself: the host's files are passed to each command, as
`design@rootpoint-protocol@session-files` says, and the map is not a file system. `DiskStore`, the
word of the discussion, is `Store::Workspace` together with what its host adds: the lock, the
socket, the autosave and the watcher.

A host path, the argument of `open`, `open-copy`, `open-pack`, `pack`, `unpack`, `import`,
`asset-replace` and `render`, is read by the host's files as the host reads it: relative to the
host's working directory, natively, also for a line of the control socket. `open <dir>` and
`unpack <dir>` store the root as `Files::canonical` gives it, absolute with no symlink, so no
later comparison sees two forms of one directory, the cause of the save-mixes defect.

| command | from | to |
| --- | --- | --- |
| `new`, `sample` | any | in memory |
| `open <dir>` | any | a workspace at `<dir>`; with no `presentation.ron` there, an empty presentation, written at the first save; `err` when `<dir>` does not exist |
| `open-copy <dir>` | any | in memory, `<dir>`'s presentation and its assets copied into the map (the browser's picked directory) |
| `open-pack <file>` | any | in memory, the pack's entries in the map |
| `save` | a workspace | writes `presentation.ron` at the root |
| `save` | in memory, after `open-pack` or `pack` | writes that pack again; `err` naming `pack` and `unpack` otherwise |
| `pack <file>` | any | writes a pack; the store is unchanged |
| `unpack <dir>` | in memory | writes every entry under `<dir>`, which is empty or absent, then the store is a workspace at `<dir>`; `err` when `<dir>` holds a file |
| `unpack <dir>` | a workspace | `err`, naming `pack` |
| `store` | any | changes nothing; replies `ok memory unsaved=<n>`, with ` pack=<file>` once a pack is its target, or `ok workspace <root> unsaved=<n>` |

`store` is the query of what the state label shows, so the label is a view an agent reads too, as
`goal@rootpoint-protocol@commands-drive-everything` asks whatever reading of its "mode" applies.

Asset paths never change when the store changes, since they are relative to the root in both, so no
command rewrites a path in the document or in the stacks. `Session::directory` and the stack rewrite
of `save-updates-sources`, now `rejected@rootpoint-protocol@free-save-path`, leave.

### #workspace-layout

```
<root>/
  presentation.ron
  <asset paths>
  .rootpoint/
    .gitignore      "*"
    lock            held by WorkspaceLock; holds the pid of its holder, for the refusal's message
    socket          the path of the window's control socket, and nothing else
    trash/<id>      the bytes of a removed asset, until the workspace is opened again, which empties it
```

`.rootpoint/` and `presentation.ron` are never asset paths: `asset_path_is_confined` refuses a path
whose first component is `.rootpoint`, or which is `presentation.ron`, in both stores.

### #workspace-lock

`WorkspaceLock` is in the protocol crate, native only, beside the client of the control socket: the
headless binary, `rootpoint-ctl` and the window all use it, and the session does not.
`design@rootpoint@gui-only-in-app` rules out the protocol depending on the app, so the lock cannot
be the app's. A host runs every command through one function of its own that, before `open <dir>`
or `unpack <dir>`, takes `File::try_lock` on `<dir>/.rootpoint/lock`, creating `.rootpoint/` with
its `.gitignore`, and replies `err` naming the pid the file holds when another process holds it,
without running the command; and, after any command whose reply is `ok` and that changed
`Session::store`, drops the lock of the old store. The window's commands from the control socket
go through the same function, `path@rootpoint-app@src/control.rs` included.

### #socket-place

The runtime directory is `$XDG_RUNTIME_DIR/rootpoint/`, or `$TMPDIR/rootpoint/`, then
`/tmp/rootpoint-<uid>/`, where the variable is unset; it is created with mode 0700. The window's
socket there is `<name>-<hash>.sock` in a workspace, where `<name>` is the root's basename truncated
to 40 bytes and `<hash>` the first 8 hexadecimal digits of the FNV-1a hash, 64 bits, of the
canonical root; and `memory-<pid>.sock` in memory. A workspace's socket is bound only while its lock
is held; an in-memory socket needs no lock, since its pid makes its name unique. The bind removes a
socket file at its path, and refuses and keeps any other file, as
`design@rootpoint-app@only-a-stale-socket-is-replaced` argues. `.rootpoint/socket` holds the
socket's path.

When a command changes the store, the window renames its socket file to the new store's name, AC5,
updates the path `Listener` removes on drop, writes `.rootpoint/socket` of the new workspace, and
removes that of the old one. `rootpoint-ctl <dir>` reads `<dir>/.rootpoint/socket`;
`rootpoint-ctl <socket>` connects to that path; `rootpoint-ctl list` prints one line per socket of
the runtime directory that accepts a connection, with its file name.

### #asset-paths-only

`Asset { path }`. `asset_path_is_confined` holds for an asset path with no empty, `.` or `..`
component, no leading `/`, no `\`, a first component other than `.rootpoint`, and other than
`presentation.ron` and, from #pack-commands, `mimetype`, a pack's first entry. `load_ron` loads a file whose asset fails it: the path is a cosmetic breach,
`Breach::UnconfinedAsset`, named by `Document::breaches`, its image drawn grey and its font the
built-in sans, as a missing file's are, by D4. `Document::breaches` gives one per asset of the
table whose path fails, used or not, after the breaches of the areas, in the order of the ids.

The store is the one check of every access to an asset: it refuses each read, write, rename and
removal of an asset path that fails `asset_path_is_confined`, and, in a workspace, each whose
`Files::canonical` path is not under the canonical root, which a symlink that leads out of the root
makes so. For a path that does not exist yet, as an `import`'s target, the check takes the
canonical path of its nearest existing parent. `open-copy <dir>` reads through the same check,
against `<dir>`: an asset it refuses is not copied, and is missing in the map. The store's own
files, `presentation.ron` and `.rootpoint/`, are no asset and are not refused. `Files::canonical`
gives the path with every symlink resolved, `std::fs::canonicalize` natively, and the path itself
in a map.

So no command reads or writes outside the root, whatever the RON names. Every command that names a
new asset path that fails confinement, `import`, `asset-move` and `asset-relink`, replies `err`, per
`design@rootpoint-protocol@no-new-breach`; `undo` and `redo` bring back what a file was loaded with,
as `apply` accepts, `design@rootpoint-model@cosmetic-fallbacks`. On an asset whose path fails:
`render` draws it as missing; `pack` writes no bytes for it, as for a missing one, and its reply
names each; `asset-move` and `asset-replace` reply `err`; `asset-remove`, with or without `into=`,
removes it from the table and touches no file, so nothing enters the trash, and its undo restores
the entry; `asset-relink` to a confined path repairs it. So such an asset can always leave the
table, from the window.

`import <host-path> [<asset-path>]` reads a file of the host's files, anywhere. In a workspace, a
host path whose canonical form lies under the root is added at its path relative to the root,
with no copy, and an `<asset-path>` given then must be that path. Any other file is written into
the store at `<asset-path>`, or at the root under its file name, with `-2`, `-3` before the
extension when the name is taken by other bytes and the same name when the bytes are equal, as
`asset-save-rule`, now `design@rootpoint-protocol@import-copies-into-the-store`, named; then the asset is added.

### #asset-tree-shape

Content names an asset by its id; `Op::SetAssetPath` moves one asset. A folder is the set of asset
paths that share a prefix; no folder is stored. The table holds the assets added by `import`,
`open-copy`, `open-pack` or `unpack`; a file under the root that no asset names is untracked,
listed by `assets untracked`, and is never packed.

### #asset-disk-changes

In a workspace, `asset-move` renames the file through `Files::rename`, `asset-remove` moves it to
`.rootpoint/trash/<id>`, `asset-replace` writes new bytes after moving the old ones to the trash,
and undo moves them back; `asset-relink` sets an asset's path to a file already there, and moves no
file; in memory, the same moves happen between the map and `Session::trash`. The undo step of such
a command holds its file moves with its operations, and a step whose file move fails applies none
of them; `design@rootpoint-protocol@gesture-undo` is rewritten for it at #asset-commands. `open`
of a workspace empties its trash, through `Files::list` and `Files::remove`.

The window writes `presentation.ron` after every undo step, on the autosave thread: the GUI thread
clones the document and hands the clone over; a write in progress is followed only by the newest
clone. The thread writes with the native file system, as app code may, to a temporary file in the
root, then renames it. It reads no session and applies no change, so it is no action of
`design@rootpoint-app@actions-are-commands`. `rootpoint-headless` has no autosave: a script writes
`presentation.ron` by `save`, and between an asset command and its `save`, the RON names the old
paths.

### #asset-watch

The window watches, natively and in a workspace only, the directories that hold an asset of the
table, not recursively, AC3; a change of an asset's file decodes it again in the next frame,
`Assets::refresh` keying on its modification time. In memory, the map is the presentation, so
nothing on disk is its file and nothing is watched, the browser's picked directory included. The
leads of the closed issue `changed-asset-not-read-again` other than a notification, a check of the
modification time at each draw and an action "reload assets", are not built: the owner asked in
round 1 for the assets to be watched.

### #ron-overwrite-guard

When autosave starts, at the open of a workspace, the thread reads `presentation.ron` as the bytes
last read. Before each write, it reads the file and compares it with the bytes it last wrote or
read. When they differ, it writes nothing, autosave stops, and the window shows a banner with two
actions: "Load the file", which runs `reload`: `presentation.ron` is read and applied as
`Op::ReplaceDocument`, one undo step, and a file that does not load is named in the banner, which
stays; and "Keep mine", which writes the window's document. After either action, autosave resumes,
the bytes now read or written as its baseline. A write between the comparison and the rename, a
window of about one write's time, is not detected; the session's reply of round 3 accepted that
window, since closing it needs a lock git and editors do not take.

### #asset-manager-scope

The features, each change a command: the tree with a search by name; a filter, all, unused,
missing, untracked; each asset's size, a thumbnail and its uses, each use a jump to its node; move
and rename, by drag or by typing a path, `asset-move`; replace the bytes, keeping the id,
`asset-replace`; relink a missing asset, offering first an untracked file of the same bytes,
`asset-relink`; remove an unused asset, `asset-remove`, and all unused at once, one undo step,
`asset-remove-unused`; remove a used asset with a replacement, `asset-remove a3 into=a5`; import by
drag and drop or by a picker, `import`; the total size. The asset tab is a tab of the side panel and
keeps its width, `design@rootpoint-app@sidebar-width`.

### #pack-container

A zip: first the entry `mimetype`, stored with no compression, holding `PACK_MIMETYPE`; then
`presentation.ron`, compressed by deflate; then each asset of the table at its asset path, stored
with no compression. The pack carries no version of its own: the RON in it names its format's
version, per `design@rootpoint-model@save-format`. `read_pack`, which `open-pack` runs, refuses a
pack whose first entry is not that `mimetype`, and an entry whose name fails confinement or is a
symlink, AC2; `unpack <dir>` writes the map, which only confined paths reach.

### #browser-pack-save

The browser build opens a `.rpk` file from its file picker by `open-pack` on the file's bytes, and
a picked directory by `open-copy`; "Save as pack…" runs `pack` into `MemoryFiles` and hands the
bytes to the page as a download, and Ctrl+S does the same once a pack is the session's target.

### #sample-explains-modes

One text slide of the sample, with no image: in memory or in a workspace, what each save writes, the
`.rpk` file, and `.rootpoint/`.

### #unsaved-close-warning

In memory, a close of the window, or of the page, while `Session::unsaved` is not 0 asks first; a
workspace never asks. The native window cancels the close by
`ViewportCommand::CancelClose` and shows the question; the page installs a `beforeunload`
listener. D3 puts to the owner the title of `design@rootpoint-app@no-discard-confirmation`, which
this question contradicts.

## Mapping tables

**Old commands to new ones**, total over the commands that change:

| old | new |
| --- | --- |
| `load <path>` of a `.ron` file | `open <dir>` of its directory, the file renamed `presentation.ron` |
| `save <path>` | `save` in a workspace; `pack <file>` or `unpack <dir>` in memory |
| `asset <path>` | `import <host-path> [<asset-path>]` |
| `remove-asset a<n>` | `asset-remove a<n>` |
| `new`, `sample` | unchanged; the store becomes the map |
| `rootpoint --control <socket> [<file>.ron]` | `rootpoint [<dir> \| <file>.rpk]`; the socket starts by itself |
| `rootpoint-ctl <socket> …` | unchanged, and also `rootpoint-ctl <dir> …`, `rootpoint-ctl list` |

**`AssetSource` to `Asset`**, total over the format's values:

| old | new |
| --- | --- |
| `Path(p)`, `p` confined | `path: p` |
| `Path(p)`, `p` not confined | `path: p`, a cosmetic breach no command reads or writes (D4) |
| `Embedded(bytes)` | no form in the RON: the bytes are an entry of the map or a file at an asset path |

**The store's state to the state label**, total over the states, the form of #c-visible-mode
widened by D5:

| store | `Session::unsaved` | label |
| --- | --- | --- |
| in memory, no pack its target | any | "In memory, not saved" |
| in memory, a pack its target | 0 | "In memory, saved to `<file>.rpk`" |
| in memory, a pack its target | n > 0 | "In memory, saved to `<file>.rpk`, n changes since" |
| a workspace, autosave running | any | "Workspace `<path>`, saved" |
| a workspace, autosave stopped by the guard | any | "Workspace `<path>`, not saved: the file changed outside rootpoint" |
| a workspace, before #autosave-and-watch | 0 | "Workspace `<path>`, saved" |
| a workspace, before #autosave-and-watch | n > 0 | "Workspace `<path>`, n changes since the save" |

**The store's change to the socket's name**, total over the changes the window makes:

| from | to | socket |
| --- | --- | --- |
| none, at start | in memory | bind `memory-<pid>.sock` |
| none, at start | a workspace | take its lock, bind `<name>-<hash>.sock` |
| in memory | a workspace (`open`, `unpack`) | take the lock, rename to `<name>-<hash>.sock` |
| a workspace | in memory (`new`, `sample`, `open-copy`, `open-pack`) | rename to `memory-<pid>.sock`, drop the lock |
| a workspace | another workspace | take the new lock, rename, drop the old lock |
| in memory | in memory | unchanged |

## Losing alternatives

- **Packing as `Embedded` bytes in the RON**, the shape of `asset-source`, now `rejected@rootpoint-model@bytes-in-the-file`, and
  of the closed `pack-and-unpack`: lost to #pack-container and #asset-paths-only on
  #arg-1, 20.3 bytes of text per byte of asset, against #c-pack-overhead.
- **A free save path and directory, as today**: lost to #editing-modes on #arg-4.
- **A pid file as the lock**: lost to #workspace-lock on #arg-6, a crash leaves it held.
- **The socket at `.rootpoint/control.sock`, refused when too long**: the session's first shape of
  #socket-place, superseded by the owner's runtime directory on #arg-7 and #arg-18.
- **The socket named by the workspace's name alone, with a lock of its own and a flag of state**:
  the owner's first form of #socket-place, refined on #arg-18.
- **An abstract-namespace socket**, **a bind through `/proc/self/fd`**, **TCP on localhost**,
  **named pipes**: lost to #socket-place on #arg-19.
- **Content that names an asset by its path**, **folders stored in the RON**, **a table that mirrors
  every file on disk**: lost to #asset-tree-shape on #arg-9.
- **Shape B, asset changes held until an explicit save**: lost to #asset-disk-changes on #arg-10 and
  the owner's word of round 2.
- **A reload of `presentation.ron` whenever it changes**, withdrawn by the session after #arg-15; and
  **a reload that empties the undo stacks**, lost on `design@rootpoint-app@no-discard-confirmation`.
  **No check before an autosave** lost to #ron-overwrite-guard on #arg-17.
- **A binary format of rootpoint's own**, and **tar**: lost to #pack-container on #arg-13.
- **A check of an asset's modification time at each draw**, and **an action "reload assets"**, the
  other leads of the closed `changed-asset-not-read-again`: not built, since the owner asked in
  round 1 for the assets to be watched, #asset-watch.
- **A map of the host as the in-memory store**: lost to #editing-modes, "Decided design", since a
  command could not switch a host's store and every host would build the same map.
- **The workspace lock in the app**: lost to #workspace-lock, "Decided design", on
  `design@rootpoint@gui-only-in-app`, since the headless binary is the protocol crate's.

## Readings

- `unix(7)` on the owner's AMD Ryzen 9 9950X, `char sun_path[108]`, the limit of #arg-7. The 104
  bytes of macOS, read at #workspace-host's audit in xnu's `bsd/sys/un.h`: `char sun_path[104]`.
  macOS's per-user temporary directory, `_CS_DARWIN_USER_TEMP_DIR` of `confstr(3)`, is "created
  with access permissions of 0700"; that `$TMPDIR` names it is the reply of round 2 as recalled, not
  read.
- The zip format, through the crate #pack-commands chooses, and the convention of a first entry `mimetype`
  stored with no compression, of EPUB's container. Read at #pack-commands's audit in EPUB 3.3,
  section 4.3.3, "OCF ZIP container media type identification": the `mimetype` file is the first
  file of the container, holds the media type in US-ASCII with no leading or trailing padding or
  whitespace and no byte order mark, is neither compressed nor encrypted, and has no extra field in
  its ZIP header; and section 4.3.2 allows only stored and Deflate entries. The pack follows that
  shape with its own media type.
- The browser's storage, IndexedDB and the Origin Private File System, is not read: it is the lead
  of `issue@rootpoint-app@browser-autosave`, outside the work.

## Premortem

Assume the work shipped and failed. The causes, each with the thread it stresses and its verdict;
the owner's word of round 4, "agreed on tripwires and AC, keep all.", rules on every label.

1. A workspace in a folder that a sync tool shares between two machines is open on both; the lock
   holds on one machine only, and both autosave. Stresses #workspace-lock. A tripwire, **T1**: it
   fires when a workspace opened on two machines through a sync tool leaves a conflict copy of
   `presentation.ron`, or loses an edit from one side; the response is the host's name in the lock
   file, and a refusal when it names another host; re-entry at the firing.
2. Something rewrites the editor's own bytes with no real change, as git's `autocrlf` or a sync
   tool's normalising does, and the guard's banner shows at every gesture. Stresses
   #ron-overwrite-guard. A tripwire, **T2**: it fires when the banner shows while no program other
   than rootpoint wrote the file; the response is a comparison of the parsed document instead of the
   bytes; re-entry at the firing. The window of about one write between comparison and rename
   survives into the decided design of #ron-overwrite-guard as an accepted cost.
3. An author keeps a file the presentation does not name, a source `.svg` beside its `.png`, packs,
   unpacks elsewhere and finds it missing. Stresses #asset-tree-shape. A tripwire, **T3**: it fires
   when an author loses such a file through a pack and an unpack and expected it kept; the response
   is an option to pack the untracked files; re-entry at the firing.
4. The map holds every byte, and wasm32 addresses at most 4 GB: a talk with video fails to open or
   save in the browser. Stresses #editing-modes and #browser-pack-save. A tripwire, **T4**: it
   fires when a presentation fails to open or save in the browser build by its size; the response is
   reading the assets from the zip's index when they are drawn, or the store of
   `issue@rootpoint-app@browser-autosave`; re-entry at the firing, or at the design of video.
5. A pack names an entry `../../.bashrc`, an absolute path or a symlink, or a RON names an asset
   outside the root. The owner, in round 4: "packs are meant to be shared with and from trusted
   sources, security features can be designed and added later", and confinement stays. Stresses #pack-container and #editing-modes. An acceptance criterion, **AC2**,
   #ac-confined-paths; and, on the owner's word of round 4, confinement became a rule of
   #asset-paths-only.
6. A watch on the whole root sees `.git/` and every build tree, and a commit decodes the assets
   again, or a deep tree exhausts inotify's watches. Stresses #asset-watch. An acceptance criterion,
   **AC3**, #ac-watch-asset-dirs.
7. The trash keeps every removed asset, a video included, and nothing reaches it after a close.
   Stresses #asset-disk-changes. An acceptance criterion, **AC4**, #ac-trash-emptied.
8. A socket file renamed while bound stops accepting at its new path. Stresses #socket-place. An
   acceptance criterion, **AC5**, #ac-renamed-socket.

The owner's condition on #asset-disk-changes, "with autosave time measured (including
scalability)", is an acceptance criterion, **AC1**, #ac-autosave-gui-cost, ruled "ac1 accepted" in
round 3.

T1 to T4 are written at the harvest of the slice that records the decision each guards, as
`skill@knowledge-architect-decision-recording@ruled-tripwires` says: T1 at #workspace-host, T2 at
#autosave-and-watch, T3 at #asset-commands, T4 at #browser-packs.

## Acceptance criteria

### The GUI thread's share of one autosave stays under 1 ms for a document of 1000 slides `##ac-autosave-gui-cost`

AC1. Guards #asset-disk-changes. Judged at #autosave-and-watch. Observable: a benchmark, outside the
gates since timings vary, builds a document of 1000 slides as #arg-16 did, in the ignored test
`autosave_gui_cost` of `planned@rootpoint-app@tests/autosave.rs`, and times the work the GUI
thread does for one autosave, the clone and the hand-over, as a median of 21 runs, on the owner's
AMD Ryzen 9 9950X; it fires above 1 ms, a bound the owner may reset. Response: the clone is replaced by a
cheaper snapshot, or the serialization moves to the GUI thread's idle time; #asset-disk-changes is
argued again only if no snapshot meets the bound.

### No access to an asset reaches outside the root, whatever a file or a pack names `##ac-confined-paths`

AC2. Guards #asset-paths-only and #pack-container. Judged at #rooted-assets (the RON, `render`,
`open-copy`, `import`, and a read or a write through a symlink), at #pack-commands (`pack` of such
an asset, and `open-pack`, which runs `read_pack`), and at #asset-commands (`asset-move`,
`asset-replace`, `asset-remove` and `asset-relink` of such an asset). Observable, each through the
protocol tests' `RecordingFiles`, which records every path it is given: a RON whose asset path is
absolute, holds `..`, starts with `.rootpoint` or is `presentation.ron` or `mimetype` loads, each such asset named
by `Document::breaches` and drawn as missing; each command on such an asset replies as "Decided
design" of #asset-paths-only says; an `import` to such a path replies `err`; and a crafted pack
with such entries and a symlink entry is refused whole. No path outside the root is given to the
host's files, but by the host paths a command names, as `import` and `open-copy`'s source and the
file or directory of `pack`, `open-pack` and `unpack`. Response: the case is fixed in the slice
that judges it; a case the design cannot keep inside the root reopens #asset-paths-only.

### The watcher watches only the directories that hold an asset of the table `##ac-watch-asset-dirs`

AC3. Guards #asset-watch. Judged at #autosave-and-watch. Observable: a test makes the workspace a git
repository, commits to it, and sees no asset decoded again; a test changes an asset's file and sees
it decoded again. Response: the watcher's set of directories is fixed in the slice; a watcher that
cannot be bound to them reopens #asset-watch.

### The trash is emptied when a workspace opens `##ac-trash-emptied`

AC4. Guards #asset-disk-changes. Judged at #asset-commands. Observable: a test removes an asset,
ends the session, opens the workspace again and finds `.rootpoint/trash/` empty; a test removes an
asset and undoes, and finds the file at its path with its bytes. Response: the trash's life is fixed
in the slice.

### A socket file renamed while bound accepts connections at its new path `##ac-renamed-socket`

AC5. Guards #socket-place. Judged at #workspace-host. Observable: a test binds a socket, renames its
file, connects at the new path and gets a reply. Response, if it fails: the window binds a new socket
at the new name and removes the old one, a shape the decision admits.

## Implementation sequence

1. #rooted-assets, landed, its spec deleted with its harvest: `Asset { path }`, confinement, the store, `open`,
   `open-copy`, `import`, `save` at the root, the sample in memory; the browser's picked directory
   in memory; the query `store`. Fails alone on: a round trip of the fixture, the reproduction of
   the save-mixes defect, and a confinement test.
2. #pack-commands, landed, its spec deleted with its harvest: `pack`, `open-pack`, `unpack`, a save in memory to its pack; the zip crate
   in the wasm gate. Fails alone on: a pack's round trip, and its extraction by `unzip`.
3. [#workspace-host](workspace-host.md): the native window's command line, the workspace lock, the
   socket in the runtime directory, `rootpoint-ctl`, the state label and the warning on close in
   memory. Fails alone on: the lock's refusal, and the bind race test.
4. [#autosave-and-watch](autosave-and-watch.md): the autosave thread, the guard and
   `Op::ReplaceDocument`, the asset watcher. Fails alone on: AC1, AC3, and the guard's banner after
   an outside write.
5. [#asset-commands](asset-commands.md): `asset-move`, `asset-replace`, `asset-remove`, `assets`,
   the trash. Fails alone on: undo of a removal in a workspace, and AC4.
6. [#asset-manager](asset-manager.md): the asset tab. Fails alone on: each feature driving its
   command, checked against a second session.
7. [#browser-packs](browser-packs.md): the browser opens a pack by its picker and saves by a
   download, the warning on leaving the page, the sample's slide on the modes. Fails alone on: the
   browser tests of a pack's round trip.

## Order rationale

- #rooted-assets before #pack-commands: a pack is the map of the store written as a zip, so the store comes
  first.
- #pack-commands before #workspace-host: the window's command line opens a `.rpk` file, which needs
  `open-pack`.
- #workspace-host before #autosave-and-watch: the autosave and the watcher run in a workspace the
  window holds by its lock.
- #autosave-and-watch before #asset-commands: in the window, an asset command changes the disk at
  once because the RON is saved after it, so autosave comes first; headless, a script's `save`
  does it.
- #asset-commands before #asset-manager: the tab drives the commands, each tested headless before.
- #asset-manager before #browser-packs: the browser build gets the asset tab as the native window
  has it, and the sample's slide describes the whole of the modes once each exists.

## Defaults awaiting the owner

- **D1**, ruled: the owner accepted it in the message "all five defaults accepted, proced"; its one
  branch per slice was replaced by the owner's later word, as "How the work is done" says. Its text: on #editing-modes and the sequence: the work is a milestone of seven slices, in the order
  above, each on its own branch. The session's default, put to the owner at the hand-off; it blocks
  this branch's merge.
- **D2**, ruled: the owner accepted it in the message "all five defaults accepted, proced". Its text: on #asset-paths-only: the session's reading of the owner's word of round 4, that "should
  not accept relative path" means a path that is not under the root, `..` or absolute, while asset
  paths stay relative to the root, as #asset-paths-only approved. Put to the owner at the hand-off.
  #rooted-assets is built on it as written, and merges once the owner has ruled.
- **D3**, ruled: the owner accepted it in the message "all five defaults accepted, proced". Its text: on #unsaved-close-warning: the question before closing an in-memory window with unsaved
  edits contradicts the title of `design@rootpoint-app@no-discard-confirmation`, "No action of the
  window that discards content asks a confirmation, since undo restores what it took", whose ground
  is the owner's. Default: the head's title becomes "No action of the window that undo can reverse
  asks a confirmation", and a close in memory with unsaved edits, which no undo reaches, is named
  in its body as the case #unsaved-close-warning asks for; rewritten at #workspace-host, which
  builds the native question. Found by the review of this document.
- **D4**, accepted by the owner in the message "all five defaults accepted, proced", on the narrower
  statement of the principle; put to the owner again with the principle whole, its default
  changed, and ruled, as below. Its text: on #asset-paths-only: the owner's word of round 4, "the project should not accept relative
  path, and especially not one sitting outside the workspace directory", against the owner's
  principle in `design@rootpoint-model@cosmetic-fallbacks`, that "a cosmetic one falls back
  wherever a fallback can be used, in the owner's opinion, and that a file whose structural change
  makes it impossible to render properly, or not obvious for a user to recover from, may be
  refused", as the head records it; the reply that put D4 stated the principle without its hedge
  and its last clause. Its first default was: `load_ron` refuses such a file, as a structural
  breach, on the round 4 word. Put again whole, the owner answered: "i think it is enough to refuse at
  runtime to look up the path pointing outside the workspace. but i see no strong argument between
  both shapes: refuse at load, or load but never read or write the file." The session's default
  now: load, and never read or write outside the root, "Decided design" of #asset-paths-only. The
  runtime refusal exists anyway, since a symlink out of the root is the disk's state and is found
  only at a read; a refusal at load would add a second mechanism for the paths the RON names, and
  refuse a file whose layout and node ids are defined, against the structural class of
  `design@rootpoint-model@validated-documents` and `goal@rootpoint-model@faithful-readable-file`;
  a repair at load, copying the outside file in, would read outside the root. Ruled: the owner
  answered "agreed on this new shape, best for both users and protection I believe."
- **D5**, ruled: the owner accepted it in the message "all five defaults accepted, proced". Its text: on #c-visible-mode: the owner ruled the form "the window always shows one of three
  states". The guard of #ron-overwrite-guard, which the owner accepted in the same round, leaves a
  workspace that is not saved, and the slices before #autosave-and-watch leave a workspace saved
  only by Ctrl+S; no one of the three labels is true of either. Default: the label takes the
  states of the mapping table "The store's state to the state label". Found by the review of this
  document; it bears on #workspace-host.

## Harvest

The harvest of each slice is the row in its spec. This document leaves in the commit that completes
the harvest of #browser-packs, the last slice; its row: every acceptance criterion reported once
more, as `skill@knowledge-architect-planning@acceptance-criteria` says; each tripwire T1 to T4
written with its decision; and `issue@rootpoint-app@browser-autosave` re-read against what the
browser build then holds.

## Later consequences

- A second window could open a workspace another holds, read-only, for example to present on
  another screen: an idea the session raised in round 1 and proposed nothing for. The workspace
  lock refuses it today; a later design would take a shared lock, `File::try_lock_shared`.

- `issue@rootpoint-app@browser-autosave` gives the browser build a store of its own, which would add
  a third store to #editing-modes, or a workspace in the browser.
- `issue@rootpoint-app@wider-lock` adds locks inside one window; the workspace lock stays one per
  process.
- Video, `issue@rootpoint@more-content-kinds`, meets T4 first.
- A pack from an untrusted source, beyond confinement, is a later design, on the owner's word of
  round 4.
- Windows: TCP with a token is the candidate transport, #arg-19, and `rootpoint-ctl` resolving the
  socket keeps that change out of every client.
