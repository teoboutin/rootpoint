<!-- The report of the prior-art survey run on 2026-10-21 for the design session held at the audit
of the slice that builds the native GUI, in the milestone of the first prototype, on how shipped
tools edit a split layout and group undo steps. It is the message the subagent returned, verbatim,
since it could not write a file; the slice's arguments quote it. Its scratchpad paths no longer
exist. -->

I could not write report.md: the harness refuses report files from a subagent ("Subagents should return findings as text"). The report is below instead. The source copies the survey read are kept in /tmp/claude-266682/-home-catA-tb266682-Documents-code-rootpoint/3aa7f0e1-0c36-4134-9ed6-7435d1cdd42e/scratchpad/survey/ under a/src, b/src and c/dl.

Three subagents read the sources: one for Blender, i3, sway and tmux; one for the GUI widgets and editors; one for undo grouping. I re-read these load-bearing claims myself in the downloaded sources:
- tmux layout.c:943, the same-axis insert;
- Blender screen_ops.cc:3083-3092, the twelfths;
- iced pane_grid.rs:95-96, the dead band and the edge strip;
- egui_tiles lib.rs:214, the joining of nested same-axis containers;
- egui 0.37 style.rs:1594, a grab radius of 3.0;
- Qt qundostack.cpp:752-753, undo refused inside a macro;
- Apple `undo()`, "Closes the top-level undo group if necessary";
- egui undoer.rs:29-31, its defaults.

V means verified: read in the cited source. A means assumed.

**Revisions:**
- Blender: manual 5.2, source `main`.
- i3: be04c853 (4.25).
- sway: 1652c54b (1.13-dev).
- tmux: 3.4.
- VS Code: fdb4b3c5.
- iced: 0.14.0.
- egui_tiles: 0.18.0, on egui 0.37.
- Dear ImGui docking branch: d88f238d.
- Qt: 6.12 docs, qtbase `dev`.
- prosemirror-history: 1.5.0.
- @codemirror/commands: 6.10.3.

**URLs:**
- Blender manual: docs.blender.org/manual/en/latest/interface/window_system/areas.html
- Blender source: github.com/blender/blender/blob/main/source/blender/editors/screen/screen_ops.cc
- i3 userguide: i3wm.org/docs/userguide.html; source: github.com/i3/i3 at be04c853, src/tree.c, resize.c, tiling_drag.c
- sway: sway.5.scd at 1652c54b
- tmux: man.openbsd.org/tmux.1; github.com/tmux/tmux/blob/3.4/layout.c
- VS Code: github.com/microsoft/vscode at fdb4b3c5: src/vs/base/browser/ui/grid/gridview.ts, splitview/splitview.ts, sash/sash.ts, workbench/browser/parts/editor/editorDropTarget.ts, workbench.contribution.ts
- iced: github.com/iced-rs/iced/blob/0.14.0/widget/src/pane_grid.rs
- egui_tiles: docs.rs/crate/egui_tiles/0.18.0/source/src/
- Dear ImGui: github.com/ocornut/imgui at d88f238d, imgui.cpp
- Figma: help.figma.com/hc/en-us/articles/360040449873
- PowerPoint guides: support.microsoft.com/en-gb/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3
- Qt: doc.qt.io/qt-6/qundostack.html, qundocommand.html; raw.githubusercontent.com/qt/qtbase/dev/src/gui/util/qundostack.cpp
- Apple: developer.apple.com/documentation/foundation/undomanager/undo(), groupsbyevent
- GTK: gitlab.gnome.org/GNOME/gtk/-/raw/main/gtk/gtktexthistory.c
- Emacs: gnu.org/software/emacs/manual/html_node/elisp/Atomic-Changes.html
- ProseMirror: raw.githubusercontent.com/ProseMirror/prosemirror-history/master/src/history.ts
- CodeMirror: raw.githubusercontent.com/codemirror/commands/main/src/history.ts
- Blender undo: wm_event_system.cc lines 1279-1316 on `main`

## 1. Creating a split: direction and position

**Blender**
- **Corner drag** (V, `actionzone_modal`):
  - The direction is the dominant axis of the drag. A split triggers after 1.2 widget units of movement.
  - The position follows the pointer.
- **Area Options > Vertical/Horizontal Split** (V, manual and `area_split_modal`):
  - Opened by a right-click on a border, it shows an indicator line that follows the pointer.
  - The position is `factor` = (pointer − area_min) / area_size. The target area is re-read under the pointer on every move.
  - Tab or MMB toggles the axis. Esc or RMB cancels.
  - Ctrl snaps to twelfths of the area (V, screen_ops.cc:3083), or to a vertex of another area.
- **Minimum:** a split is refused below 2 × AREAMINX (29 px × UI scale) (V).

**i3 and sway**
- `split h` or `split v` is set first, and the next window takes the split.
- The new child gets the mean of its siblings' percentages, which are then renormalised, so it takes an equal share (V, `con_fix_percent`).

**tmux**
- `split-window -h` / `-v`, on the keys `%` and `"`.
- Half by default; `-l N` or `-l N%` sets the size. `-b` puts the new pane before. `-f` makes it span the full window (V).

**VS Code**
- **Commands:** Split Up/Down/Left/Right. The size follows `workbench.editor.splitSizing`; `auto`, the default, halves the active group, or equalises all groups when they are already equal (V).
- **Drag a tab onto a group:**
  - The edge regions are cut at 1/3 and 2/3 along the preferred direction.
  - A drop more than 10% from every edge falls in the centre and merges.
  - An overlay previews the half the result will occupy (V, editorDropTarget.ts).

**JetBrains**
- Split Right/Down from the tab menu, or drag a tab. The help page documents no size (V).

**iced `pane_grid`**
- `State::split(axis, pane)` gives a ratio of 0.5 (V).
- Dragging onto an edge region splits there; the regions are cut at thirds of the target.

**egui_tiles**
- Dropping onto a half of a tile splits it. The candidate whose preview centre is nearest the pointer wins (V).

**Dear ImGui**
- A five-way drop widget. The new node takes the dragged window's own size if that is at most half the space, otherwise half (V).

**Slide tools**
- PowerPoint and Keynote have guides, not split layouts.
- No documented split editor was found for Gamma, Pitch, Canva or Beautiful.ai.

**Tree normalisation**
- Three tools normalise as our merge rule does:
  - tmux inserts a sibling for a split along the parent's axis, and collapses a parent left with one child (V, `layout_split_pane`, `layout_destroy_cell`).
  - VS Code's `removeView` splices an only child branch into its grandparent (V).
  - egui_tiles has `join_nested_linear_containers: true` by default (V).
- i3 differs:
  - In a parent with two or more children, a split always wraps, whatever the axis (V, tree.c 365-379).
  - In a single-child parent, a split only flips the parent's orientation.
  - `tree_flatten` runs only after `move`.

**Comparison**
- **Pointer positioning, the owner's intuition:** its shipped precedent is Blender. Blender shows a preview line that follows the pointer before it commits, allows the axis to change and the gesture to be cancelled, and snaps with Ctrl.
- **Half-split commands, the design's buttons and keys:** every command-driven tool has them: tmux, i3, VS Code, JetBrains and iced.
- **Both in one tool:** none surveyed combines a pointer-positioned split and a half split.
- **What Ctrl+click lacks:** the design's Ctrl+click and Shift+click commit with no preview, no axis toggle and no snapping. Whether any shipped tool uses modifier+click with no preview was not searched (A).

## 2. Acting on a container rather than a leaf

- **i3 and sway** (V, i3 userguide):
  - `focus parent` (`$mod+a`) can be repeated up to the workspace.
  - `focus child` returns to the child last focused, and `focus next|prev sibling` steps between siblings.
  - Move, resize, layout and new windows then apply to the container.
- **Figma** (V, help.figma.com/hc/en-us/articles/360040449873):
  - A click inside a group selects the outer parent first.
  - A double-click or Enter selects a child, and Shift+Enter selects the parent.
  - Tab and Shift+Tab select the siblings, and Cmd/Ctrl+click is deep select.
  - Esc deselects. The article does not say that Esc selects the parent.
- **tmux:** no command targets a layout node. `select-layout -E` equalises the nearest ancestor where equalising changes something, and `split-window -f` targets the root (V).
- **Blender:** areas are rectangles over shared vertices, not a tree (V, DNA_screen_types.h).
- **Drop zones:**
  - egui_tiles gives every tile drop zones, containers included (V).
  - iced's `Target::Edge` and ImGui's outer docking split the root.
  - So a dropped tile can span its siblings without the parent being selected first.

**Comparison**
- **Alt+Up and the breadcrumb:** they are i3's `focus parent` with a visible path.
- **What the design lacks:** a way back down, as i3's `focus child` and Figma's Enter, and sideways, as Figma's Tab.
- **Pointer-only, the owner's intuition:** it cannot make a spanning split, since a click lands in a leaf. The tools surveyed reach a parent either by an explicit command (i3, Figma) or by drop zones on a container (egui_tiles, iced).

## 3. Dividers

**Grab width and hover feedback**

| tool | grab width | drawn line | hover feedback |
| --- | --- | --- | --- |
| VS Code | `workbench.sash.size` 4 px (1-20) | 1-8 px | highlight after `hoverDelay` 300 ms (V) |
| egui_tiles | 2 × `resize_grab_radius_side` = 6 px (V) | 1 px, `fg_stroke` on hover | |
| iced | `spacing + leeway`; the example uses 10 | style width | |
| Dear ImGui | 2 px separator + 4 px hover padding | | highlight after 0.04 s (V) |
| Blender | `border_width·scale + 3·scale` (V, `BORDERPADDING`) | | highlight fades in over 0.1 s |
| sway | `border_thickness` | | |
| tmux | the 1-cell border, ±1 cell at a junction | | |

- If the design's "within 6 px" means ±6 px, its band is 12 px wide, wider than every default above. If it means 6 px in total, it equals egui_tiles's.

**Cursor**
- Every tool shows a double-headed resize cursor.
- VS Code shows a one-way cursor at a limit, and `all-scroll` where two sashes cross. There the two sashes are linked and drag together (V, `trySet2x2`).
- tmux resizes both axes at a junction, and so does a sway corner drag (V).

**Snapping**
- Blender, moving a border with Ctrl held: it snaps to 1/24 of the window and to the vertices of other areas on the same line. The default mode soft-snaps to the minimum and maximum positions (V).
- VS Code: a view marked `snap` collapses when dragged half its minimum size past its minimum (V).
- No snapping to fractions was found in VS Code, egui_tiles, iced, i3 or tmux (A, from reading their resize code).

**A value shown while dragging**
- No split tool shows one (A).
- PowerPoint shows the distance of a guide from the slide's centre next to the pointer (V).

**Commit and cancel**
- i3 drags a 2 px preview bar and commits on release; any key press reverts it (V).
- sway, VS Code, egui_tiles and tmux resize live (V).
- Blender cancels on Esc or RMB (V).

**Other mechanisms**
- **Double-click:** egui_tiles sets the two neighbours to their mean (V). VS Code sets the preferred size, else equalises the branch (V).
- **A drag past the neighbour:** VS Code and egui_tiles let it shrink further views down to their minimum sizes (V).
- **VS Code Alt-drag:** keeps the neighbour's centre (V).
- **Minimum size:** Blender 29 px × scale, egui_tiles 32 (V).

**Comparison**
- **Overlay line and resize cursor:** the design matches every tool.
- **Snapping:** missing from the design. Blender is the precedent, for fractions and for other dividers.
- **Showing the fraction:** no split tool does; only PowerPoint does, for guides.
- **Left unspecified by the design:**
  - Esc to cancel a drag;
  - a minimum area size;
  - which divider a press grabs when two lie closer than the band's width.

## 4. Swapping or moving content between areas

- **Blender** (V):
  - Ctrl+drag from a corner swaps the contents of two areas. They need not be adjacent but must be in the same window, and the cursor shows a swap.
  - Shift+drag duplicates the area.
  - Docking: a drop in the middle of an area replaces it, and a drop at its edge splits it.
- **i3 `tiling_drag`** (4.21+, V):
  - **Start:** drag the title bar, or floating_modifier+LMB. The drag starts after 15 px.
  - **Centre zone:** a drop moves the container there. Holding `swap_modifier` swaps instead (Shift, since 4.24).
  - **Edge zone,** 30% of the target's smaller side: a drop inserts the container as a sibling.
  - **Outer zone:** a drop places it beside the target's parent.
  - **Commands:** `swap container with mark|id`, and `move <dir>`.
- **tmux:** `select-pane -m` marks a pane, then `swap-pane` or `join-pane` act on it (V). The mark replaces the drag.
- **iced:** `split_with` swaps on a Center drop and splits on an Edge drop. A drag starts only from the title-bar pick area, after 10 px (V).
- **egui_tiles:**
  - The app chooses the drag handle, by returning `DragStarted` from its pane UI (V).
  - A tile cannot be dropped onto its own descendants (V).
- **VS Code:** a centre drop merges, an edge drop splits, and Ctrl/Alt copies (V).

**Comparison**
- **Grip-drag onto an area:** the design's swap matches Blender's swap and the centre drop of i3 and iced.
- **Edge drop, missing from the design:** each of the drag tools above other than Blender's corner swap also treats an edge drop as move-and-split. That one gesture replaces the design's remove, split and re-edit.
- **Previewing the target:** missing from the design. Shipped tools highlight the region or change the cursor.
- **Unspecified:** a drop onto the dragged area's own subtree.

## 5. Undo grouping

**Explicit begin/end with a nesting counter**
- **Qt** (V, qundostack.cpp:752):
  - Macros nest.
  - While a macro is open, `canUndo` is false and `undo()` only warns.
  - An empty macro is kept as a step.
- **Cocoa** (V, undo() doc):
  - `undo()` closes a group open at level 1, then undoes it.
  - With more than one level open, it raises `NSInternalInconsistencyException`.
  - An unmatched `endUndoGrouping` also raises.
- **GTK4 `begin_user_action` / `end_user_action`** (V, gtktexthistory.c):
  - Pairs nest, and undo is refused while one is open.
  - An empty group is dropped.
  - A group with one child is unwrapped, so the child can merge with its neighbours.
- **Emacs change groups** (V):
  - `cancel-change-group` undoes everything since the group started.
  - Only proper nesting is supported.

**Implicit boundary at the end of an event or command**
- Cocoa `groupsByEvent` (default true) makes one group per run-loop pass (V).
- Emacs places a boundary before each key sequence (V).
- Vim makes one command one block, and `:undojoin` joins the next change (V).
- **Blender** (V, wm_event_system.cc 1279-1316; transform.cc):
  - It pushes one step per finished operator, and only at depth 0.
  - A drag is a modal operator. It pushes on confirm and nothing on cancel, and on Esc `restoreTransObjects` restores the data.
  - Area edits push no undo step (A, from their operator flags).

**Merge by key**
- **Qt `id()` / `mergeWith`** (V):
  - Merging works inside a macro.
  - It is blocked at the clean index, so a save breaks a run.
  - A command that is obsolete after a merge is deleted, which drops a no-op gesture.
  - The Undo Framework example merges consecutive moves of one item, so two separate drags merge.
- **Blender `OPTYPE_UNDO_GROUPED`:** drops the previous step when it has the same name (V).
- **CodeMirror:** a change joins only when all of these hold: it is an `input.type` or `delete` userEvent, it comes within 500 ms, and it is adjacent (V).

**Time-based grouping**
- **egui `Undoer`:** a point is made after 1.0 s of stability, or every 30 s while the state keeps changing (V). `undo()` mid-change drops the change in progress.
- **ProseMirror:** a new event after 500 ms or a non-adjacent change. `closeHistory` forces a boundary; there is no begin/end (V).
- **Failure modes,** derived from the rules, not measured:
  - a pause splits a gesture;
  - fast distinct edits merge;
  - the grouping depends on wall-clock time, so a replayed command script groups differently.

**Comparison with the design**
- **Its family:** begin/end is the family Qt, Cocoa, GTK and Emacs ship.
- **What those tools specify that the design does not:**
  - undo inside an open group: refuse it (Qt, GTK), or close the group and undo it at depth 1 (Cocoa);
  - nesting: a counter, with only the outermost end closing;
  - an empty group: dropped (GTK) or kept (Qt);
  - a cancelled gesture: undo the group (Emacs), or push nothing (Blender).
- **Time-based grouping:** it conflicts with deterministic replay of text commands.
- **Merge by key, as Qt's `id`:** consecutive `divider n4 0 x` commands merge with no begin. It needs a break rule; otherwise two separate drags of the same divider merge.
- **A third shape, preview then commit once on release:** i3's bar and Blender's modal operator. It needs no group. Its cost here would be losing the live re-layout of the slide during the drag (A).

## 6. Mechanisms the design misses or leaves unspecified

1. **A preview before a split commits,** with the axis switchable (Tab) and Esc to cancel (Blender).
2. **Snapping of divider and split positions,** with a modifier. Blender snaps to twelfths of the area when splitting, and to 1/24 of the window or to other areas' vertices when moving.
3. **Edge drop zones,** which make the swap drag a move-and-split. iced uses thirds, i3 30% of the smaller side, VS Code 1/3 and 2/3, and ImGui a five-way widget. Outer or container zones let a drop span siblings.
4. **Moving down and sideways in the tree:** i3's `focus child` (back to the child last focused) and `focus next|prev sibling`; Figma's Enter and Tab.
5. **Resetting and equalising:**
   - A double-click on a divider resets it: egui_tiles takes the mean of the two neighbours; VS Code takes the preferred size, else equalises the branch.
   - Equalise commands: tmux `select-layout -E`, VS Code's Reset Editor Group Sizes.
6. **Esc during a drag restores the starting state:** i3 on any key, Blender on Esc or RMB. In the design's terms, this means undoing the open group, or a gesture that never pushed anything.
7. **A minimum area size** that bounds both drags and splits. Blender refuses a split below 2 × the minimum.
8. **Linked dividers at a crossing,** dragged in both axes (VS Code 2x2, tmux, sway corners).
9. **The four rules of an explicit undo group:** undo while it is open, nesting, the empty group and the cancelled gesture. See section 5.
10. **A rule for a `begin` never ended,** if a pointer release is never delivered or the editor closes mid-gesture.
    - In Qt and GTK, an unmatched begin leaves undo disabled; in Cocoa, ending a group never begun raises (V).
    - That the GUI can miss the release: A.

## Decided

- I read the brief's "within 6 px" as a band of ±6 px, 12 px in total. Ground: the wording. Section 3 gives both readings.

## Met outside the task

- The harness refused the report files: the subagents' report-a.md, report-b.md and report-c.md, and my own report.md. Dropped, because I could not write them without going around the refusal. The full content is in this message, and the downloaded sources are kept in the scratchpad survey directory.
