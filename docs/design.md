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

### The crates depend in one direction, and only the app depends on a GUI toolkit `##crate-dependencies`

`rootpoint-render` depends on `rootpoint-model`; `rootpoint-protocol` on both; `rootpoint-app` on
the three and on eframe. No crate depends on one after it in that order, and no crate but the app
links a GUI toolkit, so the model, the renderer and the protocol's session run with no window, a
test or a script, which `goal@rootpoint@headless-core` asks of every operation the GUI offers; and
a crate's tests need no crate above it. The crates are the four that
`design@rootpoint@crate-existence-rule` gives.

### Every crate the browser build links builds for the wasm target, checked by a gate from the crate's first commit `##wasm-from-the-start`

The gate `wasm` of `path@xtask@src/main.rs` builds the model, render, protocol and app crates for
`wasm32-unknown-unknown`, the app's binary included, the browser build's entry point; a crate the
browser build links joins the gate in the change that creates it. So a dependency that cannot
build for the browser fails the gates when it is added, not when the browser build is made, and
the cost of `goal@rootpoint@runs-in-browser` shows at each change rather than at the end. It is a
reason respected at every dependency a crate adds. The gate builds; it runs nothing, so a `std`
function that compiles for the wasm target and fails when called, as `std::fs` does, passes it,
`design@rootpoint-protocol@session-files`; the browser tests,
`design@xtask@browser-tests-gated`, run what the gate cannot.

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
