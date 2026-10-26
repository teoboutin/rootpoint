# Rejected designs of rootpoint-math

The designs not kept for rootpoint-math, and why: one level-three heading per design, each cited by
the design head whose argument covers it, in `register@rootpoint-math@design`.

### A fork of KenyC/ReX or latex-rust for the layout `##rex-fork`

`live`. Both lay out from the MATH table into a tree of boxes, but no box names the atom or the
cell it came from, as the survey read in their sources,
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2, so a fork would add that identity to a parser and node types of their own.

### mathtex for the layout `##mathtex-layout`

`live`. The one typesetter of the survey whose boxes carry ids and source spans, it is XeTeX
translated by c2rust: unsafe code, and a TeX format file to load,
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### typst's math layout `##typst-math-layout`

`live`. Its math module is private, and its output a frame reached through a typst compilation,
which the survey read in the API and did not test, with 240 packages on the browser's target,
measured in
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### RaTeX for the layout `##ratex-typesetter`

`live`. It lays out from KaTeX's metrics compiled in, with no MATH table, so it is tied to the
KaTeX fonts, and its large delimiters come out as SVG paths, as read in
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### unicode-math's table of commands `##unicode-math-table`

`live`. The largest table with a class for each of its 2448 commands, it is under the LaTeX Project
Public License 1.3c, whose condition on a modified file's name would bind the binary that embeds a
table derived from it, `path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### KaTeX's table of commands `##katex-table`

`live`. Under MIT, it holds 490 math commands of a name, about a fifth of unicode-math's 2448, as
counted in `path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### Names taken from unicode-math's column first, else from `<latex>` `##unicode-math-names-first`

`live`. Read in `unicode.xml` at efed20d: that rule names α `\mupalpha`, ε `\mupvarepsilon`, ≤
`\leq` alone and … `\unicodeellipsis`, so `\alpha`, `\epsilon`, `\le` and `\dots` would be unknown
commands.

### The LaTeX string as the model of a formula, edited as source `##latex-string-model`

`live`. `Formula` is a public type, which `lay_out` takes; a string in its place makes LaTeX the way
to write math in the window, which `goal@rootpoint@no-language-no-build` rules out, and gives an
editor no cell to place a caret in.

### A formula's tree written out in the file, as atoms of RON `##written-out-tree`

`live`. It would replace the text grammar of `parse` and `to_latex`, and it is about ten times
longer than LaTeX for `\frac{a}{b}`, a text nobody writes by hand, against
`goal@rootpoint-math@latex-text`.

### Right visiting every position, a numerator then its denominator `##every-position-moves`

`live`. It is not LyX's caret, which the window copies: LyX's `InsetMathFrac::idxForward` and
`InsetMathScript::idxForward` return false, so its Right leaves a fraction at its numerator's end and
a script at its nucleus's end, as section 1 of `path@rootpoint@docs/grounding/math-typesetting.md`
reads LyX's source; an author who knows LyX's keys would meet another caret.
