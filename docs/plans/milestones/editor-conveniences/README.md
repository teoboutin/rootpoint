# Milestone: conveniences of the editor

## Status and audience

This is the milestone document of a set of conveniences of the editor:
- the slide's whole tree in the side panel, with Alt and the arrows following it;
- a side panel whose width no content changes;
- named text styles, with a Styles tab;
- a menu on a right-click;
- a preview of the final slide inside the editor;
- no `Content::Empty`;
- a rule that tells a structural breach of a document, which is refused, from a cosmetic one, which
  is drawn with a stand-in and reported.

It is written for the sessions that implement its slices, which did not witness the discussion that
produced it. Each slice's spec leaves when that slice lands. This document leaves in the commit
that completes the harvest of its last slice, `spec@editor-conveniences@preview-mode`.

- Where this document, or a slice's spec, and a design home disagree on a point that "What is
  already decided" does not list as reversed or rewritten, the design home wins.
- Every name it uses is defined in it, in "Names" or "New names, in one place", or exists in the
  code.
- Where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says what
  waits. The defaults awaiting the owner are listed in their own section.
- The milestone was assembled from the transcript of one session. The discussion begins in it at
  the owner's message that opens "I have a bunch of small to less small items I want to implement
  to improve the convenience of use of this project".

**The owner's messages of the discussion**, verbatim, in order, up to its convergence. Round n is the
owner's message n and the session's reply to it. Round 5 was typed while the session worked on the
hand-off. The owner's later rulings on D1 and D2 are in "Defaults awaiting the owner".

**Round 1**

> I have a bunch of small to less small items I want to implement to improve the convenience of use of this project. Here they are, let's discuss them, choose designs, and plan for implementation:
> - show node selection (the one on the right) as the full tree (of current slide) and highlight currently selected node. Maybe make a visual disctinction between intermediate nodes and leaf nodes that can hold content. Under this shape, I believe for the alt+arrow movements (a recorded design issue), the natural movement becomes "alt+up/down" navigates subtrees at the same depth, while "alt+left/right" navigates by changing depth. At least this is natural from the viewpoint of the tree display. Not sure how this would translate in the rendered view. And if that is actually the current shape, this means we can keep it for now. The tree display with dynamic update will make it clearer about what this navigation shortcut does. If that keeps being confusing, the spatial layout based navigation will be reopened again.
> - the right sidebar should have a constant width. The width changes abruptly between the state where a node is selected and the one where only the slide is selected, which is inconvenient, as it resizes the render area upon clicking on a node.
> - ability to save and edit predef styles for text (used for things like Title, Subtitle, Legend...), with a style editor view. text nodes select a style or choose "custom" for a local style only. Each node kind should probably have its own "Style" object.
> - context menu on right clicking a node: allow choosing to select among parents, to remove the clicked child, or to change its content kind (and style).
> - ask confirmation before removing a node with non empty content. It is important that a cell with content cannot be lost "by mistake". At least for text content, which is not stored outside of cell content (images are not unloaded if no cell reference it AFAIK).
> - button in top menu bar to render in "final result" mode (without the splits or other editor overlay) without switching to presenter view.
> - the default content of a node should probably be text, not empty. Actually, the empty kind is useless, it is basically a text content with no character. I think it is just noise in both the gui and the implementation.

**Round 2**

> tree-outline approved, this is the look I wanted.
> selection-keys: I have a hard time deciding on that. I think we can go with my proposal for now, at least it would be relatively consistent with the tre view (though your up/down moving one line might be a bit more natural). This is not very hard to change anyway.
> sidebar-width: accepted, draggable and width do not follow content.
> named-styles: accepted, this is the shape I was thinking about.
> style-key: approved, referral by name is important. Also, a missing style should use a fallback and give a visible warning, not be a hard error.
>
> style-kinds: agreed, deferred for when it makes more sense, for images it makes little sense right now with a single option.
>
> new-area-style: accepted, this looks convenient.
> no-empty-kind: accepted.
>
> format-migration, Q1: there is not a single existing presentation for now, and as long as the project is still in initial development, I won't be using it for real. This should be recorded, probably in claude.md. The project does not keep retrocompat at all while in initial development, it only starts being needed when I rule it (when I start using the project for actual presentations).
>
> node-menu: accepted, select is good.
>
> removal-guard: on second thought, I withdraw this proposal. You are right that undo covers loss of content. Instead, I think the better way to improve on this aspect, rather than adding inconvenient dialogs, would be to provide more detailed informations about what history is saved, the ability to fetch something in the history and reapply it (or at least try to), maybe even keeping tracks of forks in history (do A, undo, do B, undo, redo A). But this can be kept as a todo issue, not in scope today.
>
> final-preview: instead of blocking selection, having a selection (click) exit the preview mode might be more convenient. Hitting Preview exits all selections, clicking on the slide exits Preview and selects. Intuitively, I think I would use it this way. I'm not fully certain though.

**Round 3**

> missing-style-fallback: I think this principle of fallbacks should be generalized. The important point is to distinguish structural vs cosmetic inconsistencies in documents. When a cosmetic fallback can be used, it should IMO. When the file has a structural change that would make it either impossible to render properly, or one that is not obvious to recover from by a user, we can refuse it.
> Though, maybe we should just make sure that none of those situation can ever happen under normal usage (without manual file edits), instead of giving a fallback.
> What would happen when a style is removed by the user, for nodes that are using it ? An automatic fallback to default ? Maybe the user can name a style that should be used as replacement.
>
> Also, I think later we should start fuzzing the project. This is gonna help us make sure no user edit can lead to an invalid style. The text protocol is quite adapted for this I believe.  There is already some property based testing in place too, right ? This is a todo issue too.
> What do you think is best, fallback allowed, or no fallback and instead thorough testing of invariants, property based and fuzzing ?
>
> no-compat-in-initial-dev: accepted on recording locations.

**Round 4**

> Approved all three, record T1–T4 and AC1–AC4.
> Record the issue not about just fuzzing, but more generally about the thorough testing that needs to be setup and a few open proposal of common approaches (including widening property based tests).

**Round 5**

> final-preview accepted too

## How the work is done

Before each slice, read `skill@knowledge-architect-planning@working-a-slice` whole, and follow it.
The slices are built in the order of "Implementation sequence". #empty-text was one branch, merged
as `instructions@git-workflow` says. On the owner's word, every later slice is built on one branch:
each slice is audited, built, reviewed and harvested in turn, and the branch is reviewed as before
a merge after each harvest, with no merge between slices. The owner asked: "implement all the
remaining slices of the editor-conveniences milestone, in a single branch. Proceed with the normal
workflow, audit, implementation, harvest, review (as if you were going to merge), then back to
audit for the next slice. Only pause when you absolutely need my word to fill a gap, otherwise
proceed with sensible defaults." A gap the owner must rule on is still put to the owner, by
`primer@question-authority`: a default is built and the work goes on, and a load-bearing gap stops
its slice.

- **Gates.** `cargo x gates` runs every check a branch owes before it merges,
  `instructions@merge-gates`. Every slice keeps them
  passing, the gate `wasm` and the browser tests of the gate `wasm-test` included: a test added to
  `path@rootpoint-app@tests/actions.rs` or `path@rootpoint-app@tests/web.rs` builds and passes on the
  wasm target, as `path@rootpoint-app@CLAUDE.md` says.
- **Actions are commands.** Every new action of the window is a function of
  `path@rootpoint-app@src/actions.rs` that issues protocol commands through `App::run`, checked
  against the same commands on a second session in `path@rootpoint-app@tests/actions.rs`, per
  `design@rootpoint-app@actions-are-commands`.
- **No compatibility.** A slice that changes the save format bumps `FORMAT_VERSION` in
  `path@rootpoint-model@src/file.rs` and writes no migration, per
  `design@rootpoint@no-compat-in-initial-dev`. The committed files in the old format are rewritten
  in the new one in the same slice: every `.ron` file `find crates -name '*.ron'` lists, four at the
  time of writing. `path@rootpoint-model@tests/data/fixture.ron` is written again by running its
  test `the_fixture_round_trips_and_matches_its_committed_file` with `ROOTPOINT_BLESS=1`; the others
  are edited by hand. A test that writes the version as a literal, such as `version: 1`, follows
  `FORMAT_VERSION`.
- **This milestone's branch** carries this document, the two issues the discussion asked for, and
  the decisions no work implements: `design@rootpoint@no-compat-in-initial-dev` with
  `rejected@rootpoint-model@format-migration`, and `design@rootpoint-app@no-discard-confirmation` with
  `rejected@rootpoint-app@removal-guard`. It merged once the owner had ruled on every default
  awaiting the owner, before the first slice, #empty-text, began. #empty-text branched from the
  main branch; every later slice is on one branch, as above.

## Names

- **The side panel**: the right panel of the editor, `Panel::right("selection")` in `App::editor`,
  `path@rootpoint-app@src/lib.rs`, drawn by `App::side_panel` in
  `path@rootpoint-app@src/panels.rs`.
- **The outline**: `App::outline` in `path@rootpoint-app@src/panels.rs`, the side panel's rows of
  the shown slide's tree, which replaced the breadcrumb at #side-panel.
- **The overlay**: `App::overlay` in `path@rootpoint-app@src/view.rs`, which draws the editor's
  marks over the slide's texture.
- **The node keys**: Alt with an arrow, read in `App::shortcuts` in `path@rootpoint-app@src/lib.rs`,
  which calls `App::move_selection` with a `selection::Move` of
  `path@rootpoint-app@src/selection.rs`.
- **The report**: `rootpoint_render::Report`, what a render could not draw, read by
  `App::status` in `path@rootpoint-app@src/lib.rs` and printed by the reply of the protocol's
  `render`.
- **A stand-in**: what is drawn in place of a missing target: the grey of a missing image, the
  built-in sans for a missing font, `TextStyle::default()` for a missing style.
- **A structural breach** and **a cosmetic breach**: the two kinds of broken document,
  `thread@editor-conveniences@cosmetic-fallbacks` defines them.
- **The predefined styles**: Title, Subtitle, Body and Legend, the styles a new document holds,
  `thread@editor-conveniences@new-area-style`.
- **T1 to T4, AC1 to AC4**: the labels under which the premortem's tripwires and acceptance
  criteria were put to the owner, "Premortem" below.
- **D1 to D7**: the labels of the defaults put to the owner, "Defaults awaiting the owner" below.

## What the work is

**What exists today, at each site the work touches.**

- **Content.** `Content` in `path@rootpoint-model@src/document.rs` is `Text(TextContent)` or
  `Image(ImageContent)`, and `TextContent` holds a `text` and a `StyleChoice`. A new area, made by
  `Op::Split` or by `Op::AddSlide`, holds an empty text in the style `split_style` or
  `Document::new_area_style` gives, in `apply_in` of `path@rootpoint-model@src/ops.rs`, per
  `design@rootpoint-model@new-area-style`. The slice #empty-text removed `Content::Empty`, and
  #styles-model set `FORMAT_VERSION` to 3.
- **Validation.** `Document::validate` in `path@rootpoint-model@src/validate.rs` refuses a
  structural breach, an asset id at or above the counter included, per
  `design@rootpoint-model@validated-documents`; `load_ron` and `apply` call it. `Document::breaches`
  names the cosmetic ones, `Breach::MissingAsset`, `Breach::UnknownFont` and
  `Breach::MissingStyle`, which load and apply, per `design@rootpoint-model@cosmetic-fallbacks`.
  The commands refuse a missing asset or style, per `design@rootpoint-protocol@no-new-breach`. The
  slices #breach-tiers and #styles-model built these.
- **Stand-ins.** The renderer draws an image it was not given as grey and lists it in
  `missing_images`, in `render_slide` of `path@rootpoint-render@src/lib.rs`; a font it cannot find
  draws in the built-in sans, chosen by `FontSet::family` in `path@rootpoint-render@src/text.rs`, and
  `render_slide` lists it in `font_fallbacks`. A `FontRef::Builtin` naming a family outside
  `BUILTIN_FAMILIES` of `path@rootpoint-model@src/document.rs`, "sans" alone, falls back the same way.
- **Styles.** A document holds a table of named styles, and a text names one or holds its own
  values, per `design@rootpoint-model@named-styles`; `Document::resolve_style` gives the values
  drawn. The protocol's `style add`, `style set`, `style rename`, `style remove` and `styles` change
  and list the table; the slice #styles-model built these. The side panel's Selection tab names a
  style or makes a text custom, and its value controls, `App::style`, which return one
  `StyleOption`, make a named text custom; its Styles tab, `App::styles_tab`, is the one part of the
  window that changes a named style, per `design@rootpoint-app@styles-tab-edits-styles`, and asks
  for a replacement when a style in use is removed. The slice #styles-window built these.
- **Selection.** The side panel's outline shows the slide's tree, a click on a row selects its node,
  and Alt with ↑ and ↓ selects a sibling, with ← the parent and with → a child, per
  `design@rootpoint-app@node-selection` and `design@rootpoint-app@selection-keys`; the key table
  is in `App::shortcuts`. The slice
  #side-panel built these.
- **The side panel's width.** `Panel::right("selection").default_size(260.0)`, its content a scroll
  area the size of the panel, so no content changes the width, per
  `design@rootpoint-app@sidebar-width`, which holds the reading of egui 0.37.0's panel sizing. The
  width test of `path@rootpoint-app@tests/panel.rs` measures it in each state. The slice
  #side-panel built these.
- **Right-click.** The slide view and the outline's rows read a secondary click, and open the node
  menu, `design@rootpoint-app@node-menu`; the slice #context-menu built these. In the browser, eframe 0.37.0 stops the browser's own context menu on the canvas, in
  `install_event_handlers` of eframe's `path@elsewhere@src/web/events.rs`.
- **The view.** `View` in `path@rootpoint-protocol@src/session.rs` holds `selected`, `shown` and
  `presenting`, per `design@rootpoint-protocol@view-state`.
- **Property tests.** proptest runs in rootpoint-model, in `path@rootpoint-model@tests/ops.rs`,
  `path@rootpoint-model@tests/file.rs` and `path@rootpoint-model@tests/geometry.rs`, and in
  rootpoint-protocol, in `path@rootpoint-protocol@tests/properties.rs`, which #breach-tiers added:
  random sequences of commands leave no breach, keep the document valid, and save to a text that
  loads back equal.

**Outside the work, and why.**

- A confirmation before a removal: withdrawn by the owner,
  `thread@editor-conveniences@removal-guard`. Browsing, fetching from and branching the undo history
  is `issue@rootpoint-protocol@history-browsing`.
- Fuzzing, and testing beyond the one property test of
  `thread@editor-conveniences@no-new-breach`: `issue@rootpoint@thorough-testing`.
- Named styles for a kind other than text: `thread@editor-conveniences@style-kinds`.
- A style per span of a text: `issue@rootpoint@rich-text-spans`.
- Templates shared by slides: `issue@rootpoint-model@slide-templates`.
- Spatial selection keys: watched by T1, `thread@editor-conveniences@selection-keys`.
- A migration of format version 1: `design@rootpoint@no-compat-in-initial-dev`.

## What is already decided

**Recorded decisions the design rests on, not argued again.**

- `design@rootpoint@no-compat-in-initial-dev`: no migration for the format changes of this work.
- `design@rootpoint-app@no-discard-confirmation`: no slice adds a confirmation before a removal;
  undo restores it.
- `design@rootpoint-model@edit-operation-core`: every new change is an `Op` with an inverse.
- `design@rootpoint-app@actions-are-commands`: every new window action is protocol commands.
- `design@rootpoint-app@one-controller-at-a-time`: new edit actions return while the socket holds
  the authority; new view-only inputs of the window stay usable.
- `design@rootpoint-render@slide-renderer`: the preview draws through `render_slide`.
- `design@rootpoint-render@area-text-layout`: `area_text` and the drawing share one layout. Its
  signature takes the document, so it can resolve a named style without a change of its interface.
- `design@rootpoint-model@split-tree-arity`: the outline shows the merged tree, where a split has two
  children or more and no child split of its own axis.
- `design@rootpoint-model@stable-ids` and `design@rootpoint-protocol@protocol-addressing`: nodes,
  slides and assets keep their ids; styles are named by name, `design@rootpoint-model@style-key`,
  which #styles-model wrote.
- `design@rootpoint-model@save-format`: RON, with the version first.
- `design@rootpoint-app@editor-overlay`: the marks are drawn over the texture, never into it.

**Recorded decisions the work rewrites, with every text that cites each, and where each is
updated.** The texts are those `cargo klarch show` lists.

| decision | change | slice | citing texts, judged at that slice's harvest |
| --- | --- | --- | --- |
| `design@rootpoint-model@inline-content-assets` | "empty" leaves its list of contents | #empty-text, landed | none |
| `design@rootpoint-model@inline-content-assets` | a text's style may name an entry of the document's style table, by name: shared data held apart from the area, argued against `rejected@rootpoint-model@content-by-reference` below | #styles-model, landed | none |
| `design@rootpoint-app@selection-actions` | "the new empty area" becomes the new area, an empty text | #empty-text, landed | `design@rootpoint-app@edit-entry` |
| `design@rootpoint-app@edit-entry` | a second click enters editing on any area that is not an image, since every such area is a text | #empty-text, landed | none |
| `design@rootpoint-app@edit-entry` | a secondary click, inside the edited area too, ends editing before it opens the node menu; built as D7, a default awaiting the owner, and recorded in the head once the owner rules | #context-menu, landed | none |
| `design@rootpoint-model@validated-documents` | a cosmetic breach is accepted, reported and drawn with a stand-in, `design@rootpoint-model@cosmetic-fallbacks`; a structural breach is still refused | #breach-tiers, landed | the comments of `path@rootpoint-model@src/ops.rs` and `path@rootpoint-model@src/validate.rs` |
| `design@rootpoint-model@text-style-per-area` | a text's style is named or custom | #styles-model, landed | `issue@rootpoint@rich-text-spans` |
| `design@rootpoint-protocol@protocol-addressing` | planned as a widening to names of styles, argued by `thread@editor-conveniences@style-key`; as landed, the head keeps ids alone and points at `design@rootpoint-model@style-key`, which holds the names, since the two lose to different rivals | #styles-model, landed | `design@rootpoint-app@one-controller-at-a-time` |
| `design@rootpoint-app@node-selection` | the outline replaces the breadcrumb; the node keys follow the outline | #side-panel, landed | `design@rootpoint-app@edit-entry`, `design@rootpoint-app@editor-keys-kept-from-browser`, the comment of `path@rootpoint-app@src/hit.rs` |
| `design@rootpoint-app@span-selection` | "Alt+↑ or the breadcrumb" becomes Alt+← or the outline | #side-panel, landed | none |
| `design@rootpoint-protocol@view-state` | `View` gains the preview | `spec@editor-conveniences@preview-mode` | `design@rootpoint-app@node-selection`, `design@rootpoint-app@one-controller-at-a-time`, `issue@rootpoint-protocol@history-browsing` |
| `design@rootpoint-app@editor-overlay` | the overlay draws nothing while the preview is on | `spec@editor-conveniences@preview-mode` | `path@rootpoint-app@CLAUDE.md`, `design@rootpoint-app@edit-in-place`, `design@rootpoint-app@take-over-at-once`, the comment of `path@rootpoint-app@tests/presenter.rs` |
| `design@rootpoint-app@one-controller-at-a-time` | the outline, the node menu's "Select" and the Preview button join the window's view-only inputs | #side-panel, landed for the outline; #context-menu, landed for the menu; `spec@editor-conveniences@preview-mode` | the 15 texts `cargo klarch show` lists; each slice reads the ones its change makes false |

The design issue `alt-arrows-follow-the-tree` of rootpoint-app, which asked for this discussion of the
node keys, is closed by the commit that adds this document, since the document schedules its work.
T1 takes its place, and keeps the rival the issue named.

**The standing entries the discussion's search returned, and their outcome.**

- `alt-arrows-follow-the-tree`, a design issue of rootpoint-app: this work is the design session it
  asks for; closed by the commit that adds this document.
- `issue@rootpoint@rich-text-spans`: named styles are per area and leave spans out; a span would
  later override its area's style. Not closed.
- `issue@rootpoint@more-content-kinds`: a new kind decides then whether it shares the text styles,
  `thread@editor-conveniences@style-kinds`. Not closed.
- `issue@rootpoint-model@slide-templates`: the style table is a second table of the document beside
  the assets; templates are not designed here, and nothing here constrains them beyond that. Not
  closed.
- `issue@rootpoint@image-export`: the preview draws `render_slide`'s image, as an export would; no
  mechanism is shared. Not closed.
- `issue@rootpoint@pack-and-unpack`: the styles live in the presentation file, so packing carries
  them with no change. Bears on nothing.
- `issue@rootpoint-app@socket-commands-not-shown`: its view of the commands would go beside the
  slide list or under the slide view, not in the side panel, whose width
  `thread@editor-conveniences@sidebar-width` fixes. Bears on nothing.
- `issue@rootpoint-app@abandoned-composition-undo-step`: an empty text in every new area makes a
  stay in editing more frequent; the defect is unchanged. Bears on nothing in the work.
- `issue@rootpoint-app@changed-asset-not-read-again` and `issue@rootpoint-app@wider-lock`: bear on
  nothing.
- `tripwire@rootpoint-model@grouping-lost-to-merge`: the outline shows the merged tree, which is
  where the owner would meet a lost grouping; #side-panel read it at its audit.
- `tripwire@rootpoint-render@render-within-bounds`: its re-entry is the design audit of work that
  adds a content kind or changes the renderer; #empty-text, #breach-tiers and #styles-model read it
  at their audits, and `spec@editor-conveniences@preview-mode` reads it at its audit.
- `tripwire@rootpoint-app@control-ends-a-gesture`: the node menu and the dialog of a style's removal
  are open states of the window; a socket's `control` closes them as it ends a gesture.
  #context-menu and #styles-window read it at their audits.

**The design and `rejected` entries the design-entry searches graded as bearing, and their
outcome.** The goals are the criteria below. Beyond the decisions listed above:

- `rejected@rootpoint-model@content-by-reference`: named styles are shared data referred to by name,
  which this entry rejects for content. Its costs are answered for styles: a style no area uses is
  a library entry, not an orphan; editing a shared style is the purpose, and only the Styles tab
  does it; the file reads `Named("Title")` in place. Not brought back.
- `rejected@rootpoint-model@gui-mutation`: every style edit is an `Op`. Not brought back.
- `rejected@rootpoint-model@unvalidated-documents`: its three counterexamples are structural
  breaches, which stay refused. Not brought back.
- `design@rootpoint-model@split-any-node`: the node menu's "Select" and the outline select splits,
  as this head allows. Unchanged.
- `design@rootpoint-model@slide-units`: a style's size is in slide units. Unchanged.
- `design@rootpoint-protocol@protocol-transport`: the new commands follow its grammar. Unchanged.
- `design@rootpoint@uniform-scaling`: the preview and a constant panel width keep one scale per
  display. Unchanged.
- `design@rootpoint@gui-only-in-app`: styles, their operations and the preview's view state are in
  the model and the protocol; only their widgets are in the app. Unchanged.

## Criteria

### Every new edit is an operation of the model with an inverse, and a command of the protocol `##c-headless-commands`

Binding. Source: `goal@rootpoint@headless-core`, `goal@rootpoint-protocol@commands-drive-everything`
("every view of the presentation it shows (which slide, which node and which mode), is a command"). Met by `thread@editor-conveniences@named-styles` (the style
operations and commands), `thread@editor-conveniences@no-empty-kind`,
`thread@editor-conveniences@final-preview` (`preview`), `thread@editor-conveniences@node-menu` and
`thread@editor-conveniences@selection-keys`, which use existing commands.

### The preview shows the image `render_slide` draws, with no mark `##c-clean-preview`

Binding. Source: `goal@rootpoint-app@faithful-slide-view`. Met by
`thread@editor-conveniences@final-preview`.

### The saved file stays readable and editable by hand, and loads back equal `##c-readable-file`

Binding. Source: `goal@rootpoint@readable-save-file`, `goal@rootpoint-model@faithful-readable-file`.
Met by `thread@editor-conveniences@named-styles`, `thread@editor-conveniences@style-key`.

### Styles serve a content kind, and nothing outside the kinds `##c-kind-scoped-styles`

Binding. Source: `goal@rootpoint@minimal-content-set`. Met by
`thread@editor-conveniences@style-kinds`.

### Everything works in the browser build `##c-browser-parity`

Binding. Source: `goal@rootpoint-app@browser-window`. Met by `thread@editor-conveniences@node-menu`
(`argument@editor-conveniences@arg-13`); the rest uses egui only.

### While the socket holds control, edits are refused and view-only inputs still work `##c-one-controller`

Binding as a presumption. Source: `design@rootpoint-app@one-controller-at-a-time`. Met by
`thread@editor-conveniences@node-menu`, `thread@editor-conveniences@final-preview`.

### Clicks and keys per common task `##c-fewer-actions`

Weighed. Source: the owner's request in round 1, `goal@rootpoint-app@direct-manipulation`. Met by
`thread@editor-conveniences@new-area-style`, `thread@editor-conveniences@no-empty-kind`,
`thread@editor-conveniences@final-preview`.

## Threads

Every thread lives here, since this document cites each.

### The side panel shows the slide's whole tree as an indented outline `##tree-outline`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-1`); shaped by the session in
round 1 (`argument@editor-conveniences@arg-16`). Final state: approved, in round 2: "tree-outline
approved, this is the look I wanted." Built by #side-panel. Harvested: `design@rootpoint-app@tree-outline`,
since the keys rely on its shape too, and the outline's rows as a way of selecting in
`design@rootpoint-app@node-selection`.

### Alt with ↑ and ↓ selects a sibling, and Alt with ← and → the parent and a child `##selection-keys`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-2`); the session showed that
it changes the current keys (`argument@editor-conveniences@arg-9`) and gave the rival
(`argument@editor-conveniences@arg-17`). Final state: approved, in round 2: "selection-keys: I have
a hard time deciding on that. I think we can go with my proposal for now, at least it would be
relatively consistent with the tre view (though your up/down moving one line might be a bit more
natural). This is not very hard to change anyway." The owner chose among the argued rivals, for
now, and hedged. Built by #side-panel. Harvested: `design@rootpoint-app@selection-keys`, attributed
to the owner with the hedges, cut out of the rewritten `design@rootpoint-app@node-selection`, and
T1 as `tripwire@rootpoint-app@keys-confuse-with-the-outline`.

### The side panel's width never follows its content `##sidebar-width`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-3`); mechanism and rival by the
session (`argument@editor-conveniences@arg-12`, `argument@editor-conveniences@arg-18`). Final
state: approved, in round 2: "sidebar-width: accepted, draggable and width do not follow content."
Built by #side-panel. Harvested: `design@rootpoint-app@sidebar-width`, since its argument turns on
egui's behaviour. AC2 was judged at #styles-window, not fired.

### A table of named text styles in the document, and each text names one or holds its own `##named-styles`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-4`); shaped by the session in
round 1 (`argument@editor-conveniences@arg-19`, `argument@editor-conveniences@arg-20`,
`argument@editor-conveniences@arg-21`). Final state: approved, in round 2: "named-styles:
accepted, this is the shape I was thinking about." Its round-1 detail that removing a style in use
is refused is replaced by `thread@editor-conveniences@style-removal`. Built in the model and the
protocol by the slice #styles-model, harvested as `design@rootpoint-model@named-styles`, with the
rewrite of `design@rootpoint-model@text-style-per-area`. The window's half, built by #styles-window,
harvested as `design@rootpoint-app@styles-tab-edits-styles`.

### A new area holds an empty text, with the style of the area it was split from, or Body `##new-area-style`

Proposed by the session in round 1, as a consequence of `thread@editor-conveniences@named-styles`
and `thread@editor-conveniences@no-empty-kind`. Final state: approved, in round 2: "new-area-style:
accepted, this looks convenient." Built by #styles-model. Harvested: the head
`design@rootpoint-model@new-area-style`, and T4 as `tripwire@rootpoint-model@inherited-style-in-the-way`.
D3 and D5 await the owner's word on two conditions of it,
`issue@rootpoint-model@new-area-style-awaits-rulings`.

### Styles are named in the file and in commands by their name `##style-key`

Proposed by the session in round 1 (`argument@editor-conveniences@arg-22`), as a weighing for the
owner. Final state: approved, in round 2: "style-key: approved, referral by name is important."
(`argument@editor-conveniences@arg-32`). The owner chose names over ids. Built by #styles-model.
Harvested: the head `design@rootpoint-model@style-key`, with the owner's ground, and
`rejected@rootpoint-model@style-ids`; and the rewrite of
`design@rootpoint-protocol@protocol-addressing`.

### Named styles are for text only, until another kind needs its own `##style-kinds`

Proposed by the session in round 1 (`argument@editor-conveniences@arg-23`), against the owner's
hedged "Each node kind should probably have its own "Style" object." Final state: approved, in round
2: "style-kinds: agreed, deferred for when it makes more sense, for images it makes little sense
right now with a single option." (`argument@editor-conveniences@arg-33`). Built by #styles-model.
Harvested: the head `design@rootpoint-model@style-kinds`, of its own since its rival is another
than named styles'; no issue, since `issue@rootpoint@more-content-kinds` is where a new kind is
decided.

### `Content::Empty` is removed: an area holds a text, possibly with no characters, or an image `##no-empty-kind`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-8`); consequences and rival by
the session (`argument@editor-conveniences@arg-15`, `argument@editor-conveniences@arg-24`). Final
state: approved, in round 2: "no-empty-kind: accepted." Built by the slice #empty-text. Harvested:
the head `design@rootpoint-model@no-empty-kind`, which holds its shape, with
`rejected@rootpoint-model@empty-content-kind`; the rewrites of
`design@rootpoint-model@inline-content-assets`, `design@rootpoint-app@selection-actions` and
`design@rootpoint-app@edit-entry`; and T3 as `tripwire@rootpoint-app@empty-text-takes-the-keys`.

### Format version 1 is migrated on load `##format-migration`

Proposed by the session in round 1 (`argument@editor-conveniences@arg-25`), with Q1. Final state:
ruled-out, on the owner's answer to Q1 in round 2 (`argument@editor-conveniences@arg-34`): "there
is not a single existing presentation for now" and "The project does not keep retrocompat at all
while in initial development". Shape: "Losing alternatives". Harvest: recorded on this milestone's
branch, as `rejected@rootpoint-model@format-migration`, which `design@rootpoint@no-compat-in-initial-dev`
cites.

### No backward compatibility while the project is in initial development `##no-compat-in-initial-dev`

Proposed by the owner in round 2 (`argument@editor-conveniences@arg-34`); its homes proposed by the
session (`argument@editor-conveniences@arg-39`, `argument@editor-conveniences@arg-40`). Final state:
approved, in round 3: "no-compat-in-initial-dev: accepted on recording locations." Recorded on this
milestone's branch as `design@rootpoint@no-compat-in-initial-dev`, with the pointer
`instructions@no-backward-compat`, a decision that no work implements.

### Right-clicking an area or an outline row selects that node and opens a menu `##node-menu`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-5`); shaped by the session
(`argument@editor-conveniences@arg-13`, `argument@editor-conveniences@arg-26`). Final state:
approved, in round 2: "node-menu: accepted, select is good." Shape:
#context-menu, landed. Harvest: `design@rootpoint-app@node-menu`, and AC3 reported.

### A dialog before any action of the window that discards content `##removal-guard`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-6`); widened and argued by the
session (`argument@editor-conveniences@arg-10`, `argument@editor-conveniences@arg-11`,
`argument@editor-conveniences@arg-27`). Final state: withdrawn by the owner, in round 2
(`argument@editor-conveniences@arg-35`): "on second thought, I withdraw this proposal. You are right
that undo covers loss of content." The defeating reason: `Op::Remove` returns a snapshot of the
slide's tree as its inverse, so undo restores what a removal took. Replaced by
`issue@rootpoint-protocol@history-browsing`. Shape: "Losing alternatives". Harvest: recorded on this
milestone's branch, a decision no work implements: `design@rootpoint-app@no-discard-confirmation`,
a head of absence, and `rejected@rootpoint-app@removal-guard`.

### A "Preview" button toggles a final-result view inside the editor, and a click on the slide ends it `##final-preview`

Proposed by the owner in round 1 (`argument@editor-conveniences@arg-7`); the session placed it in
the session's view state (`argument@editor-conveniences@arg-14`) and first proposed gestures off
(`argument@editor-conveniences@arg-28`). The owner proposed the click that ends it in round 2
(`argument@editor-conveniences@arg-36`): "Hitting Preview exits all selections, clicking on the
slide exits Preview and selects. Intuitively, I think I would use it this way. I'm not fully
certain though." The session adopted it as a rule of the session
(`argument@editor-conveniences@arg-42`). Final state: approved, in round 5: "final-preview accepted
too". Shape: `spec@editor-conveniences@preview-mode`. Harvest: the rewrites of
`design@rootpoint-protocol@view-state` and `design@rootpoint-app@editor-overlay`, and T2.

### A text naming a missing style draws with the default style and is reported `##missing-style-fallback`

Proposed by the owner in round 2 (`argument@editor-conveniences@arg-32`): "a missing style should
use a fallback and give a visible warning, not be a hard error." The session showed that it is an
exception to `design@rootpoint-model@validated-documents` (`argument@editor-conveniences@arg-37`,
`argument@editor-conveniences@arg-38`). Final state: superseded by
`thread@editor-conveniences@cosmetic-fallbacks`, which the owner generalized from it in round 3 and
which carries its shape whole; closed with it in round 4. Built by #styles-model, as a member of
`thread@editor-conveniences@cosmetic-fallbacks`. Harvested: `Breach::MissingStyle` is named in
`design@rootpoint-model@cosmetic-fallbacks`. Its third place, the outline's row, is judged by AC1.

### A structural breach of a document is refused; a cosmetic one is drawn with a stand-in and reported `##cosmetic-fallbacks`

Proposed by the owner in round 3 (`argument@editor-conveniences@arg-44`); the test that tells the
two apart by the session (`argument@editor-conveniences@arg-50`,
`argument@editor-conveniences@arg-51`, `argument@editor-conveniences@arg-52`,
`argument@editor-conveniences@arg-53`). Final state: approved, in round 4: "Approved all three".
Built by the slice #breach-tiers. Harvested: the rewrite of
`design@rootpoint-model@validated-documents`, and the head `design@rootpoint-model@cosmetic-fallbacks`,
which holds its shape and attributes the principle to the owner, with
`rejected@rootpoint-model@every-breach-refused`.

### Removing a style that areas use takes a replacement: another named style, or a custom copy `##style-removal`

Proposed by the owner in round 3 (`argument@editor-conveniences@arg-46`): "Maybe the user can name
a style that should be used as replacement."; the custom copy and the dialog by the session
(`argument@editor-conveniences@arg-54`). Final state: approved, in round 4: "Approved all three".
Built in the model and the protocol by #styles-model, harvested as
`design@rootpoint-model@style-removal`. The dialog, built by #styles-window, holds the rule's window
half in the doc comment of `App::removal_dialog`, since the head holds the rule.

### No command creates a cosmetic breach, and a property test over the protocol's commands checks it `##no-new-breach`

Proposed by the owner in round 3 (`argument@editor-conveniences@arg-45`,
`argument@editor-conveniences@arg-47`); the split between commands and operations and the test by
the session (`argument@editor-conveniences@arg-49`, `argument@editor-conveniences@arg-55`). Final
state: approved, in round 4: "Approved all three". Built by the slice #breach-tiers, and widened by
#styles-model. Harvested: the protocol head
`design@rootpoint-protocol@no-new-breach`. AC4 fired at #styles-model, D4.
Fuzzing and wider testing: `issue@rootpoint@thorough-testing`.

## Arguments

Every argument lives here.

### The selection is shown as the whole tree, with split rows told from content rows `##arg-1`

Round 1, owner. Bears on `thread@editor-conveniences@tree-outline`,
`thread@editor-conveniences@selection-keys`. "show node selection (the one on the right) as the
full tree (of current slide) and highlight currently selected node. Maybe make a visual disctinction
between intermediate nodes and leaf nodes that can hold content."

### In a tree view, Alt with ↑ and ↓ moves at one depth and Alt with ← and → changes depth `##arg-2`

Round 1, owner. Bears on `thread@editor-conveniences@selection-keys`. "the natural movement becomes
"alt+up/down" navigates subtrees at the same depth, while "alt+left/right" navigates by changing
depth. At least this is natural from the viewpoint of the tree display." / "The tree display with
dynamic update will make it clearer about what this navigation shortcut does. If that keeps being
confusing, the spatial layout based navigation will be reopened again."

### A panel width that changes on a click resizes the slide view `##arg-3`

Round 1, owner. Bears on `thread@editor-conveniences@sidebar-width`. "The width changes abruptly
between the state where a node is selected and the one where only the slide is selected, which is
inconvenient, as it resizes the render area upon clicking on a node."

### Predefined styles with an editor, and a custom style per text `##arg-4`

Round 1, owner. Bears on `thread@editor-conveniences@named-styles`,
`thread@editor-conveniences@style-kinds`. "ability to save and edit predef styles for text (used
for things like Title, Subtitle, Legend...), with a style editor view. text nodes select a style or
choose "custom" for a local style only. Each node kind should probably have its own "Style"
object."

### A menu on a right-click selects a parent, removes, or changes the kind `##arg-5`

Round 1, owner. Bears on `thread@editor-conveniences@node-menu`. "context menu on right clicking a
node: allow choosing to select among parents, to remove the clicked child, or to change its content
kind (and style)."

### Content must not be lost by mistake `##arg-6`

Round 1, owner. Bears on `thread@editor-conveniences@removal-guard`. "It is important that a cell
with content cannot be lost "by mistake". At least for text content, which is not stored outside of
cell content (images are not unloaded if no cell reference it AFAIK)."

### A final-result view without presenter mode `##arg-7`

Round 1, owner. Bears on `thread@editor-conveniences@final-preview`. "button in top menu bar to
render in "final result" mode (without the splits or other editor overlay) without switching to
presenter view."

### The empty kind is a text with no characters `##arg-8`

Round 1, owner. Bears on `thread@editor-conveniences@no-empty-kind`. "the empty kind is useless, it
is basically a text content with no character. I think it is just noise in both the gui and the
implementation."

### The current keys are the outline's turned by 90° `##arg-9`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@selection-keys`.
"The current Alt keys are the outline layout turned 90°, not the same thing." / "the current keys
suit a tree drawn top-down like an org chart, and yours suit an indented outline. Whichever drawing
we pick decides the keys."

### Removing a node can already be undone `##arg-10`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@removal-guard`.
"Removing a node can already be undone. `Op::Remove` returns a snapshot of the slide's tree as its
inverse" / "A confirmation only protects against a removal you don't notice, or one whose undo
history is gone (closed window, reloaded page: `issue@rootpoint-app@page-close-loses-edits`)."

### Removing a slide and changing a text to an image also lose content `##arg-11`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@removal-guard`.
"Removing a slide loses all its content, and changing a text area's kind to image throws its text
away."

### egui widens a side panel to its widest row `##arg-12`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@sidebar-width`.
"egui's `default_size` is only "the initial wrapping width"" / "The panel widens to fit its widest
row" / "I read this in the source but have not measured it."

### Right-click works in the browser `##arg-13`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@node-menu`.
"Right-click works in the browser. eframe's web runner stops the browser's own context menu on the
canvas".

### A view the editor shows is a command `##arg-14`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@final-preview`.
"`goal@rootpoint-protocol@commands-drive-everything` says that "every view [the editor] shows, is a
command". So the final-result mode has to be view state in the session, not a flag only the window
holds."

### With no empty kind, a second click on any area that is not an image enters editing `##arg-15`

Round 1, session, a fact of the grounding. Bears on `thread@editor-conveniences@no-empty-kind`.
"Once every new area is text, a second click on any non-image area starts editing. Typing into a
fresh area then needs no "text" button first."

### An outline fits a narrow panel, where an org chart does not `##arg-16`

Round 1, session. Bears on `thread@editor-conveniences@tree-outline`. "always expanded (slides are
small, and collapsing would add state)"; the rival org chart "loses on width. The sample's 3 areas
already need 3 columns, and a 10-area slide does not fit in a 260 pt panel. An outline grows
downward, one row per node."

### Alt with ↑ and ↓ moving one row would make Alt with → redundant `##arg-17`

Round 1, session. Bears on `thread@editor-conveniences@selection-keys`. Rival "Alt+↑ / Alt+↓ move to
the previous or next row of the outline" … "Its cost is that Alt+→ becomes redundant and the
same-depth move you asked for is gone. I propose your shape." / "On the slide itself, Alt+↓ moves
down inside a stacked split and right inside a beside split. The current keys have the same
mismatch, only on the other axis."

### A fixed width with no drag leaves no room for a deep tree `##arg-18`

Round 1, session. Bears on `thread@editor-conveniences@sidebar-width`. Rival "a fixed width with no
drag (`exact_size`). It is simpler, but a deep tree or a long style name could then never get more
room."

### Only the Styles tab changes a named style `##arg-19`

Round 1, session. Bears on `thread@editor-conveniences@named-styles`. "The selection's style
controls never change a named style." / "That split answers the cost
`rejected@rootpoint-model@content-by-reference` names: "editing a shared content changes every
slide that shows it"." / "a second tab of the side panel, "Selection | Styles", so the width stays
constant".

### Presets copied with no link change nothing placed; overrides per field are a cascade `##arg-20`

Round 1, session. Bears on `thread@editor-conveniences@named-styles`. Rival A "presets copied into
the area when picked, with no link" … "changing "Title" later would change nothing already placed,
and that is the point of having named styles." Rival B "per-field overrides" … "every field becomes
optional" … "That is a cascade, which this per-area design avoids. Your "custom" choice already
covers the red title".

### Named styles fit with later rich text `##arg-21`

Round 1, session. Bears on `thread@editor-conveniences@named-styles`. "Named styles also fit with
later rich text (`issue@rootpoint@rich-text-spans`), where a span would override the area's style."

### Names read in place; ids make a rename touch nothing `##arg-22`

Round 1, session. Bears on `thread@editor-conveniences@style-key`. "`Named("Title")` reads in place,
where `Named(21)` sends the reader to the table." The rival "ids from the counter (`t21`) … A rename
then touches nothing else. Both meet the readability criterion … So this is a weighing, and it is
yours to make. My default is names."

### A named image style would save one click `##arg-23`

Round 1, session. Bears on `thread@editor-conveniences@style-kinds`. "The image kind's only option
is `fit`, so a named image style would save one click." / "Splitting styles by kind now would be a
guess. A kind gets named styles when it is added and has options worth sharing."

### `Empty` differs from an empty text only in one arm of the renderer `##arg-24`

Round 1, session. Bears on `thread@editor-conveniences@no-empty-kind`. "17 lines of Rust name
`Empty`" / the rival, keep `Empty`, "has no remaining argument. An empty text draws nothing, as
`Empty` does: the renderer's `Content::Empty => {}` arm is the only difference".

### A migration of version 1, or its refusal `##arg-25`

Round 1, session. Bears on `thread@editor-conveniences@format-migration`. "This comes from
#named-styles and #no-empty-kind. Default: `load_ron` still reads version 1." Q1: "If no, version 1
is simply refused, which saves the migration code and its test."

### The menu selects the clicked node, so it and the panel describe one node `##arg-26`

Round 1, session. Bears on `thread@editor-conveniences@node-menu`. "While the socket holds control,
only "Select" is enabled." / "No split entries: the panel and the keys already do that, and you
didn't ask." / the rival "a menu that does not select the clicked node. Then the menu and the side
panel would describe two different nodes."

### Only the window asks, and undo is the rival `##arg-27`

Round 1, session. Bears on `thread@editor-conveniences@removal-guard`. "Only the window asks. The
protocol's `remove` does not, since an agent cannot answer a dialog, and undo is there for it." The
rival, no dialog, "costs no click on a removal you meant. It fails your requirement only when the
history is gone before you notice. You called that loss "important", so the weighing is yours."

### In the preview, marks you cannot see are worse than clicks that select nothing `##arg-28`

Round 1, session. Bears on `thread@editor-conveniences@final-preview`. "Clicks that select nothing
are better than marks you cannot see." / "The slide list and the side panel keep working, so you can
tune a style and see the clean result as you go." The rival "hide the marks but keep the gestures"
… "a Ctrl-click would then split with no preview line, and a selection would be invisible."

### The owner approves the outline `##arg-29`

Round 2, owner. Bears on `thread@editor-conveniences@tree-outline`. "tree-outline approved, this is
the look I wanted."

### The owner takes the outline keys for now `##arg-30`

Round 2, owner. Bears on `thread@editor-conveniences@selection-keys`. "I have a hard time deciding
on that. I think we can go with my proposal for now, at least it would be relatively consistent
with the tre view (though your up/down moving one line might be a bit more natural). This is not
very hard to change anyway."

### Draggable, and the width does not follow content `##arg-31`

Round 2, owner. Bears on `thread@editor-conveniences@sidebar-width`. "accepted, draggable and width
do not follow content."

### Reference by name, and a fallback with a visible warning `##arg-32`

Round 2, owner. Bears on `thread@editor-conveniences@style-key`,
`thread@editor-conveniences@missing-style-fallback`. "referral by name is important. Also, a
missing style should use a fallback and give a visible warning, not be a hard error."

### The owner defers styles of other kinds `##arg-33`

Round 2, owner. Bears on `thread@editor-conveniences@style-kinds`. "deferred for when it makes more
sense, for images it makes little sense right now with a single option."

### No presentation exists, and no compatibility is kept in initial development `##arg-34`

Round 2, owner. Bears on `thread@editor-conveniences@format-migration`,
`thread@editor-conveniences@no-compat-in-initial-dev`. "there is not a single existing
presentation for now, and as long as the project is still in initial development, I won't be using
it for real. This should be recorded, probably in claude.md. The project does not keep retrocompat
at all while in initial development, it only starts being needed when I rule it"

### The owner withdraws the dialog, for a history that can be read and branched `##arg-35`

Round 2, owner. Bears on `thread@editor-conveniences@removal-guard`. "I withdraw this proposal. You
are right that undo covers loss of content." / "provide more detailed informations about what
history is saved, the ability to fetch something in the history and reapply it (or at least try
to), maybe even keeping tracks of forks in history (do A, undo, do B, undo, redo A). But this can be
kept as a todo issue, not in scope today."

### A click on the slide ends the preview and selects `##arg-36`

Round 2, owner. Bears on `thread@editor-conveniences@final-preview`. "instead of blocking selection,
having a selection (click) exit the preview mode might be more convenient. Hitting Preview exits
all selections, clicking on the slide exits Preview and selects. Intuitively, I think I would use
it this way. I'm not fully certain though."

### A missing style is an exception to the refusal of every breach `##arg-37`

Round 2, session. Bears on `thread@editor-conveniences@missing-style-fallback`. "a document that
breaks a rule is refused (`design@rootpoint-model@validated-documents`). A style reference to a name
the table does not hold would be such a break. So your rule makes style references an explicit
exception" / "No argument conflicts with your rule: a missing font already works this way".

### A style in use cannot be removed, so a missing style comes only from a hand edit `##arg-38`

Round 2, session. Bears on `thread@editor-conveniences@missing-style-fallback`,
`thread@editor-conveniences@style-removal`. "removing a style that areas use is still refused …
So a missing style comes only from editing the file by hand. The rival is to allow the removal and
let warnings show up: undo covers that, but a deliberate removal would then leave warnings to
clear."

### A policy no code states goes in a design home, with a pointer from `CLAUDE.md` `##arg-39`

Round 2, session. Bears on `thread@editor-conveniences@no-compat-in-initial-dev`. "The record's
rules put a policy that no code states in a design home. The fitting one is the root's, as a head
whose ground is your word. The root `CLAUDE.md` would then get one sentence that delivers it to
agents, pointing at that head."

### Under the policy, the version is there so an old file is refused `##arg-40`

Round 2, session. Bears on `thread@editor-conveniences@no-compat-in-initial-dev`,
`thread@editor-conveniences@format-migration`. "`design@rootpoint-model@save-format` … says a later
change of format "is met by a migration". Under this policy the version number at the top of the
file is there so an old file is refused, not misread".

### Undo restores a removal; the history issue belongs to the session `##arg-41`

Round 2, session. Bears on `thread@editor-conveniences@removal-guard`. "The reason that defeats it
is fact 2: undo already restores what a removal took." The history issue goes "in
rootpoint-protocol, because the session holds the undo and redo stacks
(`design@rootpoint-protocol@view-state`)." Fact 2 is
`argument@editor-conveniences@arg-10`.

### The preview's rule lives in the session `##arg-42`

Round 2, session. Bears on `thread@editor-conveniences@final-preview`. "`preview on` deselects.
`select n<id>` turns preview off." / "The same rule then holds for an agent and in a headless test."
/ "A press in preview starts nothing else … You did not see where it would act." / "My rival made a
click in preview do nothing, which costs you an extra click every time." / "What you give up: you
cannot tune a selected area's own custom style while previewing."

### The premortem of round 2 `##arg-43`

Round 2, session. Bears on `thread@editor-conveniences@selection-keys`,
`thread@editor-conveniences@final-preview`, `thread@editor-conveniences@no-empty-kind`,
`thread@editor-conveniences@new-area-style`, `thread@editor-conveniences@missing-style-fallback`,
`thread@editor-conveniences@sidebar-width`, `thread@editor-conveniences@node-menu`. T1 "The keys
still confuse you even with the tree in view"; T2 "A click meant only to look ends the preview"; T3
"A second click on a new, empty area starts editing, and then Delete or the node keys go to the
empty text"; T4 "Inheriting the split area's style mostly gets in the way, for example a Title split
to add a logo"; AC1, AC2 "six states", AC3 "secondary click".

### Structural inconsistencies are refused, cosmetic ones fall back `##arg-44`

Round 3, owner. Bears on `thread@editor-conveniences@missing-style-fallback`,
`thread@editor-conveniences@cosmetic-fallbacks`. "this principle of fallbacks should be generalized.
The important point is to distinguish structural vs cosmetic inconsistencies in documents. When a
cosmetic fallback can be used, it should IMO. When the file has a structural change that would make
it either impossible to render properly, or one that is not obvious to recover from by a user, we
can refuse it."

### Or make sure no breach happens in normal use `##arg-45`

Round 3, owner. Bears on `thread@editor-conveniences@cosmetic-fallbacks`,
`thread@editor-conveniences@no-new-breach`. "Though, maybe we should just make sure that none of
those situation can ever happen under normal usage (without manual file edits), instead of giving a
fallback."

### A removed style's users could take a replacement the author names `##arg-46`

Round 3, owner. Bears on `thread@editor-conveniences@style-removal`. "What would happen when a style
is removed by the user, for nodes that are using it ? An automatic fallback to default ? Maybe the
user can name a style that should be used as replacement."

### Fuzzing through the text protocol, beside the property tests `##arg-47`

Round 3, owner. Bears on `thread@editor-conveniences@no-new-breach`. "later we should start fuzzing
the project. This is gonna help us make sure no user edit can lead to an invalid style. The text
protocol is quite adapted for this I believe. There is already some property based testing in place
too, right ? This is a todo issue too." / "What do you think is best, fallback allowed, or no
fallback and instead thorough testing of invariants, property based and fuzzing ?"

### The owner accepts the homes of the policy `##arg-48`

Round 3, owner. Bears on `thread@editor-conveniences@no-compat-in-initial-dev`. "accepted on
recording locations."

### Property tests exist in the model only `##arg-49`

Round 3, session, a fact. Bears on `thread@editor-conveniences@no-new-breach`. "Property tests
exist, in rootpoint-model only." / "No property test drives the protocol's commands, and nothing is
fuzzed. A search for proptest, quickcheck, arbitrary and libfuzzer finds only the model's tests."

### The renderer already has stand-ins; validation is stricter `##arg-50`

Round 3, session, a fact. Bears on `thread@editor-conveniences@cosmetic-fallbacks`. "A missing image
draws as a grey area and is listed in `missing_images`" / "A font that cannot be found draws in the
built-in sans" / "Validation is stricter than the renderer. `validate` refuses a document whose
area names an asset id the table lacks".

### Fallbacks and tests handle inputs the other cannot reach `##arg-51`

Round 3, session. Bears on `thread@editor-conveniences@cosmetic-fallbacks`,
`thread@editor-conveniences@no-new-breach`. "My answer is both, because each one handles inputs the
other cannot reach." / "Testing … cannot cover a file you or an agent edited by hand, and editing by
hand is a goal (`goal@rootpoint-model@faithful-readable-file`). A fallback covers those files.
Without tests, a fallback would also hide an editor bug behind a warning."

### The test between structural and cosmetic `##arg-52`

Round 3, session. Bears on `thread@editor-conveniences@cosmetic-fallbacks`. Structural: "the layout
or the identity of nodes is undefined. No area rectangle can be computed, an operation could panic
or reuse an id, or a repair would have to guess the author's layout." Cosmetic: "a reference to a
missing target that has a defined stand-in, whose effect stays inside one area's drawing."

### The rejected rival's counterexamples were all structural `##arg-53`

Round 3, session. Bears on `thread@editor-conveniences@cosmetic-fallbacks`. "It does not bring back
`rejected@rootpoint-model@unvalidated-documents`. All three counterexamples that defeated that rival
were structural: a missing divider, a lowered id counter, and dividers rounded onto each other."

### A custom copy is the replacement that changes no slide `##arg-54`

Round 3, session. Bears on `thread@editor-conveniences@style-removal`. "`style remove "Title"
into=custom` gives each user a custom copy of the removed style, so nothing changes on screen." /
"The default is the custom copy, because it is the only choice that changes no slide. This dialog
is a choice that has to be made, not a confirmation, so it does not conflict with why #removal-guard
was withdrawn."

### Commands refuse a missing name; operations do not `##arg-55`

Round 3, session. Bears on `thread@editor-conveniences@no-new-breach`. "Operations do not check it.
An undo can bring back a missing style that a file loaded with, and the undo must not be refused." /
the property test "in rootpoint-protocol … never produces a cosmetic breach" / fuzzing "needs a
nightly toolchain"; AC4.

### The owner approves the three threads and the premortem `##arg-56`

Round 4, owner. Bears on `thread@editor-conveniences@cosmetic-fallbacks`,
`thread@editor-conveniences@style-removal`, `thread@editor-conveniences@no-new-breach`, the
premortem. "Approved all three, record T1–T4 and AC1–AC4." / "Record the issue not about just
fuzzing, but more generally about the thorough testing that needs to be setup and a few open
proposal of common approaches (including widening property based tests)."

### The preview was read as confirmed `##arg-57`

Round 4, session. Bears on `thread@editor-conveniences@final-preview`. "you have not named
#final-preview yet, and I read your reply as confirming it. The plan carries it as a default, and
you rule on it before the merge."

### The owner approves the preview `##arg-58`

Round 5, owner. Bears on `thread@editor-conveniences@final-preview`. "final-preview accepted too"

## New names, in one place

Illustrations of the shapes, not authority: each slice's audit reads them against the code.

```
crates/rootpoint-model/src/document.rs
  Document::styles            the style table: name → TextStyle             (#styles-model, landed)
  StyleChoice                 enum: Named(String) | Custom(TextStyle)        (#styles-model, landed)
  TextContent::style          becomes a StyleChoice                          (#styles-model, landed)
  Document::resolve_style     the TextStyle a StyleChoice draws with         (#styles-model, landed)
  Document::breaches          the cosmetic breaches of a document            (#breach-tiers, landed)
  Breach                      enum: MissingAsset { area, asset } | UnknownFont { area, name }
                              | MissingStyle { area, name }
                                                                             (#breach-tiers, landed; MissingStyle: #styles-model, landed)
  Content::Empty              removed                                        (#empty-text, landed)
crates/rootpoint-model/src/ops.rs
  Op::AddStyle, Op::SetStyle, Op::RenameStyle, Op::RemoveStyle, Op::RestoreStyle (#styles-model, landed)
crates/rootpoint-model/src/file.rs
  FORMAT_VERSION              existing; 2 since #empty-text, 3 since #styles-model
crates/rootpoint-render/src/lib.rs
  Report::missing_styles      (area, name) of each text drawn with the default style (#styles-model, landed)
crates/rootpoint-protocol/src/command.rs
  Command::Style(StyleCommand) style add | set | rename | remove             (#styles-model, landed)
  Command::Styles             lists the styles                               (#styles-model, landed)
  Command::Preview { on }     preview on | off                               (`spec@editor-conveniences@preview-mode`)
  StyleOption::Named, StyleOption::Custom   text n3 style="Title" | custom   (#styles-model, landed)
crates/rootpoint-protocol/src/session.rs
  View::preview               bool                                           (`spec@editor-conveniences@preview-mode`)
crates/rootpoint-protocol/tests/properties.rs   the property test of #no-new-breach (#breach-tiers, landed)
crates/rootpoint-app/src/panels.rs
  App::outline                the side panel's tree                          (#side-panel, landed)
  App::styles_tab             the Styles tab                                 (#styles-window, landed)
crates/rootpoint-app/src/menu.rs
  App::node_menu              the menu of a right-click                      (#context-menu, landed)
```

## Decided design

The shape of each decided thread is in the spec of the slice that builds it, as each thread's item
in "Threads" says; the shape of a thread whose slice landed is in the design heads its harvest
wrote, which the thread's item names. `thread@editor-conveniences@no-compat-in-initial-dev` is
recorded already.

## Mapping tables

Each mapping table is in the spec of the slice that needs it. The map of every check of
`Document::validate` left with the slice #breach-tiers, and the map of every site of
`TextContent::style` with #styles-model.

## Losing alternatives

- **`thread@editor-conveniences@format-migration`**, ruled out: a loader of the previous format
  version. It lost to the owner's answer that no presentation exists, and to
  `design@rootpoint@no-compat-in-initial-dev`.
- **`thread@editor-conveniences@removal-guard`**, withdrawn by the owner: a dialog before a removal,
  a slide's removal or a change of kind that discards content. Defeating reason: `Op::Remove`
  returns a snapshot of the slide's tree as its inverse, so undo restores what a removal took.
  Replaced by `issue@rootpoint-protocol@history-browsing`.
- **An org chart of the tree** lost to `thread@editor-conveniences@tree-outline`: it needs one column
  per area, and a 10-area slide does not fit in a 260 pt panel.
- **Alt with ↑ and ↓ moving one row of the outline** lost to
  `thread@editor-conveniences@selection-keys` on the owner's choice: it makes Alt with → redundant
  and drops the move at one depth. The owner found it "a bit more natural", and T1 names it as a
  response.
- **A fixed width with no drag** lost to `thread@editor-conveniences@sidebar-width`: a deep tree or a
  long style name could never get more room.
- **Presets copied with no link** and **overrides per field** lost to
  `thread@editor-conveniences@named-styles`: the first changes nothing placed when a style changes;
  the second makes every field optional and needs a cascade the per-area design avoids.
- **A menu that does not select the clicked node** lost to `thread@editor-conveniences@node-menu`:
  the menu and the side panel would describe two nodes.
- **A preview where a click does nothing** lost to `thread@editor-conveniences@final-preview`: it
  costs a click to leave every time. **A preview that hides the marks and keeps the gestures** lost
  too: a Ctrl-click would split with no preview line, and the selection would be invisible.
- **Removing a style in use, leaving its users with the default style** lost to
  `thread@editor-conveniences@style-removal`: a deliberate removal would leave warnings to clear.
- **A fallback with no test**, or **tests with no fallback**, lost to
  `thread@editor-conveniences@cosmetic-fallbacks` with `thread@editor-conveniences@no-new-breach`:
  tests cannot reach a file edited by hand, which `goal@rootpoint-model@faithful-readable-file`
  makes a purpose, and a fallback alone hides an editor bug behind a warning.

## Readings

Empty: the work reads no external specification. The behaviours of egui 0.37.0 and eframe 0.37.0
it relies on, read in their sources, are in "What the work is", and the panel's sizing, since
#side-panel, in `design@rootpoint-app@sidebar-width`.

## Premortem

The premortem assumed the work shipped and failed. Each cause, the thread it stresses, and its
verdict. The owner ruled in round 4: "record T1–T4 and AC1–AC4."

- **T1**, `thread@editor-conveniences@selection-keys`: the keys still confuse the owner with the
  tree in view. A tripwire, recorded on the owner's word at the harvest of #side-panel as
  `tripwire@rootpoint-app@keys-confuse-with-the-outline`, guarding
  `design@rootpoint-app@selection-keys`, the head of the keys the repair of the slice's review
  before the merge cut out of `design@rootpoint-app@node-selection`.
  - Fires when: the owner reports, more than once after the outline lands, that Alt with an arrow
    moved the selection where the owner did not expect.
  - Response: reopen `design@rootpoint-app@selection-keys`, toward the spatial keys, or toward Alt
    with ↑ and ↓ moving one row of the outline. The spatial keys, as the closed issue named them:
    Alt with an arrow selects the neighbouring area in that direction, as i3's `focus left`,
    `focus right`, `focus up` and `focus down` do, with the parent and the child on other keys, such
    as Alt+Page Up and Alt+Page Down.
  - What was seen before this work, measured with no window by driving `App::frame` on the sample's
    first slide: from n2, Alt+↓ did nothing and Alt+→ selected n6, the whole bottom row; from n6,
    Alt+↓ selected n3; from n4, Alt+↓ selected the child last selected in it. The owner first
    suspected a link with leaving a text with Esc; the measurement showed none, since Esc leaves the
    selection where it was. A key that moves the selection where the author did not expect costs a
    look and another key each time, against `goal@rootpoint-app@direct-manipulation` and
    `goal@rootpoint@free-slide-division`.
  - Re-entry: the design audit of the next work on the app's keys, or earlier on the owner's word.
- **T2**, `thread@editor-conveniences@final-preview`: a click meant only to look ends the preview.
  A tripwire, recorded at the harvest of `spec@editor-conveniences@preview-mode`, guarding
  `design@rootpoint-protocol@view-state` as that slice rewrites it, with the rule that a click on the
  slide ends the preview.
  - Fires when: the owner reports, more than once, leaving the preview by a click the owner did not
    mean as an edit.
  - Response: reopen `design@rootpoint-protocol@view-state`, toward a click in the preview that does
    nothing.
  - Re-entry: the standing-state review before every merge.
- **T3**, `thread@editor-conveniences@no-empty-kind`: a second click on a new, empty area enters
  editing, and Delete or the node keys then go to the empty text instead of acting on the area. A
  tripwire, recorded at the harvest of #empty-text as `tripwire@rootpoint-app@empty-text-takes-the-keys`,
  guarding `design@rootpoint-app@edit-entry`.
  - Fires when: the owner reports this more than once.
  - Response: reopen `design@rootpoint-app@edit-entry`, toward an empty text that enters editing
    only when the author types.
  - Re-entry: the standing-state review before every merge.
- **T4**, `thread@editor-conveniences@new-area-style`: inheriting the split area's style mostly gets
  in the way, as when a Title is split to add a logo. A tripwire, recorded at the harvest of
  #styles-model as `tripwire@rootpoint-model@inherited-style-in-the-way`, guarding
  `design@rootpoint-model@new-area-style`.
  - Fires when: the owner reports changing the new area's style right after most splits of a styled
    area.
  - Response: reopen that head, toward a new area that always takes Body.
  - Re-entry: the standing-state review before every merge.
- **AC1**, `thread@editor-conveniences@missing-style-fallback`: a missing style drawn silently in
  the default style. An acceptance criterion, `acceptance@editor-conveniences@ac-missing-style-warned`.
- **AC2**, `thread@editor-conveniences@sidebar-width`: some row still widens the side panel. An
  acceptance criterion, `acceptance@editor-conveniences@ac-panel-width-constant`.
- **AC3**, `thread@editor-conveniences@node-menu`: a right-click that does not select, or opens no
  menu. An acceptance criterion, `acceptance@editor-conveniences@ac-right-click-menu`.
- **AC4**, `thread@editor-conveniences@no-new-breach`: a command that leaves a reference to a missing
  target. An acceptance criterion; it fired at #styles-model and left the document, D4.

## Acceptance criteria

### A document naming a missing style loads, draws in the default style, and is reported in three places `##ac-missing-style-warned`

AC1. Guards `thread@editor-conveniences@missing-style-fallback` and
`thread@editor-conveniences@cosmetic-fallbacks`. Judged by #styles-window,
when the outline exists, and not fired there. Observable: a test loads a document whose text names a style the table
lacks; it fires when the document is refused, when the text draws with any style but
`TextStyle::default()`, or when the status line, the outline's row of that area or the reply of
`render` does not name the missing style. Response: reopen
`thread@editor-conveniences@cosmetic-fallbacks` for styles.

### The side panel's width is the same in every state of its content `##ac-panel-width-constant`

AC2. Guards `thread@editor-conveniences@sidebar-width`. Judged by
#styles-window, the last slice that adds content to the side panel, and not fired there;
#side-panel landed the test, in `path@rootpoint-app@tests/panel.rs`. Observable: a test of `App::frame` reads the
side panel's width, two frames after each change, in six states: the slide alone selected, a text
area, an image area, the Styles tab, a text of 200 characters, and a tree 6 levels deep. It fires
when any two widths differ. The six states are the ones the owner approved. Response: reopen
`thread@editor-conveniences@sidebar-width`, toward a fixed width.

### A secondary click on an area or an outline row selects it and opens the node menu `##ac-right-click-menu`

AC3. Guards `thread@editor-conveniences@node-menu`. Judged by
#context-menu, and not fired there. Observable: two tests of `App::frame`, one with a secondary
click on an area of the slide, one on a row of the outline; each fires when the view's selection is
not that node after the click, or when the menu is not open on the next frame. Response: reopen
`thread@editor-conveniences@node-menu`.

## Implementation sequence

1. Content::Empty removed, #empty-text: landed.
2. Structural and cosmetic breaches, #breach-tiers: landed.
3. Named styles in the model and the protocol, #styles-model: landed.
4. The outline, the node keys and the panel's width, #side-panel: landed.
5. Named styles in the window, #styles-window: landed.
6. The node menu, #context-menu: landed.
7. [The preview](preview-mode.md)

## Order rationale

- #empty-text before #breach-tiers: the breaches are defined over the content kinds that remain.
- #breach-tiers before #styles-model: a missing style is a cosmetic breach, and #styles-model
  widens the property test #breach-tiers landed.
- #styles-model before #side-panel: the outline shows each text's style name.
- #side-panel before #styles-window: the Styles tab is a tab of the side panel whose width
  #side-panel fixes, and AC1 needs the outline's rows.
- #styles-window before #context-menu: the menu's "Style" entry offers the named styles the window
  shows.
- #context-menu before `spec@editor-conveniences@preview-mode`: the two are independent; the preview goes last, since it changes
  the view state every other slice reads.

## Defaults awaiting the owner

- **D1**, the plan's form, all threads: a milestone of seven slices, one branch each, rather than
  one spec. The session's reason: the work changes rootpoint-app in all seven slices,
  rootpoint-protocol in five, rootpoint-model in three and rootpoint-render in two, in seven pieces
  each of which can fail alone, and one branch of all of it is too large to review. In the event,
  rootpoint-render changed in #empty-text, #breach-tiers and #styles-model. Ruled: the
  owner approved it. The owner later ruled that every slice after #empty-text is built on one
  branch, which "How the work is done" states: the slices stay, and share one branch. Each slice's
  work and harvest are still reviewed apart, so no review reads the whole branch at once but the
  review before the merge, which reads it again after each slice.
- **D2**, `thread@editor-conveniences@named-styles`, `thread@editor-conveniences@node-menu`: which
  states of the window `goal@rootpoint-protocol@commands-drive-everything` counts as a "view it
  shows". The design-conformance review found that the Styles tab, the style selected in its list,
  the node menu and the dialog of a style's removal are states only the window holds, while
  `argument@editor-conveniences@arg-14` applies the goal to the preview. Default, built as written:
  a view is what the editor shows of the presentation, the slide shown, the selection, presenter
  mode and the preview; a widget's own state, a tab, a list's selection, an open menu or dialog, a
  hover, is not, as the outline's hover and the divider highlight are not today. The rival: each of
  these states becomes view state of the session, with a command. Ruled by the owner: the owner
  approved a rewording of the goal that states reading (a), a view of the presentation, and excludes
  the window's own state; the default stands as built.

- **D3**, `thread@editor-conveniences@new-area-style`: a new area takes Body only while the table
  holds Body and Body draws with no stand-in, its font an asset of the table or a family of
  `BUILTIN_FAMILIES`, and the default values otherwise. The review of the work of #styles-model
  found that a file edited by hand, whose Body names a font asset the table lacks and which no text
  uses, got a breach from `add-slide`, against `design@rootpoint-protocol@no-new-breach`; and the
  transcript review of the slice found that a table without Body, which `style remove "Body"
  into="Title"` leaves, would give a new area `Named("Body")`, a breach a command makes. The
  condition narrows the rule the owner approved, "A new area holds an empty text, with the style of
  the area it was split from, or Body", for those cases alone, so it is a scope change. Default,
  built as written: the condition. The rival: Body always, and a breach a command makes from such
  a document. Awaiting the owner. Until the ruling, the head `design@rootpoint-model@new-area-style`
  states the rule as approved, and the code builds the condition; the ruling is harvested into the
  head, or the code follows the head.
- **D4**, `thread@editor-conveniences@no-new-breach`: AC4 fired. Its observable, a sequence of
  commands from a document with no breach that leaves a breach, was reproduced by the review of the
  work of #styles-model: `add-slide` on a file whose unused Body names a missing font asset. The
  landing of #styles-model had reported AC4 as not fired, from its property test, which starts from
  an empty document and never meets such a file; that report is superseded. AC4 left the
  document. Its response, reopening #no-new-breach and opening a defect for the command, stands on
  the owner's word. Default: the response is spent, since the decision holds and the command is
  repaired by D3; the instrument's gap, no property test starting from a file edited by hand, is
  named in `issue@rootpoint@thorough-testing`. The rival: reopen #no-new-breach, which goes
  through `skill@knowledge-architect-design`. Awaiting the owner.
- **D5**, `thread@editor-conveniences@new-area-style`: a split of a text area naming a style the
  table lacks does not copy the name; its new area takes the style a new area takes. The owner
  approved "the style of the area it was split from"; the spec added the condition, so that no
  split spreads a breach a file brought, which narrows the approved rule. Default, built as
  written: the condition. The rival: the name copied, as `duplicate-slide` copies a breach.
  Awaiting the owner; until the ruling, the head states the rule as approved.
- **D6**, `thread@editor-conveniences@node-menu` and `thread@editor-conveniences@named-styles`: the
  firing clause of `tripwire@rootpoint-app@control-ends-a-gesture`, T6 of the first prototype,
  recorded on the owner's word, named a stay in editing and a drag. The socket's `control` now also
  closes the dialog of a style's removal as a gesture ends, and closes the node menu of
  #context-menu the same way, so a report that `control` closed one of
  them in the middle of a choice would not fire it. Default: the clause names a dialog or a menu of
  the window too. The rival: the clause as the owner ruled it, a dialog or a menu closed being no
  loss of work. Awaiting the owner; until the ruling, the tripwire states the clause as the owner
  ruled it, since it is recorded on the owner's word, as the head of D3 and D5 states the rule as
  approved.
- **D7**, `thread@editor-conveniences@node-menu`: a secondary click inside the area being edited.
  `design@rootpoint-app@edit-entry` states, on the owner's word, that a press outside the edited
  area leaves editing and acts as it does outside editing; a press of the secondary button inside
  the area is in no ruling, and the thread's approval did not name it. Default, built as written by
  #context-menu: the click leaves editing, then selects and opens the node menu. While editing, the
  keys go to the text, so with the stay open Delete would delete characters while the menu names
  the area. The rival: the stay stays open under the menu, so that closing the menu with no entry
  returns to the text where it was; any entry would still end the stay, since every action of the
  window ends the gestures first. Awaiting the owner; until the ruling,
  `design@rootpoint-app@edit-entry` does not state the case.

## Harvest

This document's own row: when the last slice, `spec@editor-conveniences@preview-mode`, lands, its harvest commit deletes this
document, after checking that every slice's row is done, that each acceptance criterion was reported
on, and that T1 to T4 are written. Each slice's row is in its spec, and is done once the slice's
spec leaves.

## Later consequences

- `issue@rootpoint-protocol@history-browsing` builds on the undo stacks this work leaves unchanged.
- `issue@rootpoint@thorough-testing` widens the property test this work adds.
- A new content kind, `issue@rootpoint@more-content-kinds`, decides whether it shares the text
  styles, by `thread@editor-conveniences@style-kinds`.
- Rich text, `issue@rootpoint@rich-text-spans`, puts spans over an area's named or custom style.
- Compatibility is needed from the owner's ruling that real use has begun,
  `design@rootpoint@no-compat-in-initial-dev`; how it is kept then is not decided.
