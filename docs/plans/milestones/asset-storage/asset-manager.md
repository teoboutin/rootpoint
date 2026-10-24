# Slice: the asset tab

The sixth slice of `milestone@plans@asset-storage`. It lands the window's half of
`thread@asset-storage@asset-manager-scope`: a tab of the side panel that drives the commands of
#asset-commands.

## Builds

- `path@rootpoint-app@src/asset_tab.rs`: the asset tab, beside the Styles tab, keeping the panel's
  width, `design@rootpoint-app@sidebar-width`. It shows the tree of asset paths, folders built from
  their prefixes, with a search by name and the filter of `assets`; for the selected asset, its size,
  a thumbnail and its uses, each use a button that selects its node; and the actions: move or rename
  by typing a path or by a drag onto a folder, replace, relink a missing asset (an untracked file
  of the same bytes offered first), remove, remove all unused, remove with a replacement, and
  import by a file dialog or by a drop of a file on the window. The total size of the assets is
  shown at the tab's foot.
  - The tab reads the document and the store, and runs no query through `App::run`, which would
    render the slide again at each frame. What it reads from the files, each asset's size and the
    untracked files, is kept until the next command or watcher event, so a frame reads no file.
  - A missing asset has no bytes to compare, so the relink offers first the untracked files of its
    file's name, then the other untracked files, then a path typed.
  - The thumbnail is the selected image asset's, decoded once per selection and per change of the
    store's files, by `rootpoint_render::decode_rgba`, a public function `ImageSet::insert_bytes`
    also uses, and drawn as an egui texture at most 128 points wide; a font shows no thumbnail.
  - A drag of an asset's row onto a folder's row, or onto the root's, runs `asset-move` to that
    folder under the asset's file name, through egui 0.37's `Ui::dnd_drag_source` and
    `Ui::dnd_drop_zone`.
  - A dropped file is `RawInput::dropped_files`, each an `egui::DroppedFileHandle`: natively its
    `path()` is absolute, read in `path@elsewhere@egui-0.37.0/src/data/input/dropped_file.rs`, and the window runs
    `import <path>`; in the browser, `path@elsewhere@eframe-0.37.0/src/web/events.rs` gives the file's name as
    its path and `bytes_async()` reads it, so `path@rootpoint-app@src/web.rs` reads it as it reads a
    picked image, and `App::add_picked_image` imports it. Replace in the browser takes a picked
    file the same way, held in `MemoryFiles` under its name, then `asset-replace`.
- Each action is a function of `path@rootpoint-app@src/actions.rs` running the commands of
  #asset-commands through `App::run`, per `design@rootpoint-app@actions-are-commands`.
- The node menu's image entries, labelled by `App::asset_name`, and the outline's image rows, per
  `design@rootpoint-app@tree-outline`, show the asset path. Each action of the tab runs the command
  the milestone's "Decided design" of #asset-manager-scope names for it.

## Claims

- **Each action matches a second session** running the same commands, in
  `path@rootpoint-app@tests/actions.rs`, natively and in the wasm test.
- **A use's button selects its node and shows its slide**, a test of `App::frame`.
- **The tab keeps the panel's width**, a test of `App::frame` that switches tabs.

## Audit subjects

- `path@rootpoint-app@src/panels.rs`, `path@rootpoint-app@src/actions.rs`,
  `design@rootpoint-app@styles-tab-edits-styles`, `design@rootpoint-app@sidebar-width`,
  `design@rootpoint-app@node-menu`, `design@rootpoint-app@tree-outline`.
- How eframe 0.37 reports a dropped file, natively and in the browser.

## Fails alone on

- An action of the tab whose commands differ from the second session's.

## Premises that expire

None.

## Decided design

The milestone's section on `thread@asset-storage@asset-manager-scope` holds the shape.

## Acceptance criteria

None is judged at this slice.

## Harvest

- A head of the app for the asset tab, by the tests of `primer@design-heads`, citing
  `design@rootpoint-app@styles-tab-edits-styles`'s reason.
- Update `design@rootpoint-app@node-menu` and `design@rootpoint-app@tree-outline` where they name an
  image by its file's name, and `path@rootpoint-app@README.md`.
