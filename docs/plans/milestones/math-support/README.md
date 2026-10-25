# Milestone: math, inline and in numbered equations, edited as in LyX

## Status and audience

This is the milestone document of math in rootpoint:
- a formula is a tree in memory, written as a closed subset of LaTeX in the file and in commands, and
  laid out by the project's own code from the OpenType MATH table of Libertinus Math, built in;
- a text holds formulas inside its sentences, and references to equations;
- an equation is a kind of content of its own, one per area, numbered, fitted to its area;
- the window edits a formula in place, as LyX does, with LyX's math keys;
- Open Sans replaces Noto Sans as the built-in text family.

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
  the owner's message that opens "I want to tackle adding math support to this project".
- The survey that grounded the discussion is `path@rootpoint@docs/grounding/math-typesetting.md`,
  with LyX's earlier survey `path@rootpoint@docs/grounding/lyx-nested-text.md`, section 3; read both
  before the first slice and before #math-editing.

**The owner's messages of the discussion**, verbatim, in order. Round n is the owner's message n and
the session's reply to it; a message the owner typed while the session was working is marked
mid-turn, in the round it arrived in.

**Round 1**

> I want to tackle adding math support to this project. This is the only piece that is missing before I can start using it for real presentations.
> I think a good part of the design is already settled, by following what lyx does, as we already did in multiple places. I would like to get both shapes available: the inline math, and the node kind holding a numbered equation, if possible that can be pointed at from text anywhere. I would also like to provide by default the same math typing shortcuts that lyx provides (alt+m into other keys for most math related chars, f for a fraction, alt+m into g then a letter for greek... if i recall correctly).

**Round 2**

> math-storage-form: agreed on that shape
> math-text-item: agreed
>
> equation-content-kind: agreed
>
> equation-rows: agreed on rows, I do need them
>
> equation-numbering agreed
> equation-reference-target: I do believe a naming would help here. Name would be a property of the node kind, set at the same place text styles and image format are setup, in the right sidebar. Maybe we can go for the simpler shape you propose at first, a leave a todo issue with leads for the equation labelling.
>
> removed-equation-references: agreed on this shape. this is better than allowing the removal I think, under the condition that there must be an easy way to retarget all references.
>
> math-editing-model agreed
>
> math-symbol-palette: agreed. Though, one more thread I want to open: lyx accepts typing any backslash command of latex and replaces it with correct symbol/node (like `\frac` or `\leq`). Can't we support that too ?
>
>  empty-cell-placeholder: agreed, I believe this is needed and an unfinished formula is not supposed to exist in a real presenter run.
>
> math-in-protocol-grammar: agreed
>
> math-edit-commands: agreed
>
> math-keys: agreed on copying the keys from lyx exactly. The browser mode can have a few bindings disabled in case of conflicts, not a problem.
>
> equation-options: is it possible to have an equation automatically fit to the allocated area ? this would be very convenient, but I suspect quite hard to do.
>
>
> Q1: often aligned multi row, occasionally matrices. I think your default will cover my use cases.
> Q2 I quite like noto sans, if it has a matching math font I'd like to use it.
> Q3: I would like for keybinds to be configurable later. Do you think it is more costly to implement fixed first then make them customisable later ?
> Q4: actually, I will need bullet list too, I had forgotten them. we can focus on math today though. code snippets can come later, no need for my planned talk.
> Q5: yes

**Round 3**

> For the font, on second thought, can you look at which font I used in my thesis at https://github.com/teoboutin/thesis ? I like it a lot, can you look whethe rit can be vendored ?
>
> reference-retarget agreed
> latex-command-set agreed on shape
>
> equation-fit: both could be made available, no ? If yes, I'd go for this.
>
> keymap-table accepted

**Round 4**

> about the font, I was talking about the math one. though i remenber setting one different than the default. on the pdf, it does not look like computer moderm to me (for math) but I could be mistaken. can you check on the pdf ?
> Also, Could you show me a few side by side examples before I settle on what to bundle ?
>
> About licensing, I plan to make the project open source ultimately. probably soon, so we could choose a license right now. what would you recommend for such a project ? Iwant it as widely available as possible for everyone and everywhere

Mid-turn:

> Q9: lets go with this dual license setup

**Round 5**

> Among the sample, my preference goes to Libertinus Math. Since it is also the one I chose for my thesis text, I am considering making it the default for both text and math here in this project.

**Round 6**

> can you also show me a sample of text fonts to choose ? The current font was chosen in a rush, without putting much thought on it.
> About libertinus Math: which symbols does it lack though ? Before I fully settle on this choice

**Round 7**

> math-glyph-fallback: agreed on default. the missing set here is not much of a problem. Keep the exact missing set recorded in the porject btw.
>
> Q2: from the sample, the looks I prefer and find easier to read are either Noto Sans or Open Sans together with Libertinus Math. I have a slight preference for Open Sans over Noto Sans.

Mid-turn:

> The dual license branch can be bundled with the math work. It has little to review

**Round 8**, given against the checkpoint table of round 7 and its premortem:

> builtin-text-family accepted on this default
> you can keep all tripwires and AC.
> You can proceed with planning, under the milestone shape.
> We will merge the milestone work with the dual licensing work, as the same branch. The missing set can be recorded as you proposed

**Round 9**, after the reviews of this document, on its defaults D1 to D5:

> D1 agreed, this is what i meant
> agreed on the other defaults

The owner's "Q2:" of round 7 answers Q12 of round 6, the text font, by its content; Q2 of round 1
asked whether the math should be serif or sans. The owner's hedge of round 6 on Libertinus Math is
answered by the owner's words of round 7, the missing set "not much of a problem" and Open Sans
"together with Libertinus Math".

**The checkpoint table of round 7**, against which the owner's word of round 8 was given, verbatim:

| thread | state | position |
| --- | --- | --- |
| #math-storage-form | approved | a tree in memory; a LaTeX subset in the file (raw strings, `escape_strings(false)`) and in commands |
| #math-text-item | approved | `TextItem::Math`, one offset, one Parley inline box; takes colour and size from its text node |
| #equation-content-kind | approved | `Content::Equation`, number at the area's right edge; "Equation" in the node menu |
| #equation-rows | approved | rows at `&` under one number; matrix, pmatrix, bmatrix, cases |
| #equation-numbering | approved | computed: slide order, then tree order; `numbered` per equation; Alt+M N toggles it |
| #equation-reference-target | approved | `TextItem::Ref(n<id>)` showing `(n)`, picked from a list; a todo issue for naming equations |
| #removed-equation-references | approved | removing or converting a referenced equation is refused, and the refusal names the references |
| #reference-retarget | approved | `retarget n7 n9`; "Retarget all to…" in the side panel; the refusal names the count |
| #math-layout-engine | argued | our own layout from the MATH table, parsed by ttf-parser 0.25, a box per atom and per cell; todo issue for read-fonts ≥ 0.45 |
| #math-font | approved | Libertinus Math, built in |
| #math-glyph-fallback | approved | no fallback math font; missing glyphs reported; the missing set recorded in the project |
| #builtin-text-family | **presumed-settled** | Open Sans `noto-set` replaces Noto Sans; text falls back to Libertinus Math if fontique allows it |
| #math-editing-model | approved | structural editing in place, LyX's cursor; `^` `_` `\`; Backspace selects first; an empty formula is removed |
| #latex-command-set | approved | a command typed after `\` becomes its symbol or structure; table generated from W3C `unicode.xml`, limited to the font's glyphs |
| #math-symbol-palette | approved | palette in the Selection tab while math is edited |
| #empty-cell-placeholder | approved | space reserved in every display; outline drawn by the overlay |
| #math-in-protocol-grammar | approved | `$…$` literal, no escaping inside; `{ref n7}` |
| #math-edit-commands | approved | one formula replaced whole per edit |
| #math-keys | approved | LyX's bindings copied exactly; the browser build may disable a binding that conflicts |
| #keymap-table | approved | the math keys as a built-in table; a todo issue for configurability |
| #equation-options | approved | size `Fit` (the default) or `AtMost(size)`, plus a colour, unnamed |
| #equation-fit | approved | one layout scaled by k; both modes |

The table's "todo issue for read-fonts ≥ 0.45" was opened as a `deferred` issue,
`issue@rootpoint-math@read-fonts-math-table`, since an event, parley's move, gates its work. The
table's `TextItem::Ref(n<id>)` writes a reference as commands print it; the file writes it
`Ref(7)`, since an id is a plain number there, `design@rootpoint-model@save-format`.

The table was put "for you to contest": "A positive word on this table closes it,
#builtin-text-family included." The owner's word of round 8 closes with the table what the table
shows and no other word of the owner addresses: ron's `escape_strings(false)`, put in round 2 as a
default "unless you say otherwise", and #equation-options as finally shaped. The session closed on
their argument, and reported under "Decided" in round 4, #math-layout-engine, `argued`, the source
of the table of commands, the choice of ttf-parser, and LyX's math autocorrection left out; the
table showed the first three, and the owner did not contest them.

**The questions put to the owner under a label**, and their answers:

- Q1, round 1, how heavy the math of the owner's talks is; answered in round 2.
- Q2, round 1, serif or sans math; answered in round 2, then superseded by the owner's request of
  round 3 to look at the thesis's font.
- Q3, round 1, whether a configurable keymap is intended; answered in round 2.
- Q4, round 1, whether the first talks do without bullet lists and code snippets; answered in round 2.
- Q5, round 1, whether to run the survey; answered in round 2.
- Q6, round 2: "#equation-fit: fit both ways (the default) or shrink only?"; answered in round 3.
- Q7, round 2: whether the side panel's "Retarget all to…" and the refusal's message are "easy
  enough"; answered in round 3, "reference-retarget agreed".
- Q8, round 3, the Computer Modern math or the Libertine look; superseded in round 4, the thesis's
  math being Computer Modern, #arg-53.
- Q9, round 4, the licence; answered mid-turn in round 4.
- Q10, round 4 ("round 5" of the session's headings), which math font; answered in round 5.
- Q11, round 6, #math-glyph-fallback; answered in round 7.
- Q12, round 6: "Which text font or fonts, and shape A or B?"; answered in round 7, under the
  owner's label "Q2".
- Q13, round 7, the tripwires and the acceptance criteria; answered in round 8.

## How the work is done

Before each slice, read `skill@knowledge-architect-planning@working-a-slice` whole, and follow it.
The slices are built in the order of "Implementation sequence".

- **Branches.** This document, its slice specs, the survey, the issues it opens and the licence of
  the project, `design@rootpoint@permissive-licence`, are on the branch math-milestone. By D1,
  that branch merges on its own, after its review, before #math-layout begins; each slice then has a
  branch of its own, reviewed and merged before the next begins.
- **Grounding of the first slice.** #math-layout touches the root, the render crate and the
  maintenance tool, and creates the math crate: point 1 of the slice's procedure reads the
  `CLAUDE.md`, the design home and the rejected home of each of the root, `path@rootpoint@crates/rootpoint-render/`
  and `path@rootpoint@tools/xtask/`. By `path@xtask@CLAUDE.md`, the new crate's library root reads
  `ROOTPOINT_CHECKOUT`, as every library root of the workspace does.
- **A new Component's goals are the owner's.** The harvest of #math-layout drafts the math crate's
  goals under `skill@knowledge-architect-goal-setting`, and the owner rules on them goal by goal
  before that harvest merges.
- **Gates.** `cargo x gates` runs every check a branch owes before it merges,
  `instructions@merge-gates`. Every slice keeps them passing, the gate `wasm` and the browser tests
  included: a test added to `path@rootpoint-app@tests/actions.rs` or
  `path@rootpoint-app@tests/web.rs` builds and passes on the wasm target, as
  `path@rootpoint-app@CLAUDE.md` says.
- **Actions are commands.** Every new action of the window issues protocol commands through
  `App::run`, checked against the same commands on a second session in
  `path@rootpoint-app@tests/actions.rs`, per `design@rootpoint-app@actions-are-commands`. Every new
  command parses back from its printed form, per `design@rootpoint-protocol@protocol-transport`.
- **No compatibility.** #equation-kind and #inline-math each change the save format, to
  `FORMAT_VERSION` 6 and 7 in `path@rootpoint-model@src/file.rs`, with no migration, per
  `design@rootpoint@no-compat-in-initial-dev`; each rewrites the `.ron` files of the tree, which
  `find crates -name '*.ron'` lists: today `path@rootpoint-protocol@sample/sample.ron`,
  `path@rootpoint-app@tests/data/mixed.ron`, `path@rootpoint-model@tests/data/fixture.ron` and
  `path@rootpoint-model@tests/data/fixture-edited-by-agent.ron`.
- **Licences of what is built in.** A font or a data file built into a binary whose licence has
  conditions of its own keeps its licence text beside it, as
  `path@rootpoint-render@fonts/open-sans/OFL.txt` sits beside Open Sans, by
  `design@rootpoint@permissive-licence`: the fonts' OFL, and the W3C notice of the table of commands.

## Names

- **The session**: `Session` of `path@rootpoint-protocol@src/session.rs`, which holds the document
  and runs every command.
- **An operation**: a variant of `Op` in `path@rootpoint-model@src/ops.rs`, applied by `apply`,
  which returns its inverse, per `design@rootpoint-model@edit-operation-core`.
- **A text**: `TextContent { root: TextNode, style: StyleChoice }` of
  `path@rootpoint-model@src/document.rs`; its items are `TextItem::Chars` and `TextItem::Span`.
- **A content**: `Content` of the same file, today `Text` or `Image`.
- **The renderer's layout of a text**: `lay_out_area` and `text::layout_text` of
  `path@rootpoint-render@src/lib.rs` and `path@rootpoint-render@src/text.rs`; `area_text` gives it
  with its `TextMap`, `path@rootpoint-render@src/map.rs`.
- **The built-in fonts**: `BUILTIN_FONTS` and `BUILTIN_SANS` of `path@rootpoint-render@src/text.rs`;
  `FontRef::Builtin("sans")` names the family; `BUILTIN_FAMILIES` of
  `path@rootpoint-model@src/document.rs` is `&["sans"]`.
- **The lexer of commands**: `split_words` and `Word` of `path@rootpoint-protocol@src/command.rs`.
- **The survey**: `path@rootpoint@docs/grounding/math-typesetting.md`.
- **The missing set**: the 44 math commands of KaTeX, each mapping to one character, whose character
  Libertinus Math 7.051 has no glyph for, listed in the survey, section 4.
- **The reference formulas**: the five formulas `REFERENCE` of `path@rootpoint-math@tests/layout.rs`,
  which AC1 and AC2 lay out.
- **The slices**: #math-layout, #builtin-open-sans, #equation-kind, #inline-math and #math-editing,
  each named by its spec's file name.

## What the work is

What exists today, at each site the work touches:

- **The model.** A text is a tree of runs of characters and spans,
  `design@rootpoint-model@text-node-tree`, whose items are an enumeration that admits a math item and
  a reference and builds neither, `design@rootpoint-model@text-node-kinds`; an area holds a text or
  an image, `Content` of `path@rootpoint-model@src/document.rs`; ids come from one counter,
  `design@rootpoint-model@stable-ids`; `Document::breaches` names the cosmetic breaches of
  `design@rootpoint-model@cosmetic-fallbacks`, `Breach` of `path@rootpoint-model@src/validate.rs`.
- **The file.** RON, `FORMAT_VERSION` 5, written by `save_ron` with ron's extensions
  `UNWRAP_VARIANT_NEWTYPES` and `IMPLICIT_SOME` and the default escaping of strings.
- **The renderer.** `layout_text` lays a text's string out with Parley's `ranged_builder`, each
  span's values over its byte range, with a font stack of the text's family then `BUILTIN_SANS`,
  "Noto Sans" before #builtin-open-sans, four files embedded by `BUILTIN_FONTS`, and after it
  Open Sans then Libertinus Math; `render_slide` draws a text area and an image
  area, and the report names `missing_glyphs`; no `InlineBox` is used; ttf-parser is in no crate's
  tree for the browser's target, `cargo tree -i ttf-parser --target wasm32-unknown-unknown` printing
  nothing.
- **The protocol.** `split_words` reads quoted texts, bare words and the punctuation of a tree;
  `text` takes a tree of quoted texts and spans, `tree` prints one,
  `design@rootpoint-protocol@protocol-transport`; `image` sets an area's content to an image;
  `insert`, `delete`, `wrap`, `unwrap`, `span` and `split-span` edit a text by one operation each,
  `design@rootpoint-protocol@text-edit-commands`.
- **The window.** The caret is a caret position of a text's tree, placed by Parley's `Cursor` on the
  layout the renderer draws, `design@rootpoint-app@edit-in-place`; the shortcuts are tested in
  `App` of `path@rootpoint-app@src/lib.rs`, Ctrl with B calling `App::bold`; the node menu offers
  text and image under "Content", `design@rootpoint-app@node-menu`; the browser's listener
  `keep_editor_keys` of `path@rootpoint-app@src/web.rs` prevents the defaults of the editor's keys.
- **The workspace.** Five crates, `rootpoint-model`, `rootpoint-render`, `rootpoint-protocol`,
  `rootpoint-app` and `xtask`, each declaring `license = "MIT OR Apache-2.0"` on this branch.

**Outside the work, and why:**

- **Naming equations**: `issue@rootpoint-model@equation-names`, on the owner's word of round 2.
- **Configurable keys**: `issue@rootpoint-app@configurable-keys`, on the owner's answer to Q3.
- **A number per row of an equation, and a reference to a row**: #equation-rows numbers an equation
  once; a later consequence.
- **A fallback math font for the missing set**: #math-glyph-fallback; T4 watches for its need.
- **A math font chosen per presentation, from a font asset**: a later consequence, #arg-23.
- **Bullet lists and code snippets**: `issue@rootpoint@more-content-kinds`.
- **LyX's math autocorrection**: off in LyX by default, #arg-62.
- **Copy and paste of a formula**: `issue@rootpoint-app@styled-copy-paste`, read at #inline-math's
  audit.

## What is already decided

**The decisions the design rests on and does not argue again:**

- `goal@rootpoint@no-language-no-build`: math is written in the window with no LaTeX to know, C1.
- `goal@rootpoint@minimal-content-set`: the equation is a kind the goal lists; its options are its
  own, C7.
- `design@rootpoint-render@slide-renderer`, `rejected@rootpoint-render@toolkit-painter`: the
  project's renderer draws every formula, from one layout; no MathML or KaTeX layer.
- `design@rootpoint-render@font-sources`, `rejected@rootpoint-render@system-fonts`: the math font is
  built in, never read from the system.
- `design@rootpoint-render@caller-supplies-assets`: the renderer reads no file; the built-in fonts
  are embedded bytes.
- `design@rootpoint-model@edit-operation-core`, `design@rootpoint-model@validated-documents`,
  `rejected@rootpoint-model@gui-mutation`: every change is an `Op` with its inverse.
- `rejected@rootpoint-model@addressing-by-position`: a reference stores its equation's id, never a
  number.
- `rejected@rootpoint-model@flat-style-runs`, `rejected@rootpoint-model@runs-beside-spans`: a
  formula is an item of the text tree; it carries no span and no styling model of text inside it.
- `rejected@rootpoint-model@untagged-text-items`: the new items are tagged in the file.
- `design@rootpoint-protocol@no-new-breach`, `design@rootpoint-protocol@protocol-addressing`: no
  command makes a dangling reference; a command names an equation by its id.
- `rejected@rootpoint-protocol@json-rpc-commands`: commands stay one line of words.
- `rejected@rootpoint-protocol@whole-text-replacement`: a formula is replaced whole, which renews no
  id, #arg-28.
- `design@rootpoint-app@actions-are-commands`, `design@rootpoint-app@gesture-groups`,
  `design@rootpoint-app@caret-editor-state`, `rejected@rootpoint-app@caret-command`: the caret in a
  formula is editor state; every change is a command, inside the stay's group.
- `design@rootpoint-app@editor-overlay`, `rejected@rootpoint-app@reserved-frame-space`: the marks
  of a formula are drawn over the texture; the placeholder of an empty cell is reserved in every
  display, which is not that rejected design, #arg-26.
- `design@xtask@browser-tests-gated`, `design@xtask@one-tool-binds-and-tests`: the math crate's
  browser tests run in the gates.
- `design@rootpoint@permissive-licence`: every dependency and every built-in file the work adds fits
  MIT OR Apache-2.0, and a built-in file keeps its licence text beside it.

**Each recorded decision the work rewrites, every text that `cargo klarch show` lists as referencing
it, and the slice that judges or updates each.** Each slice's audit runs `cargo klarch show` again on
the heads it changes, and reads each text against the change:

- `design@rootpoint@crate-existence-rule`: its list of crates gains `rootpoint-math`, a member the
  rule admits, at #math-layout, and its sentence that the model's serialization stays inside it is
  rewritten, since the grammar of a formula is the math crate's, at #equation-kind, when the model
  first holds a formula. Referenced by
  `design@rootpoint@gui-only-in-app`, whose text does not change on that account.
- `design@rootpoint@gui-only-in-app`: its graph of dependencies gains the math crate, which the
  renderer depends on from #math-layout and the model from #equation-kind. Referenced at
  `register@rootpoint-protocol@design`, which does not change.
- `design@rootpoint@wasm-from-the-start`: the crates the gate builds gain the math crate,
  #math-layout. Referenced at `path@rootpoint-model@CLAUDE.md` and `path@rootpoint-render@CLAUDE.md`,
  read at that slice.
- `builtin-noto-sans`, now `design@rootpoint-render@builtin-open-sans`: rewritten for Open Sans,
  #builtin-open-sans. Referenced by `design@rootpoint-render@font-sources`.
- `design@rootpoint-render@font-sources`: the built-in fonts gain the math font, #math-layout, and
  the text's fallback to it, #builtin-open-sans. Referenced at `register@rootpoint-model@design`
  (`design@rootpoint-model@cosmetic-fallbacks`) and by
  `tripwire@rootpoint-render@glyph-missing-everywhere`: read at both slices; neither changes.
- `text-padding`, now `design@rootpoint-render@typeset-padding`: an equation is inset by the same
  padding as a text, a member its argument, the wrap width of a text, does not cover; its ground is
  the session's choice, so the head is rewritten at #equation-kind with the reason for the member:
  an equation's edge then stands where a text's would. Referenced by nothing.
- `design@rootpoint-model@inline-content-assets` and `design@rootpoint-model@style-kinds`: a third
  kind of content, whose options are unnamed, #equation-kind. `design@rootpoint-model@style-kinds` is
  referenced by `issue@rootpoint@more-content-kinds`, which stays true.
- `design@rootpoint-model@no-empty-kind`: its sentence that the editor has one kind to enter for
  typing, the text, stops being true when the window edits an equation's formula, #math-editing.
  Referenced at `register@rootpoint-app@design` (twice) and `register@rootpoint-model@design`, read at
  that slice.
- `design@rootpoint-model@save-format`: versions 6 and 7 and `escape_strings(false)`,
  #equation-kind and #inline-math. Referenced at `register@rootpoint-model@design`,
  `path@rootpoint-model@src/file.rs`, `path@rootpoint-model@tests/file.rs`,
  `register@rootpoint-protocol@design`, `register@rootpoint@design` and
  `issue@rootpoint@compatibility-after-real-use`: each read at those slices; the comment of
  `path@rootpoint-model@src/file.rs` changes with the configuration it explains.
- `design@rootpoint-model@edit-operation-core`: an edit of a text tree returns the area's former
  content, which `Op::SetMath` follows; `Op::RetargetReferences` changes several texts at once, and
  its inverse, which restores each, is a member the head gains at #inline-math. Referenced at
  `register@rootpoint-app@design`, `path@rootpoint-model@src/document.rs` and
  `register@rootpoint-protocol@design`, read at that slice.
- `design@rootpoint-model@text-node-kinds`: the two kinds built, #inline-math. Referenced at
  `path@rootpoint-model@src/document.rs`, the doc comment of `TextItem`, which changes with it.
- `design@rootpoint-model@validated-documents`: a reference to an id at or above the document's
  counter is structural, by the head's own argument for an asset id, since a later node would take
  that id, #inline-math. Referenced at `register@rootpoint-model@rejected`,
  `path@rootpoint-model@src/ops.rs` and `path@rootpoint-model@src/validate.rs` (twice), read at that
  slice.
- `design@rootpoint-model@cosmetic-fallbacks`: a reference to an area that holds no equation is a
  new member, #inline-math. Referenced at `issue@rootpoint-app@styled-copy-paste`,
  `register@rootpoint-model@design` (four times), `path@rootpoint-model@src/document.rs`,
  `path@rootpoint-model@src/ops.rs` (three times), `path@rootpoint-model@src/validate.rs` and
  `register@rootpoint-protocol@design` (three times); the doc comment of `Breach` changes with it.
- `design@rootpoint-model@stable-ids`: a duplicated slide redirects its own references,
  #inline-math. Referenced at `issue@rootpoint-model@linked-dividers`,
  `register@rootpoint-model@tripwire` (twice, by `tripwire@rootpoint-model@span-ids-in-the-way`),
  `path@rootpoint-model@src/document.rs`, `path@rootpoint-model@src/ops.rs`,
  `register@rootpoint-protocol@design` (five times) and `register@rootpoint-protocol@rejected`.
- `design@rootpoint-render@area-text-layout`: inline boxes, #inline-math. Referenced at
  `register@rootpoint-app@design`, `register@rootpoint-app@rejected`,
  `register@rootpoint-model@design` and `path@rootpoint-render@src/map.rs`, whose doc comment changes
  with the map.
- `design@rootpoint-protocol@protocol-transport`: the `$…$` literal, #equation-kind, a head of its
  own, `design@rootpoint-protocol@math-in-protocol-grammar`; and the tree's new items,
  #inline-math. Referenced at `register@rootpoint-app@design`,
  `issue@rootpoint-app@styled-copy-paste`, `register@rootpoint-protocol@design`,
  `issue@rootpoint-protocol@style-command-prints-a-line-that-does-not-parse`,
  `register@rootpoint-protocol@rejected` (twice) and `register@rootpoint-protocol@tripwire` (twice, by
  `tripwire@rootpoint-protocol@one-line-tree-written-badly`).
- `design@rootpoint-protocol@text-edit-commands`: the insertion of items, `math` and `retarget`,
  #inline-math. Referenced at `register@rootpoint-app@design` (three times),
  `register@rootpoint-app@rejected`, `path@rootpoint-app@src/actions.rs`,
  `path@rootpoint-app@src/editing.rs`, `path@rootpoint-app@tests/editing.rs`,
  `register@rootpoint-protocol@rejected` and `register@rootpoint-protocol@tripwire`.
- `design@rootpoint-protocol@no-new-breach`: `equation`, `math`, `retarget` and the removals refuse a
  dangling reference, #inline-math. Referenced at `path@rootpoint-app@src/actions.rs` (twice),
  `path@rootpoint-app@src/panels.rs`, `register@rootpoint-model@design` (five times),
  `path@rootpoint-model@src/ops.rs` (three times), `path@rootpoint-protocol@src/session.rs` (three
  times), `path@rootpoint-protocol@tests/properties.rs` and `path@rootpoint-protocol@tests/spans.rs`.
- `design@rootpoint-app@node-menu` and `design@rootpoint-app@tree-outline`: "Equation" in the menu,
  approved by the owner in round 2, and the outline's row of an equation, #equation-kind.
  `design@rootpoint-app@node-menu` is referenced at `register@rootpoint-app@design` (three times),
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/menu.rs`,
  `path@rootpoint-app@src/panels.rs`, `path@rootpoint-app@src/view.rs` and
  `path@rootpoint-app@tests/gestures.rs` (twice); `design@rootpoint-app@tree-outline` at
  `register@rootpoint-app@design` (twice) and `path@rootpoint-app@src/panels.rs`.
- `design@rootpoint-app@edit-entry`: its sentence that every area holding no image holds a text,
  #equation-kind; entering a formula's editing, #math-editing. Referenced at
  `register@rootpoint-app@design`, `register@rootpoint-app@tripwire` (twice, by
  `tripwire@rootpoint-app@empty-text-takes-the-keys`), `path@rootpoint-app@src/actions.rs`,
  `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@tests/editing.rs`,
  `path@rootpoint-app@tests/gestures.rs` (twice) and `register@rootpoint-model@design`.
- `design@rootpoint-app@edit-in-place`, `design@rootpoint-app@boundary-caret-positions`,
  `design@rootpoint-app@editor-overlay` and `design@rootpoint-app@editor-keys-kept-from-browser`:
  the caret in a formula, its marks and the new keys, #math-editing. Referenced, the first at
  `register@rootpoint-app@design` (twice), `path@rootpoint-app@src/actions.rs`,
  `path@rootpoint-app@src/editing.rs`, `path@rootpoint-app@src/view.rs`,
  `path@rootpoint-app@tests/editing.rs` (twice), `path@rootpoint-app@tests/edit_ops.rs` and
  `register@rootpoint-render@design`; the second at `register@rootpoint-app@design` (three times),
  `register@rootpoint-app@rejected`, `path@rootpoint-app@src/actions.rs` (twice),
  `path@rootpoint-app@src/editing.rs` (twice) and `path@rootpoint-app@tests/editing.rs` (four times);
  the third at `path@rootpoint-app@CLAUDE.md`, `register@rootpoint-app@design` (three times),
  `register@rootpoint-app@rejected`, `register@rootpoint-app@tripwire` (twice),
  `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@tests/gestures.rs` and
  `path@rootpoint-app@tests/presenter.rs`; the fourth at `register@rootpoint-app@design`,
  `register@rootpoint-app@rejected`, `path@rootpoint-app@src/lib.rs` and `path@rootpoint-app@src/web.rs`.

**The standing entries the discussion's search returned, each with its outcome:**

- `issue@rootpoint@more-content-kinds`: equations leave it for this milestone, in the commit that
  adds this document.
- `issue@rootpoint-model@linked-dividers`: bears on nothing in the work; its trigger is not met.
- `tripwire@rootpoint-model@span-ids-in-the-way`: its re-entry, the next work on the save format, is
  met at #equation-kind and #inline-math, which read it; `Ref(<id>)` written by hand is its concern.
- `tripwire@rootpoint-model@span-styles-confuse`: bears on nothing; equations name no style,
  #equation-options.
- `tripwire@rootpoint-app@empty-text-takes-the-keys`: read at #math-editing, which enters a formula's
  editing.
- `tripwire@rootpoint-render@glyph-missing-everywhere`: read at #math-layout and
  #builtin-open-sans; the coverage of Open Sans and the missing set answer its concern.
- `tripwire@rootpoint-render@render-within-bounds`: its re-entry is met; AC5 measures it.
- `tripwire@rootpoint-app@keys-confuse-with-the-outline`: read at #math-editing, the next work on the
  app's keys; Alt+M acts only while editing, #arg-29.
- `tripwire@rootpoint-protocol@one-line-tree-written-badly`: read at #inline-math; T2 watches the
  formula literal.
- `tripwire@rootpoint-app@frames-mislead`: read at #math-editing, whose frames of atoms it watches.
- `issue@rootpoint-app@styled-copy-paste`: read at #inline-math's audit.
- `issue@rootpoint-app@abandoned-composition-undo-step`: read at #math-editing, which removes an
  empty formula as an empty span is unwrapped.
- `issue@rootpoint@compatibility-after-real-use`: bears on the two format changes; its trigger, the
  owner's ruling that the project is in real use, is not met.
- `issue@rootpoint-protocol@style-command-prints-a-line-that-does-not-parse`: read at
  #equation-kind, whose `equation` command prints its options.
- `issue@rootpoint@thorough-testing`: bears on nothing new; the property tests of each slice extend
  the existing ones.
- `tripwire@rootpoint-app@browser-pack-too-large`: bears on nothing; it watches a presentation's
  size, and the fonts are the binary's.
- `tripwire@rootpoint-model@inherited-style-in-the-way`: bears on nothing; a split of an equation
  area gives its new area a text, by `design@rootpoint-model@new-area-style`.

**The design and rejected entries the design-entry search graded `bears`, each with its outcome**,
other than those listed above:

- The root: `goal@rootpoint@real-presentations` (served; its talks need math);
  `goal@rootpoint@free-slide-division` (an equation is one piece of content in its area);
  `goal@rootpoint@readable-save-file` (C2); `goal@rootpoint@headless-core` and
  `goal@rootpoint@text-command-protocol` (C3); `goal@rootpoint@runs-in-browser` (C5);
  `design@rootpoint@no-compat-in-initial-dev` (no migration).
- The renderer: `goal@rootpoint-render@same-slide-everywhere` (C4); `goal@rootpoint-render@browser-buildable` (C5).
- The model: `goal@rootpoint-model@headless-changes` (every change an operation);
  `goal@rootpoint-model@browser-buildable` (C5); `goal@rootpoint-model@faithful-readable-file` (C2);
  `design@rootpoint-model@text-node-tree` and `design@rootpoint-model@text-tree-normal-form` (a
  formula is an item, merged runs stay merged around it); `design@rootpoint-model@named-span-styles`
  and `design@rootpoint-model@span-style-overrides` (a formula takes size and colour from its node,
  #arg-11); `design@rootpoint-model@no-empty-kind` (an equation adds what a text lacks, a formula; an
  emptied equation keeps an empty formula).
- The protocol: `goal@rootpoint-protocol@commands-drive-everything` (C3);
  `goal@rootpoint-protocol@browser-buildable` (C5).
- The window: `goal@rootpoint-app@direct-manipulation`, `goal@rootpoint-app@faithful-slide-view`,
  `goal@rootpoint-app@live-agent-edits` and `goal@rootpoint-app@browser-window` (served by the
  slices' window halves); `design@rootpoint-app@gui-toolkit` (egui draws no formula);
  `design@rootpoint-app@selection-keys` (#arg-29); `design@rootpoint-app@no-discard-confirmation`
  (the refusal asks nothing, #arg-40); `design@rootpoint-app@delete-in-caret-node`,
  `design@rootpoint-app@style-action-target`, `design@rootpoint-app@controls-act-at-the-caret`,
  `design@rootpoint-app@split-at-caret` and `design@rootpoint-app@selection-one-level` (read at
  #math-editing's audit, whose keys must not collide with theirs); `rejected@rootpoint-app@one-position-per-edge`
  (not brought back: a formula's edge is two positions, as a span's); `rejected@rootpoint-app@removal-guard`
  (not brought back: a refusal is no confirmation).
- The maintenance tool: `goal@xtask@one-command-runs-every-gate` (every new test runs in the gates).

## Criteria

### An author writes math in the window with no LaTeX to know `##c-no-latex-to-know`

C1. Binding, from `goal@rootpoint@no-language-no-build`. Met by #math-editing-model,
#math-symbol-palette and #math-keys.

### The saved file stays readable and editable by hand, and loads back equal `##c-readable-file`

C2. Binding, from `goal@rootpoint@readable-save-file` and `goal@rootpoint-model@faithful-readable-file`.
Met by #math-storage-form.

### An agent creates and edits math with one-line commands `##c-one-line-commands`

C3. Binding, from `goal@rootpoint-protocol@commands-drive-everything`. Met by
#math-in-protocol-grammar, #math-edit-commands and #reference-retarget.

### One layout is drawn and carries the caret, the same at every size `##c-one-layout`

C4. Binding, from `goal@rootpoint-render@same-slide-everywhere` and
`design@rootpoint-render@area-text-layout`. Met by #math-layout-engine and #math-text-item.

### Everything builds for the browser and works there, keys included `##c-browser`

C5. Binding, from `goal@rootpoint@runs-in-browser` and `design@rootpoint@wasm-from-the-start`. Met
by #math-layout-engine, whose ttf-parser is pure Rust, #math-font and #math-keys.

### A reference survives every edit but its equation's removal `##c-references-survive`

C6. Binding, from `design@rootpoint-model@stable-ids`, a presumption. Met by
#equation-reference-target and #removed-equation-references.

### Each kind has only the options that make sense for it `##c-own-options`

C7. Binding, from `goal@rootpoint@minimal-content-set`. Met by #equation-options.

### The owner's LyX keys work as they do in LyX `##c-lyx-keys`

C8. Weighed, from the owner's message of round 1. Met by #math-keys.

### The render bounds hold `##c-render-bounds`

C9. Weighed, from `tripwire@rootpoint-render@render-within-bounds`. Met if AC5 holds.

## Threads

### A formula is a tree in memory, and a closed LaTeX subset in the file and in commands `##math-storage-form`

Proposed by the session in round 1, on #arg-1, #arg-6 and #arg-7. Its open assumption, #arg-8, was
measured in round 2, #arg-39, and the file written with `escape_strings(false)`, a default "unless
you say otherwise" that round 8's word on the table closes. The rival, the LaTeX string as the model,
edited as source, lost on #arg-9. Approved in round 2: "math-storage-form: agreed on that shape".
Its shape is in "Decided design"; its harvest is the math crate's design home, at #math-layout, and
`design@rootpoint-model@save-format`, at #equation-kind.

### Inline math is one item of a text node, laid out as one inline box `##math-text-item`

Proposed by the session in round 1, from the owner's request for inline math, on #arg-1, #arg-10 and
#arg-11. Approved in round 2: "math-text-item: agreed". Harvested into
`design@rootpoint-model@text-node-kinds` and `design@rootpoint-render@area-text-layout`, at
#inline-math.

### A displayed equation is a kind of content of its own, one per area `##equation-content-kind`

Proposed by the session in round 1, from the owner's "node kind holding a numbered equation", on
#arg-12; LyX's display block lost on #arg-13; the node menu's entry was put to the owner, #arg-14.
Approved in round 2: "equation-content-kind: agreed". Harvested into the model's design home and
`design@rootpoint-app@node-menu`, at #equation-kind.

### An equation holds rows aligned at `&` under one number `##equation-rows`

Proposed by the session in round 1, #arg-15, with Q1. Approved in round 2: "equation-rows: agreed on
rows, I do need them", and Q1: "often aligned multi row, occasionally matrices. I think your default
will cover my use cases." The default of Q1 named matrix, pmatrix and cases; the delta of round 2
added bmatrix, which the table of round 7 shows and round 8's word closes. Built at #math-layout,
its rows and matrices harvested into the math crate's design home; its one number, at
#equation-kind.

### Equations are numbered by computation, in the order of slides then of each slide's tree `##equation-numbering`

Proposed by the session in round 1, #arg-16 and #arg-17. Approved in round 2: "equation-numbering
agreed". Harvested into the model's design home, at #equation-kind.

### A reference names its equation's area by id, and shows its number `##equation-reference-target`

Proposed by the session in round 1 as a fork against labels, #arg-18 and #arg-19. Closed by the owner
in round 2 on ids, for a first shape: "Maybe we can go for the simpler shape you propose at first, a
leave a todo issue with leads for the equation labelling", with the owner's lead "Name would be a
property of the node kind, set at the same place text styles and image format are setup, in the
right sidebar", #arg-32; the issue is `issue@rootpoint-model@equation-names`. Harvested into the
model's design home, at #inline-math.

### Removing or converting a referenced equation is refused, and the refusal names the references `##removed-equation-references`

Proposed by the session in round 1, #arg-20; allowing the removal lost on #arg-21. Approved in round
2 on a condition: "this is better than allowing the removal I think, under the condition that there
must be an easy way to retarget all references", #arg-33, which #reference-retarget meets. Harvested
into `design@rootpoint-protocol@removed-equation-references`, at #inline-math.

### One operation moves every reference from one equation to another `##reference-retarget`

Proposed by the session in round 2, carrying the owner's condition, #arg-40, with Q7. Approved in
round 3: "reference-retarget agreed". Harvested into `design@rootpoint-model@reference-retarget`, at
#inline-math.

### Math is laid out by the project's own code from the font's MATH table, a box per atom and per cell `##math-layout-engine`

Proposed by the session in round 1, #arg-22, with the survey of Q5. Closed `argued` by the session
in round 4, on the survey: #arg-57, #arg-58 and #arg-59; the rivals lost there are in "Losing
alternatives". No word of the owner addresses it but round 8's word on the table. Harvested into the
math crate's design home, at #math-layout.

### The built-in math font is Libertinus Math `##math-font`

Proposed by the session in round 1, #arg-23, with Q2; the thesis's math read in rounds 3 and 4,
#arg-48 and #arg-53; the samples and the measurements, #arg-54 and #arg-63. Chosen by the owner in
round 5: "Among the sample, my preference goes to Libertinus Math." The owner's hedge of round 6,
"Before I fully settle on this choice", answered by #arg-67 and settled by the owner's words of round
7, #arg-73 and #arg-74. Harvested into the render crate's design home, at #math-layout.

### A command whose glyph the math font lacks is drawn missing and reported, with no fallback math font `##math-glyph-fallback`

Proposed by the session in round 6, #arg-68; STIX Two Math as a fallback lost on #arg-69, weighed by the
owner in #arg-73. Approved in round 7: "math-glyph-fallback: agreed on default. the missing set here is not
much of a problem. Keep the exact missing set recorded in the porject btw." and in round 8: "The
missing set can be recorded as you proposed", #arg-79. Harvested into a head of its own, a head of
absence beside the head of #math-font, which carries the missing set and the test that pins it, at
#math-layout.

### Open Sans replaces Noto Sans as the built-in text family, and a text falls back to the math font `##builtin-text-family`

Raised by the owner in rounds 5 and 6, #arg-64 and #arg-66; proposed by the session in round 6,
#arg-70, #arg-71 and #arg-72; measured in round 7, #arg-75; the fallback added as a member, #arg-76;
shape B lost on #arg-77. Chosen by the owner in round 7: "I have a slight preference for Open Sans
over Noto Sans"; accepted in round 8: "builtin-text-family accepted on this default". Harvested into
the render crate's design home, at #builtin-open-sans.

### A formula is edited in place, structurally, as in LyX `##math-editing-model`

Proposed by the session in round 1, #arg-2 and #arg-24; a LaTeX source field lost on C1. Approved in
round 2: "math-editing-model agreed". Harvested into the app's design home, at #math-editing.

### Any command of the table typed after `\` becomes its symbol or structure `##latex-command-set`

Raised by the owner in round 2, #arg-34; proposed by the session in round 2, #arg-41. Its shape
approved in round 3: "latex-command-set agreed on shape". The source of its table, W3C `unicode.xml`,
closed on its argument in round 4, after the owner's choice of licence, #arg-61; LyX's
autocorrection left out, #arg-62. Harvested into the math crate's design home at #math-layout, but
for the table's limit to the font's characters, whose reason, a palette that offers no command the
font draws as a box, #arg-68, is harvested with the palette, at #math-editing.

### While a formula is edited, the Selection tab shows a palette of symbols and structures `##math-symbol-palette`

Proposed by the session in round 1, #arg-25. Approved in round 2: "math-symbol-palette: agreed."
Harvested into the app's design home, at #math-editing.

### An empty cell keeps a placeholder box in every display, and the overlay draws its outline `##empty-cell-placeholder`

Proposed by the session in round 1, #arg-26. Approved in round 2: "empty-cell-placeholder: agreed, I
believe this is needed and an unfinished formula is not supposed to exist in a real presenter run",
#arg-35. Built at #math-layout and #math-editing; harvested into the math crate's design home.

### In commands, a formula is written between `$` signs with no escaping, and a reference as `{ref n<id>}` `##math-in-protocol-grammar`

Proposed by the session in round 1, #arg-27. Approved in round 2: "math-in-protocol-grammar: agreed".
Harvested into `design@rootpoint-protocol@math-in-protocol-grammar`, at #equation-kind, and into
`design@rootpoint-protocol@protocol-transport` for the tree's items, at #inline-math.

### The window changes a formula by replacing that one formula whole `##math-edit-commands`

Proposed by the session in round 1, #arg-28. Approved in round 2: "math-edit-commands: agreed".
Harvested into `design@rootpoint-protocol@math-edit-commands`, at #inline-math.

### LyX's math keys are copied exactly, the browser build disabling a binding that conflicts `##math-keys`

Proposed by the session in round 1, from the owner's request, #arg-5, #arg-29 and #arg-30. Approved in
round 2: "math-keys: agreed on copying the keys from lyx exactly. The browser mode can have a few
bindings disabled in case of conflicts, not a problem.", #arg-36. Which of LyX's keys are bound,
and which are left out with their grounds, is D2, ruled by the owner in round 9 on its default: the
keys LyX gives an action this milestone has are bound, and those whose action it lacks, as T's
types of equation or Shift+N's number per row, are not. Harvested into the app's design home, at
#math-editing.

### The math keys are a built-in table of key sequences to actions `##keymap-table`

Proposed by the session in round 2, from the owner's answer to Q3, #arg-44; moving every key now lost
on #arg-45. Accepted in round 3: "keymap-table accepted". The configurable keymap is
`issue@rootpoint-app@configurable-keys`. Harvested with #math-keys, at #math-editing.

### An equation has a size mode and a colour, its own, unnamed `##equation-options`

Proposed by the session in round 1, #arg-31; a text style named by the equation lost on C7. The owner
asked in round 2 for fitting, #arg-37, which #equation-fit answers and this thread absorbs. Approved
as finally shaped by round 8's word on the table. Harvested into the model's design home, at
#equation-kind.

### An equation fits its area by one layout scaled, or is held at most at a size `##equation-fit`

Proposed by the session in round 2, from the owner's question, #arg-42 and #arg-43, with Q6. The
owner: "equation-fit: both could be made available, no ? If yes, I'd go for this." in round 3; the
session answered yes and named the two modes, #arg-50; linearity confirmed by the survey in round 4,
#arg-60. Harvested with #equation-options, at #equation-kind.

## Arguments

### The record already expects math as a tree `##arg-1`

Round 1, the session. Bears on #math-storage-form and #math-text-item.
`design@rootpoint-model@text-node-kinds` "already admits a math item and a reference item in a text";
`design@rootpoint-model@text-node-tree` "rejects flat runs partly because 'it cannot hold a
fraction's two cells, which maths will need'."

### Math is set aside in the caret rule `##arg-2`

Round 1, the session. Bears on #math-editing-model and #math-text-item. The owner's ground in
`design@rootpoint-app@boundary-caret-positions` "sets math aside: trees 'rarely go deep, maths
aside, where LaTeX has the same problem'."

### The open issue held leads `##arg-3`

Round 1, the session. Bears on #math-text-item, #math-font and #math-layout-engine. The issue on
content kinds recorded "Parley's `InlineBox`, Noto Sans having no `MATH` table, and one layout shared
by inline and display math", and "an unchecked claim from the first prototype's survey: Rust has no
small, mature TeX-math renderer."

### Rejected designs a proposal must not bring back `##arg-4`

Round 1, the session. Bears on #math-layout-engine, #math-font, #equation-reference-target,
#math-text-item and #empty-cell-placeholder. "no MathML or KaTeX layered over the slide, and no
system math font; … a reference never stores a number; … the seam between two styling models".

### The owner's memory of LyX's keys, checked against its source `##arg-5`

Round 1, the session. Bears on #math-keys. "It is correct on all three points": `M-m f` gives
`\frac`, `M-m g a` gives `\alpha`; "The Alt+M table has 122 bindings in total"; "Nothing in rootpoint
binds M today, with or without a modifier".

### LaTeX text in the file beats a RON tree `##arg-6`

Round 1, the session. Bears on #math-storage-form and C2. "LaTeX math is the form a person and an
agent both already read and write. The structured rival … is about 10 times longer for
`\frac{a}{b}`, and nobody writes it by hand."

### The cost of the storage form, and its edge cases `##arg-7`

Round 1, the session. Bears on #math-storage-form. The cost is "a parser and a writer for the
subset"; an unbalanced brace "is a syntax error of the file. The file is refused"; an unknown command
"is kept as an 'unknown' atom and drawn in red … a cosmetic breach".

### Raw strings in ron, assumed `##arg-8`

Round 1, the session. Bears on #math-storage-form. "that ron 0.12 can write `r"..."` raw strings … I
measure it before this thread closes."

### The source as the model fails C1 `##arg-9`

Round 1, the session. Bears on #math-storage-form and #math-editing-model. The nearest rival is "the
LaTeX string as the model itself, edited as source. … C1 defeats it there."

### A span holds no cells, and an area is not inline `##arg-10`

Round 1, the session. Bears on #math-text-item. "Math as a kind of span: a span holds characters,
not cells. Inline math as an area of its own: then it is no longer inline."

### The caret and the style of a formula in a text `##arg-11`

Round 1, the session. Bears on #math-text-item. "The caret enters and leaves the formula like an edge
of a span … the caret is a path of cells and offsets, held as editor state only"; "Bold and italic do
not apply to it, because math has its own font rules, as in TeX."

### The goals name the equation kind `##arg-12`

Round 1, the session. Bears on #equation-content-kind. "`goal@rootpoint@minimal-content-set` names
'equation' as a kind. `goal@rootpoint@free-slide-division` places each piece of content in its own
area. Your intuition points the same way."

### LyX's display block duplicates the areas `##arg-13`

Round 1, the session. Bears on #equation-content-kind. "a text area that flows equations between
paragraphs duplicates that placement with a second mechanism"; blocks in Parley "would be new work:
this is my reading … not a measurement."

### A member of the node menu goes to the owner `##arg-14`

Round 1, the session. Bears on #equation-content-kind. "That adds a member to
`design@rootpoint-app@node-menu`, whose ground is your word. So I put it to you as a proposal".

### Rows under one number, against a number per row `##arg-15`

Round 1, the session. Bears on #equation-rows. "rows separated by `\\` and aligned at `&`, under one
number … LaTeX's `equation` with `split`." A number per row means "a reference would name a row
inside an equation and no longer just an area."

### Numbering in tree order survives a drag `##arg-16`

Round 1, the session. Bears on #equation-numbering. "the areas in depth-first tree order … So
dragging a divider never changes a number. A rival that orders by position on the slide would
renumber when a drag moves one area above another."

### Numbered by default, across the presentation `##arg-17`

Round 1, the session. Bears on #equation-numbering. "A new equation area is numbered, since you
called the kind 'a numbered equation'. LyX's default for a new display formula is unnumbered." The
count runs over the whole presentation: "`stdcounters.inc` has no `Within`."

### Ids against labels `##arg-18`

Round 1, the session. Bears on #equation-reference-target. Ids: "There is no new namespace, no
uniqueness rule and no rename operation. Cost: the file reads `Ref(n7)`." Labels: "every referenced
equation needs a name the author types … plus a uniqueness check and a rename operation".

### A duplicated slide keeps its own references `##arg-19`

Round 1, the session. Bears on #equation-reference-target. "a reference on the duplicated slide to an
equation on the same slide is redirected to the copy. A reference to an equation elsewhere keeps its
target."

### A dangling reference is a breach no command may create `##arg-20`

Round 1, the session. Bears on #removed-equation-references. "`design@rootpoint-protocol@no-new-breach`
says no command creates a breach … `Op::RemoveAsset` already refuses … in the same way." A file with
a dangling reference loads, "and the reference draws `(??)`."

### Allowing the removal reverses a decision `##arg-21`

Round 1, the session. Bears on #removed-equation-references. "allow the removal and draw `(??)`, as
LaTeX does. It breaks a recorded decision, so it would have to go through a reversal."

### Boxes per atom are needed `##arg-22`

Round 1, the session. Bears on #math-layout-engine and C4. "the layout must give the box of every
atom and every cell, so that the overlay can put the caret in a numerator … a typesetter that returns
only glyphs or paths, as KaTeX and MathJax do, cannot place a caret in a cell."

### The constraints on a math font `##arg-23`

Round 1, the session. Bears on #math-font. "the font comes from the binary, never from the system
… It is OFL-licensed like Noto Sans, and it has a `MATH` table." "Choosing the font per presentation,
from a font asset: later, under a tripwire."

### Editing in place, as LyX does `##arg-24`

Round 1, the session. Bears on #math-editing-model. "the caret moves in cells as `Cursor::mathForward`
does … Backspace on a formula selects it first … a formula left empty … is removed". The rival, "a
LaTeX source field with a live preview … makes LaTeX the way to write math in the GUI, which fails
C1."

### The keys cover too few symbols for C1 `##arg-25`

Round 1, the session. Bears on #math-symbol-palette. "the Alt+M table covers about 60 symbols.
Without a palette, ≤, →, ∈, ∀ … can only be typed as `\leq`. That is LaTeX, and it fails C1."

### A placeholder in every display is not the rejected reserved space `##arg-26`

Round 1, the session. Bears on #empty-cell-placeholder. "TeX gives an empty cell zero width, so the
caret would have nowhere to stand. … that design reserved space in the editor alone … Here the space
is the same in the editor and in presenter mode".

### A literal with no escapes for formulas `##arg-27`

Round 1, the session. Bears on #math-in-protocol-grammar. "inside a quoted string, every backslash
would be doubled (`"\\frac"`). Agents get that wrong … `$…$` is the form agents know from LaTeX and
Markdown."

### Replacing a formula whole loses nothing `##arg-28`

Round 1, the session. Bears on #math-edit-commands. "atoms carry no ids. So replacing one formula
whole loses nothing, unlike `rejected@rootpoint-protocol@whole-text-replacement`." Commands by cell
path make "the grammar bigger … for no gain".

### The keys do not meet the node keys `##arg-29`

Round 1, the session. Bears on #math-keys. "Alt with the arrows … acts only outside editing. Alt+M
acts only while editing." Two assumptions not measured: "Firefox gives Ctrl+M to muting the tab, and
no Firefox menu uses Alt+M." On a Mac, "Option+M types µ … I would not handle that until it is
measured."

### LyX's oddities are copied `##arg-30`

Round 1, the session. Bears on #math-keys and C8. "the point of the keys is your muscle memory, so a
'fixed' table would cost exactly what C8 asks to avoid." Left out: the sub-tables T, C and W, and M,
D, L and O.

### An equation's options need no name `##arg-31`

Round 1, the session. Bears on #equation-options and C7. "two options … would save one click per area
if named." A text style named by the equation: "Its font, bold, italic and alignment have no meaning
for an equation, which fails C7."

### Naming would help, later `##arg-32`

Round 2, the owner. Bears on #equation-reference-target. "I do believe a naming would help here.
Name would be a property of the node kind, set at the same place text styles and image format are
setup, in the right sidebar."

### An easy way to retarget `##arg-33`

Round 2, the owner. Bears on #removed-equation-references and #reference-retarget. "this is better
than allowing the removal I think, under the condition that there must be an easy way to retarget all
references."

### LyX turns any typed command into its symbol `##arg-34`

Round 2, the owner. Bears on #latex-command-set. "lyx accepts typing any backslash command of latex
and replaces it with correct symbol/node (like `\frac` or `\leq`). Can't we support that too ?"

### No unfinished formula in a real talk `##arg-35`

Round 2, the owner. Bears on #empty-cell-placeholder. "an unfinished formula is not supposed to exist
in a real presenter run."

### The browser may lose a few bindings `##arg-36`

Round 2, the owner. Bears on #math-keys. "The browser mode can have a few bindings disabled in case of
conflicts, not a problem."

### Fitting would be convenient `##arg-37`

Round 2, the owner. Bears on #equation-options and #equation-fit. "is it possible to have an equation
automatically fit to the allocated area ? this would be very convenient, but I suspect quite hard to
do."

### The owner's answers to Q1 to Q4 `##arg-38`

Round 2, the owner. Bears on #equation-rows, #math-font, #keymap-table. "Q1: often aligned multi row,
occasionally matrices." "Q2 I quite like noto sans, if it has a matching math font I'd like to use
it." "Q3: I would like for keybinds to be configurable later." "Q4: actually, I will need bullet list
too".

### ron writes raw strings, measured `##arg-39`

Round 2, the session. Bears on #math-storage-form. "With `PrettyConfig::escape_strings(false)`, it
writes `Math(r#"\frac{a}{b} + \alpha"#)`. That form reads back equal … The setting applies to the
whole file … The rival, keeping the escapes, writes every backslash of a formula twice."

### Retargeting is one operation, and the refusal asks nothing `##arg-40`

Round 2, the session. Bears on #reference-retarget. "`retarget n7 n9` … one operation and one undo
step … The refusal asks no question, so it is not a confirmation". The rival, "retarget to… and
remove", is "a dialog in the middle of a Delete, which the editor avoids everywhere else."

### Typing commands was approved; its extent is the table `##arg-41`

Round 2, the session. Bears on #latex-command-set and #math-editing-model. "What is open is the extent
of the set. My proposal is every command that has a glyph in the math font, plus the structures".

### Fitting is linear, if the MATH values are `##arg-42`

Round 2, the session. Bears on #equation-fit. "a formula laid out once at size s and scaled by k is
exactly the formula laid out at size k·s. Our glyphs are unhinted … k = min(area width ÷ formula
width, area height ÷ formula height)".

### Fit both ways, or shrink only `##arg-43`

Round 2, the session. Bears on #equation-fit. Fit both ways: "The area becomes the size control …
Cost: two equations on adjacent slides come out at different sizes". Shrink only: "Sizes stay
consistent across slides, and a large area leaves white space." The number "stays at Body size and
does not scale."

### The cost of configurability lies in the form of the keys `##arg-44`

Round 2, the session. Bears on #keymap-table. "It depends on the form the fixed keys take now, not on
the order … If the math keys enter as a table now, making them configurable later is only a loader".

### Moving every key now is the same work earlier `##arg-45`

Round 2, the session. Bears on #keymap-table. "This is the same work done earlier, and it puts the
current key tests at risk in a milestone about math."

### The thesis's font, liked `##arg-46`

Round 3, the owner. Bears on #math-font. "can you look at which font I used in my thesis … I like it a
lot, can you look whethe rit can be vendored ?"

### Both fit modes, if possible `##arg-47`

Round 3, the owner. Bears on #equation-fit. "both could be made available, no ? If yes, I'd go for
this."

### The thesis's text is Libertine, its math Computer Modern `##arg-48`

Round 3, the session. Bears on #math-font. The thesis sets its text in Linux Libertine and its math
in Computer Modern; the defense slides set their text in Open Sans and their math in Computer Modern.

### Serif math beside sans text was already shown `##arg-49`

Round 3, the session. Bears on #math-font. "Your defense used Computer Modern math with Open Sans
text, so you have already shown math in that pairing."

### Two modes, and no third `##arg-50`

Round 3, the session. Bears on #equation-fit. "`Fit`, the default … `AtMost(size)`: that size, shrunk
only when the equation does not fit. A third mode, a fixed size that overflows its area, would only
differ by overflowing".

### The thesis's math does not look like Computer Modern `##arg-51`

Round 4, the owner. Bears on #math-font. "on the pdf, it does not look like computer moderm to me (for
math) but I could be mistaken. can you check on the pdf ?"

### A licence as wide as possible `##arg-52`

Round 4, the owner. Bears on #latex-command-set. "I plan to make the project open source ultimately …
Iwant it as widely available as possible for everyone and everywhere".

### The thesis's math is Computer Modern, measured `##arg-53`

Round 4, the session. Bears on #math-font. "`pdffonts -f 60 -l 60` lists Linux Libertine … CMMI12,
CMMI8, CMR12, CMSY10, CMEX10 and esint10 … Because the page has no figure, these fonts cannot come
from an embedded figure." `main.lyx` has `\font_math "auto"`.

### The samples show each font's own glyphs `##arg-54`

Round 4, the session. Bears on #math-font. "The browser draws the formulas from MathML with each font
embedded … The browser's layout is not ours, but the glyphs, weights and proportions are the fonts'
own".

### MIT OR Apache-2.0 fits the dependencies and the fonts `##arg-55`

Round 4, the session. Bears on #latex-command-set. "I measured `cargo metadata` over the 513 external
packages … No dependency forces a copyleft licence." "a GPL table, such as LyX's `lib/symbols`, cannot
be embedded."

### A copyleft licence narrows who builds on it `##arg-56`

Round 4, the session. Bears on #latex-command-set. A copyleft licence "stops companies, and
permissively licensed projects, from building on rootpoint."

### No typesetter gives boxes tied to the formula `##arg-57`

Round 4, the session, from the survey. Bears on #math-layout-engine. "caret placement needs boxes tied
to our formula tree's atoms and cells, and only mathtex has such ids." Each rival is in "Losing
alternatives".

### `ssty` through shaping is not verified `##arg-58`

Round 4, the session. Bears on #math-layout-engine. "Whether harfrust, which parley already links,
can do that for one run is not verified. It becomes an acceptance criterion."

### ttf-parser rather than a second read-fonts `##arg-59`

Round 4, the session. Bears on #math-layout-engine. "The rival, read-fonts 0.45, would link a second
copy of read-fonts until parley moves."

### Linearity confirmed `##arg-60`

Round 4, the session, from the survey. Bears on #equation-fit. "The MATH specification gives every
value in font units, or as a percentage. The only exception is device tables … No candidate font uses
one on its constants."

### The table from `unicode.xml` `##arg-61`

Round 4, the session. Bears on #latex-command-set. "Its licence is permissive". "unicode-math's table
is LPPL. Embedding it would add LPPL's renaming rule to the binary … KaTeX's table is MIT but has a
fifth of the entries."

### LyX's autocorrection is off by default `##arg-62`

Round 4, the session. Bears on #latex-command-set and #math-editing-model. "LyX ships it off
(`autocorrection_math = false`). Since you asked for LyX's defaults, I leave it out."

### The fonts measured for the choice `##arg-63`

Round 4, the session. Bears on #math-font. Against Noto Sans Math: "scripts at 60%/50% … no math
kerning; a single weight". Against Libertinus Math: "operators 220/256, supplemental 23/256".

### Libertinus for text and math `##arg-64`

Round 5, the owner. Bears on #math-font and #builtin-text-family. "my preference goes to Libertinus
Math. Since it is also the one I chose for my thesis text, I am considering making it the default for
both text and math".

### The text default is a recorded decision `##arg-65`

Round 5, the session. Bears on #builtin-text-family. "Making Libertinus the text default too would
change a recorded decision, `builtin-noto-sans`, so I gather the facts first"; the head is now
`design@rootpoint-render@builtin-open-sans`.

### The text font was chosen in a rush `##arg-66`

Round 6, the owner. Bears on #builtin-text-family and #math-font. "The current font was chosen in a
rush, without putting much thought on it. About libertinus Math: which symbols does it lack though ?"

### Libertinus lacks 44 commands of KaTeX `##arg-67`

Round 6, the session. Bears on #math-font and #math-glyph-fallback. "Libertinus Math has a glyph for
all but 44 … The missing ones are the less common `amssymb` signs, plus `\diamond` and the two small
triangles." Latin Modern Math lacks 29, Noto Sans Math 22.

### No fallback: report the missing glyph `##arg-68`

Round 6, the session. Bears on #math-glyph-fallback. "The glyph is drawn as the font's missing-glyph
box and reported, as a missing text glyph already is … The command palette shows only the commands
the font has".

### STIX as fallback costs 0.8 MB for 44 rare symbols `##arg-69`

Round 6, the session. Bears on #math-glyph-fallback. "STIX Two Math covers all 2448 commands for 819
KB more in every build … This trades about 0.8 MB against 44 rare symbols."

### The owner's preference suffices for the text font `##arg-70`

Round 6, the session. Bears on #builtin-text-family. Its head's "ground is the session's choice at
the renderer's design audit, not yours, so your preference is enough to change it."

### One family or two `##arg-71`

Round 6, the session. Bears on #builtin-text-family. Shape A, "one built-in family, replacing Noto
Sans"; shape B, "two built-in families … Cost: one more family in every build, about 1 to 2 MB." No
saved presentation is migrated.

### Coverage must be measured `##arg-72`

Round 6, the session. Bears on #builtin-text-family. "Noto Sans covers far more scripts than
Libertinus does … this is an assumption until I measure it."

### The missing set is not a problem; keep it recorded `##arg-73`

Round 7, the owner. Bears on #math-glyph-fallback and #math-font. "the missing set here is not much of
a problem. Keep the exact missing set recorded in the porject btw."

### Open Sans with Libertinus Math `##arg-74`

Round 7, the owner. Bears on #builtin-text-family and #math-font. "the looks I prefer and find easier
to read are either Noto Sans or Open Sans together with Libertinus Math. I have a slight preference
for Open Sans over Noto Sans."

### Open Sans's `noto-set` build loses no coverage `##arg-75`

Round 7, the session. Bears on #builtin-text-family. "The `noto-set` build covers everything Noto
Sans covers, and also Hebrew and 13 math operators, among them a real minus sign". The plain build
covers "21/208 of Latin Ext-B and 101/256 of Vietnamese".

### Text lacks arrows; the math font has them `##arg-76`

Round 7, the session. Bears on #builtin-text-family. "Both fonts lack every arrow (0/112) … Libertinus
Math has the arrows." The fallback depends on a condition "not yet verified: that fontique can fall
back character by character".

### One family when one was named `##arg-77`

Round 7, the session. Bears on #builtin-text-family. "Shape B, two text families, would add about 2 MB
to every build when you named a single preferred font."

### The premortem defeats nothing `##arg-78`

Round 7, the session. "No premortem cause defeats a decided thread, so nothing reopens."

### The missing set is pinned by a test `##arg-79`

Round 7, the session. Bears on #math-glyph-fallback and #math-font. "A test in the render crate pins
that exact set against the font's character map, so any change of font or version fails the test".
"A test is the form that notices when the font changes."

## New names, in one place

Every name is an illustration of a shape; a slice's audit may rename one, in place here.

```text
crates/rootpoint-math/                 the math crate, new
  Formula, Atom, List                  the tree of a formula: a grid of rows of cells, a cell a List
  MAX_DEPTH                            how deep atoms nest, 64
  Formula::parse, Formula::to_latex    the LaTeX subset, read and written
  COMMANDS, ABSENT                     the table of commands, and the commands of characters the font
                                       lacks, generated
  MathFont, MathFont::new              a font file's bytes: cmap, metrics, MATH and GSUB, read by ttf-parser
  lay_out, FormulaLayout, AtomBox      the layout: a box per atom and per cell, and its glyphs
  CellPath                             a path of cells and an offset in a row, for the caret
crates/rootpoint-model/src/document.rs
  Content::Equation(EquationContent)   new kind
  EquationContent { formula, numbered, size, colour }
  EquationSize::{Fit, AtMost(f64)}
  TextItem::Math(Formula), TextItem::Ref(NodeId)
  Document::equation_numbers           computed numbers
crates/rootpoint-model/src/ops.rs
  Op::SetMath, Op::RetargetReferences
crates/rootpoint-model/src/validate.rs
  Breach::DanglingReference
crates/rootpoint-render/src/
  draw_formula                         draws a FormulaLayout, crate-private
  area_formula                         an equation area's layout, origin and scale, as drawn
  Report::unknown_commands             the formulas holding an unknown command
  BUILTIN_MATH                         Libertinus Math's bytes
crates/rootpoint-render/src/formula.rs  draw_formula, and the image test of AC1, ignored
crates/rootpoint-protocol/src/command.rs
  Word::Math(String)                   the `$…$` literal
  Command::Equation, Command::Math, Command::Retarget
  TreeItem::Math(String), TreeItem::Ref(NodeId)
crates/rootpoint-app/src/
  Keymap, KeySequence                  the built-in table of key sequences
tools/xtask/src/main.rs
  cargo x math-commands                the generator of COMMANDS from unicode.xml and a font
```

Existing: `Content`, `TextItem`, `Op`, `Breach`, `Command`, `TreeItem`, `Word`, `split_words`,
`layout_text`, `area_text`, `AreaText`, `TextMap`, `FontSet`, `BUILTIN_FONTS`, `BUILTIN_SANS`,
`FORMAT_VERSION`, `save_ron`, `keep_editor_keys`.

## Decided design

### #math-storage-form

The model holds `Formula`, a tree: a formula is rows, separated by `\\`; a row is atoms, aligned at
`&`; an atom is one of the kinds of `Atom` in `path@rootpoint-math@src/lib.rs`. The file and every command hold the
formula as LaTeX, written by `to_latex` and read by `parse`, over the closed subset of the mapping
table "A LaTeX construct, to the atom it reads as". Text that does not parse makes the file refused,
as a broken RON file is, and a command refused: an unbalanced brace, a second superscript or a second
subscript on one nucleus, as in `x^a^b`, as TeX refuses it, an `\end` that closes no
`\begin`, a `\right` with no `\left`, a missing argument, an `&` or a `\\` outside a grid, an
environment the subset does not know, or atoms nested deeper than the bound `MAX_DEPTH`, 64, which
keeps reading and laying out from overflowing the stack. An unknown command is an atom of its own that takes no argument, so the braced groups after
it stay groups and write back as they were read; it is drawn red, and the renderer's report names it,
as it names a missing glyph, so it is no breach of the document and a command may write one. The file
sets ron's `escape_strings(false)`, so formulas read as raw strings. Nearest rival, the LaTeX string as the model edited as source: fails C1, #arg-9. A RON tree
of atoms: fails C2, #arg-6.

### #math-text-item

`TextItem::Math(Formula)` counts as one offset in its text node. Its layout is placed in the text's
one Parley layout as an `InlineBox` of the formula's width, height and baseline. It takes size and
colour from its node's resolved values; bold and italic do not apply to it. The caret enters and
leaves it like a span's edge; inside it, the caret is a `CellPath`, editor state alone. Nearest rival,
a span of math: holds no cells, #arg-10.

### #equation-content-kind

`Content::Equation(EquationContent)`, one per area. The formula is centred in the area less its
padding; the number `(n)` stands at the area's right edge, on the formula's axis. "Equation" is an
entry of the node menu's "Content". Nearest rival, a display block in a text: duplicates the areas,
#arg-13.

### #equation-rows

Rows separated by `\\`, aligned at `&`, under one number centred vertically on the rows; matrix,
pmatrix, bmatrix and cases are atoms. Nearest rival, a number per row: a reference would name a row,
#arg-15.

### #equation-numbering

`Document::equation_numbers` gives each equation whose `numbered` is true its number, from 1, slides
in their order, then the areas of a slide in depth-first order of its tree. The number is never
stored. Alt+M N toggles `numbered`; a new equation is numbered. Nearest rival, order by position on
the slide: a divider drag would renumber, #arg-16.

### #equation-reference-target

`TextItem::Ref(NodeId)` names an equation area, written `Ref(7)` in the file, since an id is a plain
number there, and `{ref n7}` in commands; it draws `(n)`, the equation's current number, in its
node's style. The window inserts one from a picker listing the equations by number. A duplicated
slide redirects its own references to the copies. `Op::Swap`, which the `swap` command and the
window's drag of an area run, carries the references with the contents it exchanges, so a moved
equation keeps its references, D4. Nearest rival, labels: a namespace, a uniqueness
rule and a rename, #arg-18; deferred to `issue@rootpoint-model@equation-names`.

### #removed-equation-references

The commands that remove an equation area, or a split or a slide that holds one, or turn it into
another kind, refuse while a text that the command leaves in place references it, naming the texts, by
`design@rootpoint-protocol@no-new-breach`; a reference inside the slide removed goes with it.
`apply` accepts each, as `design@rootpoint-model@cosmetic-fallbacks` has it, so an undo, as of a
duplicated slide, which removes the copy, is never refused. A file holding a reference to an area
that holds no equation, at an id below the document's counter, loads; the reference draws `(??)`,
`Breach::DanglingReference`. A reference to an id at or above the counter is structural, and refused
at load and by `apply`, by `design@rootpoint-model@validated-documents`. Nearest rival, allowing
the removal: reverses `design@rootpoint-protocol@no-new-breach`, #arg-21.

### #reference-retarget

`Op::RetargetReferences { from, to }` moves every reference to `from` onto `to`; the command
`retarget` refuses it unless `to` holds an equation; its inverse restores each text it changed to its
former content, as `design@rootpoint-model@edit-operation-core` has an edit of a text tree do, so it
moves back only those it moved. The command `retarget` replies the
count. The Selection tab of an equation shows "Referenced n times" and "Retarget all to…"; a refused
removal's message in the status line names the count and the control. Nearest rival, a dialog that
retargets and removes: a question in the middle of a Delete, #arg-40.

### #math-layout-engine

The math crate lays a formula out from the font file's bytes, all of which it reads with ttf-parser
0.25: the cmap, the glyphs' advances and bounds, the MATH table and the `ssty` lookups of GSUB, which in Libertinus Math 7.051 substitute the six primes alone, ′ to ‷, as
measured at #math-layout's audit, so its other glyphs are set in scripts at their base form, scaled.
It gives a box per atom and per cell, each tied to its place in the formula, with the glyphs to draw and
their positions; the renderer draws them. An equation is laid out in display style, a math item of a
text in text style, and scripts in script and scriptscript styles, as TeX's rules have them, each
style taking the MATH constants that name it. A script's glyphs are the font's `ssty` forms, chosen
by the layout before it measures them, so the boxes are those of the glyphs drawn. The space between
two atoms comes from their classes, by the mapping table "A pair of classes, to the space between
them"; a character's class is the type `unicode.xml` gives it, by the mapping table "A `unicode.xml`
type, to the class of an atom". A letter takes the code point of its font change, by the mapping table
"A character under a font change, to the code point drawn". Rows are set baseline to baseline at
1.2 em at least, and further apart where their ink would come closer than the font's
`mathLeading`, a default of the session that #math-layout's audit may revise; the `&`s of a row
alternate right and left alignment, as amsmath's `align` does. A `\text{…}` run is set in the math
font's upright Latin glyphs. Device tables are ignored. Nearest rival, a fork of KenyC/ReX: no node identity, its own parser and node types, #arg-57.

### #math-font

Libertinus Math 7.051, OFL 1.1, embedded with its licence; the missing set is pinned by a test, and
recorded in the head at the harvest. Nearest rival, Noto Sans Math: matches Noto Sans, but its
scripts are small and it has no kerning, and the owner chose Libertinus from the samples.

### #math-glyph-fallback

A command whose character the math font lacks is drawn as the font's missing glyph and reported in
`missing_glyphs`; the palette lists only the commands the font has. The generator writes such
commands apart from `COMMANDS`, so `parse` reads one as its character, where a single table limited
to the font's characters would read it as an unknown command, drawn red: found at the review of
#math-layout, which reconciles this design with #latex-command-set's. Nearest rival, STIX Two Math as a
fallback: 819 KB in every build for 44 rare symbols, #arg-69, weighed by the owner in #arg-73.

### #builtin-text-family

Open Sans, the four static styles of its `noto-set` build, is the one built-in text family, under
`FontRef::Builtin("sans")`; Noto Sans leaves the binary. The font stack of a text ends with the math
font, so a character neither the text's font nor Open Sans has is drawn from Libertinus Math. Nearest
rival, two built-in families: about 2 MB more in every build when the owner named one, #arg-77.

### #math-editing-model

The caret moves in a formula by the mapping table "A key in a formula, to what it does". `^` and `_`
make scripts, `&` sets an alignment point, Ctrl+Return starts a new row, `\` opens a command name,
Space leaves a cell; Backspace selects an atom with cells before deleting it; at a math item's edge
in its text, Backspace or Delete selects the whole item by a first press and deletes it by a second,
as for a span with items; a math item left empty when the caret leaves it is removed, and an
equation area keeps its formula, empty or not. The window edits an equation's formula on the layout
the renderer draws, which `area_formula` gives it as `area_text` gives a text's. Nearest rival, a source
field with a preview: fails C1, #arg-24.

### #latex-command-set

`COMMANDS`, generated from W3C `unicode.xml` by the generator of `path@xtask@src/main.rs`, maps each
command name to its character and class, limited to the characters Libertinus Math has, the W3C
notice beside it. A name means what it means in LaTeX's math mode: the generator takes every name
a `<character>` gives in its columns `<AMS>`, `<mathlatex>` without a set, `<IEEE>`, `<mathlatex
set="unicode-math">` and `<latex>`, and a name two characters claim goes to the first column of
that order, so `\epsilon` is ϵ and `\varepsilon` ε, as in TeX; LaTeX's aliases of
`path@elsewhere@base/fontdef.dtx` that no column gives, as `\to`, `\le` and `\neq`, and its
`\emptyset`, are a list of the generator's own, D6; unicode-math's names of an upright letter,
`\mupalpha` and the others, are left out, since `\mathrm` sets one upright, D6; the structures of #math-storage-form's subset are commands too. A command typed
after `\` and closed by a character that is not a letter becomes its atom. Nearest rival, unicode-math's
table: LPPL, #arg-61.

### #math-symbol-palette

While a formula is edited, the Selection tab lists the commands of `COMMANDS` by category; a click
inserts at the caret, by commands. Nearest rival, a palette in a menu: more clicks, #arg-25.

### #empty-cell-placeholder

An empty cell is laid out as a placeholder box, half an em wide, in every display; the overlay draws
its outline while editing. An empty group, and an empty nucleus of scripts, take no width, as TeX
gives them none, so `{}^{14}\mathrm{C}` and `a{}={}b` are set as in LaTeX, D8. Nearest rival, a zero-width cell: the caret has nowhere to stand, #arg-26.

### #math-in-protocol-grammar

`$…$` is a word of the lexer, its content raw up to the first `$` preceded by an even number of
`\`, none included, so `\$` is a dollar of the formula and `\\$` a row break that ends it; a math item of a tree
is that word, and a reference `{ref n<id>}`. Nearest rival, a quoted string: every backslash doubled,
#arg-27.

### #math-edit-commands

Each change of a formula in the window sends `math` or `equation` with the whole formula. Nearest
rival, commands by cell path: a bigger grammar for no gain, #arg-28.

### #math-keys

LyX's math keys, copied with their oddities, by the mapping table "A LyX math key, to the action it
runs", acting while a text or a formula is edited, but for Ctrl+Shift+M and Ctrl+Alt+N, which act
while an area is selected and not edited; the browser build disables a binding the browser keeps, and
every action a key runs has a control of the side panel or the node menu too, D3. Nearest rival, a corrected table: costs the owner's muscle memory, C8, #arg-30.

### #keymap-table

The math keys are entries of a built-in table from key sequences to actions; the other shortcuts stay
where they are until `issue@rootpoint-app@configurable-keys`. Nearest rival, every key moved now: the
same work earlier, with the key tests at risk, #arg-45.

### #equation-options

An equation holds `numbered`, `size`, `Fit` or `AtMost(size)`, and `colour`, unnamed; a new one is
numbered, `Fit`, in the Body style's colour or black, but by Ctrl+Shift+M, which makes it
unnumbered, as LyX's `math-display` does. Nearest rival, a text style named by the
equation: options with no meaning for it, C7.

### #equation-fit

`Fit` lays the formula out once and scales it by the largest k at which it and its number fit the
area less its padding; `AtMost(size)` lays it out at `size`, scaled down by the same k when that k is
below 1. The number is drawn at the Body style's size and does not scale. Nearest rival, shrink only:
the owner chose both, #arg-47.

## Mapping tables

### A LaTeX construct, to the atom it reads as

| construct | atom |
| --- | --- |
| a letter, a digit, a character of `COMMANDS`, `\name` of `COMMANDS` | symbol, with its class |
| `\frac{a}{b}` | fraction |
| `\sqrt{a}`, `\sqrt[n]{a}` | radical, with an index for the second |
| `x_a`, `x^b`, `x_a^b`, `x^b_a` | script of the atom before; a second superscript or subscript on one nucleus does not parse |
| `\left( … \right)`, with an open, a close, a bar or a slash of `COMMANDS`, `<` and `>` as ⟨ and ⟩, or `.` | delimited group |
| `\hat`, `\bar`, `\vec`, `\dot`, `\ddot`, `\tilde`, `\check`, `\breve`, `\acute`, `\grave`, `\overline`, `\underline` | accent |
| `\sum`, `\prod`, `\int`, `\oint` and the other operators of class op, with `_` and `^` | scripts whose nucleus is the operator, set as limits in display style but for the integrals |
| `x'`, `x''` | a prime, or primes, in the superscript of the atom before, as `x^{\prime}`, D7 |
| `\mathbf`, `\mathrm`, `\mathcal`, `\mathbb`, `\mathit` | font change |
| `\text{…}` | text run |
| `\log`, `\lg`, `\ln`, `\lim`, `\limsup`, `\liminf`, `\sin`, `\arcsin`, `\sinh`, `\cos`, `\arccos`, `\cosh`, `\tan`, `\arctan`, `\tanh`, `\cot`, `\coth`, `\sec`, `\csc`, `\max`, `\min`, `\sup`, `\inf`, `\arg`, `\ker`, `\dim`, `\hom`, `\det`, `\exp`, `\Pr`, `\gcd`, `\deg` | operator name, upright, of class op; with limits for `\lim`, `\limsup`, `\liminf`, `\max`, `\min`, `\sup`, `\inf`, `\det`, `\Pr` and `\gcd`, as `path@elsewhere@base/ltmath.dtx` defines the 32; D7 |
| `\begin{matrix}`, `pmatrix`, `bmatrix`, `cases` … `\end{…}`, cells at `&`, rows at `\\` | matrix |
| `\,`, `\:`, `\;`, `\quad`, `\qquad`, `\!` | space |
| `{…}` | group |
| `\\` and `&` at the top level | the formula's rows and their alignment |
| any other `\name` | unknown atom, keeping its name, taking no argument |

### A `unicode.xml` type, to the class of an atom

The `type` attribute of a `<character>` of `unicode.xml`, which the generator of `COMMANDS` reads:

| type | class |
| --- | --- |
| `relation` | rel |
| `binaryop` | bin |
| `large` | op |
| `opening` | open |
| `closing` | close |
| `punctuation` | punct |
| `alphabetic`, `normal`, `diacritic`, `other`, and a character with no type | ord |

Six ASCII characters take the class LaTeX's `path@elsewhere@base/fontdef.dtx` declares instead, by
D6: `.` ord, `-` and `*` bin, `:` rel, `!` and `?` close.

### A pair of classes, to the space between them

TeX's table of the spacing between atoms, The TeXbook, chapter 18: "thin" is 3/18 em, "medium" 4/18
em, "thick" 5/18 em; a space in parentheses is set in display and text styles only, not in script
styles. The row is the left atom's class, the column the right one's; inner, the class of a
fraction or a delimited group, is TeX's.

| left \ right | ord | op | bin | rel | open | close | punct | inner |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ord | 0 | thin | (medium) | (thick) | 0 | 0 | 0 | (thin) |
| op | thin | thin | — | (thick) | 0 | 0 | 0 | (thin) |
| bin | (medium) | (medium) | — | — | (medium) | — | — | (medium) |
| rel | (thick) | (thick) | — | 0 | (thick) | 0 | 0 | (thick) |
| open | 0 | 0 | — | 0 | 0 | 0 | 0 | 0 |
| close | 0 | thin | (medium) | (thick) | 0 | 0 | 0 | (thin) |
| punct | (thin) | (thin) | — | (thin) | (thin) | (thin) | (thin) | (thin) |
| inner | (thin) | thin | (medium) | (thick) | (thin) | 0 | (thin) | (thin) |

A "—" cannot occur: a bin first in its list, or after a bin, an op, a rel, an open, a punct or the
start of a delimited group, becomes an ord, and so does a bin last in its list, or before a rel, a
close, a punct or the end of a delimited group, as TeX's rules 5 and 6 of appendix G have it. The
table was checked at #math-layout's audit against `math_spacing` of `path@elsewhere@tex.web`, §764,
which it matches cell for cell, and the conversion against §728 and §729 of the same file; the next
table against `path@elsewhere@UnicodeData.txt`, every base code point and every hole it names.

### A character under a font change, to the code point drawn

| character | no font change | `\mathit` | `\mathrm` | `\mathbf` | `\mathcal` | `\mathbb` |
| --- | --- | --- | --- | --- | --- | --- |
| Latin letter | math italic, U+1D434 onwards, `h` U+210E | math italic | the letter itself | math bold, U+1D400 onwards | math script, U+1D49C onwards, the holes of Letterlike Symbols filled from U+212C and the others | double-struck, U+1D538 onwards, the holes filled from U+2102 and the others |
| lowercase Greek letter | math italic, U+1D6FC onwards | math italic | the letter itself | math bold, U+1D6C2 onwards | unchanged | unchanged |
| uppercase Greek letter | the letter itself, upright, as TeX sets it | math italic, U+1D6E2 onwards | the letter itself | math bold, U+1D6A8 onwards | unchanged | unchanged |
| digit | the digit itself | the digit itself | the digit itself | math bold, U+1D7CE onwards | unchanged | double-struck, U+1D7D8 onwards |
| any other character | itself | itself | itself | itself | itself | itself |

A code point the math font lacks is drawn as its missing glyph and reported, #math-glyph-fallback.

### A command, to the operation it runs

| command | operation |
| --- | --- |
| `equation <area> $f$ [options]` | `Op::SetContent` with an equation, or with the changed equation |
| `insert <n\|t> <offset> $f$` | the insertion of a math item, by #inline-math's audit |
| `insert <n\|t> <offset> {ref n<id>}` | the insertion of a reference item |
| `math <n\|t> <offset> $f$` | `Op::SetMath` |
| `retarget n<from> n<to>` | `Op::RetargetReferences` |
| `text <area> [… $f$ … {ref n<id>} …]` | `Op::SetText`, as today, with the new items |

### A key in a formula, to what it does

| key | in a formula |
| --- | --- |
| Right, Left | the next or the previous position; into an atom with cells at its first or last cell; out of an atom at its end; out of the formula at its end, one press, as at a span's edge |
| Up, Down | between numerator and denominator; between a script and its nucleus; between the rows of a matrix or of the formula |
| Tab | the next cell |
| a letter, a digit, an operator character | inserts its symbol |
| `^`, `_` | makes the atom before the nucleus of a script, the caret in the new script |
| `{` | opens a group |
| `&` | sets an alignment point in the row, or, in a matrix, moves to the next cell |
| Ctrl+Return | starts a new row of the formula, or of the matrix that holds the caret |
| `\` | opens a command name; letters extend it, with completion; a space closes it and is swallowed; any other character closes it and then acts |
| Space | leaves the current cell, then the formula |
| Backspace, Delete | deletes a symbol; selects an atom with cells, then deletes it; at the first or last position of the formula, does nothing |

At a math item's edge in its text, the keys are the text's: Backspace or Delete selects the item by
a first press and deletes it by a second, #math-editing-model.

### A LyX math key, to the action it runs

From `path@elsewhere@lib/bind/math.bind` and `path@elsewhere@lib/bind/cua.bind`, as section 1 of the
survey reads them; each acts while a text or a formula is edited, but the two that say otherwise:

| keys | action |
| --- | --- |
| Ctrl+M | insert an inline formula at the caret, the selection becoming its content |
| Ctrl+Shift+M | while an area is selected and not edited: make it an unnumbered equation |
| Ctrl+Alt+N | while an area is selected and not edited: make it a numbered equation |
| Alt+M F, S | `\frac`, `\sqrt` |
| Alt+M R | a radical with an index, `\sqrt[]{}`, which LyX's `\root` is |
| Alt+M X, E | subscript, superscript |
| Alt+M U, I, Y, P, 8, +, = | `\sum`, `\int`, `\oint`, `\partial`, `\infty`, `\pm`, `\neq` |
| Alt+M N | toggle the equation's number |
| Alt+M `"` H `\` `/` `&` `-` `.` Shift+V Shift+U V B | `\ddot`, `\hat`, `\grave`, `\acute`, `\tilde`, `\bar`, `\dot`, `\check`, `\breve`, `\vec`, `\overline` |
| Alt+M `_` | `\underline`, which LyX's `\underbar` is |
| Alt+M `'` | the prime ′, a symbol |
| Alt+M `(` `[` `{` `<` `>` `\|` | the delimiters of section 1 of the survey, as `\left…\right` |
| Alt+M G, then a letter, or Shift and a letter | the Greek letter of section 1 of the survey, with LyX's oddities |
| Alt+M C, then I, D, C or S | append, delete, copy or swap a column of the matrix that holds the caret, D2 |
| Alt+M W, then I, D, C or S | append, delete, copy or swap a row of the matrix that holds the caret, D2 |
| Alt+M Space | a thin space, `\,`, D2 |

Not bound, D2: LyX's sub-table T, which changes the hull type, where an equation has one kind;
Alt+M C and W with the keys of rules and of alignment, which the subset has no construct for;
Shift+N, a number per row, which #equation-rows leaves out; and Alt+M M, D, L and O, #arg-30.

## Losing alternatives

- **The LaTeX string as the model**, edited as source: lost to #math-storage-form on C1, #arg-9.
- **A RON tree of atoms in the file**: lost to #math-storage-form on C2, #arg-6.
- **Math as a kind of span**, and **inline math as an area**: lost to #math-text-item, #arg-10.
- **A display formula as a block of a text, as LyX has it**: lost to #equation-content-kind, #arg-13.
- **A number per row**: lost to #equation-rows, #arg-15; a later consequence.
- **Numbering by position on the slide**: lost to #equation-numbering, #arg-16.
- **Labels**: deferred by the owner, #equation-reference-target, `issue@rootpoint-model@equation-names`.
- **Allowing the removal of a referenced equation**: lost to #removed-equation-references, #arg-21.
- **A dialog that retargets and removes**: lost to #reference-retarget, #arg-40.
- **A fork of KenyC/ReX or latex-rust**, boxes with no node identity; **mathtex**, unsafe code from
  c2rust and a TeX format file; **typst's math layout**, private and a whole compilation; **RaTeX**,
  KaTeX's fonts and no MATH table: lost to #math-layout-engine, #arg-57.
- **read-fonts 0.45 for the MATH table**: lost to ttf-parser, #arg-59; `issue@rootpoint-math@read-fonts-math-table`.
- **Noto Sans Math, Latin Modern Math, New Computer Modern Math, STIX Two Math, Fira Math**: the
  owner chose Libertinus Math from the samples, #math-font.
- **STIX Two Math as a fallback font**: lost on #arg-69, weighed by the owner in #arg-73.
- **Libertinus Serif or two built-in families as text**: the owner chose Open Sans; two families lost
  on #arg-77.
- **A LaTeX source field with a preview**: lost to #math-editing-model on C1, #arg-24.
- **unicode-math's and KaTeX's tables**: lost to #latex-command-set's `unicode.xml`, #arg-61.
- **LyX's math autocorrection**: left out, #arg-62.
- **A zero-width empty cell**: lost to #empty-cell-placeholder, #arg-26.
- **Formulas in quoted strings**: lost to #math-in-protocol-grammar, #arg-27.
- **Commands by cell path**: lost to #math-edit-commands, #arg-28.
- **A corrected key table**: lost to #math-keys, #arg-30.
- **Every key moved into the table now**: lost to #keymap-table, #arg-45.
- **A text style named by an equation**: lost to #equation-options on C7, #arg-31.
- **Shrink only**, and **a fixed size that overflows**: lost to #equation-fit, #arg-47 and #arg-50.

## Readings

- **LyX's bind files and math model**: LyX `master` at `e7ffd8d`, read in the survey, section 1; the
  keys of the mapping table "A LyX math key, to the action it runs" and the numbering's scope come
  from it.
- **The OpenType MATH specification**: read in the survey, section 2; #math-layout reads the
  constants, the glyph info and the variants against it.
- **The TeXbook**, chapter 18 and appendix G: the styles, the spacing between classes and the
  conversion of a bin to an ord, by the mapping table "A pair of classes, to the space between
  them"; #math-layout's audit reads them.
- **The Unicode block Mathematical Alphanumeric Symbols** and its holes in Letterlike Symbols: the
  mapping table "A character under a font change, to the code point drawn"; #math-layout's audit
  reads it.
- **ron 0.12.2**: `escape_strings(false)` writes raw strings that read back equal, measured, #arg-39;
  read in its source by the reviews, `serialize_unescaped_or_raw_str` writes a raw string only for a
  value holding a `"` or a `\`, so a text with a line break and neither stays a quoted string, its
  break written as is, where #arg-39 said every newline made a raw string.

## Premortem

Assume the work shipped and failed. The causes, each with the thread it stresses and its verdict; the
owner's word of round 8, "you can keep all tripwires and AC.", rules on every label.

1. The project's layout looks wrong next to TeX's, in spacing, script placement or delimiter sizes.
   Stresses #math-layout-engine. An acceptance criterion, **AC1**, #ac-layout-looks-right.
2. Fitting is not exact. Stresses #equation-fit. **AC2**, #ac-fit-is-scaling.
3. The caret drifts off a formula's boxes. Stresses #math-text-item and #math-editing-model. **AC3**,
   #ac-caret-on-formula.
4. Scripts draw with full-size glyphs, because no shaper applies `ssty` to one run. Stresses
   #math-layout-engine. **AC4**, #ac-script-glyphs.
5. Rendering math is slow. Stresses C9. **AC5**, #ac-math-render-bound.
6. The chords do not reach the editor in the browser. Stresses #math-keys. **AC6**,
   #ac-chords-in-browser.
7. "→" typed in a text still draws as a missing glyph. Stresses #builtin-text-family's fallback.
   **AC7**, #ac-arrows-fall-back.
8. The owner sets most equations to `AtMost`. Stresses #equation-fit. A tripwire, **T1**: it fires
   when, while preparing one real talk, the owner sets `AtMost` on more than half of its equations;
   the response reopens the default toward `AtMost`; re-entry at the first real talk with math, and
   the standing-state review before every merge.
9. Agents write the formula literal badly. Stresses #math-in-protocol-grammar. **T2**: it fires when,
   in one session of the owner's, an agent has a command refused for its `$…$` formula three times or
   more; the response reopens the formula literal of `design@rootpoint-protocol@protocol-transport`;
   re-entry at the design audit of the next work on the protocol, and the standing-state review.
10. An Alt+M chord does not reach the native window. Stresses #math-keys. **T3**: it fires when the
    owner reports once that an Alt+M chord, in the native window on the owner's desktop, did not
    reach the editor or did something else; the response reopens #math-keys toward another prefix on
    that platform; re-entry at the first real talk prepared with math.
11. A talk needs a symbol of the missing set. Stresses #math-glyph-fallback. **T4**: it fires when a
    real slide needs a command of the missing set, and drawing it as missing is not acceptable to the
    owner; the response reopens #math-glyph-fallback toward STIX Two Math as a fallback; re-entry at
    the first real talk with math, and the standing-state review.

## Acceptance criteria

### The project's layout of the reference formulas is acceptable to the owner beside the browser's `##ac-layout-looks-right`

AC1. Guards #math-layout-engine. Judged at #math-layout. Observable: the reference formulas, drawn by the renderer in Libertinus Math into PNG files, and set by
the owner beside the browser's rendering of the same formulas in the same font, on a comparison
page the session publishes, are judged acceptable by the owner before the slice merges. Reworded at
#math-layout, where the owner judged on such a page, https://claude.ai/artifact/BUDsoxR7rZqDrWpdXjExmK,
in place of the font sample page of the discussion,
https://claude.ai/artifact/AWzepf7P1m4S7T27EiRrMd. Response: the layout is
fixed in the slice; a construct the owner judges wrong that the MATH data cannot place reopens
#math-layout-engine.

### A formula fitted by scaling equals the formula laid out at the scaled size `##ac-fit-is-scaling`

AC2. Guards #equation-fit. Judged at #math-layout. Observable: a test lays the reference formulas
out at size s and at size k·s, for several k, and finds every box of the second within 0.001 slide
units of the first's scaled by k. The tolerance is the session's, kept by the owner in round 8, and
the owner's to reset. Response: the layout is fixed; a construct that cannot scale
exactly reopens #equation-fit.

### Every caret position in and around a formula is drawn on its cell `##ac-caret-on-formula`

AC3. Guards #math-text-item and #math-editing-model. Judged at #math-editing. Observable: the caret's
property test of `path@rootpoint-app@tests/editing.rs`, extended to trees with formulas, puts every
caret position inside and around a formula on its drawn box, at the three window sizes the test uses,
within half a pixel, the tolerance of `design@rootpoint-app@edit-in-place`, the owner's to reset. Response: the map is fixed in the slice; a position that cannot be placed
reopens #math-text-item.

### A script is drawn with the font's `ssty` glyph `##ac-script-glyphs`

AC4. Guards #math-layout-engine. Judged at #math-layout. Observable: a test finds that the ′ of
`x^{\prime}` is drawn with the glyph Libertinus Math's `ssty` substitutes for it, not its base glyph,
and that the `a` of `x^{a}` keeps its base glyph, which that font's `ssty` does not cover. The
observable was reworded at #math-layout's audit: the font's `ssty` covers the six primes alone, so
the first wording, on the `a`, could not be observed.
Response: `ssty` applied by the math crate itself if no shaper applies it; none at all reopens
#math-layout-engine.

### A slide of equations and inline formulas renders within the bound `##ac-math-render-bound`

AC5. Guards #math-layout-engine and #math-text-item, against C9 and
`tripwire@rootpoint-render@render-within-bounds`. Judged at #inline-math. Observable: a 3840×2160
slide holding 4 equations and a text with 6 inline formulas renders within 200 ms, in a release build
on the owner's machine, by the ignored test `time_the_mixed_slide_at_4k` of
`path@rootpoint-render@tests/text.rs`, extended with that slide. The bound is the tripwire's, a
default the session proposed and the owner kept; the counts of the slide are the session's, kept by
the owner in round 8; both are the owner's to reset. Response: the layout is cached or made faster; a
render beyond the bound fires the tripwire.

### The math keys reach the editor in the browser `##ac-chords-in-browser`

AC6. Guards #math-keys. Judged at #math-editing. Observable: a test of
`path@rootpoint-app@tests/web.rs` finds that, while a text is edited, Alt+M then F inserts a fraction,
and that the browser's default of Ctrl+M is prevented. Response: a binding the browser keeps is
disabled in the browser build, on the owner's word of round 2; the chords failing natively too
reopens #math-keys.

### An arrow typed in a text is drawn from the math font `##ac-arrows-fall-back`

AC7. Guards #builtin-text-family. Judged at #builtin-open-sans. Observable: a test renders "→" in a
text in the built-in sans and finds it drawn with Libertinus Math's glyph, and not named in
`missing_glyphs`. Response: if Parley's font stack cannot fall back to the math font by character,
the fallback member is dropped and the owner told.

## Implementation sequence

1. The formula, its LaTeX form, and its layout from a MATH table, #math-layout, landed:
   the math crate, Libertinus Math, the table of commands, the layout and its drawing. Fails alone
   on: the subset's round trip, the MATH values read against the survey's, AC2 and AC4; AC1 is the
   owner's judgement.
2. Open Sans as the built-in text family, #builtin-open-sans, landed. Fails alone on:
   the goldens in Open Sans, and AC7.
3. The equation, a kind of content, #equation-kind, landed: the model, the file at version 6, the
   renderer, `equation`, the window's entries and options. Fails alone on: the fixture's round trip
   and the numbering test.
4. Math inside a text, and references to equations, #inline-math, landed: the items, the file at
   version 7, inline boxes, the tree's items, `math`, `retarget`, the refusals, the picker. Fails
   alone on: the refusals, and AC5.
5. [Editing a formula in the window, and LyX's math keys](math-editing.md), #math-editing. Fails
   alone on: AC3 and AC6.

## Order rationale

- #math-layout before #builtin-open-sans: the text's fallback names the math font, built in first.
- #builtin-open-sans before #equation-kind: the goldens change once for the font, before equations
  add images of their own; and the equation's number is drawn in the built-in sans.
- #equation-kind before #inline-math: a reference names an equation, which must exist.
- #inline-math before #math-editing: the window's caret in a formula needs the math item's box on
  the layout, which #inline-math gives `AreaText`.

## Defaults awaiting the owner

### D1, ruled: how the slices' work shares the branch of the licence

Bears on the branch of "How the work is done". The owner, round 8: "We will merge the milestone work
with the dual licensing work, as the same branch." The default: the branch math-milestone carries
the licence, the survey, the issues and this milestone's documents, and is reviewed and merged on its
own before #math-layout begins, as `skill@knowledge-architect-planning@plan-reviews` allows a plan
document; each slice then has a branch of its own, merged before the next begins. The other reading,
every slice built on math-milestone and one merge at the end, puts five slices behind one review,
where `skill@knowledge-architect-planning@cutting-steps-and-slices` asks one branch and one merge per
slice.

Ruled by the owner on the default, round 9: "D1 agreed, this is what i meant".

### D2, ruled: the keys of LyX's that the milestone binds beyond those shown in round 1

Bears on #math-keys. The owner, round 2: "math-keys: agreed on copying the keys from lyx exactly."
Round 1 left LyX's sub-tables T, C and W out "unless #equation-rows admits matrices", which it does;
the reviews found the plan had kept them out, and had added Shift+N and Alt+M Space to what is left
out with no ground shown to the owner. The default, in the mapping table "A LyX math key, to the
action it runs": Alt+M C and W bound for the columns and rows of a matrix, appended, deleted, copied
or swapped; Alt+M Space bound to a thin space; and left out, each for the ground the table gives, T,
the keys of C and W for rules and alignment, Shift+N, and M, D, L and O. The keys of a formula
`&` and Ctrl+Return, which LyX handles in `InsetMathNest::interpretChar` and its newline action, are
in the mapping table "A key in a formula, to what it does" as LyX has them, and need no ruling.

Ruled by the owner on the default, round 9: "agreed on the other defaults".

### D3, ruled: every action a key runs has a control too

Bears on #math-keys and #math-symbol-palette, against `goal@rootpoint-app@browser-window`, which asks
the browser build the same actions as the native window, while the owner allowed it, in round 2, to
disable a key the browser keeps. The default: every action of the mapping table "A LyX math key, to
the action it runs" has a control in the window, so a disabled key loses no action: "Insert formula"
in the Selection tab while a text is edited, for Ctrl+M; the node menu's "Equation", for
Ctrl+Shift+M and Ctrl+Alt+N; the equation's "numbered" option, for Alt+M N; the palette, for every
symbol and structure, the Greek letters, the accents, the delimiters, and the rows and columns of a
matrix. The palette and the menu entry were approved; "Insert formula" and the palette's matrix
controls are what the default adds.

Ruled by the owner on the default, round 9: "agreed on the other defaults".

### D4, ruled: a moved equation keeps its references

Bears on #equation-reference-target, closed by the owner in round 2 on ids, and on C6, "a reference
survives every edit but its equation's removal", which that closure claimed to meet. A finding of
the reviews, after the closure: `Op::Swap` exchanges the contents of two areas, keeping their ids,
`path@rootpoint-model@src/ops.rs`, and the window moves an area by a split, a swap and a removal,
`path@rootpoint-app@src/actions.rs`; so a referenced equation that is swapped or moved stands under
another id, and its references would name the area it left. The default: `Op::Swap` carries the
references of both areas with their contents, a member of #equation-reference-target's shape that
keeps the area id as what a reference names. The rival: an id of the equation's own, stored in its
content, which a swap carries by itself, at the cost of a fifth kind of id beside those of
`design@rootpoint-model@stable-ids`. #inline-math, which builds the references, is held until the
owner's word; the slices before it do not rest on it.

Ruled by the owner on the default, round 9: "agreed on the other defaults". #inline-math no longer
waits.

### D5, ruled: whether "as widely available as possible" is a goal of the project

Bears on `design@rootpoint@permissive-licence`, written by the reviews of this document since the
licence had landed with no head. The owner's words of round 4, "I plan to make the project open
source ultimately. probably soon", and "Iwant it as widely available as possible for everyone and
everywhere", state what the project is for, which a goal records,
`skill@knowledge-architect-goal-setting`. The default: no goal is written; the head attributes the
purpose to the owner. A goal would be drafted under that skill and ruled by the owner.

Ruled by the owner on the default, round 9: "agreed on the other defaults". No goal is written.

### D6, ruled: the names of the table of commands

Bears on #latex-command-set. The owner, round 3: "latex-command-set agreed on shape", the shape
being a table "generated from W3C `unicode.xml`". A finding of #math-layout's audit: the column the
slice's spec read, `<mathlatex set="unicode-math">` else `<latex>`, names α `\mupalpha` and ≤ `\leq`
alone, so `\alpha`, `\le` and `\dots` would be unknown, and the columns disagree on `\epsilon` and
`\phi`; and LaTeX defines `\to`, `\gets`, `\le`, `\ge`, `\land`, `\lor`, `\lnot`, `\owns`,
`\neq` and `\iff` as aliases or macros of other commands, which no column of `unicode.xml` gives.
The default, in #latex-command-set's decided design: every name of the five columns, a disputed name
going to the first column of the order AMS, `<mathlatex>` without a set, IEEE, unicode-math,
`<latex>`, which gives each Greek command its TeX character; and a list of the generator's own for
LaTeX's aliases, each naming the command or character it stands for in `path@elsewhere@base/fontdef.dtx`.
The same finding for the ASCII characters: `unicode.xml` types `.` a punctuation, so `3.14` would
take a thin space after its point, and `-` and `*` ordinary characters, where LaTeX's fontdef.dtx
declares `.` an ord, `-` and `*` bins drawn as − and ∗, `:` a rel, `!` and `?` closes. The default:
those six characters take LaTeX's classes, and `-` and `*` are drawn as LaTeX draws them, the other
characters keeping the mapping table "A `unicode.xml` type, to the class of an atom". The reviews of #math-layout found three more of the same kind, built
on the same default: `\emptyset`, which no column gives, is the generator's too, as ∅; `\langle`
and `\rangle`, `\lgroup` and `\rgroup`, `\lmoustache` and `\rmoustache`, which `unicode.xml`
types `other`, take the open and close classes fontdef.dtx declares; and unicode-math's 56 names of
an upright letter, as `\mupalpha`, are left out, since the layout sets a letter italic, as LaTeX
does, and `\mathrm` sets one upright. It widens the table's source beyond `unicode.xml` by these
lists, which is the scope change.

Ruled by the owner on the default, in the session that built #math-layout, by selecting
"D6 command names" among the defaults put to the owner.

### D7, ruled: the operator names of LaTeX

Bears on #math-storage-form and #latex-command-set. A finding of #math-layout's audit: the reference
formulas, then in #math-layout's "Fixtures", write `\cos` and `\sin`, which the closed subset read as
unknown atoms, drawn red. The default, in the mapping table "A LaTeX construct, to the atom it reads
as": the 32 operator names LaTeX defines in `path@elsewhere@base/ltmath.dtx`, each an atom of its own,
upright, of class op, with limits for the ten that LaTeX sets without `\nolimits`, `\liminf` and
`\limsup` with LaTeX's thin space; and, found at the review of #math-layout, `'` read as LaTeX
reads it, a prime in the superscript of the atom before, `f'` being `f^{\prime}`. It adds a kind of
atom and a construct to the subset the owner approved, which is the scope change.

Ruled by the owner on the default, in the session that built #math-layout, by selecting "D7
operator names, prime".

### D8, ruled: an empty group takes no width

Bears on #empty-cell-placeholder. The owner, round 2: "empty-cell-placeholder: agreed, I believe
this is needed and an unfinished formula is not supposed to exist in a real presenter run". A
finding of the review of #math-layout: LaTeX's own idioms `{}^{14}\mathrm{C}`, a prescript, and
`a{}={}b`, a relation spaced as between ords, hold an empty group in a finished formula, which a
placeholder would widen by half an em in presenter mode, outside the premise of the owner's words.
The default: an empty group, and an empty nucleus of scripts, take no width, as TeX gives them,
their box still marked empty for the editor's overlay; every other empty cell, of a fraction, a
radical, a script, a delimited group, an accent, a font change, a matrix or the formula's grid,
keeps the placeholder. It narrows the placeholder the owner approved, which is the scope change.

Ruled by the owner on the default, in the session that built #math-layout, by selecting "D8 empty
group".

## Harvest

Each slice's row is in its spec. This document's own row, at the commit that deletes it, after the
last slice:

- Report on AC1 to AC7 once more, each to a tripwire on the owner's word or deleted.
- Check that T1 to T4 are written, each in the tripwires home of the head it guards.
- Check that each later consequence below is in the issue entry, the plan document or the tripwire
  that it names, or in a `todo` issue opened for it then.
- Check that `issue@rootpoint-math@read-fonts-math-table` stands in the math crate's issue home,
  moved there by #math-layout's harvest.

## Later consequences

- **Naming equations**: `issue@rootpoint-model@equation-names`.
- **Configurable keys**: `issue@rootpoint-app@configurable-keys`.
- **One font parser**: `issue@rootpoint-math@read-fonts-math-table`.
- **A number per row, and a reference to a row**: no entry; opened as a `todo` issue at the harvest
  if the owner's talks need it by then.
- **A math font from a font asset**: no entry; #arg-23 named it for later "under a tripwire", which
  the premortem did not propose and the owner did not rule on, so a `todo` issue is opened at the
  harvest instead, which records the work without watching for its need.
- **The built-in sample teaching math**: the sample's slides teach the editor's features,
  `design@rootpoint-protocol@builtin-sample`; a slide of math is added by whichever slice the audit
  finds best placed, or a `todo` issue at the harvest.
- **Bullet lists**, needed by the owner before the first real talk: `issue@rootpoint@more-content-kinds`.
