# Slice: the asset tab

The sixth slice of `milestone@plans@asset-storage`. It lands the window's half of
`thread@asset-storage@asset-manager-scope`: a tab of the side panel that drives the commands of
#asset-commands.

## Builds

- `planned@rootpoint-app@src/asset_tab.rs`: the asset tab, beside the Styles tab, keeping the panel's
  width, `design@rootpoint-app@sidebar-width`. It shows the tree of asset paths, folders built from
  their prefixes, with a search by name and the filter of `assets`; for the selected asset, its size,
  a thumbnail and its uses, each use a button that selects its node; and the actions: move or rename
  by typing a path or by a drag onto a folder, replace, relink a missing asset (an untracked file
  of the same bytes offered first), remove, remove all unused, remove with a replacement, and
  import by a file dialog or by a drop of a file on the window. The total size of the assets is
  shown at the tab's foot.
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
