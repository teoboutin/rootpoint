# Design of rootpoint

How rootpoint is built, and why: one level-three heading per decision, under level-two subjects.
Design
that is decided and not built is in a plan document, not here. What rootpoint is for is in
`register@rootpoint@goal`.

## Crates

### A crate exists when it has its own tests, its own public API and its own reason to change `##crate-existence-rule`

The workspace is split into crates by this rule: a part of the code becomes a crate of its own
when it has tests of its own, a public API others use, and a reason to change that the rest does
not share; a crate that loses one of the three is merged back. Today the rule gives
`path@rootpoint@crates/rootpoint-model/`, whose reason to change is the document model, and whose
serialization stays inside it until it needs migrations of its own, but for a formula's text, which
the math crate reads and writes;
`path@rootpoint@crates/rootpoint-render/`, whose reason to change is how slides look;
`path@rootpoint@crates/rootpoint-math/`, whose reason to change is how a formula is written and laid
out from a math font;
`path@rootpoint@crates/rootpoint-protocol/`, whose reason to change is the command grammar; and
`path@rootpoint@crates/rootpoint-app/`, whose reason to change is the user interface.

On the owner's word: the owner set the rule for the project, stated as an approximation of these
three conditions, and ruled that it is a decision of the design rather than a goal. 

### No crate but the app depends on a GUI toolkit `##gui-only-in-app`

`rootpoint-math` depends on no crate of the workspace; `rootpoint-model` on `rootpoint-math`;
`rootpoint-render` on the model and the math crate; `rootpoint-protocol` on those three; and
`rootpoint-app` on those four and on eframe, the one GUI toolkit of the workspace. So the model, the
renderer and the protocol's session run with no window, in a test or a script, which
`goal@rootpoint@headless-core` asks of every operation the GUI offers. The rival, a model, a
renderer or a session that links the toolkit, would need a window, or the toolkit's headless mode,
for every test of the core. It is a reason respected at every crate's manifest. The crates are the
five that `design@rootpoint@crate-existence-rule` gives.

### Every crate the browser build links builds for the wasm target, checked by the gate `wasm` `##wasm-from-the-start`

The gate `wasm` of `path@xtask@src/main.rs` builds the math, model, render, protocol and app crates for
`wasm32-unknown-unknown`, the app's binary included, the browser build's entry point. So a
dependency that cannot build for the browser fails the gates when it is added to a crate the gate
builds, not when the browser build is made, and the cost of `goal@rootpoint@runs-in-browser` shows
at each change rather than at the end. It is a
reason respected at every dependency a crate adds. The gate builds; it runs nothing, so a `std`
function that compiles for the wasm target and fails when called, as `std::fs` does, passes it,
`design@rootpoint-protocol@session-files`; the browser tests,
`design@xtask@browser-tests-gated`, run what the gate cannot.

## Compatibility

### A saved presentation of any version from 7 on loads in every later version, a breaking change of the format migrating the version before `##saved-presentations-stay-loadable`

A presentation saved as a workspace's `presentation.ron` or as a pack loads in every later version
of rootpoint. A breaking change of the save format, one after which a file of the version before
would be refused or would load as another presentation, raises `FORMAT_VERSION`, of
`path@rootpoint-model@src/file.rs`, and adds a step that migrates a file of the version before to
the new one, run by `load_ron` before it reads the file. Version 7, `OLDEST_VERSION`, is the oldest
that loads, and every other version is refused, as `design@rootpoint-model@save-format` says. A
pack carries the same RON text and no version of its own,
`design@rootpoint-protocol@pack-container`, so it migrates the same way. How a step is written is
open, `issue@rootpoint-model@migration-shape`; the cost is a step and frozen files for each breaking
change.

A frozen file of every version from 7 on, in `path@rootpoint-model@tests/data/formats/`, loads in
the test `a_frozen_file_of_every_version_loads` of `path@rootpoint-model@tests/formats.rs`, saves as
its committed text, and the newest holds every part of the format, its formulas every kind of atom
the math crate's parser builds and a grid of rows; a frozen pack of every version, in
`path@rootpoint-protocol@tests/data/formats/`, opens to the same presentation in
`path@rootpoint-protocol@tests/formats.rs`. So a breaking change with no step fails the gates. The
rival, refusing a file of every earlier version, is `rejected@rootpoint@refuse-earlier-versions`.

It serves `goal@rootpoint@real-presentations`: a file the owner prepares a talk in must still open
after an update. The compatibility is the file's: it promises nothing of the look of a slide, and a
change of the renderer may draw a loaded presentation differently.

On the owner's word: with the project holding most of the features the owner wanted, the owner
proposed declaring the compatibility from then on, every later breaking change of the file format
coming with a migration to the new format; and ruled that, for now, it holds for saved presentations, packed and unpacked,
and not for the protocol or the crates' API.

### The protocol's grammar and the crates' public API keep no form of an earlier version until the owner rules otherwise `##grammar-and-api-stay-free`

A change of the protocol's grammar or of a crate's public API may break the scripts and the
callers of the version before it, and no deprecated form is written for them. When that ends is
`issue@rootpoint@grammar-and-api-compatibility`.

On the owner's word: for now, the compatibility the owner ruled covers saved presentations and not
the protocol nor the crates' API, `design@rootpoint@saved-presentations-stay-loadable`; and the
crates are not meant to be pulled in by another program, not yet at least. The crates are not
published, `issue@rootpoint@publish-to-crates-io`.

## Displays

### Every display scales the whole slide uniformly to the largest size that fits, and letterboxes the rest `##uniform-scaling`

The editor's slide view, presenter mode and a headless image all show a slide at one scale on
both axes, the largest at which the whole slide fits the view, centred, the rest black.
`slide_to_pixels` of the render crate gives that scale and offset for an image of a given size;
`render_slide` draws through it, as `design@rootpoint-render@slide-renderer` says, and the app places its marks and tests the pointer through it,
so every display of one slide is the same slide, at any size, which
`goal@rootpoint-render@same-slide-everywhere` asks. It applies the model's
`design@rootpoint-model@slide-units` to every display: a scale per axis, or a slide that fills the
window, would show another slide in each window, as `rejected@rootpoint-model@reflowing-slides`
says of a slide reflowed to its window.

## Documentation

### While its interface is to be reworked, the README previews the editor by a link to its published build, not by pictures of it `##readme-preview-link`

The root README, `path@rootpoint@README.md`, holds no picture of the editor. Its section "Try it"
links the browser build that `path@rootpoint@.github/workflows/pages.yml` publishes from main, so
the preview a reader opens is the editor as it is built, which `goal@rootpoint@runs-in-browser`
makes possible with no install. A picture taken before the planned rework of the interface would
show an interface that the rework replaces.

On the owner's word: the owner keeps pictures out of the README for now, since the owner plans to
rework the interface and does not want to carry outdated pictures, and wants the published build to
serve as the preview.

## Licence

### The project is licensed MIT OR Apache-2.0, and builds in nothing that binds it to a copyleft licence `##permissive-licence`

Every crate of the workspace declares `license = "MIT OR Apache-2.0"`, and
`path@rootpoint@LICENSE-MIT` and `path@rootpoint@LICENSE-APACHE` sit at the root. A dependency a
crate adds, and a font or a data file built into a binary, carries a licence that lets the whole
be distributed under either; one under a licence with conditions of its own, as a font under the
SIL Open Font License, keeps its licence text beside the file, as the built-in fonts' `OFL.txt`
does. So a table under the GPL, as LyX's `path@elsewhere@lib/symbols`, is not built in. It is a
reason respected at every dependency added and every file embedded.

The dependencies already meet it: `cargo metadata` over the 513 packages the workspace takes from a
registry finds no licence that forces a copyleft one, three offering a GPL or LGPL branch only as an
alternative to MIT or Apache-2.0, as measured in `path@rootpoint@docs/grounding/math-typesetting.md`,
section 4. MIT lets nearly any other licence combine with the code, GPLv2 included; Apache-2.0 adds
an explicit grant of patents. The rival, a copyleft licence, keeps derived versions open at a cost to
who builds on rootpoint: GPL-3.0 binds every program that links the code to its own terms, which
companies and permissively licensed projects avoid, and MPL-2.0 binds every changed file of the
code itself to stay under it.

On the owner's word: the owner plans to make the project open source, ultimately and probably
soon, and wants it as widely available as possible, for everyone and everywhere.
