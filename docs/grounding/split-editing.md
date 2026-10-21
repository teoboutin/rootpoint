# Prior art: editing a split layout, and grouping undo steps

A survey run on 2026-10-21 for the design session held at the audit of the slice that builds the
native GUI, in the milestone of the first prototype. It asked how shipped tools create a split, select a container, show and
drag a divider, swap or move content between areas, and make one gesture one undo step. The design
it was compared with, as it stood in round 1 of that session: click to select, a breadcrumb and
Alt+Up to select the parent; buttons and keys that split the selection at its middle, plus
Ctrl+click and Shift+click that split it at the pointer; overlay lines for dividers with a resize
cursor within 6 px; a grip dragged onto another area to swap; `begin` and `end` around a gesture
for undo. The owner's first intuition was the pointer split alone.

V means read in the cited source; A means assumed. Revisions read: Blender manual 5.2 and source
`main`; i3 be04c853 (4.25); sway 1652c54b (1.13-dev); tmux 3.4; VS Code fdb4b3c5; iced 0.14.0;
egui_tiles 0.18.0 on egui 0.37; Dear ImGui docking d88f238d; Qt 6.12 docs and qtbase `dev`;
prosemirror-history 1.5.0; @codemirror/commands 6.10.3.

Sources:

- Blender manual: docs.blender.org/manual/en/latest/interface/window_system/areas.html; source:
  github.com/blender/blender/blob/main/source/blender/editors/screen/screen_ops.cc
- i3 userguide: i3wm.org/docs/userguide.html; source github.com/i3/i3 at be04c853, src/tree.c,
  resize.c, tiling_drag.c
- sway: sway.5.scd at 1652c54b
- tmux: man.openbsd.org/tmux.1; github.com/tmux/tmux/blob/3.4/layout.c
- VS Code: github.com/microsoft/vscode at fdb4b3c5: src/vs/base/browser/ui/grid/gridview.ts,
  splitview/splitview.ts, sash/sash.ts, workbench/browser/parts/editor/editorDropTarget.ts,
  workbench.contribution.ts
- iced: github.com/iced-rs/iced/blob/0.14.0/widget/src/pane_grid.rs
- egui_tiles: docs.rs/crate/egui_tiles/0.18.0/source/src/
- Dear ImGui: github.com/ocornut/imgui at d88f238d, imgui.cpp
- Figma: help.figma.com/hc/en-us/articles/360040449873
- PowerPoint guides: support.microsoft.com/en-gb/office/guides-for-arranging-things-on-a-slide-33854dfa-e0d1-43ff-8971-667b19512de3
- Qt: doc.qt.io/qt-6/qundostack.html, qundocommand.html;
  raw.githubusercontent.com/qt/qtbase/dev/src/gui/util/qundostack.cpp
- Apple: developer.apple.com/documentation/foundation/undomanager/undo(), groupsbyevent
- GTK: gitlab.gnome.org/GNOME/gtk/-/raw/main/gtk/gtktexthistory.c
- Emacs: gnu.org/software/emacs/manual/html_node/elisp/Atomic-Changes.html
- ProseMirror: raw.githubusercontent.com/ProseMirror/prosemirror-history/master/src/history.ts
- CodeMirror: raw.githubusercontent.com/codemirror/commands/main/src/history.ts
- Blender undo: wm_event_system.cc lines 1279-1316 on `main`

The session re-read these claims in the downloaded sources: tmux layout.c:943, the same-axis
insert; Blender screen_ops.cc:3083-3092, the twelfths; iced pane_grid.rs:95-96, the dead band and
the edge strip; egui_tiles lib.rs:214, the joining of nested same-axis containers; egui 0.37
style.rs:1594, a grab radius of 3.0; Qt qundostack.cpp:752-753, undo refused inside a macro; Apple
`undo()`, "Closes the top-level undo group if necessary"; egui undoer.rs:29-31, its defaults.

## 1. Creating a split: direction and position

**Blender.**

- Corner drag (V, `actionzone_modal`): the direction is the dominant axis of the drag, and a split
  triggers after 1.2 widget units of movement. The position follows the pointer.
- Area Options > Vertical/Horizontal Split (V, manual and `area_split_modal`): opened by a
  right-click on a border, it shows an indicator line that follows the pointer. The position is
  `factor` = (pointer − area_min) / area_size, and the target area is re-read under the pointer on
  every move. Tab or the middle button toggles the axis; Esc or the right button cancels. Ctrl
  snaps to twelfths of the area (V, screen_ops.cc:3083), or to a vertex of another area.
- Minimum: a split is refused below 2 × AREAMINX, 29 px × UI scale (V).

**i3 and sway.** `split h` or `split v` is set first, and the next window takes the split. The new
child gets the mean of its siblings' percentages, which are then renormalised, so it takes an
equal share (V, `con_fix_percent`).

**tmux.** `split-window -h` / `-v`, on the keys `%` and `"`. Half by default; `-l N` or `-l N%` sets
the size, `-b` puts the new pane before, `-f` makes it span the full window (V).

**VS Code.**

- Commands Split Up/Down/Left/Right. The size follows `workbench.editor.splitSizing`; `auto`, the
  default, halves the active group, or equalises all groups when they are already equal (V).
- Dragging a tab onto a group: the edge regions are cut at 1/3 and 2/3 along the preferred
  direction; a drop more than 10% from every edge falls in the centre and merges. An overlay
  previews the half the result will occupy (V, editorDropTarget.ts).

**JetBrains.** Split Right/Down from the tab menu, or by dragging a tab. The help page documents no
size (V).

**iced `pane_grid`.** `State::split(axis, pane)` gives a ratio of 0.5 (V). Dragging onto an edge
region splits there; the regions are cut at thirds of the target.

**egui_tiles.** Dropping onto a half of a tile splits it. The candidate whose preview centre is
nearest the pointer wins (V).

**Dear ImGui.** A five-way drop widget. The new node takes the dragged window's own size if that is
at most half the space, otherwise half (V).

**Slide tools.** PowerPoint and Keynote have guides, not split layouts. No documented split editor
was found for Gamma, Pitch, Canva or Beautiful.ai.

**Tree normalisation.** Three tools normalise as rootpoint's merge rule does: tmux inserts a
sibling for a split along the parent's axis and collapses a parent left with one child (V,
`layout_split_pane`, `layout_destroy_cell`); VS Code's `removeView` splices an only child branch
into its grandparent (V); egui_tiles has `join_nested_linear_containers: true` by default (V). i3
differs: in a parent with two or more children a split always wraps, whatever the axis (V, tree.c
365-379); in a single-child parent a split only flips the parent's orientation; `tree_flatten`
runs only after `move`.

**Compared with the design.** The pointer split, the owner's intuition, has Blender as its shipped
precedent, which shows a preview line following the pointer before it commits, lets the axis
change and the gesture be cancelled, and snaps with Ctrl. The half split of the buttons and keys is
in every command-driven tool: tmux, i3, VS Code, JetBrains, iced. None surveyed combines a
pointer-positioned split and a half split. The design's Ctrl+click and Shift+click commit with no
preview, no axis toggle and no snapping; whether a shipped tool uses modifier+click with no preview
was not searched (A).

## 2. Acting on a container rather than a leaf

- **i3 and sway** (V, i3 userguide): `focus parent` (`$mod+a`) can be repeated up to the workspace;
  `focus child` returns to the child last focused; `focus next|prev sibling` steps between
  siblings. Move, resize, layout and new windows then apply to the container.
- **Figma** (V): a click inside a group selects the outer parent first; a double-click or Enter
  selects a child, Shift+Enter the parent; Tab and Shift+Tab select the siblings; Cmd/Ctrl+click is
  deep select. Esc deselects; the article does not say that Esc selects the parent.
- **tmux**: no command targets a layout node. `select-layout -E` equalises the nearest ancestor
  where equalising changes something, and `split-window -f` targets the root (V).
- **Blender**: areas are rectangles over shared vertices, not a tree (V, DNA_screen_types.h).
- **Drop zones**: egui_tiles gives every tile drop zones, containers included (V); iced's
  `Target::Edge` and ImGui's outer docking split the root. A dropped tile can then span its
  siblings without the parent being selected first.

**Compared with the design.** Alt+Up and the breadcrumb are i3's `focus parent` with a visible path.
The design lacks a way back down, as i3's `focus child` and Figma's Enter, and sideways, as Figma's
Tab. A pointer-only design cannot make a spanning split, since a click lands in a leaf; the tools
surveyed reach a parent by an explicit command (i3, Figma) or by drop zones on a container
(egui_tiles, iced).

## 3. Dividers

| tool | grab width | drawn line | hover feedback |
| --- | --- | --- | --- |
| VS Code | `workbench.sash.size` 4 px (1-20) | 1-8 px | highlight after `hoverDelay` 300 ms (V) |
| egui_tiles | 2 × `resize_grab_radius_side` = 6 px (V) | 1 px, `fg_stroke` on hover | |
| iced | `spacing + leeway`; the example uses 10 | style width | |
| Dear ImGui | 2 px separator + 4 px hover padding | | highlight after 0.04 s (V) |
| Blender | `border_width·scale + 3·scale` (V, `BORDERPADDING`) | | highlight fades in over 0.1 s |
| sway | `border_thickness` | | |
| tmux | the 1-cell border, ±1 cell at a junction | | |

- A band of ±6 px is 12 px wide, wider than every default above; 6 px in total equals
  egui_tiles's.
- Cursor: every tool shows a double-headed resize cursor. VS Code shows a one-way cursor at a
  limit, and `all-scroll` where two sashes cross, where the two are linked and drag together (V,
  `trySet2x2`). tmux resizes both axes at a junction, and so does a sway corner drag (V).
- Snapping: Blender, moving a border with Ctrl held, snaps to 1/24 of the window and to the
  vertices of other areas on the same line; its default mode soft-snaps to the minimum and maximum
  positions (V). VS Code collapses a view marked `snap` when dragged half its minimum size past its
  minimum (V). No snapping to fractions was found in VS Code, egui_tiles, iced, i3 or tmux (A, from
  reading their resize code).
- A value shown while dragging: no split tool shows one (A). PowerPoint shows the distance of a
  guide from the slide's centre next to the pointer (V).
- Commit and cancel: i3 drags a 2 px preview bar and commits on release, and any key press reverts
  it (V). sway, VS Code, egui_tiles and tmux resize live (V). Blender cancels on Esc or the right
  button (V).
- Double-click: egui_tiles sets the two neighbours to their mean (V); VS Code sets the preferred
  size, else equalises the branch (V).
- A drag past the neighbour: VS Code and egui_tiles let it shrink further views down to their
  minimum sizes (V). VS Code Alt-drag keeps the neighbour's centre (V).
- Minimum size: Blender 29 px × scale, egui_tiles 32 (V).

**Compared with the design.** The overlay line and the resize cursor match every tool. Snapping is
missing from the design, with Blender as the precedent, for fractions and for other dividers. No
split tool shows the fraction; only PowerPoint does, for guides. The design leaves unspecified Esc
to cancel a drag, a minimum area size, and which divider a press grabs when two lie closer than
the band's width.

## 4. Swapping or moving content between areas

- **Blender** (V): Ctrl+drag from a corner swaps the contents of two areas, which need not be
  adjacent but must be in the same window; the cursor shows a swap. Shift+drag duplicates the area.
  Docking: a drop in the middle of an area replaces it, and a drop at its edge splits it.
- **i3 `tiling_drag`** (4.21+, V): drag the title bar, or floating_modifier+LMB; the drag starts
  after 15 px. A drop in the centre zone moves the container there, and with `swap_modifier` held
  (Shift, since 4.24) swaps instead. A drop in the edge zone, 30% of the target's smaller side,
  inserts it as a sibling; in the outer zone, beside the target's parent. Commands: `swap container
  with mark|id`, and `move <dir>`.
- **tmux**: `select-pane -m` marks a pane, then `swap-pane` or `join-pane` act on it (V). The mark
  replaces the drag.
- **iced**: `split_with` swaps on a Center drop and splits on an Edge drop. A drag starts only from
  the title-bar pick area, after 10 px (V).
- **egui_tiles**: the app chooses the drag handle, by returning `DragStarted` from its pane UI (V).
  A tile cannot be dropped onto its own descendants (V).
- **VS Code**: a centre drop merges, an edge drop splits, and Ctrl/Alt copies (V).

**Compared with the design.** The grip dragged onto an area matches Blender's swap and the centre
drop of i3 and iced. Every drag tool above other than Blender's corner swap also treats an edge
drop as move-and-split, one gesture that replaces a remove, a split and a re-edit; the design lacks
it. Shipped tools also preview the target, by a highlighted region or a cursor. A drop onto the
dragged area's own subtree is unspecified.

## 5. Undo grouping

**Explicit begin and end, with a nesting counter.**

- Qt (V, qundostack.cpp:752): macros nest; while one is open, `canUndo` is false and `undo()` only
  warns; an empty macro is kept as a step.
- Cocoa (V, `undo()` doc): `undo()` closes a group open at level 1, then undoes it; with more than
  one level open it raises `NSInternalInconsistencyException`; an unmatched `endUndoGrouping` also
  raises.
- GTK4 `begin_user_action` / `end_user_action` (V, gtktexthistory.c): pairs nest, and undo is
  refused while one is open; an empty group is dropped; a group with one child is unwrapped, so the
  child can merge with its neighbours.
- Emacs change groups (V): `cancel-change-group` undoes everything since the group started; only
  proper nesting is supported.

**Implicit boundary at the end of an event or a command.** Cocoa `groupsByEvent` (default true)
makes one group per run-loop pass (V). Emacs places a boundary before each key sequence (V). Vim
makes one command one block, and `:undojoin` joins the next change (V). Blender pushes one step per
finished operator, only at depth 0; a drag is a modal operator that pushes on confirm and nothing
on cancel, and on Esc `restoreTransObjects` restores the data (V, wm_event_system.cc 1279-1316;
transform.cc). Area edits push no undo step (A, from their operator flags).

**Merge by key.** Qt `id()` / `mergeWith` (V): merging works inside a macro; it is blocked at the
clean index, so a save breaks a run; a command obsolete after a merge is deleted, which drops a
no-op gesture; the Undo Framework example merges consecutive moves of one item, so two separate
drags merge. Blender `OPTYPE_UNDO_GROUPED` drops the previous step when it has the same name (V).
CodeMirror joins a change only when it is an `input.type` or `delete` user event, within 500 ms,
and adjacent (V).

**Time-based grouping.** egui `Undoer` makes a point after 1.0 s of stability, or every 30 s while
the state keeps changing (V); `undo()` mid-change drops the change in progress. ProseMirror starts
a new event after 500 ms or a non-adjacent change; `closeHistory` forces a boundary, and there is
no begin/end (V). Failure modes, derived from the rules and not measured: a pause splits a
gesture; fast distinct edits merge; the grouping depends on wall-clock time, so a replayed script
of commands groups differently.

**Compared with the design.** Begin and end is the family Qt, Cocoa, GTK and Emacs ship. Those tools
specify what the design did not: undo inside an open group, refused (Qt, GTK) or closing the group
and undoing it at depth 1 (Cocoa); nesting, by a counter with only the outermost end closing; an
empty group, dropped (GTK) or kept (Qt); a cancelled gesture, undoing the group (Emacs) or pushing
nothing (Blender). Time-based grouping conflicts with the deterministic replay of text commands.
Merge by key, as Qt's `id`, would merge consecutive `divider n4 0 x` commands with no begin, and
needs a break rule, since otherwise two separate drags of one divider merge. A third shape,
preview then one commit on release, is i3's bar and Blender's modal operator; it needs no group,
and would lose the live re-layout of the slide during the drag (A).

## 6. Mechanisms the design misses or leaves unspecified

1. A preview before a split commits, with the axis switchable (Tab) and Esc to cancel (Blender).
2. Snapping of divider and split positions, with a modifier: Blender snaps to twelfths of the area
   when splitting, and to 1/24 of the window or to other areas' vertices when moving.
3. Edge drop zones, which make the swap drag a move-and-split: iced thirds, i3 30% of the smaller
   side, VS Code 1/3 and 2/3, ImGui a five-way widget. Outer or container zones let a drop span
   siblings.
4. Moving down and sideways in the tree: i3's `focus child` and `focus next|prev sibling`; Figma's
   Enter and Tab.
5. Resetting and equalising: a double-click on a divider resets it (egui_tiles, the mean of the two
   neighbours; VS Code, the preferred size, else equalise the branch); equalise commands (tmux
   `select-layout -E`, VS Code's Reset Editor Group Sizes).
6. Esc during a drag restores the starting state: i3 on any key, Blender on Esc or the right
   button.
7. A minimum area size that bounds both drags and splits; Blender refuses a split below twice the
   minimum.
8. Linked dividers at a crossing, dragged in both axes: VS Code 2x2, tmux, sway corners.
9. The four rules of an explicit undo group: undo while it is open, nesting, the empty group, the
   cancelled gesture (section 5).
10. A rule for a `begin` never ended, if a pointer release is never delivered or the editor closes
    mid-gesture. In Qt and GTK an unmatched begin leaves undo disabled; in Cocoa, ending a group
    never begun raises (V). That the GUI can miss a release is assumed.
