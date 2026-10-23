# Slice: Named styles in the window

The fifth slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec. It builds the window's half of `thread@editor-conveniences@named-styles`; the model and
the protocol are `spec@editor-conveniences@styles-model`.

## Builds

- **The style row of a text area**, in `App::style` of `path@rootpoint-app@src/panels.rs`.
  - A combo box lists the table's styles and "custom", with the area's choice selected. Choosing a
    style sends `text <area> "<text>" style="<name>"`; choosing "custom" sends
    `text <area> "<text>" custom`.
  - Under it, the size, bold, italic, colour, alignment and font controls show the resolved values.
  - A change to any of them sends `text` with that option, which forks a named area to custom, per
    `spec@editor-conveniences@styles-model`. No control of the selection changes a named style.
- **The Styles tab**, `App::styles_tab`: a second tab of the side panel, "Selection | Styles".
  - The tab lists each style with the number of areas, over every slide, that name it.
  - A click on a style selects it in the tab. That selection is the window's own state, not the
    session's view: it shows nothing on the slide.
  - Below the list, the selected style's controls send `style set`. A name field sends
    `style rename` when it loses focus or Enter is pressed.
  - "New" sends `style add` with a free name, "Style 1", "Style 2", …. "Duplicate" sends
    `style add` with the selected style's values.
  - "Remove" sends `style remove` at once when no area uses the style. Otherwise it opens the
    replacement dialog below.

  Every change is one command and one undo step. A drag control is not used, for the reason the
  comments of `SIZES` and `COLOURS` give. An illustration of the shape, not authority:

  ```
   [Selection] [Styles]
   Body       7 areas
   Legend     2 areas
   Subtitle   1 area
  >Title      4 areas
   ── Title ──────────────
   name [Title      ]
   size [120 ▾] ☑ bold ☐ italic
   ■■■■■■■■  left centre right | top middle bottom
   [New] [Duplicate] [Remove]
  ```
- **The replacement dialog** of a style in use, an egui window:
  - its text is "Remove "<name>", used by <n> areas?";
  - a choice offers "a custom copy", selected first, then every other style;
  - "Remove" sends `style remove "<name>" into=custom` or `into="<other>"`; "Cancel" sends nothing.

  The dialog closes when the socket takes the authority, as a gesture ends,
  `design@rootpoint-app@control-passing`.
- **The authority.** While the socket holds the authority to edit, every control of both tabs that
  sends an edit is disabled. A click that selects a style in the list still works.
- **The width test** gains the state of the Styles tab with a style selected, and judges AC2.
- **AC1's test**, `acceptance@editor-conveniences@ac-missing-style-warned`: the status line, the
  outline's row and the reply of `render`.

## Claims

- **Choosing a style names it; choosing custom copies it.** Test: `path@rootpoint-app@tests/actions.rs`
  checks the two actions against the same commands on a second session, and again after one `undo`.
  Shown to fail: against a combo box that sends `style set`, the second session's Title changes.
- **A control of the selection never changes a named style.** Test: on a Body area, a click on the
  size 64 makes the area custom at 64, and Body's size stays 40. Shown to fail: against `App::style`
  sending `style set` for a named area, Body's size becomes 64.
- **The Styles tab edits the table.** Tests, in `path@rootpoint-app@tests/actions.rs`: one test per
  action of the tab, each checked against its command on a second session. The actions are New,
  Duplicate, a rename, a size change, and Remove of an unused style. Shown to fail: against a rename
  that changes the table only, the second session's areas name a missing style.
- **Removing a style in use asks for a replacement.** Test: a test of `App::frame` clicks Remove on
  Body in the sample. The dialog is open, and no command has run. "Remove" with "a custom copy" then
  leaves both users custom with Body's values, and one `undo` restores Body. Shown to fail: against
  a Remove that sends `style remove` at once, the session replies `err` and the dialog never opens.
- **AC1.** The test of `acceptance@editor-conveniences@ac-missing-style-warned`.
- **AC2.** The width test, with the Styles tab.

## Audit subjects

- `App::side_panel`, `App::content`, `App::style` and the outline in
  `path@rootpoint-app@src/panels.rs`; `path@rootpoint-app@src/actions.rs`;
  `path@rootpoint-app@tests/actions.rs`, which also runs on the wasm target.
- `design@rootpoint-app@actions-are-commands`, `design@rootpoint-app@one-controller-at-a-time`,
  `design@rootpoint-app@control-passing`, `design@rootpoint-app@gesture-groups`;
  `tripwire@rootpoint-app@control-ends-a-gesture`, which the dialog's closing meets.

## Fails alone on

- A control of the Selection tab changes a named style, or a control of the Styles tab does not.
- The width test differs in the Styles tab.

## Premises that expire

None: the slices after this one add no content to the side panel.

## Decided design

The window's half of `thread@editor-conveniences@named-styles`: **only the Styles tab changes a
named style**. The selection's controls never change one, and a change from them makes the area
custom. So a change to a style that slides out of view use is made only from the view that lists
those uses, answering the cost that `rejected@rootpoint-model@content-by-reference` names, "editing a
shared content changes every slide that shows it" (`argument@editor-conveniences@arg-19`). The Styles
tab is a tab of the side panel, so the panel's width stays as
`thread@editor-conveniences@sidebar-width` fixes it.

The window's half of `thread@editor-conveniences@style-removal`: **a removal in use opens a choice of
replacement, with a custom copy first**. A custom copy is the only replacement that changes no
slide (`argument@editor-conveniences@arg-54`). The dialog is a choice the removal needs, not a
confirmation, so it does not bring back `thread@editor-conveniences@removal-guard`. A removal of a
style discards no content of a slide, so `design@rootpoint-app@no-discard-confirmation` does not
cover it.

## Implementation sequence

1. The style row of a text area, with the combo box and the fork to custom. Fails alone on the
   selection's claims.
2. The Styles tab and its actions. Fails alone on the tab's action tests.
3. The replacement dialog. Fails alone on the dialog's test.
4. AC1's and AC2's tests. Fails alone on AC1 or AC2.

## Acceptance criteria

This slice judges `acceptance@editor-conveniences@ac-missing-style-warned` (AC1) and
`acceptance@editor-conveniences@ac-panel-width-constant` (AC2), and reports both in its landing
commit.

## Harvest

- An app head: only the Styles tab changes a named style, and a change from the selection makes the
  area custom. It passes entry test 2, a reason respected by every control of both tabs.
- The replacement dialog, under the head of `thread@editor-conveniences@style-removal` written by
  `spec@editor-conveniences@styles-model`, or a comment at the dialog if that head holds the rule
  already.
- `design@rootpoint-app@one-controller-at-a-time`: re-read; the Styles tab's edits return while the
  socket holds the authority, and a selection in its list is a view-only input of the window.
