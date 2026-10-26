# Spec: a built-in sample that teaches the editor, with pictures the editor draws of itself

## Status and audience

This is the spec of the redesign of the built-in sample:
- the sample's slides teach each feature of the editor to a new user, each slide's layout an
  example of the split tree;
- some slides show pictures of the editor, which the editor draws of itself, at runtime, when it
  opens the sample, with numbered callouts naming what its buttons do;
- the tests that read the sample's content move to a frozen copy of today's sample, so the sample
  can change.

It is written for the session that implements it, which did not witness the discussion. It leaves
in the commit that completes its harvest.

- Where it and a design home disagree on a point that "What is already decided" does not list as
  reversed or rewritten, the design home wins.
- Every name it uses is defined in it, in "Names" or "New names, in one place", or exists in the
  code.
- Where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says
  what waits. The defaults awaiting the owner are listed in their own section.
- It was assembled from the transcripts of two sessions. In the first, the discussion begins at
  the owner's message that opens "Hey, now that the project has most of the features I wanted";
  that session also held three other subjects, the compatibility of saved presentations, the
  publication on GitHub Pages and the README, all merged before this spec, and it was interrupted
  while writing this spec up to its Arguments. The second begins at the owner's message that opens
  "I had a design session for improving the sample presentation be interrupted", and wrote the
  rest from an extraction of the first, then repaired it after its reviews.

**The owner's messages on the sample**, verbatim, the parts on other subjects elided.

**Round 1**, the request:

> […] And lastly, I would like to improve the Sample presentation a little bit. Right now, it
> looks a bit bare, and also it is not making very good use of our split tree layout system for
> the showcase. I think the description of the features are a bit bare and not very easy to
> understand for a new user. It would also be nice if it could include self rendered (at runtime,
> not bundled assets) images of the UI with overlays that say what buttons do.

The same message gives, of the README, the reason the session read as the pictures' criterion:
"no images yet, the github pages should serve as preview, because I plan to rework the ui and do
not want to carry outdated images".

**Round 2**, the answer to the session's first round:

> Q2: I meant (a), a picture of the editor drawn by itself
>
> […]
>
> toolkit-snapshot is the shape I would go for. The ui tour is not fully ruled out IMO. I don't
> want the sample pictures to be fully exhaustive, I think the ui tour can come later for more
> completeness and interactive help.
> The pictures should not be full screen of the ui, but several each focussed one one menu, maybe
> one picture with the full ui to name each part of it before going into details.
>
> sample-slide-plan: this outline looks good.

**Round 3**, the answer to the convergence proposal, which showed the checkpoint table of the
threads ("contest any line") and the premortem:

> keep all AC and tripwires, proceed

**Round 4**, the answer to the session's status report, which put D1, D2 and the choice between
one spec and a milestone of three slices to the owner:

> Agreed on D1, D2, and on one spec + one branch.

**Round 5**, the answer to D3, which the review of this spec produced:

> D3: I'm not sure what you mean here. The question is about whether saving the sample
> presentation in a workspace or a pack should save the pictures with it ? I think it should, I
> don't want an additional mechanism to make sure reloading a sample from one saved to a workspace
> in a previous verison replaces the pictures with current ones. The sample can stay frozen, it is
> not meant to be saved anyway, at most it is used as a starting point were everything will be
> replaced, but in most cases it is more of a playground where users can try out the features in a
> live presentation. […]

The session answered that this was the question, and that the shape the owner chose also makes
`sample` headless hold the placeholders where the window holds the pictures, so that a script of
`sample` then `pack` or `render` replies differently in the two, against three heads that each gain
a stated exception; it asked the owner to say if that cost was not accepted.

**Round 6**, the answer:

> cost if fine.
> we won't implement the spec right now, once ready, merge it to main.

The spec was merged to the main branch on that word, its work to be built later; it waits there
until a session starts it.

## How the work is done

`skill@knowledge-architect-planning@working-a-slice`, its paragraph on the work of a spec.

## Names

- **the sample**: the presentation that the protocol's command `sample` opens, its RON text
  `path@rootpoint-protocol@sample/sample.ron` and its image files, included at build time by
  `path@rootpoint-protocol@src/sample.rs`.
- **the window**: the editor, `App` of `path@rootpoint-app@src/lib.rs`, natively or in the
  browser.
- **a frame**: one run of `App::frame` in an `egui::Ui`, as the window draws it and as the tests
  drive it with `Context::run_ui` and no window, in `path@rootpoint-app@tests/presenter.rs`.
- **a picture**: an image of part of the window, drawn by the window's own frame on the CPU, held
  by the sample as an image asset.
- **a callout**: a numbered mark with a short label drawn over a picture, naming what one widget
  does.
- **the recorded rectangles**: `App::view.buttons`, the rectangle of each widget a frame records by
  its label, which `App::button_rect` returns; today `App::button` of
  `path@rootpoint-app@src/panels.rs`, the side panel's tabs, the tab buttons of the asset tab and a
  few named buttons record theirs, and most of the toolbar's buttons, drawn by `ui.button` or
  `ui.add_enabled`, do not.
- **the fixture**: the frozen copy of today's sample that the tests open in its place.
- **the prototype**: a throwaway test, not committed, that drew one frame of the window at
  1600×900 into a PNG by filling egui's triangles on the CPU.

## What the work is

**Today.** `sample` replaces the session by the sample, in memory, its images in the session's
map, with empty undo stacks: `Command::Sample`, in `Session::run_inner` of
`path@rootpoint-protocol@src/session.rs`, calls `crate::sample::sample()`, then
`*self = Session::new()`, then sets the document and `Store::Memory(images)`. The sample has six
slides, s1 "rootpoint", s7 "Dividers and moving", s13, whose legend reads "The same image in fit,
left, and in fill, right.", s20 "Where it is saved", s24 "Styling text" and s33 "Equations", one
image asset, `splits.png`, 580 bytes, and `next_id` 38. Its texts name keys, as the Alt and arrow
keys in s1, Esc in s7, and Ctrl+B, Ctrl+I and Ctrl+Space in s24. The window opens it by the
toolbar's button "Sample", `App::open_sample` of `path@rootpoint-app@src/actions.rs`, and the
browser build opens it at start, in the wasm `main` of `path@rootpoint-app@src/main.rs`.

**The tests read its content.** The app's tests name it on 54 lines in 16 files of
`path@rootpoint-app@tests/`: `actions.rs` 9, `panel.rs` 8, `view.rs` 7, `web.rs` 7, `keys.rs` 5,
`control.rs` 4, `assets_tab.rs` 3, `socket_move.rs` 2, `workspace.rs` 2, and `autosave.rs`,
`editing.rs`, `edit_ops.rs`, `gestures.rs`, `hit.rs`, `presenter.rs`, `slides.rs` 1 each, counted by
`grep -c -e open_sample -e '"sample"' -e Command::Sample`; several of these lines are helpers that
many tests call. A 17th file, `native_workspace.rs`, reads `splits.png` from the sample's
directory. They name its ids: `editing.rs` edits the area n3 of s1; `gestures.rs` reads s1's tree,
`n4 stacked[n2 | n6 beside[n3 | n5]]`, and the spans of s24; `web.rs` compares s1 and s33, drawn in
a headless Firefox, with the native golden images `path@rootpoint-app@tests/data/sample-s1.png` and
`sample-s33.png`. The protocol's tests run `sample` in 5 files, `pack.rs`, `styles.rs`,
`commands.rs`, `view.rs` and `workspace.rs`, the last in scripts piped into `rootpoint-headless`.
`pack.rs` reads `path@rootpoint-protocol@sample/splits.png`, and its
`a_pack_adds_only_the_zip_s_headers` prints, for the sample, a pack's overhead over its assets and
its deflated RON, 318 bytes against a bound of 390, the figure that
`design@rootpoint-protocol@pack-container` quotes.

**The window draws its own frame and keeps no image of it.** egui 0.37 gives the frame's shapes,
`Context::tessellate` turns them into triangle meshes, each vertex with a position in points,
texture coordinates and a premultiplied sRGB colour, and the font atlas and every other texture
arrive as `ColorImage` in `FullOutput::textures_delta`, a list of deltas per texture. The prototype
filled those triangles on the CPU: a full 1600×900 frame took 30 ms at 1 pixel per point and 115
ms at 2, in release, on an AMD Ryzen 9 9950X, and the picture matched the window, the slide drawn
by the render crate included, by sight. The prototype is not committed.

**Outside the work:**
- an interactive tour over the real window, #live-ui-tour, which the owner left for later;
  `issue@rootpoint-app@interactive-tour` carries it;
- a change of the save format: the sample is a version-7 file, and nothing here changes the file's
  grammar, `design@rootpoint@saved-presentations-stay-loadable`;
- the browser build's speed at drawing the pictures on the owner's machine, which no test can
  measure: T2 watches it. Step 4's browser test prints a first figure on the machine that runs it.

## What is already decided

The design rests on, and does not argue again:
- `design@rootpoint@gui-only-in-app`: no crate but the app links egui, so the protocol cannot draw a
  picture.
- `design@rootpoint-protocol@session-files`: the session reads files only through the `Files` its
  caller passes to each command; a picture is read through it, by `Files::picture`.
- `design@rootpoint-protocol@no-new-breach`: no command creates a breach, so the sample headless
  holds a placeholder at each picture path, not a missing image.
- `design@rootpoint@readme-preview-link`: the owner keeps pictures out of the README because the
  interface will be reworked; the session read that reason as the criterion of the sample's
  pictures, `argument@sample-redesign@arg-4`.
- `design@rootpoint-app@editor-overlay` and `goal@rootpoint-app@faithful-slide-view`: no editor mark
  in a slide's image; a callout is part of a picture's pixels, not a mark over the slide.
- `design@rootpoint-model@asset-paths-only`: an image is an asset path whose bytes the store holds.
- `design@rootpoint-render@caller-supplies-assets`: the renderer draws an image the caller hands it.
- `rejected@rootpoint-render@toolkit-painter`: drawing slides with the toolkit's painter. Not
  brought back: the slides are still the render crate's; only the picture of the window comes from
  egui, and from its triangles filled on the CPU, not from a GPU offscreen path.
- `goal@rootpoint@minimal-content-set`: no content kind is added; a picture is an image.

Decisions the work rewrites. The texts that `cargo klarch show` lists as citing each, other than
this spec, are read in the step named, and each one the rewrite makes false is updated there.
- `design@rootpoint-protocol@builtin-sample`: rewritten at step 5 to hold the pictures a host draws,
  the slides' subjects and the tests' fixture. No text cites it. `path@rootpoint-protocol@README.md`
  describes the sample in prose, "a presentation of six slides", and is rewritten at step 5.
- `design@rootpoint-app@gui-toolkit`: its list of what the host needs gains frames run with no
  window, their shapes tessellated and their textures given, at step 5, as #toolkit-snapshot argues.
  Cited by 2 texts, in `register@rootpoint-app@design`,
  `register@rootpoint-render@rejected`. Among them `rejected@rootpoint-app@other-gui-hosts`
  stands, as #toolkit-snapshot says.
- `design@rootpoint-protocol@editing-modes`, `design@rootpoint-protocol@control-commands` and
  `design@rootpoint-app@actions-are-commands`: each states that a presentation in memory, or a
  script of commands, behaves the same headless and in the window, which `sample` no longer does
  for the pictures. Each gains at step 5 a stated exception for the sample's pictures, D3, on the
  owner's word in rounds 5 and 6. Their citing texts are read at step 5:
  - `design@rootpoint-protocol@editing-modes`: cited by 6 texts, in `register@rootpoint-app@design`,
    `register@rootpoint-model@design`, `register@rootpoint-protocol@design`,
    `path@rootpoint-protocol@tests/store.rs`.
  - `design@rootpoint-protocol@control-commands`: cited by 5 texts, in
    `register@rootpoint-app@design`, `register@rootpoint-protocol@rejected`,
    `path@rootpoint-protocol@src/session.rs`. The comment in `Session::run_inner` on refusing a
    command while a group is open says that a script holding the authority "gives the same replies
    in a window and headless"; it gains the same exception.
  - `design@rootpoint-app@actions-are-commands`: cited by 9 texts, in
    `path@rootpoint-app@CLAUDE.md`, `register@rootpoint-app@design`,
    `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/autosave.rs`,
    `path@rootpoint-app@tests/actions.rs`.
- `design@rootpoint-protocol@pack-container` quotes 318 bytes "for the sample": reworded at step 1
  to name the fixture, the figure unchanged. Cited by 8 texts, in
  `register@rootpoint-model@design`, `register@rootpoint-protocol@design`,
  `path@rootpoint-protocol@src/pack.rs`, `path@rootpoint-protocol@tests/pack.rs`,
  `register@rootpoint@design`.
- `design@rootpoint-app@workspace-autosave` times `autosave_gui_cost` on slides "of the sample,
  whose document holds no image's bytes": its test moves to the fixture at step 1, and the head
  names the fixture in the same step. Cited by 6 texts, in `register@rootpoint-app@design`,
  `issue@rootpoint-app@abandoned-composition-undo-step`,
  `path@rootpoint-app@src/autosave.rs`, `path@rootpoint-app@tests/autosave.rs`.
- `design@rootpoint-app@edit-in-place` and `design@rootpoint-app@editor-overlay` name tests that run
  over the sample's n3 and its styled text: rewritten to name the fixture at step 1, when the tests
  move. `design@rootpoint-app@edit-in-place`: cited by 10 texts, in `register@rootpoint-app@design`,
  `path@rootpoint-app@src/actions.rs`, `path@rootpoint-app@src/editing.rs`,
  `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@tests/editing.rs`,
  `path@rootpoint-app@tests/edit_ops.rs`, `register@rootpoint-render@design`.
  `design@rootpoint-app@editor-overlay`: cited by 11 texts, in `path@rootpoint-app@CLAUDE.md`,
  `register@rootpoint-app@design`, `issue@rootpoint-app@interactive-tour`,
  `register@rootpoint-app@rejected`, `register@rootpoint-app@tripwire`,
  `path@rootpoint-app@src/view.rs`, `path@rootpoint-app@tests/gestures.rs`,
  `path@rootpoint-app@tests/presenter.rs`.
- `path@rootpoint-app@README.md` names the sample's slides as a copy of the editor's keys: kept,
  since the new slides still name keys.

Standing entries the discussion's searches returned, and their outcome here:
- the `todo` issue `sample-teaches-no-math-editing` of the protocol: this spec schedules its work,
  slide 10, so the commit that adds this spec closes it, and its leads are in the paragraph under
  the table of slides of #sample-slide-plan. Its closing condition named the golden image of s33
  rewritten; the golden images stay on the fixture instead, #sample-test-fixture, and slide 10 does
  the issue's work.
- `issue@rootpoint@image-export`: an export of slides through `render_slide`. The pictures are of
  the window, drawn by egui's triangles, not slides, so the two share no path; bears on nothing.
- `rejected@rootpoint-app@other-gui-hosts`: weighs a toolkit's screenshots only if slides are drawn
  with its painter, which stays false; bears on the rewrite of `design@rootpoint-app@gui-toolkit`
  above, and stands.
- `tripwire@rootpoint-render@render-within-bounds`: its re-entry, work that adds a content kind or
  changes the renderer: this work does neither; bears on nothing.
- `tripwire@rootpoint-render@glyph-missing-everywhere`: the new texts and the callouts could use a
  character no font has: step 4's claim that every slide renders with no missing glyph watches the
  texts. The callouts are drawn by egui, whose own fonts the window already uses.
- `issue@rootpoint-app@configurable-keys`: the sample's texts and callouts name the fixed keys,
  which a later configuration of keys would leave stale; that work updates them.
- `tripwire@rootpoint-app@browser-pack-too-large`: the pictures add bytes to the sample held in the
  page; bears on nothing below its clause.
- `issue@rootpoint-model@linked-dividers`: a showcase of layouts cannot align rows across columns;
  the layouts of slide 6 stay guillotine layouts.
- The palette's math font, which a picture of formula editing needs: `App::frame` installs it in
  every frame, so a staged frame has it; bears on nothing.

## Criteria

### The sample opens headless by `sample`, with no missing image `##c-headless`

Binding, from `design@rootpoint-protocol@builtin-sample` and `goal@rootpoint@headless-core`. Met by
#sample-host-pictures: a placeholder stands for each picture when no host draws it.

### The protocol links no GUI toolkit `##c-no-toolkit`

Binding, from `design@rootpoint@gui-only-in-app`. Met by #toolkit-snapshot: the app draws.

### The browser opens and presents the same sample `##c-browser`

Binding, from `goal@rootpoint-app@browser-window`. Met by #toolkit-snapshot: the CPU filling runs
on every target.

### A slide shows no editor mark `##c-no-mark`

Binding, from `goal@rootpoint-app@faithful-slide-view`. Met: the pictures are image assets.

### No content kind exists for the sample alone `##c-no-new-kind`

Binding, from `goal@rootpoint@minimal-content-set`. Met by #toolkit-snapshot.

### The bundled images stay under 50 KB in all `##c-bundle-size`

Binding, from `design@rootpoint-protocol@builtin-sample`. Met: the pictures are drawn at runtime,
and their placeholders are small.

### The pictures show the editor as built, after any rework, with no step by hand `##c-pictures-current`

Weighed, from the session's reading of the owner's reason for keeping pictures out of the README,
`design@rootpoint@readme-preview-link`. Met by #toolkit-snapshot
and #picture-staging, and watched by AC4.

### A new user learns from each slide what a feature does, how to use it, and can try it there `##c-teaches`

Weighed, from the owner's request. The editing of a formula in the window serves
`goal@rootpoint@no-language-no-build`, an author with no LaTeX to know, so slide 10 teaches it. Met
by #sample-slide-plan.

## Threads

### The pictures of the editor are drawn by the editor itself `##ui-picture-source`

Proposed by the session in round 1, with three shapes: #toolkit-snapshot, #split-tree-mockup and
#live-ui-tour, and question Q2 on what "self rendered" meant. Approved in round 2: "Q2: I meant
(a), a picture of the editor drawn by itself". Shape: "Decided design". Harvest: a head of
`register@rootpoint-app@design`, by the primer's rules on heads.

### The window draws a staged frame and fills egui's triangles on the CPU `##toolkit-snapshot`

Proposed by the session in round 1, `argument@sample-redesign@arg-13` for it,
`argument@sample-redesign@arg-14` its costs, `argument@sample-redesign@arg-15` against the rejected
painter. Approved in round 2: "toolkit-snapshot is the shape I would go for." The prototype showed
it can be built, `argument@sample-redesign@arg-26`. Shape: "Decided design". Harvest: a head of
`register@rootpoint-app@design`, by the primer's rules on heads.

### A mock-up of the window built from slide areas `##split-tree-mockup`

Proposed by the session in round 1 as a rival, `argument@sample-redesign@arg-16` for it,
`argument@sample-redesign@arg-17` against. Ruled out in round 3 by the session, from the owner's
choice of #toolkit-snapshot and `argument@sample-redesign@arg-17`; it stood uncontested in the
checkpoint of round 3. Harvest: none, see "Losing alternatives".

### An interactive tour over the real window, later `##live-ui-tour`

Proposed by the session in round 1 as a rival, `argument@sample-redesign@arg-18`. Parked on the
owner's word in round 2: "The ui tour is not fully ruled out IMO. […] I think the ui tour can come
later for more completeness and interactive help." Its re-entry is the issue entry the commit that
adds this spec opens, `issue@rootpoint-app@interactive-tour`.

### Several pictures, each of one part of the window, and one of the whole window naming its parts `##picture-focus`

Proposed by the owner in round 2: "The pictures should not be full screen of the ui, but several
each focussed one one menu, maybe one picture with the full ui to name each part of it before going
into details", and "I don't want the sample pictures to be fully exhaustive". Approved, the slug
minted by the session in round 3. The picture of the whole window was a default put to the owner,
D2, since the owner wrote "maybe", and the owner agreed in round 4. Shape: "Decided design", the
table of pictures. Harvest: a head of `register@rootpoint-app@design`, by the primer's rules on
heads.

### The sample's slides follow the outline of fourteen slides `##sample-slide-plan`

Proposed by the session in round 1, `argument@sample-redesign@arg-21`. Approved in round 2:
"sample-slide-plan: this outline looks good." Shape: "Decided design", the slides. Harvest: the
rewritten `design@rootpoint-protocol@builtin-sample`.

### A host gives the session the pictures through its `Files`, which `sample` asks inside the command `##sample-host-pictures`

Proposed by the session in round 1 as a placeholder the app replaces, then in round 3 as a source
`sample` asks, `argument@sample-redesign@arg-24`, `argument@sample-redesign@arg-27`. Approved by the
session's reading of round 3's "proceed" against the checkpoint table, a default put to the owner,
D1, which the owner agreed in round 4. The review of this spec found that a source stored in the
session is a channel `design@rootpoint-protocol@session-files` does not admit; the session moved the
source into `Files`, which that head admits, keeping the protocol's source, the placeholder headless
and the asking inside the command that D1 approved. It found also that the session's map then
differs between the window and headless, which three heads state it does not: D3, which the owner
ruled in round 5 for the pictures in the session's map. Shape: "Decided design". Harvest:
`design@rootpoint-protocol@builtin-sample`.

### The pictures are a table in the app of stages, regions and anchored callouts `##picture-staging`

Proposed by the session in round 3, `argument@sample-redesign@arg-28`,
`argument@sample-redesign@arg-32`. Approved as #sample-host-pictures, D1. Shape: "Decided design".
Harvest: a head of `register@rootpoint-app@design`, by the primer's rules on heads.

### The pictures are drawn when the sample opens, at 2 pixels per point, cropped to their region `##picture-cost`

Proposed by the session in round 3, `argument@sample-redesign@arg-29`,
`argument@sample-redesign@arg-31`. Approved as #sample-host-pictures, D1. Shape: "Decided design".
Harvest: a head of `register@rootpoint-app@design`, by the primer's rules on heads, and
T2.

### The tests open a frozen copy of today's sample, not the sample `##sample-test-fixture`

Proposed by the session at the assembly of this spec, `argument@sample-redesign@arg-33`. Closed
`argued`: the approved outline changes every slide, and 54 lines in 16 files of the app's tests, a
17th file, and 5 files of the protocol's read today's slides, their ids, their texts and two golden
images; keeping today's ids in the new sample would bind the outline to the tests. Shape: "Decided
design". Harvest: the rewritten `design@rootpoint-protocol@builtin-sample`.

## Arguments

### The sample looks bare and does not use the split tree `##arg-1`

Round 1, the owner, on #sample-slide-plan: "it looks a bit bare, and also it is not making very
good use of our split tree layout system for the showcase".

### The descriptions are hard for a new user `##arg-2`

Round 1, the owner, on #sample-slide-plan: "the description of the features are a bit bare and not
very easy to understand for a new user".

### Pictures drawn at runtime, with overlays `##arg-3`

Round 1, the owner, on #ui-picture-source: "self rendered (at runtime, not bundled assets) images
of the UI with overlays that say what buttons do".

### The interface will be reworked `##arg-4`

Round 1, the owner, on the README, cited for #ui-picture-source: "because I plan to rework the ui
and do not want to carry outdated images".

### Repeated tasks become commands, and the goldens move `##arg-5`

Round 1, the grounding search of xtask, on #picture-cost: drawing the pictures again is a repeated
task, which `goal@xtask@repeated-tasks-are-automated` makes a command, not a step by hand; and "the
gate's tests compare the sample's slides 1 and 33 against native golden images".

### The protocol cannot draw the window `##arg-6`

Round 1, the grounding search of the root, on #sample-host-pictures and #toolkit-snapshot: "A sample
built headless, or by the protocol crate, has no editor UI to render"; the drawing "has to happen
in `rootpoint-app`"; a new content kind or overlay feature would strain
`goal@rootpoint@minimal-content-set`.

### A picture of the window is no slide `##arg-7`

Round 1, the grounding search of the render crate, on #toolkit-snapshot: "A picture of the editor UI
(toolbar, buttons) is not a slide"; "a runtime-produced image enters a slide through the caller's
`ImageSet`"; the toolkit's offscreen path is what `rejected@rootpoint-render@toolkit-painter`
distrusts.

### The sample opens headless, its images under 50 KB `##arg-8`

Round 1, the grounding search of the protocol, on #sample-host-pictures and #sample-slide-plan:
"a runtime rendering of the editor UI has to produce the sample headless as well"; images
"stay under 50 KB in all"; the pack of the sample measures "318 bytes"; an image generated at
runtime reaches the asset table "as assets of the sample's file, not as untracked files"; a
missing one is a cosmetic breach.

### An area holds one content `##arg-9`

Round 1, the grounding search of the model, on #split-tree-mockup: "an overlay over a UI image
cannot be a second content in the same area"; a picture is an asset of a store; layouts that are
no guillotine stay open in `issue@rootpoint-model@linked-dividers`.

### Standing entries the sample meets `##arg-10`

Round 1, the grounding search of the issues and tripwires: the sample could close
the protocol's issue `sample-teaches-no-math-editing`; callouts and texts could use a
character no font has, `tripwire@rootpoint-render@glyph-missing-everywhere`; texts naming keys go
stale under `issue@rootpoint-app@configurable-keys`.

### Pictures are slide content, never marks `##arg-11`

Round 1, the grounding search of the app, on #toolkit-snapshot: "Images of the editor UI with
overlays placed inside the sample's slides must be slide content, never editor marks."

### Reading (a) cannot go stale `##arg-12`

Round 1, the session, on Q2: "(a) […] It can never go stale after your interface rework."

### The snapshot is the current window `##arg-13`

Round 1, the session, for #toolkit-snapshot: "the picture is always the current UI. The same code
runs natively and in the browser, it is deterministic and testable with no GPU, and it adds no
content kind."

### What the snapshot costs `##arg-14`

Round 1, the session, on #toolkit-snapshot: "a CPU rasterizer of perhaps 150 lines (my estimate,
not measured)"; "opening the sample waits for each picture to be drawn"; "headless, the protocol
has no picture".

### Not the rejected painter `##arg-15`

Round 1, the session, on #toolkit-snapshot: "Here the slides are still drawn by rootpoint's
renderer; only the picture of the editor comes from egui, and it is CPU-drawn, not taken from the
toolkit's offscreen path that entry distrusts."

### The mock-up uses the split tree `##arg-16`

Round 1, the session, for #split-tree-mockup: "it uses the split tree itself and works headless."

### The mock-up floats and goes stale `##arg-17`

Round 1, the session, against #split-tree-mockup: "areas have no fill and no border, so it reads as
floating text, and adding those changes the format, which now owes a migration"; "it is drawn by
hand, so it goes stale after a rework like a bundled image would."

### The tour is exact and not in the presentation `##arg-18`

Round 1, the session, on #live-ui-tour: for, "exact and never stale"; against, "it is not part of
the presentation, so a pack or a slide cannot hold it."

### A placeholder keeps the headless sample whole `##arg-19`

Round 1, the session, on #sample-host-pictures: "This keeps the headless sample complete, without
the "missing image" fallback."

### The prototype's pre-commitment `##arg-20`

Round 1, the session, on #ui-picture-source: "if the text is legible at slide size and it takes
under 200 ms on your machine, #toolkit-snapshot holds." The owner gave no pre-commitment.

### Each slide teaches and is an example `##arg-21`

Round 1, the session, on #sample-slide-plan: "Each slide states what a feature is, how to do it
(gesture and key), and "try it here". Its layout is itself an example of a split."; "several tests
use the sample and must move with it."

### The owner's choice and the tour later `##arg-22`

Round 2, the owner, on #toolkit-snapshot and #live-ui-tour: "toolkit-snapshot is the shape I would
go for. The ui tour is not fully ruled out IMO. I don't want the sample pictures to be fully
exhaustive, I think the ui tour can come later for more completeness and interactive help."

### Pictures focused on one part each `##arg-23`

Round 2, the owner, on #picture-focus: "The pictures should not be full screen of the ui, but
several each focussed one one menu, maybe one picture with the full ui to name each part of it
before going into details."

### `sample` resets the session `##arg-24`

After round 2, the session, on #sample-host-pictures: "`Command::Sample` replaces the whole session
with `Session::new()`, so a host hook would have to survive that reset."

### The keys are copied in the sample `##arg-25`

The review of the Pages branch, on #sample-slide-plan: the sample's texts restate keys, Ctrl+B,
Ctrl+I and Ctrl+Space; the editor's README now names the sample among the copies of its keys.

### The prototype works `##arg-26`

Round 3, the session, on #toolkit-snapshot: "A full 1600×900 frame takes 30 ms at 1 pixel per point
and 115 ms at 2 […]. A picture cropped to one panel costs a fraction of that."; "The image matches
the window, including the slide drawn by rootpoint's renderer".

### A source inside the command keeps the stacks empty `##arg-27`

Round 3, the session, on #sample-host-pictures: "so the pictures are part of the opened sample with
empty undo stacks"; rival 1, replacing the placeholders by commands, "creates undo steps, so Ctrl+Z
would take the pictures back"; rival 2, no picture headless, "The sample would then name an image
it does not have."

### Callouts anchored on recorded rectangles `##arg-28`

Round 3, the session, on #picture-staging: "A callout is anchored on a widget's recorded rectangle,
with a number and a short label."; "The callouts are drawn as egui shapes on a top layer, so the
same rasterizer draws them."

### Drawn at the open, cropped `##arg-29`

Round 3, the session, on #picture-cost: "All pictures are drawn when the app opens the sample, at 2
pixels per point and cropped to their region."

### The mock-up's closure `##arg-30`

Round 3, the session, on #split-tree-mockup: "you chose #toolkit-snapshot; it would go stale and
needs a format change".

### The browser's speed is assumed `##arg-31`

Round 3, the session, on T2: "This is a tripwire because the wasm speed is not measured; I assume it
is 1.5 to 3 times slower."

### Most toolbar buttons record no rectangle `##arg-32`

After round 3, the session, on #picture-staging: "The toolbar draws most of its buttons with
`ui.button` directly, so their rectangles are not recorded. The panel rectangles come from
`Panel::…show`."

### The tests read today's sample `##arg-33`

At the assembly, the session, on #sample-test-fixture: the app's tests open the sample at 54 sites
in 16 files and name its ids and its golden images, and the approved outline changes every slide.

## New names, in one place

Each name is a default of this spec; the implementer may rename one, and the commit says so.

```
rootpoint-protocol, src/files.rs
  Files::picture                 trait method, default None: the PNG bytes a host draws for
                                 a picture path of the sample, or None
rootpoint-protocol, src/session.rs
  Session::open_sample           private method: what `Command::Sample` runs, taking the
                                 list of picture paths, so a unit test passes its own list
rootpoint-protocol, src/sample.rs
  PICTURES                       const: the asset paths of the sample that a host may draw
  sample/placeholder.png         file: the one image every picture path holds headless
rootpoint-protocol, tests/data/sample-fixture/
  presentation.ron, splits.png   files: today's sample.ron and splits.png, frozen
rootpoint-app, src/pictures.rs   module, new, `pub mod pictures` of src/lib.rs
  Stage                          struct: what the staged window shows before it is drawn
  Region                         enum: the part of the window a picture is cropped to
  Anchor                         enum: Label(&str), a recorded label; SelectedOutlineRow
  Callout                        struct: an anchor, a number from 1, a text of a few words
  PictureSpec                    struct: path, stage, region, callouts
  PICTURE_TABLE                  const: one PictureSpec per path of PICTURES
  staged_app                     fn: an App on files in memory that draws no picture
  draw_picture                   fn: a staged App and one PictureSpec to PNG bytes, or an
                                 error naming what failed
  rasterize                      fn: tessellated primitives and textures to RGBA pixels, or
                                 an error for a primitive it cannot draw
  STAGE_SIZE                     const: 1600×900 points, the staged window's size
  PICTURE_PPP                    const: 2.0, pixels per point
  CALLOUT_FILL                   const: the colour of a callout's circle and box
rootpoint-app, src/lib.rs
  App::draws_pictures            field: whether `App::run` hands the session a Files that
                                 draws pictures; set by App::new, cleared by staged_app
  App::view.regions              field: Vec<(String, Rect)>, each panel's rectangle by name
  App::region_rect               method: the rectangle the last frame gave a panel, by name
rootpoint-app, tests/
  pictures.rs                    test file: steps 3 and 4, AC4, AC5, AC7, and AC6 ignored
  common/mod.rs                  test module: fixture_files and open_fixture
existing, used as they are
  App::button, App::button_rect, App::side_panel_rect, App::in_memory, App::new, App::run,
  App::frame, App::layout, App::outline_rows, App::show_slide, App::show_tab,
  App::select_node, App::edit_at, App::select_all, App::select_asset, App::open_directory,
  Context::run_ui, Context::tessellate, FullOutput::textures_delta, MemoryFiles,
  Command::Sample, Command::OpenCopy
rootpoint-app, Cargo.toml
  image 0.25, feature png        moves from [dev-dependencies] to [dependencies], to encode a
                                 picture; the render crate already links the same version
```

## Decided design

### The tests open a frozen copy of today's sample (#sample-test-fixture)

**Shape.** Today's `sample.ron` and `splits.png` are copied, unchanged, to
`planned@rootpoint-protocol@tests/data/sample-fixture/`, as `presentation.ron` and `splits.png`, so
the directory is one that `open-copy` and a workspace read. Every test that runs `sample` to get
content opens the fixture instead, as the first mapping table says site by site:
- the app's tests use `planned@rootpoint-app@tests/common/mod.rs`. Its `fixture_files()` gives a
  `MemoryFiles` holding the fixture's two files under `sample-fixture/`, written through
  `Files::write` from `include_bytes!`, so the tests of the wasm target, which have no file system,
  read it too. Its `open_fixture(app)` runs `Command::OpenCopy` of `sample-fixture` through
  `App::run`. A test whose app is built on `NoFiles` builds it on `fixture_files()` instead. A test
  whose app is built on the native files runs `open-copy` of the fixture's absolute path,
  the app crate's `CARGO_MANIFEST_DIR` joined to the relative path of
  `planned@rootpoint-protocol@tests/data/sample-fixture/`. A test whose app is
  made by `App::in_memory` gives the files to `App::open_directory`;
- the protocol's tests write the fixture's files into the `MemoryFiles` of
  `path@rootpoint-protocol@tests/common/mod.rs` and run `open-copy`, or, for a script piped into
  `rootpoint-headless`, run `open-copy` of the fixture's absolute path;
- the two golden images, `sample-s1.png` and `sample-s33.png`, stay as they are: they are images of
  the fixture's s1 and s33, which do not change.

A test whose subject is `sample` itself keeps running it, and asserts of its content only what
`design@rootpoint-protocol@builtin-sample` states: that the command opens a presentation in memory,
with empty undo stacks, every slide rendered with no breach and no missing glyph. Its assertions on
today's slides, as `the_sample_opens_with_its_images_and_no_history` of
`path@rootpoint-protocol@tests/commands.rs` makes on the last slide's equation, move at step 1 to a
test of the fixture.

`a_pack_adds_only_the_zip_s_headers` of `path@rootpoint-protocol@tests/pack.rs` bounds the bytes a
pack adds over its assets and its deflated RON. On the sample today it prints an overhead of 318
bytes against a bound of 390, measured on this branch's base by
`cargo test -p rootpoint-protocol --test pack a_pack_adds_only_the_zip_s_headers -- --nocapture`. It
moves to the fixture, whose content is the same, so it prints the same figures, and
`design@rootpoint-protocol@pack-container`, which quotes 318 bytes "for the sample", is reworded at
step 1 to name the fixture.

**Argument.** The approved outline changes every slide. 54 lines in 16 test files of the app, a
17th that reads `splits.png`, and 5 test files of the protocol read today's slides by id, by text,
by tree and by golden image. Moving them to a frozen copy first lets the sample change with no test
edit after step 1, so a test that fails after step 1 is a fault of the new code, never of the
content.

**Nearest rival**: keep today's slides and ids inside the new sample. Defeated by the outline: it
has fourteen slides with new subjects, so keeping s1, s7, s13, s20, s24 and s33 with their trees
binds the outline to the tests, which the owner's approval of the outline did not include.

### A host gives the session the pictures through its `Files`, which `sample` asks inside the command (#sample-host-pictures)

**Shape.**
- The trait `Files` gains a method `picture(&mut self, path: &str) -> Option<Vec<u8>>`, whose
  default answers `None`. The session reads a picture through the `Files` its caller passes to the
  command, as it reads every file, `design@rootpoint-protocol@session-files`, and stores no source,
  so the five resets of `Session::run_inner` keep the session as they do today.
- `sample.rs` bundles one placeholder image,
  `planned@rootpoint-protocol@sample/placeholder.png`, and lists in `PICTURES` the asset paths of
  the sample that are pictures, all under `ui/`. `sample()` gives each path of `PICTURES` the
  placeholder's bytes, and looks in `IMAGES` for the other paths only: today it fails on any asset
  with no bundled bytes.
- `Command::Sample` runs `Session::open_sample`, which takes the list of picture paths: the command
  passes `PICTURES`, and a unit test of `session.rs` passes a list of its own. After the reset, it
  asks `files.picture(path)` for each path of the list, in order, and puts each `Some` answer in
  the store's map in place of the placeholder; a `None` keeps the placeholder. It runs inside the
  one command, so the opened sample has empty undo stacks and no unsaved change, as today.
- The headless binary's `Files` and the tests' keep the default, so their sample holds the
  placeholders. The app answers through a wrapper of its own `Files` that `App::run` passes to the
  session while `App::draws_pictures` holds: the wrapper hands every other method to the app's
  files, and answers `picture` by drawing. `App::new` sets the flag, so every `sample` that reaches
  `App::run` gets pictures: the toolbar's button, the browser's start, and a `sample` sent over the
  control socket. `staged_app` clears it, so drawing never recurses.
- A picture that fails to draw keeps its placeholder, and `App::run` sets the app's error to a line
  naming the path and the failure, which the window shows as it shows every error. The tests of
  AC4 and AC5 call `draw_picture` directly, so they see the failure itself.

**Argument.** The protocol links no GUI toolkit, `design@rootpoint@gui-only-in-app`, so the
drawing is the app's, and the protocol needs a way to receive it. `Files` is the one channel by
which a host gives the session bytes, `design@rootpoint-protocol@session-files`, and it is passed to
each command, so a source stored in the session, which every reset of `sample` would have to keep,
`argument@sample-redesign@arg-24`, is not needed. Asking inside the command keeps one command, one
opened state, `argument@sample-redesign@arg-27`.

**Nearest rivals.** A source stored in the session, the shape put to the owner in round 3: it adds
a channel `design@rootpoint-protocol@session-files` does not admit, and must survive five resets.
Replacing the placeholders afterwards through protocol commands creates undo steps, so Ctrl+Z
would take the pictures back. Leaving the picture missing headless makes the sample name an image
it does not have, a breach, which `design@rootpoint-protocol@no-new-breach` rules out for any
command.

**What the session's map holds depends on the host.** In the window, `sample` puts the drawn
pictures in the map; headless, the placeholders. So `sample` followed by `assets`, `render` or
`pack` replies differently in the two, where `design@rootpoint-protocol@editing-modes`,
`design@rootpoint-protocol@control-commands` and `design@rootpoint-app@actions-are-commands` state
that a presentation in memory behaves the same headless and in the window. Every shape that puts
pictures the window draws into the presentation has this difference, since the headless binary
cannot draw them. The owner accepted it, D3: in round 5 writing that the owner thinks a sample saved
to a workspace or a pack should keep its pictures: a sample saved and opened again shows the
pictures drawn when it was first opened, and no mechanism replaces them by current ones, since the
sample is a playground, not meant to be saved, and in round 6 accepting the difference itself.

**What a placeholder is.** One PNG, flat and small, that says in words that the editor draws this
picture when it opens the sample. It is made once and committed; its size counts in the 50 KB of
`design@rootpoint-protocol@builtin-sample`, with a bound of 4 KB, a default of this spec.

### The window draws a staged frame, and the app fills egui's triangles on the CPU (#toolkit-snapshot)

**Shape.** In `planned@rootpoint-app@src/pictures.rs`:
1. `staged_app()` builds an `App` by `App::new` on a `MemoryFiles`, with `App::draws_pictures`
   cleared. Its caller opens a presentation in it: the app's `Files` wrapper opens the sample, a
   test opens the fixture.
2. `draw_picture(app, row)` runs frames on a fresh `egui::Context` with no window, as
   `path@rootpoint-app@tests/presenter.rs` drives `App::frame`: `Context::run_ui` with a `RawInput`
   whose `screen_rect` is `STAGE_SIZE`, whose viewport's `native_pixels_per_point` is
   `PICTURE_PPP`, and whose `max_texture_side` is 8192, as that harness sets it. egui's default is
   2048, and `texture_size` of `path@rootpoint-app@src/lib.rs` caps the slide view's texture at
   it, which at 2 pixels per point would shrink a slide view wider than 1024 points.
3. It runs one frame, then applies the row's stage through the app's actions, as the second
   mapping table says, since entering a text needs the layout a frame records. It then runs frames
   until two in a row record the same rectangles, at most 4, a default of this spec: egui sizes its
   panels again on the second frame, as `path@rootpoint-app@CLAUDE.md` says. A last frame paints
   the callouts after `App::frame`, inside the same `run_ui`, on the layer order `Order::Tooltip`,
   above every layer the window uses, with the rectangles that frame records.
4. It keeps every texture the frames send, from `FullOutput::textures_delta`: for each texture id,
   it applies the id's list of deltas in order, a delta with no `pos` replacing the whole texture
   and one with a `pos` writing its rectangle; `free` removes a texture.
5. It tessellates the last frame's shapes, `Context::tessellate`, and `rasterize` fills them into
   RGBA pixels of `STAGE_SIZE × PICTURE_PPP`. `draw_picture` crops them to the row's region and
   encodes them as PNG.

`rasterize(primitives, textures, size, pixels_per_point)` is public, so a test hands it a primitive
of its own. It is the prototype's method: a triangle covers a pixel whose centre has three
non-negative barycentric weights, with no back-face culling, since egui's winding order varies;
each vertex position, in points, is scaled by the pixels per point; the triangle is clipped to the
primitive's clip rectangle; the colour is the weighted vertex colour times the texture, sampled
bilinearly; the pixel is blended over the buffer as premultiplied alpha, on the stored sRGB values,
with no conversion to linear light. The prototype blended so, and its picture matched the window by
sight; that it matches the GPU's blending to the pixel is an assumption, not measured. A primitive
that is no mesh, `Primitive::Callback`, or a mesh whose texture the frames did not send, is an
error `rasterize` returns, never a silent gap (AC5). A slide view's own texture is an egui texture
like any other, so the slide drawn by the render crate appears as in the window.

The prototype filled a frame of 1600×900 points in 30 ms at 1 pixel per point and 115 ms at 2, in
release, on an AMD Ryzen 9 9950X. Those times cover the tessellation and the filling only, not the
four frames it ran before them nor the creation of the context. The prototype is not committed, so
its figures cannot be taken again; step 3 measures the whole.

**Argument.** The picture is the window as built, by its own code, so it cannot go stale after a
rework of the interface, the session's reading of the owner's reason in
`argument@sample-redesign@arg-4`. The same code runs natively and in the browser, with no GPU, so a
test checks it, `argument@sample-redesign@arg-13`. It adds no content kind,
`goal@rootpoint@minimal-content-set`: a picture is an image asset.

**It adds a need of the window's host.** `design@rootpoint-app@gui-toolkit` lists what the host
needs under the project's renderer, "only panels, a list, a texture view, input forwarding, a
browser build, …". Drawing the window's picture needs one more: frames run with no window, their
shapes tessellated and their textures given. egui 0.37 meets it, as the prototype showed. The head
is rewritten at the harvest to name it, a member its argument covers: egui was chosen over the
hosts of `rejected@rootpoint-app@other-gui-hosts` on grounds this need does not change, and that
entry weighs a toolkit's screenshots only if slides were drawn with its painter, which stays false.

**Nearest rival**: #split-tree-mockup, the window mocked from slide areas. Defeated by the owner's
choice in round 2 and by `argument@sample-redesign@arg-17`: areas have no fill and no border, and a
mock-up drawn by hand goes stale. The rejected painter, `rejected@rootpoint-render@toolkit-painter`,
is not brought back: the slides stay the render crate's, and the picture is filled on the CPU, not
taken from the toolkit's offscreen path.

### The pictures are a table in the app of stages, regions and anchored callouts (#picture-staging)

**Shape.** `PICTURE_TABLE` holds one `PictureSpec` per path of the protocol's `PICTURES`, and a
test checks that the two lists name the same paths. A row gives:
- **its stage**: the slide shown, by its position in the presentation, from 1; the side panel's
  tab; and at most one of an area selected, an asset selected, a text edited in place with all its
  text selected, or a formula edited. The second mapping table gives the actions that reach each;
  the stage never sets the app's fields;
- **its region**: the whole window, or one panel by name: `toolbar`, `slides`, `side`, `view` or
  `status`. The frame records the rectangles of `toolbar`, `slides`, `view` and `status` in
  `App::view.regions`, from the response of each `Panel::…show` and of the `CentralPanel` in
  `App::editor`; `App::region_rect` returns them, and returns for `side` what
  `App::side_panel_rect` already returns;
- **its callouts**: each an anchor, a number from 1, and a text of at most four words, a default
  of this spec. An anchor is a label, found among the labels `App::view.buttons` records and the
  names `App::view.regions` records, or the selected outline row, the row of `App::outline_rows`
  whose `selected` holds.

`draw_picture` returns an error when an anchor is found in neither list, or when its label occurs
more than once in the two lists together (AC4). It scans the lists itself, since
`App::button_rect` returns the first match only. A callout never points at a place the window does
not have, nor at one of two.

A callout is drawn as a filled circle of radius 10 points, in `CALLOUT_FILL`, centred on the
anchor's top-left corner, holding its number in white at 13 points; its text is drawn in white at
13 points in a box of the same fill, at the circle's right, or at its left where the box would
leave the stage. The colour is an opaque saturated orange, a default of this spec, chosen to differ
from the colours of the window's own widgets. A slide that shows a picture explains each number
in a sentence.

The toolbar's buttons draw through `App::button` instead of `ui.button` and `ui.add_enabled`, so
they record their rectangles, `argument@sample-redesign@arg-32`; "Preview" and the state label
already do.

**Argument.** The callouts are what the owner asked for, overlays "that say what buttons do",
`argument@sample-redesign@arg-3`; anchoring them on recorded rectangles keeps them on the widget
after a rework, and makes a widget that no longer exists a test failure rather than a wrong
picture, `argument@sample-redesign@arg-28`.

**Nearest rival**: callouts placed at fixed coordinates in the table. Defeated by the reason the
session read into the owner's words on the README, `argument@sample-redesign@arg-4`: fixed
coordinates go stale after a rework exactly as a bundled image would.

### Several pictures, each of one part of the window, and one of the whole window naming its parts (#picture-focus)

**Shape.** The table's rows, as the work starts. The implementer may change a row, a callout or a
text, within the owner's words below.

| path | slide | stage | region | callouts |
| --- | --- | --- | --- | --- |
| `ui/editor.png` | 2 | slide 3, one area selected, tab Selection | the whole window | 1 `toolbar` "Files, preview, undo"; 2 `slides` "The slides"; 3 `view` "The shown slide"; 4 `side` "Selection, styles, assets"; 5 `status` "Messages"; 6 `state` "Where it is saved" |
| `ui/outline.png` | 4 | slide 3, a nested area selected, tab Selection | `side` | 1 `Selection` "This tab"; 2 the selected outline row "The selected area"; 3 `Styles` "Named styles"; 4 `Assets` "Images and fonts" |
| `ui/styles.png` | 8 | slide 8, tab Styles | `side` | 1 `New` "A new style"; 2 `Duplicate` "Copy this style"; 3 `New span style` "A new span style" |
| `ui/text.png` | 9 | slide 9, a text edited with all its text selected, tab Selection | `side` | 1 `Split here` "Cut the span here"; 2 `Insert formula` "A formula here" |
| `ui/formula.png` | 10 | slide 10, its equation edited, tab Selection | `side` | 1 `a/b` "A fraction"; 2 `√` "A square root"; 3 `matrix` "A matrix" |
| `ui/assets.png` | 11 | slide 11, the asset `splits.png` selected, tab Assets | `side` | 1 `Import…` "Add a file"; 2 `Remove unused` "Drop unused files"; 3 `Replace…` "Swap this file" |
| `ui/toolbar.png` | 12 | slide 12 | `toolbar` | 1 `Open…` "Open a directory"; 2 `Open pack…` "Open a pack"; 3 `Save` "Save"; 4 `Save as pack…` "Write a pack"; 5 `state` "Where it is" |

`Replace…` shows only with an asset selected, which the stage does. Every label above is one the
window records today or records once the toolbar draws through `App::button`, and each exists
natively and in the browser: "Save as pack…" is drawn by the browser build always, and natively
while the store has no root, as the sample's store in memory has none. The step that builds the
table checks each label against the frame.

**Argument.** The owner's words in round 2: the pictures are "several each focussed one one menu",
"maybe one picture with the full ui to name each part of it before going into details", and "I
don't want the sample pictures to be fully exhaustive". The picture of the whole window was a
default put to the owner, D2, which the owner agreed in round 4.

### The pictures are drawn when the sample opens, at 2 pixels per point, cropped (#picture-cost)

**Shape.** The source draws every picture when `sample` asks, at `PICTURE_PPP`, 2, cropped to its
region, before the command returns. Natively, all of them must take at most 500 ms in release
(AC6). The browser is watched by T2.

**Argument.** A picture is shown inside a slide area, which at full screen is a large part of a
1920×1080 slide, so it needs more pixels than one per point (AC7); drawing at the open keeps the
sample one command with one state, `argument@sample-redesign@arg-29`. The prototype's 115 ms for a
full frame at 2 pixels per point, 7 pictures of which 6 are crops, puts the total under the bound on
the machine that took it, an extrapolation the step's measurement replaces.

**Nearest rival**: drawing each picture lazily, the first time a slide shows it. It is T2's
response, held back because it makes the session's store change outside a command, which the
browser's speed has not yet been shown to need, `argument@sample-redesign@arg-31`.

### The sample's slides follow the outline of fourteen slides (#sample-slide-plan)

**Shape.** Each slide says what a feature is, how to use it, with its gesture and its key, and
invites the reader to try it on that slide; its layout is itself an example of a split,
`argument@sample-redesign@arg-21`. The keys a slide names are the ones
`path@rootpoint-app@README.md` lists when the step is written, and that README keeps naming the
sample among the copies of its keys. The outline, approved in round 2, with the layout and the
picture this spec gives each slide:

| # | subject | layout | picture |
| --- | --- | --- | --- |
| 1 | the title, and how to use the sample: anything tried here is undone by Ctrl+Z | a title over a short text | none |
| 2 | the editor at a glance | the picture beside the six numbered explanations | `ui/editor.png` |
| 3 | areas and splits | a tree of three levels, each area's text naming its own place, as "left of the top row" | none |
| 4 | selecting in the tree: the outline, and Alt with the arrows | a text beside the picture of the outline | `ui/outline.png` |
| 5 | dividers, swapping and moving areas | areas the reader drags, each text saying what to do with it | none |
| 6 | a gallery of layouts | four quadrants, each a common layout: title and body, two columns, image and caption, a grid | none |
| 7 | the content kinds side by side | a text, an image and an equation in three areas | `splits.png`, as today |
| 8 | styles and span styles | a text in a named style beside the picture of the Styles tab | `ui/styles.png` |
| 9 | editing text in place, Ctrl+B, Ctrl+I and Ctrl+Space | a text to edit beside the picture of the tab while editing | `ui/text.png` |
| 10 | formulas edited in place, an inline formula and a reference to an equation | a numbered equation, a text holding an inline formula and a reference, the picture | `ui/formula.png` |
| 11 | images, fit and fill, and the Assets tab | one image in fit and in fill, and the picture | `ui/assets.png` |
| 12 | where the presentation is saved | the picture of the toolbar over the explanations | `ui/toolbar.png` |
| 13 | previewing and presenting, F5 and Esc | a title over a short text | none |
| 14 | agents: the text protocol and the control socket | a title over a short text | none |

A picture of the side panel is tall and narrow, about 520 by 1,700 pixels, an estimate: the panel's
default width, 260 points, `default_size` in `App::editor` of `path@rootpoint-app@src/lib.rs`, and
about 850 points of height, the stage's 900 less the toolbar and the status line, each times 2, so
its area is a column at most 520 pixels wide at 1920×1080 (AC7). Slide 10 does the work of the
protocol's issue `sample-teaches-no-math-editing`, closed when this spec was added: its text names a
second click on a formula to edit it, Alt+M then f for a fraction, `\` to type a command, and the
palette of the Selection tab, and a text of the slide holds an inline formula and a reference to its
equation. The sample's texts use only characters the bundled fonts hold,
`tripwire@rootpoint-render@glyph-missing-everywhere`.

**Argument.** The owner's request, `argument@sample-redesign@arg-1` and
`argument@sample-redesign@arg-2`: the sample looks bare, does not use the split tree, and its
descriptions are hard for a new user. The owner approved the outline in round 2.

### The pictures of the editor are drawn by the editor itself (#ui-picture-source)

**Shape.** The shapes of #toolkit-snapshot, #sample-host-pictures, #picture-staging,
#picture-focus and #picture-cost above.

**Argument.** The owner's reading of Q2 in round 2, reading (a), "a picture of the editor drawn by
itself". The session offered (a) as the default on the owner's reason for keeping pictures out of
the README, `argument@sample-redesign@arg-4`: the interface will be reworked, and a bundled picture
would go stale. Reading (b), a picture drawn by the slide
renderer from slide content, is #split-tree-mockup's shape and loses with it.

## Mapping tables

**Each test site that reads the sample today, to what it does after step 1.** The sites are the 54
lines in 16 files of the app and the 5 files of the protocol that "What the work is" counts, and
`path@rootpoint-app@tests/native_workspace.rs`, which reads `splits.png` from the sample's
directory.

| the site | after step 1 |
| --- | --- |
| an app test that reads the sample's content: an id, a text, a tree, a style, an asset or a golden image | opens the fixture through the helpers of `planned@rootpoint-app@tests/common/mod.rs`, by the kind of files its app is built on, as #sample-test-fixture says |
| an app test whose subject is the action that opens the sample: `App::open_sample` or the toolbar's "Sample" | unchanged, asserting of the content only what `design@rootpoint-protocol@builtin-sample` states |
| a protocol test that reads the sample's content, in `pack.rs`, `styles.rs`, `commands.rs` or `view.rs` | `open-copy` of the fixture, its files written into the harness's `MemoryFiles` |
| a script piped into `rootpoint-headless` by `path@rootpoint-protocol@tests/workspace.rs` | `open-copy` of the fixture's absolute path in place of `sample` |
| a protocol test whose subject is `sample` | keeps `sample`; its assertions on today's slides move to a test of the fixture |
| `pack.rs` reading `path@rootpoint-protocol@sample/splits.png`, and `native_workspace.rs` reading it | read the fixture's `splits.png` |

**Each part of a stage, to the actions that reach it.** Each runs after the first frame, on the
staged app, in this order.

| part of the stage | actions |
| --- | --- |
| the slide at position k | `App::show_slide` of the id of the presentation's k-th slide |
| the side panel's tab | `App::show_tab` |
| an area selected | `App::select_node` of the area |
| a text edited, all its text selected | `App::select_node` of the area, then `App::edit_at` at the centre of the area's rectangle in `App::layout`, then `App::select_all` |
| a formula edited | `App::select_node` of the equation's area, then `App::edit_at` at the centre of its rectangle, which enters a formula on an equation area |
| an asset selected | `App::select_asset` of the asset's id |

**Each picture path, to its row.** `PICTURES` of the protocol and the paths of `PICTURE_TABLE` are
the same set, which a test checks; the table of #picture-focus gives each row.

## Losing alternatives

- **#split-tree-mockup**, the window mocked from slide areas, lost to #toolkit-snapshot: areas have
  no fill and no border, adding them changes the save format, and a mock-up drawn by hand goes
  stale after a rework, `argument@sample-redesign@arg-17`, `argument@sample-redesign@arg-30`.
- **#live-ui-tour**, callouts over the real window, is parked, not lost: the owner left it for
  later, for completeness and interactive help. It is no part of the presentation, so a pack or a
  slide cannot hold it, `argument@sample-redesign@arg-18`. Its issue entry is
  `issue@rootpoint-app@interactive-tour`.
- **A picture source stored in the session**, set by its host and kept across its resets, the
  shape put to the owner in round 3, lost to the same thread's source in `Files` at the review of
  this spec: `design@rootpoint-protocol@session-files` admits no channel of bytes but the `Files`
  passed to each command, and the stored source had to survive five resets.
- **Drawing the pictures into the window's view only**, the app giving its renderer the drawn bytes
  for the sample's picture paths while the session's map keeps the placeholders: the rival of D3,
  lost on the owner's word in round 5, since a saved sample would hold placeholders and need a
  mechanism to draw its pictures again.
  It keeps the session the same headless and in the window, and costs a pack saved from the window
  holding placeholders, and `render` replying a picture other than the window's.
- **Replacing the placeholders by protocol commands** lost to #sample-host-pictures: it makes undo
  steps, so Ctrl+Z would remove the pictures, `argument@sample-redesign@arg-27`.
- **No picture headless** lost to #sample-host-pictures: the sample would name an image it does
  not have, `argument@sample-redesign@arg-27`.
- **Reading (b) of Q2**, the picture drawn by the slide renderer from slide content, lost to
  reading (a) on the owner's word in round 2.
- **Keeping today's slides and ids** lost to #sample-test-fixture: the approved outline changes
  every slide.
- **Callouts at fixed coordinates** lost to #picture-staging: they go stale after a rework.
- **Drawing lazily, at first show** is T2's response, held back as #picture-cost says.

## Readings

The work reads no external specification the project implements. It relies on egui 0.37's
behaviour, read in its source by the session in round 1 and confirmed by the prototype:
`Context::tessellate` returns triangle meshes whose vertices carry a position, texture coordinates
and a premultiplied colour; a texture arrives as a `ColorImage` in `FullOutput::textures_delta`,
a list of deltas per texture id, each whole or at a `pos`; egui's winding order is not consistent,
as a comment on `epaint::Mesh` says. The harvest records these in the head of #toolkit-snapshot,
which earns its entry by entry test 3.

## Premortem

The labels AC4 to AC7 and T2 follow AC1 to AC3 and T1, which the same session put to the owner in
the same messages on another subject, the compatibility of saved presentations, already merged.
Run by the session in round 3, in the convergence proposal, as "assume it shipped and failed". The
owner's ruling in round 3: "keep all AC and tripwires, proceed". The responses of AC4 to AC7 below
are worded by the session, each reopening the thread the premortem names as stressed.

| cause | thread stressed | verdict |
| --- | --- | --- |
| after a rework of the interface, a callout points at a widget that no longer exists | #picture-staging | AC4, on the owner's word |
| the frame produces a primitive the rasterizer cannot draw, a paint callback or an unknown texture, and the picture silently lacks it | #toolkit-snapshot | AC5, on the owner's word |
| opening the sample becomes slow | #picture-cost | AC6, on the owner's word |
| the browser build takes longer than 2 s to open the sample on the owner's machine | #picture-cost | T2, on the owner's word: a tripwire, since the speed of the wasm build is not measured, and the session assumed it 1.5 to 3 times slower than native, `argument@sample-redesign@arg-31` |
| a picture is blurry when presented full screen | #toolkit-snapshot | AC7, on the owner's word |

## Acceptance criteria

### AC4: every callout's anchor is found in its frame `##ac-callout-anchors`

Guards #picture-staging. Judged at step 4, by a test of `planned@rootpoint-app@tests/pictures.rs`
that draws every row of `PICTURE_TABLE` and fails when `draw_picture` reports an anchor not
recorded in the frame. Fires when that test fails on the built table. A label recorded twice is
also an error of `draw_picture`, a guard of #picture-staging that step 3 tests, not part of this
criterion. Response, worded by the session: the row is
repaired if the anchor's label changed; if no recorded rectangle can anchor the callout, the
anchoring of #picture-staging is reopened.

### AC5: every primitive of every picture is drawn `##ac-every-primitive`

Guards #toolkit-snapshot. Judged at step 4, by a test of the same file that draws every row and
fails when `draw_picture` reports a primitive that is no mesh or a texture it was not sent. Fires
when that test fails. Response, worded by the session: #toolkit-snapshot is reopened for that
primitive: the rasterizer learns to draw it, or the stage avoids it, on the owner's word.

### AC6: all pictures are drawn in at most 500 ms natively, in release `##ac-pictures-time`

Guards #picture-cost. Judged at step 4, by an ignored test of the same file, run by hand in release
like the project's other timing tests, that times one `sample` of an app made by `App::new`, every
picture drawn. The bound, 500 ms, was proposed by the session in round 3 and kept by the owner.
Fires when the time exceeds it on the machine the step names. Response, worded by the session:
#picture-cost is reopened: lazy drawing, a lower density, or fewer pictures.

### AC7: each picture is at least as wide in pixels as its area at 1920×1080 `##ac-picture-width`

Guards #toolkit-snapshot and the density of #picture-cost. Judged at step 4, by a test of the same
file that, for each slide of the sample showing a picture, compares the picture's width in pixels
with its area's width in slide units, which `area_rects` of `path@rootpoint-model@src/geometry.rs`
gives and which are pixels at 1920×1080. Fires when one picture is narrower. Response, worded by
the session: the slide's layout is changed first; if no layout of the outline fits, #picture-cost's
density is reopened.

## Implementation sequence

### Step 1: the tests open the fixture

**Builds**: `planned@rootpoint-protocol@tests/data/sample-fixture/`, the helpers of
`planned@rootpoint-app@tests/common/mod.rs`, and every test site moved as the first mapping table
says; `design@rootpoint-protocol@pack-container`, `design@rootpoint-app@workspace-autosave`,
`design@rootpoint-app@edit-in-place` and `design@rootpoint-app@editor-overlay` reworded to name the
fixture where they name the sample's tests. No code of `src/` changes.

**Claims**:
- the fixture is byte for byte today's `sample.ron` and `splits.png`: `cmp` of each pair, in the
  commit message;
- every moved test passes unchanged in what it asserts: the gates. A wrong implementation, a helper
  that opens an empty presentation, fails the tests that read s1's tree, which the step shows once
  by that mutation;
- after the step, the only tests that run `sample` are those of the mapping table's rows that keep
  it: the grep of "What the work is", `grep -c -e open_sample -e '"sample"' -e Command::Sample`
  over each crate's `tests/`, run again, and each remaining site named in the commit message with
  its subject;
- `a_pack_adds_only_the_zip_s_headers` prints, for the fixture, the overhead of 318 bytes and the
  bound of 390.

**Fails alone on**: a moved test that fails, since nothing else changed.

### Step 2: the session asks its `Files` for the pictures

**Builds**: `Files::picture` with its default, `Session::open_sample`, the placeholder
`planned@rootpoint-protocol@sample/placeholder.png` in `IMAGES`, the change of `sample()` that gives
each path of `PICTURES` the placeholder, and `PICTURES` empty. No `Files` answers a picture and the
sample names none, so nothing a user sees changes.

**Claims**, each a unit test in `path@rootpoint-protocol@src/session.rs` that calls
`Session::open_sample` with a list of its own and a `Files` whose `picture` answers some paths:
- a path the files answer holds their bytes, and a path they do not holds the placeholder's;
- after it, the undo stack is empty and nothing is unsaved;
- with the default `Files`, every path of the list holds the placeholder. A wrong implementation
  that skips the asking fails the first claim, which the step shows once by that mutation.

**Fails alone on**: the protocol's tests, since the app answers no picture yet.

### Step 3: the app draws a picture

**Builds**: `planned@rootpoint-app@src/pictures.rs` with `staged_app`, `draw_picture`,
`rasterize`, the stages, the regions and the callouts; `App::view.regions` and
`App::region_rect`; the toolbar's buttons drawn through `App::button`; `App::draws_pictures` and
the `Files` wrapper of `App::run`, with the flag cleared in `App::new` until step 4; `image` as a
normal dependency. `PICTURE_TABLE` is empty, so nothing a user sees changes.

**Claims**, tests of `planned@rootpoint-app@tests/pictures.rs` on a staged app that opened the
fixture, with rows of their own:
- a picture of the whole window has `STAGE_SIZE × PICTURE_PPP` pixels, 3200 by 1800, and the pixel
  at the centre of the "Preview" button's rectangle times the density is not the colour of the
  pixel at the centre of the empty part of the toolbar: a test with the density forgotten, one
  pixel per point, fails it;
- a crop to `toolbar` has the size of `App::region_rect("toolbar")` times the density;
- a callout anchored on "Save" puts `CALLOUT_FILL` at the top-left corner of "Save"'s rectangle,
  times the density;
- a callout on a label the frame does not record is an error, and so is a label recorded twice;
- `rasterize` given a `Primitive::Callback` returns an error;
- `cargo x gates` builds the module for the wasm target.

**Measures**: an ignored test, run by hand in release, prints the time of one `draw_picture` of
the whole window, frames and context included; the commit message gives it with the machine,
beside the prototype's 115 ms. No bound: AC6 bounds the whole at step 4.

**Fails alone on**: a test of `pictures.rs`, since the sample is unchanged.

### Step 4: the new sample, its pictures, and the window drawing them

**Builds**: the new `sample.ron` of fourteen slides, `PICTURES` and `PICTURE_TABLE` as
#picture-focus gives them, and `App::draws_pictures` set by `App::new`. `splits.png` stays.

**Claims**:
- the sample opens headless with no breach and no missing image, every slide renders with
  `Report::missing_glyphs` empty, the report `tripwire@rootpoint-render@glyph-missing-everywhere`
  names, and the bundled images stay under 50 KB: the test of `sample` in
  `path@rootpoint-protocol@tests/commands.rs`, with the sum of the bundled bytes printed;
- `PICTURES` and the table's paths are the same set: a test;
- an app made by `App::new` that opens the sample holds, for each path of `PICTURES`, PNG bytes
  other than the placeholder's: a test of `planned@rootpoint-app@tests/pictures.rs`. A wrong
  implementation, the flag left cleared, fails it;
- AC4, AC5 and AC7: the tests named in "Acceptance criteria";
- AC6: the ignored test, run by hand in release, its time in the commit message with the machine;
- the browser draws the pictures: a new test of `path@rootpoint-app@tests/web.rs` opens the sample
  in an app made by `App::in_memory` and finds, for each path of `PICTURES`, bytes other than the
  placeholder's, which it reads with `include_bytes!`. It prints the time `sample` took, read from
  `js_sys::Date::now`, since `std::time::Instant` panics on the wasm target: a first figure for
  T2, which the commit message reports with the machine. It is not the owner's machine, so it fires
  nothing.

**Fails alone on**: the content, a slide's text or layout, or a row of the table, since the
mechanism passed steps 2 and 3.

### Step 5: the record

**Builds**: the rewritten `path@rootpoint-protocol@README.md` on the sample's slides, and every
text "What is already decided" lists as read at step 5, the three heads of D3 and the comment in
`Session::run_inner` among them; the report on AC4 to AC7; the harvest.

**Claims**: `cargo klarch check` passes, and no text of the record but a test of the fixture
describes six slides: `git grep -n "six slides"`, its hits in the commit message, each one a test
of the fixture.

**Fails alone on**: the checker, since the code is unchanged.

## Order rationale

- Step 1 before step 2: the tests stop reading the sample before any code can change it, so a
  later test failure is never a fixture's.
- Step 2 before step 3: the app's wrapper has a method to answer, proven in the protocol alone.
- Step 3 before step 4: the drawing is proven on the fixture before any content depends on it.
- Step 4 before step 5: the record describes what was built and measured.

## Defaults awaiting the owner

- **D1, ruled in round 4.** Bears on #sample-host-pictures, #picture-staging and #picture-cost: the
  session read the owner's "proceed" of round 3 as approving the three threads the checkpoint table
  showed as proposed. The owner agreed in round 4.
- **D2, ruled in round 4.** Bears on #picture-focus: the owner wrote "maybe" for a picture of the
  whole window naming its parts; the default included it, as slide 2. The owner agreed in round 4.
- **D3, ruled in round 5.** Bears on #sample-host-pictures. With the pictures in the session's
  map, `sample` fills the map differently in the window and headless, so `sample` then `assets`,
  `render` or `pack` replies differently in the two. `design@rootpoint-protocol@editing-modes`,
  `design@rootpoint-protocol@control-commands` and `design@rootpoint-app@actions-are-commands`
  state that a presentation in memory, or a script, behaves the same in both. No goal is defeated:
  `goal@rootpoint@headless-core` asks that every operation the window offers can be performed and
  its result checked in a test or a script that opens no window. The drawing of the pictures is
  performed and checked by the tests of `planned@rootpoint-app@tests/pictures.rs`, which open no
  window; what no headless script obtains is the drawn bytes, since the headless binary links no
  toolkit, `design@rootpoint@gui-only-in-app`. Two shapes meet every binding constraint:
  - **default, the pictures in the session's map**, as the spec is written: the three heads gain,
    at the harvest, a stated exception for the pictures of the sample, the one content a host
    draws. A pack saved from the window holds the pictures, and `render` draws them;
  - **the pictures in the window's view only**: the session keeps the placeholders everywhere, and
    the app hands its renderer the drawn bytes for the sample's picture paths. The three heads hold
    as they are; a pack saved from the window holds placeholders, and `render`, natively and over
    the control socket, replies a picture other than the window's.

  The owner chose the first, writing that the owner thinks saving the sample should keep its
  pictures, with no mechanism that replaces them when a saved sample is opened again; the sample is
  a playground, not meant to be saved. The answer addressed the saving; the difference between
  the window and headless follows from the same shape, and the owner accepted it in round 6, when
  the session put it as a cost.
- No acceptance criterion awaits the owner: AC4 to AC7 were kept in round 3.

## Harvest

At step 5, in the commits that land the work, under
`skill@knowledge-architect-decision-recording` and `skill@knowledge-architect-issue-tracking`,
except the rows whose "when" names an earlier step, which land in that step. The primer's rules on
design heads decide how the threads are cut into heads and whether each earns one; a thread's name
is its head's slug, as those rules say.

| what | where | when | from |
| --- | --- | --- | --- |
| the pictures of the window, drawn by the window and filled on the CPU, their staging, their callouts and their cost | new heads in `register@rootpoint-app@design`, among #ui-picture-source, #toolkit-snapshot, #picture-staging, #picture-focus and #picture-cost | step 5 | "Decided design" |
| the pictures through `Files`, the slides' subjects and the tests' fixture | `design@rootpoint-protocol@builtin-sample`, rewritten in place | step 5 | #sample-host-pictures, #sample-slide-plan, #sample-test-fixture |
| the host's new need | `design@rootpoint-app@gui-toolkit`, its list of needs | step 5 | #toolkit-snapshot |
| the session's map differing by host for the sample's pictures | `design@rootpoint-protocol@editing-modes`, `design@rootpoint-protocol@control-commands` and `design@rootpoint-app@actions-are-commands`, each gaining the exception on the owner's word in rounds 5 and 6, and the comment in `Session::run_inner` that restates the sameness | step 5 | #sample-host-pictures, D3 |
| the pictures in the window's view only, the rival of D3 | `register@rootpoint-protocol@rejected`, if a recording test of the decision-recording skill holds, cited by the three heads that gain the exception | step 5 | D3 |
| the window mocked from slide areas | `register@rootpoint-app@rejected`, if a recording test of the decision-recording skill holds, cited by the head of #toolkit-snapshot | step 5 | #split-tree-mockup |
| T2 | `register@rootpoint-app@tripwire`, guarding the head of #picture-cost | step 5 | "Premortem" |
| AC4 to AC7 | reported in the landing commits; each one that did not fire and recurs at later work is proposed to the owner as a tripwire | steps 4 and 5 | "Acceptance criteria" |
| the pack's figure named on the fixture | `design@rootpoint-protocol@pack-container` | step 1 | #sample-test-fixture |
| the premise of `autosave_gui_cost` and the tests named by `design@rootpoint-app@edit-in-place` and `design@rootpoint-app@editor-overlay` | those heads, restated on the fixture | step 1 | #sample-test-fixture |
| the six slides described | `path@rootpoint-protocol@README.md` | step 5 | #sample-slide-plan |
| the tests open the fixture, never the sample, unless `sample` is their subject | `path@rootpoint-app@CLAUDE.md` and `path@rootpoint-protocol@CLAUDE.md` | step 1 | #sample-test-fixture |
| a picture's callout anchors on a recorded label, so a label the window records occurs once | `path@rootpoint-app@CLAUDE.md`, its section on the places the last frame records | step 3 | #picture-staging |
| #live-ui-tour | `issue@rootpoint-app@interactive-tour`, opened by the commit that adds this spec; its sentence citing this spec is restated against the pictures as built in the commit that deletes this spec | done; restated at step 5 | "Losing alternatives" |

## Later consequences

- **The interactive tour**, `issue@rootpoint-app@interactive-tour`, adds callouts over the real
  window; it may reuse the anchors of #picture-staging, and replaces no picture.
- **The rework of the interface** the owner plans redraws every picture with no step by hand. A
  label that changes fails AC4's test, and its row of `PICTURE_TABLE` is repaired then.
- **A configuration of the keys**, `issue@rootpoint-app@configurable-keys`, leaves the keys the
  slides name stale; that work updates the sample's texts.
- **T2 firing** replaces drawing at the open by drawing at first show, under #picture-cost.
- **A change of the sample** no longer touches a test, except those whose subject is `sample`.
