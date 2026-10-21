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
`path@rootpoint@crates/rootpoint-model/`, whose reason to change is the
document model, and `path@rootpoint@crates/rootpoint-render/`, whose reason to change is how slides
look; its
serialization stays inside it until it needs migrations of its own.

On the owner's word: the owner set the rule for the project, stated as an approximation of these
three conditions, and ruled that it is a decision of the design rather than a goal. 