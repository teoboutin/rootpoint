# Design of rootpoint-math

How rootpoint-math is built, and why: one level-three heading per decision, under level-two
subjects. Design that is decided and not built is in a plan document, not here. What
rootpoint-math is for is in `register@rootpoint-math@goal`.

## Formulas

### A formula is held as a tree of atoms, in a grid of cells `##formula-tree`

`Formula` is a grid of rows of cells, each cell a list of `Atom`s, and an atom with cells, as a
fraction, holds lists of its own. An editor changes a formula cell by cell, and a layout gives each
atom and each cell its box, `goal@rootpoint-math@editable-in-place`. The rival, the LaTeX string as
the model, edited as source, makes LaTeX the way to write math in the window, which
`goal@rootpoint@no-language-no-build` rules out, `rejected@rootpoint-math@latex-string-model`.

### A formula's text is a closed subset of LaTeX `##math-storage-form`

`Formula::parse` reads a formula from LaTeX and `Formula::to_latex` writes it back, over the closed
subset of `path@rootpoint-math@src/latex.rs`: symbols and the commands of the table, fractions,
radicals, scripts, `\left…\right`, twelve accents, five font changes, `\text`, the matrix, pmatrix,
bmatrix and cases environments, six spaces, groups, and the 32 operator names LaTeX defines in
`path@elsewhere@base/ltmath.dtx`; `'` reads as LaTeX reads it, a prime in a superscript. Its
environments set a matrix or a case analysis with no specification of their columns, where the
rival, LaTeX's `array`, adds the grammar of one. On the owner's word, the owner thinks matrix,
pmatrix and cases cover the owner's talks; bmatrix is the bracketed form of the first. A text
`to_latex` wrote reads back equal, which the proptest of `path@rootpoint-math@tests/latex.rs`
checks, and so does every formula that keeps the invariants the doc comment of `Formula` states.
A text that does not parse is refused, for the reasons the doc comment of `Formula::parse` lists,
among them a nesting of atoms deeper than `MAX_DEPTH`: reading and laying out recurse once a
level, and a stack overflow aborts the program. An unknown command is an atom of its own that takes
no argument, so the groups after it stay groups and write back as they were read; it is drawn red
and `draw_formula` returns it to its caller, as it returns a missing glyph, so it is no error of the
formula.

A person and an agent already read and write LaTeX, `goal@rootpoint-math@latex-text`, where a tree
of atoms written out, the rival, `rejected@rootpoint-math@written-out-tree`, is about ten times
longer for `\frac{a}{b}` and written by nobody.

### A formula's rows are aligned at `&`, right then left in turn, as amsmath's `align` sets them `##equation-rows`

A formula's grid is rows separated by `\\`, each row's cells separated by `&`; `lay_out` sets the
cells of a row right-aligned then left-aligned in turn, as amsmath's `align` sets them, so the
relation after each `&` stands under the one above, and a formula with no `&` as one column, each
row centred. The rival, rows centred with no alignment, as amsmath's `gather`, cannot stand the
relations of a derivation under one another.

On the owner's word: the owner's talks need aligned rows often.

### A matrix is an atom of its own `##matrix-atom`

`Atom::Matrix` holds a grid of cells and its environment, written `\begin{…}…\end{…}`, which sets
its delimiters. Its cells are in text style, as LaTeX's arrays set them, and it is centred on the
axis. A matrix stands inside a formula, between other atoms, so it is an atom with a grid of its
own: the rival, the formula's own grid, holds the formula's rows and cannot stand inside
parentheses beside `R(\theta) =`.

On the owner's word: the owner's talks need matrices occasionally.

## Editing

### Right and Left enter an atom by one cell and leave it at that cell's end, as LyX's caret does `##lyx-caret-moves`

`right` enters an atom at the start of its first cell, a root's index where it has one, and at a
cell's end goes to a matrix's next column in the same row, or from a root's index to its body, and
else out of the atom; `left` is the mirror, entering at the end of an atom's last cell, a root's body.
In the formula's grid both keep to the caret's row and give no position past its first or last cell.
`vertical_move` moves between a fraction's numerator and denominator, between the rows of a matrix,
to the nearest column of a shorter row, and of the formula, and from a script's nucleus to its script
only at the nucleus's end, a script going back to that end; `next_cell` gives the next cell, as Tab
does, and `leave_cell` the position after the atom whose cell holds the caret. So a denominator or a
script is reached by `left`, `vertical_move` or `next_cell`, and every position by one of them, which
`goal@rootpoint-math@editable-in-place` asks; `positions` gives them all.

LyX's source, read in section 1 of `path@rootpoint@docs/grounding/math-typesetting.md`: Right is
`Cursor::mathForward`, which enters by `idxFirst` and, at a cell's end, moves by `idxForward` or
leaves the inset; `InsetMathFrac::idxForward` and `InsetMathScript::idxForward` return false, a
grid's moves to its next column, a root's from its index to its body; `InsetMathScript::idxUpDown`
reaches a script from the nucleus's end, and from its start for an operator with displayed limits,
which the moves here do not, since they do not depend on the style a formula is laid out in. The
window copies LyX's editing of math, `design@rootpoint-app@lyx-formula-keys`. The rival that visits
every position by Right is `rejected@rootpoint-math@every-position-moves`.

## Layout

### A formula is laid out by the crate's own code from the font's MATH table, a box per atom and per cell `##math-layout-engine`

`lay_out` sets a formula by the rules of appendix G of The TeXbook, with the font's MATH constants
in place of TeX's font parameters, read with ttf-parser 0.25.1 from the font file's bytes: the
four styles and their cramped forms; the space between two atoms by their classes, as `math_spacing`
of tex.web §764 gives it; a bin turned into an ord as §728 and §729 turn it; the font's `ssty`
glyphs in script styles, chosen before they are measured; large operators centred on the axis,
with limits in display style but for the integrals; delimiters, radicals and stretched glyphs from
the font's variants and assemblies; a letter at the code point of its font change, in the
Mathematical Alphanumeric Symbols, their holes filled from the Letterlike Symbols. It gives each
atom and each cell a box, tied to its `CellPath`, beside the glyphs and the rules to draw, which the
renderer draws.

An editor places a caret in a numerator or a script, so the layout says where each atom and cell
is drawn, `goal@rootpoint-math@editable-in-place`, and TeX's rules give the look a reader of TeX
expects, `goal@rootpoint-math@tex-quality`. The survey of
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2, found one Rust typesetter whose
boxes are tied to the atoms and cells of the formula, mathtex, which is XeTeX translated to unsafe
code, and none built on the MATH table that a renderer can call for such boxes alone:
`rejected@rootpoint-math@rex-fork`, `rejected@rootpoint-math@mathtex-layout`,
`rejected@rootpoint-math@typst-math-layout` and `rejected@rootpoint-math@ratex-typesetter`. Every
math font the survey measured has a MATH table. ttf-parser reads it; read-fonts, which parley
links at 0.41, reads it from 0.45, which would link a second copy of read-fonts until parley moves,
`issue@rootpoint-math@read-fonts-math-table`.

### A layout at `k` times a size is the layout at that size scaled by `k` `##layout-scales-exactly`

Every value `lay_out` gives is a value of the font, in design units, times the font size of a
style; nothing is rounded to a pixel, and no device table is read. The test
`a_formula_fitted_by_scaling_equals_the_formula_laid_out_at_the_scaled_size` of
`path@rootpoint-math@tests/layout.rs` checks every box, glyph and rule of the reference formulas to
0.001 at five factors. So a formula fitted to an area is one layout and one multiplication, and the
same at every display, `goal@rootpoint-math@same-at-every-size`. The MATH specification gives
every value in design units or as a percentage, but for device tables, which correct a value per
pixel size and which an engine may ignore; none of the measured fonts carries one on a constant, as
section 2 of `path@rootpoint@docs/grounding/math-typesetting.md` measures. The rival, honouring
device tables or snapping rules to pixels, would make a formula's layout differ between window
sizes.

### An empty cell is laid out as a placeholder half an em wide, in every display `##empty-cell-placeholder`

A cell with no atom is a box half an em wide and as high as the font's x-height, marked empty in
the layout. An empty group and an empty nucleus of scripts take no width, as TeX gives them none,
so LaTeX's `{}^{14}\mathrm{C}` and `a{}={}b` are set as LaTeX sets them; their boxes are marked
empty too. TeX gives an empty cell no width, which would leave a caret nowhere to stand in a
numerator or a script, `goal@rootpoint-math@editable-in-place`. The space is the same in every
display, so it is not `rejected@rootpoint-app@reserved-frame-space`, which reserved space in the
editor alone and made the displays differ; a formula with no empty cell, as a finished one, takes
no such space.

On the owner's word: an unfinished formula is not expected in a real talk.

## The table of commands

### The table of commands is generated from W3C `unicode.xml` `##latex-command-set`

`path@rootpoint-math@src/commands.rs` maps each command name to its character and its class. The
command `cargo x math-commands` of `path@xtask@src/math_commands.rs` writes it, and
`path@rootpoint-math@W3C-NOTICE.txt` beside it, from `unicode.xml` and the math font. `unicode.xml`
gives a TeX-like type to most of its characters, and a licence that lets the binary be distributed
under MIT OR Apache-2.0, `design@rootpoint@permissive-licence`, which rules out LyX's table, under
the GPL. Its rivals are `rejected@rootpoint-math@unicode-math-table` and
`rejected@rootpoint-math@katex-table`.

### A command of a character the math font lacks is read as that character `##absent-commands-read`

The generator writes the commands of the characters the font lacks in `ABSENT`, beside `COMMANDS`,
and `command` reads both, so a formula holding such a command, as a hand-written `\Game`, reads it
as LaTeX does, `goal@rootpoint-math@latex-text`, and draws it as the font's missing glyph,
`design@rootpoint-render@no-fallback-math-font`. The rival, a table of the font's characters alone,
reads `\Game` as an unknown command, drawn red, where that head draws a missing glyph.

### The commands offered to an author are those of the characters the math font has `##offered-commands-drawn`

`commands` holds the commands of the table whose characters the built-in math font has, apart from
`ABSENT`, so a list offered to an author, the palette of the window's Selection tab and the
completion of a command being typed, offers none that the font draws as its missing glyph,
`design@rootpoint-render@no-fallback-math-font`. A command of `ABSENT` is still read where it is
written, `design@rootpoint-math@absent-commands-read`. The rival, a list of every command of
`unicode.xml`, offers the author a box for each character the font lacks.

### A command's name means what it means in LaTeX's math mode `##latex-command-meanings`

An author who knows LaTeX has nothing new to learn, `goal@rootpoint-math@latex-text`. The generator
takes every name of a character's columns `<AMS>`, `<mathlatex>` without a set, `<IEEE>`,
`<mathlatex set="unicode-math">` and `<latex>`, a name two characters claim going to the first
column of that order, so `\epsilon` is ϵ and `\varepsilon` ε as in TeX; it adds LaTeX's aliases
that no column gives, as `\to`, `\le` and `\neq`, each naming the command it stands for in
`path@elsewhere@base/fontdef.dtx`, and fontdef.dtx's `\emptyset`; it leaves out unicode-math's names
of an upright letter, as `\mupalpha`, since a letter is set italic and `\mathrm` sets one upright.
The test `a_command_means_what_it_means_in_latex` of `path@rootpoint-math@tests/font.rs` pins nine
names. The rival is `rejected@rootpoint-math@unicode-math-names-first`.

### A character's class is LaTeX's where LaTeX declares one other than `unicode.xml`'s type `##latex-character-classes`

A character takes the class its `type` gives in `unicode.xml`, ord where it has none, but for the
characters whose class `path@elsewhere@base/fontdef.dtx` declares otherwise: `.` an ord, `-` and
`*` bins, `:` a rel, `!` and `?` closes, and `\langle`, `\rangle`, `\lgroup`,
`\rgroup`, `\lmoustache` and `\rmoustache` opens and closes. The class sets the space around a
character, so a formula is spaced as a reader of TeX expects, `goal@rootpoint-math@tex-quality`.
The rival, the type alone, types `.` a punctuation, so `3.14` would take a thin space after its
point, and `-` an ordinary character, so `a - b` would be set as `a-b`.

### `-` and `*` are drawn as the minus sign and the asterisk operator `##latex-ascii-glyphs`

A `-` of a formula is drawn as − U+2212 and a `*` as ∗ U+2217, which `drawn` of
`path@rootpoint-math@src/table.rs` gives, as LaTeX's `path@elsewhere@base/fontdef.dtx` takes them
from its symbols font, so `a - b` shows a minus, as a reader of TeX expects,
`goal@rootpoint-math@tex-quality`. The rival, the hyphen and the asterisk of the text, draws
characters LaTeX's math does not set. The test `minus_and_asterisk_are_drawn_as_latex_draws_them`
of `path@rootpoint-math@tests/layout.rs` pins the two.
