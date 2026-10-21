# Slice: The native GUI and presenter mode

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites, and AC6; the threads, the arguments and the two criteria of the design session held at this
slice's audit are below, since only this slice cites them.

## Builds

- The crate `rootpoint-app` at `path@rootpoint@crates/rootpoint-app/`, native only in this
  slice: a library root holding the app's state, its action handlers and its frame code, and the
  binary `rootpoint`. It offers every UI action that "The UI actions" in "The prototype's scope" of
  the milestone document lists, in the interaction design "Decided design" below gives, the text of
  an area being edited in the side panel's plain text field, and presenter mode, full screen. Its
  library root and its binary target read `ROOTPOINT_CHECKOUT`, as `path@xtask@CLAUDE.md` requires.
  It depends on eframe 0.37.0 without its `system_fonts` feature, and on `rfd` 0.17.2.
- The crate is not added to the `wasm` gate here: its web target is built by
  `spec@first-prototype@browser-build`, as "How the work is done" says. The comment on
  `WASM_CRATES` in `path@xtask@src/main.rs`, which says a crate the browser build links is listed
  when it is created, is reworded in the commit that creates the crate.
- Per D2, the crate's Component, with its goals drafted for the owner's ruling.
- In the model crate:
  - in `path@rootpoint-model@src/geometry.rs`, `node_rects`: every node of a slide, splits
    included, with its rectangle in slide units, in tree order. The app finds a split and its
    dividers from it and `Split::child_extent`, since the model owns the area geometry ("Slide
    coordinates" in the milestone document);
  - in `path@rootpoint-model@src/ops.rs`, `Op::SetAssetSource { id, source }`, which replaces an
    asset's source and returns the operation that restores the old one, per "Saving assets" below.
- In the render crate, `slide_to_pixels`, the public form of the private `slide_transform` of
  `path@rootpoint-render@src/lib.rs`: the scale and the offset that carry slide units to the pixels
  of a `width`×`height` image. The app's hit testing and overlay use it, so they place every mark
  where `render_slide` draws.
- In the protocol crate:
  - the commands `begin`, `end` and `cancel`, per "Undo groups", with their grammar, their printing,
    their replies and explicit arms in `Session::run`;
  - the command `sample`, which opens the built-in sample, per "The built-in sample";
  - the view's shown slide, per "Selection";
  - in `save`, the asset rule of "Saving assets", which closes
    `asset-paths-follow-save`;
  - in `path@rootpoint-protocol@src/files.rs`, `Files::exists`, so a save tells a missing file from
    one it cannot read;
  - in `path@rootpoint-protocol@src/session.rs`, `Session::asset_bytes` made public, so the app
    decodes the images and fonts it hands `render_slide` from the bytes the session reads and the
    rule that reads an asset's path stays in one place, and `Session::directory`, the directory an
    asset's path is read against;
  - the texts that state the asset rule as it stands, updated: the protocol's README, the doc
    comment of `path@rootpoint-protocol@src/bin/rootpoint-headless.rs`, and the test
    `assets_are_read_beside_the_presentation` of `path@rootpoint-protocol@tests/files.rs`, which the
    issue says changes with the fix;
  - each new verb added to the hand-written lists of `path@rootpoint-protocol@tests/commands.rs`:
    the commands printed back, and the refused commands.

## Claims

1. **Every UI action goes through the session.** Each UI action issues the protocol `Command`s that
   "UI actions to commands" below maps it to, so the document and the view after a UI action equal
   those after the same commands. Test: each of the app's action handlers is called on a session
   in a unit test, with no window, and the same commands are run on a second session; their
   documents and views must be equal. `Session::document` gives only a shared reference, so a
   handler cannot edit the document but through a command. Shown to fail: a handler that issues
   another command than its action's, such as a split along the other axis.
2. **Hit testing matches the drawing.** Through `slide_to_pixels`, at two view sizes: a click at a
   point selects the area whose rectangle contains it; a press within 4 points of a divider grabs
   it, the nearer of two; "Pointer input to gestures" below decides every overlap. At step 4,
   also: a pointer over a drop target falls in the zone the hint names, and a snapped position lies
   on a twelfth of the node or on another divider within 8 points. Test: unit tests over the
   geometry, with no window. Shown to fail: a grab band computed in slide units instead of points,
   which the test at two view sizes catches.
3. **Presenter mode shows the slide the editor shows.** At the same pixel size, the presenter's
   image equals the image the editor view hands to its texture for the same slide, so no editor
   mark is in the slide image. Test: both images, taken from the app's code with no window and
   compared pixel by pixel. Shown to fail: an outline of the selection drawn into the image.
4. **One gesture is one undo step.** Every edit between `begin` and `end` is undone by one `undo`
   and redone by one `redo`; `cancel` restores the document and both stacks as they were at the
   `begin`; the other rules of "Undo groups" hold for `undo` inside a group, a nested `begin`, an
   empty group, and `new`, `load` or `sample` inside a group. Test: protocol scripts. Shown to fail:
   a session that pushes one step per edit inside a group, caught by an `undo` that leaves the
   drag's middle position.
5. **A save leaves every asset readable beside the saved file, and overwrites nothing else.**
   After a `save` into another directory, every asset renders with `missing=-`; an embedded asset
   is written out as a file and stored as a path; an existing file of other bytes keeps its bytes,
   and the copy takes a suffix; an asset the save cannot read makes it reply `err`, naming it, and
   write nothing. Test: protocol tests over `MemoryFiles`, and the reproduction of
   `asset-paths-follow-save`. Shown to fail: the session as it stands,
   whose render after the save reports `missing=a3`.

## Audit subjects

The audit is held; its findings are in the commit "Apply the design audit of the fourth slice, the
native GUI", and its outcomes are in this spec. What it read:

- `thread@first-prototype@gui-toolkit`, `thread@first-prototype@slide-renderer`,
  `thread@first-prototype@slide-coordinate-space`.
- The sources of eframe and egui 0.37.0: textures, input, full screen. Three traps for the steps:
  with click and drag both sensed, egui reports a drag only once the pointer has moved, so the press
  position is read from `InputState::pointer.press_origin()`, not from the `drag_started` frame;
  `consume_shortcut` ignores extra Shift, so Ctrl+Shift+Z is tested before Ctrl+Z; `eframe::App`
  implements `ui`, and `Panel::show` replaces the deprecated `show_inside`.
- The native libraries: a release build of a probe with eframe's default features and the render
  crate linked only libc, libm and libgcc_s, and loads X11, Wayland, Vulkan and GL at run time; all
  are present on the owner's machine. Assumed, not measured: the debug builds of the `clippy` and
  `test` gates need the same libraries. Assumed, not seen: the full-screen command takes effect on
  the owner's screen; step 5 sees it.
- The duplicate `vello_cpu` versions, 0.2 through epaint and 0.3 through the render crate: about 6
  to 11 s of a clean release build and 0.8 MB of binary, which the audit judges no cost worth acting
  on. eframe's `system_fonts` feature also duplicates fontique, 0.11 beside parley's 0.12; the app
  turns it off, since the editor's own text needs only egui's built-in fonts and the browser has no
  system fonts.

## What is already decided

The decisions the design session's searches graded as bearing on this slice, and how each bears:

- `design@rootpoint-protocol@view-state`: stands, and gains a member, the slide the editor shows,
  which its argument covers, "Selection". Undo groups change the undo stack, which the head does not
  describe, and nothing of the view.
- `design@rootpoint-model@edit-operation-core`: stands, and gains a member, `Op::SetAssetSource`,
  which its argument covers; every change of a save goes through `apply`.
- `design@rootpoint-protocol@protocol-transport`: stands; the new verbs print as lines that parse
  back to them.
- `design@rootpoint-protocol@protocol-addressing`: stands; every command names slides, nodes and
  assets by id, and `move-slide` names its slide by id and its destination by index.
- `design@rootpoint-protocol@session-files`: stands; every read and write of a save goes through
  `Files`, which gains `exists`.
- `design@rootpoint-model@image-storage` and `design@rootpoint-model@asset-source`: stand; every
  `Path` the session holds is relative to the presentation's directory, the GUI passing a path
  relative to `Session::directory`, and a saved file holds no embedded bytes.
- `design@rootpoint-render@caller-supplies-assets` and `design@rootpoint-render@font-sources`: stand;
  the app fills `ImageSet` and `FontSet` from `Session::asset_bytes`, and a font asset follows the
  save rule as an image does.
- `design@rootpoint-render@slide-renderer`, `goal@rootpoint-render@same-slide-everywhere` and
  `rejected@rootpoint-render@toolkit-painter`: stand; the editor's marks are drawn over the texture,
  and the drop hint's label is editor text, not slide content.
- `design@rootpoint-model@divider-fractions` and `design@rootpoint-model@validated-documents`: stand;
  a drag sends fractions, and stops before a divider would leave (0, 1) or meet its neighbour.
- `design@rootpoint-model@inline-content-assets`, `goal@rootpoint-model@headless-changes`,
  `goal@rootpoint-model@faithful-readable-file`, `goal@rootpoint-protocol@commands-drive-everything`
  and `goal@rootpoint@headless-core`: met, by "UI actions to commands" and by the save rule.
- `rejected@rootpoint-model@gui-mutation`: not brought back; a drag is session commands.
- `asset-paths-follow-save`: closed by this slice. `issue@rootpoint-model@linked-dividers`:
  its trigger is not met; snapping eases the need it defers. `tripwire@rootpoint-model@grouping-lost-to-merge`:
  does not fire; no gesture moves a group of same-axis areas as one unit.

## The design session at the audit

The audit stopped the slice on four questions: the interaction design; what one undo step is for a
drag or for typing, since each `divider` and `text` command pushes one step; what a save into
another directory does to asset paths, `asset-paths-follow-save`; and
what a save of the built-in sample writes, since its images are embedded and
`design@rootpoint-model@asset-source` names packing as what writes embedded bytes. The session
that held it begins at the owner's message "Start working on the next slice" that opens this
slice. The session read the goals, design and rejected homes of rootpoint and of the model and
protocol crates, and the render crate's entries its search graded; a prior-art survey ran on the
owner's word, kept as `path@rootpoint@docs/grounding/split-editing.md`. It converged on the owner's
word in O16.

**The owner's messages in the session**, verbatim, in order:

**O13**

> Start working on the next slice

**O14**, answering round 1:

> Q15: shape B is the shape A controls plus an addition, right ? If yes, shape B is fine. The shape A controls look easy/intuitive to use, while the shape B additions would allow faster actions once keybinds are mastered. My initial idea (in my head) had just the shape B additions, but with shape A added the design is better IMO.
> Q16: (c) is best.
> Q17: beside the presentation file for now. But ultimately (later than initial prototype), I'd like to allow the user to organize the assets in any way they want with subdirectories of any level, and for a "pack then unpack elsewhere" action to preserve the tree.
>
> Q18: can you run it anyway, so we can compare with what my intuition led me to ? At the end, I want a permanent survey report stored under docs/grounding/ from repo root. Maybe the survey done at the setup of the project could have its report saved there too. Those are useful material for the long term.

The session answered the question in Q15: shape B is shape A's controls plus the pointer gesture.

**O15**, answering round 2:

> Q19: preview + snapping is good
> Q20: yes
> Q21: swap + edge moves loks fine, as long as there is a preview/hint of what dropping will do (a preview of the exact result might not be meaningful in this case, so some sort of hint has to do).

**O16**, answering the checkpoint and the premortem, which listed every thread with its state, put
AC6 and T5, and said that a positive word given against the table also confirms its
presumed-settled threads, `thread@first-prototype@node-selection`,
`thread@first-prototype@span-selection`, `thread@first-prototype@editor-overlay`,
`thread@first-prototype@divider-drag` and `thread@first-prototype@slide-reorder`, and the overwrite
clause of `thread@first-prototype@asset-save-rule`:

> Keep both AC6 and T5,proceed

**O17**, answering Q22, the material finding of `argument@first-prototype@arg-80`, put after the
spec's review:

> Q22 default accepted

**O18**, answering Q23, the goals of the app crate's Component drafted per D2 of the milestone
document, all five written in `register@rootpoint-app@goal`:

> Q23: all goals approved, proceed with step 3

**O19**, the owner's report of AC6, judged at step 5 on the owner's machine and desktop, after the
session listed Ctrl+click and Shift+click with their lines, Alt snapping during a split and a
divider drag, and Alt with each arrow:

> AC6: all of these controls are working just fine.

## Criteria

The session judged its threads against the milestone's criteria
`criterion@first-prototype@c6-fast-division`, `criterion@first-prototype@c1-headless-editing`,
`criterion@first-prototype@c8-same-slide-everywhere`,
`criterion@first-prototype@c3-readable-roundtrip` and `criterion@first-prototype@c4-browser-parity`,
against AC2's drag bound, and against these two:

### No edit is implemented a second time outside the model's operations `##c11-one-implementation`

Binding, derived from `design@rootpoint-model@edit-operation-core` and
`rejected@rootpoint-model@gui-mutation`, as a presumption. Met by
`thread@first-prototype@gesture-undo`, whose drag frames are session commands, and by
`thread@first-prototype@swap-gesture`, whose edge move is existing commands.

### One undo reverts one gesture: a whole drag, or one stay of the text field's focus `##c12-one-gesture-undo`

Weighed. Source: the audit's question on the undo history. Met by
`thread@first-prototype@gesture-undo`, with `cancel`.

## Threads

### A node is selected by a click, and its ancestors, children and siblings by keys and a breadcrumb `##node-selection`

Proposed by the session in round 1, with an addition in round 2 from the survey
(`argument@first-prototype@arg-55`). Final state: approved, by the owner's batch confirmation of the
checkpoint in O16.
Shape: "Selection" in "Decided design". Its rival, a click on a divider selecting its split, lost
(`argument@first-prototype@arg-54`). Watched by AC6 for its keys. Harvest: judged by the entry
tests, into the app crate's design home.

### A subtree is selected for a spanning split by parent navigation `##span-selection`

Proposed by the session in round 1. Final state: approved, by the owner's batch confirmation of the
checkpoint in O16. Shape: "Selection" in "Decided design". Its rival, a rectangle drawn across areas, lost
(`argument@first-prototype@arg-56`). Harvest: judged by the entry tests, into the app crate's design
home.

### A split is started from the selection at its middle by buttons and keys, or at the pointer with a modifier `##selection-actions`

Proposed by the session in round 1 as shape A, the middle split, and shape B, shape A plus a split
at the pointer (`argument@first-prototype@arg-57`). Q15 asked: shape A alone, or shape B, the
session's default. Closed by the owner in O14: "Q15: shape B is the
shape A controls plus an addition, right ? If yes, shape B is fine. The shape A controls look
easy/intuitive to use, while the shape B additions would allow faster actions once keybinds are
mastered. My initial idea (in my head) had just the shape B additions, but with shape A added the
design is better IMO." (`argument@first-prototype@arg-58`). Final state: approved. Refined by
`thread@first-prototype@split-preview`. Shape: "Actions on the selection" in "Decided design".
Harvest: judged by the entry tests, into the app crate's design home, the choice attributed to the
owner.

### A split at the pointer is previewed before it commits, and snaps with Alt `##split-preview`

Proposed by the session in round 2, from the survey (`argument@first-prototype@arg-59`,
`argument@first-prototype@arg-60`). Q19 asked: the preview and the snapping, or the preview only.
Closed by the owner in O15: "Q19: preview + snapping is good".
Final state: approved. Watched by AC6 for its keys. Shape: "Actions on the selection" and
"Dividers" in "Decided design". Harvest: judged by the entry tests, into the app crate's design
home.

### The editor's marks are drawn over the slide's texture, never into the slide's image `##editor-overlay`

Proposed by the session in round 1 (`argument@first-prototype@arg-61`). Final state: approved, by
the owner's batch confirmation of the checkpoint in O16. Its band of 6 pixels is replaced by
`thread@first-prototype@divider-drag`'s. Shape: "Dividers" in "Decided design". Harvest: judged by
the entry tests, into the app crate's design home, citing
`rejected@rootpoint-render@toolkit-painter` if it earns a head.

### A divider is grabbed within 4 pixels, keeps its areas at 40 slide units or more, and resets on a double-click `##divider-drag`

Proposed by the session in round 2, from the survey (`argument@first-prototype@arg-62`). Final
state: approved, by the owner's batch confirmation of the checkpoint in O16. Shape: "Dividers" in "Decided design".
Harvest: judged by the entry tests, into the app crate's design home.

### One gesture is one undo step, grouped by the commands begin, end and cancel `##gesture-undo`

Proposed by the session in round 1 among three shapes (`argument@first-prototype@arg-63`,
`argument@first-prototype@arg-64`, `argument@first-prototype@arg-65`), its rules completed and
`cancel` proposed in round 2 (`argument@first-prototype@arg-66`,
`argument@first-prototype@arg-67`). Q16 asked: (a) a preview on a copy, (b) the session merging
consecutive commands, or (c) explicit groups, the session's default; Q20 asked: add `cancel`, or
not. Closed by the owner in O14, "Q16: (c) is best.", and in O15, "Q20: yes". The completing rules were stated as completing the approved rule and confirmed with the
checkpoint by O16. Final state: approved. Shape: "Undo groups" in "Decided design". Watched by T5 of
the milestone's premortem. Harvest: a design head of the protocol crate's Component, since the
commands are the crate's interface, the choices of Q16 and Q20 attributed to the owner; the losing
shapes judged for a `rejected` entry; T5 into that
Component's tripwires home, guarding the head.

### A save copies every asset outside the presentation's directory, and every embedded asset, beside the saved file `##asset-save-rule`

Proposed by the session in round 1 (`argument@first-prototype@arg-68`,
`argument@first-prototype@arg-69`, `argument@first-prototype@arg-70`). Q17 asked: the copied files
beside the presentation file, the session's default, or in a subfolder. Closed by the owner in O14:
"Q17: beside the presentation file for now. But ultimately (later than initial prototype), I'd like
to allow the user to organize the assets in any way they want with subdirectories of any level, and
for a "pack then unpack elsewhere" action to preserve the tree." (`argument@first-prototype@arg-71`).
The overwrite clause came from the premortem (`argument@first-prototype@arg-72`) and was confirmed
by O16. The change in memory, `Op::SetAssetSource` with no undo step, came back after the spec's
review as a material finding (`argument@first-prototype@arg-80`), and the owner accepted its default
in O17: "Q22 default accepted". Final state: approved. Absorbs `thread@first-prototype@sample-save`. Shape: "Saving assets"
in "Decided design". Harvest: a design head of the protocol crate's Component, citing
`design@rootpoint-model@image-storage`, the choice of Q17 attributed to the owner with its "for
now"; the losing shapes judged for a `rejected` entry; the closure
of `asset-paths-follow-save`.

### Saving the built-in sample writes its images as files `##sample-save`

Proposed by the session in round 1 (`argument@first-prototype@arg-73`). Final state: superseded by
`thread@first-prototype@asset-save-rule`, which carries its shape whole. Harvest: none of its own.

### Two areas are swapped, or one is moved to another's edge, by dragging its grip, with a hint of the drop's effect `##swap-gesture`

Proposed by the session in round 1 as a grip drag that swaps (`argument@first-prototype@arg-74`),
with edge moves added in round 2 from the survey (`argument@first-prototype@arg-75`). Q21 asked: swap
only, or swap plus edge moves. Closed by the owner in O15: "Q21: swap + edge moves loks fine, as long as there is a preview/hint of what dropping
will do (a preview of the exact result might not be meaningful in this case, so some sort of hint
has to do)." The hint's shape was put at the checkpoint (`argument@first-prototype@arg-76`) and
confirmed by O16. Final state: approved. Shape: "Moving content" in "Decided design". Harvest:
judged by the entry tests, into the app crate's design home.

### Slides are reordered by dragging them in the slide list `##slide-reorder`

Proposed by the session in round 1 (`argument@first-prototype@arg-77`). Final state: approved, by
the owner's batch confirmation of the checkpoint in O16. Shape: "The slide list" in "Decided design".
Harvest: judged by the entry tests, into the app crate's design home.

## Arguments

### Selection runs through the session's view commands `##arg-53`

Round 1. Session. "The whole selection lives in the session's `View`, as `view-state` requires."
Bears on `thread@first-prototype@node-selection`, `thread@first-prototype@span-selection`.

### A click on a divider is taken by dragging it `##arg-54`

Round 1. Session, against selecting a split by a click on its divider: "That loses to divider
dragging, which uses the same 6 px band around the divider." Bears on
`thread@first-prototype@node-selection`.

### A pointer alone reaches no parent; shipped tools go up by a command, and down and sideways too `##arg-55`

Round 2, from the survey. "Pointer-only, the owner's intuition: it cannot make a spanning split,
since a click lands in a leaf"; "Alt+Up and the breadcrumb: they are i3's `focus parent` with a
visible path"; it lacks "a way back down, as i3's `focus child` and Figma's Enter, and sideways, as
Figma's Tab." Bears on `thread@first-prototype@node-selection`,
`thread@first-prototype@span-selection`.

### A rectangle selection is a second path whose result can exceed the rectangle `##arg-56`

Round 1. Session, against selecting a subtree by a rectangle: "it costs a second selection path";
"Its result surprises when the smallest containing node is larger than the dragged rectangle";
"Two of three children are no node of their own. Building one is the grouping that
`grouping-lost-to-merge` watches." Bears on `thread@first-prototype@span-selection`.

### A split at the pointer saves one drag per split `##arg-57`

Round 1. Session. "shape B saves one drag per split, which counts for AC4"; "It costs one more
gesture to build and test. Both shapes meet every criterion"; "`split` takes `at` as a fraction of
the node, so the pointer's position maps straight onto it." From the survey, round 2: "Half-split
commands, the design's buttons and keys: every command-driven tool has them: tmux, i3, VS Code,
JetBrains and iced"; "none surveyed combines a pointer-positioned split and a half split." Bears on
`thread@first-prototype@selection-actions`.

### The buttons are easy to learn, and the pointer gesture is faster once learnt `##arg-58`

Round 1, the owner's answer, O14. "The shape A controls look easy/intuitive to use, while the shape
B additions would allow faster actions once keybinds are mastered. My initial idea (in my head) had
just the shape B additions, but with shape A added the design is better IMO." Bears on
`thread@first-prototype@selection-actions`.

### Blender previews a pointer split, and snaps it `##arg-59`

Round 2, from the survey. "Pointer positioning, the owner's intuition: its shipped precedent is
Blender. Blender shows a preview line that follows the pointer before it commits, allows the axis to
change and the gesture to be cancelled, and snaps with Ctrl"; "the design's Ctrl+click and
Shift+click commit with no preview, no axis toggle and no snapping"; "Ctrl snaps to twelfths of the
area (V, screen_ops.cc:3083)". Bears on `thread@first-prototype@split-preview`.

### Snapping to other dividers aligns rows across columns without linking `##arg-60`

Round 2. Session. "The preview makes the split visible before it happens, which Ctrl+click alone
did not"; "Snapping to other dividers lines up rows across columns by hand. That is the need
`issue@rootpoint-model@linked-dividers` defers, met without linking." Convergence: "It survives:
snapping acts only while Alt is held." Bears on `thread@first-prototype@split-preview`,
`thread@first-prototype@divider-drag`.

### The editor's marks are not slide content `##arg-61`

Round 1. Session. "Claim 3 requires the presenter's image to equal the editor's, so the marks cannot
be part of the slide"; "`toolkit-painter` rejected the toolkit laying out slide text. These marks
are lines and rectangles, with no text." From the render crate's design-entry search: "dividers
are UI chrome, not slide content." Bears on `thread@first-prototype@editor-overlay`.

### Shipped dividers grab within 6 to 10 pixels, reset on a double-click and keep a minimum size `##arg-62`

Round 2, from the survey and the session. The survey: "If the design's "within 6 px" means ±6 px,
its band is 12 px wide, wider than every default above"; "egui_tiles | 2 × `resize_grab_radius_side`
= 6 px"; "iced ... the example uses 10"; left unspecified: "Esc to cancel a drag; a minimum area
size; which divider a press grabs when two lie closer than the band's width"; "Double-click:
egui_tiles sets the two neighbours to their mean"; minimum "Blender 29 px × scale, egui_tiles 32".
The session: "±4 px, 8 px in total, between egui_tiles's 6 and iced's 10"; "These are defaults no
fact decides between." Bears on `thread@first-prototype@divider-drag`.

### A drag sending one command per frame leaves sixty undo steps a second `##arg-63`

Round 1. Session. "A drag that sends one command per frame would leave about 60 steps per second."
From the model crate's design-entry search: "Each `apply` returns one inverse, so one undo step is
one operation today." From the root's: "one undo step" for a drag or for typed text "has to be
definable without the GUI, not only as a GUI-side grouping." Bears on
`thread@first-prototype@gesture-undo`.

### A preview on a copy leaves the session behind the screen; merging by the session is invisible in a script `##arg-64`

Round 1. Session. Against (a), a preview on a copy: "During the drag, the session's document and
the image on screen differ. An agent watching live, in the `live-control` slice, sees the old
divider until release." For it: "The protocol does not change." Against (b), the session merging:
"Two `divider` lines in a script, then `undo`, revert both lines. A script reader cannot see that
from the script." From the survey: merge by key "needs a break rule; otherwise two separate drags of
the same divider merge"; "Time-based grouping: it conflicts with deterministic replay of text
commands"; preview then commit "Its cost here would be losing the live re-layout of the slide
during the drag (A)". Bears on `thread@first-prototype@gesture-undo`.

### Explicit groups keep the session the only state and the grouping visible `##arg-65`

Round 1. Session, for (c): "it keeps the session the only state; it keeps the grouping visible in a
script; it covers dragging and typing with one rule." Cost: "It adds two verbs to the protocol."
From the protocol crate's design-entry search: "A new command or option widens this crate's
interface, and it must still parse back so scripts replay." From the survey: "begin/end is the
family Qt, Cocoa, GTK and Emacs ship." Bears on `thread@first-prototype@gesture-undo`.

### Shipped undo groups define undo inside a group, nesting, the empty group and a lost end `##arg-66`

Round 2, from the survey and the session. The survey: "undo inside an open group: refuse it (Qt,
GTK), or close the group and undo it at depth 1 (Cocoa)"; "nesting: a counter"; "an empty group:
dropped (GTK) or kept (Qt)"; "A rule for a `begin` never ended, if a pointer release is never
delivered or the editor closes mid-gesture." The session: "Most of these complete the rule you
approved"; "The GUI sends `end` at the pointer release, and when its window loses focus. That
covers a release that never arrives." Bears on `thread@first-prototype@gesture-undo`.

### Cancelling with end then undo leaves a redo step `##arg-67`

Round 2. Session. "One rule widens what you approved, so it goes to you"; "That is Emacs's
`cancel-change-group`"; "Without it, cancelling a drag is `end` then `undo`, which leaves a redo step
behind." The survey: "a cancelled gesture: undo the group (Emacs), or push nothing (Blender)".
Bears on `thread@first-prototype@gesture-undo`, `thread@first-prototype@divider-drag`.

### The browser build cannot read a file outside the presentation's directory `##arg-68`

Round 1. Session, against rewriting paths lexically: "D3's browser build opens one presentation
directory, so it cannot read a file outside that directory." Bears on
`thread@first-prototype@asset-save-rule`.

### Refusing the save refuses the first save of every GUI presentation with an image `##arg-69`

Round 1. Session, against refusing a save into another directory: "The first save of every
presentation built in the GUI that holds an image would be refused, which fails
`goal@rootpoint@no-language-no-build`." Bears on `thread@first-prototype@asset-save-rule`.

### Assets are files beside the presentation file `##arg-70`

Round 1. Session, for "beside": "because `image-storage` says "beside the presentation file"";
"All reads and writes go through `Files`, as `session-files` requires." Bears on
`thread@first-prototype@asset-save-rule`.

### Later, assets in subdirectories of any depth, and packing keeps the tree `##arg-71`

Round 1, the owner's answer, O14. "ultimately (later than initial prototype), I'd like to allow the
user to organize the assets in any way they want with subdirectories of any level, and for a "pack
then unpack elsewhere" action to preserve the tree." The session, round 2: "The rule already allows
it: an asset already inside the presentation's directory, at any depth, keeps its path. A later
packing must keep that tree." Bears on `thread@first-prototype@asset-save-rule`.

### A save that overwrites a file of the author's loses data `##arg-72`

Convergence, the premortem. Session. "Saving into `talk/` copies `cat.png` there, but
`talk/cat.png` already exists and is another picture"; "A save that destroys an existing file is
data loss, and a rule plus a test prevents it." Bears on `thread@first-prototype@asset-save-rule`.

### Writing embedded images out keeps embedded bytes for packing alone `##arg-73`

Round 1. Session. "The saved RON file therefore never holds embedded bytes, and embedded bytes stay
what packing writes, per `asset-source`." From the model crate's design-entry search: "Base64 or
byte arrays in the RON file strain readability." Bears on `thread@first-prototype@sample-save`,
`thread@first-prototype@asset-save-rule`.

### A drag inside a text area belongs to text selection `##arg-74`

Round 1. Session, against a plain drag inside the area: "the next slice needs a drag inside a text
area to select text, for AC1"; against a swap button: "Not defeated by a fact. It adds a mode, and
it costs the same two actions." Bears on `thread@first-prototype@swap-gesture`.

### Shipped drag tools move and split on an edge drop, and preview the target `##arg-75`

Round 2, from the survey and the session. The survey: "Grip-drag onto an area: the design's swap
matches Blender's swap and the centre drop of i3 and iced"; "Edge drop, missing from the design:
... treats an edge drop as move-and-split. That one gesture replaces the design's remove, split and
re-edit"; "Previewing the target: missing from the design. Shipped tools highlight the region or
change the cursor"; "iced uses thirds, i3 30% of the smaller side, VS Code 1/3 and 2/3". The
session: "This needs no new operation, only the existing commands inside `begin`/`end`"; "A drop
onto the dragged area itself is refused." Bears on `thread@first-prototype@swap-gesture`.

### A drop's effect is hinted, not rendered `##arg-76`

Round 2, the owner's answer, O15: "as long as there is a preview/hint of what dropping will do (a
preview of the exact result might not be meaningful in this case, so some sort of hint has to do)."
Convergence, the session: "That is the region highlight the surveyed tools use, without trying to
render the result." Bears on `thread@first-prototype@swap-gesture`.

### Move buttons cost a click per position `##arg-77`

Round 1. Session, against move-up and move-down buttons: "They cost one click per position moved."
For the drag: "egui has drag-and-drop helpers for this (from memory, not checked)." Bears on
`thread@first-prototype@slide-reorder`.

### The desktop can take the app's modifiers `##arg-78`

Convergence, the premortem. Session. "Many Linux window managers take Alt+drag to move a window,
and GNOME uses some Alt+arrow and Ctrl+Alt combinations"; "Only the built app on your desktop can
check it." Bears on `thread@first-prototype@split-preview`, `thread@first-prototype@node-selection`;
gives AC6.

### An agent's command can join the GUI's open group `##arg-79`

Convergence, the premortem. Session. "an agent may send `text n5 "…"` while you are dragging. That
edit joins your drag's group, and one `undo` reverts both"; "The live-control slice is where this
first becomes possible." Bears on `thread@first-prototype@gesture-undo`; gives T5.

### The model has no operation that changes an asset's source `##arg-80`

The review of the spec, a material finding, put to the owner as Q22. Four axes found it alone. The
session: "after a save, the session's in-memory asset paths must change to the new ones. The model
allows a document to change only through an operation, and none can change an existing asset's
path: an asset in use can't be removed and re-added." The default: `Op::SetAssetSource`, applied
through `apply`, no undo step, the sources on the undo and redo stacks rewritten. Its rivals: the
change as an undo step, "an undo would then name a missing file"; the paths left unchanged, "the
open document would differ from the file it just saved." Bears on
`thread@first-prototype@asset-save-rule`.

## Decided design

### The host

eframe hosts the slide view as a texture of `render_slide`'s image and forwards input to the app's
own hit testing; it draws no slide content itself (`thread@first-prototype@gui-toolkit`,
`thread@first-prototype@slide-renderer`).

- The image `render_slide` returns is premultiplied RGBA8 in sRGB, the format of egui's
  `ColorImage::from_rgba_premultiplied`, so it is uploaded with no conversion, by
  `Context::load_texture` once and `TextureHandle::set` after each render.
- The view is rendered with `Precision::Native` at its size in physical pixels, its size in points
  times `pixels_per_point`, and at most 8192 pixels per side: egui-wgpu 0.37.0 asks the GPU for
  textures of at most 8192 per side, in `egui-wgpu/src/setup.rs`, below the render crate's
  `MAX_SIDE` of 16384. Presenter mode renders the same way.
- The app shows every list of the `Report` of `render_slide` that names something not drawn as the
  document asks: missing images, font fallbacks, missing glyphs and undrawn text, so a missing image
  or a missing glyph reaches the author (`tripwire@rootpoint-render@glyph-missing-everywhere` names
  `Report::missing_glyphs` as what it watches).
- The frame code takes an `egui::Ui`, so a test drives it with `egui::Context::run_ui` and synthetic
  input events, with no window and no GPU.

Nearest rival: iced, and the others; see "Losing alternatives" of the milestone document.

### Uniform scaling

The slide view and presenter mode scale the slide uniformly to the largest size that fits the
window, and letterbox the rest (`thread@first-prototype@slide-coordinate-space`), through
`slide_to_pixels`.

### Selection

`thread@first-prototype@node-selection`, `thread@first-prototype@span-selection`.

- The session's `View` gains `shown`, the slide the editor shows. `select n<id>` selects a node and
  shows its slide; `select s<id>` shows a slide and selects no node; `deselect` keeps the slide
  shown. An operation that removes the shown slide drops it, and the editor then shows the first
  slide, as it does when `shown` is none. `view` reports it. So the slide the editor shows is a
  command, as `goal@rootpoint-protocol@commands-drive-everything` asks of every view the editor
  shows.
- A click on an area sends `select`. A click on a slide in the slide list sends `select s<id>`. The
  side panel shows the path from the slide's root to the selection, as a breadcrumb, for instance
  `s1 › n4 beside › n8 stacked › n7 area`; a click on an entry selects it.
- Alt+↑ selects the parent, and does nothing at the root. Alt+↓ selects the child of the selection
  selected last, or its first child, and does nothing on an area; the app keeps, for each split, the
  child last selected in it, which only chooses the node a `select` names. Alt+← and Alt+→ select the
  previous and next sibling, and do nothing at the first or the last.
- A subtree is selected for a spanning split by these, and by no rectangle selection.

Nearest rival: a rectangle drawn across areas selecting the smallest node that holds them, defeated
by `argument@first-prototype@arg-56`.

### Actions on the selection

`thread@first-prototype@selection-actions`, `thread@first-prototype@split-preview`.

- Four buttons in the side panel split the selection at its middle, the new area left, right,
  above or below, with the keys Shift+←, Shift+→, Shift+↑ and Shift+↓: `split <node> beside|stacked
  0.5 before|after`. Delete sends `remove`. The side panel sets the content, its style or its fit,
  and adds an image asset.
- While Ctrl is held over the selected node, a vertical preview line follows the pointer; while
  Shift is held, a horizontal one. A click splits the node at the line, `beside` for Ctrl and
  `stacked` for Shift, at the line's fraction of the node. The new empty area is the smaller of the
  two parts, `before` when it lies left of or above the line, `after` otherwise, and `after` when the
  two are equal. Releasing the key or pressing Esc removes the line and issues nothing.
- With Alt also held, the line snaps to the nearest twelfth of the node, and to a divider elsewhere
  on the slide that lies within 8 points, the nearer of the two.
- A split is refused, its button disabled and its click ignored, where a part would be less than 40
  slide units along the split's axis, the bound a divider drag keeps.

```text
Ctrl held over n2:        Ctrl+Alt, near n9's divider:
+------:---------+        +--------+-------+
|      :   n2    |        |   n2   :       |  the line snaps onto the
|      :         |        |        :       |  divider of the next column
+------:---------+        +--------+-------+
```

Nearest rival: the middle split alone, shape A, which the owner ruled below shape B in O14.

### Dividers

`thread@first-prototype@editor-overlay`, `thread@first-prototype@divider-drag`.

- The editor draws, with egui over the slide's texture, a 1-point line on each divider, a highlight
  on the divider under the pointer, an outline on the selected node, the preview line, and the drop
  hint, whose label is editor text, not slide content. None of them is drawn by `render_slide`, so
  the slide's image is the presenter's.
- A press within 4 points of a divider grabs it, the nearer one where two are within 4 points; the
  cursor is a resize cursor there. The drag runs in an undo group, per "Undo groups", with a
  `divider` command at each frame the position changes; Esc sends `cancel`. A drag stops where a
  child of the split would be less than 40 slide units along the split's axis. With Alt held, the
  position snaps as the preview line does. A double-click on a divider sets it halfway between its
  two neighbours, the neighbouring dividers or the split's edges, in one `divider` command.

Nearest rival: marks drawn into the slide image, defeated by claim 3,
`argument@first-prototype@arg-61`.

### Moving content

`thread@first-prototype@swap-gesture`. The selected area shows a grip, a square of 12 points set 8
points inside its top-left corner, so no divider's band reaches it. A drag of the grip runs in an
undo group, per "Undo groups". The area under the pointer is the target. Its middle third, by width
and by height, is the swap zone; the rest is the move zone of the edge nearest the pointer, the
distances to the edges taken as fractions of the target's width and height, left and right winning
a tie. The zone under the pointer is tinted, with a label beside the pointer: "swap", or "move
left", "move right", "move up", "move down". A drop on the swap zone sends `swap a b`. A drop on a
move zone sends a `split` of the target on that side at 0.5, a `swap` of the dragged area with the
new area, and a `remove` of the dragged area. The group then ends. A drop onto the dragged area, or
outside any area, sends `cancel`.

Nearest rival: a drag inside the area itself, defeated by AC1's text selection,
`argument@first-prototype@arg-74`.

### The slide list

`thread@first-prototype@slide-reorder`. The slide list shows each slide; its buttons add a slide
after the shown one, remove the shown slide and duplicate it, and a drag of a slide to another place
sends `move-slide <slide> <index>`, with egui's `Ui::dnd_drag_source` and `Ui::dnd_drop_zone`.
Should they not serve, step 3 builds the drag from the rows' own drag responses. Nearest rival:
move-up and move-down buttons, defeated by `argument@first-prototype@arg-77`.

### Undo groups

`thread@first-prototype@gesture-undo`. Three commands of the protocol, each replying `ok` or `err`:

- `begin` opens a group; `err` while one is open, since groups do not nest.
- `end` closes the open group: every operation applied inside it is one undo step, undone and
  redone together; an empty group leaves no step; `err` with no group open. The session's undo and
  redo stacks hold steps, each the inverses of one group or of one command.
- `cancel` reverts every operation applied inside the open group, and restores the undo and redo
  stacks as they were at its `begin`; `err` with no group open.
- `undo` and `redo` inside a group close it first, as `end` does, then act. `new`, `load` and
  `sample` close an open group first, as part of replacing the session's document and stacks. View
  commands inside a group apply no operation and join nothing.
- The GUI opens a group when a drag of a divider or of a grip starts, and when the text field gains
  focus; it closes it at the release, when the field loses focus, and when the window loses focus,
  so a release never delivered leaves no group open. Within one frame, the app closes an open group
  before it opens another.

```text
press     -> begin
drag ...  -> divider n4 0 0.41 / 0.43 / 0.47 ...
release   -> end           one undo step: back to 0.40
Esc       -> cancel        back to 0.40, no step
```

Nearest rivals: a preview on a copy, committed once at the release, and the session merging
consecutive commands, both defeated by `argument@first-prototype@arg-64`.

### Saving assets

`thread@first-prototype@asset-save-rule`. Every asset the session holds by `Path` holds a path
relative to `Session::directory`, the directory of the presentation, or the working directory before
a first save or load: `asset <path>` stores the path it is given, and the GUI gives the path its
dialog returns made relative to that directory, with `..` where it lies outside. At a `save` to a
file whose directory is D, each asset is placed as follows, paths compared lexically after joining
them to their directory and removing `.` and `..` components:

- a `Path` asset whose file lies inside D, at any depth, keeps its file, and its path is written
  relative to D;
- any other asset, a `Path` asset whose file lies outside D or an embedded one, is written into D:
  under its file name, or `asset-<id>` with the extension its bytes name for embedded bytes, `.png`,
  `.jpg`, `.ttf`, `.otf`, or none; and its path becomes that name;
- where `Files::exists` finds a file of that name in D, it is read: with the same bytes it is used as
  it is and not written; with other bytes the name takes a suffix, `-2`, `-3` and on, before the
  extension;
- every asset to write is read before anything is written; an asset that cannot be read, or a
  `Files` that cannot tell whether a file exists, makes the save reply `err`, naming the asset, and
  write nothing.

The new paths enter the document through `apply`, one `Op::SetAssetSource` per asset whose source
changes, after the files are written. They are no undo step, since an undo to the old source after
the directory changed would name another file; the session rewrites the same asset's source in the
operations its undo and redo stacks hold, so an undo after the save restores an asset with the path
the saved file uses. A saved file therefore never holds embedded bytes, and
`thread@first-prototype@sample-save` is this rule applied to the sample.

```text
before: dir /home/u/a, a3 = "../Pictures/cat.png", a4 = "logo.png"
save /home/u/a/talk/deck.ron -> talk/deck.ron, talk/cat.png, talk/logo.png
                                a3 = "cat.png", a4 = "logo.png"
```

Nearest rivals: rewriting each path lexically, with no copy, defeated by
`argument@first-prototype@arg-68`, and refusing the save, defeated by
`argument@first-prototype@arg-69`. For the change in memory: a save whose new paths are an undo
step, whose undo would name another file once the directory changed; and a session that keeps its
old paths while the saved file holds new ones, whose document would differ from the file it saved,
against `criterion@first-prototype@c3-readable-roundtrip`.

### The built-in sample

The sample, per D3, is a presentation the protocol crate holds: a RON text and its image files,
included in the crate at build time. `sample` replaces the session's document by it, its images as
`AssetSource::Embedded`, with empty stacks and the working directory as its directory, as `load`
does; a save writes its images beside the saved file, per "Saving assets". So the sample opens in
the GUI, headless and in the browser by one command. It holds at least three slides: a title over
two columns, a slide of text in two styles, and a slide of one image in `fit` and in `fill`; its
texts say how to split, select, drag and swap. Its images stay under 50 KB in all.

Nearest rival: a path the app's `Files` alone serves, which `load` would open in the GUI and in no
headless session, against `goal@rootpoint@headless-core`.

### UI actions to commands

| UI action | commands |
| --- | --- |
| click an area; a breadcrumb entry; Alt+arrows | `select <node>` |
| click a slide in the list | `select <slide>` |
| add slide | `add-slide <index after the shown slide>` |
| remove slide; duplicate slide | `remove-slide <shown>`; `duplicate-slide <shown>` |
| drag a slide in the list | `move-slide <slide> <index>` |
| split buttons, Shift+arrows | `split <node> beside\|stacked 0.5 before\|after` |
| Ctrl or Shift click | `split <node> beside\|stacked <at> before\|after` |
| Delete | `remove <node>` |
| drag a divider | `begin`, `divider <split> <index> <pos>` per frame, `end`; Esc `cancel` |
| double-click a divider | `divider <split> <index> <halfway>` |
| drag the grip | `begin`, then at the drop `swap`, or `split` `swap` `remove`, then `end`; else `cancel` |
| set the content to empty | `clear <area>` |
| set the content to text; type in the field | `begin` at focus, `text <area> "<whole text>"` per change, `end` at focus loss |
| set a style option | `text <area> "<text>" <option>` |
| set the content to an image; its fit | `image <area> <asset> fit\|fill` |
| add an image asset | `asset <path relative to the session's directory>` |
| undo: Ctrl+Z; redo: Ctrl+Shift+Z, Ctrl+Y | `undo`; `redo` |
| open: Ctrl+O; save: Ctrl+S | `load <path>`; `save <path>` |
| open the built-in sample | `sample` |
| present: F5; leave: Esc | `present <shown>`; `stop` |
| next: →, Space, Page Down; previous: ←, Page Up | `next`; `prev` |

While the text field has focus, keys go to the field, and Delete and the arrows act on the text.

### Pointer input to gestures

A press in the slide view, outside presenter mode, takes the first that applies:

| # | where the press is | gesture |
| --- | --- | --- |
| 1 | on the grip of the selected area | grip drag |
| 2 | within 4 points of a divider | divider drag, or a reset on a double-click |
| 3 | inside the selected node, Ctrl or Shift held | split at the preview line |
| 4 | inside an area | `select` of the area |
| 5 | elsewhere, in the letterbox | nothing |

## Losing alternatives

- **Selecting a split by a click on its divider**, lost to `thread@first-prototype@node-selection`:
  the press is a divider grab (`argument@first-prototype@arg-54`).
- **Selecting a subtree by a rectangle**, lost to `thread@first-prototype@span-selection`: a second
  selection path whose result can exceed the rectangle (`argument@first-prototype@arg-56`).
- **The middle split alone**, shape A, lost to `thread@first-prototype@selection-actions` on the
  owner's word in O14 (`argument@first-prototype@arg-58`).
- **Marks drawn into the slide image**, lost to `thread@first-prototype@editor-overlay`: claim 3
  (`argument@first-prototype@arg-61`).
- **A preview on a copy committed at the release, and the session merging consecutive commands**,
  lost to `thread@first-prototype@gesture-undo` (`argument@first-prototype@arg-64`), on the owner's
  word in O14.
- **Rewriting asset paths lexically, and refusing a save into another directory**, lost to
  `thread@first-prototype@asset-save-rule` (`argument@first-prototype@arg-68`,
  `argument@first-prototype@arg-69`). Under it, `thread@first-prototype@sample-save`, superseded:
  its shape, the sample's images written as files, is the rule applied to the sample
  (`argument@first-prototype@arg-73`).
- **A subfolder for the copied assets**, lost to `thread@first-prototype@asset-save-rule` on the
  owner's word in O14, "for now".
- **A plain drag inside an area to swap, and a swap button with a target click**, lost to
  `thread@first-prototype@swap-gesture` (`argument@first-prototype@arg-74`).
- **Move-up and move-down buttons**, lost to `thread@first-prototype@slide-reorder`
  (`argument@first-prototype@arg-77`).

## Fails alone on

- A UI action whose effect differs from its commands', with the protocol's tests passing.
- A drag frame over AC2's bound with the render crate's measured times within it, which points at
  the host's texture path, not at the renderer.

## Premises that expire

- The text of an area is edited in the side panel's plain text field in this slice. Guard:
  `spec@first-prototype@text-editing` replaces it with editing in place, and removes the field.
- The undo groups have one client, the GUI or a script. Guard: T5 of the milestone's premortem,
  at the audit of `spec@first-prototype@live-control`.

## Implementation sequence

1. **The protocol's changes.** `begin`, `end` and `cancel`; the save rule, with
   `Op::SetAssetSource`, `Files::exists`, `Session::directory` and `Session::asset_bytes` public; the
   view's shown slide; `sample` and the sample itself; the texts and the test the save rule changes.
   Claims 4 and 5. Fails alone on: a protocol script, with no app code.
2. **A window that shows one slide.** The app crate; `slide_to_pixels`; the app opens a file and
   shows the shown slide's render, letterboxed, and the render's report. Fails alone on: the texture
   upload.
3. **Selection, dividers and the slide list.** `node_rects`, the selection, the breadcrumb and its
   keys, the overlay, divider drags in groups, the slide list. Claim 2, its first part.
4. **Editing from the UI.** The actions on the selection with the preview and snapping, moving
   content with its hint, content and style, undo and redo, open and save, the built-in sample.
   Claim 1, and claim 2, its second part.
5. **Presenter mode.** Claim 3. Then AC2's drag bound, and AC6.

## Acceptance criteria

This slice judges, at step 5, the drag bound of AC2, `acceptance@first-prototype@render-fast-enough`,
and AC6, `acceptance@first-prototype@modifiers-reach-app`, whose item is in the milestone document.

AC2's drag bound is measured on the owner's machine, release build, in a 1600×900 editor view: a
drag of one divider of the fixture `path@rootpoint-app@tests/data/mixed.ron`, a
slide of five areas mixing text and an image, saved with its image beside it from the render
crate's `mixed` test slide. A frame's time runs from the start of the app's `ui` call to the end of
its `TextureHandle::set`, taken with `std::time::Instant` and logged for each of 100 frames; the
bound holds when every frame is within it, and the largest is reported.

Judged at step 5. AC2's drag bound holds: every frame of the 100 took 4.7 ms to 5.9 ms, release
build, on the owner's Intel Core Ultra 7 165H, the largest 5.9 ms; the measurement is in the
commit "Build presenter mode, and measure AC2's drag bound, for the native GUI slice". AC6 does
not fire: every gesture it lists reaches the app on the owner's desktop, on the owner's report in
O19.

## Harvest

Judged by the entry tests of `primer@design-heads`: `thread@first-prototype@gui-toolkit`, and the
threads of the design session whose harvest names the app crate's Component, into its design home;
the built-in sample as a permanent feature of every build, attributed to the owner, per D3, with its
command `sample`, into the protocol crate's design home; the uniform scaling of
`thread@first-prototype@slide-coordinate-space`, which the render and app crates both respect, into
rootpoint's design home; the losing hosts, judged for a `rejected` entry; the app crate added to the
crates `design@rootpoint@crate-existence-rule` lists. `thread@first-prototype@gesture-undo` and
`thread@first-prototype@asset-save-rule` into the protocol crate's design home, each attributed to
the owner where the owner chose: Q16 and Q20 for the first, Q17's "for now" for the second; their
losing shapes judged for `rejected` entries; the shown slide into `design@rootpoint-protocol@view-state`;
`Op::SetAssetSource` into `design@rootpoint-model@edit-operation-core`'s list where it names the
operations; T5 into the protocol crate's tripwires home, guarding the head of
`thread@first-prototype@gesture-undo`; the closure of
`asset-paths-follow-save`. AC6 reported at step 5.
