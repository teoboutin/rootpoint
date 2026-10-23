# Slice: The node menu

The sixth slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec.

## Builds

- **A secondary click on an area of the slide, or on a row of the outline,** selects that node, with
  `select`. It then opens the node menu, `App::node_menu`, on that node. A secondary click on a
  divider's band or on the grip does the same for the area under the pointer, since no drag starts
  on a secondary button. A secondary click on the letterbox, or on the outline's row of the slide,
  which names no node, opens no menu; the slide's row shows its slide, as its click does.
- **The menu is a state of the window**, `NodeMenu` in `App::menu`: the node it was opened on and
  the point of the click. `App::menu_node` returns its node, `None` when it is closed, and
  `App::menu_entry_rect` where the last frame put an entry, by its label, as `App::button_rect` does
  for the side panel. It is drawn after the central panel, as the dialog of a style's removal is,
  with an `egui::Popup` whose open state is the window's, `Popup::open_bool`, never egui's memory, so
  that the socket's `control` and a test close it by clearing `App::menu`.
- **The menu**, an egui popup at the pointer, its entries in one column, each section under its
  name, with no submenu: egui opens a submenu on the pointer's hover, which a test of `App::frame`
  would have to imitate frame by frame. An illustration of the shape the owner approved, not
  authority:

  ```
   Select   ▸ n4 stacked
              n6 beside
              n3 text  ✓
   Content  ▸ ● text   ○ image ▸ a19 splits.png …
   Style    ▸ ● Body ○ Title ○ Subtitle ○ Legend ○ custom
   ─────────
   Remove n3
  ```

  - "Select" lists the node's ancestors from the root down, then the node itself, each with its
    kind, as `selection::path` of `path@rootpoint-app@src/selection.rs` gives them, and a click
    selects it.
  - "Content", on an area, offers text and image, as the side panel's content row does, image taking
    the first image asset as its button does; on an image area, it also offers each image asset, as
    the list of `App::image` does.
  - "Style", on a text area, offers the table's styles and custom, as the side panel's
    `App::style_choice` does.
  - "Remove" removes the node, as Delete does. It is absent on the slide's root.

  Each entry runs the action the side panel's equivalent runs, a function of
  `path@rootpoint-app@src/actions.rs`.
- **While editing**, a secondary click anywhere ends the editing first, as any press outside the
  edited area does, `design@rootpoint-app@edit-entry`. It then acts as above.
- **While the socket holds the authority**, the menu opens with only "Select" enabled. The menu
  closes when the socket takes the authority, as a gesture ends.
- **The menu closes whenever the selection is no longer its node**, as after a key, an undo or the
  socket's `select`: every entry but "Select" acts on the selection, so an entry never acts on
  another node than the one the menu names.
- **The menu closes** on a click of one of its entries, on a click outside it, which then acts as it
  would, and on Esc, egui 0.37.0's `PopupCloseBehavior::CloseOnClick`; a click inside it that is no
  entry also closes it, as egui's own context menus do. The `Popup::show` of egui 0.37.0 closes a
  popup only on a click whose press began while it was already open, so the release of the
  secondary click that opened it does not close it.

## Claims

- **AC3.** The two tests of `acceptance@editor-conveniences@ac-right-click-menu`.
- **"Select" selects an ancestor.** Test: on n3 of the sample, the menu's entry n4 leaves n4
  selected. Shown to fail: against an entry that selects the clicked node, n3 stays selected.
- **"Content", "Style" and "Remove" run the side panel's actions.** Test: each entry checked against
  the same commands on a second session in `path@rootpoint-app@tests/actions.rs`, and after one
  `undo`. Shown to fail: against a "Remove" that removes the parent, the documents differ.
- **The menu respects the authority.** Test: in `path@rootpoint-app@tests/control.rs`, with the
  socket holding control, a secondary click opens a menu whose "Remove" is disabled and whose
  "Select" works. Shown to fail: against a menu that ignores the controller, "Remove" removes.

## Audit subjects

- `App::slide_view` and the press handling of `path@rootpoint-app@src/view.rs`, `Press` in
  `path@rootpoint-app@src/hit.rs`; the outline's rows; `path@rootpoint-app@src/actions.rs`.
- egui 0.37.0's popup and its reading of `PointerButton::Secondary`, read in its source at the
  audit.
- `design@rootpoint-app@edit-entry`, `design@rootpoint-app@one-controller-at-a-time`,
  `design@rootpoint-app@control-passing`; `tripwire@rootpoint-app@control-ends-a-gesture`.
- In the browser: eframe 0.37.0 prevents the browser's context menu on the canvas
  (`argument@editor-conveniences@arg-13`), read in `install_event_handlers` of eframe's
  `path@elsewhere@src/web/events.rs`. No browser test checks it: the harness of
  `path@rootpoint-app@tests/web.rs` runs no `WebRunner`, and installs an imitation of eframe's
  listener for its F5 test.

## Fails alone on

- A secondary click does not select, or opens no menu.
- An entry of the menu does other than its equivalent in the side panel.

## Premises that expire

None.

## Decided design

**Right-clicking an area or an outline row selects that node and opens a menu**,
`thread@editor-conveniences@node-menu`. The owner asked for a menu that selects among the parents,
removes, and changes the content kind and the style (`argument@editor-conveniences@arg-5`). The
click selects first, so the menu and the side panel describe one node. The nearest rival, a menu
that leaves the selection, would describe another node than the panel does
(`argument@editor-conveniences@arg-26`). It has no entry to split, since the panel's buttons and
Shift with an arrow do that. Right-click works in the browser, since eframe stops the browser's own
menu on the canvas (`argument@editor-conveniences@arg-13`).

## Implementation sequence

1. The secondary click selects and opens an empty menu, on the slide and on the outline. Fails alone
   on AC3.
2. The entries, and the authority. Fails alone on the entries' tests.

## Acceptance criteria

This slice judges `acceptance@editor-conveniences@ac-right-click-menu` (AC3), and reports it in its
landing commit.

## Harvest

- An app head for the node menu: a right-click selects, then opens the menu. It passes entry test 2:
  the slide view and the outline both respect it.
- `design@rootpoint-app@one-controller-at-a-time`: the menu's "Select" joins the view-only inputs.
