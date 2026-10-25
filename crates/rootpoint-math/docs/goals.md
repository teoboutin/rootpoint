# Goals of rootpoint-math

What rootpoint-math is for, and what would show it achieved: one level-two heading per goal. A goal
is met or unmet. It stays here while it is met, and it leaves only on the owner's word. How
rootpoint-math is built is in `register@rootpoint-math@design`.

## A formula is written in the LaTeX authors and agents already know `##latex-text`

A formula is stored in the saved file and passed in commands as LaTeX, so a person or an agent who
knows LaTeX math reads and writes it with nothing new to learn. It refines
`goal@rootpoint@readable-save-file` and `goal@rootpoint@text-command-protocol`. It is met while the
LaTeX of the owner's talks reads, and writes back, as the formula it was.

## A formula looks as a reader of TeX expects `##tex-quality`

A formula is set with the spacing, the scripts, the fractions and the delimiters a reader of TeX
expects, in the built-in math font. It refines `goal@rootpoint@real-presentations`. It is met while
the owner judges the formulas of the owner's talks acceptable beside TeX's rendering of them.

## Every atom and every cell of a formula can carry the caret `##editable-in-place`

The layout of a formula tells where each atom and each cell is drawn, so the window edits a formula
in place, with no source to type. It refines `goal@rootpoint@no-language-no-build`. It is met while
every caret position of a formula is drawn on its atom or its cell.

## A formula looks the same at every size `##same-at-every-size`

A formula laid out at one size and scaled is the formula laid out at the scaled size, so it is the
same slide in every window, and an equation fits its area exactly. It refines
`goal@rootpoint-render@same-slide-everywhere`. It is met while scaling a layout and laying it out at
the scaled size agree.

## The math crate builds for the browser `##browser-buildable`

The math crate is part of the browser build, so it uses nothing that exists only on a desktop and
reads no file. It refines `goal@rootpoint@runs-in-browser`. It is met while the gate `wasm` builds
this crate for the browser's target.
