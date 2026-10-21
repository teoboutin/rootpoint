<!-- The report of the prior-art survey run on 2026-10-20 for the design discussion of the
milestone of the first prototype, on slide tools that lay out a slide as a recursive split tree. Kept as
the subagent wrote it; the milestone document quotes its facts. -->

# Prior art: slides laid out as a recursive split tree

Labels: **V** means I read the docs or the page in this session. **K** means it comes from my prior knowledge and I did not re-read it here, so treat it as an assumption. **S** means it comes only from a search-result summary.

## Tools checked

| Tool | URL | Layout model | Recursive split tree, editable by drag? | GUI / code |
|---|---|---|---|---|
| open-scrl, issue #23 | github.com/HeyPortal/open-scrl/issues/23 | **Proposal.** Start from one cell, split it horizontally or vertically, drag dividers to set ratios. Stored as a "normalised split tree" with snapped fractions, at most 12 cells and a 10% minimum cell size. It applies only to the *photo-grid* panel. | Yes, but not built. Issue opened 2026-10-07, still open, no PR (V). | GUI (web + Mac) |
| presenterm (Rust, TUI) | mfontanini.github.io/presenterm/features/layout.html | `column_layout: [3, 2]` gives integer-weighted columns, `column: n` sends content to a column, `reset_layout` returns to full width. The docs describe one level only; nesting is not documented. | No. One level of columns, no rows (V). | Code (Markdown) |
| Slidev | sli.dev/builtin/layouts | Named Vue layouts (`two-cols`, `two-cols-header`, …) with `::right::` slot markers. Anything deeper is hand-written CSS/HTML inside a slot. | No. Fixed templates (V/S). | Code |
| mdx-deck | dev.to/davefollett/learning-mdx-deck-layouts-1f2g | Fixed layout components: `Split`, `SplitRight`, `Horizontal`, … | No. Templates (S). | Code |
| Typst: Touying / Polylux | typst.app/universe/package/touying | Touying's `composer` takes grid widths such as `(1fr, 2fr)`; Polylux has `side-by-side`. Typst `grid`/`stack` nest to any depth, so you can write a split tree by hand. | Nesting is possible but written as code, no dragging, no split-tree data model (S/K). | Code |
| LaTeX beamer `columns` | ctan.org/pkg/beamer | `columns`/`column{0.4\textwidth}`; minipages nest. | Code-level nesting only (K). | Code |
| Quarto / pandoc / reveal.js | quarto.org/docs/presentations | `:::: columns` / `::: {.column width="40%"}` fenced divs; reveal.js layout is HTML/CSS (`r-stack`, `r-hstack`). | Columns, nestable as divs. Not a tree editor (K). | Code |
| sent (suckless) | tools.suckless.org/sent | One item per slide, centred. | No (K). | Code |
| pinpoint (GNOME) | wiki.gnome.org/Apps/Pinpoint | One text block over a background. | No (K). | Code |
| Pympress | github.com/Cimbali/pympress | PDF viewer only. | N/A (K). | GUI viewer |
| Figma / Figma Slides | help.figma.com (auto layout) | Auto layout: nested horizontal/vertical/grid frames with hug, fill or fixed sizes. Grid flow is in beta and children can span cells. Not available on the slide layer itself (forum request, April 2026). | Close structurally: a nested stack tree. But children size by fill/hug, not by a ratio you drag on a divider (S). | GUI |
| Gamma | gamma.app | Cards with AI-chosen layouts; 1–3 columns and asymmetric 1/3–2/3 splits, per a third-party guide. | No. Templates plus columns (S, unverified). | GUI |
| Pitch | help.pitch.com/en/articles/7003075 | Free-placed blocks, guides and gridlines, a fixed set of workspace Layouts, reflow inside groups. | No (S). | GUI |
| Canva Grids | canva.com/help/using-grids | Preset photo grids with a spacing slider. No cell splitting was found. | No (S). | GUI |
| Beautiful.ai | beautiful.ai | Over 300 "Smart Slide" templates that reflow as content is added. | No. Adaptive templates (S). | GUI |
| PowerPoint / Keynote / Google Slides | n/a | Master layouts with placeholders, plus free placement. | No (K). | GUI |
| Tome | n/a | Tiles in rows on a page (K, not re-verified; the product has pivoted). | Rows of tiles, not a recursive tree (K). | GUI |

**Research and patents (the structure, not a slide tool):** guillotine / slicing-tree page layout is well studied for *automatic* layout:
- "Optimal guillotine layout", DocEng 2012, doi.org/10.1145/2361354.2361359;
- US patent 7634723, "Constrained document layout", which optimises a slicing tree;
- Microsoft Research TR-2002-126, DRESS: binary slicing trees for adaptive web pages.

These choose the tree by an algorithm; none lets a user edit it by hand (S).

**Toolkits that already implement the interaction:**
- iced `pane_grid`, modelled on i3/awesome/tmux (S);
- ratatui-hypertile, a BSP layout engine (S);
- hjkl-layout, a binary split tree (S).

These are UI widgets, not slide tools.

## Conclusion

- **No shipped tool does exactly this.** I found no presentation tool, GUI or markup, whose slide model is a recursive tree of horizontal/vertical splits with draggable ratios and one content leaf per region. Absence of evidence from roughly 15 searches is not proof; GUI tools I did not open (Keynote, Google Slides, Gamma's help centre) are labelled as assumptions above.
- **Closest in GUI: Figma auto layout.** It is a nested row/column/grid tree, and grid children can span cells. It lacks ratio-on-divider editing, since sizes come from fill/hug/fixed rules, and the slide itself is not the root of the tree.
- **Closest in concept: open-scrl issue #23.** It describes precisely this model, a normalised split tree with snapped divider fractions, but it is an unbuilt proposal and covers photo grids only.
- **Closest in code: Typst (grid/stack nesting) and beamer/Quarto nested columns.** These can express a split tree, but only as text. There is no tree data model, no dragging, and no splitting of a leaf in place.
- **presenterm is one level of weighted columns.**
- **Spanning or linked splits:** I found this only in Figma's beta grid (cell spanning) and in CSS grid. No split-tree tool links splits across siblings. In i3/bspwm terms, nothing found has a cross-branch constraint.
- **Theory:** guillotine/slicing-tree layout is established prior art for automatic document layout. That supports the idea that the model is sound. It does not cover the hand-edited, i3/Blender-style interaction.

## Decided

- I scoped "verified" as "read in this session, page or search summary", and labelled prior-knowledge items as assumptions instead of fetching each one. Ground: the word budget in the brief.

## Met outside the task

None.
