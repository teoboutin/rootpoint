# Rejected designs of rootpoint-render

The designs not kept for rootpoint-render, and why: one level-three heading per design, each cited
by the design head whose argument covers it, in `register@rootpoint-render@design`. An entry
describes a design that is not built.

### Drawing slides with a GUI toolkit's own painter and text engine `##toolkit-painter`

`live`. The toolkit survey of the first prototype found that a toolkit lays text out at pixel
sizes, so uniform scaling lays it out again at each size and hinting and rounding can move line
breaks between window sizes (assumed by the survey, not measured), and that a headless image is
only as good as the toolkit's offscreen path. A toolkit text field over a slide drawn by another
engine would break lines differently and let the caret drift. Kept open by the doubt that this
renderer edits text in place with the caret on the drawn glyphs,
redraws a divider drag in 33 ms at 1600×900 and draws a 3840×2160 slide in 200 ms, which the
milestone `milestone@plans@first-prototype` checks.

### System fonts, with a fallback for missing ones `##system-fonts`

`live`. The toolkit survey found that the browser has no system fonts, read in the notes of blitz
on building for wasm; and system fonts differ between machines, so a presentation opened on another
computer lays out differently.
