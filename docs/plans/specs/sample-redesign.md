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
- It was assembled from the transcript of one session, through an extraction of it. The discussion
  begins at the owner's message that opens "Hey, now that the project has most of the features I
  wanted". That session also held three other subjects, the compatibility of saved presentations,
  the publication on GitHub Pages and the README, all merged before this spec; only the sample is
  planned here.

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
- **the recorded rectangles**: `App::view.buttons`, the rectangle of each widget a frame records
  by its label, which `App::button_rect` returns; today `App::button` of
  `path@rootpoint-app@src/panels.rs`, the side panel's tabs, the tab buttons of the asset tab and a
  few named buttons record theirs, and most of the toolbar's buttons, drawn by `ui.button`, do not.
- **the fixture**: the frozen copy of today's sample that the tests open in its place.
- **the prototype**: a throwaway test, not committed, that drew one frame of the window at
  1600×900 into a PNG by filling egui's triangles on the CPU.

## What the work is

**Today.** `sample` replaces the session by the sample, in memory, its images in the session's
map, with empty undo stacks: `Command::Sample` of `path@rootpoint-protocol@src/session.rs` calls
`crate::sample::sample()`, then `*self = Session::new()`, then sets the document and
`Store::Memory(images)`. The sample has six slides, s1 "rootpoint", s7 "Dividers and moving", s13
"The same image in fit, left, and in fill, right.", s20 "Where it is saved", s24 "Styling text" and
s33 "Equations", one image asset, `splits.png`, 580 bytes, and `next_id` 38. Its texts name keys:
Ctrl+B, Ctrl+I and Ctrl+Space in s24. The window opens it by the toolbar's button "Sample",
`App::open_sample` of `path@rootpoint-app@src/actions.rs`, and the browser build opens it at
start, `path@rootpoint-app@src/main.rs`.

**The tests read its content.** The app's tests open it at 54 sites in 16 files of
`path@rootpoint-app@tests/`: `actions.rs` 9, `panel.rs` 8, `view.rs` 7, `web.rs` 7, `keys.rs` 5,
`control.rs` 4, `assets_tab.rs` 3, `socket_move.rs` 2, `workspace.rs` 2, and `autosave.rs`,
`editing.rs`, `edit_ops.rs`, `gestures.rs`, `hit.rs`, `presenter.rs`, `slides.rs` 1 each, counted by
`grep -c` of `open_sample`, `"sample"` and `Command::Sample`. They name its ids: `editing.rs` edits
the area n3 of s1; `gestures.rs` reads s1's tree, `n4 stacked[n2 | n6 beside[n3 | n5]]`, and the
spans of s24; `web.rs` compares s1 and s33, drawn in a headless Firefox, with the native golden
images `path@rootpoint-app@tests/data/sample-s1.png` and `sample-s33.png`. The protocol's tests run
`sample` in `pack.rs`, `styles.rs` and `commands.rs`; `pack.rs` reads
`path@rootpoint-protocol@sample/splits.png` and measures the pack of the sample at 318 bytes against
390, a figure that `design@rootpoint-protocol@pack-container` quotes.

**The window draws its own frame and keeps no image of it.** egui 0.37 gives the frame's shapes,
`Context::tessellate` turns them into triangle meshes, each vertex with a position, texture
coordinates and a premultiplied colour, and the font atlas and every other texture arrive as
`ColorImage` in `FullOutput::textures_delta`, a list of deltas per texture. The prototype filled
those triangles on the CPU: a full 1600×900 frame took 30 ms at 1 pixel per point and 115 ms at 2,
in release, on an AMD Ryzen 9 9950X, and the picture matched the window, the slide drawn by the
render crate included, by sight. The prototype is not committed.

**Outside the work:**
- an interactive tour over the real window, #live-ui-tour, which the owner left for later; an
  issue entry carries it, opened by the commit that adds this spec;
- a change of the save format: the sample is a version-7 file, and nothing here changes the file's
  grammar, `design@rootpoint@saved-presentations-stay-loadable`;
- the browser build's speed at drawing the pictures, which no test measures: T2 watches it.

## What is already decided

The design rests on, and does not argue again:
- `design@rootpoint-protocol@builtin-sample`: every build carries the sample, which `sample` opens,
  headless too; its images stay under 50 KB in all. **Rewritten at the harvest**: it gains the
  pictures a host draws, its slides' subjects, and the tests' fixture.
- `design@rootpoint@gui-only-in-app`: no crate but the app links egui, so the protocol cannot draw a
  picture.
- `design@rootpoint-app@gui-toolkit`: the window draws no slide content itself; the pictures are
  image assets the render crate draws like any other, so the slides stay the renderer's.
- `design@rootpoint-app@editor-overlay` and `goal@rootpoint-app@faithful-slide-view`: no editor mark
  in a slide's image; a callout is part of a picture's pixels, not a mark over the slide.
- `design@rootpoint-app@actions-are-commands`: every action runs protocol commands through
  `App::run`; the pictures enter the session inside `sample`, not by an action of their own.
- `design@rootpoint-protocol@session-files`: the session opens no file.
- `design@rootpoint-model@asset-paths-only`: an image is an asset path whose bytes the store holds.
- `design@rootpoint-render@caller-supplies-assets`: the renderer draws an image the caller hands it.
- `rejected@rootpoint-render@toolkit-painter`: drawing slides with the toolkit's painter. Not
  brought back: the slides are still the render crate's; only the picture of the window comes from
  egui, and from its triangles filled on the CPU, not from a GPU offscreen path.
- `goal@rootpoint@minimal-content-set`: no content kind is added; a picture is an image.

Texts that cite what the work rewrites, each updated at the harvest:
- `design@rootpoint-protocol@builtin-sample` is cited by `path@rootpoint-protocol@README.md`, which
  describes "a presentation of six slides": rewritten to the new slides.
- `design@rootpoint-protocol@pack-container` quotes "318 bytes for the sample": the measurement
  moves to the fixture, the figure measured again.
- `design@rootpoint-app@workspace-autosave` times `autosave_gui_cost` on slides "of the sample,
  whose document holds no image's bytes": the test moves to the fixture, the premise restated.
- `design@rootpoint-app@edit-in-place` and `design@rootpoint-app@editor-overlay` name tests that run
  over the sample's n3 and s24: rewritten to the fixture.
- `path@rootpoint-app@README.md` names the sample's slides as a copy of the editor's keys: kept.

Standing entries the discussion's searches returned, and their outcome here:
- the `todo` issue `sample-teaches-no-math-editing` of the protocol: this spec schedules its work,
  slide 10, so the commit that adds this spec closes it, and its leads are in slide 10's row of
  #sample-slide-plan.
- `tripwire@rootpoint-render@render-within-bounds`: its re-entry, work that adds a content kind or
  changes the renderer: this work does neither; bears on nothing.
- `tripwire@rootpoint-render@glyph-missing-everywhere`: the new texts and the callouts could use a
  character no font has: the claim of step 6, no breach in the sample, watches it.
- `issue@rootpoint-app@configurable-keys`: the sample's texts and callouts name the fixed keys,
  which a later configuration of keys would leave stale; that work updates them.
- `tripwire@rootpoint-app@browser-pack-too-large`: the pictures add bytes to the sample held in the
  page; bears on nothing below its clause.
- `issue@rootpoint-model@linked-dividers`: a showcase of layouts cannot align rows across columns;
  the layouts of slide 6 stay guillotine layouts.
- `rejected@rootpoint-render@toolkit-painter`: not brought back, as above.

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

Weighed, from the owner's reason for keeping pictures out of the README. Met by #toolkit-snapshot
and #picture-staging, and watched by AC4.

### A new user learns from each slide what a feature does, how to use it, and can try it there `##c-teaches`

Weighed, from the owner's request. Met by #sample-slide-plan.

## Threads

### The pictures of the editor are drawn by the editor itself `##ui-picture-source`

Proposed by the session in round 1, with three shapes: #toolkit-snapshot, #split-tree-mockup and
#live-ui-tour, and question Q2 on what "self rendered" meant. Approved in round 2: "Q2: I meant
(a), a picture of the editor drawn by itself". Shape: "Decided design". Harvest: a head of
`register@rootpoint-app@design`, `sample-pictures` or the thread's own slug.

### The window draws a staged frame and fills egui's triangles on the CPU `##toolkit-snapshot`

Proposed by the session in round 1, `argument@sample-redesign@arg-13` for it,
`argument@sample-redesign@arg-14` its costs, `argument@sample-redesign@arg-15` against the rejected
painter. Approved in round 2: "toolkit-snapshot is the shape I would go for." The prototype showed
it can be built, `argument@sample-redesign@arg-26`. Shape: "Decided design". Harvest: a head of
`register@rootpoint-app@design`, `sample-pictures` or the thread's own slug.

### A mock-up of the window built from slide areas `##split-tree-mockup`

Proposed by the session in round 1 as a rival, `argument@sample-redesign@arg-16` for it,
`argument@sample-redesign@arg-17` against. Ruled out in round 2 by the session, from the owner's
choice of #toolkit-snapshot and `argument@sample-redesign@arg-17`; it stood uncontested in the
checkpoint of round 3. Harvest: none, see "Losing alternatives".

### An interactive tour over the real window, later `##live-ui-tour`

Proposed by the session in round 1 as a rival, `argument@sample-redesign@arg-18`. Parked on the
owner's word in round 2: "The ui tour is not fully ruled out IMO. […] I think the ui tour can come
later for more completeness and interactive help." Its re-entry is the issue entry the commit that
adds this spec opens, `issue@rootpoint-app@interactive-tour`.

### One picture per part of the window, and one of the whole window naming its parts `##picture-focus`

Proposed by the owner in round 2: "The pictures should not be full screen of the ui, but several
each focussed one one menu, maybe one picture with the full ui to name each part of it before going
into details", and "I don't want the sample pictures to be fully exhaustive". Approved, the slug
minted by the session in round 3. The picture of the whole window was a default put to the owner,
D2, since the owner wrote "maybe", and the owner agreed in round 4. Shape: "Decided design", the
table of pictures. Harvest: a head of `register@rootpoint-app@design`, `sample-pictures` or the
thread's own slug.

### The sample's slides follow the outline of fourteen slides `##sample-slide-plan`

Proposed by the session in round 1, `argument@sample-redesign@arg-21`. Approved in round 2:
"sample-slide-plan: this outline looks good." Shape: "Decided design", the slides. Harvest: the
rewritten `design@rootpoint-protocol@builtin-sample`.

### A host gives the session a source of pictures, which `sample` asks inside the command `##sample-host-pictures`

Proposed by the session in round 1 as a placeholder the app replaces, then in round 3 as a source
`sample` asks, `argument@sample-redesign@arg-24`, `argument@sample-redesign@arg-27`. Approved by the
session's reading of round 3's "proceed" against the checkpoint table, a default put to the owner,
D1, which the owner agreed in round 4. Shape: "Decided design". Harvest:
`design@rootpoint-protocol@builtin-sample`.

### The pictures are a table in the app of stages, regions and anchored callouts `##picture-staging`

Proposed by the session in round 3, `argument@sample-redesign@arg-28`,
`argument@sample-redesign@arg-32`. Approved as #sample-host-pictures, D1. Shape: "Decided design".
Harvest: a head of `register@rootpoint-app@design`, `sample-pictures` or the thread's own slug.

### The pictures are drawn when the sample opens, at 2 pixels per point, cropped to their region `##picture-cost`

Proposed by the session in round 3, `argument@sample-redesign@arg-29`,
`argument@sample-redesign@arg-31`. Approved as #sample-host-pictures, D1. Shape: "Decided design".
Harvest: a head of `register@rootpoint-app@design`, `sample-pictures` or the thread's own slug, and
T2.

### The tests open a frozen copy of today's sample, not the sample `##sample-test-fixture`

Proposed by the session at the assembly of this spec, `argument@sample-redesign@arg-33`. Closed
`argued`: the approved outline changes every slide, and 54 sites of the app's tests and three files
of the protocol's read today's slides, their ids, their texts and two golden images; keeping today's
ids in the new sample would bind the outline to the tests. Shape: "Decided design". Harvest: the
rewritten `design@rootpoint-protocol@builtin-sample`.

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
rootpoint-protocol, src/session.rs
  PictureSource                  type: Box<dyn FnMut(&str) -> Option<Vec<u8>>>, an asset path
                                 to the PNG bytes of its picture, or None
  Session::pictures              field: Option<PictureSource>, kept by every reset of the session
  Session::set_picture_source    method: sets or clears the field
rootpoint-protocol, src/sample.rs
  PICTURES                       const: the asset paths of the sample that a host may draw
  sample/placeholder.png         file: the one image every picture path holds headless
rootpoint-protocol, tests/data/sample-fixture/
  presentation.ron, splits.png   files: today's sample.ron and splits.png, frozen
rootpoint-app, src/pictures.rs   module, new
  Stage                          enum: what the staged window shows before it is drawn
  Region                         enum: the part of the window a picture is cropped to
  Callout                        struct: a recorded label to anchor on, a number, a short text
  PictureSpec                    struct: path, stage, region, callouts
  PICTURE_TABLE                  const: one PictureSpec per path of PICTURES
  draw_picture                   fn: one PictureSpec to PNG bytes, or an error naming what failed
  rasterize                      fn: tessellated primitives and textures to RGBA pixels
  STAGE_SIZE                     const: 1600×900 points, the staged window's size
  PICTURE_PPP                    const: 2.0, pixels per point
rootpoint-app, src/lib.rs
  App::view.regions              field: Vec<(String, Rect)>, each panel's rectangle by name
  App::region_rect               method: the rectangle the last frame gave a panel, by name
rootpoint-app, tests/
  pictures.rs                    test file: AC4, AC5, AC7, and AC6 as an ignored test
  common/mod.rs                  test module: fixture_app, which opens the fixture
existing, used as they are
  App::button, App::button_rect, App::in_memory, App::new, App::open_directory, App::run,
  App::frame, Context::run_ui, Context::tessellate, FullOutput::textures_delta, MemoryFiles,
  Command::Sample, Command::OpenCopy
rootpoint-app, Cargo.toml
  image 0.25, feature png        moves from [dev-dependencies] to [dependencies], to encode a
                                 picture; the render crate already links the same version
```

## Decided design

### The tests open a frozen copy of today's sample (#sample-test-fixture)

**Shape.** Today's `sample.ron` and `splits.png` are copied, unchanged, to
`planned@rootpoint-protocol@tests/data/sample-fixture/`, as `presentation.ron` and `splits.png`, so
the directory is one a workspace or `open-copy` reads. Every test that runs `sample` to get
content opens the fixture instead:
- the protocol's tests, `pack.rs`, `styles.rs` and `commands.rs`, run `open-copy` of the fixture's
  directory, with the files of their own test harness;
- the app's tests call one helper of `planned@rootpoint-app@tests/common/mod.rs`, which gives the
  fixture's files to the app, through `App::open_directory` on an app made by `App::in_memory`, or,
  for a test whose app is built on other files, through `App::run` of `Command::OpenCopy`. The
  helper reads the files with `include_bytes!`, so the tests of the wasm target, which have no file
  system, open it too;
- the two golden images, `sample-s1.png` and `sample-s33.png`, stay as they are: they are images of
  the fixture's s1 and s33, which do not change.

A test whose subject is `sample` itself keeps running it: that the command opens the sample, with
empty undo stacks, in memory, and with no missing image. `pack.rs`'s measurement of a pack against
its files, 318 bytes against 390, moves to the fixture, whose content is the same, and the figure
in `design@rootpoint-protocol@pack-container` is taken again at the step.

**Argument.** The approved outline changes every slide. 54 sites in 16 test files of the app, and 3
test files of the protocol, read today's slides by id, by text, by tree and by golden image. Moving
them to a frozen copy first lets the sample change with no test edit after step 1, and a test that
fails after step 1 is a fault of the new code, never of the content.

**Nearest rival**: keep today's slides and ids inside the new sample. Defeated by the outline: it
has fourteen slides with new subjects, so keeping s1, s7, s13, s20, s24 and s33 with their trees
binds the outline to the tests, which the owner's approval of the outline did not include.

### A host gives the session a picture source, which `sample` asks inside the command (#sample-host-pictures)

**Shape.**
- `Session` gains a field `pictures: Option<PictureSource>`, set by
  `Session::set_picture_source`, `None` in `Session::new`. Every place that resets the session,
  `*self = Session::new()` at five sites of `Session::run` today, keeps the field: a private reset
  method takes it out, builds the new session, and puts it back, so no site can forget it.
- `sample.rs` bundles one placeholder image, `planned@rootpoint-protocol@sample/placeholder.png`,
  and lists in `PICTURES` the asset paths of the sample that are pictures, all under `ui/`.
  `sample()` gives each of those paths the placeholder's bytes, as it gives `splits.png` its own.
- `Command::Sample`, after the reset, asks the source, when one is set, for each path of
  `PICTURES` in order, and puts each `Some` answer in the store's map in place of the placeholder.
  A `None` keeps the placeholder. It runs inside the one command, so the opened sample has empty
  undo stacks and no unsaved change, as today.
- The headless binary, `rootpoint-headless`, sets no source, so its sample holds the placeholders.
  The app sets one on its session in `App::new`. The source draws each picture in a staged window,
  an `App` of its own, which clears its session's source before it opens the sample, so drawing
  never recurses and the staged window shows the placeholders where the sample has pictures.

**Argument.** The protocol links no GUI toolkit, `design@rootpoint@gui-only-in-app`, so the drawing
is the app's, and the protocol needs a way to receive it. The reset of `sample` replaces the whole
session, `argument@sample-redesign@arg-24`, so the source must outlive the reset. Asking inside the
command keeps one command, one opened state, `argument@sample-redesign@arg-27`.

**Nearest rivals.** Replacing the placeholders afterwards through protocol commands creates undo
steps, so Ctrl+Z would take the pictures back. Leaving the picture missing headless makes the
sample name an image it does not have, a breach every headless open reports, which
`design@rootpoint-protocol@builtin-sample` rules out.

**What a placeholder is.** One PNG, flat and small, that says in words that the editor draws this
picture when it opens the sample. It is made once and committed; its size counts in the 50 KB of
`design@rootpoint-protocol@builtin-sample`, with a bound of 4 KB as a default of this spec.

### The window draws a staged frame, and the app fills egui's triangles on the CPU (#toolkit-snapshot)

**Shape.** `draw_picture` of `planned@rootpoint-app@src/pictures.rs`, for one row of the table:
1. builds a staged window, `App::new` on files in memory with no source, opens the sample, and
   applies the row's stage through the app's own actions;
2. runs frames on a fresh `egui::Context` with no window, as `path@rootpoint-app@tests/presenter.rs`
   drives `App::frame`: `Context::run_ui` with a `RawInput` whose `screen_rect` is `STAGE_SIZE` and
   whose viewport's `native_pixels_per_point` is `PICTURE_PPP`. It runs frames until two in a row
   give the same recorded rectangles, at most 4: egui sizes its panels again on the second frame,
   as `path@rootpoint-app@CLAUDE.md` says;
3. keeps every texture the frames send, from `FullOutput::textures_delta`: a delta with no `pos`
   replaces the whole texture, one with a `pos` writes its rectangle, and `free` removes one; each
   texture id holds a list of deltas;
4. draws the callouts, as the next subsection says, on a layer above the window's;
5. tessellates the last frame's shapes, `Context::tessellate`, and `rasterize` fills each mesh's
   triangles, clipped to the primitive's clip rectangle, into RGBA pixels of
   `STAGE_SIZE × PICTURE_PPP`, crops them to the row's region, and encodes them as PNG.

`rasterize` is the prototype's method: a triangle covers a pixel whose centre has three
non-negative barycentric weights, with no back-face culling, since egui's winding order varies;
the colour is the weighted vertex colour times the texture, sampled bilinearly; the pixel is
blended over the buffer as premultiplied alpha. A primitive that is no mesh, such as a paint
callback, or a mesh whose texture the frames did not send, is an error that `draw_picture`
returns, never a silent gap (AC5). A slide view's own texture is an egui texture like any other,
so the slide drawn by the render crate appears as in the window.

The prototype filled a frame of 1600×900 points in 30 ms at 1 pixel per point and 115 ms at 2, in
release, on an AMD Ryzen 9 9950X. Those times cover the tessellation and the filling only, not the
four frames run before them nor the creation of the context: step 2 measures the whole.

**Argument.** The picture is the window as built, by its own code, so it cannot go stale after a
rework of the interface, the owner's reason in `argument@sample-redesign@arg-4`; the same code runs
natively and in the browser, with no GPU, so a test checks it, `argument@sample-redesign@arg-13`. It
adds no content kind, `goal@rootpoint@minimal-content-set`: a picture is an image asset.

**Nearest rival**: #split-tree-mockup, the window mocked from slide areas. Defeated by the owner's
choice in round 2 and by `argument@sample-redesign@arg-17`: areas have no fill and no border, and a
mock-up drawn by hand goes stale. The rejected painter, `rejected@rootpoint-render@toolkit-painter`,
is not brought back: the slides stay the render crate's, and the picture is filled on the CPU, not
taken from the toolkit's offscreen path.

### The pictures are a table in the app of stages, regions and anchored callouts (#picture-staging)

**Shape.** `PICTURE_TABLE` holds one `PictureSpec` per path of the protocol's `PICTURES`, and a
test checks that the two lists name the same paths. A row gives:
- **its stage**: the slide shown, by id, the side panel's tab, and at most one of an area
  selected, a text edited in place with a range selected, or a formula edited. The stage is
  reached through the app's own actions on the staged window, never by setting its fields;
- **its region**: the whole window, or one panel the frame records by name: `toolbar`, `slides`,
  `side`, `view` or `status`. The frame records each panel's rectangle in `App::view.regions`,
  which `App::region_rect` returns, as `App::button_rect` returns a button's;
- **its callouts**: each a recorded label to anchor on, a number from 1, and a text of at most four
  words. An anchor is a label `App::button_rect` or `App::region_rect` finds. The callout is a
  filled circle holding the number, drawn at the anchor's top-left corner, and the text in a box
  beside it, both on a layer above every window layer, so the same rasterizer draws them. A slide
  that shows a picture explains each number in a sentence.

The toolbar's buttons draw through `App::button` instead of `ui.button`, so they record their
rectangles, `argument@sample-redesign@arg-32`; "Preview" and the state label already do.
`draw_picture` returns an error when an anchor is not recorded in the frame, or recorded twice
(AC4): a callout never points at a place the window does not have.

**Argument.** The callouts are what the owner asked for, overlays "that say what buttons do",
`argument@sample-redesign@arg-3`; anchoring them on recorded rectangles keeps them on the widget
after a rework, and makes a widget that no longer exists a test failure rather than a wrong picture,
`argument@sample-redesign@arg-28`.

**Nearest rival**: callouts placed at fixed coordinates in the table. Defeated by the owner's reason
for runtime pictures, `argument@sample-redesign@arg-4`: fixed coordinates go stale after a rework
exactly as a bundled image would.

### One picture of the whole window, then one per part, not exhaustive (#picture-focus)

**Shape.** The table's rows, as the work starts; the implementer may add a callout or change a
text within a row, and adds no row without the owner, since the owner asked that the pictures
not be exhaustive.

| path | slide | stage | region | callouts |
| --- | --- | --- | --- | --- |
| `ui/editor.png` | 2 | slide 3, one area selected, tab Selection | the whole window | 1 `toolbar`, 2 `slides`, 3 `view`, 4 `side`, 5 `status`, 6 `state` |
| `ui/outline.png` | 4 | slide 3, a nested area selected, tab Selection | `side` | 1 `Selection`, 2 the selected outline row, 3 `Styles`, 4 `Assets` |
| `ui/styles.png` | 8 | slide 8, tab Styles | `side` | 1 `New`, 2 `Duplicate`, 3 `New span style` |
| `ui/text.png` | 9 | slide 9, a text edited, one word selected, tab Selection | `side` | 1 `text`, 2 `Split here`, 3 `Insert formula` |
| `ui/formula.png` | 10 | slide 10, a formula edited, tab Selection | `side` | 1 `a/b`, 2 `√`, 3 `matrix` |
| `ui/assets.png` | 11 | slide 11, tab Assets | `side` | 1 `Import…`, 2 `Remove unused`, 3 `Replace…` |
| `ui/toolbar.png` | 12 | slide 12 | `toolbar` | 1 `Open…`, 2 `Open pack…`, 3 `Save`, 4 `Save as pack…`, 5 `state` |

The selected outline row is an anchor of its own kind: the row of `App::outline_rows` whose
`selected` holds. The callout on `Replace…` needs an asset selected in the tab: the stage selects
`splits.png`. Every label above is one the window records today or records once the toolbar draws
through `App::button`, and each exists natively and in the browser; the step that builds the table
checks each against the frame.

**Argument.** The owner's words in round 2: the pictures are "several each focussed one one menu",
"maybe one picture with the full ui to name each part of it before going into details", and not
"fully exhaustive". The whole-window picture was a default the owner ruled on in round 4, D2.

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

A picture of the side panel is tall and narrow, about 520 by 1,700 pixels at the default width of
260 points, so its area is a column at most 520 pixels wide at 1920×1080 (AC7). Slide 10 does the
work of the protocol's issue `sample-teaches-no-math-editing`, closed when this spec was added: its
text names a second click on a formula to edit it, Alt+M then f for a fraction, `\` to type a
command, and the palette of the Selection tab, and a text of the slide holds an inline formula and a
reference to its equation. The sample's texts use only characters the bundled fonts hold,
`tripwire@rootpoint-render@glyph-missing-everywhere`.

**Argument.** The owner's request, `argument@sample-redesign@arg-1` and
`argument@sample-redesign@arg-2`: the sample looks bare, does not use the split tree, and its
descriptions are hard for a new user. The owner approved the outline in round 2.

### The pictures of the editor are drawn by the editor itself (#ui-picture-source)

**Shape.** The shapes of #toolkit-snapshot, #sample-host-pictures, #picture-staging,
#picture-focus and #picture-cost above.

**Argument.** The owner's reading of Q2 in round 2, reading (a), "a picture of the editor drawn by
itself", with the owner's reason from round 1, `argument@sample-redesign@arg-4`: the interface will
be reworked, and a bundled picture would go stale. Reading (b), a picture drawn by the slide
renderer from slide content, is #split-tree-mockup's shape and loses with it.

## Mapping tables

**Each test site that runs `sample` today, to what it runs after step 1.** The sites are the 54 of
the app and the 3 files of the protocol that "What the work is" counts.

| the site | after step 1 |
| --- | --- |
| an app test that reads the sample's content: an id, a text, a tree, a style, an asset or a golden image | the helper of `planned@rootpoint-app@tests/common/mod.rs`, which opens the fixture |
| an app test whose subject is the action that opens the sample: `App::open_sample`, the toolbar's "Sample", the browser opening it at start | unchanged: it still opens the sample, and asserts nothing of its content but what `design@rootpoint-protocol@builtin-sample` states |
| a protocol test that reads the sample's content, in `pack.rs`, `styles.rs` or `commands.rs` | `open-copy` of the fixture's directory |
| a protocol test whose subject is `sample` | unchanged, as the second row |
| `pack.rs` reading `path@rootpoint-protocol@sample/splits.png` | reads the fixture's `splits.png` |

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
as a comment on `epaint::Mesh` says. The harvest records these in the head of
`sample-pictures`, entry test 3.

## Premortem

Run by the session in round 3, in the convergence proposal, as "assume it shipped and failed". The
owner's ruling in round 3: "keep all AC and tripwires, proceed".

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
recorded, or recorded twice. Fires when that test fails on the built table. Response: the row is
repaired if the anchor's label changed; if no recorded rectangle can anchor the callout, the
anchoring of #picture-staging is reopened.

### AC5: every primitive of every picture is drawn `##ac-every-primitive`

Guards #toolkit-snapshot. Judged at step 4, by a test of the same file that draws every row and
fails when `draw_picture` reports a primitive that is no mesh or a texture it was not sent. Fires
when that test fails. Response: #toolkit-snapshot is reopened for that primitive: the rasterizer
learns to draw it, or the stage avoids it, on the owner's word.

### AC6: all pictures are drawn in at most 500 ms natively, in release `##ac-pictures-time`

Guards #picture-cost. Judged at step 4, by an ignored test of the same file, run by hand in
release like the project's other timing tests, that times one `sample` of an app's session with
its source set, every picture drawn. The bound, 500 ms, is the owner's. Fires when the time
exceeds it on the machine the step names. Response: #picture-cost is reopened: lazy drawing, a
lower density, or fewer pictures.

### AC7: each picture is at least as wide in pixels as its area at 1920×1080 `##ac-picture-width`

Guards #toolkit-snapshot and the density of #picture-cost. Judged at step 4, by a test of the same
file that, for each slide of the sample showing a picture, compares the picture's width in pixels
with its area's width in slide units, which `area_rects` of
`path@rootpoint-model@src/geometry.rs` gives and which are pixels at 1920×1080. Fires when
one picture is narrower. Response: the slide's layout is changed first; if no layout of the
outline fits, #picture-cost's density is reopened.

## Implementation sequence

### Step 1: the tests open the fixture

**Builds**: `planned@rootpoint-protocol@tests/data/sample-fixture/`, the helper of
`planned@rootpoint-app@tests/common/mod.rs`, and every test site moved as the first mapping table
says. No code of `src/` changes.

**Claims**:
- the fixture is byte for byte today's `sample.ron` and `splits.png`: `cmp` of each pair, in the
  commit message;
- every moved test passes unchanged in what it asserts: the gates. A wrong implementation, the
  helper opening an empty presentation, fails the tests that read s1's tree, which the step shows
  once by that mutation;
- after the step, no test but those of the mapping table's "unchanged" rows runs `sample`: the
  grep of "What the work is", run again, lists each remaining site and its subject.
- the pack measurement of `pack.rs` gives the same figures on the fixture, 318 and 390 bytes, or
  the head of `design@rootpoint-protocol@pack-container` takes the new ones in this step.

**Fails alone on**: a moved test that fails, since nothing else changed.

### Step 2: the session keeps a picture source, and `sample` asks it

**Builds**: `PictureSource`, `Session::pictures`, `Session::set_picture_source`, the reset that
keeps it, `PICTURES` empty, and the asking in `Command::Sample`. Nothing sets a source, so nothing
a user sees changes.

**Claims**, each a test of `path@rootpoint-protocol@tests/commands.rs` with a test source and a
test list of paths:
- a source set before `sample` gives its bytes to each path it answers, and a `None` keeps the
  placeholder;
- the source survives each of the five resets of `Session::run`: a test runs each resetting
  command, then `sample`, and finds the source's bytes. A wrong implementation, a reset that drops
  the source, fails it;
- after `sample` with a source, the undo stack is empty and nothing is unsaved.

**Fails alone on**: the protocol's tests, since the app sets no source yet.

### Step 3: the app draws a picture

**Builds**: `planned@rootpoint-app@src/pictures.rs` with `rasterize`, `draw_picture`, the stages,
the regions and the callouts; `App::view.regions` and `App::region_rect`; the toolbar's buttons
drawn through `App::button`; `image` as a normal dependency. `PICTURE_TABLE` is empty and no source
is set, so nothing a user sees changes. Measures the cost of one picture of the whole window, frames
and context included, in release, against the prototype's figures.

**Claims**, tests of `planned@rootpoint-app@tests/pictures.rs` with test rows on the fixture:
- a picture of the whole window has `STAGE_SIZE × PICTURE_PPP` pixels, and the pixel at the
  centre of the "Preview" button's recorded rectangle has the colour of a selected toggle or of the
  button's fill, not the background: a test with a wrong scale, 1 pixel per point, fails it;
- a crop to `toolbar` has the toolbar's recorded size times the density;
- a callout drawn on "Save" puts the callout's colour at the top-left corner of "Save"'s rectangle;
- a callout on a label the frame does not record is an error, and so is a label recorded twice;
- a primitive that is no mesh is an error: a test frame that adds a paint callback gets one;
- `cargo x gates` builds the module for the wasm target.

**Fails alone on**: a test of `pictures.rs` with the test rows, since the sample is unchanged.

### Step 4: the new sample, its pictures and the source

**Builds**: the new `sample.ron` of fourteen slides,
`planned@rootpoint-protocol@sample/placeholder.png`, `PICTURES` and `PICTURE_TABLE` as
#picture-focus gives them, and the source set in `App::new`. `splits.png` stays.

**Claims**:
- the sample opens headless with no breach and no missing image, and its bundled images stay under
  50 KB: `path@rootpoint-protocol@tests/headless.rs` or a test beside it, with the sum of the
  bundled bytes printed;
- `PICTURES` and the table's paths are the same set: a test;
- AC4, AC5 and AC7: the tests named in "Acceptance criteria";
- AC6: the ignored test, run by hand in release, its time in the commit message with the machine;
- every slide's text uses characters the bundled fonts hold: a test that renders each slide and
  finds `Report::missing_glyphs` empty, the report
  `tripwire@rootpoint-render@glyph-missing-everywhere` names;
- the browser draws the pictures: a new test of `path@rootpoint-app@tests/web.rs` opens the sample
  in an app made by `App::in_memory` and finds, for each path of `PICTURES`, bytes other than the
  placeholder's. It prints the time `sample` took in the headless browser, a first figure for T2,
  which the commit message reports with the machine; it is not the owner's machine, so it fires
  nothing.

**Fails alone on**: the content, a slide's text or layout, or a row of the table, since the
mechanism passed steps 2 and 3.

### Step 5: the record

**Builds**: the rewritten `path@rootpoint-protocol@README.md` on the sample's slides, and every
text "What is already decided" lists as updated at the harvest; the report on AC4 to AC7; the
harvest.

**Claims**: `cargo klarch check` passes, and `cargo klarch show` of each rewritten head lists no
text that still describes the sample of six slides: the check and a grep of "six slides" in the
commit message.

**Fails alone on**: the checker, since the code is unchanged.

## Order rationale

- Step 1 before step 2: the tests stop reading the sample before any code can change it, so a
  later test failure is never a fixture's.
- Step 2 before step 3: the app's source has a place to plug into, proven in the protocol alone.
- Step 3 before step 4: the drawing is proven on test rows before any content depends on it.
- Step 4 before step 5: the record describes what was built and measured.

## Defaults awaiting the owner

- **D1, ruled in round 4.** Bears on #sample-host-pictures, #picture-staging and #picture-cost: the
  session read the owner's "proceed" of round 3 as approving the three threads the checkpoint table
  showed as proposed. The owner agreed in round 4.
- **D2, ruled in round 4.** Bears on #picture-focus: the owner wrote "maybe" for a picture of the
  whole window naming its parts; the default included it, as slide 2. The owner agreed in round 4.
- No acceptance criterion awaits the owner: AC4 to AC7 were kept in round 3.

## Harvest

At step 5, in the commits that land the work, under
`skill@knowledge-architect-decision-recording` and `skill@knowledge-architect-issue-tracking`. The
primer's rules on design heads decide how the threads are cut into heads and whether each earns
one; a thread's name is its head's slug, as those rules say.

| what | where | from |
| --- | --- | --- |
| the pictures of the window, drawn by the window and filled on the CPU, their staging, their callouts and their cost | new heads in `register@rootpoint-app@design`, under the slug `sample-pictures` or the threads' own: #ui-picture-source, #toolkit-snapshot, #picture-staging, #picture-focus, #picture-cost | "Decided design" |
| the picture source, the slides' subjects and the tests' fixture | `design@rootpoint-protocol@builtin-sample`, rewritten in place | #sample-host-pictures, #sample-slide-plan, #sample-test-fixture |
| the window mocked from slide areas | `register@rootpoint-app@rejected`, if a recording test of the decision-recording skill holds, cited by the head of #toolkit-snapshot | #split-tree-mockup |
| T2 | `register@rootpoint-app@tripwire`, guarding the head of #picture-cost | "Premortem" |
| AC4 to AC7 | reported in the landing commits; each one that did not fire and recurs at later work is proposed to the owner as a tripwire | "Acceptance criteria" |
| the measurement of the pack | `design@rootpoint-protocol@pack-container`, taken on the fixture | step 1 |
| the premise of `autosave_gui_cost` and the tests named by `design@rootpoint-app@edit-in-place` and `design@rootpoint-app@editor-overlay` | those heads, restated on the fixture | step 1 |
| the six slides described | `path@rootpoint-protocol@README.md` | step 4 |
| the tests open the fixture, never the sample, unless `sample` is their subject | `path@rootpoint-app@CLAUDE.md` and `path@rootpoint-protocol@CLAUDE.md` | #sample-test-fixture |
| a picture's callout anchors on a recorded label | `path@rootpoint-app@CLAUDE.md`, its section on the places the last frame records | #picture-staging |
| #live-ui-tour | `issue@rootpoint-app@interactive-tour`, opened by the commit that adds this spec | "Losing alternatives" |

## Later consequences

- **The interactive tour**, `issue@rootpoint-app@interactive-tour`, adds callouts over the real
  window; it may reuse the anchors of #picture-staging, and replaces no picture.
- **The rework of the interface** the owner plans redraws every picture with no step by hand. A
  label that changes fails AC4's test, and its row of `PICTURE_TABLE` is repaired then.
- **A configuration of the keys**, `issue@rootpoint-app@configurable-keys`, leaves the keys the
  slides name stale; that work updates the sample's texts.
- **T2 firing** replaces drawing at the open by drawing at first show, under #picture-cost.
- **A change of the sample** no longer touches a test, except those whose subject is `sample`.
