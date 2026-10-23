# Slice: The outline, the node keys and the panel's width

The fourth slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec.

## Builds

- **The outline**, `App::outline` in `path@rootpoint-app@src/panels.rs`, in place of
  `App::breadcrumb`.
  - Its first row is the shown slide. A click on it shows the slide with no node selected, as the
    breadcrumb's first entry does.
  - Then one row per node of the slide's tree, in depth-first order, each indented by its depth and
    always expanded. A split's row shows its axis; an area's row shows its content.
  - The selected node's row is highlighted.
  - A click on a row selects its node.
  - While the pointer is over a row, the overlay outlines that node on the slide.

  An illustration of the shape the owner approved, not authority:

  ```
   Slide s1
   [=] n4  stacked
   │  T  n2  Title   "rootpoint"
   │ [|] n6  beside
   │  │  T  n3  Body    "A slide is a tree of…"   ◀ selected (highlighted row)
   │  │  T  n5  Body    "Click an area to…"
  ```

  `[=]` and `[|]` mark a split stacked and beside. `T` marks a text, with its style name and the
  start of its text: `custom` for a custom style, and `⚠` before the name of a missing one. `I`
  marks an image, with its asset's file name. The glyphs are the implementer's, within egui's
  default fonts. A row's text is cut to the panel's width, never wrapped and never widening it.
- **The node keys follow the outline**, `thread@editor-conveniences@selection-keys`:
  - Alt+↑ and Alt+↓ select the previous and the next sibling;
  - Alt+← selects the parent;
  - Alt+→ selects the child last selected, or the first child.

  The table of `App::shortcuts` maps each arrow to its `selection::Move`. `selection::Memory` and
  Shift with an arrow are unchanged.
- **The width.** `Panel::right("selection")` stays resizable by a drag of its edge, starting at
  260 points, and no row of its content widens or narrows it. egui starts each frame of a panel from
  the size its content took in the last one, so a widget fills the available width, a `ScrollArea`
  or `Ui::take_available_space`, as egui's doc of `Panel::resizable` asks, and the rows wrap or cut
  their text inside it.
- **The sample's help text** in `path@rootpoint-protocol@sample/sample.ron` names the new keys and
  the outline in place of the breadcrumb.
- **The width test**, AC2's instrument, in the states that exist at this slice: the slide alone, a
  text area, an image area, a text of 200 characters, and a tree 6 levels deep.

## Claims

- **The outline lists every node, in depth-first order, with the selection highlighted.** Test: a
  test of `App::frame` on the sample reads the outline's rows, recorded as `App::slide_rows` records
  the slide list's. It finds s1, n4, n2, n6, n3, n5 at depths 0 to 3, and n3's row highlighted after
  `select n3`. Shown to fail: against an outline that lists areas only, the row of n4 is missing.
- **A click on a row selects its node; a click on the slide's row selects none.** Test: clicks on
  the rows of n6 and of s1 leave the view's selection at n6, then at none, with s1 shown. Shown to
  fail: against rows that do not sense a click, the selection is unchanged.
- **The keys move as the outline reads.** Test: from n2, Alt+↓ selects n6 and Alt+→ selects n3;
  from n3, Alt+↓ selects n5, Alt+← selects n6, and Alt+↑ at n3 does nothing. Shown to fail: against
  the former table, Alt+↓ from n2 selects nothing.
- **Hovering a row outlines its node.** Test: with the pointer on n6's row, the overlay's marks of
  that frame include the rectangle of n6 in `App::layout`. The test reads the marks through the
  recorded layout, as the other overlay tests do. Shown to fail: against an outline with no hover,
  no mark has n6's rectangle.
- **The width is the same in each state that exists.** Test: the width test above, two frames after
  each change, as `path@rootpoint-app@CLAUDE.md` says of a test that reads a place. Shown to fail:
  against a `Panel::right` whose rows are laid out in `ui.horizontal`, as the style rows are today,
  the widths of the slide alone and of a text area differ.

## Audit subjects

- `App::side_panel`, `App::breadcrumb` and `App::style` in `path@rootpoint-app@src/panels.rs`;
  `App::editor` and `App::shortcuts` in `path@rootpoint-app@src/lib.rs`;
  `path@rootpoint-app@src/selection.rs`; `App::overlay` in `path@rootpoint-app@src/view.rs`.
- `design@rootpoint-app@node-selection`, `design@rootpoint-app@span-selection`,
  `design@rootpoint-app@editor-keys-kept-from-browser`, `design@rootpoint-app@one-controller-at-a-time`,
  `tripwire@rootpoint-model@grouping-lost-to-merge`,
  which the outline makes visible.
- egui 0.37.0's `Panel` sizing, in its `containers/panel.rs`, measured by the width test rather
  than read.

## Fails alone on

- A row of the outline is missing, misplaced, or does not select its node.
- An Alt key selects other than the outline's neighbour.
- The panel's width differs between two states.

## Premises that expire

- The Styles tab does not exist yet. `spec@editor-conveniences@styles-window` adds it and its state
  to the width test, and judges AC2.
- The outline's `⚠` on a missing style is drawn here, from `Document::breaches`. AC1 is judged at
  `spec@editor-conveniences@styles-window`.

## Decided design

**The side panel shows the slide's whole tree as an indented outline**,
`thread@editor-conveniences@tree-outline`. The owner asked for the whole tree with the selection
highlighted and splits told from areas (`argument@editor-conveniences@arg-1`). An outline grows
downward, one row per node. The nearest rival, an org chart with the root on top, needs one column
per area, and a 10-area slide does not fit in a 260-point panel
(`argument@editor-conveniences@arg-16`). It is always expanded, since a slide's tree is small and
collapsing would add state.

**Alt with ↑ and ↓ selects a sibling, and Alt with ← and → the parent and a child**,
`thread@editor-conveniences@selection-keys`. In the outline, siblings are the rows above and below
at one depth, and the parent and the children are a step left and right. On the owner's word: the
owner chose this shape for now, as consistent with the outline, over Alt with ↑ and ↓ moving one row
of the outline, which the owner found possibly a bit more natural. The owner noted that it is not
hard to change. The rival makes Alt with → redundant and drops the move at one depth
(`argument@editor-conveniences@arg-17`). On the slide, Alt+↓ moves down in a stacked split and right
in a beside split. The former keys had the same mismatch on the other axis. T1 watches for the
confusion, and names the spatial keys as a response.

**The side panel's width never follows its content**, `thread@editor-conveniences@sidebar-width`. A
width that changes with the selection resizes the slide view at a click
(`argument@editor-conveniences@arg-3`). egui sizes a panel from what its content took in the last
frame, wider or narrower (`argument@editor-conveniences@arg-12`, and the reading of egui's source in
"What the work is" of the milestone document). So the panel keeps the width its edge was dragged to, and
its rows fit inside it. The rival, a fixed width with no drag, gives a deep tree or a long style name
no room (`argument@editor-conveniences@arg-18`).

## Implementation sequence

1. The width: the rows fit the panel, and the width test lands. Fails alone on the width test.
2. The outline in place of the breadcrumb, its clicks and its hover. Fails alone on the outline's
   tests.
3. The keys' table and the sample's help text. Fails alone on the keys' test.

## Acceptance criteria

This slice lands the instrument of `acceptance@editor-conveniences@ac-panel-width-constant` (AC2),
judged by `spec@editor-conveniences@styles-window`.

## Harvest

- `design@rootpoint-app@node-selection` rewritten: the outline and the keys. The keys are attributed
  to the owner with the owner's hedges, "for now" and "not very hard to change". The texts that cite
  it are re-read: `design@rootpoint-app@edit-entry`, `design@rootpoint-app@editor-keys-kept-from-browser`
  and the comment of `path@rootpoint-app@src/hit.rs`.
- `design@rootpoint-app@span-selection`: "Alt+↑ or the breadcrumb" becomes Alt+← or the outline.
- `design@rootpoint-app@one-controller-at-a-time`: the outline joins the inputs that change only the
  view, in place of the breadcrumb.
- T1, recorded on the owner's word in `register@rootpoint-app@tripwire`, guarding
  `design@rootpoint-app@node-selection`, with its label in its text.
- The width: a comment at `App::editor`, with the commit's argument; no head, since one site holds
  the reason.
- The note of `path@rootpoint-app@CLAUDE.md` that a selection changes the side panel's size on the
  next frame is re-read: it is false once no content changes the width.
