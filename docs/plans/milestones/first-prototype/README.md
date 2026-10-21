# Milestone: the first prototype of rootpoint

## Status and audience

This is the milestone document of the first prototype of rootpoint: an editor with a GUI that lays
out slides as trees of splits, puts plain text or an image in each area, saves to a readable text
file, shows a presenter mode, runs headless, is driven by text commands and builds for the browser.
It is written for the sessions that implement its slices, which did not witness the discussion that
produced it. It leaves the repository when its last slice lands; each slice's spec leaves when that
slice lands.

- Where this document, or a slice's spec, and a design home disagree on a point that "What is
  already decided" does not list as reversed or rewritten, the design home wins.
- Every name it uses is defined in it, in "Names" or "New names, in one place", or exists in the
  code.
- Where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says what
  waits. The defaults awaiting the owner are listed in their own section.
- The milestone was assembled from the transcript of one session. The discussion begins in it at
  the owner's message that opens "No need for review here, we only wrote goals and basic setup",
  and the owner's two first messages of that session, which open "Hey ! I just setup a blank
  project" and "Good, thanks for the analysis", carry proposals the discussion argued.

**The owner's messages in the session**, verbatim, in order. O1 to O9 come before round 1, O10
answers round 1, O11 round 2, O12 round 3.

**O1**

> Hey ! I just setup a blank project using the knowledge-architect workflow. I haven't setup much outside of the skill set and a nearly empty knowledge.toml.
>
> About what I intend to do here: a minimalistic presentation editor, with an interactive gui, written in rust.
>
> I'm tired of having to master a programming language, having to write a build procedure, or battle with slow and complex graphical editors everytime I have to make a presentation. 
> The necessary ingredients to make a good presentation are quite small IMO: I just need to be able to split the screen in different parts freely and conveniently, and in each split area, put one piece of content, among those kinds: wrapped text, bullet list, image, equation, code snippet (and maybe a few more, but this is already enough for my current needs).
> Some existing tools are already going toward this kind of direction AFAIK, but none have really tried to focus on the minimalistic aspect IMO, and they are often based on code languages (usually markup kind, which is the worst kind of language). So I want to try making my own tool, with a gui.
>
> The idea I had to enable convenient splitting of a slide is too use a tree shaped list of splits: recursively, allow to split an area in two parts, either vertically or horizontally, with the split position being movable by drag.
> Then, for each area defined this way, choose a kind of content among those I listed above, and allows editing it in the ways that make sense under the given kind, and with options that go with it (such as bold/color control for text, scaling method for image,...).
>
> For now, I really want to go for the minimal part:  a gui, that allows editing the slides interactively this way, the first prototype handling only images and simple text, and no need to convert to pdf, instead allow seerializing the slides in a text file for saving (probably json or ron), and display in presentater mode.
> I want to keep the implementation as modular as possible, in particular I want the core logic to be able to run in headless mode. I want the project to be split into components/crates, with the condition being something along the line of "a crate exists when it has its own tests, its own public API, and a reason to change on its own".
>
> Also, as I try to do in most of my projects with GUI nowadays, I want to provide a "text protocol" interface, which allows driving the gui and the common operations by writing commands, the point being to allow usage by LLM (like you) live with the gui, which is useful for testing and for making demos, as well as for headless testing. This might be less useful is the full slide description serializes easily, but it is still quite useful IMO.  
>
> To be honest, I can guess this might be harder than what I'm thinking, and the result might be a lot less convenient than I am hoping for, but I want to try it out anyway.
>
> What do you think of this project idea ?
>
> In term of work order, first thing first, we'll finish setting up the knowledge-architect workflow, and only then have a design discussion for the initial implementation.
> Or maybe we could have the design discussion first, since it might reshape my goals, or the design idea might even crumble from the very start...

**O2**

> Good, thanks for the analysis, this reassures me about the project, and the difficulty analysis gives a clear idea of how to progress on this.
> About my layout idea being known: is there no presentation tool already making use of it ? TBH, this surprises me, as the idea is quite natural, and in latex, I have actually written some basic makeshift macros to create my slides in a similar way. I used tikz figures taking the whole slide area and placed nodes in screen ratio coordinates. This was a bit painful to use though...
> About rich text editing: I know it will be the hardest one. as you said, first prototype can go with style per text area as you say, this is more than enough for me.
> About binary split not expressing everything: the case you describe would be handled by having a horizontal split span multiple vertical areas, or "linking" two splits together so each follow the other one.
> About consistency: with my model, I think defining things like a common background is quite easy. Having a reserved area on all slides (such as a footer) is easy too. Reusing predefined layouts too. And I think that covers most of what is needed. More generally, instead of a piece of content being tied to an area, I was thinking it could just be stored outside, and an area would just hold a reference to it, so they would be reusable, and scaling and such would be applied locally. It might not be that useful though... but it costs nearly nothing IMO.
> I agree the equation thing is probably harder to do. But how hard can it be to make a math renderer actually ? It can be a minimal set too. Anyway, this is a question for later.
>
> I wanted to react to your highlighted difficulties while hot in my head, so that's done.
> Let's proceed with your recommended order, finishing the minimal working setup, then setting up goals.

**O3**

> Wait a bit, I want the xtask tool to be placed under tools/xtask/, not directly at root.

**O4**

> I think I interrupted the survey subagent, can you restart it ?

**O5**

> I agree with your judgements for Q1,Q2,Q3.
> Q4: 
> Who is it for: me only at the beginning, then probably published to crates.io later if it works out well.
> What would show failure: me deciding it, basically. If I find it not as convenient as I hoped in a minimal prototype, or too hard to push to the point of being usable for real presentations (I kinda needs math in the long term, though images, text and code snippets already cover a good part of my needs, and worst case I can prebuild equations as images). 
> What it is not for: animations are one kind of content I'd like, though only as a video content piece probably. PDF exports are not fully ruled out (but it can be restricted to an image export I guess, especially early on.), the main way to present is using the tool itself. Maybe having the project build to webasm could be a plus for that, allowing easy portability/accessibility. collaborative editing is not in scope.

**O6**

> Q5: right, it stays a goal.
> Q6: I agree on all goal wordings, including the new one real-presentations.
> On second thought, webasm should be a goal too, at least initially. One that could be abandonned later it if imposes too much constraints.

**O7**

> resume

**O8**

> Q7 approved

**O9**

> No need for review here, we only wrote goals and basic setup. We'll be merging when we have started writing something that actually needs a review. I'll be creating the remote while you start the design discussion.

**O10**

> Q1: for prototyping, shape 1 is fine. I think that shape 1 would stay the preferred editing method long term, while shape 3 would be the final goal, so it is possible to pack your presentation, carry just the file with you, and present it from any computer with a web browser and internet connection using the webapp hosted from a static site. Does that make sense ?
>
> I agree with your positions over this whole table. b for the renderer, I think that precise control of the renderer is important for the project. n-ary split is better as you pointed out. I agree with your inline-content-assets, the reference thing I proposed brings more trouble (especially ui side, for both dev and usage) than convenience. 
> slide-templates parked for later.

**O11**

> Q2: both rewording approved, thanks for the proposal.
> I agree on the new font-sources and asset-source too

**O12**

> All tripwires and AC can be kept, proceed

## How the work is done

Before each slice, read `skill@knowledge-architect-planning@working-a-slice` whole, and follow it.
Each slice is one branch and one pull request, merged before the next slice begins.

The project's gates run with `cargo x gates`; every slice keeps them passing. The project has no
development procedure of its own: point 3 of that procedure is the claims, tests and steps of the
slice's spec, with the gates run before each commit.

- **The wasm gate.** The slice `model-core` created one gate named `wasm`. Each crate
  the browser build links joins it in the slice that creates it: the model, render and protocol
  crates. The app crate is created native in `spec@first-prototype@editor-shell` and joins it in
  `spec@first-prototype@browser-build`, the slice that builds its web target. This keeps
  `thread@first-prototype@prototype-scope`'s wasm build checked from the first slice; the per-crate
  cut is this milestone's.
- **The checkout tie.** Every library root, binary target and build script a slice adds reads
  `ROOTPOINT_CHECKOUT`, as `path@xtask@CLAUDE.md` requires.
- **Branches.** This milestone's branch merges into the main branch before
  the slice `model-core` begins, and each slice branches from the main branch. The
  branch also carries the commits that set up the workflow and state the goals; the owner exempted
  those from review and ruled that they merge with the first branch whose work needs one, which is
  this one.

## Names

- **C1 to C10**: the criteria, the items of "Criteria" below, in the order they are listed.
- **AC1 to AC5**: the acceptance criteria, labelled in their items. **T1 to T4**: the tripwires of
  the premortem.  **Q1 to Q3**: the questions put to the owner in rounds 1 to 3, whose answers are
  O10 to O12; **Q4**, put to the owner at the review of this document. The labels Q1 to Q7 inside
  O5, O6 and O8 and in `argument@first-prototype@arg-20`, `argument@first-prototype@arg-21` and
  `argument@first-prototype@arg-25` are those of the goal setting, A2 to A4, a separate numbering.
  **D1 to D4**: the defaults put to the owner, listed in their own section.
- **O1 to O12**: the owner's messages, quoted in "Status and audience".
- **A1 to A8**: the session's messages: A1 the first analysis of the idea, A2 to A4 the drafts of
  the goals, A5 round 1, A6 round 2, A7 round 3 with the checkpoint and the premortem, A8 the
  proposal of this milestone.
- **round 1, 2, 3**: the session's messages that opened round 1 (grounding, criteria, threads),
  round 2 (the answer to Q1, two new threads) and round 3 (the checkpoint and the premortem), each
  with the owner's answer.
- **slide units**: the coordinate space of a slide, 1920×1080 for the 16:9 aspect ratio, in which
  every size is stated. See `thread@first-prototype@slide-coordinate-space`.
- **guillotine layout**: a layout made by straight cuts that each run across the whole area being
  cut.
- **the model, render, protocol and app crates**: `rootpoint-model`, `rootpoint-render`,
  `rootpoint-protocol` and `rootpoint-app`, under `path@rootpoint@crates/`.
- **the gates**: the command `cargo x gates`, served by `path@xtask@src/main.rs`.
- **the wasm target**: the Rust target `wasm32-unknown-unknown`.
- **(V), (A)**: a fact verified by reading a source, or assumed, in the surveys quoted by the
  arguments.
- **the prior-art survey, the toolkit survey**: two subagent reports of the session, on slide tools
  that use split trees, and on Rust GUI toolkits and renderers; their facts are quoted in
  `argument@first-prototype@arg-19` and `argument@first-prototype@arg-26`. "The blitz wasm notes"
  is the file `HOWTO_WASM.md` of the blitz repository, which the toolkit survey read.
- **difficulty 1 to 5**: the items of A1's list of what is harder than it looks, each quoted in its
  argument. **best combination 1**: the first of the toolkit survey's recommended stacks, eframe
  hosting a separate renderer.
- **shape 1, 2, 3** of image storage: 1, image files beside the presentation, referenced by relative
  path; 2, images embedded in the text file as base64; 3, both, by commands that pack and unpack.
- **the i3 claim**: round 1's statement that i3 uses n-ary containers, labelled as from memory and
  not checked.

## What the work is

**What exists today.**

- The workspace, `path@rootpoint@Cargo.toml`, has one member, the maintenance crate at
  `path@rootpoint@tools/xtask/`. Its `main` builds the gate list with
  `rust_project(checker, "origin/main")` from the library `knowledge-architect-gates` 0.8.0, whose
  `Gate` struct is public and whose list a project may extend by pushing a `Gate`.
- No editor code existed when this document was written; the slice `model-core` has since built
  `path@rootpoint@crates/rootpoint-model/` and harvested its decisions into its design homes and
  rootpoint's.
- The Rust targets installed on the owner's machine are `x86_64-unknown-linux-gnu` only, measured
  with `rustup target list --installed`; the wasm target is not installed.

**What the work builds**: the four crates of `thread@first-prototype@crate-split`, a wasm gate per
crate, and the prototype `thread@first-prototype@prototype-scope` lists, in the seven slices of
"Implementation sequence".

**Outside the work**, each owned elsewhere:

- bullet lists, code snippets, equations and video: later content kinds, per
  `goal@rootpoint@minimal-content-set`; equations are wanted later, per
  `goal@rootpoint@real-presentations`;
- packing and unpacking a presentation into one file: the later half of
  `thread@first-prototype@image-storage`;
- templates, backgrounds and footers: `thread@first-prototype@slide-templates`, parked;
- linked dividers: `thread@first-prototype@linked-dividers`, parked;
- rich text with per-span styles: excluded by `thread@first-prototype@text-style-per-area`;
- image and PDF export: secondary, per `goal@rootpoint@presenter-mode`;
- the protocol's transports, stdin and the local socket, in the browser build: excluded by
  `thread@first-prototype@protocol-transport`; the protocol's session is part of the browser build;
- collaborative editing: out of scope, per `goal@rootpoint@minimal-content-set`.

## What is already decided

**Recorded decisions the design rests on**: none. The design homes and the rejected homes of
rootpoint and xtask held no entry when this document was written. The work reverses or rewrites no
recorded decision.

**The goals it serves**, all of `register@rootpoint@goal`, each the source of a criterion below:
`goal@rootpoint@real-presentations`, `goal@rootpoint@no-language-no-build`,
`goal@rootpoint@free-slide-division`, `goal@rootpoint@minimal-content-set`,
`goal@rootpoint@presenter-mode`, `goal@rootpoint@readable-save-file`,
`goal@rootpoint@headless-core`, `goal@rootpoint@text-command-protocol` and
`goal@rootpoint@runs-in-browser`.

**What the discussion's searches returned.**

- The standing-entry search returned one entry, the `todo` issue `build-the-editor` of rootpoint:
  no goal of rootpoint was met and no plan scheduled the work. This milestone schedules it, so the
  commit that adds this document deletes that issue, as its own "What would close it" asked.
- The design-entry search read the nine goals, no design entry and no rejected entry, and graded
  every goal `bears`, each as its criterion below states, and none `may-bear`. It graded
  `goal@rootpoint@runs-in-browser` "may change", since the goal says the owner may abandon it;
  a thread that strains it goes to the owner.

## Criteria

### Every operation the GUI offers runs and is checkable with no window `##c1-headless-editing`

Binding. Source: `goal@rootpoint@headless-core`, whose body reads every operation the GUI offers.
Met by `thread@first-prototype@edit-operation-core` and `thread@first-prototype@crate-split`: the
model crate has no GUI dependency; and by the view commands,
`design@rootpoint-protocol@view-state`, through which the GUI selects and presents.

### Text commands drive a live GUI and a headless session `##c2-command-driving`

Binding. Source: `goal@rootpoint@text-command-protocol`. Met by
`thread@first-prototype@protocol-addressing`, `thread@first-prototype@protocol-transport`,
`thread@first-prototype@edit-operation-core` and `thread@first-prototype@slide-renderer`, which
gives the headless PNG.

### The save file is human-readable text, and saving then loading gives back the same presentation `##c3-readable-roundtrip`

Binding. Source: `goal@rootpoint@readable-save-file`. Met by `thread@first-prototype@save-format`,
`thread@first-prototype@image-storage` and `thread@first-prototype@asset-source`.

### The same editing operations and presenter mode work in a browser build `##c4-browser-parity`

Binding; the owner may abandon the goal it derives from. Source: `goal@rootpoint@runs-in-browser`.
Met by `thread@first-prototype@gui-toolkit`, `thread@first-prototype@slide-renderer`,
`thread@first-prototype@font-sources` and `thread@first-prototype@prototype-scope`, which gates the
wasm build from the first slice. Partly assumed, so checked by
`acceptance@first-prototype@browser-build-holds`. Per D3, ruled, the prototype's browser build is
load only: it opens a presentation directory and the built-in sample, edits them in memory, and does
not save; saving in the browser comes with packing.

### Authoring needs no code or markup `##c5-no-markup`

Binding. Source: `goal@rootpoint@no-language-no-build`. Met: the GUI is the authoring path, and the
protocol and the file are optional to an author.

### Dividing and resizing areas is fast and needs no coordinates `##c6-fast-division`

Binding. Source: `goal@rootpoint@free-slide-division`. Met by
`thread@first-prototype@split-tree-arity` and `thread@first-prototype@split-any-node`; judged by the
owner in `acceptance@first-prototype@split-editing-convenient`, which compares with a
general-purpose slide editor, as the goal does, per D4.

### Every feature serves a content kind or slide division `##c7-minimal-features`

Binding. Source: `goal@rootpoint@minimal-content-set`. Met by
`thread@first-prototype@prototype-scope` and `thread@first-prototype@inline-content-assets`, as the
goal reads since the owner approved its rewording at Q4: every feature of slide content serves a
kind or the division, and the features other goals require are not counted against it.

### The editor and the presenter show the same slide at any window size `##c8-same-slide-everywhere`

Binding. Source: `goal@rootpoint@presenter-mode` with `goal@rootpoint@real-presentations`. Met by
`thread@first-prototype@slide-coordinate-space`, `thread@first-prototype@slide-renderer` and
`thread@first-prototype@font-sources`.

### The prototype is small enough that the owner can judge convenience early `##c9-early-judgement`

Weighed. Source: `goal@rootpoint@real-presentations`. Met by
`thread@first-prototype@prototype-scope` and the two parked threads.

### A crate exists when it has its own tests, its own public API and its own reason to change `##c10-crate-rule`

Binding, on the owner's word in O1. Met by `thread@first-prototype@crate-split` and
`thread@first-prototype@crate-existence-rule`.

## Threads

### Text is styled per area in the prototype `##text-style-per-area`

Proposed by the session before round 1 (`argument@first-prototype@arg-9`), adopted by the owner in
O2: "first prototype can go with style per text area as you say, this is more than enough for me."
Final state: approved. Shape: built, and recorded in `design@rootpoint-model@text-style-per-area`.
Harvest: a
design head of the model crate's Component, if the entry tests admit it.

### A crate exists when it has its own tests, its own public API and its own reason to change `##crate-existence-rule`

Proposed by the owner in O1 (`argument@first-prototype@arg-2`): "a crate exists when it has its own
tests, its own public API, and a reason to change on its own". Before round 1 the session proposed
recording it in the design home rather than as a goal (`argument@first-prototype@arg-21`), and the
owner answered in O5: "I agree with your judgements for Q1,Q2,Q3." The owner's O1 states the rule as
"something along the line of" that sentence, and the attribution keeps that hedge. Final state:
approved. Shape: "The crate rule" in "Decided design" below. Harvest: a design head of rootpoint,
attributed to the owner with that hedge, at the harvest of the slice `model-core`.

### Splits are n-ary, and a split along its parent's axis merges into the parent `##split-tree-arity`

Proposed by the owner in O1 as binary splits (`argument@first-prototype@arg-1`); the session raised
the limit of binary splits before round 1 and proposed the n-ary rival in round 1
(`argument@first-prototype@arg-10`, `argument@first-prototype@arg-27`). Closed in O10: "n-ary split
is better as you pointed out." Final state: approved; the binary shape lost (see "Losing
alternatives"). Shape: built, and recorded in `design@rootpoint-model@split-tree-arity`. Harvest: a
design
head of the model crate's Component, citing the rejected binary shape; T1 is written in that
Component's tripwires home.

### Any node can be split, so an area can span several others `##split-any-node`

Proposed by the session in round 1 from the owner's spanning remark in O2
(`argument@first-prototype@arg-14`, `argument@first-prototype@arg-28`). Closed by O10: "I agree
with your positions over this whole table." Final state: approved. Shape: built, and recorded in
`design@rootpoint-model@split-any-node`. Harvest: a design head of the model crate's Component.

### Linked dividers are left out of the prototype `##linked-dividers`

Proposed by the owner in O2 (`argument@first-prototype@arg-14`); the session proposed leaving it out
while giving every node a stable id (`argument@first-prototype@arg-29`). Closed by O10's whole-table
agreement, round 1's default being "parked". Final state: parked. Tripwire T3, recorded on the
owner's word (see "Premortem"); re-entry point: the first real talk given with the prototype.
Harvest: T3 as the trigger of a `deferred` issue, since a parked thread records no decision for a
tripwire to guard.

### A presentation has one aspect ratio, sizes are in slide units, and every display scales uniformly `##slide-coordinate-space`

Proposed by the session in round 1 (`argument@first-prototype@arg-11`,
`argument@first-prototype@arg-30`), "closes on its argument" unless contested. Closed by O10's
whole-table agreement. Final state: approved. Shape: "Decided design" below. Harvest: the slide
units and the area geometry in the model crate's Component, at the slice `model-core`;
the uniform scaling of every display in rootpoint's design home, since the render and app crates
both respect it, at `spec@first-prototype@editor-shell`.

### Content stored apart, with areas holding references to it `##content-by-reference`

Proposed by the owner in O2 (`argument@first-prototype@arg-16`); the session listed its costs in
round 1 (`argument@first-prototype@arg-31`). Withdrawn by the owner in O10: "I agree with your
inline-content-assets, the reference thing I proposed brings more trouble (especially ui side, for
both dev and usage) than convenience." Final state: withdrawn, with that defeating reason. Harvest:
judged as a rejected design at the harvest of the slice `model-core`.

### Content lives in its area, and heavy data in a presentation-level asset table `##inline-content-assets`

Proposed by the session in round 1 (`argument@first-prototype@arg-32`), as the rival of
`thread@first-prototype@content-by-reference` at a fork. Closed in O10: "I agree with your
inline-content-assets". Final state: approved. Shape: built, and recorded in
`design@rootpoint-model@inline-content-assets`. Harvest: a design head of the model crate's
Component, citing the
rejected shape of `thread@first-prototype@content-by-reference` if it earns an entry.

### Background, footer and reusable layouts come from templates, left out of the prototype `##slide-templates`

Proposed by the owner in O2 (`argument@first-prototype@arg-15`) and shaped by the session in round 1
(`argument@first-prototype@arg-33`). Parked by the owner in O10: "slide-templates parked for later."
Final state: parked. Tripwire T4, recorded on the owner's word; re-entry point: the first real talk
given with the prototype. Harvest: T4 as the trigger of a `deferred` issue, as T3.

### Every change is an operation value, and applying it returns its inverse `##edit-operation-core`

Proposed by the session before round 1 and in round 1 (`argument@first-prototype@arg-7`,
`argument@first-prototype@arg-34`). Closed by O10's whole-table agreement. Final state: approved.
Shape: built, and recorded in `design@rootpoint-model@edit-operation-core`. Harvest: a
design head of the model crate's Component.

### Slides are drawn by the project's own renderer, which the GUI hosts `##slide-renderer`

Proposed by the session in round 1 as shape (b), on the toolkit survey
(`argument@first-prototype@arg-26`, `argument@first-prototype@arg-35`). Closed in O10: "b for the
renderer, I think that precise control of the renderer is important for the project." Final state:
approved; shape (a) lost. Shape: built, and recorded in `design@rootpoint-render@slide-renderer`.
Harvest: a design head of the render crate's Component, standing on its argument and on the owner's
stated reason.

### The GUI host is eframe `##gui-toolkit`

Proposed by the session in round 1, conditional on `thread@first-prototype@slide-renderer`
(`argument@first-prototype@arg-36`): "eframe, if #slide-renderer closes on (b). If (a) wins
instead, iced." Closed by O10's whole-table agreement, with the renderer on (b). Final state:
approved. If `thread@first-prototype@slide-renderer` is reopened toward (a), iced is the host the
discussion named.
Shape: the decided design of `spec@first-prototype@editor-shell`. Harvest: a design head of the app
crate's Component.

### Slides and nodes are addressed by stable ids `##protocol-addressing`

Proposed by the session in round 1 (`argument@first-prototype@arg-37`). Closed by O10's
whole-table agreement. Final state: approved. Shape: built, for the ids, in
`design@rootpoint-model@stable-ids`, and, for addressing, in
`design@rootpoint-protocol@protocol-addressing`.
Harvest: the rule that ids are stored and never reused, in the model crate's Component; addressing
by id in the protocol, in the protocol crate's Component, citing the first.

### Commands are text lines, read from stdin headless and from a local socket by the live GUI `##protocol-transport`

Proposed by the session before round 1 and in round 1 (`argument@first-prototype@arg-8`,
`argument@first-prototype@arg-38`). Closed by O10's whole-table agreement. Final state: approved;
JSON-RPC lost. Shape: built, for stdin, in `design@rootpoint-protocol@protocol-transport`; for the
socket, the decided design of `spec@first-prototype@live-control`. Harvest: the line grammar and the stdin binary in the protocol
crate's Component, at the slice `command-protocol`; the socket listener and its threads
in the app crate's Component, at `spec@first-prototype@live-control`.

### The save file is RON, with a version field `##save-format`

Proposed by the owner in O1 as "probably json or ron" (`argument@first-prototype@arg-5`); the
session proposed RON, "held weakly" (`argument@first-prototype@arg-39`). Closed by O10's whole-table
agreement. Final state: approved; JSON lost. Shape: built, and recorded in
`design@rootpoint-model@save-format`. Harvest: a design head of the model crate's Component.

### Images are files beside the presentation now; packing into one file comes later `##image-storage`

Proposed by the session in round 1 as a question to the owner, among three shapes (see "Names"),
with shape 1 as its default (`argument@first-prototype@arg-40`).
The owner answered Q1 in O10: "for prototyping, shape 1 is fine. I think that shape 1 would stay the
preferred editing method long term, while shape 3 would be the final goal, so it is possible to pack
your presentation, carry just the file with you, and present it from any computer with a web browser
and internet connection using the webapp hosted from a static site." The session confirmed it in
round 2 (`argument@first-prototype@arg-47`), and the two goals it bears on were reworded on the
owner's word in O11 (`argument@first-prototype@arg-48`). Final state: approved. Shape: built, and
recorded in `design@rootpoint-model@image-storage`, attributed to the owner with
the scope and the hedge of the answer.

### The first crates are model, render, protocol and app `##crate-split`

Proposed by the session in round 1 (`argument@first-prototype@arg-41`), by
`criterion@first-prototype@c10-crate-rule`. Closed by O10's whole-table agreement. Final state:
approved. Shape: "Decided design" below. Harvest: a design head of rootpoint, at the harvest of
`spec@first-prototype@browser-build`, the slice that completes the split.

### The prototype holds the scope of round 1, and its wasm build is gated from the first slice `##prototype-scope`

Proposed by the session in round 1 (`argument@first-prototype@arg-42`); packing left out in round 2.
Closed by O10's whole-table agreement. Final state: approved. Shape: "Decided design" below.
Harvest: the scope leaves with this document; the rule that every crate the browser build links
builds for the wasm target, a reason respected at each crate's dependencies, is harvested with
`thread@first-prototype@crate-split` into rootpoint's design home at the milestone's closing commit
(see "Harvest").

### Fonts come only from the binary or the asset table, never from the system `##font-sources`

Proposed by the session in round 2, on the toolkit survey's finding that the browser has no system
fonts (`argument@first-prototype@arg-49`). Closed in O11: "I agree on the new font-sources and
asset-source too". Final state: approved; system fonts with fallback lost. Shape: built, and
recorded in `design@rootpoint-render@font-sources`. Harvest: a design head of the render crate's
Component;
T2 written in that Component's tripwires home.

### An asset entry is a path or embedded bytes, from version 1 of the format `##asset-source`

Proposed by the session in round 2 (`argument@first-prototype@arg-50`). Closed in O11, by the same
words, on the argument that adding `Embedded` later would force a
migration. That argument proved false at the review before the slice's merge; the owner then kept
the decision on the other argument, as D5. Final state: approved. Shape: built, and recorded in
`design@rootpoint-model@asset-source`.
Harvest: part of the head that `thread@first-prototype@inline-content-assets` gives, if the rules on
one decision per head join them; a head of its own otherwise.

## Arguments

### A slide is split recursively in two by draggable dividers `##arg-1`

Round: before round 1. Owner, O1. Split tree: "recursively, allow to split an area in two parts,
either vertically or horizontally, with the split position being movable by drag". Bears on
`thread@first-prototype@split-tree-arity`.

### A crate exists when it has its own tests, public API and reason to change `##arg-2`

Round: before round 1. Owner, O1. Crate rule: "a crate exists when it has its own tests, its own
public API, and a reason to change on its own". Bears on
`thread@first-prototype@crate-existence-rule`, `thread@first-prototype@crate-split`.

### The core logic runs headless `##arg-3`

Round: before round 1. Owner, O1. Headless core: "I want the core logic to be able to run in
headless mode". Bears on `thread@first-prototype@edit-operation-core`,
`thread@first-prototype@crate-split`.

### A text protocol drives the GUI for an agent, for demos and for headless tests `##arg-4`

Round: before round 1. Owner, O1. Text protocol: "allow usage by LLM (like you) live with the gui,
which is useful for testing and for making demos, as well as for headless testing. This might be
less useful is the full slide description serializes easily, but it is still quite useful IMO."
Bears on `thread@first-prototype@protocol-addressing`, `thread@first-prototype@protocol-transport`.

### Authoring uses no markup language; the save file is JSON or RON `##arg-5`

Round: before round 1. Owner, O1. No markup: tools "often based on code languages (usually markup
kind, which is the worst kind of language)"; save "in a text file for saving (probably json or
ron)". Bears on `thread@first-prototype@save-format`, `criterion@first-prototype@c5-no-markup`.

### The split tree is a known layout model and every layout it holds is valid `##arg-6`

Round: before round 1. Assistant, A1. Known layout model: "how tiling window managers and Blender's
editor areas work"; "every layout is valid by construction. That last property is what makes it more
convenient than free placement." Bears on `thread@first-prototype@split-tree-arity`,
`thread@first-prototype@split-any-node`.

### One operation API serves the GUI, the protocol and the tests, and gives undo `##arg-7`

Round: before round 1. Assistant, A1. One operation type: "the GUI, the protocol and the tests all
call the same API. Undo and redo then come almost free." Bears on
`thread@first-prototype@edit-operation-core`.

### The protocol serves what a readable file cannot `##arg-8`

Round: before round 1. Assistant, A1. Protocol beyond the file: "live operations, queries ... and
driving a running GUI. So it stays useful even with a readable RON file." Bears on
`thread@first-prototype@protocol-transport`.

### Rich text editing is the hardest piece; style per area avoids it `##arg-9`

Round: before round 1. Assistant, A1, difficulty 1. "Rich text editing is the hardest single piece";
"A prototype with styling per text area, not per span, avoids this". Bears on
`thread@first-prototype@text-style-per-area`, `thread@first-prototype@slide-renderer`.

### Binary splits cannot express equal columns or aligned rows `##arg-10`

Round: before round 1. Assistant, A1, difficulty 2. "Binary splits cannot express everything. Three
equal columns need nested ratios ... dragging the outer divider changes the inner proportions. Rows
that should line up across two columns cannot be tied together". Bears on
`thread@first-prototype@split-tree-arity`, `thread@first-prototype@linked-dividers`.

### Sizes are in slide units, not pixels `##arg-11`

Round: before round 1. Assistant, A1, difficulty 3. "Font sizes, padding and image scaling have to
be defined in slide units (for example a fixed 16:9 coordinate space), not in pixels." Bears on
`thread@first-prototype@slide-coordinate-space`.

### Without layout reuse every slide is set up by hand `##arg-12`

Round: before round 1. Assistant, A1, difficulty 4. "without some form of theme or \"duplicate
slide\" or layout reuse, every slide is set up by hand." Bears on
`thread@first-prototype@slide-templates`.

### Equations are costly and the renderer must not be tied to the GUI toolkit `##arg-13`

Round: before round 1. Assistant, A1, difficulty 5. Equations: "Rust has no small, mature TeX-math
renderer"; "the renderer should not be tied to the GUI toolkit, so that export stays possible."
Bears on `thread@first-prototype@slide-renderer`, `thread@first-prototype@prototype-scope`.

### A split can span areas, or two splits can be linked `##arg-14`

Round: before round 1. Owner, O2. "having a horizontal split span multiple vertical areas, or
\"linking\" two splits together so each follow the other one." Bears on
`thread@first-prototype@split-any-node`, `thread@first-prototype@linked-dividers`.

### A background, a footer and reusable layouts cover consistency `##arg-15`

Round: before round 1. Owner, O2. Consistency: "defining things like a common background is quite
easy. Having a reserved area on all slides (such as a footer) is easy too. Reusing predefined
layouts too." Bears on `thread@first-prototype@slide-templates`.

### Content stored apart and referenced by areas would be reusable `##arg-16`

Round: before round 1. Owner, O2. "instead of a piece of content being tied to an area ... an area
would just hold a reference to it, so they would be reusable ... it costs nearly nothing IMO." Bears
on `thread@first-prototype@content-by-reference`.

### The first prototype styles text per area `##arg-17`

Round: before round 1. Owner, O2. "first prototype can go with style per text area". Bears on
`thread@first-prototype@text-style-per-area`.

### A minimal equation renderer is a question for later `##arg-18`

Round: before round 1. Owner, O2. Equations: "how hard can it be to make a math renderer actually ?
It can be a minimal set too. Anyway, this is a question for later." Bears on
`thread@first-prototype@prototype-scope` (out).

### No shipped slide tool uses an editable split tree `##arg-19`

Round: before round 1. Prior-art subagent report. "I found no shipped slide tool ... that lays a
slide out as a recursive tree"; "Only Figma's beta grid ... and plain CSS grid have spanning. No
tool I found links splits across different branches"; "iced `pane_grid` ... already implement the
dragging interaction". Bears on `thread@first-prototype@split-tree-arity`,
`thread@first-prototype@split-any-node`, `thread@first-prototype@linked-dividers`,
`thread@first-prototype@gui-toolkit`.

### The split tree is a mechanism, not a goal `##arg-20`

Round: before round 1. Assistant, A2, Q1. "A goal states an outcome ... The tree is one way to reach
it, and the design discussion should be able to weigh it against n-ary splits or your linked
splits." Owner agreed (O5). Bears on `thread@first-prototype@split-tree-arity`.

### The crate rule is a design matter recorded on the owner's word `##arg-21`

Round: before round 1. Assistant, A2, Q3. The crate rule "describes how the project is built, not
what it is for, so it belongs in the design home as a head that rests on your words." Owner agreed
(O5). Bears on `thread@first-prototype@crate-existence-rule`.

### Math is needed later; video, image export and a browser build are wanted; collaboration is out `##arg-22`

Round: before round 1. Owner, O5. "I kinda needs math in the long term"; "animations ... only as a
video content piece probably"; "PDF exports are not fully ruled out (but it can be restricted to an
image export ...)"; "the main way to present is using the tool itself"; "Maybe having the project
build to webasm could be a plus"; "collaborative editing is not in scope." Bears on
`thread@first-prototype@prototype-scope`, `thread@first-prototype@slide-renderer` (export path),
`criterion@first-prototype@c4-browser-parity`.

### WebAssembly is a constraint to weigh in the design `##arg-23`

Round: before round 1. Assistant, A3. WebAssembly as "a constraint to weigh, for example keeping the
core free of anything that cannot compile to wasm." Bears on `thread@first-prototype@crate-split`,
`thread@first-prototype@gui-toolkit`.

### The browser build is a goal the owner may abandon `##arg-24`

Round: before round 1. Owner, O6. "webasm should be a goal too, at least initially. One that could
be abandonned later it if imposes too much constraints." Bears on
`criterion@first-prototype@c4-browser-parity`, `thread@first-prototype@gui-toolkit`,
`thread@first-prototype@prototype-scope`.

### The strict browser goal constrains the GUI more than a weaker one `##arg-25`

Round: before round 1. Assistant, A4, Q7. "\"the same editing operations … as in the native build\"
is strict. A weaker form ... constrains the GUI less." The owner approved the strict form in O8, and
the session confirmed in A4's follow-up: "word for word as you approved it, including the strict
form". Bears on `criterion@first-prototype@c4-browser-parity`,
`thread@first-prototype@protocol-transport`.

### A separate slide renderer shares one layout between editor, presenter and export `##arg-26`

Round: round 1 (grounding). Toolkit subagent report. (a) vs (b): (b) "text is laid out once in slide
units and drawn through an affine transform. The editor, the presenter and the PNG/PDF export share
one layout by construction"; cost "in-place editing must be built on the slide's own engine.
Parley's `PlainEditor` drives caret, selection and IME"; risks "Two text engines on one slide",
"Font fallback differs between platforms ... the browser has none", "Disable hinting", "Golden-image
tests should use the CPU backend only", "vello on the web needs WebGPU", "Version churn ... Pin
versions"; best combination 1 "eframe as host, plus a separate slide renderer: Parley with vello_cpu
or vello_hybrid"; "Masonry/xilem and gpui fail the browser requirement today, or need nightly Rust."
Facts of the survey, verbatim: "egui / eframe 0.37.0 (2026-10-15)"; "`request_repaint()` from any
thread (A)"; "parley | 0.12.0 (2026-10-09)", with "rich spans, `PlainEditor` with editable styles
(V)"; "vello_cpu | 0.3.0 (2026-10-02)"; "egui pins vello_cpu 0.2 while 0.3 is current (V), so the
tree will hold duplicate versions"; "Headless output is only as good as the toolkit's offscreen
path".
Bears on `thread@first-prototype@slide-renderer`, `thread@first-prototype@gui-toolkit`,
`thread@first-prototype@font-sources`, `thread@first-prototype@crate-split`.

### One layout has several binary trees; n-ary trees with merging are unique `##arg-27`

Round: round 1. Assistant, A5, `thread@first-prototype@split-tree-arity`. "in a binary tree, one
layout has several trees"; "in `V(1/3)[A, V(1/2)[B,C]]`, dragging A|B to 1/2 makes both B and C
shrink to 1/4"; "With merging, the n-ary tree is unique for each layout"; cost "operations take a
child index, and the merge rule must be tested". The i3 claim is labelled unchecked.

### Splitting a subtree gives every guillotine layout `##arg-28`

Round: round 1. Assistant, A5, `thread@first-prototype@split-any-node`. "What it gives: every
guillotine layout"; "What it rules out: ... a pinwheel of 4 rectangles around a 5th".

### Linked dividers need stable ids, which can be given now `##arg-29`

Round: round 1. Assistant, A5, `thread@first-prototype@linked-dividers`. "A link is a constraint
"divider X = divider Y", stated in slide coordinates. It needs stable divider ids, and a rule for
what dragging one does to the other"; "Give every node a stable id from the start ... so a link can
be added later without changing the file format." Also bears on
`thread@first-prototype@protocol-addressing`.

### Pixel units and free aspect ratio both fail C8 `##arg-30`

Round: round 1. Assistant, A5, `thread@first-prototype@slide-coordinate-space`. Rivals: "Pixel
units: a 1080p laptop and a 4K projector would then wrap text differently, which fails C8"; "Free
aspect ratio with reflow ... also fails C8."

### Content by reference costs deletion, editing, readability and protocol complexity `##arg-31`

Round: round 1. Assistant, A5, `thread@first-prototype@content-by-reference`. Costs: "Deletion:
removing an area leaves its content orphaned. You need garbage collection, or a library panel to
manage the orphans"; "Editing: changing a shared text changes every slide that uses it. The GUI must
mark shared content and offer "make unique""; "File readability ... works against C3"; "Protocol:
content ids exist next to area ids." "This is a fork ... choosing between them weighs cost against
reuse, which no goal ranks." Bears on `thread@first-prototype@inline-content-assets`.

### Inline content with a shared asset table covers reuse of heavy data `##arg-32`

Round: round 1. Assistant, A5, `thread@first-prototype@inline-content-assets`. "Only heavy data goes
in a table at the presentation level, referenced by id"; "Reusing text, such as a footer, goes
through `thread@first-prototype@slide-templates`."

### One template mechanism covers background, footer and layouts `##arg-33`

Round: round 1. Assistant, A5, `thread@first-prototype@slide-templates`. "One mechanism for all
three"; "The prototype offers \"duplicate slide\", which covers case 3 by copying. The model only
has to admit templates later: a slide gains an optional template id."

### Direct mutation by the GUI duplicates every edit in the protocol `##arg-34`

Round: round 1. Assistant, A5, `thread@first-prototype@edit-operation-core`. "Rival: the GUI mutates
the document directly. Then the protocol re-implements every edit, and C1 and C2 cost twice the code
and drift apart."

### The toolkit painter breaks scale invariance; our own renderer keeps it `##arg-35`

Round: round 1. Assistant, A5, `thread@first-prototype@slide-renderer`. "(a) ... line breaks can
move between window sizes (A). Headless output is only as good as the toolkit's offscreen path"; (b)
"share one layout by construction, so C8 holds by design"; "The headless PNG lets an agent see what
it built, which serves C2"; "Rival trap: a toolkit text field laid over a Parley-drawn area breaks
lines differently ... the caret drifts off the text"; "Unmeasured risk: the CPU cost of drawing a 4K
presenter frame (A)".

### Under our own renderer the host needs little, and eframe provides it `##arg-36`

Round: round 1. Assistant, A5, `thread@first-prototype@gui-toolkit`. "Given (b), the host needs:
panels, a slide list, a texture view, input forwarding, a browser build, and a way for another
thread to wake it"; iced "Under (b), its `pane_grid` does not help, because the dividers are inside
our slide image. Its releases are slow: 0.13 came out in 2024-09 and 0.14 in 2025-12"; iced "a
strong option under (a), with one text engine and screenshots in `iced_test` (V)"; "Slint: building
a free-form canvas from declarative items is the friction (A)"; "xilem and gpui: fail C4 today."
Depends on `thread@first-prototype@slide-renderer`.

### Paths shift after edits; stable ids do not `##arg-37`

Round: round 1. Assistant, A5, `thread@first-prototype@protocol-addressing`. "A path shifts after
every split or removal, so an agent's command 3 can address the wrong area after command 2 changed
the tree."

### Line commands are easier to type than JSON-RPC; the browser needs no protocol `##arg-38`

Round: round 1. Assistant, A5, `thread@first-prototype@protocol-transport`. "JSON-RPC: easier to
parse, harder for you or an agent to type by hand"; "In the browser build there is no protocol. C2
does not require one there, and C4 covers editing and presenter mode only."

### RON reads enums naturally and allows comments; held weakly `##arg-39`

Round: round 1. Assistant, A5, `thread@first-prototype@save-format`. RON "Enum variants read
naturally"; "allows trailing commas and comments, which helps hand editing (C3)"; RON: "Agents read
it fine"; JSON "Universal, and slightly better for tools other than ours"; "The file carries a
`version` field from the start"; "I hold it weakly".

### Which image storage the goal means is the owner's intent `##arg-40`

Round: round 1. Assistant, A5, `thread@first-prototype@image-storage`. Three shapes; "Which shape
your goal means is your intent, so I ask". Default shape 1 "the simplest to build and to read".

### Serialization stays in the model crate until it needs migrations `##arg-41`

Round: round 1. Assistant, A5, `thread@first-prototype@crate-split`. Table by
`criterion@first-prototype@c10-crate-rule`; "Serialization stays inside `model` until it needs
migrations of its own. That is the moment it gains its own reason to change."

### The prototype scope, with a wasm build checked from the first slice `##arg-42`

Round: round 1. Assistant, A5, `thread@first-prototype@prototype-scope`. In/out lists; "a wasm build
checked by the gates from the first slice, so C4's cost shows up early and not at the end". Serves
`criterion@first-prototype@c9-early-judgement`.

### Text files for editing, a packed file to carry and present from a static site `##arg-43`

Round: round 1, the owner's answer. Owner, O10, `thread@first-prototype@image-storage`. "shape 1
would stay the preferred editing method long term, while shape 3 would be the final goal, so it is
possible to pack your presentation, carry just the file with you, and present it from any computer
with a web browser and internet connection using the webapp hosted from a static site." Bears on
`thread@first-prototype@asset-source`, the goal rewordings.

### Precise control of the renderer is important for the project `##arg-44`

Round: round 1, the owner's answer. Owner, O10, `thread@first-prototype@slide-renderer`. "precise
control of the renderer is important for the project".

### N-ary splits are better `##arg-45`

Round: round 1, the owner's answer. Owner, O10, `thread@first-prototype@split-tree-arity`. "n-ary
split is better as you pointed out".

### Content by reference brings more trouble than convenience `##arg-46`

Round: round 1, the owner's answer. Owner, O10, `thread@first-prototype@content-by-reference` /
`thread@first-prototype@inline-content-assets`. "the reference thing I proposed brings more trouble
(especially ui side, for both dev and usage) than convenience."

### Packing is one more asset source, and a static site needs no server `##arg-47`

Round: round 2. Assistant, A6. "A static site needs no server"; "Packing is one more source for an
asset. It fits `thread@first-prototype@inline-content-assets` directly". Bears on
`thread@first-prototype@asset-source`, `thread@first-prototype@image-storage`.

### The save-file goal text no longer matched the owner's intent `##arg-48`

Round: round 2. Assistant, A6. "The goal text no longer says what you intend" for
`goal@rootpoint@readable-save-file`; rewordings of `goal@rootpoint@readable-save-file` and
`goal@rootpoint@runs-in-browser` proposed (Q2), approved O11.

### System fonts break C8 on any other computer `##arg-49`

Round: round 2. Assistant, A6, `thread@first-prototype@font-sources`. "the browser build has no
system fonts (V, in the blitz wasm notes). On native, system fonts also differ between machines ...
a packed file opened on another computer can lay out differently, which breaks C8"; "The rival,
system fonts with a fallback, fails C8 on any computer other than the author's".

### Asset entries carry Path or Embedded from format version 1 `##arg-50`

Round: round 2. Assistant, A6, `thread@first-prototype@asset-source`. "An asset entry is
`Path(relative)` or `Embedded(bytes)` from the first version of the format. The prototype writes
only `Path`. Adding `Embedded` later would otherwise force a migration of the file format."

### The premortem's causes and their observables `##arg-51`

Round: round 3. Assistant, A7, premortem. AC1 Parley in-place editing ("the caret sits on the drawn
glyph boundary"; dead key "é"), AC2 CPU rendering ("at most 33 ms per frame"; "3840×2160 ... at most
200 ms"; "The bounds are my proposal"), AC3 browser build, AC4 split-tree convenience ("rebuild 3
slides of a past presentation ... faster than with your tikz macros"), AC5 RON readability by an
agent; T1 "Merging same-axis splits removes a grouping you want" (response "an explicit group
node"), T2 "A needed glyph is missing from every available font". Kept by O12.

### A milestone of seven slices, not one spec `##arg-52`

Round: after convergence. Assistant, A8 (after O12, not yet answered). Milestone
`milestone@plans@first-prototype` in 7 slices over one spec: "the 4 crates and the browser build are
7 changes that can each fail on their own"; "slices 4 and 5 will raise design questions of their
own"; one spec "would be one PR of perhaps several thousand lines ... which is what C9 asks for."

## New names, in one place

The names below are the shapes the design needs. A slice's design audit may rename one, in place,
before the slice's code is written. Every one was new when this document was written; those of the
model crate now exist in the code.

```text
crates/rootpoint-model/   (crate rootpoint-model)
  Document, Slide, SlideId, Node, NodeId, Split, Axis { LeftToRight, TopToBottom },
  Side { Before, After }, Area,
  Content { Empty, Text(TextContent), Image(ImageContent) }, TextContent, TextStyle,
  FontRef { Builtin(String), Asset(AssetId) }, Rgba, HAlign { Left, Center, Right },
  VAlign { Top, Middle, Bottom }, ImageContent, ImageFit { Fit, Fill }, AssetId, Asset,
  AssetSource { Path(String), Embedded(Vec<u8>) }, AspectRatio { width, height },
  Rect { x, y, w, h }, area_rects, FORMAT_VERSION, save_ron, load_ron, LoadError,
  Op { AddSlide, RemoveSlide, RestoreSlide, MoveSlide, DuplicateSlide, Split, Remove,
       SetTree, MoveDivider, Swap, SetContent, AddAsset, RemoveAsset, RestoreAsset },
  OpError, apply
  (fields and shapes: "Decided design" and "Mapping tables" of its slice's spec)
crates/rootpoint-render/  (crate rootpoint-render)
  FontSet, RenderError, render_slide
crates/rootpoint-protocol/ (crate rootpoint-protocol)
  Command, ParseError, parse_command, Session, Reply, the view state
  binaries: rootpoint-headless, rootpoint-ctl
crates/rootpoint-app/     (crate rootpoint-app)
  binary: rootpoint
tools/xtask/src/main.rs   (existing file)
  the gate "wasm", pushed onto the list rust_project returns
```

## Decided design

The design of each thread built by one slice is in that slice's spec. The threads below cross
slices.

### Slide coordinates

Each presentation has one aspect ratio, 16:9 by default. A slide is a canvas in slide units,
1920×1080 for 16:9, so a font size of 48 means what it would on a 1080p screen. A divider's position
is a fraction of its parent split. Every display, the editor, the presenter and a headless PNG,
scales the whole slide uniformly and letterboxes it. The model crate owns the units and the area
geometry; the render crate draws in them.

Nearest rival: pixel units. Defeated by `criterion@first-prototype@c8-same-slide-everywhere`: a
1080p laptop and a 4K projector would wrap the same text differently. A free aspect ratio with
reflow fails `criterion@first-prototype@c8-same-slide-everywhere` the same way.

### The crate split

| crate | public API | own tests | own reason to change | depends on |
| --- | --- | --- | --- | --- |
| `rootpoint-model` | document types, `apply`, area geometry, RON load and save | operations, inverses, merging, round trip | the document model | serde, ron |
| `rootpoint-render` | slide to pixels, fonts | golden images | how slides look | model, parley, vello_cpu |
| `rootpoint-protocol` | command grammar, a session executing commands; the headless and control-client binaries | command scripts | the grammar | model, render |
| `rootpoint-app` | the eframe GUI, native and wasm, its control socket | thin; the others cover the logic | the UI | model, render, protocol, eframe |

Serialization stays inside the model crate until it needs migrations of its own, the moment it gains
its own reason to change. The model crate holds no dependency that fails to build for the wasm
target, and neither do the render and protocol crates, which the browser build links; the gates
check each from the slice that creates it, and the app crate from
`spec@first-prototype@browser-build`.

No rival to this split was argued in the discussion; the owner's rule,
`thread@first-prototype@crate-existence-rule`, is what each row is judged by.

### The crate rule

A crate exists when it has its own tests, its own public API and its own reason to change. This is
the owner's rule, stated in O1 as "something along the line of" those words, and recorded on the
owner's word (`thread@first-prototype@crate-existence-rule`). A part of a crate that gains all
three, such as serialization once it needs migrations, becomes a crate of its own; a crate that
loses one is merged.

### The prototype's scope

**In**: slides (add, remove, reorder, duplicate); splitting any node, removing an area, moving a
divider, swapping two areas; text with style per area; images with the fit and fill modes; undo;
save and load; presenter mode, full screen, with previous and next; the protocol, headless and
live; a headless PNG render; a wasm build checked by the gates from the first slice; a built-in
sample presentation, in every build, as a tutorial and demonstrator, per D3.

**The UI actions** of the native GUI, which `spec@first-prototype@editor-shell` builds and
`spec@first-prototype@browser-build` checks in the browser:

- in the slide list: add, remove, reorder by drag, and duplicate a slide;
- in the slide view: select an area or a split, drag a divider, split the selected node along
  either axis, remove it, swap two areas;
- in the side panel: set an area's content (empty, text, image) and its style or fit; add an image
  asset;
- in the menus: undo, redo, open, open the built-in sample, save, enter and leave presenter mode;
- in presenter mode: previous and next slide.

Each action issues a command of the protocol's `Session`: an editing command or a view command
(selection, presenter mode, current slide), so that every operation the GUI offers runs headless
too, per `goal@rootpoint@headless-core`.

**Out**: bullet lists, code, equations, video, templates, content sharing, linked dividers, export,
packing.

## Mapping tables

The mapping tables are in the code that builds their functions:
`path@rootpoint-model@src/ops.rs`, built by the slice `model-core` (each operation to its
inverse, in the doc comments of `Op`), and
`path@rootpoint-protocol@src/session.rs`, built by the slice `command-protocol` (each command to
its operation, in `Session::operation`).

## Losing alternatives

- **Binary splits**, lost to `thread@first-prototype@split-tree-arity`. The fact: in a binary tree
  one layout has several trees, so "remove B" and "drag the A|B divider" behave differently on
  layouts that look identical, and the file and the tree listing show a structure the user never
  chose; with merging, the n-ary tree is unique per layout (`argument@first-prototype@arg-27`).
- **Content stored apart and referenced by areas**, `thread@first-prototype@content-by-reference`,
  withdrawn by the owner; lost to `thread@first-prototype@inline-content-assets`. The fact: its
  costs in deletion (garbage collection, or a library panel for orphans), editing (marking shared
  content, and a "make unique" action), file readability and protocol ids, and the owner's weighing
  "more trouble ... than convenience" (`argument@first-prototype@arg-31`,
  `argument@first-prototype@arg-46`).
- **Pixel units, and a free aspect ratio with reflow**, lost to
  `thread@first-prototype@slide-coordinate-space`. The fact: both fail
  `criterion@first-prototype@c8-same-slide-everywhere` (`argument@first-prototype@arg-30`).
- **Mutation of the document by the GUI**, lost to `thread@first-prototype@edit-operation-core`. The
  fact: the protocol would re-implement every edit, so
  `criterion@first-prototype@c1-headless-editing` and `criterion@first-prototype@c2-command-driving`
  would cost twice the code and drift apart (`argument@first-prototype@arg-34`).
- **Drawing slides with the toolkit's own painter and text engine** (shape (a)), lost to
  `thread@first-prototype@slide-renderer`. The fact: the toolkit lays text out in pixels, so line
  breaks can move between window sizes (A); headless output is only as good as the toolkit's
  offscreen path, which weighs on `criterion@first-prototype@c2-command-driving`; and a toolkit text
  field over a Parley-drawn area breaks lines differently, so the caret drifts
  (`argument@first-prototype@arg-35`); and the owner's reason in O10.
- **iced, Slint, xilem and gpui as the host**, lost to `thread@first-prototype@gui-toolkit`. The
  facts: under our own renderer, iced's `pane_grid` does not help and its releases are slow (0.13 in
  2024-09, 0.14 in 2025-12); iced stays the host the discussion named if the renderer decision is
  reopened toward the toolkit's own painter, where its one text engine and its `iced_test`
  screenshots count; Slint makes a free-form canvas from declarative items (A); xilem and gpui fail
  `criterion@first-prototype@c4-browser-parity` today (`argument@first-prototype@arg-36`).
- **Addressing by paths**, lost to `thread@first-prototype@protocol-addressing`. The fact: a path
  shifts after a split or a removal, so a later command addresses the wrong area
  (`argument@first-prototype@arg-37`).
- **JSON-RPC**, lost to `thread@first-prototype@protocol-transport`. The fact: harder to type by
  hand for the owner or an agent (`argument@first-prototype@arg-38`).
- **JSON as the save format**, lost to `thread@first-prototype@save-format`, held weakly. The fact:
  RON writes enum variants as they read and allows comments and trailing commas, which helps hand
  editing. JSON's merit, kept for a later reversal of a decision held weakly: it is universal, and
  slightly better for tools other than ours (`argument@first-prototype@arg-39`).
- **System fonts with a fallback**, lost to `thread@first-prototype@font-sources`. The fact: they
  differ between machines and do not exist in the browser, so a slide lays out differently on
  another computer, which fails `criterion@first-prototype@c8-same-slide-everywhere`
  (`argument@first-prototype@arg-49`).

## Readings

The work implements no external specification. The facts it reads from outside tools are quoted from
the surveys in `argument@first-prototype@arg-19` and `argument@first-prototype@arg-26`, and from
round 1 in `argument@first-prototype@arg-35`, `argument@first-prototype@arg-36` and
`argument@first-prototype@arg-49`; the survey marks each (V) or (A) where it did. The versions the
slices name (parley 0.12.0, vello_cpu 0.3.0, eframe 0.37.0) were the latest releases on crates.io at
the review of this document, and the duplicate `vello_cpu` 0.2 comes through epaint, a dependency of
egui, not eframe's own manifest.

## Premortem

Each cause assumes the prototype was built and failed. Each verdict below is the owner's ruling in
O12, "All tripwires and AC can be kept, proceed", for AC1 to AC5 and T1 and T2, and in the owner's
message after the milestone was written, "D2 accepted; T3 and T4 too", for T3 and T4.

- **In-place text editing on Parley is poor.** Stresses `thread@first-prototype@slide-renderer` and
  `thread@first-prototype@gui-toolkit`. Becomes AC1, `acceptance@first-prototype@caret-on-glyph`.
- **CPU rendering is too slow.** Stresses `thread@first-prototype@slide-renderer`. Becomes AC2,
  `acceptance@first-prototype@render-fast-enough`.
- **The browser build does not hold.** Stresses `thread@first-prototype@gui-toolkit`,
  `thread@first-prototype@slide-renderer` and `thread@first-prototype@font-sources`. Becomes AC3,
  `acceptance@first-prototype@browser-build-holds`.
- **Split-tree editing is not convenient in practice.** Stresses
  `thread@first-prototype@split-tree-arity` and `thread@first-prototype@split-any-node`. Becomes
  AC4, `acceptance@first-prototype@split-editing-convenient`.
- **The RON file is not readable in practice.** Stresses `thread@first-prototype@save-format` and
  `thread@first-prototype@protocol-addressing`. Becomes AC5,
  `acceptance@first-prototype@file-editable-by-agent`.
- **T1. Merging same-axis splits removes a grouping the owner wants.** Stresses
  `thread@first-prototype@split-tree-arity`. Recorded on the owner's word, at the harvest of
  the slice `model-core`, in the model crate's Component's tripwires home. Fires when: in
  real use, the owner wants to move or resize a group of same-axis areas as one unit, and the merge
  has made that group impossible. Response: reopen the n-ary split decision, for example with an
  explicit group node. Re-entry: the first real talk given with the prototype.
- **T2. A needed glyph is missing from every available font.** Stresses
  `thread@first-prototype@font-sources`. Recorded on the owner's word, at the harvest of
  the slice `slide-rendering`, in the render crate's Component's tripwires home. Fires
  when: a real slide needs a glyph that no built-in font covers, and adding a font file to the
  presentation is not an acceptable fix for the owner. Response: reopen
  `thread@first-prototype@font-sources`. Re-entry: the first real talk given with the prototype.
- **T3. Rows must align across two columns.** Stresses the parked
  `thread@first-prototype@linked-dividers`. Fires when: a real slide needs rows aligned across two
  columns. Response: reopen linked dividers in a design session. Re-entry: the first real talk given
  with the prototype. Recorded as the trigger of a `deferred` issue, since a parked thread records
  no decision for a tripwire to guard.
- **T4. Repeated slide furniture costs too much by hand.** Stresses the parked
  `thread@first-prototype@slide-templates`. Fires when: preparing a real talk needs the same
  background, footer or layout on several slides beyond what "duplicate slide" gives. Response:
  reopen templates in a design session. Re-entry: the first real talk given with the prototype.
  Recorded as T3 is.

## Acceptance criteria

### AC1. The caret sits on the drawn text, and selection and dead keys work `##caret-on-glyph`

Guards `thread@first-prototype@slide-renderer`, and `thread@first-prototype@gui-toolkit`. Judged by
`spec@first-prototype@text-editing`. Fires when, in a text area at three window sizes, the caret
does not sit on the drawn glyph boundary, selection by drag or by shift+arrow fails, or a dead-key
character such as "é" cannot be typed. Response: reopen `thread@first-prototype@slide-renderer`.

### AC2. Divider drags and 4K slides render within their bounds `##render-fast-enough`

Guards `thread@first-prototype@slide-renderer`. Judged in two parts: the 4K bound by
the slice `slide-rendering`, where it did not fire, the drag bound by
`spec@first-prototype@editor-shell`. Fires
when, in a release build on the owner's machine, any frame of a divider drag in a 1600×900 editor
view takes more than 33 ms to redraw, or any render of one 3840×2160 presenter slide with text and
an image takes more than 200 ms. The owner kept these bounds in O12. Response: reopen
`thread@first-prototype@slide-renderer`, with GPU vello or vello_hybrid as the candidate.

### AC3. The browser build loads, edits and presents presentations as natively `##browser-build-holds`

Guards `thread@first-prototype@gui-toolkit`, `thread@first-prototype@slide-renderer` and
`thread@first-prototype@font-sources`. Judged by `spec@first-prototype@browser-build`. Fires when
the wasm build, served from a static directory, cannot open the built-in sample and a presentation
directory the user picks, run the editing operations, or show presenter mode, or lays text out
differently from a native PNG of the same slide. Response: where the failure is in eframe's web
support, reopen `thread@first-prototype@gui-toolkit`; where it is the cost of parity itself, bring
`goal@rootpoint@runs-in-browser` to the owner. The session that judges it states which, with the
evidence.

### AC4. Rebuilding three real slides is faster than with a general-purpose slide editor `##split-editing-convenient`

Guards `thread@first-prototype@split-tree-arity` and `thread@first-prototype@split-any-node`. Judged
after `spec@first-prototype@browser-build` lands, by the owner. Fires when the owner, rebuilding 3
slides of a past presentation in the prototype, judges it not faster than with a general-purpose
slide editor, the comparison `goal@rootpoint@free-slide-division` states, per D4; the owner is the
judge, per `goal@rootpoint@real-presentations`. Response: reopen both decisions.

### AC5. An agent edits a saved file by hand and it loads back `##file-editable-by-agent`

Guards `thread@first-prototype@save-format` and `thread@first-prototype@protocol-addressing`.
Judged by the slice `model-core`, where it did not fire. Fires when an agent session, given only a
saved file,
cannot change one text and one divider position by editing the file so that it loads back with
those changes. Response: reopen `thread@first-prototype@save-format`.

## Implementation sequence

Each slice's steps are in its spec, linked from this directory's `README.md` rows below.

1. the slice `model-core`: the model crate, its operations, its geometry and its RON
   format; the wasm gate. Judges AC5.
2. the slice `slide-rendering`: the render crate and a headless PNG. Judges AC2's 4K
   bound.
3. the slice `command-protocol`: the protocol crate and the headless binary.
4. `spec@first-prototype@editor-shell`: the app crate, native, with presenter mode. Judges AC2's
   drag bound.
5. `spec@first-prototype@text-editing`: editing text in place. Judges AC1.
6. `spec@first-prototype@live-control`: the control socket and its client.
7. `spec@first-prototype@browser-build`: the browser build. Judges AC3. The owner then judges AC4,
   and the closing commit of "Harvest" follows.

| slice spec |
| --- |
| [The native GUI and presenter mode](editor-shell.md) |
| [Editing text in place](text-editing.md) |
| [Driving the running GUI by text commands](live-control.md) |
| [The editor and presenter in a web browser](browser-build.md) |

## Order rationale

- the slice `model-core` before the slice `slide-rendering`: the renderer
  draws the rectangles and contents that the model defines.
- the slice `slide-rendering` before the slice `command-protocol`: the
  protocol's `render` command calls the renderer.
- the slice `command-protocol` before `spec@first-prototype@editor-shell`: the GUI
  applies the same operations, and its tests drive it through the protocol's session.
- `spec@first-prototype@editor-shell` before `spec@first-prototype@text-editing`: in-place editing
  needs a hosted slide view and input forwarding.
- `spec@first-prototype@text-editing` before `spec@first-prototype@live-control`: the control socket
  drives a GUI whose editing is complete, so a failure of the socket is not one of editing.
- `spec@first-prototype@live-control` before `spec@first-prototype@browser-build`: the browser build
  has no transport for the protocol, stdin or socket, though it links the session, so it comes
  last, on a GUI whose native features are all built.

## Defaults awaiting the owner

- **D1, ruled.** The work is a milestone of seven slices, not one spec. Bears on
  `thread@first-prototype@prototype-scope`. The owner ruled: "A milestone was the right call
  here."
- **D2, ruled.** Each crate is declared a Component of the manifest in the slice that creates it, at
  `path@rootpoint@crates/` under the crate's own name, and its goals are drafted under
  `skill@knowledge-architect-goal-setting` for the owner's ruling before that slice merges. Bears on
  `thread@first-prototype@crate-split`. The harvest rows of the slice specs name those Components'
  homes. The owner ruled: "D2 accepted".
- **T3 and T4, ruled**, in "Premortem": recorded. The owner ruled: "T3 and T4 too".
- **D3, ruled.** The default put to the owner: in the browser build, opening, saving and adding an
  image wait on packing, and the build edits only a built-in sample. Its rival: open a presentation
  and its images through the file picker and save by download. The owner ruled, in the owner's
  words: "D3: a good rival would be to open a full directory, rather than just file by file, isn't
  that possible ? TBH, I think that for the prototype, the web build can be "load only", without the
  ability to save. That reduces the work without losing too much on the feature side. The idea of a
  builtin sample presentation is good though. It can serve as a tutorial/demonstrator anywhere, not
  just on the web build, so I'd keep the idea as a permanent thing." So: the prototype's browser
  build opens a presentation directory, the RON file and its images, through the browser's directory
  picker, and the built-in sample; it edits them in memory and presents them; it does not save.
  Saving in the browser comes with packing. The built-in sample is a permanent feature of every
  build, native and web, as a tutorial and demonstrator. At the review before the merge, the owner
  confirmed, as Q6, the directory opening and the load-only browser build as written: "Q6 default
  accepted." Bears on
  `thread@first-prototype@prototype-scope` and `thread@first-prototype@image-storage`.
- **D4, ruled.** AC4 compares with a general-purpose slide editor, the comparison
  `goal@rootpoint@free-slide-division` states. The owner ruled: "D4: against a general purpose
  editor. Though I'd be the judge anyway. I'm not even sure where my tikz macros are, I have not
  been using them for a while." The tikz comparison is dropped. Bears
  on `thread@first-prototype@split-tree-arity` and `thread@first-prototype@split-any-node`.
- **D5, ruled.** `thread@first-prototype@asset-source` keeps `Embedded` in the format's
  first version, on the argument that the round-trip tests then cover it before packing writes it;
  the argument it was approved on, a migration, proved false. Bears on
  `design@rootpoint-model@asset-source`. The owner ruled: "D5 keep".
- **Q4, ruled.** A rewording of `goal@rootpoint@minimal-content-set`, whose "every feature the
  editor has" excluded, read literally, presenter mode, saving and the protocol, which other goals
  require. The owner ruled: "Q4 accepted, go ahead". The goal is reworded under
  `skill@knowledge-architect-goal-setting`.

## Harvest

- **This document** leaves in a closing commit of its own, on a branch after
  `spec@first-prototype@browser-build` has merged and the owner has judged AC4. That commit
  reports on AC4, harvests `thread@first-prototype@crate-split` and the wasm rule of
  `thread@first-prototype@prototype-scope` into rootpoint's design home, and deletes this
  directory.
- **Each slice** harvests by the row of its spec.
- **T3 and T4**, recorded on the owner's word, are written at the harvest of
  the slice `model-core` as the triggers of two `deferred` issue entries in the model
  crate's Component, as `skill@knowledge-architect-decision-recording@ruled-tripwires` says of a
  parked thread's tripwire, each naming in words the parked thread it watches.

## Later consequences

- **Packing and unpacking** add `AssetSource::Embedded` writers and the browser's open of a packed
  file, completing `goal@rootpoint@readable-save-file` and `goal@rootpoint@runs-in-browser`.
- **More content kinds** (bullet lists, code, equations, video) each add a variant of `Content`, its
  options, its drawing and its editor, per `goal@rootpoint@minimal-content-set`.
- **Templates and linked dividers** are re-entered at the first real talk, by T3 and T4.
- **Rich text with per-span styles** replaces `TextStyle` per area with styled spans, on the
  in-place editor of the slice `spec@first-prototype@text-editing`.
- **An image export** reuses `render_slide`.
- **Publishing to crates.io**, which the owner named in O5 and `goal@rootpoint@real-presentations`
  admits as "may be published later", gives each published crate a `CRATES-IO.md` page, as
  `skill@knowledge-architect-setup@rust-project` proposes.
