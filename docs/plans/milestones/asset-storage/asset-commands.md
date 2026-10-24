# Slice: the asset commands and the trash

The fifth slice of `milestone@plans@asset-storage`. It lands the commands of
`thread@asset-storage@asset-manager-scope` and the trash of `thread@asset-storage@asset-disk-changes`,
in both stores, headless.

## Builds

- `Files::rename(from, to)` and `Files::remove(path)`; `Files::list`, which #pack-commands lands for
  `unpack`, serves the trash's emptying and `assets untracked`. The two new ones are in
  `path@rootpoint-protocol@src/files.rs`, default methods replying `Err("unsupported")`, implemented
  by `NativeFiles`, the app's `MemoryFiles` and the protocol tests' `MemoryFiles` and
  `RecordingFiles`; a native rename creates the directories above its target, as a write does.
- The commands `asset-move <a> <asset-path>`, `asset-relink <a> <asset-path>`,
  `asset-replace <a> <host-path>`, `asset-remove <a> [into=<a>]`, `asset-remove-unused` and
  `assets [all|unused|missing|untracked]`, the last one reply line per asset; `remove-asset` leaves.
  A move to a path that holds a file, a relink to a path that holds no file, a move or a relink to
  a path that fails confinement, and a removal of a used asset with no `into=` reply `err`, so no
  command creates a breach or names a file that is not there, `design@rootpoint-protocol@no-new-breach`.
  On an asset whose path fails confinement, each command replies as the milestone's "Decided
  design" of #asset-paths-only says. `asset-remove-unused` is one undo step, and none when no
  asset is unused.
  - `asset-move` of an asset whose file is missing replies `err`, naming `asset-relink`.
  - `asset-remove <a> into=a<n>` names its replacement by its id, as `image` names an asset: every
    image of `a`, every text whose own font is `a` and every named style whose font is `a` take
    `a<n>`, which must be another asset of the table, then `a` leaves; one undo step.
  - `assets` replies one line, as every reply: its entries, in the order of the ids, separated by
    `; `, each `a<id> <path> <bytes|missing> uses=<nodes|->`, with `<path>` quoted as a path is;
    `assets untracked` replies the paths alone. The size is the length of the file the store
    reads, and `missing` an asset it cannot read, an unconfined one included.
- The trash: in a workspace, `.rootpoint/trash/<id>-<n>`, filled by a removal and by a replace,
  emptied by `open` of the workspace; in memory, `Session::trash: BTreeMap<String, Vec<u8>>` under
  the same names. `<n>` counts the session's entries, since one asset replaced twice keeps two
  versions, which its undo and its redo take back in turn. An undo step holds the file moves of its
  command with its operations, and undo and redo apply both, a step whose move fails applying
  neither.
  - A step is its operations and its moves, each move a rename between an asset path and the
    trash, or between two asset paths; its inverse runs the moves back in reverse order. A group of
    `begin` and `end` gathers both.
  - `asset-replace` writes the new bytes to an entry of the trash first, then moves the old file to
    the trash and the new one to the asset's path: every change of the store is then a move, so
    the undo of a replace is one too.

## Claims

- **Undo restores a removed asset's file with its bytes**, in a workspace and in memory; shown to
  fail with the trash bypassed.
- **AC4**, `acceptance@asset-storage@ac-trash-emptied`: a removal, a new session and an `open` of
  the workspace leave `.rootpoint/trash/` empty.
- **A move keeps every slide**: after `asset-move`, every area and style that named the asset draws
  it, and `presentation.ron` holds the new path once saved.
- **A relink repairs a missing asset**: an asset whose file is gone, relinked to an untracked file,
  draws it, and `assets missing` no longer lists it.
- **`asset-remove` with `into=`** makes every area and style name the replacement, one undo step.
- **`assets untracked`** lists a file under the root that no asset names, and never `.rootpoint/`
  or `presentation.ron`.
- **No command creates a breach**: the property test gains the six commands.
- **AC2 on the asset commands**, `acceptance@asset-storage@ac-confined-paths`: on an asset loaded
  with a path that fails confinement, `asset-move` and `asset-replace` reply `err`, `asset-remove`
  removes the entry and moves no file, and `asset-relink` to a confined path repairs it, each giving
  `RecordingFiles` no path outside the root.

## Audit subjects

- `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@src/command.rs`,
  `path@rootpoint-protocol@tests/properties.rs`, `path@rootpoint-model@src/ops.rs`.
- `design@rootpoint-model@cosmetic-fallbacks`, `design@rootpoint-protocol@no-new-breach`,
  `design@rootpoint-protocol@gesture-undo`, `design@rootpoint-model@stable-ids`,
  `design@rootpoint-model@style-removal`, whose `into=` the asset removal mirrors.

## Fails alone on

- Undo of a removal in a workspace, and AC4.

## Premises that expire

- The table holds only added assets; T3 watches an author who expected an untracked file packed.

## Decided design

The milestone's sections on `thread@asset-storage@asset-tree-shape`,
`thread@asset-storage@asset-disk-changes` and `thread@asset-storage@asset-manager-scope` hold the
shape.

## Acceptance criteria

This slice judges `acceptance@asset-storage@ac-trash-emptied`, and
`acceptance@asset-storage@ac-confined-paths` on the asset commands.

## Harvest

- Heads of the protocol for the asset commands and the trash; rewrite
  `design@rootpoint-protocol@gesture-undo` for a step that holds file moves.
- The three decisions of `thread@asset-storage@asset-tree-shape`, as its harvest says: ids kept,
  read first as a member of `design@rootpoint-model@stable-ids`; folders from the paths; a table of
  added assets. `rejected` entries for content naming an asset by its path, for folders stored in
  the RON and for a table that mirrors the disk, #arg-9, by the tests of
  `skill@knowledge-architect-decision-recording`.
- Write T3, guarding the head of the table of added assets.
- Update `path@rootpoint-protocol@README.md` for the commands.
