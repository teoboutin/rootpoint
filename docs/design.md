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
