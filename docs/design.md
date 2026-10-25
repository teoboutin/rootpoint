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
serialization stays inside it until it needs migrations of its own;
`path@rootpoint@crates/rootpoint-render/`, whose reason to change is how slides look;
`path@rootpoint@crates/rootpoint-protocol/`, whose reason to change is the command grammar; and
`path@rootpoint@crates/rootpoint-app/`, whose reason to change is the user interface.

On the owner's word: the owner set the rule for the project, stated as an approximation of these
three conditions, and ruled that it is a decision of the design rather than a goal. 

### No crate but the app depends on a GUI toolkit `##gui-only-in-app`

`rootpoint-render` depends on `rootpoint-model`; `rootpoint-protocol` on both; `rootpoint-app` on
the three and on eframe, the one GUI toolkit of the workspace. So the model, the renderer and the
protocol's session run with no window, in a test or a script, which `goal@rootpoint@headless-core`
asks of every operation the GUI offers. The rival, a model, a renderer or a session that links the
toolkit, would need a window, or the toolkit's headless mode, for every test of the core. It is a
reason respected at every crate's manifest. The crates are the four that
`design@rootpoint@crate-existence-rule` gives.

### Every crate the browser build links builds for the wasm target, checked by the gate `wasm` `##wasm-from-the-start`

The gate `wasm` of `path@xtask@src/main.rs` builds the model, render, protocol and app crates for
`wasm32-unknown-unknown`, the app's binary included, the browser build's entry point. So a
dependency that cannot build for the browser fails the gates when it is added to a crate the gate
builds, not when the browser build is made, and the cost of `goal@rootpoint@runs-in-browser` shows
at each change rather than at the end. It is a
reason respected at every dependency a crate adds. The gate builds; it runs nothing, so a `std`
function that compiles for the wasm target and fails when called, as `std::fs` does, passes it,
`design@rootpoint-protocol@session-files`; the browser tests,
`design@xtask@browser-tests-gated`, run what the gate cannot.

## Compatibility

### Nothing is kept compatible with an earlier version while the project is in initial development `##no-compat-in-initial-dev`

A change of the save format, of the protocol's grammar or of a crate's public API may break the
files, the scripts and the callers of the version before it, and no migration and no deprecated
form is written for them. A saved file names its format's version first, so a file of another
version is refused rather than misread, `design@rootpoint-model@save-format`. This holds until the
owner rules that the project is in real use, when the owner starts using it for actual
presentations, `goal@rootpoint@real-presentations`.

On the owner's word: no presentation exists that a change could break, the owner will not use the
project for real while it is in initial development, and backward compatibility is needed only from
the owner's ruling. The rival, a migration for each change of format,
`rejected@rootpoint-model@format-migration`, costs a loader and a test for each version with no file to
read.

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
an explicit grant of patents. The rival, a copyleft licence such as GPL-3.0 or MPL-2.0, keeps derived
versions open, and keeps companies and permissively licensed projects from building on rootpoint.

On the owner's word: the owner plans to make the project open source, ultimately and probably
soon, wants it as widely available as possible, for everyone and everywhere, and chose the dual
licence MIT OR Apache-2.0, which the session recommended for that purpose.
