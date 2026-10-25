# Rejected designs of rootpoint-render

The designs not kept for rootpoint-render, and why: one level-three heading per design, each cited
by the design head whose argument covers it, in `register@rootpoint-render@design`. An entry
describes a design that is not built.

### Drawing slides with a GUI toolkit's own painter and text engine `##toolkit-painter`

`live`. The toolkit survey of the first prototype found that a toolkit lays text out at pixel
sizes, so uniform scaling lays it out again at each size and hinting and rounding can move line
breaks between window sizes (assumed by the survey, not measured), and that a headless image is
only as good as the toolkit's offscreen path. A toolkit text field over a slide drawn by another
engine would break lines differently and let the caret drift; this renderer's caret stands on the
drawn glyphs, which `path@rootpoint-app@tests/editing.rs` checks. Its two speed bounds,
`tripwire@rootpoint-render@render-within-bounds`, held when measured, release build,
on the owner's Intel Core Ultra 7 165H: a frame of a divider drag in a 1600×900 view stayed within
33 ms, as `design@rootpoint-app@gui-toolkit` records, and a 3840×2160 slide of text and an image at most 45.3 ms against 200 ms
in its measured run, while one cold first render, under a load average of 35 from parallel work,
took 199.6 ms, 0.4 ms within the bound.

### System fonts, with a fallback for missing ones `##system-fonts`

`live`. The toolkit survey found that the browser has no system fonts, read in the notes of blitz
on building for wasm; and system fonts differ between machines, so a presentation opened on another
computer lays out differently.

### STIX Two Math built in as a fallback for the characters the math font lacks `##stix-math-fallback`

`live`. STIX Two Math 2.13, measured in `path@rootpoint@docs/grounding/math-typesetting.md`,
sections 2 and 4, covers every command of unicode-math's 2448, and so the 44 of KaTeX that
Libertinus Math lacks, for 838,652 bytes in every build, the browser build included: about 0.8 MB
for 44 rare symbols, whose absence the owner judged not much of a problem, by
`design@rootpoint-render@builtin-libertinus-math`.
