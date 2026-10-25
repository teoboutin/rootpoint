# How LyX represents, navigates and decorates nested text

## When, what, and from which sources

Run on 2026-10-24, as grounding for a design discussion on rich text (styled spans) and later
inline maths in rootpoint's text areas.

It asked:

1. How LyX stores a paragraph, and whether character styling is a tree or a flat attribute.
2. How the cursor is represented, and how horizontal arrows enter and leave an inset.
3. How the maths editor stores formulas, moves the cursor into cells, draws its frames, and maps
   LaTeX-like typing to tree edits.
4. Which editor-only markers LyX draws, and whether they take layout space.
5. How a selection is constrained across nesting levels.
6. Known pain points of the model, in particular flat fonts versus character-style insets.
7. Secondary: TeXmacs, the closest relative.

Sources read:

- LyX source, official repository `https://git.lyx.org/repos/lyx.git`, `master` at
  `64141cf0e7a073e1226ec168d317618537ba0fab` (2026-10-24, latest tag `2.5.0-RC1`). Every
  citation below is to that commit. The first reading was done on the GitHub mirror
  `github.com/cburschka/lyx`, commit `3a02e477a00077e5442d8e34e92690d26a7ed4ae` (2023-03-21);
  see "Version check" at the end for what differs. `github.com/lyx/lyx` returns HTTP 404, and git.lyx.org no longer serves gitweb (its page says
  browsing moved to a Codeberg mirror because of bot traffic).
- Files read in the LyX tree:
  - `path@elsewhere@src/Paragraph.h`
  - `path@elsewhere@src/Paragraph.cpp`
  - `path@elsewhere@src/FontList.h`
  - `path@elsewhere@src/FontList.cpp`
  - `path@elsewhere@src/FontInfo.h`
  - `path@elsewhere@src/Font.cpp`
  - `path@elsewhere@src/InsetList.h`
  - `path@elsewhere@src/CursorSlice.h`
  - `path@elsewhere@src/DocIterator.h`
  - `path@elsewhere@src/DocIterator.cpp`
  - `path@elsewhere@src/Cursor.h`
  - `path@elsewhere@src/Cursor.cpp`
  - `path@elsewhere@src/Text.cpp`
  - `path@elsewhere@src/RowPainter.cpp`
  - `path@elsewhere@src/Color.cpp`
  - `path@elsewhere@src/insets/Inset.h`
  - `path@elsewhere@src/insets/Inset.cpp`
  - `path@elsewhere@src/insets/InsetText.h`
  - `path@elsewhere@src/insets/InsetText.cpp`
  - `path@elsewhere@src/insets/InsetCollapsible.h`
  - `path@elsewhere@src/insets/InsetCollapsible.cpp`
  - `path@elsewhere@src/insets/InsetFlex.cpp`
  - `path@elsewhere@src/insets/InsetLayout.cpp`
  - `path@elsewhere@src/mathed/InsetMath.h`
  - `path@elsewhere@src/mathed/InsetMath.cpp`
  - `path@elsewhere@src/mathed/InsetMathNest.h`
  - `path@elsewhere@src/mathed/InsetMathNest.cpp`
  - `path@elsewhere@src/mathed/InsetMathFrac.h`
  - `path@elsewhere@src/mathed/InsetMathFrac.cpp`
  - `path@elsewhere@src/mathed/MathData.h`
  - `path@elsewhere@src/mathed/MathRow.h`
  - `path@elsewhere@src/mathed/MathRow.cpp`
  - `path@elsewhere@lib/layouts/stdinsets.inc`
  - `path@elsewhere@lib/doc/Math.lyx`
  - `path@elsewhere@lib/doc/UserGuide.lyx`
- LyX bug tracker: tickets 1973 and 6919 (`https://www.lyx.org/trac/ticket/1973`,
  `https://www.lyx.org/trac/ticket/6919`), both cited in a comment of `Text::setFont`.
- Web search results summarising the LyX 2.2.1 `ANNOUNCE` file and the `CharStyle` to `Flex`
  rename commit, on the mirror `code.ascee.nl/anne/lyx_mirror` (not opened in full).
- TeXmacs: `https://www.texmacs.org/tmdoc/devel/scheme/edit/edit-model.en.html`,
  `https://www.texmacs.org/tmdoc/main/editing/man-structured-move.en.html`, and search
  summaries of texmacs-users threads 2019-05/msg00065 and msg00079. TeXmacs source was not read.

## 1. Text representation

**A paragraph is a flat string plus two side tables.** `Paragraph::Private` (`path@elsewhere@src/Paragraph.cpp`)
holds:

- `TextContainer text_`, a `docstring` (one `char_type` per position);
- `FontList fontlist_`, the fonts;
- `InsetList insetlist_`, the insets;
- `Changes changes_`, the change-tracking ranges.

**An inset occupies exactly one position of the string.** `Paragraph::insertInset` inserts the
code point `META_INSET` (`0x200001`, `path@elsewhere@src/Paragraph.h`) at `pos`, and `InsetList` stores
`Element { pos_type pos; Inset * inset; }` in a sorted vector (`path@elsewhere@src/InsetList.h`). The parent
paragraph sees the whole inset as one character.

**Character fonts are a flat run-length attribute, not a tree.** `FontList` is a sorted
`std::vector<FontTable>`; each `FontTable` holds an end position and a `Font`. The header comment
in `path@elsewhere@src/FontList.h` states the invariant: `pos_1 < pos_2 < ...`, `font_{i-1} != font_i`, and
`font_i` covers the positions after `pos_{i-1}` up to `pos_i`. `FontList::set`
(`path@elsewhere@src/FontList.cpp`) splits a run in three when needed and merges a run with an equal neighbour,
so two adjacent bold runs become one. There is no identity for "this bold span": the attribute is
a function of the position.

A `Font` wraps a `FontInfo` (`path@elsewhere@src/FontInfo.h`) with the fields `family_`, `series_` (bold),
`shape_` (italic), `size_`, `style_`, `color_`, `background_`, `paint_color_`, and the toggles
`emph_`, `underbar_`, `strikeout_`, `xout_`, `uuline_`, `uwave_`, `noun_`, `nospellcheck_`,
`langlocal_`. A value may be `INHERIT`, resolved from the paragraph layout and enclosing
environments (`FontTable` comment, `path@elsewhere@src/FontList.h`). The file format writes changes as toggles, such
as `\series bold` ... `\series default` (`Font::lyxWriteChanges`, `path@elsewhere@src/Font.cpp`).

**Nesting comes only from insets.** The class chain is `Inset` ← `InsetText` ←
`InsetCollapsible` ← `InsetFlex` (the headers of LyX's src/insets directory). `InsetText` owns a `Text text_`, and
`Text` owns a `ParagraphList pars_` (`path@elsewhere@src/Text.h`). So an inset in text is a box that holds a
whole sub-document of paragraphs, each again a flat string with flat fonts and its own insets.

- `InsetText`: the generic text container (footnotes, table cells, boxes...).
- `InsetCollapsible`: an `InsetText` with a button or label, open or collapsed.
- `InsetFlex`: user-defined insets from layout files. A layout with `LyXType charstyle` is a
  "character style"; it defaults to `multipar_ = false` and `forceplain_ = true`
  (`path@elsewhere@src/insets/InsetLayout.cpp`). Character styles exist since LyX 1.4.0 and were renamed Flex insets in
  1.6.0 (search summary of the rename commit and the User's Guide).
- `InsetMath*`: maths, a separate tree (section 3).

So LyX has two styling mechanisms side by side: direct formatting (bold, colour, size) is a flat
per-position attribute of a paragraph, and semantic character styles (`Flex:Emph`, `Flex:Code`
from the Logical Markup module, for example) are child insets in the tree.

## 2. The cursor

**The cursor is a stack of slices, one per nesting level.** `DocIterator` (`path@elsewhere@src/DocIterator.h`)
holds `std::vector<CursorSlice> slices_`; `bottom()` is the document's top-level text, `top()`
the innermost level, and `depth()` the size. A `CursorSlice` (`path@elsewhere@src/CursorSlice.h`) holds
`Inset * inset_` (the owning inset), `idx_` (cell index, for tables and maths cells), `pit_`
(paragraph index in that cell) and `pos_` (position in that paragraph or maths array). The class
comment says that the separation between maths and text slices should vanish "after inset
unification"; it has not.

**Entering an inset costs one key press, and so does leaving it.** With text `ab[XY]cd`, where
`[XY]` is an inset at position 2, the caret positions met by repeated `LFUN_CHAR_FORWARD` are:

| depth | slice positions | caret shown |
| --- | --- | --- |
| 1 | pos 2 | `ab|[XY]cd` |
| 2 | outer pos 2, inner pos 0 | `ab[|XY]cd` |
| 2 | inner pos 1 | `ab[X|Y]cd` |
| 2 | inner pos 2 | `ab[XY|]cd` |
| 1 | pos 3 | `ab[XY]|cd` |

The mechanism, from the source:

- `Text::cursorForward` (`path@elsewhere@src/Text.cpp`) first calls `checkAndActivateInset(cur, true)`. If the
  next position holds an `editable()` inset, it calls `inset->edit(cur, front)`.
  `InsetCollapsible::edit` pushes a slice (`cur.push(*this)`), and `InsetText::edit` sets the new
  top slice to the first paragraph, position 0 (or the last paragraph's end when entering
  backward). The cursor did not advance in the outer paragraph: the outer slice stays at the
  inset's position.
- Entering backward first does `--cur.pos()`, with the comment that "when entering an inset we
  are expected to be positioned *before* it in the containing paragraph, regardless of the
  direction". The outer slice of a cursor inside an inset therefore always points at the inset's
  own `META_INSET` position.
- At the end of the inner text, `cursorForward` does not move. `Text::dispatch` for
  `LFUN_CHAR_FORWARD` (`path@elsewhere@src/Text.cpp`) then marks the command undispatched and replaces it with
  `LFUN_FINISHED_FORWARD`. `Cursor::dispatch` (`path@elsewhere@src/Cursor.cpp`) loops
  `for (; depth(); pop(), ...)`, handing the command to each enclosing inset in turn, so the
  next level up receives `LFUN_FINISHED_FORWARD`, whose handler does `++cur.pos()`: the caret lands
  after the inset. `LFUN_FINISHED_BACKWARD` only resets the font, since the outer position
  already points before the inset.
- `DocIterator::forwardPos` (`path@elsewhere@src/DocIterator.cpp`), used by search and iteration rather than by
  the arrow keys, follows the same order: descend into the next inset if `isActive()`, else step,
  and at the end of a cell `pop_back()` then `++top().pos()`.
- An inset that is not `editable()` (a collapsed one, for example) is skipped as one character.
  `forwardPosIgnoreCollapsed` does the same for iteration.

**Font boundaries in flat text have no positions of their own.** Between `a` and a bold `b` there
is one caret position. `Cursor::setCurrentFont` (`path@elsewhere@src/Cursor.cpp`) picks the font that typing
will use: the font of the character after the caret, except at the paragraph's end or before a
separator, where it takes the previous character's font (plus a `boundary()` flag case for line
ends). Typing at `a|b` therefore produces bold text unless the user toggles. A character-style
inset, in contrast, gives the boundary two distinct positions, outside and inside.

## 3. Maths

**A formula is a tree of `MathAtom`s.** `MathData` (`path@elsewhere@src/mathed/MathData.h`) is
`private std::vector<MathAtom>`, one cell's content. `InsetMathNest` holds
`std::vector<MathData> cells_` (`path@elsewhere@src/mathed/InsetMathNest.h`); `InsetMathFrac`, `InsetMathGrid`,
`InsetMathScript`, `InsetMathFont` (`\mathbf` and the like) and the others derive from it. The
outer formula in text is `InsetMathHull` (an `InsetMathGrid`). In maths, a font change is a
nested inset with one cell, so maths styling is a tree, unlike text styling.

**The same slice stack covers maths.** A slice in maths uses `idx` for the cell and `pos` for the
index into that cell's `MathData`.

**Horizontal movement** is `Cursor::mathForward` (`path@elsewhere@src/Cursor.cpp`):

- if `pos() < lastpos()` and the next atom is `openable`, it calls `pushBackward(nextMath())` and
  `inset().idxFirst(*this)`: the caret goes to the start of the inset's first cell;
- otherwise it calls `posForward()`;
- at the end of a cell it calls `inset().idxForward(*this)`. The default,
  `InsetMathNest::idxForward`, calls `idxNext`: next cell, position 0. `InsetMathFrac::idxForward`
  returns false except for the `\unitfrac` kinds, so from the end of a numerator the right arrow
  leaves the fraction instead of going to the denominator; the denominator is reached vertically;
- when no next cell exists it calls `popForward()`, but only if the parent slice is also maths
  ("don't pop out of math! leave that to the FINISH lfuns"); leaving the formula itself uses the
  same `LFUN_FINISHED_FORWARD` path as text.

**LaTeX-like input edits the tree directly.** `InsetMathNest::interpretChar`
(`path@elsewhere@src/mathed/InsetMathNest.cpp`):

- `\` inserts an `InsetMathUnknown` holding the name being typed ("macro mode"); each letter
  extends the name (`activeMacro()->setName`);
- a non-letter closes macro mode: `Cursor::macroModeClose` replaces the unknown atom by
  `createInsetMath(name)`, moves any saved selection into its first cell, and `niceInsert` enters
  the new inset's first cell (`editInsertedInset`). Typing `\frac` then space thus produces an
  `InsetMathFrac` with the caret in the numerator;
- `^` and `_` call `InsetMathNest::script`: the previous atom becomes the nucleus of a new
  `InsetMathScript` (`cur.prevAtom() = MathAtom(new InsetMathScript(buffer_, cur.prevAtom(), up))`),
  or an existing script inset gets the missing script cell (`ensure(up)`), and the caret moves
  into that cell;
- `{` inserts an `InsetMathBrace`.

**The frames.** Two different marks exist:

- **Empty cells are drawn as small blue boxes.** In the `MathRow` constructor
  (`path@elsewhere@src/mathed/MathRow.cpp`), an empty `MathData` gets a `BOX` element coloured `Color_mathline`
  (`blue` in the light theme, `#ff86a4ff` in the dark one, `path@elsewhere@src/Color.cpp`). The Math manual and
  the User's Guide describe these as the "blue boxes" in which formula parts are typed
  (`path@elsewhere@lib/doc/Math.lyx`, `path@elsewhere@lib/doc/UserGuide.lyx`).
- **Corner marks around insets with cells.** `InsetMath::marker` returns `MARKER` (lower
  corners) for any `isActive()` inset, one with editable cells; fractions and grids return `MARKER2` (upper and
  lower corners); macros may return `BOX_MARKER` (corners plus a grey name label below).
  `drawMarkers` in `path@elsewhere@src/mathed/MathRow.cpp` draws an L of two 3-pixel strokes at each corner. Their colour is
  `Color_mathframe` (magenta) when `inset->editing(bv)` or `mouseHovered`, and `Color_mathcorners`
  otherwise. `Color_mathcorners` defaults to `Linen` (`#fffaf0e6`), which is also the default
  `Color_background`, so idle corners are painted in the page colour and are not seen; the code
  also skips painting when the pen colour equals `Color_mathbg` (`transparent` by default). In the
  default theme, corners therefore appear only on the insets that contain the cursor, and on the
  hovered one.
- `Inset::editing` (`path@elsewhere@src/insets/Inset.cpp`) is `bv->cursor().isInside(this)`, that is, the
  inset appears in some slice of the cursor's stack: every ancestor of the caret, and no other
  node. During a mouse selection it uses the anchor instead, to avoid flicker.
- `InsetMathHull::drawMarkers` draws the formula's own outer corners the same way.

## 4. Editor-only markers and layout space

**Every marker reserves its space in the metrics; none is drawn over text.** Measured from the
code, not from a running LyX:

- Maths corners: `markerMargin` returns 2 pixels for any marker type, and the `MathRow`
  constructor raises the `before`/`after` spacing of the element and of its neighbours to at
  least that. `afterMetricsMarkers` adds 1 pixel of descent (`MARKER`) or 1 of ascent and 1 of
  descent (`MARKER2`), and the label height for `BOX_MARKER`. The space is reserved whether or
  not the corners are painted, so moving the cursor does not reflow the formula.
- Text insets: `Inset::textOffset` is 4 pixels, the default of `topOffset`, `bottomOffset`,
  `leftOffset` and `rightOffset` (`path@elsewhere@src/insets/Inset.h`). `InsetText::metrics` adds them to the
  inset's dimension, and `InsetText::draw` paints its frame (`drawFrame_`, colour `frameColor()`)
  inside that margin.
- Character styles (`InsetFlex`) default to `InsetDecoration::CONGLOMERATE`
  (`InsetFlex::decoration`). Its geometry is `SubLabel` when open and `Corners` when collapsed
  (`InsetCollapsible::geometry`). For those, `topOffset` is 0 and `bottomOffset` is a quarter of
  the default. `InsetCollapsible::draw` draws: a bracket under the text (a bottom line with two
  4-pixel vertical ends), the inset's label in a font two sizes smaller under the bracket when
  open, and the two upper corners only when `cur.isInside(this)`. The label takes space:
  `InsetCollapsible::metrics`, `SubLabel` branch, adds the label's ascent and descent to the
  inset's descent and widens the inset to the label's width when the label is wider than the
  text. The upper corners, drawn only while the cursor is inside, fall inside the text's ascent
  and take no space of their own. `metrics` also sets `auto_open_` when the cursor is inside, so
  a collapsed inset opens while being edited.
- Other insets use `CLASSIC` (a button with the label, left of or above the content) or
  `MINIMALISTIC` (no button when open) decorations (`path@elsewhere@lib/layouts/stdinsets.inc`).
- Change bars: `RowPainter::paintChangeBar` fills a 3-pixel-wide bar at
  `leftMargin() - zoomedPixels(7)`, in a margin that `TextMetrics` widens by `changebarMargin()`
  when change tracking is shown.
- Foreign-language text: `RowPainter::paintForeignMark` draws a line under the text in
  `Color_language`, within the row's descent.
- Paragraph-end marks (`lyxrc.paragraph_markers`) are drawn as a glyph in `Color_paragraphmarker`
  (`path@elsewhere@src/TextMetrics.cpp`).
- Direct font changes (bold, colour) have no marker: the text is drawn in its font, and the
  boundary is visible only through the rendering.

## 5. Selection across nesting levels

**A selection lives in one slice: the cursor's top level.** The `Cursor` keeps an `anchor_`, a
full `DocIterator`. `CursorData::normalAnchor` (`path@elsewhere@src/Cursor.cpp`) cuts the anchor to the cursor's
depth (`anchor_[depth() - 1]`); if the anchor was deeper and lies after the cursor, it moves one
position on, past the inset. `selBegin`/`selEnd` compare that normalised slice with `top()`.
The consequences:

- the anchor must be at least as deep as the cursor; a deeper cursor is logged as "Cursor is
  deeper than anchor. PLEASE REPORT." and the anchor is reset;
- starting a selection inside an inset and extending it out of the inset selects the whole inset
  as one character at the outer level;
- `checkAndActivateInset` refuses to enter an inset during a selection unless the anchor is inside
  it (`cur.realAnchor().find(inset) == -1`), so shift-arrow from outside steps over insets.

The selection is therefore always a contiguous range of one `Text` or one `MathData` at the
level of the common ancestor; it never selects half of an inset.

**Applying a font to a selection that contains insets is a special case.** `Text::setFont(bv,
begin, end, font)` (`path@elsewhere@src/Text.cpp`) steps over the range with `CursorSlice::forwardPos` (no
descent), sets each position's font, and, for an inset whose `resetFontEdit()` is true, calls
`setInsetFont` to push the change into all its text cells. The comment cites bugs 1973 and 6919.

## 6. Known pain points

- **Fonts across inset boundaries.** Ticket 1973 (fixed for 1.5.0): bold applied to a selection
  that contains a table changed the screen but not the LaTeX output. Developers wrote that they
  did not fully understand "this font business". Ticket 6919 (fixed for 2.0.0): a language set on
  a line did not reach the text inside a character style; the fix uses the `ResetsFont` layout
  tag during editing only. Both arise from the two models meeting: a flat attribute in the
  parent, and a separate sub-document inside the inset.
- **Two models for one user concept.** Direct formatting is flat; semantic styles are insets. The
  User's Guide steers users to semantic markup, and the default text classes provide only
  `Emph` and `Noun` as character styles (search summary of the User's Guide). The LyX file format
  and LaTeX export work run by run, which fits LaTeX's own commands.
- **Character styles constrain content.** LyX 2.2.1's `ANNOUNCE` states that character styles
  now force the plain layout by default, because a paragraph layout inside them produced invalid
  LaTeX; `ForcePlain 0` restores the old behaviour (search summary, file not opened).
- **Text and maths are separate implementations.** The `CursorSlice` comment calls the split
  between `InsetMath`/`MathData` and `Inset`/`Text` something that "should vanish". Cursor code
  checks `inMathed()` and `asInsetMath()` in many places (for example the FIXME in
  `forwardPosIgnoreCollapsed`).
- Not found: a user or developer thread arguing for or against making bold an inset. The searches
  returned only code mirrors. Whether such a discussion exists is unverified.

## 7. TeXmacs (secondary)

- TeXmacs stores the whole document as one tree of tags; formatting such as `strong` is a tag
  wrapping its content (the content-model page's example
  `(tree-set t 0 `(strong ,(tree-ref t 0)))`).
- A cursor position is a path, a list of integers from the root; the editing-model page gives
  `(1 0 1)` as the position just after the subscript `1` whose own path is `(1 0)`.
- It draws a cyan box around the tag containing the cursor ("focus"); per a 2019 texmacs-users
  reply, the colour is hard-coded and drawn by `draw_env` in `edit_repaint.cpp`. Whether it boxes
  only the innermost tag or every ancestor was not checked.
- The manual's structured-movement page describes keys that exit the innermost tag
  and that visit "all accessible cursor positions". Whether a plain arrow key costs one press
  at a tag border, as in LyX, was not checked.

## What transfers, and the property each depends on

- **Cursor as a stack of (container, index) slices, one per level.** Transfers if rootpoint's
  text model is a tree in which each node owns an ordered sequence of children and characters.
  It does not depend on how fonts are stored.
- **A child counts as one position in its parent, and entering or leaving it costs one key
  press.** This is the owner's "invisible character" model, and LyX does exactly that: the
  outer slice points at the child's `META_INSET` position while the cursor is inside. It depends
  on the child being an entity in the sequence. With flat runs, as LyX's bold, the boundary has
  one position only, and the typing style is chosen by a heuristic (`setCurrentFont`).
- **Frames only on the cursor's ancestors.** LyX's `Inset::editing` is "appears in the cursor's
  slice stack", which is cheap to compute from the stack. It depends on the stack naming each
  ancestor node.
- **Frames do not move text when the caret moves.** LyX reserves the marker space in the metrics
  of every inset that could carry a marker, painted or not. Rootpoint gets the same property only
  if the layout reserves the space unconditionally, or if frames are drawn in space that never
  holds text (for example over inter-glyph gaps or as an overlay outside the line box). Reserving
  it changes the editor's layout relative to the output; LyX accepts that because its screen is
  not its output.
- **Selection confined to the common ancestor's level, children taken whole.** Transfers to any
  tree model. It keeps every selection a contiguous range of one container, so cut, paste and
  restyle never split a node. It costs the ability to select from the middle of one styled span
  to the middle of the next, which LyX's flat fonts allow and its character styles do not.
- **Two styling models (flat attributes plus nested insets).** LyX's tickets 1973 and 6919 show
  the cost at the seam. A single model avoids that seam. Which single model fits depends on
  whether rootpoint wants span identity (a tree) or canonical merging of equal neighbours (flat
  runs, as `FontList::set` does).
- **Maths as typed LaTeX that rewrites the tree.** The macro-mode pattern (an "unknown" node that
  holds the name being typed and is replaced on the next non-letter) transfers if inline maths is
  its own node kind with cells. It does not depend on the text model.

## Version check

Between the 2023 mirror commit and `64141cf0e7`, `git diff --stat` over the 23 cited source files
reports 1521 insertions and 5508 deletions. Each claim above was re-checked by grep or by reading
at `64141cf0e7`. What changed:

- `path@elsewhere@src/Text2.cpp` and `path@elsewhere@src/Text3.cpp` were merged into `path@elsewhere@src/Text.cpp`; `checkAndActivateInset`,
  `cursorForward`, the `LFUN_FINISHED_FORWARD` handler and `setFont` are unchanged in substance.
  `setFont` now also skips pass-through paragraphs (bug 13350).
- `FontInfo` lost `number_` and gained `langlocal_`.
- `InsetMath::marker` tests `isActive()` instead of `nargs() > 0`.
- `Color_mathbg` became `transparent` (it was `Linen`); `Color_mathcorners` stays `Linen`, now
  equal to the background rather than to `Color_mathbg`. The visible result is the same.

Unchanged: the `Paragraph::Private` fields, `META_INSET`, the `FontList` invariant and merge,
`CursorSlice` and its "inset unification" comment, `DocIterator::forwardPos`, the
`Cursor::dispatch` pop loop, `normalAnchor`, `mathForward`, `InsetMathFrac::idxForward`,
`InsetMathNest::idxForward`, the macro mode and `script`, `MathRow` marker margins and
`drawMarkers`, `Inset::editing`, `InsetCollapsible` metrics and corner cue, `InsetFlex`'s
`CONGLOMERATE` default, `Inset::textOffset` of 4, the change bar.
