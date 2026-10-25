# Milestone: rich text, edited through an overlay on the text

## Status and audience

This is the milestone document of styled text in rootpoint: an area's text becomes a tree of nodes,
each child carrying a partial style or naming a span style; the caret moves through the tree as
LyX's does, a child's edge being one caret position; frames over the slide show the nodes that hold
the caret; and the protocol, the file and the renderer carry the tree. Maths is not built; the tree
admits it as a node kind.

It is written for the sessions that implement its slices, which did not witness the discussion that
produced it. Each slice's spec leaves when that slice lands; this document leaves in the commit that
completes its own harvest, once every slice has landed and the owner has ruled on its defaults.

- Where this document, or a slice's spec, and a design home disagree on a point that "What is
  already decided" does not list as reversed or rewritten, the design home wins.
- Every name it uses is defined in it, in "Names" or "New names, in one place", or exists in the
  code.
- Where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says what
  waits. The defaults awaiting the owner are listed in their own section.
- The milestone was assembled from the transcript of one session. The discussion begins in it at
  the owner's message that opens "I want to discuss adding rich text editing to the project".
- The survey of LyX that grounded the discussion is
  `path@rootpoint@docs/grounding/lyx-nested-text.md`; read it before the slices on the caret and
  the frames.

**The owner's messages of the discussion**, verbatim, in order, up to its convergence. Round n is
the owner's message n and the session's reply to it.

**Round 1**

> I want to discuss adding rich text editing to the project, for which I have an idea: going through a sort of "editor overlay" shown directly on the text, rather than the usual ways which are full WYSIWYG of Word/powerpoint or the full WYMIWYG of markup languages. One of my inspirations for this project is lyx (a layer over latex), which to me is just the right balance of  WYSIWYG and WYMIWYG. It shows content that is very close to the end result, but with just the right amount of informations added to it so that it is easy to distinguish how style applies to content and how to edit it (including via gui interaction and not just markup).
> The lyx style of editing latex equations inline with characters drawn in WYSIWYG way is the main inspiration that we should take IMO. Quite funnily, I believe it works using a tree structure just like our slide layouts. The text is represented as a tree, with the ability to split a node into childs upon which styles are applied. Navigating the text with arrows enters one layer deeper into the tree when there is a child starting at cursor position (like an invisible character that would mark the opening of a child). Lyx shows discrete blue frames around each text node in the parents of the cursor location, so the user can see how deep the cursor is and edit the local style, or open a new child.
> At least this is how I imagine the implementation as a user. Since it is open source IIRC, maybe we can look into how it actually does it.
>
> If we do it this way, I believe this would also pave the way toward allowing writing equations/math

**Round 2**

> text-node-tree: agreed on A. another point I want to add: in most rich text editors, when you type letters at the end of a bold word, you type bold letters (and keep any other styling) without the ability to chose, instead you have to type then manuallly change the style of what you typed. i want it to be possible to exit the style block at any place (or even split it).
>
> text-tree-normal-form: agreed
>
> span-style-overrides: agreed
> the sub question: yes, i want it possible.
> in your shape, each span overrides only a single value of the style ? instead, i think a span should get a full style, but with a special "empty" value for any item. the goal is to avoid the need for multiple span to set mutliple overides. th, i'm not sure this is needed. it is rarely needed to change multiple style settings at once in the middle of text, and the possibility of naming a style covers its use cases IMO.
>
> boundary-caret-positions: agreed. the possibility of choosing explicitely is the key feature (and i was asking for it just above). in practice, there is never a place with lots of edges that meet. those tree rarely go very deep (except for math, but then latex has the same problem)
>
> ancestor-frames: we will start with that, but probably iterate on it once I get a live view of it. if we cannot have it work with a pure overlay, i might consider relaxing the goal. But I believe this has good chances of working out.
>
> text-node-ids: agreed
>
> text-edit-commands: agreed. goal@rootpoint@no-language-no-build might need clarification that it applies to the edit via gui. the text protocol is a language after all, but not one that users need to master.
>
> selection-one-level: agreed. this is actually a feature IMO, more than a cost. I always found the lyx selection model very convenient. you pretty much never need to "cut" a style node in half in practice.
>
> inline-math-node: agreed, shape to admit math.
>
> Q1: always
> Q2: both

**Round 3**

> about q1: What I'm considering is to have inline math available, but also an equaion kind, which would hold only a full eauation, and would also be numbered and referenceable (while inline math would not allow that). I think this separation of responsibility/ use case will make it easier for both users and for implementation, though this is only an intuition
>
> split-at-caret: agreed, and ctrl+space looks fine for shortcut.
>
> named-span-styles: A looks fine, and your arguments convinced me. let's go with that. if users find it unintuitive, we will rethink the shape later.
>
> goal-gui-authoring: agreed on wording

**Round 4**, given against the checkpoint table of round 3 and its premortem:

> table confirmed, keep all tripwires and ac

The questions of round 1 the owner answers in round 2 were: Q1, how often the owner's talks need
maths inside a sentence against a formula shown alone; Q2, whether opening a new child means
creating an empty child at the caret, applying a style to a selection, or both. The answer "Q1:
always" was read in round 2 as "a formula shown alone is a text holding one maths node"; round 3
corrected that reading, and the session withdrew it.

**The checkpoint table of round 3**, against which the owner's word of round 4 was given, verbatim;
the premortem it was given against, with its labels and their thresholds, is the section "Premortem"
below, as round 3 put it:

| thread | state | position |
| --- | --- | --- |
| #text-node-tree | approved | a tree of nodes with partial styles |
| #text-tree-normal-form | approved | dissolve no-op and empty children; keep equal siblings apart |
| #span-style-overrides | approved | area keeps its full style; a span sets any text option, each optional |
| #boundary-caret-positions | approved | one caret position per child edge; typing goes into the caret's node |
| #ancestor-frames | approved | frames over the slide, no space; ancestor row in the side panel; you will iterate when you see it live |
| #text-node-ids | approved | stored ids `t<n>`; a range only when creating a child |
| #text-edit-commands | approved | small `Op`s for the editor; `text` takes a whole tree on one line for agents |
| #selection-one-level | approved | a range of one node; children taken whole |
| #inline-math-node | approved | the tree admits `Math` (and later a reference); none built; the equation is an area kind of its own |
| #style-action-target | approved | no selection: open an empty child; selection: wrap it |
| #split-at-caret | approved | split the innermost node at the caret, caret between; Ctrl+Space and a panel button |
| #named-span-styles | approved | a second table of span styles with optional fields; you will rethink it if users find it unintuitive |
| #goal-gui-authoring | approved | written in [the commit "Say that the goal of authoring with no language concerns the editor", named here by its subject where round 3 gave its SHA] |

The reviews of this document found one consequence of that table that the owner did not see: the
dissolution of "no-op" children removes named spans, D6.

## How the work is done

Before each slice, read `skill@knowledge-architect-planning@working-a-slice` whole, and follow it.
The slices are built in the order of "Implementation sequence", each on its own branch, merged
before the next begins, D2.

- **Gates.** `cargo x gates` runs every check a branch owes before it merges,
  `instructions@merge-gates`. Every slice keeps them passing, the gate `wasm` and the browser tests
  included: a test added to `path@rootpoint-app@tests/actions.rs` or
  `path@rootpoint-app@tests/web.rs` builds and passes on the wasm target, as
  `path@rootpoint-app@CLAUDE.md` says.
- **Actions are commands.** Every new action of the window issues protocol commands through
  `App::run`, checked against the same commands on a second session in
  `path@rootpoint-app@tests/actions.rs`, per `design@rootpoint-app@actions-are-commands`. Every new
  command parses back from its printed form, per `design@rootpoint-protocol@protocol-transport`.
- **No compatibility.** The first slice changed the save format to `FORMAT_VERSION` 5, in
  `path@rootpoint-model@src/file.rs`, with no migration, per
  `design@rootpoint@no-compat-in-initial-dev`. It rewrites in the new format the four `.ron` files
  that `find crates -name '*.ron'` lists at the time of writing:
  `path@rootpoint-protocol@sample/sample.ron`, `path@rootpoint-app@tests/data/mixed.ron`,
  `path@rootpoint-model@tests/data/fixture.ron` and
  `path@rootpoint-model@tests/data/fixture-edited-by-agent.ron`. The fixture is written again by
  running its test `the_fixture_round_trips_and_matches_its_committed_file` with
  `ROOTPOINT_BLESS=1`; the others are edited by hand. Two tests hold the format as text too:
  `path@rootpoint-protocol@tests/breaches.rs` writes a text in the old form inline, and
  `path@rootpoint-model@tests/invalid.rs` replaces the fixture's literal `next_id: 8,`, which a
  fixture with spans moves. A change of the protocol's grammar keeps no old command either.
- **This milestone's branch**, `rich-text-design`, carries the reworded
  `goal@rootpoint@no-language-no-build`, the survey of LyX, this document and its slice specs, and
  the closing of `rich-text-spans`, which the milestone schedules. It merges once
  the owner has ruled on D1, D2 and D6, before the first slice, #text-tree, begins.

## Names

- **The session**: `Session` of `path@rootpoint-protocol@src/session.rs`, which holds the document
  and runs every command.
- **An operation**: a variant of `Op` in `path@rootpoint-model@src/ops.rs`, applied by `apply`,
  which returns its inverse, per `design@rootpoint-model@edit-operation-core`.
- **A text**: `TextContent { root: TextNode, style: StyleChoice }` of
  `path@rootpoint-model@src/document.rs`, a tree of characters and spans since #text-tree.
- **A text style**: `TextStyle` of the same file, with `font`, `size`, `bold`, `italic`, `colour`,
  `h_align` and `v_align`, every field set.
- **The style table**: `Document::styles`, the named text styles, by name; the span style table is
  `Document::span_styles`.
- **The text command**: `Command::Text { area, text, style }` of
  `path@rootpoint-protocol@src/command.rs`, which sets an area's whole text.
- **The layout of a text**: `layout_text` of `path@rootpoint-render@src/text.rs`, which builds a
  Parley `Layout<TextBrush>` with `ranged_builder`, each span's values over its byte range and each
  run's colour its brush, since #text-tree.
- **`area_text`**: the public function of `path@rootpoint-render@src/lib.rs` that gives an area's
  `AreaText { layout, origin, map }`, the layout `render_slide` draws and its `TextMap`, per
  `design@rootpoint-render@area-text-layout`.
- **The editor**: `Editing { area, selection, preedit }` of `path@rootpoint-app@src/editing.rs`,
  the text being edited in place, its caret a Parley `Selection` on the layout `area_text` gives.
- **A stay**: a stay in editing a text, from entering editing to leaving it, one undo group per
  `design@rootpoint-app@gesture-groups`.
- **The overlay**: `App::overlay`, which draws the editor's marks with egui over the slide's
  texture, per `design@rootpoint-app@editor-overlay`.
- **A node**, alone, is a node of a slide's tree, `n<n>`, as in the code. **A text node** is the
  root of a text tree or a span: a sequence of characters and spans.
- **A span**: a text node below the root, which has an id and a span's style.
- **A span style**: the values a span can set, `font`, `size`, `bold`, `italic` and `colour`, each
  optional; an entry of the span style table is one.
- **A span's style**: `Named(name)`, a span style of the table by its name, or `Custom(values)`, a
  span style of its own.
- **An offset**: a position in a text node's sequence, counted in characters, Unicode scalar
  values, with each span counting as one item. Parley's index is a byte index; the map of
  `AreaText` gives a caret position's byte index, and #tree-caret builds the other direction.
- **An edge**: the place in a text node's sequence where a span begins or ends. Each edge is a
  caret position of its own, #boundary-caret-positions.
- **A caret position**: a text node and an offset in it.
## What the work is

What exists today, at each site the work touches:

- **The model**, as #text-tree landed it. A text is a tree of runs of characters and spans,
  `design@rootpoint-model@text-node-tree`, with its `StyleChoice`; the operations of the tree and
  of the span style table are built, each undone by the area's former content or by its table
  inverse, `path@rootpoint-model@src/ops.rs`; ids come from `Document::next_id`, one counter, for
  slides, nodes, assets and spans, `design@rootpoint-model@stable-ids`.
- **The file**, as #text-tree landed it. RON, `FORMAT_VERSION` 5; a text is written
  `Text(root: [Chars("…"), Span(…)], style: …)`, and `span_styles` follows `styles`.
- **The renderer**, as #text-tree landed it. `layout_text` lays a text's string out through
  `ranged_builder`, each span's values over its byte range, and each run carries its colour, the
  brush `TextBrush`; `AreaText` carries the `TextMap`; no `InlineBox` is used.
- **The protocol**, as #span-commands landed it. `insert`, `delete`, `wrap`, `unwrap`, `span` and
  `split-span` change a text by one operation each, `design@rootpoint-protocol@text-edit-commands`;
  `text` takes a whole tree on one line, and `tree` prints one,
  `design@rootpoint-protocol@protocol-transport`; `text` with options and no tree changes the area's
  style alone, `design@rootpoint-protocol@text-style-alone`; the `span-style` commands and the
  query `span-styles` change and list the span style table.
- **The editor**, before #tree-caret. Every change of a text is one `text` command with the area's
  whole text as a tree of one run, a style changed from the side panel aside, which drops the
  spans of a text loaded with them, until #tree-caret sends the small commands; no release is made
  between the slices. Every change of a text is so sent, `design@rootpoint-app@edit-in-place`; the caret is a Parley `Selection`, placed again by byte
  index after each command; the caret, the selection and the composing range are editor state,
  `design@rootpoint-app@caret-editor-state`.

**Outside the work, and why:**

- **Maths.** The tree admits a node kind for it, #inline-math-node; laying out, editing and
  drawing maths is the design of `issue@rootpoint@more-content-kinds`, which also records the
  owner's separate equation kind, numbered and referenceable.
- **A reference to an equation inside a text.** A node kind the tree admits, #inline-math-node,
  designed with the equation kind.
- **Copy and paste of styled text between texts.** A paste inserts plain text, D3; carrying spans
  through the clipboard is a later consequence.
- **Reconciling concurrent edits.** Outside `goal@rootpoint@minimal-content-set`.
- **Spans in the outline.** The outline row of a text keeps its style's name and the start of its
  text, `design@rootpoint-app@tree-outline`; the text's nodes are not listed there.

## What is already decided

**The decisions the design rests on and does not argue again:**

- `goal@rootpoint-app@faithful-slide-view` and `design@rootpoint-app@editor-overlay`: no editor
  mark in the slide's image, every mark drawn over the texture.
- `design@rootpoint-render@slide-renderer` and `rejected@rootpoint-render@toolkit-painter`: the
  project's renderer draws every text, with Parley; no toolkit draws slide text.
- `rejected@rootpoint-app@plain-editor`: no Parley editor that owns its own layout.
- `rejected@rootpoint-app@caret-command`: no command places or reads the caret or the selection;
  `insert`, `delete` and `wrap` take an offset or a range read at the moment of the command, and
  place no caret, so the rejected design does not come back.
- `rejected@rootpoint-model@gui-mutation`: the editor changes a text tree only by commands, each an
  operation, #text-edit-commands, so the rejected design does not come back.
- `design@rootpoint-model@edit-operation-core`, `design@rootpoint-model@validated-documents`:
  every change is an `Op` with its inverse, and a structural breach is refused.
- `rejected@rootpoint-protocol@json-rpc-commands`: no JSON payload.
- `design@rootpoint-protocol@gesture-undo`, `rejected@rootpoint-protocol@merged-commands`: a
  gesture is one undo group, and the session merges nothing.
- `design@rootpoint-model@style-key`, `design@rootpoint-model@style-removal`: styles are named by
  name, and a removal gives the users a replacement. The span style table follows both,
  #named-span-styles.
- `rejected@rootpoint-model@addressing-by-position`: no node is named by a position across edits.
- `design@rootpoint-app@caret-editor-state`, on the owner's word: the caret and the selection are
  the editor's state, with no command. A caret position names a text node and an offset, so the
  caret's text node is part of the caret, D5.
- `design@rootpoint@no-compat-in-initial-dev`: the format and the grammar change with no migration.
- `design@rootpoint-app@editor-keys-kept-from-browser`: no editor key reaches the browser's own
  action; the new keys are members of that rule.

**The recorded decisions the work reverses or rewrites**, with every text `cargo klarch show`
lists as referencing each, and the slice whose harvest judges or updates it:

| decision | change | referencing texts | judged at |
| --- | --- | --- | --- |
| `text-style-per-area`, now `design@rootpoint-model@text-node-tree` | reversed: a text is a tree with spans over the area's style; its ground was the owner's word that one style per area was enough "for the first prototype" | the issue `rich-text-spans`, closed by this milestone's branch; the doc comment of `TextContent`, `path@rootpoint-model@src/document.rs` line 306, which states the incumbent without naming it | #text-tree |
| `design@rootpoint-model@named-styles` | rewritten: the area's style stays one full style, named or custom; per-field overrides are rejected for areas and are the shape of a span, whose cascade is its purpose, #span-style-overrides; spans name a second table, #named-span-styles | `register@rootpoint-app@design` line 203, `register@rootpoint-model@design` line 77, `register@rootpoint-model@design` line 154, `path@rootpoint-protocol@src/session.rs` line 1231, the issue `rich-text-spans`, closed | #text-tree, the comment of `session.rs` at #span-commands |
| `design@rootpoint-model@inline-content-assets` | rewritten: "a text with its style" becomes a text tree with its style, and the style table is no longer "the one part of a content shared by name", the span style table being another | referenced by nothing | #text-tree |
| `design@rootpoint-model@style-kinds` | rewritten: its body's one style table of text styles becomes two, the text styles and the span styles, both for text | `issue@rootpoint@more-content-kinds`, its section "What would close it" | #text-tree |
| `design@rootpoint-model@validated-documents` | members added: a span id unique in the document and below the counter, and a span's or a span style's font asset id below the counter, structural breaches | `register@rootpoint-model@rejected` line 112, `path@rootpoint-model@src/ops.rs` line 177, `path@rootpoint-model@src/validate.rs` lines 2 and 132 | #text-tree |
| `design@rootpoint-model@stable-ids` | rewritten: spans get ids from the one counter, printed `t<n>` | `path@rootpoint-model@docs/open-issues/linked-dividers.md` line 24, `path@rootpoint-model@src/document.rs` line 48, `path@rootpoint-model@src/ops.rs` line 374, `register@rootpoint-protocol@design` line 32, `:179`, `:303` | #text-tree; the protocol's lines at #span-commands |
| `design@rootpoint-model@cosmetic-fallbacks` | members added: a span naming a missing span style, or a font asset the table lacks | `register@rootpoint-model@design` line 149, `:194`, `:245`, `path@rootpoint-model@src/document.rs` line 414, `path@rootpoint-model@src/ops.rs` line 300, `:308`, `:320`, `path@rootpoint-model@src/validate.rs` line 3, `register@rootpoint-protocol@design` line 46, `:234`, `:408` | #text-tree |
| `design@rootpoint-render@area-text-layout` | rewritten: `AreaText` carries the layout with a colour per run and the map between caret positions and byte indices | `register@rootpoint-app@design` line 357, `register@rootpoint-app@rejected` line 23 | #text-tree |
| `design@rootpoint-protocol@protocol-addressing` | rewritten: the id kinds gain `t<n>`, and a command that makes a span or changes characters names a text node by its id and takes an offset or a range in it, read at that moment; no command names a span by a position | `register@rootpoint-app@design` line 524 | #span-commands |
| `design@rootpoint-protocol@protocol-transport` | rewritten: `text` takes a whole tree on one line, a new token form of brackets, braces and commas beside the words, the quoted strings and the options; the owner approved that form under #text-edit-commands | `register@rootpoint-app@design` line 440, `register@rootpoint-protocol@rejected` line 47 | #span-commands |
| `design@rootpoint-protocol@no-new-breach` | members added: the span commands that name a span style or a font asset refuse a missing one, and the property test covers them | `path@rootpoint-app@src/actions.rs` line 576, `path@rootpoint-app@src/panels.rs` line 559, `register@rootpoint-model@design` lines 147, 192, 197, 215 and 271, `path@rootpoint-model@src/ops.rs` lines 301, 309 and 372, `path@rootpoint-protocol@src/session.rs` lines 1308 and 1330, `path@rootpoint-protocol@tests/properties.rs` line 6 | #span-commands |
| `design@rootpoint-protocol@asset-commands` | members added: a span's font is a use of an asset, for `assets uses=` and `asset-remove … into=` | `register@rootpoint-app@design` line 217, `register@rootpoint-protocol@design` line 202 | #span-commands |
| `design@rootpoint-app@edit-in-place` | rewritten: a change of the text is one small command, #text-edit-commands, and the caret is a caret position mapped to Parley's byte index | `register@rootpoint-app@design` line 340, `:412`, `path@rootpoint-app@src/editing.rs` line 1, `path@rootpoint-app@src/view.rs` line 351, `path@rootpoint-app@tests/edit_ops.rs` line 4, `path@rootpoint-app@tests/editing.rs` line 4, `register@rootpoint-render@design` line 39, the issue `rich-text-spans`, closed | #tree-caret |
| `design@rootpoint-app@editor-overlay` | members added: the frames of the caret's nodes | `path@rootpoint-app@CLAUDE.md` line 17, `register@rootpoint-app@design` line 108, `:364`, `:549`, `path@rootpoint-app@tests/presenter.rs` line 3 | #span-styling |
| `design@rootpoint-app@gesture-groups` | to be changed, open at the audit of #span-styling: a control of the side panel and the button of the ancestor row act on the caret's span during a stay, where the head ends a stay at any other action; the audit takes it to the design skill, a load-bearing gap, found by the review before the merge of #text-tree | judged at #span-styling's audit | #span-styling |
| `design@rootpoint-app@editor-keys-kept-from-browser` | to be changed, open at the audit of #span-styling: its argument covers keys whose default leaves the page, and Ctrl+B, Ctrl+I and Ctrl+Space do not leave it; the audit takes the new keys to the design skill | judged at #span-styling's audit | #span-styling |
| `design@rootpoint-app@styles-tab-edits-styles` | rewritten: the Styles tab also lists and edits the span styles | `register@rootpoint-app@design` line 223, `path@rootpoint-app@src/panels.rs` line 438, `:484` | #span-styling |

**The standing entries the discussion's search returned, with their outcome:**

- `rich-text-spans`: the subject; closed by the commit that adds this document.
- `issue@rootpoint@more-content-kinds`: bears; its equation kind stays as the owner described in
  round 3; #inline-math-node leaves room for maths in the tree. Its wording is read again at the
  milestone's harvest.
- `editing-actions-outside-actions-rs`: bore on #tree-caret and #span-styling, which rework
  `path@rootpoint-app@src/editing.rs`; closed by #tree-caret, which moved the actions of editing to
  `path@rootpoint-app@src/actions.rs`, as `design@rootpoint-app@actions-are-commands` says.
- `issue@rootpoint-app@abandoned-composition-undo-step`: bears on #tree-caret, AC3.
- `issue@rootpoint@compatibility-after-real-use`: bears on nothing; its trigger is not met, and the
  format changes under `design@rootpoint@no-compat-in-initial-dev`.
- `issue@rootpoint@thorough-testing`: bears; AC1 is a property test of the kind it asks for.
- `issue@rootpoint-protocol@history-browsing`: bears on nothing built here; the descriptions of the
  new undo steps are what it would read.
- `issue@rootpoint-model@slide-templates`, `issue@rootpoint@publish-to-crates-io`,
  `issue@rootpoint-app@wider-lock`, `issue@rootpoint-app@double-click-across-buttons`: bear on
  nothing.
- `tripwire@rootpoint-render@render-within-bounds`: bears; read at the audit of #text-tree, which
  changes the layout of every text.
- `tripwire@rootpoint-render@glyph-missing-everywhere`: bears on nothing built here; bold and
  italic use the built-in sans's four styles, `design@rootpoint-render@builtin-noto-sans`; maths
  glyphs are the equation design's.
- `tripwire@rootpoint-app@keys-confuse-with-the-outline`: bears; its re-entry is the next work on
  the app's keys, so the audits of #tree-caret and #span-styling read it. While editing, every key
  goes to the text, as `design@rootpoint-app@edit-entry` says; outside editing, Alt with an arrow
  stays the node keys.
- `tripwire@rootpoint-app@empty-text-takes-the-keys`: bears on #tree-caret, which keeps
  `design@rootpoint-app@edit-entry`'s entry.
- `tripwire@rootpoint-model@inherited-style-in-the-way`: bears on nothing: a split of a text area
  gives the new area the area's style, `design@rootpoint-model@new-area-style`, and the new area's
  text is empty, with no span.
- `tripwire@rootpoint-app@control-ends-a-gesture`: bears on nothing new: a stay is ended as before.

**The design and `rejected` entries graded `bears` by the design-entry search**, beyond those
above, with their outcome: `design@rootpoint@gui-only-in-app` (the tree and its operations are in
the model), `design@rootpoint@wasm-from-the-start` (no new dependency of a built crate is planned, `proptest` at most among rootpoint-app's
dev-dependencies at #tree-caret; Parley is
already built for the browser), `design@rootpoint@uniform-scaling` (the frames are placed through
`slide_to_pixels`), `design@rootpoint-model@save-format`, `design@rootpoint-model@slide-units`
(a span's size is in slide units), `design@rootpoint-render@caller-supplies-assets` and
`design@rootpoint-render@font-sources` (a span's font comes through the `FontSet`),
`design@rootpoint-render@builtin-noto-sans` (no `MATH` table, measured; maths is outside the work),
`design@rootpoint-protocol@view-state`
(no span is part of the view; the frames are not drawn in the final view),
`design@rootpoint-app@gesture-groups`, `design@rootpoint-app@gui-toolkit`,
`design@rootpoint-app@edit-entry`, `goal@rootpoint-app@live-agent-edits` and
`goal@rootpoint-app@browser-window`, each kept as it stands.

## Criteria

### The slide's image holds no editor mark, and no mark takes layout space `##c-no-mark-no-space`

Binding, from `goal@rootpoint-app@faithful-slide-view` and
`goal@rootpoint-render@same-slide-everywhere`. Met by #ancestor-frames.

### The author styles text with no markup to write `##c-no-markup-for-author`

Binding, from `goal@rootpoint@no-language-no-build`. Met by #boundary-caret-positions,
#split-at-caret and #style-action-target.

### Every edit of a text tree is an operation with its inverse, run headless `##c-headless-ops`

Binding, from `goal@rootpoint-model@headless-changes` and
`design@rootpoint-model@edit-operation-core`. Met by #text-edit-commands and #split-at-caret.

### Every edit is a one-line command, and an agent writes and reads styled text `##c-one-line-commands`

Binding, from `goal@rootpoint-protocol@commands-drive-everything` and
`design@rootpoint-protocol@protocol-transport`. Met by #text-edit-commands and #text-node-ids.

### A text tree saves as readable RON, and loads back equal `##c-readable-tree`

Binding, from `goal@rootpoint-model@faithful-readable-file`. Met by #text-node-tree and
#named-span-styles.

### A span offers only the options of text `##c-text-options-only`

Binding, from `goal@rootpoint@minimal-content-set`. Met by #span-style-overrides.

### The caret sits on the one layout the renderer draws `##c-one-layout`

Binding as a presumption, from `design@rootpoint-render@area-text-layout`. Met by #text-node-tree:
Parley lays the tree out in one layout.

### Inline maths can be added later without changing the text model again `##c-math-admissible`

Weighed, from the owner's aim in round 1. Met by #inline-math-node.

### The editing keys do not collide with the node keys `##c-keys-apart`

Binding as a presumption, from `design@rootpoint-app@edit-entry` and
`design@rootpoint-app@selection-keys`. Met by #boundary-caret-positions and #split-at-caret.

## Threads

### An area's text is a tree of nodes, each child carrying a partial style `##text-node-tree`

The owner proposed the tree in round 1; the session stated it as shape A in round 1, against B,
flat runs of styles over one string, as LyX stores bold, and C, both at once, as LyX does.
Approved in round 2: "text-node-tree: agreed on A." C was ruled out on #arg-1; B lost on #arg-2 and
#arg-3. Its shape is in "Decided design"; its harvest is the model's design home, at #text-tree.

### A custom span that sets nothing is refused, the editor unwraps an empty span, and equal siblings stay apart `##text-tree-normal-form`

Proposed by the session in round 1, on #arg-5. Approved in round 2: "text-tree-normal-form:
agreed". The reviews of this document found that dissolving a child "whose style changes nothing"
dissolves a named span whose style is missing or equal to its parent's values, which loses its
name, against #named-span-styles; the rule is narrowed to a custom span that sets no field, D6,
which the owner accepted: "defaults accepted, proceed". Its harvest is the model's design home, at
#text-tree.

### The area keeps one full style, and a span sets any text option, each optional `##span-style-overrides`

Proposed by the session in round 1, on #arg-6. Approved in round 2: "span-style-overrides: agreed".
In the same message the owner asked whether a span overrides a single value, proposed a full style
with an empty value for any item, and doubted it was needed, on #arg-14; the session answered in
round 2 that the shape is that one, every field optional, several set at once. The owner confirmed
the checkpoint table in round 4, whose line reads "area keeps its full style; a span sets any text
option, each optional": "table confirmed". Its harvest is a head of its own in the model's design
home, under its slug, at #text-tree; `design@rootpoint-model@named-styles` keeps the area's one
full style and is rewritten to say that per-field overrides are the shape of a span.

### A child's edge is a caret position of its own, and typing goes into the caret's node `##boundary-caret-positions`

The owner proposed the "invisible character" in round 1; the session stated it as LyX's rule in
round 1, on #arg-2, read in the survey. Approved in round 2: "boundary-caret-positions: agreed. the
possibility of choosing explicitely is the key feature (and i was asking for it just above). in
practice, there is never a place with lots of edges that meet. those tree rarely go very deep
(except for math, but then latex has the same problem)", #arg-12. Its harvest is the app's design
home, at #tree-caret.

### Frames over the slide show the caret's nodes, taking no space, with the ancestor row in the side panel `##ancestor-frames`

The owner proposed LyX's frames in round 1; the session shaped them in round 1, on #arg-7, and
added the ancestor row, a second decision, of the side panel, harvested on its own. Approved in
round 2: "ancestor-frames: we will start with that, but
probably iterate on it once I get a live view of it. if we cannot have it work with a pure overlay,
i might consider relaxing the goal. But I believe this has good chances of working out." T1 and AC2
watch it. Its harvest: the frames are members of `design@rootpoint-app@editor-overlay`, marks over
the texture, recorded directly at #span-styling, with the reason that they take no space; the
ancestor row is judged by the entry tests there.

### A span has a stored id, printed `t<n>`, and a range is given only to create one `##text-node-ids`

Proposed by the session in round 1, on #arg-8. Approved in round 2: "text-node-ids: agreed". T3
watches its cost in the file. Its harvest is `design@rootpoint-model@stable-ids` and
`design@rootpoint-protocol@protocol-addressing`, at #text-tree and #span-commands.

### The editor changes a tree by small operations, and `text` takes a whole tree on one line `##text-edit-commands`

Proposed by the session in round 1, on #arg-9. Approved in round 2: "text-edit-commands: agreed.
goal@rootpoint@no-language-no-build might need clarification that it applies to the edit via gui.
the text protocol is a language after all, but not one that users need to master.", #arg-15, which
opened #goal-gui-authoring. T2 and AC3 watch it. It holds two decisions, each harvested on its own:
the editor's small commands, against whole-text replacement, #arg-9, in the protocol's design home
under the slug of #text-edit-commands at #span-commands and in `design@rootpoint-app@edit-in-place`
at #tree-caret; and `text` taking a whole tree on one line, against a JSON payload, in the
rewritten `design@rootpoint-protocol@protocol-transport` at #span-commands.

### A selection is a range of one node, children taken whole `##selection-one-level`

Proposed by the session in round 1, on #arg-10. Approved in round 2: "selection-one-level: agreed.
this is actually a feature IMO, more than a cost. I always found the lyx selection model very
convenient. you pretty much never need to "cut" a style node in half in practice.", #arg-16. Its
harvest is the app's design home, at #tree-caret.

### The tree admits node kinds beside the span, maths among them, and builds none `##inline-math-node`

The owner raised maths in round 1; the session proposed in round 1 that the tree admit a `Math`
kind, on #arg-3, #arg-4 and #arg-11. Approved in round 2: "inline-math-node: agreed, shape to admit
math." In round 3 the owner corrected the session's reading of Q1, #arg-20: inline maths and a
separate equation kind, numbered and referenceable; the session noted that a reference to an
equation is inline, #arg-21, a kind the rule admits. Its harvest is the model's design home, at
#text-tree, under a slug that names the decision, the pair `#inline-math-node → text-node-kinds`,
since nothing of maths is built; and `issue@rootpoint@more-content-kinds`, which records the owner's
two forms of maths.

### A style action opens an empty child with no selection, and wraps a selection `##style-action-target`

Asked by the session as Q2 in round 1. Closed by the owner's answer in round 2: "Q2: both". Its
harvest is the app's design home, at #span-styling.

### "Split here" cuts the caret's node at the caret, the caret between the halves, on Ctrl+Space `##split-at-caret`

The owner asked in round 2 to "exit the style block at any place (or even split it)", #arg-13; the
session shaped it in round 2, on #arg-17 and #arg-18. Approved in round 3: "split-at-caret: agreed,
and ctrl+space looks fine for shortcut." #arg-22 checked the key on the owner's machine. Its
harvest: the operation, judged by the entry tests, at #text-tree; the command's name, in the
protocol's design home, at #span-commands; the key and the button, in the app's design home, at
#span-styling.

### A span names a style of a second table, whose fields are each optional `##named-span-styles`

The owner asked in round 2 that a span can name a style, "the sub question: yes, i want it
possible."; the session proposed A in round 2 against B and C, on #arg-19. Approved in round 3:
"named-span-styles: A looks fine, and your arguments convinced me. let's go with that. if users
find it unintuitive, we will rethink the shape later." T4 watches it. Its harvest is a head of its
own in the model's design home, under its slug, at #text-tree, beside the rewritten
`design@rootpoint-model@named-styles`; and `design@rootpoint-app@styles-tab-edits-styles`, at
#span-styling.

### The goal of authoring with no language concerns the editor `##goal-gui-authoring`

Raised by the owner in round 2, #arg-15; the session drafted the wording in round 2 under
`skill@knowledge-architect-goal-setting`. Approved in round 3: "goal-gui-authoring: agreed on
wording". Written on this milestone's branch, in the commit "Say that the goal of authoring with no
language concerns the editor". It needs no harvest.

## Arguments

### LyX's two styling models disagree where they meet `##arg-1`

Round 1, the session, from the survey. Bears on #text-node-tree. LyX stores bold, colour and size
as flat runs and character styles as insets; its tickets 1973 and 6919 are a font change that did
not reach text inside an inset, and `CursorSlice.h` carries a comment wishing the text and maths
implementations unified.

### With flat runs an edge has one caret position, and the typing style is guessed `##arg-2`

Round 1, the session. Bears on #text-node-tree and #boundary-caret-positions. "Between `a` and a
bold `b` there is one caret position", and LyX's `Cursor::setCurrentFont` picks the typing font by
a rule; the two positions the owner described exist only if the child is an item of the sequence.

### Maths needs a tree `##arg-3`

Round 1, the session. Bears on #text-node-tree and #inline-math-node. A fraction has two cells; a
list of runs cannot hold them.

### Parley 0.12 builds a style tree and places inline boxes `##arg-4`

Round 1, the session, read in `path@elsewhere@parley-0.12.0/src/builder.rs` at line 237 and
`path@elsewhere@parley-0.12.0/src/inline_box.rs` at line 8. Bears on #text-node-tree and
#inline-math-node. `TreeBuilder` has `push_style_span` and `pop_style_span`; `InlineBox` has the
fields `id`, `kind`, `index`, `width`, `height`, `baseline` and `vertical_align`, and places a box
of a given size at the byte index `index`. `TreeBuilder::push_inline_box` sets `index` to the length
of the text pushed so far, and `RangedBuilder::push_inline_box` keeps the caller's.

### Equal siblings are a grouping the author made `##arg-5`

Round 1, the session. Bears on #text-tree-normal-form. Merging two bold siblings loses the author's
choice to restyle one of them later, the loss `tripwire@rootpoint-model@grouping-lost-to-merge`
watches in the slide tree; the slide tree's rule of one tree per layout,
`design@rootpoint-model@split-tree-arity`, is applied only to no-op and empty children.

### A full style per child copies the area's values, which then go stale `##arg-6`

Round 1, the session. Bears on #span-style-overrides. A bold word in Body as a custom copy of Body
keeps the old size when Body changes, which defeats the reason named styles exist; the cascade
`design@rootpoint-model@named-styles` rejects for areas is the purpose of a span.

### LyX reserves the space of its marks, which the slide cannot `##arg-7`

Round 1, the session, from the survey. Bears on #ancestor-frames. LyX keeps 2 px around a maths
inset and 4 px around a text inset whether a frame is drawn or not, because its screen is not its
output; `goal@rootpoint-app@faithful-slide-view` rules the space out, so the frames are drawn over
the slide and take none.

### A position shifts at every edit; a range is valid only at the moment it is given `##arg-8`

Round 1, the session. Bears on #text-node-ids. A script that styles a span again after another edit
names it by its id; `rejected@rootpoint-model@addressing-by-position` names the defect of offsets.
The cost is an id per span in the file.

### A whole-text replacement loses every child's id `##arg-9`

Round 1, the session. Bears on #text-edit-commands. Each keystroke sends the area's whole text
today; with a tree, that would give every span a new id at each key.

### A selection in one node keeps every edit a range of one container `##arg-10`

Round 1, the session, from the survey (`CursorData::normalAnchor`). Bears on #selection-one-level.
Bold, cut and paste then never cut a node in half; free selection makes each style action a cut of
several nodes in one operation.

### Noto Sans has no `MATH` table `##arg-11`

Round 1, the session, measured: the table tags of
`path@rootpoint-render@fonts/NotoSans-Regular.ttf` read by a script are DSIG, GDEF, GPOS, GSUB,
OS/2, cmap, cvt, fpgm, gasp, glyf, head, hhea, hmtx, loca, maxp, name, post and prep. Bears on
#inline-math-node: maths will need its own font.

### Choosing explicitly is the key feature, and edges rarely meet `##arg-12`

Round 2, the owner. Bears on #boundary-caret-positions: "the possibility of choosing explicitely is
the key feature", "there is never a place with lots of edges that meet", "those tree rarely go very
deep (except for math, but then latex has the same problem)".

### The author must be able to leave a style anywhere `##arg-13`

Round 2, the owner. Bears on #split-at-caret: "when you type letters at the end of a bold word, you
type bold letters (and keep any other styling) without the ability to chose", "i want it to be
possible to exit the style block at any place (or even split it)".

### Several overrides at once are rare, and a named style covers them `##arg-14`

Round 2, the owner. Bears on #span-style-overrides and #named-span-styles: "it is rarely needed to
change multiple style settings at once in the middle of text, and the possibility of naming a
style covers its use cases IMO".

### The protocol is a language no author needs to master `##arg-15`

Round 2, the owner. Bears on #text-edit-commands and #goal-gui-authoring: "the text protocol is a
language after all, but not one that users need to master".

### LyX's selection model is convenient `##arg-16`

Round 2, the owner. Bears on #selection-one-level: "I always found the lyx selection model very
convenient. you pretty much never need to "cut" a style node in half in practice".

### A toggle to leave has no meaning for a colour or a named span `##arg-17`

Round 2, the session. Bears on #split-at-caret. Ctrl+B pressed in a bold node could split it and go
on unbolded, but no such key exists for a red span or a named span; "split here" acts the same for
every style.

### The free keys, and the keys eframe keeps from the browser `##arg-18`

Round 2, the session. Bears on #split-at-caret. Esc leaves editing,
`design@rootpoint-app@edit-entry`, and the arrows move the caret; outside editing, Alt with an arrow
is the node keys. eframe 0.37.0's `should_prevent_default_for_key`,
`path@elsewhere@eframe-0.37.0/src/web/events.rs` at line 237, prevents the default of Ctrl with
Comma, O, P and S, of Tab, Backspace and the arrows, and of Space while its text agent lacks focus;
while a text is edited the app sets `PlatformOutput::ime`, which focuses the text agent
(`path@elsewhere@eframe-0.37.0/src/web/app_runner.rs` at line 426), so Space and Ctrl+Space keep
their default then. It never prevents Ctrl+B or Ctrl+I. That Firefox gives Ctrl+B to the bookmarks
sidebar and Ctrl+I to page information is the session's knowledge of the browser, not measured.
The style keys and Ctrl+Space then need the capture-phase listener of
`design@rootpoint-app@editor-keys-kept-from-browser`.

### One table of full styles applies every field; one table of optional fields changes areas `##arg-19`

Round 2, the session. Bears on #named-span-styles. B: a span named "Title" would take Title's font,
size, colour and alignment, which means nothing inside a line, and a "Term" inside a Title text
would leave Title's size. C: every area style would become optional field by field, with a
fallback, which `design@rootpoint-model@named-styles` rejects for areas with no new argument for
areas.

### An equation kind of its own, numbered and referenceable, beside inline maths `##arg-20`

Round 3, the owner. Bears on #inline-math-node: "have inline math available, but also an equaion
kind, which would hold only a full eauation, and would also be numbered and referenceable (while
inline math would not allow that)", "though this is only an intuition".

### A reference to an equation is inline `##arg-21`

Round 3, the session. Bears on #inline-math-node. "See (3)" stands inside a sentence, so the tree
will need a reference node beside `Span` and `Math`; the two kinds of maths can share one layout
and differ in what they add, as TeX's inline and display styles do. A lead for the equation's
design, not a decision.

### Ctrl+Space reaches the editor on the owner's machine `##arg-22`

Round 3, the session, measured: `gsettings get org.freedesktop.ibus.general.hotkey triggers` and
`gsettings get org.gnome.desktop.wm.keybindings switch-input-source` both name Super+Space. Bears
on #split-at-caret.

## New names, in one place

An illustration of the shape; each slice's audit settles the exact names against the code.

```text
crates/rootpoint-model/src/document.rs
  TextContent { root: TextNode, style: StyleChoice }   existing type, its `text` field replaced
  TextNode { items: Vec<TextItem> }                    the root and every span's sequence
  TextItem::Chars(String) | TextItem::Span(Span)       a node kind; Math and a reference are later variants
  Span { id: SpanId, style: SpanChoice, text: TextNode }
  SpanId                                               an id of the one counter, printed t<n>
  SpanChoice::Named(String) | SpanChoice::Custom(SpanStyle)
  SpanStyle { font, size, bold, italic, colour }       each an Option
  Document::span_styles: BTreeMap<String, SpanStyle>   the span style table, saved in the file
  TextNodeRef::Root | TextNodeRef::Span(SpanId)        names a text node of an area's text
  CaretPos { node: TextNodeRef, offset: usize }        a caret position
crates/rootpoint-model/src/ops.rs
  Op::InsertText { area, node: TextNodeRef, offset, text: String }
  Op::DeleteText { area, node: TextNodeRef, range }    a range of items; a span in it is taken whole
  Op::WrapSpan { area, node: TextNodeRef, range, style: SpanChoice }   the span's id is allocated by apply
  Op::UnwrapSpan { area, span: SpanId }
  Op::SetSpanStyle { area, span: SpanId, style: SpanChoice }
  Op::SplitSpan { area, span: SpanId, offset }         the second half's id is allocated by apply
  Op::SetText { area, root: TextNode }                 a whole tree; apply gives every span a new id
  Op::AddSpanStyle / SetSpanStyleValues / RenameSpanStyle / RemoveSpanStyle / RestoreSpanStyle
  ops::span_style_users                                the texts that name a span style, as ops::users does for text styles
crates/rootpoint-model/src/validate.rs
  Breach::MissingSpanStyle { area, span, name }        a span naming a span style the table lacks
crates/rootpoint-render/src/lib.rs
  AreaText { layout, origin, map }                     existing type; layout carries a colour per run
  TextMap                                              caret position -> byte index, and each span's byte range
  Report::missing_span_styles                          beside missing_styles; a span's missing font joins font_fallbacks
crates/rootpoint-protocol/src/command.rs
  insert, delete, wrap, unwrap, span, split-span       new commands; text takes a tree
  span-style add | set | rename | remove               the span style table
crates/rootpoint-app/src/editing.rs
  Editing { area, caret: CaretPos, anchor: Option<CaretPos>, preedit }   existing type
```
## Decided design

### #text-node-tree

An area's text is a root text node, a sequence of characters and spans; a span has an id, a span's
style and its own text node. The area's `StyleChoice` stays the style of the root. In RON a text
reads as below, the shape measured at #text-tree's audit with ron 0.12.2: an item is externally
tagged, `Chars("…")` or `Span(…)`, a span style writes only the fields it sets, and the file
declares `implicit_some`, so a field reads `bold: true`. An untagged item, a bare string beside a
span, was measured too: ron writes the span with no name, and a hand-written `Span(…)` then fails
to load, which `criterion@rich-text@c-readable-tree` rules out.

```ron
Text(
    root: [
        Chars("Let x be a "),
        Span(
            id: 41,
            style: Custom(bold: true),
            text: [Chars("prime")],
        ),
        Chars(" number"),
    ],
    style: Named("Body"),
)
```

Characters are kept in one canonical form: `apply` and `load_ron` merge two adjacent runs of
characters and drop an empty run, so a document and its round trip through an operation and its
inverse compare equal. The renderer lays the tree out in one Parley layout with `ranged_builder`,
as it laid out one string before, the area's values as defaults and each span's resolved values
pushed for its byte range, outer spans before inner, since a later push wins over an earlier one
where they overlap. `ranged_builder` does not collapse whitespace, per its doc comment in Parley
0.12.0's `builder.rs`, so the drawn text's byte indices are the stored text's; `TreeBuilder`
keeps whitespace too unless a style asks it to collapse, `WhiteSpaceCollapse` defaulting to
`Preserve` in Parley 0.12.0's `style/mod.rs`, so it is no rival a fact defeats, and the builder the
code had is kept. The layout carries each run's colour, a brush type in place of `()`. Nearest
rival: flat runs over one string. Defeated by #arg-2 and #arg-3.

### #text-tree-normal-form

As built on D6: the model dissolves no span by itself. An operation that would leave a custom span
setting no field is refused, and its caller unwraps the span instead; a named span is never
dissolved, whatever its values, and a span that sets a value equal to its parent's is kept, so that
a bold word inside a bold span stays bold when the outer span changes. A file that holds a custom
span setting no field loads, the span drawn with its parent's values, and an operation that leaves
it as it is is not refused; `Op::SetText`, which makes every span of its tree anew with a new id,
refuses it, as do a wrap, a change of style and a cut; a content set whole, as an undo sets it, and
a duplicated slide carry such a span as it is. The model accepts an empty span, in an
operation and in a file; the editor sends `unwrap` for the empty span the caret was in when the
caret leaves it, and for each empty span of the text when the stay ends. Adjacent spans with equal
styles stay apart. Nearest rival: merging equal siblings; defeated by #arg-5.

### #span-style-overrides

A span style holds `font`, `size`, `bold`, `italic` and `colour`, each optional; the values a span
draws with are its own where set and its parent's elsewhere, the root's being the area's style.
Alignment stays a field of the area's style alone. A value set on a named span makes it custom, with
the named values and that change, as `design@rootpoint-model@named-styles` says of a text. Nearest
rival: a full style per span; defeated by #arg-6.

### #boundary-caret-positions

A caret position is a text node and an offset in it, a span counting as one item. Right at an
offset before a span enters it at offset 0; right at its end leaves it, after it in the parent; left
is the mirror. Typing inserts into the caret's text node. A click places the caret in the deepest
text node that owns the clicked glyph, on the side of the glyph clicked. While editing, every key
goes to the text, as `design@rootpoint-app@edit-entry` says: the arrows move the caret, Shift with
an arrow extends the selection; outside editing, Alt with an arrow stays the node keys. Nearest
rival: one position per edge and a rule for the typing style; defeated by #arg-2 and #arg-12.

Backspace and Delete, with no selection, act in the caret's text node, as LyX's do, D8: on a
character, they delete it; on a span with items, a first press selects the span whole and a second
deletes it; an empty span goes at once; at a span's own start for Backspace, or its end for Delete,
the span is unwrapped, its content kept and the caret where it was drawn; at the root's start or
end, nothing.

### #ancestor-frames

While editing, the overlay draws a thin frame around each span that holds the caret, one rectangle
per line the span covers, placed through `slide_to_pixels`, lighter for an outer span; the root has
no frame. The caret is drawn at the height and in the colour of the values it will type with. No
frame is drawn into the slide's image, nor in the final view. The side panel shows the ancestor
row: root, then each span from outer to inner, each with its style; a click on a span of the row
shows its style in the panel's controls, where it is changed. Nearest rival: LyX's reserved space;
defeated by #arg-7.

### #text-node-ids

Every span has an id from `Document::next_id`, the one counter of slides, nodes and assets, unique
in the document, written in the file and printed `t<n>`. `apply` allocates it, as `Op::Split` gives
its new area an id; a span made by a duplicate of a slide takes a new id, as its nodes do. A command
that makes a span or changes characters names a text node by its id, the root by the area's, and
takes an offset or a range in it, read at that moment: `wrap n3 12..17 bold`, `wrap t41 0..3
italic`. Every later command names the span by its id. Nearest rival: spans addressed by ranges
only; defeated by #arg-8.

### #text-edit-commands

The editor sends one small command per change, each one operation: `insert`, `delete`, `wrap`,
`unwrap`, `span`, which sets a span's style, `split-span`. A stay stays one undo group. Nearest
rival: a whole-text replacement at each change; defeated by #arg-9.

`text` sets a whole tree, given on one line, `Op::SetText`, which gives its spans new ids; `tree`
prints a text in the same form. An illustration, not the grammar, whose shape #span-commands fixed
in `design@rootpoint-protocol@protocol-transport`, its details in `path@rootpoint-protocol@README.md`:

```text
text n3 ["Let x be a ", {bold "prime"}, " number"]
tree → n3 text ["Let x be a ", {t41 bold "prime"}, " number"] style="Body"
```

Nearest rival of the one-line tree: a JSON payload, `rejected@rootpoint-protocol@json-rpc-commands`.

### #selection-one-level

A selection's anchor and caret are in one text node; a span inside the range is taken whole; Shift
with an arrow at a span's edge steps over the span. Started inside a span and extended past its end,
the selection rises to the span's parent and takes the span whole, as LyX's `normalAnchor` does.
Nearest rival: free selection; defeated by #arg-10 and #arg-16.

### #inline-math-node

`TextItem` is an enumeration with room for more kinds; the caret position, the operations and the
file name a kind by its variant. Only the span is built. The equation kind, numbered and
referenceable, stays the area kind of `issue@rootpoint@more-content-kinds`, as the owner described
in round 3.

### #style-action-target

A style action, Ctrl+B, Ctrl+I, D4, or a control of the side panel, with no selection opens an
empty span with that style at the caret and puts the caret in it; with a selection, it wraps the
selection in one span, the selection being one text node's, #selection-one-level, except that a
selection that is exactly one whole span changes that span's style. A style action inside a span
already in that style opens a span all the same, which D6 keeps.

### #split-at-caret

Ctrl+Space, or the button of the ancestor row, cuts the span that holds the caret at the caret into
two spans of the same style, the second with a new id, and puts the caret between them, in the
parent. At the span's edge the editor sends nothing and moves the caret out of the span, after it at
its end and before it at its start; the command `split-span` at an edge is refused. Pressed again,
it cuts the next span out. It is `Op::SplitSpan`, and the command `split-span t<n> <offset>`; it is
not named `split`, which is the command that splits a node of a slide. Nearest rival: a toggle key
per style; defeated by #arg-17.

### #named-span-styles

`Document::span_styles` is a table of span styles, named by name. A new document holds none. A span
names one, or holds its own values. The rules of the text styles apply: a rename rewrites every span
naming the style, `design@rootpoint-model@style-key`; a removal gives its users a replacement, a
span style of the table or a custom copy of the removed values each,
`design@rootpoint-model@style-removal`, and its inverse is `Op::RestoreSpanStyle`; a span naming a
style the table lacks draws with its inherited values and is a cosmetic breach,
`Breach::MissingSpanStyle`, `design@rootpoint-model@cosmetic-fallbacks`. The Styles tab lists the
span styles in their own section. Nearest rivals: B and C, defeated by #arg-19.
## Mapping tables

### An operation of a text tree, to its inverse

Every operation that changes one area's text returns, as its inverse, `Op::SetContent` with the
area's former content, as `Op::SetTree` is the inverse of the structural edits of a slide: a
snapshot restores every span with its id and its style, and a text is small.

| operation | inverse |
| --- | --- |
| `Op::InsertText`, `Op::DeleteText`, `Op::WrapSpan`, `Op::UnwrapSpan`, `Op::SetSpanStyle`, `Op::SplitSpan`, `Op::SetText` | `Op::SetContent { area, content }` of the former content |
| `Op::AddSpanStyle { name, style }` | `Op::RemoveSpanStyle { name, into: None }` |
| `Op::SetSpanStyleValues { name, style }` | `Op::SetSpanStyleValues` with the former values |
| `Op::RenameSpanStyle { from, to }` | `Op::RenameSpanStyle { from: to, to: from }`, refused while a span names `to`, as `Op::RenameStyle` is |
| `Op::RemoveSpanStyle { name, into }` | `Op::RestoreSpanStyle { name, style, users, into }`, as `Op::RestoreStyle` |

### A command, to the operation it runs

| command | operation |
| --- | --- |
| `insert <n or t> <offset> "<text>"` | `Op::InsertText` |
| `delete <n or t> <a>..<b>` | `Op::DeleteText` |
| `wrap <n or t> <a>..<b> <style>` | `Op::WrapSpan` |
| `unwrap t<n>` | `Op::UnwrapSpan` |
| `span t<n> <style>` | `Op::SetSpanStyle` |
| `split-span t<n> <offset>` | `Op::SplitSpan` |
| `text <n> <tree> [options]` | `Op::SetText`, and the area style's options as today |
| `text <n> <options>` | `Op::SetContent` of the area's current tree, `design@rootpoint-protocol@text-style-alone` |
| `span-style add`, `set`, `rename`, `remove` | `Op::AddSpanStyle`, `Op::SetSpanStyleValues`, `Op::RenameSpanStyle`, `Op::RemoveSpanStyle` |

`<n or t>` names the root of an area's text by the area's id, or a span by its id. `<style>` is
`style="<name>"` or options of a custom span style, as `text`'s options are.

### A caret move in a text node, to the caret position it gives

| caret at | Right | Left |
| --- | --- | --- |
| offset i, item i a character | past its cluster, same text node | (by item i−1) |
| offset i, item i a span s | s, offset 0 | (by item i−1) |
| offset i, item i−1 a span s | (by item i) | s, its last offset |
| last offset of span s | after s in its parent | (by item i−1) |
| offset 0 of span s | (by item i) | before s in its parent |
| last offset of the root | no move | (by item i−1) |
| offset 0 of the root | (by item i) | no move |

A move past a character passes one cluster of the layout, as Parley 0.12.0 cuts it, and stops at
a span's edge: a character with the marks drawn on it is one cluster, so the caret stands on a
glyph's boundary, as `design@rootpoint-app@edit-in-place` asks; each character of a ligature is a
cluster of its own, given an even share of the ligature's advance by `Cluster::advance`, and
Parley's own moves stop there too, so the caret stops inside a ligature, at that share.

Up and Down, word moves and line moves are Parley's on the byte index, then mapped back to a caret
position; #text-tree's map gives the forward direction and each span's byte range. The backward
rule, settled at #tree-caret's audit, is the click's rule of #boundary-caret-positions applied to
every byte index Parley gives: the caret goes into the deepest text node that owns the glyph on the
side Parley's affinity names, the glyph after the index for `Downstream` and the glyph before it for
`Upstream`, which is the side of the glyph a click took, as Parley 0.12.0's `Cursor::from_point`
sets it; with no glyph on that side, at the text's start or end, the other side is taken. Several
edges meeting at the index then never leave a choice: one glyph owns each side.

### A style action, by the selection, to the operation it gives

| selection | operation |
| --- | --- |
| none, caret in text node N at offset i | `Op::WrapSpan` of the empty range i..i of N, then the caret into the new span |
| range a..b of text node N | `Op::WrapSpan` of a..b of N |
| range covering exactly one span s, whole | `Op::SetSpanStyle` of s |
## Losing alternatives

- **Flat runs over one string**, against #text-node-tree: lost on #arg-2 and #arg-3.
- **Flat runs and spans at once, as LyX**, against #text-node-tree: ruled out on #arg-1.
- **Merging equal siblings**, against #text-tree-normal-form: lost on #arg-5.
- **A full style per span**, against #span-style-overrides: lost on #arg-6.
- **One caret position per edge, with a rule for the typing style**, against
  #boundary-caret-positions: lost on #arg-2 and #arg-12.
- **Reserved space for the frames, as LyX**, against #ancestor-frames: ruled out on #arg-7; T1's
  response names it.
- **Spans addressed by ranges only**, against #text-node-ids: lost on #arg-8; T3's response names a
  middle shape.
- **A JSON payload for the tree**, against #text-edit-commands: already
  `rejected@rootpoint-protocol@json-rpc-commands`.
- **Free selection**, against #selection-one-level: lost on #arg-10 and #arg-16.
- **A toggle key per style to leave it**, against #split-at-caret: lost on #arg-17.
- **One table of full styles named by spans (B), and one table of optional fields for areas and
  spans (C)**, against #named-span-styles: lost on #arg-19; T4's response names C.

## Readings

The work reads no external specification. The behaviour of Parley 0.12.0 and eframe 0.37.0 it
relies on is read in their sources, #arg-4 and #arg-18, and each slice's audit reads it again.

## Premortem

Assume the work shipped and failed. The causes, each with the thread it stresses and its verdict;
the owner's word of round 4, "keep all tripwires and ac", rules on every label.

1. The caret drifts or lands at the wrong depth: Parley lays out one flat string, two caret
   positions at an edge map to one byte index, and the editor's map between them is wrong.
   Stresses #boundary-caret-positions and #text-node-tree. An acceptance criterion, **AC1**,
   #ac-caret-walk.
2. The author cannot tell the two positions at an edge apart, since a frame taking no space does
   not move the caret sideways. Stresses #ancestor-frames. An acceptance criterion, **AC2**,
   #ac-edge-positions-differ, and a tripwire, **T1**: it fires when the owner reports, on two
   occasions after the overlay lands, that they typed into a node they did not mean to, or that a
   frame hid a letter they needed to read; the response reopens #ancestor-frames, toward reserved
   space, which needs `goal@rootpoint-app@faithful-slide-view` relaxed as the owner said they might, or
   another mark of depth; re-entry at the design audit of the next work on the overlay, or earlier
   on the owner's word.
3. A composition of the input method inside a span goes into another node, or splits the stay's
   undo step. Stresses #text-edit-commands. An acceptance criterion, **AC3**,
   #ac-composition-in-span.
4. Agents write the one-line tree badly, brackets and quotes nested on one line. Stresses
   #text-edit-commands; an assumption, not measured. A tripwire, **T2**: it fires when, in one
   session of the owner's, an agent has `text` with a tree refused for its grammar three times or
   more; the response reopens the grammar of `text`, toward a flatter form; re-entry at the design
   audit of the next work on the protocol.
5. The saved file stops being easy to edit by hand, through the span ids and the span syntax.
   Stresses #text-node-ids and `goal@rootpoint-model@faithful-readable-file`. A tripwire, **T3**:
   it fires when the owner, or an agent the owner asked, edits a saved text tree by hand and
   reports the ids or the span syntax in the way, once; the response reopens #text-node-ids,
   toward ids a hand-written span may leave out and the loader gives; re-entry at the next design
   work on the save format.
6. Two style tables confuse authors. Stresses #named-span-styles. A tripwire, **T4**: it fires when
   the owner reports, more than once after the Styles tab lists the span styles, looking for a style
   in the wrong table, or applying a text style where a span style was meant; the response reopens
   #named-span-styles toward C, one table of optional fields; re-entry at the design audit of the
   next work on styles.
7. Many spans make a frame too slow. Stresses #text-node-tree. Watched by
   `tripwire@rootpoint-render@render-within-bounds`, whose re-entry is the design audit of work
   that changes the renderer, #text-tree's among them; its instruments render the mixed slide of
   `path@rootpoint-render@tests/common/mod.rs` and `path@rootpoint-app@tests/data/mixed.ron`, which
   held no span; #text-tree gave the renderer's a styled text and timed it before and after, and
   #span-styling gives `mixed.ron` one; no new tripwire.

T1 to T4 are written at the harvest of the slice that records the decision each guards, as
`skill@knowledge-architect-decision-recording@ruled-tripwires` says, each naming that head: T3 at
#text-tree, guarding the rewritten `design@rootpoint-model@stable-ids`; T2 at #span-commands,
guarding the protocol's head of #text-edit-commands and the rewritten
`design@rootpoint-protocol@protocol-transport`, its tree on one line; T1 at #span-styling,
guarding `design@rootpoint-app@editor-overlay`, with the frames recorded in it; T4 at
#span-styling, guarding the model's head of #named-span-styles, written at #text-tree.

## Acceptance criteria

### Every caret position is visited once each way, and drawn on its glyph `##ac-caret-walk`

AC1. Guards #boundary-caret-positions and #text-node-tree. Judged at #tree-caret. Observable: a
property test over random trees, in `path@rootpoint-app@tests/editing.rs`, walks the caret with
Right from the start to the end and with Left back; each caret position that stands on a cluster's
boundary is visited once in each direction, and each maps to a byte index whose drawn glyph is the
caret's, a ligature's characters after its first excepted, within the half-pixel tolerance of
`design@rootpoint-app@edit-in-place`, at the three window sizes that test uses.
Response: the map is fixed in the slice; a tree whose positions cannot be mapped to the one layout
reopens #text-node-tree.

### The two positions at an edge show different marks `##ac-edge-positions-differ`

AC2. Guards #ancestor-frames. Judged at #span-styling. Observable: for each pair of caret positions
at an edge of the sample's styled text, the overlay's shapes, read with no window, differ: a frame
is drawn at one and not the other, or the caret's height or colour differs. Response: the marks are
fixed in the slice; marks that cannot differ without taking space reopen #ancestor-frames.

### A composition inside a span commits into that span, in the stay's one undo step `##ac-composition-in-span`

AC3. Guards #text-edit-commands. Judged at #tree-caret. Observable: a test in
`path@rootpoint-app@tests/editing.rs` starts a composition with the caret inside a nested span,
commits it, and finds the committed text in that span; one `undo` then removes the stay's whole
change. Response: the composition's commands are fixed in the slice; one that cannot keep its node
reopens #text-edit-commands.

## Implementation sequence

1. #text-tree, landed, its spec deleted with its harvest: the tree in the model, its operations, the
   span style table, the file at version 5, and the renderer drawing spans; `area_text`'s map. Fails
   alone on: the fixture's round trip and a golden image of a styled text.
2. #span-commands, landed, its spec deleted with its harvest: the commands of the tree and of the
   span style table, `tree` printing the tree, `text` taking one. Fails alone on: the parse of every printed command, and
   the property test of `design@rootpoint-protocol@no-new-breach`.
3. [#tree-caret](tree-caret.md): the editor's caret over the tree, typing, deletion and composition
   by the small commands, the selection of one node. Fails alone on: AC1 and AC3.
4. [#span-styling](span-styling.md): the style actions, "split here", the frames, the ancestor row,
   the span styles in the Styles tab, the browser's keys. Fails alone on: AC2, and the actions
   checked against a second session.

## Order rationale

- #text-tree before #span-commands: a command is an operation of the model, which comes first.
- #span-commands before #tree-caret: the editor sends the commands, each tested headless before.
- #tree-caret before #span-styling: a style action and the frames act at the caret, which must
  move through the tree first.

## Defaults awaiting the owner

- **D1**, ruled: accepted by the owner, "defaults accepted, proceed". Its text: the work is a
  milestone of four slices, in the order above, rather than one spec. The session's default, put to
  the owner at the hand-off; it blocked the merge of this milestone's branch.
- **D2**, ruled: accepted by the owner, "defaults accepted, proceed". Its text: each slice on its
  own branch, merged before the next, as
  `skill@knowledge-architect-planning@cutting-steps-and-slices` says; the owner chose one branch for
  the whole of the milestone on asset storage, which is no ruling on this one. It blocked the merge
  of this milestone's branch.
- **D3**, ruled: accepted by the owner, "defaults accepted, proceed". Its text, on
  #text-edit-commands: a paste inserts plain text into the caret's node, and a copy carries no span;
  styled copy and paste is a later consequence. It blocked the merge of #tree-caret.
- **D4**, ruled: accepted by the owner, "defaults accepted, proceed". Its text, on
  #style-action-target: Ctrl+B sets bold and Ctrl+I italic; colour, size, font and a named span
  style have no key, and are set from the side panel. It blocked the merge of #span-styling.
- **D5**, ruled: accepted by the owner, "defaults accepted, proceed". Its text, on
  #boundary-caret-positions: `design@rootpoint-app@caret-editor-state` makes, on the owner's word,
  the caret and the selection the editor's state, and the owner approved the composing range apart.
  The default reads the caret's text node as part of the caret, a caret position being a text node
  and an offset, so the ruling covers it as it stands and its head is reworded at #tree-caret, with
  no new member. The other reading, a new member, goes to the owner as a proposal, since the ruling
  names its members. It blocked the merge of #tree-caret.
- **D6**, ruled: accepted by the owner, "defaults accepted, proceed". Its text, on
  #text-tree-normal-form: the owner approved, in round 2, dissolving a child whose style changes
  nothing. The reviews of this document found that the rule dissolves a named span whose style is
  missing or equal to its parent's values, losing its name, which #named-span-styles keeps, and a
  bold span inside a bold span, which then loses its bold when the outer span changes. The default
  narrows it: the model dissolves no span, an operation that would leave a custom span setting no
  field is refused, and a named span is never dissolved, as "Decided design" says. It narrows a
  ruling of the owner, so it blocked the merge of this milestone's branch.
- **D7**, ruled: accepted by the owner, "D7 accepted". Its text, on #text-node-tree and
  #span-style-overrides: the review before the merge of #text-tree found that the two heads' titles,
  taken from the thread titles the owner approved, each state a decision another head holds: "each
  span with its own values over its parent's" in the tree's title is #span-style-overrides, and "the
  area keeps one full style" in the overrides' title is the rival argument of
  `design@rootpoint-model@named-styles`. The default, built: the titles drop those clauses, "An
  area's text is a tree of characters and spans" and "A span sets each text option it changes, each
  optional, over its parent's values", the decisions unchanged. It narrows titles the owner
  approved, so it blocked the merge of #text-tree.
- **Q3** and **Q4**, ruled: accepted by the owner, "q3 and q4 accepted". Open at the design audit
  of #span-commands. Q3's text, on #text-node-ids: the tree of `text` is optional; `text n3 bold`
  changes the area's style and keeps its tree and its ids, by `Op::SetContent` of its current
  tree, and is refused on an image area, which has no text style to change; the side panel sends
  that form; the rival, a verb of its own for the area's style. Q4's text, on
  #span-style-overrides: `<key>=inherit`, as `bold=inherit`, for each of the five values, in every
  list of a span's options, `span`, `span-style set`, `wrap`, `span-style add` and a span of
  `text`, the last three starting from no value, where it changes nothing; the rival, an option
  naming the values to clear, `clear=bold,size`. Harvested as `design@rootpoint-protocol@text-style-alone` and
  `design@rootpoint-protocol@span-value-inherit`. They blocked the merge of #span-commands.

- **D8**, awaiting the owner. Open at the design audit of #tree-caret. Its text, on
  #boundary-caret-positions: the milestone did not say what Backspace and Delete do at a span's
  edge, and two shapes meet every binding constraint. The default, built: LyX's, read in its
  source, Codeberg mirror `LyX-org/lyx` at commit `64141cf0e7a073e1226ec168d317618537ba0fab`:
  `site.bind` binds both keys with `confirm`; `Text::dispatch` of `path@elsewhere@src/Text.cpp` then selects a
  non-empty text inset before the caret on a first Backspace and deletes it on a second, since
  `InsetText::confirmDeletion` is `!text().empty()`; and `Text::backspace` at position 0 of an
  inset's first paragraph calls `dissolveInset`, as `Text::erase` does at the end of its last. Here: with no
  selection, the keys act in the caret's text node; a span with items is selected by a first press
  and deleted by a second, an empty one at once; at a span's own start for Backspace, or its end
  for Delete, `unwrap` dissolves it; at the root's ends, nothing. Every edit then stays in the
  caret's node, where typing goes, and the keyboard alone can remove a span's style, which no style
  action of this slice can. The rival: the keys delete the character drawn before or after the
  caret, in whatever text node holds it, never a span whole and never unwrapping; it is nearer to
  a plain editor, but it edits a node the caret is not in, which the author cannot see until the
  frames of #span-styling, and it leaves an emptied span behind until the stay ends. It blocks the
  merge of #tree-caret.

## Harvest

Each slice's row is in its spec. This document's own row, at the commit that deletes it, after the
last slice:

- Report on AC1, AC2 and AC3 once more, each to a tripwire on the owner's word or deleted.
- Read `issue@rootpoint@more-content-kinds` again against what the tree then admits: the owner's two
  forms of maths, and the room the tree leaves for maths and for a reference to an equation.
- Check that each later consequence below is in the issue entry, the plan document or the tripwire
  that it names, or in a `todo` issue opened for it then.

## Later consequences

- **Maths**: a `Math` variant of `TextItem`, laid out by the project and placed with Parley's
  `InlineBox`, and a maths font, #arg-11; with the equation kind and its references,
  `issue@rootpoint@more-content-kinds`.
- **Styled copy and paste**, D3: a `todo` issue opened at the harvest of #tree-caret.
- **The frames, revised on the owner's live use**: T1, written at #span-styling.
- **Span styles in a new document**: none today, #named-span-styles; no later work is planned, and
  the head of #named-span-styles states it.
