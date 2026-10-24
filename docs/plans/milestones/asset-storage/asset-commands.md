# Slice: the asset commands and the trash

The fifth slice of `milestone@plans@asset-storage`. It lands the commands of
`thread@asset-storage@asset-manager-scope` and the trash of `thread@asset-storage@asset-disk-changes`,
in both stores, headless.

## Builds

- `Files::rename(from, to)` and `Files::remove(path)`, in `path@rootpoint-protocol@src/files.rs`,
  implemented natively and by `MemoryFiles`.
- The commands `asset-move <a> <asset-path>`, `asset-replace <a> <host-path>`,
  `asset-remove <a> [into=<a>]` and `assets [all|unused|missing|untracked]`, the last one reply line
  per asset; `remove-asset` leaves. A move to a path that holds another asset's file, or a removal
  of a used asset with no `into=`, replies `err`, so no command creates a breach,
  `design@rootpoint-protocol@no-new-breach`.
- The trash: in a workspace, `.rootpoint/trash/<id>`, filled by a removal and by a replace, emptied
  when the workspace opens; in memory, a second map. Undo and redo move bytes between the trash and
  the asset path with the operation they belong to.

## Claims

- **Undo restores a removed asset's file with its bytes**, in a workspace and in memory; shown to
  fail with the trash bypassed.
- **AC4**, `acceptance@asset-storage@ac-trash-emptied`: a removal, a close and an open leave
  `.rootpoint/trash/` empty.
- **A move keeps every slide**: after `asset-move`, every area and style that named the asset draws
  it, and `presentation.ron` holds the new path.
- **`asset-remove` with `into=`** makes every area and style name the replacement, one undo step.
- **`assets untracked`** lists a file under the root that no asset names, and never `.rootpoint/`
  or `presentation.ron`.
- **No command creates a breach**: the property test gains the four commands.

## Audit subjects

- `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@src/command.rs`,
  `path@rootpoint-protocol@tests/properties.rs`, `path@rootpoint-model@src/ops.rs`.
- `design@rootpoint-model@cosmetic-fallbacks`, `design@rootpoint-protocol@no-new-breach`,
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

This slice judges `acceptance@asset-storage@ac-trash-emptied`.

## Harvest

- Heads of the protocol for the asset commands and the trash, and of the model for
  `thread@asset-storage@asset-tree-shape`; `rejected` entries for content naming an asset by its
  path and for a table that mirrors the disk, #arg-9, by the tests of
  `skill@knowledge-architect-decision-recording`.
- Write T3, guarding the tree's head.
- Update `path@rootpoint-protocol@README.md` for the commands.
