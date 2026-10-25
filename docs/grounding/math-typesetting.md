<!-- The reports of the agents that grounded the design discussion on math, run on 2026-10-25 for
the milestone `milestone@plans@math-support`, verbatim but for their headings, moved down two
levels, and for paths written as references; then the measurements the session took itself.
Their scratchpad paths no longer exist. -->

# Math in LyX, Rust math typesetters, math fonts and command tables

## When, what, and from which sources

Run on 2026-10-25, as grounding for the design discussion on math support: inline math in a text,
and a numbered equation as a kind of content that text anywhere can reference. Four agents read
the sources:

- one agent read LyX's bind files and math model, in LyX `master` at `e7ffd8d`; its report is
  section 1. The session spot-checked its bindings against the agent's clone: lines 37 and 152 of
  `path@elsewhere@lib/bind/math.bind`, lines 77 to 79 of `path@elsewhere@lib/bind/cua.bind`, and
  the count of 122 `M-m` bindings;
- one agent ran the survey of Rust typesetters, of the MATH table in the font crates, of linearity
  and of the math fonts, and dispatched two subagents, one on the typesetters and one on the
  command tables and MathLive; it spot-checked their load-bearing claims against their clones. Its
  report is section 2, and its second subagent's own report is section 3;
- the measurements the session took itself are section 4.

V marks a claim verified by reading or measuring, A one assumed or from memory, in each report as
its author marked it.

## 1. LyX's math keys and math model, read in its source

I could not write findings.md. The Write tool refused it with "Subagents should return findings as text, not write report files", so the whole report is below. Nothing was built or run from the downloaded tree.

**Source:** both URLs you gave failed. The GitHub API returned 404 for lyx/lyx, and `https://git.lyx.org/lyx.git` returned "not found". `https://git.lyx.org/repos/lyx.git` uses the dumb HTTP protocol, which cannot do a shallow clone. I cloned `git://git.lyx.org/lyx` with depth 1 into `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/lyx/src`.
- I read master at commit e7ffd8d31564c08e3ccc426c898db1e9f305ebf4 (2026-10-25). Line numbers below are master's.
- Against tag 2.4.4, `lib/bind/math.bind` is identical, and the math lines of `cua.bind` are the same. `cua.bind` differs only in non-math lines.
- The C++ files differ between 2.4.4 and master mostly by renames (`label_` to `labels_`, `write` to `writeMath`). I did not check every hunk, so the C++ claims are verified for master and assumed for 2.4.4.

#### 1. Key bindings

**Which files load**
- The default bind file is cua: `path@elsewhere@src/KeyMap.cpp` line 234 has `static string const defaultBindfile = "cua";`. On macOS, `development/MacOSX/lyxrc.dist.in:15` sets `\bind_file "mac"`.
- `cua.bind:256-260` includes `menus.bind`, `math.bind`, `latinkeys.bind`, `cyrkeys.bind` and `greekkeys.bind`. `mac.bind:377-381` includes the same five.
- `site.bind` is always loaded first.
- There is no `greek.bind`. `greekkeys.bind` has 75 bindings, each mapping an X11 Greek keysym to `self-insert` (e.g. `\bind "Greek_alpha" "self-insert"`). It serves Greek keyboard layouts and has no math bindings.
- In bind files, C = Ctrl, M = Alt, S = Shift, and ~S means the Shift state is ignored.
- `mac.bind:38-43` says: "Command - C / Option - A / Control - M". By that comment, Alt+M on Linux and Windows is Control+M on a Mac. I have not tested this.

**Entry chords** (`cua.bind`, repeated at `mac.bind:163-165`):
```
77: \bind "C-m"     "math-mode"
78: \bind "C-S-M"   "math-display"
79: \bind "C-M-n"   "command-sequence math-display; math-number-toggle;"
```
- Ctrl+M makes an inline formula, Ctrl+Shift+M a display formula, Ctrl+Alt+N a numbered display formula.
- Also: `cua.bind:106` C-S-underscore → `math-macro-fold`; `cua.bind:245` C-space → `command-alternatives math-space ; space-insert protected`; `menus.bind:148` "M-c m" → `math-mode`.
- No default key inserts a label or a cross-reference in cua, menus, math or mac bind files. `label-insert` is reached by menu: Insert > Label (`stdmenus.inc:401`) and "Equation Label" in the math context menu (`stdcontext.inc:58`).
- No key inserts a matrix. Insert > Math has "Array Environment" (`math-matrix 2 2`) and "Matrix..." (`dialog-show mathmatrix`), at `stdmenus.inc:492,500`.
- No top-level menu uses M as its accelerator (`stdmenus.inc:28-38`), so Alt+M does not clash with a menu.

**The Alt+M prefix table** (`lib/bind/math.bind`)

Structures and operators (lines 37-58, 136-140):

| keys after Alt+M | function |
| --- | --- |
| f | `math-insert \frac` |
| s | `math-insert \sqrt` |
| x | `math-subscript` |
| e | `math-superscript` |
| u | `math-insert \sum` |
| i | `math-insert \int` |
| y | `math-insert \oint` |
| p | `math-insert \partial` |
| r | `math-insert \root` |
| 8 | `math-insert \infty` |
| m | `math-mode` |
| d | `math-display` |
| l | `math-limits` |
| S-L l / S-L n / S-L space | `math-limits limits` / `nolimits` / `empty` |
| n | `math-number-toggle` |
| S-N | `math-number-line-toggle` |
| o 1 … o 9 | `self-insert \#1` … `\#9` (macro arguments) |
| space | `math-space` |
| + | `math-insert \pm` |
| = | `math-insert \neq` |

Hull type changes, Alt+M t then a key (lines 70-76), all `math-mutate`:

| key | hull type |
| --- | --- |
| n | none |
| t | dump |
| a | align |
| i | simple |
| d | equation |
| e | eqnarray |
| m | multline |

Columns, Alt+M c then a key (lines 78-85, 99-101), all `tabular-feature`:
- i append-column, d delete-column, c copy-column, s swap-column
- a add-vline-left, e delete-vline-left, z add-vline-right, x delete-vline-right
- l align-left, m align-center, r align-right

Rows, Alt+M w then a key (lines 87-98), all `tabular-feature`:
- i append-row, d delete-row, c copy-row, s swap-row
- a add-hline-above, e delete-hline-above, z add-hline-below, x delete-hline-below
- t valign-top, m valign-middle, b valign-bottom

Accents (lines 108-121), all `math-insert`:

| key | inserts |
| --- | --- |
| " | `\ddot` |
| h | `\hat` |
| \ | `\grave` |
| / | `\acute` |
| & | `\tilde` |
| - | `\bar` |
| . | `\dot` |
| S-V | `\check` |
| S-U | `\breve` |
| v | `\vec` |
| _ | `\underbar` |
| b | `\overline` |
| ' | `\prime` |

Delimiters (lines 125-130), all `math-delim`:

| key | pair |
| --- | --- |
| ( | `( )` |
| [ | `[ ]` |
| { | `{ }` |
| < | `langle rangle` |
| > | `rangle langle` |
| \| | `\| \|` |

Two more lines in `math.bind`: line 139 `\bind "Escape" "escape"` and line 146 C-S-asterisk → `math-macro-unfold`. `cua.bind:251` binds Escape to `cancel` before it includes `math.bind`. I did not check which of the two wins.

**Greek: Alt+M, then g, then a letter** (`math.bind:152-196`, all `math-insert`)

The comment at line 150 says "This should be handled properly by some 'get the next key' method".

All 26 lowercase letters are bound:

| letter | inserts | letter | inserts |
| --- | --- | --- | --- |
| a | `\alpha` | n | `\nu` |
| b | `\beta` | o | `\omega` |
| c | `\chi` | p | `\pi` |
| d | `\delta` | q | `\vartheta` |
| e | `\epsilon` | r | `\rho` |
| f | `\phi` | s | `\sigma` |
| g | `\gamma` | t | `\tau` |
| h | `\eta` | u | `\upsilon` |
| i | `\iota` | v | `\theta` |
| j | `\varphi` | w | `\omega` |
| k | `\kappa` | x | `\xi` |
| l | `\lambda` | y | `\psi` |
| m | `\mu` | z | `\zeta` |

18 Shift+letter keys are bound:

| key | inserts | key | inserts |
| --- | --- | --- | --- |
| S-D | `\Delta` | S-Q | `\vartheta` |
| S-E | `\varepsilon` | S-R | `\varrho` |
| S-F | `\Phi` | S-S | `\Sigma` |
| S-G | `\Gamma` | S-T | `\varsigma` |
| S-I | `\iota` | S-U | `\Upsilon` |
| S-J | `\epsilon` | S-V | `\Theta` |
| S-L | `\Lambda` | S-W | `\Omega` |
| S-O | `\Omega` | S-X | `\Xi` |
| S-P | `\Pi` | S-Y | `\Psi` |

Oddities in the file:
- o and w both give `\omega`. There is no omicron.
- v gives `\theta` and q gives `\vartheta`.
- Six Shift keys give lowercase variants instead of capitals: S-E, S-I, S-J, S-Q, S-R, S-T.
- A, B, C, H, K, M, N and Z have no Shift binding.

**Checking the owner's memory**
- "Alt+M into other keys for most math chars": confirmed. The Alt+M prefix is the default math table (`math.bind`, included at `cua.bind:257`). But "most" is wider than the file: it binds only the symbols listed above. Every other symbol is typed as `\name` or picked from a menu or toolbar.
- "f for a fraction": confirmed. `math.bind:37`: `\bind "M-m f" "math-insert \frac"`.
- "Alt+M into g then a letter for Greek": confirmed. It is a three-key sequence, and Shift+letter gives some capitals. On a Mac the prefix is Control+M, per the comment quoted above.

**Keys that act only inside math**

These are not in bind files. They are handled by `InsetMathNest::interpretChar` (`path@elsewhere@src/mathed/InsetMathNest.cpp` line 2022).
- **Backslash** starts macro mode: lines 2177-2193 insert an `InsetMathUnknown` named `\`, carrying any selection.
  - ASCII letters extend the name (lines 2052-2055).
  - Any other character closes macro mode (`macroModeClose`, line 2131) and is then interpreted again.
  - A space closes it and is swallowed: `else if (c != ' ') interpretChar(cur, c);`.
  - A single non-letter after the backslash makes a one-character command: `\\` gives `\backslash`, `\^` gives `\mathcircumflex`, and `\{` or `\%` wrap the selection in a brace or a comment (lines 2059-2087).
- **^ and _** call `script()` (lines 2263-2272). Any selection is moved into the script.
- **~** gives `\sim`. In text mode inside math, ^ and ~ give `\textasciicircum` and `\textasciitilde`.
- **{ } & $ # % _** each insert their own inset.
- **Space** in math mode first grows a preceding space inset. Otherwise it leaves the current inset (`popForward`), and at the end of the formula it leaves the formula (lines 2232-2252).
- **Ctrl+M inside math** does not leave math. `InsetMathNest.cpp:1317-1346` inserts a `\text` inset when the cursor is in math mode, or `\ensuremath` when it is in text mode inside math. `math-mode on` is ignored inside math.
- **Ctrl+Shift+M inside a formula** toggles inline and display: `mutate(type_ == hullSimple ? hullEquation : hullSimple)` in `InsetMathHull.cpp`, case `LFUN_MATH_DISPLAY`.
- **Tab**: `site.bind:30` binds it to `command-alternatives completion-accept;cell-forward;tab-insert;outline-in;depth-increment`.
- **Auto-correction** runs only if `autocorrection_math` is set, and it defaults to `false` (`LyXRC.h:545`).

#### 2. Math model

**Hull types**
- A top-level formula is an `InsetMathHull`, which is a subclass of `InsetMathGrid`.
- `InsetMath.h:23-37` lists the types: hullNone, hullSimple, hullEquation, hullEqnArray, hullAlign, hullAlignAt, hullXAlignAt, hullXXAlignAt, hullFlAlign, hullMultline, hullGather, hullRegexp, hullUnknown.
- `InsetMath.cpp:31-70` gives their names: none, simple, equation, eqnarray, align, alignat, xalignat, xxalignat, multline, gather, flalign, regexp.
- `simple` is the inline `$...$` formula. `regexp` is the hull used by advanced find.
- `numberedType()` (`InsetMathHull.cpp:1139-1162`): none, simple, xxalignat and regexp are never numbered. The others are numbered when at least one row is.
- The comment in `mutate()` gives the chain: `none <-> simple <-> equation <-> eqnarray -> *align* -> multline, gather`.
- Other insets are in `path@elsewhere@src/mathed/`: InsetMathFrac, Script, Sqrt, Root, Delim, Array, AMSArray, Macro, Ref, Symbol, Char, and more. `path@elsewhere@src/mathed/README` has a hierarchy diagram, but it uses old names.

**Numbering**
- Each row has a state, `InsetMathHull.h:34-40`: `enum Numbered { NONUMBER, NUMBER, NOTAG }`.
- The hull keeps three parallel per-row vectors: `numbered_`, `numbers_` (the number shown) and `labels_` (`InsetLabel*`).
- `math-display` from text inserts a simple hull, then mutates it (`Text.cpp:3446` `mathDispatch`). The mutation sets `numbered(0, labels_[0] != nullptr)`, so a new equation is unnumbered unless it already has a label.
- An unnumbered equation is written `\[ \]`. A numbered one is written `\begin{equation}` (`InsetMathHull.cpp:1270-1278`).
- `math-number-toggle` (lines 1989-2012) switches every row (only the last row for multline) and deletes labels when it turns numbers off. It is enabled only for display formulas.
- `math-number-line-toggle` (lines 2015-2027) switches the current row only.
- Ctrl+Return in a simple or equation hull mutates it to align, or to eqnarray when amsmath is off (`LFUN_NEWLINE_INSERT`).
- Delete-forward at the end of a row removes the row's label first, then its number.

**Labels**
- A label is an `InsetLabel` kept in `labels_[row]`, not an inset inside a cell (`InsetMathHull::label`, lines 1009-1031).
- `eol()` writes it into the row's LaTeX as `\label{…}`, followed by `\nonumber` or `\notag` where the row's state differs (lines 1815-1840).
- `label-insert` is enabled only when `type_ != hullSimple`. Its default name is `eq:` (lines 2029-2045).
- Setting a non-empty label numbers its row: `if (!str.empty()) numbered(r, true);`.
- `label-copy-as-reference` makes an `InsetRef`. Its type defaults to `eqref` unless the session setting `default_crossreftype` gives another.

**Cross-references**
- `InsetRef` types are listed at `InsetRef.cpp:1013-1023`: ref, eqref, pageref, cpageref, vpageref, vref, nameref, formatted, labelonly.
- On screen, eqref shows `(n)` and ref shows `n`.
- In LaTeX, eqref requires amsmath. Under the refstyle package it writes `(\ref{…})` instead (lines 361-369, 976-978).
- Inside math, a reference is an `InsetMathRef`.

**How numbers are computed**
- `InsetMathHull::updateBuffer` (around lines 240-327) uses the master buffer's document-class counters.
- The counter is `equation`, or `subequation` when the current counter is already `equation` and the class has a `subequation` counter.
- For each numbered row: `cnts.step(eqstr, utype); numbers_[i] = cnts.theCounter(eqstr, lang);`. A row in a deleted, change-tracked paragraph shows `#`.
- The number and its pretty and formatted forms are pushed into the row's `InsetLabel`.
- Numbering runs in document order. `Counters::step` (`Counters.cpp:390-410`) resets child counters.
- **Per document vs per section** depends on the layout:
  - `lib/layouts/stdcounters.inc:85-88` defines `Counter equation` with no `Within`, so it counts across the whole document.
  - The module `eqs-within-sections.module` sets `Within section`, `LabelString "\thesection.\arabic{equation}"`, and `\numberwithin{equation}{section}`.
  - `scrbook.layout:166-170` sets `Within chapter`.
  - These are the only three files that define `Counter equation` (grep).

#### 3. Typing, rendering, storage

**Completion**
- Completion is offered only in macro mode (`completionSupported` returns `cur.inMacroMode()`, `InsetMathNest.cpp:2388`).
- Defaults in `LyXRC.h`: inline completion on (`completion_inline_math = true`, line 529) with a 0.2 s delay; popup on (`completion_popup_math = true`, line 539) with a 2.0 s delay.
- The candidates are the buffer's macros, the global macros, a hard-coded list (`\boxed`, `\xrightarrow`, `\split`, `\cfrac`, `\smash`, …), and the non-hidden entries of `mathedWordList()`, which is read from `lib/symbols`.
- Accepting a finished completion dispatches a space, which closes macro mode (`insertCompletion`, lines 2431-2456).
- `Cursor::niceInsert` (`Cursor.cpp:1587`) moves the selection into the new inset's first cell and puts the caret there.

**Rendering**
- LyX draws math with its own painter by default (`InsetMathHull::draw`, line 656). Screen glyph definitions come from `lib/symbols`, whose header says the `\def` lines are "for screen display", not for LaTeX output.
- A LaTeX preview image replaces the native drawing only when all of these hold (`previewState`, lines 512-522):
  - the formula is not being edited;
  - `previewMath()` is true, which requires `lyxrc.preview == PREVIEW_ON` (`RenderPreview.cpp:55-61`);
  - the hull is not regexp;
  - the image exists.
- The default is `PREVIEW_OFF` (`LyXRC.h:468`). A formula under the caret is always drawn natively.
- For a preview, LyX passes its own equation number to LaTeX with `\global\def\theequation{…}`.

**Caret movement**
- Right arrow (`Cursor::mathForward`, `Cursor.cpp:2075`) enters the next inset at the start of its first cell (`idxFirst`). At the end of a cell it calls `idxForward`; if that fails, the caret pops out after the inset, but never out of the formula.
- Fractions:
  - `InsetMathFrac::idxForward` (`InsetMathFrac.cpp:95-110`) returns false for a plain `\frac`, so Right at the end of the numerator should leave the fraction rather than go to the denominator. I read this from the code but did not see it happen.
  - Up/Down (`InsetMathFracBase::idxUpDown`, lines 51-60) move between numerator (cell 0) and denominator (cell 1), keeping the remembered x position (`x2pos(..., cur.x_target())`).
  - Tab (`cell-forward`, `InsetMathNest.cpp:1097-1109`) goes from numerator to denominator and wraps back to the first cell after the last.
- Scripts:
  - `InsetMathScript::idxForward` and `idxBackward` return false (`InsetMathScript.cpp:453-462`), so Left and Right leave the script.
  - `idxUpDown` (lines 465-505) moves from the nucleus to an existing superscript or subscript only when the caret is at the end of the nucleus, or at its start for an operator with displayed limits. From a script, the opposite direction returns to the end of the nucleus.
- Typing ^ or _ (`InsetMathNest::script`, lines 2334-2385) makes the previous atom the nucleus of a new script inset and puts the caret at position 0 of the script cell. After an existing script inset it re-enters that inset. Inside a nucleus it moves to that inset's script, creating it if needed.

**Storage**
- A formula is stored as LaTeX inside a Formula inset in the `.lyx` file. `InsetMathHull::write` (lines 2412-2420): `oss << "Formula "; writeMath(wi);`. `read` parses it back with `mathed_parse_normal` (line 2426).
- From `lib/doc/Math.lyx`, an inline formula (line 1122) is `\begin_inset Formula $\to$` … `\end_inset`.
- A numbered equation with a label (lines 15720-15725):
  ```
  \begin_inset Formula 
  \begin{equation}
  \int A=B\label{eq:red}
  \end{equation}

  \end_inset
  ```
- The reference to it is a separate inset in the text (lines 15742-15747):
  ```
  \begin_inset CommandInset ref
  LatexCommand eqref
  reference "eq:red"
  nolink "false"

  \end_inset
  ```
- So the label lives inside the formula's LaTeX, and the reference is a command inset outside it.

#### Unverified
- The macOS modifier mapping: taken from the `mac.bind` comment, not from Qt.
- Which Escape binding wins (`cua.bind:251` or `math.bind:139`).
- That the C++ behaviour above is the same in 2.4.4.
- That Right at the end of a numerator leaves the fraction: read from the code, not observed.
- Whether the standard book and report classes number equations per chapter on screen; I only know no such `Within` line is in their layout files.

#### Met outside the task
- The given clone URLs (`github.com/lyx/lyx`, `git.lyx.org/lyx.git`) do not work. The working one is `git://git.lyx.org/lyx`. This is only for your information.
- The Write tool blocks report files for subagents, so findings.md does not exist. The caller can save this report there if it is wanted.

## 2. The survey: typesetters, the MATH table, linearity, fonts, command tables, MathLive

### Prior-art survey: math support for rootpoint

**When and what.** Run on 2026-10-25. It asked six questions:
1. Which Rust math typesetters exist, and could one serve as a library under our own renderer?
2. Do the font crates we already link parse the OpenType MATH table?
3. Is OpenType MATH layout linear in the font size?
4. Which math fonts are candidates, by file measurements and their own documentation?
5. Which tables from LaTeX command names to Unicode could we embed?
6. How does MathLive model a formula for editing, compared with LyX?

**How it was run.**
- I did Q2, Q3 and Q4 myself.
- Q1 went to one subagent, and Q5 and Q6 to a second. I spot-checked their load-bearing claims against their clones, as noted inline.
- V means verified by reading or measuring. A means assumed or from memory.
- Nothing was built or run from a downloaded tree.

**Scratch locations.**
- Root: `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/`
- Font measurement script: `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/scripts/measure.py`
- Fonts: `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/fonts/`
- Q1 clones: `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/q1/`
- Q5 and Q6 clones: `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/q56/`

#### Sources (versions and dates)

**Font crates**
- Local cargo registry: read-fonts 0.41.0, skrifa 0.44.0, ttf-parser 0.25.1 (crates.io, published 2024-11-29), harfrust 0.12.0, parley 0.12.0 (published 2026-10-09).
- Downloaded .crate files: read-fonts 0.42.0 to 0.46.0 (0.45.0 on 2026-10-03, 0.46.0 on 2026-10-21) and skrifa 0.49.0 (2026-10-21).
- crates.io dependencies API, read 2026-10-25, for skrifa 0.45 to 0.49, parley, fontique and harfrust 0.12.

**Specification**
- Microsoft OpenType spec, "MATH — The mathematical typesetting table", learn.microsoft.com, page metadata ms.date 2024-05-29, updated_at 2024-05-31.

**Fonts**

| font | version | where from |
| --- | --- | --- |
| Noto Sans Math | v3.000 (2024-06-05) | notofonts/math release; also the google/fonts `ofl/notosansmath` build |
| Fira Math | v0.3.4 (2020-10-15) | GitHub release |
| STIX Two Math | 2.13 b171 (2021-05-26) | stipub/stixfonts tag v2.13b171 |
| STIX Two Math | 2.12 b168a | google/fonts `ofl/stixtwomath` |
| Libertinus Math | 7.051 (2024-09-26) | alerque/libertinus release |
| Latin Modern Math | 1.959 (2014-09-05) | CTAN `lm-math` |
| New Computer Modern Math and Sans Math | name table says 4.0 | CTAN `newcomputermodern` package 8.1.1 (2026-06-22) |

- Issues: notofonts/math issues, read through the GitHub API on 2026-10-25.

**Q1 clones (each at the date of its commit)**
- ReTeX/ReX 7bd4efc (2020-07-15); KenyC/ReX (2026-02-24).
- typst v0.15.1, 9dfd3a0 (2026-07-17).
- ophymx/formulary 05dcb38 (2026-09-24); jscarr64/latex-rust 3c9c023 (2026-10-25).
- erweixin/RaTeX 28fa5a7 (2026-10-19); gabriel-nsiqueira/mathtex f257708 (2026-07-30).
- DioxusLabs/blitz main (2026-10-25).
- .crate files: pulldown-latex 0.8.0, math-core 0.9.0, katex-rs 0.3.0, katex 0.4.6, mathra 1.2.0, iced_math 0.5.0.

**Q5 and Q6 sources**
- unicode-math v0.8r (2023/08/13, CTAN) and GitHub 184a23b.
- KaTeX 8000fbd (v0.19.1, 2026-10-23); MathJax-src fb98717 (v4.1.3, 2026-07-03).
- LyX git clone e7ffd8d (2026-10-25); a second local LyX clone at 64141cf (2026-10-24) was used for `lib/autocorrect`.
- w3c/xml-entities efed20d (2026-09-28); WHATWG entities.json (Last-Modified 2025-11-12).
- typst/codex 0.3.0; unicodeit 0.2.1; mdwright-latex 0.1.3; latex-rust 3.0.1.
- arnog/mathlive 8d148b4 (v0.111.1, 2026-10-09).

#### Q1. Rust math typesetters

**Bottom line (V).**
- No candidate meets every requirement at once. The requirements are: layout from the OpenType MATH table, a box per atom and per cell, glyphs drawn by us, a wasm32 build, and a small dependency tree.
- None of the candidates links an output box back to an input node or cell, except mathtex.

**How dependency trees were counted (V).** A throwaway project of the subagent's own depends on each candidate with default features. In it, `cargo tree --target wasm32-unknown-unknown -e normal` was run and the unique packages counted.

| candidate | version, date, maintenance | licence | MATH table | boxes | wasm32 | deps (wasm32) | under our renderer |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ReX original (ReTeX/ReX) | git only; last commit 2020-07-15 (V) | no LICENSE file; README claims MIT and Apache (V) | no: STIX2 metrics compiled in (V) | `LayoutNode` tree (V) | iReX web demo linked (V); that it is wasm is A | n/a | dead upstream |
| ReX fork KenyC/ReX (`rex` 0.1.2, git only; crates.io `rex` is an unrelated language) | last commit 2026-02-24 (V) | MIT (V) | yes, behind a `MathFont` trait; backends ttf-parser or pdf-rs; a read-fonts backend would be ours (V) | `LayoutNode { node, width, height, depth }` in units of `Unit<Px>`, with Grid, HBox, VBox, Glyph, Rule and Kern; positions implicit; no node id (V, spot-checked `path@elsewhere@src/layout/mod.rs` line 240) | pure Rust deps (A) | 10, or 11 with ttf-parser (V) | yes: a `Backend` trait lets us draw |
| typst math (`typst-layout` 0.15.1, 2026-07-17) | active | Apache-2.0 (V) | its own constants read through ttf-parser 0.25; shapes with rustybuzz (V) | math module is private; output is a `Frame`; child frames may be inlined (V) | wasm cfgs exist (V); browser use is A | 240 (V) | only through a whole typst compilation with a `World` (V for the API) |
| formulary 0.3.0 (2026-09-25) | new, 1 star (V) | MIT OR Apache-2.0 (V) | yes: its own parser on read-fonts 0.43, skrifa 0.46, harfrust 0.13 (V) | flat `Layout { width, ascent, descent, items }`; no per-node box (V, spot-checked `layout.rs:49`) | pure Rust, no I/O (V README); no CI (A) | 16 (V) | display list only; boxes need a fork |
| latex-rust 3.0.1 (2026-10-15) | very active (V) | MIT OR Apache-2.0 (V) | yes: its own parser on skrifa `=0.39.0` and read-fonts 0.36; STIX Two embedded (V) | nested `MathBox { width, height, depth, italic, shift, content }` with exact rationals; no node or cell id (V) | no evidence (A) | 10 (V) | possible |
| RaTeX (`ratex-layout` 0.1.15, 2026-10-16) | active, 1548 stars (V) | MIT (V, spot-checked) | no: KaTeX metrics hardcoded; big delimiters come out as SVG paths (V) | semantic `LayoutBox` with Fraction, SupSub, Radical, OpLimits, Accent, LeftRight and Array; no source span (V) | `ratex-wasm` crate and CI (V) | 36 (V) | tied to the KaTeX fonts |
| mathtex 0.2.0 (2026-09-27) | last push 2026-07-30 (V) | MIT OR Apache-2.0; the generated core uses TeX-family licences (V) | through XeTeX logic, with rustybuzz and ttf-parser (V) | `LayoutNode { id, origin, bounds, primary_source, kind }`: the only candidate with ids and source spans (V) | README says it runs on wasm32, and there is a web demo (V) | 19 (V) | XeTeX translated with c2rust: unsafe code, needs a TeX format file |

**Other crates.**
- pulldown-latex 0.8.0 (MIT): a parser that writes MathML, 2 packages. math-core 0.9.0: LaTeX to MathML, 24 packages. katex-rs 0.3.0: outputs HTML. None lays out (V).
- katex 0.4.6 runs KaTeX JavaScript in quick-js or duktape, or in the host's JavaScript on wasm. mathjax-svg-rs uses QuickJS-ng. Neither fits (V).
- waterui-math (201 packages), iced_math (99) and mathra (14) all read the MATH table through ttf-parser. All three output flat item lists (V).
- Blitz, parley 0.12, swash 0.2.10 and skrifa 0.44 have no MATH layout (V, grep).
- latex-rust's `rust-version = "1.99"` is met by the local rustc 1.99.1 (V, `rustc --version`). The project's pinned toolchain was not checked (A).

#### Q2. MATH-table support in the font crates we link

- **read-fonts 0.41.0: no MATH parser (V).**
  - `path@elsewhere@src/tables/` has no `math.rs`.
  - The only mention of `MATH` is a byte-range slot in the private table cache (`data/generated/generated_tables.rs:157,328`).
  - Raw bytes are reachable through `FontRef::table_data(Tag)` (`path@elsewhere@read-fonts-0.41.0/src/lib.rs` line 399).
- **skrifa 0.44.0: no math API (V).** A grep for MathConstants, MathVariants and MathGlyphInfo returns nothing. skrifa 0.49.0 has none either (V, grep).
- **read-fonts 0.45.0 and later have a full MATH module (V).** 0.45.0 was released 2026-10-03, and 0.42.0 to 0.44.0 have none.
  - Files: `path@elsewhere@src/tables/math.rs`, 1078 lines; `generated/generated_math.rs`, 2505 lines.
  - Tables covered: MathConstants; MathGlyphInfo with italics correction, top accent attachment, extended shapes and MathKernInfo; MathKern; MathVariants with MathGlyphConstruction, variant records, GlyphAssembly and part flags.
  - Helpers include `constant()`, `correction()`, `attachment()`, `kern(glyph, corner)`, `kerning(correction_height)`, `glyph_construction(...)`, `*_for_ppem` device-table helpers, and `has_swapped_min_heights()` (a Cambria quirk).
- **The cost of using read-fonts 0.45 or later (V).**
  - parley 0.12.0 pulls fontique 0.12.0, which requires read-fonts `^0.41.0`. harfrust 0.12.0 requires `^0.41.0` too.
  - Adopting read-fonts 0.45 or later now would therefore link two copies of read-fonts, until parley moves to a newer read-fonts.
  - That a later parley will move is A.
- **ttf-parser 0.25.1: complete MATH parsing (V, `path@elsewhere@src/tables/math.rs`, 938 lines).**
  - `Constants` has an accessor for every constant.
  - `GlyphInfo` has italic corrections, top accent attachments, extended shapes and kern infos.
  - `Kern` exposes `count`, `height(i)` and `kern(i)`. The lookup by height is left to the caller.
  - `Variants` has `min_connector_overlap`, vertical and horizontal `GlyphConstructions`, `GlyphAssembly`, `GlyphPart` and `PartFlags::extender`.
  - Values are `MathValue { value: i16, device: Option<Device> }`.
  - `math::Table::parse(&[u8])` is public, so it can parse the bytes that read-fonts 0.41 `table_data` returns, with no ttf-parser `Face`.
  - Feature: `opentype-layout`, which is on by default. Dependencies: only `core_maths`, which is optional. Licence: MIT OR Apache-2.0.
- **Where ttf-parser sits in our tree (V, `cargo tree`).**
  - It is linked today only through winit → sctk-adwaita → ab_glyph on native Linux.
  - `cargo tree -i ttf-parser --target wasm32-unknown-unknown` prints nothing. It is not in rootpoint-render.
  - Using it means adding a direct dependency. That dependency is small.
- **Shaping.** None of the crates applies the `ssty` substitution for us. A layout must run GSUB itself, through harfrust or parley features. Every measured font has `ssty` (V, see Q4). Whether parley can apply it per glyph run is not checked (A).

#### Q3. Linearity of the OpenType MATH data

**Confirmed (V, from the spec text).**
- "These values are expressed in font design units": MathValueRecord, and every constant except the two percentages.
- scriptPercentScaleDown and scriptScriptPercentScaleDown are percentages.
- radicalDegreeBottomRaisePercent is a percentage.
- delimitedSubFormulaMinHeight and displayOperatorMinHeight are UFWORD, which is design units.
- MathKern correctionHeight and kernValues are in design units.
- minConnectorOverlap, advanceMeasurement, startConnectorLength, endConnectorLength and fullAdvance are in design units.
- So, ignoring device tables, a layout computed in design units and multiplied by size/upem is linear in the size.
- The stretchy-variant choice compares a target measured in design units with advances in design units. The choice is therefore scale-invariant.
- The assembly algorithm adds extenders a whole number at a time and distributes overlaps against a design-unit target. That is scale-invariant too.
- The `ssty` choice depends on the script level, not on the size.

**Exceptions and caveats.**
1. **Device tables (V, spec).** A MathValueRecord may point to a device table: "corrections to a given value at particular display resolutions … for several different PPEM sizes". These corrections are not linear.
   - The spec says an engine "may limit the number of device corrections … or may ignore them altogether".
   - Measured (V, `measure.py`): no font among the seven has a device table on any of the 51 constants.
   - STIX Two Math has device tables on 1 italic correction and 3 top accent attachments. The other fonts have none in their glyph info.
   - Ignoring device tables makes all seven fonts linear.
2. **spaceAfterScript (V, spec).** It notes that "some implementations could use a constant value, such as 0.5 pt, for all text sizes". That is an engine choice. Taking the font value keeps linearity.
3. **Engine choices outside the font (A).** These break linearity if they are made: snapping rules or positions to pixels, a minimum rule thickness in pixels, hinting, and optical-size font switching.
   - None of the measured fonts is variable: no `fvar` in their table lists (V).
   - With unhinted glyphs and no snapping, scaling is exact up to floating-point rounding (A, not measured).
4. **Text shaped by parley inside math.** Linearity of parley's own layout under scaling is A.
5. **MathKern is ambiguous in the spec.** On notofonts/math#99, Khaled Hosny wrote that the spec describes the MathKernInfo algorithm incorrectly (MicrosoftDocs/typography-issues#1147), and that LuaTeX and XeTeX/Word disagree (V, the comment). The contents of #1147 were not read (A). This bears on correctness, not on linearity.

**Conclusion.** Fitting an equation to its area is one layout plus one multiplication, provided device tables are ignored and nothing is snapped. This holds for the measured fonts.

#### Q4. Math fonts (measured with `scripts/measure.py`, V)

Every font measured has a MATH table, version 1.0, and unitsPerEm 1000.

| font, file | file size (bytes) | outlines | glyphs | MATH size (bytes) | licence (name ID 13 or 0) |
| --- | --- | --- | --- | --- | --- |
| Noto Sans Math 3.000, unhinted TTF | 657,440 | glyf | 5014 | 20,426 | OFL 1.1 |
| Noto Sans Math 3.000, unhinted OTF | 479,308 | CFF | 5014 | 20,426 | OFL 1.1 |
| Noto Sans Math 3.000, Google Fonts TTF (ttfautohint) | 1,015,396 | glyf | 5130 | 20,200 | OFL 1.1 |
| Fira Math 0.3.4 | 179,840 | CFF | 2094 | 6,912 | OFL 1.1 |
| STIX Two Math 2.13 b171 OTF | 838,652 | CFF | 6760 | 27,408 | OFL 1.1 |
| (STIX Two Math 2.12 b168a, Google Fonts TTF) | 1,517,976 | glyf | 6760 | 27,408 | OFL 1.1 |
| Libertinus Math 7.051 | 588,848 | CFF | 4463 | 10,202 | OFL 1.1 |
| Latin Modern Math 1.959 | 733,736 | CFF | 4802 | 25,280 | GUST Font License (copyright string) |
| New Computer Modern Math (4.0) | 1,306,268 | CFF | 8603 | 41,170 | GUST Font License (copyright string); CTAN lists gpl3+fe and gfl |
| New CM Sans Math (4.0), extra | 1,183,540 | CFF | 8937 | 38,970 | GUST Font License |

For comparison, our Noto Sans Regular has 3317 glyphs and no MATH table. It covers none of the math blocks below (V).

**cmap coverage.** Columns are code points covered over code points tested. Reserved holes are excluded from the counts. The script and fraktur blocks have holes because Unicode assigns those letters in Letterlike Symbols.

| font | Greek 0391–03C9 | math Greek 1D6A8–1D7C9 | Operators 2200–22FF | Arrows 2190–21FF | Supp. Ops 2A00–2AFF | Math Alnum 1D400–1D7FF | bold / script / fraktur / double-struck |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Noto Sans Math | 49/56 | 290/290 | 256/256 | 112/112 | 256/256 | 994/994 | 52/52, 41/41, 47/47, 45/45 |
| Fira Math | 56/56 | 174/290 | 158/256 | 112/112 | 3/256 | 438/994 | 52/52, 0/41, 0/47, 45/45 |
| STIX Two Math | 56/56 | 290/290 | 256/256 | 103/112 | 256/256 | 994/994 | all complete |
| Libertinus Math | 56/56 | 278/290 | 220/256 | 112/112 | 23/256 | 982/994 | all complete |
| Latin Modern Math | 49/56 | 290/290 | 237/256 | 79/112 | 26/256 | 945/994 | 52/52, 18/41, 47/47, 45/45 |
| New CM Math | 49/56 | 290/290 | 256/256 | 112/112 | 256/256 | 994/994 | all complete |

The 7 Greek code points missing in the 49/56 fonts are the accented letters U+03AA to U+03B0. They do not matter for math (V). Noto Sans Math lacks one printable ASCII character, the backtick (V).

**Stretchy constructions.** Each cell gives the number of vertical variants, then "+A" with the number of parts when a glyph assembly exists.

| font | ( | [ | { | \| | √ | ∫ | ∑ |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Noto Sans Math | 13 +A3 | 13 +A3 | 13 +A5 | 13 +A2 | 6 +A3 | 2 +A3 | 2, no A |
| Fira Math | 16 +A3 | 16 +A3 | 16 +A5 | 16 +A2 | 16 +A3 | 2, no A | 2, no A |
| STIX Two Math | 13 +A3 | 13 +A3 | 13 +A5 | 1 +A2 | 4 +A3 | 2 +A3 | 2, no A |
| Libertinus Math | 13 +A3 | 13 +A3 | 13 +A5 | 13 +A2 | 8 +A3 | 2, no A | 2, no A |
| Latin Modern Math | 8 +A3 | 8 +A3 | 8 +A5 | 8 +A3 | 5 +A3 | 2, no A | 2, no A |
| New CM Math | 8 +A3 | 8 +A3 | 8 +A5 | 8 +A3 | 6 +A3 | 2 +A3 | 2, no A |

∑ has no assembly in any font. It has a text form and a display form.

**Other measured data (V).**
- Math kerning (MathKernInfo): STIX has 233 glyphs, New CM 668, New CM Sans 516. Noto, Fira, Libertinus and Latin Modern have 0.
- Script scale-down (scriptPercentScaleDown / scriptScriptPercentScaleDown):

  | font | script | scriptscript |
  | --- | --- | --- |
  | Noto Sans Math | 60 | 50 |
  | Fira Math | 72 | 58 |
  | STIX Two Math | 70 | 55 |
  | Libertinus Math | 80 | 60 |
  | Latin Modern Math and New CM Math | 70 | 50 |

  The spec suggests 80 / 60.
- GSUB features: all six main fonts have `ssty`. STIX alone has `flac`. Noto, Fira, Latin Modern, New CM and STIX have `dtls`. Libertinus has no `dtls`.

**Visual match with Noto Sans (from each font's own documentation).**
- Noto Sans Math: part of the Noto family, sans.
  - Issue #54 (closed 2024-04-25) asked to split it into a serif and a sans font. A reply says that "going all in on sans-serif was a conscious choice".
  - On #77, Khaled Hosny wrote that the non-alphabetic glyphs are over 3262 of 5104 (V).
- Fira Math: "a sans-serif font … a fork of FiraSans, FiraGO" (V, README). It matches Fira, not Noto.
- STIX Two: "A Fresh Take on Times Roman" (V, README). Serif.
- Libertinus Math: "derived from the serif family", forked from Linux Libertine (V, README).
- Latin Modern and New CM: Computer Modern designs (V, CTAN caption "Computer Modern fonts …"). New CM also ships a Sans Math, measured above.

**Noto Sans Math, explicitly.**
- It has a usable MATH table (V): 20,426 bytes.
  - Constants are present with no device tables.
  - 528 italic corrections, 1761 top accent attachments and 1150 extended shapes.
  - 219 vertical and 29 horizontal constructions.
  - All seven test characters stretch.
- History: a MATH table was first added in early 2023 and reworked by Khaled Hosny for 3.000 (V, notofonts/math#87, comment by simoncozens).
- Latest release: v3.000, 2024-06-05. There is no release since (V, releases API). Commits continue; the last, on 2026-10-08, is build tooling (V).
- Only a Regular weight exists. There is no bold and no variable weight. #105 (variable weight) and #67 (bold math) are open (V).
- Known limitations, from open issues (V):
  - #98: subscripts read too small. The measured script scale is 60/50. That this value causes the complaint is my inference (A).
  - #58: `ssty` variants are too bold.
  - #99: no math kerning. The maintainer avoided cut-ins because engines disagree (see Q3).
  - #81: Greek is not synced with the current Noto Sans Greek.
  - #91: question mark style differs from Noto Sans.
  - #101: circled operators are too large.
  - #61: script alphabet style.
  - #71: alpha and kappa are close to Latin a and k.
  - #100: Word does not list it as a math font.
- Fixed in the source but not released (V, measured):
  - #94, closed 2024-10-25: the U+0305 extender part has `endConnectorLength 0`.
  - Both the v3.000 release and the Google Fonts build still have part 1 with `start 417 end 0`, flags extender (`scripts/hparts.py`).
  - Simon Cozens and Khaled Hosny noted that SILE, Firefox and LuaTeX tolerate this and Typst does not. An engine must do the same.

#### Q5. LaTeX command → Unicode tables

These are the subagent's counts. I re-ran the unicode-math count: `grep -c '^\\UnicodeMathSymbol'` returns 2448 (V).

| source | licence | entries | format | math class per entry |
| --- | --- | --- | --- | --- |
| unicode-math `unicode-math-table.tex` v0.8r (2023-08-13) | LPPL 1.3c or later (README and LICENSE) (V) | 2448 (V) | `\UnicodeMathSymbol{"USV}{\cmd}{\mathclass}{description}`, one per line | yes: alpha 1043, rel 594, ord 455, bin 175, op 62, open 33, close 30, accents 44, and others |
| KaTeX `path@elsewhere@src/symbols.ts` v0.19.1 | MIT (V) | 612 `defineSymbol` calls, 490 of them math-mode `\name` entries (V) | JS calls | yes (`group`) |
| MathJax v4.1.3, `BaseMappings.ts` and `AmsMappings.ts` | Apache-2.0 (V) | base 256, ams 248 (V) | TS maps keyed by mi or mo | no: the class comes from the MathML operator dictionary, keyed by character |
| LyX `lib/symbols` | GPL v2 or later (repository COPYING) (V) | 1602 entry lines; 632 carry `&#x…;` (V) | ad hoc lines with `iffont` conditionals and `\def` macros | yes, for symbols |
| W3C `unicode.xml` (efed20d) | W3C Software Notice and License (V) | 41558 characters; 2334 with `<latex>` starting `\`; 2614 with `<mathlatex>`, 2437 of them set to unicode-math (V) | XML | partly: Unicode MathClass, and a TeX-like `@type` |
| WHATWG entities.json | CC BY 4.0, code portions BSD-3 (V) | 2231 keys (V) | JSON of HTML entity names, not LaTeX | no |
| typst `codex` 0.3.0 | Apache-2.0 (V) | 1223 symbols with typst names (V) | own format | no LaTeX names at all (V, grep) |
| unicodeit 0.2.1 (crate) | `MIT AND LPPL-1.3c` (V, SPDX header) | 4257 pairs, 3003 bare `\name` (V) | Rust slice | no |
| pulldown-latex 0.8.0 | MIT (V) | 570 match arms (V) | Rust `match` code | yes, by constructor |
| ReX (ReTeX/ReX) `path@elsewhere@fonts/stix/src/symbols.rs` | README claims MIT and Apache; no LICENSE file (V) | 1405 (V) | generated from unicode-math-table.tex | yes |
| MathLive `path@elsewhere@src/latex-commands/symbols.ts` | MIT (V) | 501 (V) | TS rows | per block, not per row |
| mdwright-latex 0.1.3 | MIT OR Apache-2.0 (V) | 244 (V) | Rust data | its own categories |
| latex-rust 3.0.1 `data/symbols.tsv` | MIT OR Apache-2.0 (V) | 475 (V) | TSV | from code, not the TSV |

- The crates `latex2unicode`, `unicode-math` and `unicode_math` do not exist on crates.io (V).
- unicode-math is the upstream of ReX's table and of unicode.xml's `mathlatex set="unicode-math"` column (V, file headers). It is the largest table with a class per entry.
- rootpoint declares no licence, so compatibility is not judged here.

#### Q6. MathLive compared with LyX (from the subagent's reading of mathlive 8d148b4)

**MathLive's model (V).**
- An `Atom` has fixed named branches: body, above, below, subscript and superscript.
- Arrays use `[row, col]` cell branches.
- Each branch starts with a sentinel atom of type `'first'`.
- The caret is one integer offset into a post-order flattening of the tree. A selection is a list of `[start, end]` ranges.
- Up and down movement uses rendered x positions, and creates a missing above or below branch.
- Explicit `\placeholder{}` atoms exist. Templates use `#?`, `#@` and `#0`. Tab leaps to the next placeholder.
- `\` opens a latex group with autocompletion. It closes on Space, Enter or Tab, and the whole text is parsed, arguments included.
- About 259 inline shortcuts exist, such as `alpha` and `>=`. A shortcut can test its left context. Backspace undoes a replacement back to the typed keys.

**What differs from LyX (docs/grounding/lyx-nested-text.md §3) and might matter.**
1. The caret is one flat integer over a post-order walk, not LyX's stack of `(idx, pos)` slices.
2. Scripts hang as branches on the previous atom. LyX instead wraps the nucleus in an `InsetMathScript`.
3. Empty cells hold explicit placeholder atoms that serialise to LaTeX. LyX draws a box and stores nothing.
4. `\cmd` entry stays open until Space, Enter or Tab, offers completion, and parses arguments. LyX's `InsetMathUnknown` closes on the first character that is not a letter.
5. Right arrow from the end of a numerator probably goes to the denominator, by the offset order. This was not traced (A).
6. Inline shortcuts use left-context tests and undo to the literal keys. LyX has a narrower counterpart: `lib/autocorrect`, applied by `path@elsewhere@src/mathed/MathAutoCorrect.cpp`. I verified it (V, local LyX clone 64141cf): 314 rules that combine a symbol and one character into a new symbol, for example `<` then `=` gives `\leqslant`. These are not word shortcuts.
7. Vertical movement depends on rendered geometry.

#### What transfers, and the property each depends on

- **Writing our own layout on the MATH table** transfers from typst, formulary, latex-rust and ReX, which all do it.
  - It depends on a MATH parser reachable from wasm32. ttf-parser 0.25.1 gives one, fed the bytes from read-fonts 0.41 `table_data`. read-fonts 0.45 or later gives one, at the cost of two read-fonts copies until parley moves (V).
  - It depends on applying `ssty` through GSUB (A).
- **Borrowing a box-tree engine (KenyC/ReX, latex-rust, RaTeX)** depends on adding node and cell identity to its boxes. None has it (V). Each would need a fork for caret mapping.
  - RaTeX also depends on KaTeX fonts, not on the MATH table (V).
  - ReX's `MathFont` trait would need a backend on our font stack (V).
- **"Fit to area = one layout × k"** depends on these properties:
  - No device-table corrections. The measured fonts have none on constants; STIX has 4 on glyph info.
  - No pixel snapping.
  - Unhinted glyphs.
  - A non-variable math font. None of the measured fonts is variable (V).
- **Noto Sans Math as the font** depends on the following:
  - Accepting a scale-down of 60/50.
  - Having no math kerning.
  - Carrying a Greek out of sync with Noto Sans.
  - Tolerating the U+0305 extender with `endConnectorLength 0`.
  - Having no bold weight.
  - It costs 479–657 KB (unhinted OTF or TTF) against the current 2.1 MB of Noto Sans (V).
  - Its coverage of the blocks tested is complete (V).
- **The command table from unicode-math (2448 entries, with classes)** depends on accepting LPPL 1.3c data. Alternatives with a class per entry: KaTeX (MIT, 490), pulldown-latex (MIT, 570 as code), or MathLive (MIT, 501, class per block) (V).
- **MathLive's explicit placeholders and its `\cmd` group with arguments** depend on the save format accepting a placeholder token. LyX's empty-cell model avoids that token.

#### Unverified (A claims that matter)

- No candidate in Q1 was built for wasm32. Support is claimed by a README or demo for RaTeX, mathtex and katex-rs. It is inferred from pure-Rust dependencies for KenyC/ReX, formulary and latex-rust.
- typst math cannot be called without a whole `World` and compilation; this was not tested.
- That parley will move to read-fonts 0.45 or later, and when.
- Whether parley or harfrust can apply `ssty` to one glyph run as our layout needs.
- Exact float equality of scaled layouts: argued, not measured.
- That Noto Sans Math's 60% script scale is the cause of issue #98.
- The contents of MicrosoftDocs/typography-issues#1147.
- MathLive right arrow from the end of a numerator.
- formulary's WPT pass rate (from its README).
- The project's pinned Rust toolchain against latex-rust's MSRV 1.99.

#### Decided

- Q1 dependency trees were counted with `cargo tree` for wasm32 in the subagent's own throwaway project, not by walking the crates.io API. Ground: it resolves target cfgs, and it runs nothing from a downloaded tree.
- STIX Two Math was measured from tag v2.13b171. Ground: the repository's master branch no longer holds built binaries, and the release has no assets. The Google Fonts build is 2.12 b168a and is reported beside it.
- New CM Sans Math was measured although not asked for. Ground: it is a sans MATH font with full coverage, relevant to matching Noto Sans.

#### Met outside the task

- read-fonts 0.41, which parley 0.12 pins, has no MATH table, and read-fonts 0.45.0 and later do (V). This bears on the design's cost: route to the design discussion, as the current work.
- The Q5 subagent reports that ReX's symbol table is derived from the LPPL unicode-math table and that ReX ships under an MIT and Apache claim with no LICENSE file. Reported as a fact only.
- LyX `lib/autocorrect` exists, and §3 of lyx-nested-text.md does not mention it (V). It may be worth one line in that grounding file. Not edited: left to the session.

## 3. The survey's subagent on command tables and MathLive

#### Q5: tables from LaTeX command names to Unicode

All the downloads are under `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/q56/`, one subdirectory per source. The counting scripts are in `/home/teo/.cache/claude-tmp/claude-1000/-home-teo-dev-rootpoint/6b37e113-b947-45d6-a2c8-446607e98357/scratchpad/survey/scripts-q56/` (`mjcount.py`, `ux.py`). In the table, "class" means a TeX atom class on each entry.

| source | licence (from its own text) | entries (how counted) | format | class per entry |
|---|---|---|---|---|
| **unicode-math** `unicode-math-table.tex`, v0.8r 2023/08/13 (CTAN zip; same line count as GitHub 184a23b) | LPPL 1.3c or later. Its README says so in "LICENCE", and the `LICENSE` file is LPPL 1.3c. V | 2448: `grep -c '^\\UnicodeMathSymbol'`. V | one line per entry, `\UnicodeMathSymbol{"USV}{\cmd}{\mathclass}{description}`. V | **Yes.** Counts: alpha 1043, rel 594, ord 455, bin 175, op 62, open 33, close 30, accent 25, accentwide 10, botaccentwide 7, punct 3, fence 3, over 3, under 3, botaccent 1, accentoverlay 1. These sum to 2448. V |
| **KaTeX** `path@elsewhere@src/symbols.ts` (now TypeScript), v0.19.1, commit 8000fbd, 2026-10-23 | MIT, "Copyright (c) 2013-2020 Khan Academy". V | 612 `defineSymbol(` calls. Of these, 490 are math-mode entries with a `\name` (regex grep `^defineSymbol\(math, …, "\\`). Letters and digits are added by loops. V | JS calls: `defineSymbol(mode, font, group, replace, name, acceptUnicodeChar)`. V | **Yes**, as a `group`. Counts for the 490: rel 236, textord 88, bin 66, mathord 32, op 20, open 13, close 13, accent 11, spacing 5, inner 4, punct 2. Big operators also get metadata in `path@elsewhere@src/functions/op.ts`. V |
| **MathJax** v4.1.3, commit fb98717, 2026-07-03. The tables are in `ts/input/tex/base/BaseMappings.ts` and `ts/input/tex/ams/AmsMappings.ts`; other packages have `*Mappings.ts` files under `ts/input/tex/*/` | Apache-2.0 (`LICENSE`). V | Character and delimiter maps (counted with `mjcount.py`): base 256 (lcGreek 30, ucGreek 11, mathchar0mi 31, mathchar0mo 138, mathchar7 6, delimiter 40); ams 248 (AMSsymbols mathchar0mo 194, mathchar0mi 45, other 9). V | TS objects `name: 'char'` or `name: ['char', {attrs}]`, in maps named after the MathML element (mi or mo). V | **No.** The map gives only mi or mo. The TeX class comes from the MathML operator dictionary, `ts/core/MmlTree/OperatorDictionary.ts` (`REL: OPDEF(5,5,TEXCLASS.REL)` and so on), keyed by character. V |
| **LyX** `lib/symbols`, local git.lyx.org clone at e7ffd8d (2026-10-25) | The file has no header. The repository `COPYING` is GPL v2 "or (at your option) any later version". V | 1602 entry lines: 887 plain + 715 `\def`, comments, blanks and `iffont/else/endif` excluded. 632 lines carry an `&#x…;` code point. V | Ad hoc, as the file's own header says: `name font-or-kind TeXchar-in-font symbol-font-char class &#xUSV; [hiddensymbol] [requires]`, e.g. `alpha cmm 174 97 mathord &#x3B1;`. Also `\def\name{…} class &#x…; requires`, and conditional blocks `iffont <font> … else … endif` (7 `iffont`). Kinds that are not symbols: decoration, dots, big, matrix, space, font. V | **Yes** for symbols (`mathord/mathbin/mathrel/mathop/mathopen/mathclose/mathpunct/mathalpha`). Lines carrying a class: rel 314, bin 129, ord 129, op 73, open 19, close 19, alpha 11, punct 3. V |
| **W3C `unicode.xml`** (w3c/xml-entities, efed20d, 2026-09-28) | Header: "Copyright David Carlisle 1999-2025 … W3C Software Notice and License" (2002 version URL). V | 41558 characters. Of these, 2334 have a `<latex>` value starting with `\`, and 2614 have `<mathlatex>` (2437 with `set="unicode-math"`). Counted with `ux.py`. V | XML, one `<character id="U02264">` with `<latex>`, `<mathlatex set=…>`, `<AMS>`, `<IEEE>`, entity names and more. V | **Partly.** `unicodedata/@mathclass` is the Unicode MathClass letter (R, B, N, A, L, C, O, P, D, …). The `character/@type` attribute is TeX-like: relation, binaryop, opening, closing, large, punctuation, diacritic, alphabetic, normal, other. V |
| **WHATWG `entities.json`** (Last-Modified 2025-11-12) | The HTML standard says "Creative Commons Attribution 4.0 … portions … incorporated into source code … BSD 3-Clause" (acknowledgements page, read 2026-10-25). V | 2231 keys, 2125 distinct names (Python count). V | JSON `"&le;": {codepoints, characters}`. These are HTML entity names, not LaTeX names. V | No. V |
| **typst `codex`** 0.3.0 (crate, 2026-06-04; repo 42bea3a) | Apache-2.0. V | `sym.txt` has 1223 lines that carry a character (`awk 'NF>=2'` on non-comment lines). V | Typst's own dotted names (`paren.l.flat`). V | **No LaTeX table:** `grep -rli latex` over the repository returns nothing. No class field either. V |
| **unicodeit** 0.2.1 (crates.io, data from unicodeit 0.7.5) | `MIT AND LPPL-1.3c`. `path@elsewhere@src/data.rs` carries the SPDX header `MIT AND LPPL-1.3c`. V | `REPLACEMENTS` has 4257 pairs, 3003 of them a bare `\name`. V | `&[(&str,&str)]` | No. V |
| **pulldown-latex** 0.8.0 (2026-07-28) | MIT. V | 570 match arms in `path@elsewhere@src/parser/primitives.rs` map names to a char kind, e.g. `"leq" => relation('≤')`. V | Rust `match` code, not a data table. V | **Yes, by constructor:** relation 250, ordinary 184, binary 85, large_op 35, multirelation 16. There are also `is_binary`/`is_relation` char sets in `path@elsewhere@src/parser/tables.rs`. V |
| **ReX** (github ReTeX/ReX, 7bd4efc, 2020-07-15; not on crates.io: the crates.io `rex` is an unrelated project by peterkelly) | The README says "both the MIT license and the Apache License … portions covered by various BSD-like licenses". The repository holds no LICENSE file. V | 1405 active entries plus 101 commented out, in `path@elsewhere@fonts/stix/src/symbols.rs`. V | Generated from unicode-math-table.tex (the file header says so): `"lparen" => Symbol{unicode, atom_type}`. V | **Yes** (AtomType): Relation 588, Alpha 475, Binary 173, Operator 60, Open 33, Close 29, Accent 27, and others. V |
| mdwright-latex 0.1.3 | MIT OR Apache-2.0. V | 244 `CommandEntry` rows in `path@elsewhere@src/registry.rs` ("MathJax-style"). V | Rust data | Its own categories (Relation, BinaryOperator, LargeOperator, Delimiter, Greek, …). V |
| latex-rust 3.0.1 | MIT OR Apache-2.0. V | 475 rows in `data/symbols.tsv`. V | TSV: glyph, category, kind, latex, description | The class is not in the TSV; it comes from `symbol_atom_kind()` in `path@elsewhere@src/atoms.rs`. V |

- **Crates that do not exist:** `latex2unicode`, `unicode-math` and `unicode_math` return no crate on crates.io (API, 2026-10-25). V
- **MathLive's own table:** `path@elsewhere@src/latex-commands/symbols.ts` (MIT) has 501 `['\\name', 0xUSV]` rows. The class is set once per `defineSymbols(...)` block, not per row. V
- **Shared origin:** unicode-math's table is the upstream of ReX, and unicode.xml's `mathlatex set="unicode-math"` column mirrors it. That makes it the largest table with a per-entry class, and its own README states LPPL 1.3c.

#### Q6: MathLive (MIT, arnog/mathlive v0.111.1, commit 8d148b4, 2026-10-09)

**Model.**
- `Atom` (`path@elsewhere@src/core/atom-class.ts`) has `type`, `command`, `value`, `parent`, `parentBranch` and `_branches`. V
- Named branches are fixed: `body, above, below, subscript, superscript`. Arrays use cell branches `[row, col]`. V
- Every branch starts with a sentinel atom of type `'first'`. V

**Caret.**
- The caret is one integer `Offset` into `root.children` (`model.at`, `offsetOf = atoms.indexOf`). `root.children` is a post-order flattening of the whole tree: each branch's atoms come before their parent, in the branch order above (`get children()`, line 811). V
- A selection is a list of `Range = [start, end]` over these offsets (`path@elsewhere@src/public/core-types.ts`). V
- Left and right arrows step to `nextValidPosition` over the offsets (`path@elsewhere@src/editor-model/commands.ts`, `move`). V
- Up and down climb to an ancestor in `below` or `above`, or to an array cell, create the target branch if it is missing, and pick the atom closest in x through screen geometry (`moveToClosestAtomVertically`, `getClosestAtomToXPosition`). V

**Placeholders.**
- An explicit `PlaceholderAtom` serialises as `\placeholder{}` (`path@elsewhere@src/atoms/placeholder.ts`). V
- Insert templates use `#?` (placeholder), `#@` (implicit argument: the selection or the preceding ord atoms) and `#0` (the selection) (`mode-editor-math.ts` ~192-240). After an insertion the caret selects the first placeholder. V
- Tab ("leap") jumps to the next placeholder, prompt or empty branch (`commands-move.ts` `leap`). V
- Typing while a placeholder is selected deletes the placeholder first (`keyboard-input.ts`, issue #2572). V

**Typing `\cmd`.**
- `\` runs `switchMode latex` (`path@elsewhere@src/editor/keybindings-definitions.ts`). This inserts a `LatexGroupAtom` whose body is one `LatexAtom` per typed character, in mode `'latex'` (`path@elsewhere@src/atoms/latex.ts`). V
- `updateAutocomplete` shows completions as grey inline atoms (`isSuggestion`) and in a popover. A name that matches no command is marked `isError` (`path@elsewhere@src/editor-mathfield/autocomplete.ts`). V
- Keys that finish the entry: Space, Return and Enter accept all; Tab accepts the suggestion; shift+Escape rejects. V
- On accept, `complete()` removes the group and inserts its whole text as LaTeX, with `selectionMode: 'placeholder'`. The group can therefore hold arguments, as in `\frac{a}{b}`. V

**Inline shortcuts.**
- Defined in `path@elsewhere@src/editor/shortcuts-definitions.ts`, e.g. `'alpha': '\\alpha'`, `'>=': '\\ge'`, `'sqrt': '\\sqrt{#?}'`. A top-level-key regex grep counts 259 keys, an approximate count. V
- A shortcut may carry an `after:` context test on the left sibling, e.g. `in` fires only after `nothing+letter+closefence`. V
- The keyboard code keeps a keystroke buffer with a model state for each key, and matches the longest suffix. It first inserts the literal keys and takes an undo snapshot, then reverts and inserts the replacement. Backspace right after a replacement therefore undoes it back to the typed letters (`path@elsewhere@src/editor-mathfield/keyboard-input.ts` ~92-475). V

**Compared with LyX** (from `path@rootpoint@docs/grounding/lyx-nested-text.md` §3):
1. The caret differs. LyX uses a stack of slices, `(idx, pos)` per level. MathLive uses one flat integer offset over a post-order walk, with a `'first'` sentinel per branch. Ranges are integer pairs, and column selection is several ranges.
2. Scripts differ. LyX wraps the previous atom as the nucleus of a new `InsetMathScript`. MathLive hangs `subscript`/`superscript` branches on the previous atom itself, or on an adjacent `subsup` atom when that atom cannot take limits (`moveToSuperscript`).
3. Empty slots differ. LyX draws a box for an empty cell and stores no node for it. MathLive stores explicit `\placeholder{}` atoms, which serialise to LaTeX, are selected on insertion, and are what Tab leaps to.
4. Command entry differs. LyX `InsetMathUnknown` closes on the first character that is not a letter. MathLive's latex group closes only on Space, Enter or Tab, offers completion, and parses the full text, arguments included.
5. Right arrow at the end of a numerator: LyX leaves the fraction. In MathLive the offset order (above, then below) puts the denominator next. I did not trace whether `isValidPosition` skips it.
6. Inline shortcuts with left-context tests and literal-keystroke undo have no counterpart in LyX §3.
7. Vertical movement in MathLive uses rendered x coordinates, and creates the missing above or below branch.

#### Unverified
- The LPPL part of unicodeit's data comes from Günter Milde's `unimathsymbols.txt` or from unicode-math. A, from memory; its README does not say.
- LyX has a separate math autocorrect feature (`path@elsewhere@src/mathed/MathAutoCorrect.cpp`) that is the nearest equivalent of MathLive's inline shortcuts. A; this would affect point 6 of the comparison.
- Whether right arrow from the end of a numerator actually lands in the denominator in MathLive. Inferred from the flattening order, not run.
- The 259 count of inline shortcuts is approximate: a regex over two-space-indented keys.
- Whether the LyX clone I read (e7ffd8d) descends from 64141cf, the commit the grounding document cites. `git merge-base --is-ancestor 64141cf e7ffd8d` exited with code 1, which normally means "not an ancestor". The clone may hold only part of the history (`--filter=blob:none`; the server warned that filtering was ignored), so I cannot tell that apart from missing history. The LyX counts above are for e7ffd8d.

#### Sources (all fetched or read 2026-10-25)
- unicode-math 0.8r (2023/08/13), CTAN `macros/unicodetex/latex/unicode-math.zip`; GitHub wspr/unicode-math 184a23b (2025-03-09)
- KaTeX/KaTeX 8000fbd (2026-10-23), v0.19.1
- mathjax/MathJax-src fb98717 (2026-07-03), v4.1.3
- LyX git.lyx.org clone e7ffd8d (2026-10-25), `lib/symbols`, `COPYING`
- w3c/xml-entities efed20d (2026-09-28), `unicode.xml`
- html.spec.whatwg.org/entities.json (Last-Modified 2025-11-12); the licence is on the acknowledgements page
- crates.io: codex 0.3.0, unicodeit 0.2.1, pulldown-latex 0.8.0, mdwright-latex 0.1.3, latex-rust 3.0.1; typst/codex 42bea3a (2026-10-20)
- ReTeX/ReX 7bd4efc (2020-07-15)
- arnog/mathlive 8d148b4 (2026-10-09), v0.111.1

#### Met outside the task
- `github.com/lyx-org/lyx` and `github.com/lyx/lyx` return 404 for `lib/symbols`, and git.lyx.org gitweb raw URLs return HTML. I used the existing local clone. This matches what the grounding document already notes. Dropped: already recorded there.
- The ReX table is derived from the LPPL unicode-math table but ships under an MIT/Apache claim with no LICENSE file in the repository. Reported as a fact only, per the brief; no compatibility judgement made.
- Several Rust crates on crates.io (fmd-math, latex-rust, mathtex, iced_math, katex-parser) are TeX math layout engines. Q5 did not ask about them, so I did not survey them; they may matter to another question of the survey.

## 4. The session's own measurements

**What the owner's thesis uses (V).** `github.com/teoboutin/thesis` at `53baab6`, its history one
squashed commit, `0aeed90` "squashing all work to save space". `pdffonts -f 60 -l 60 main.pdf`, on a
page with no figure, lists Linux Libertine (regular, bold, italic) for the text and, for the math,
CMMI12, CMMI8, CMR12, CMSY10, CMEX10 and esint10, which are Computer Modern. `main.lyx` has
`\font_math "auto"` and `\use_non_tex_fonts false`; the preamble,
`path@elsewhere@template/latex/UPSthesis_preamble.tex`, loads `\usepackage{libertine}`, which sets
the text font alone. The defense slides, `path@elsewhere@defense/defense.pdf`, set their text in
Open Sans, `\usepackage[default,scale=0.95]{opensans}` in `path@elsewhere@defense/defense.lyx`,
and their math in Computer Modern.

**ron 0.12.2 writes raw strings (V).** In a probe crate depending on `ron = "=0.12.2"`,
`ron::ser::to_string_pretty` writes `Math("\\frac{a}{b} + \\alpha")` with the default
`PrettyConfig`, and `Math(r#"\frac{a}{b} + \alpha"#)` with `PrettyConfig::escape_strings(false)`;
both read back equal, and so does a raw string written by hand. The setting applies to every string
of the file. A raw string is written only for a value holding a `"` or a `\`, by
`serialize_unescaped_or_raw_str` of ron 0.12.2's `path@elsewhere@src/ser/mod.rs`, read by the reviews
of the milestone; a value holding a line break and neither stays a quoted string, its break written
as is.

**The math commands each font lacks (V).** Two counts, by a script that reads each font's cmap:

- over the math commands of KaTeX v0.19.1 that map to one character: every
  `defineSymbol(math, …, "<char>", "\\<name>"` of `path@elsewhere@src/symbols.ts` whose replacement
  is one character, 458 of them. Libertinus Math 7.051 lacks 44; Latin Modern Math 29; Noto Sans
  Math 22;
- over the 2448 `\UnicodeMathSymbol` lines of unicode-math v0.8r: Libertinus Math lacks 787 (318
  of class mathrel, 261 mathord, 116 mathbin, 38 mathop, 17 mathalpha, 15 mathopen, 14 mathclose,
  and 8 accents and fences); Latin Modern Math 845; Fira Math 1379; Noto Sans Math 32; STIX Two Math
  and New Computer Modern Math none.

The 44 commands of KaTeX, with their characters, that Libertinus Math lacks: `\lessapprox` ⪅,
`\gtrapprox` ⪆, `\lnapprox` ⪉, `\gnapprox` ⪊, `\eqslantless` ⪕, `\eqslantgtr` ⪖, `\lesseqqgtr` ⪋,
`\gtreqqless` ⪌, `\precapprox` ⪷, `\succapprox` ⪸, `\precnapprox` ⪹, `\succnapprox` ⪺,
`\subseteqq` ⫅, `\supseteqq` ⫆, `\subsetneqq` ⫋, `\supsetneqq` ⫌, `\Subset` ⋐, `\Supset` ⋑,
`\bumpeq` ≏, `\Bumpeq` ≎, `\pitchfork` ⋔, `\multimap` ⊸, `\Cap` ⋒, `\Cup` ⋓, `\doublecap` ⋒,
`\doublecup` ⋓, `\barwedge` ⊼, `\veebar` ⊻, `\doublebarwedge` ⩞, `\divideontimes` ⋇,
`\leftthreetimes` ⋋, `\rightthreetimes` ⋌, `\intercal` ⊺, `\diamond` ⋄, `\triangleleft` ◃,
`\triangleright` ▹, `\bigcirc` ◯, `\blacklozenge` ⧫, `\checkmark` ✓, `\maltese` ✠, `\diagup` ╱,
`\diagdown` ╲, `\Finv` Ⅎ, `\Game` ⅁. Latin Modern Math's 29 include `\dashrightarrow` ⇢,
`\bigstar` ★ and `\digamma` ϝ; Noto Sans Math's 22 include `\wp` ℘, `\mho` ℧, `\ddagger` ‡ and
the spacing accents.

**Open Sans against Noto Sans (V).** googlefonts/opensans at `bd7e376` ships the four static styles
in two builds. Measured over each Regular file's cmap:

| font, Regular | file | code points | Latin Ext-B | Vietnamese | Greek | Cyrillic | Hebrew | punctuation | math operators | arrows | four styles |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Noto Sans, built in | 512,672 B | 2840 | 208/208 | 256/256 | 121/144 | 256/256 | 0/112 | 111/112 | 0/256 | 0/112 | 2,092,724 B |
| Open Sans, `path@elsewhere@fonts/ttf/` | 147,528 B | 1010 | 21/208 | 101/256 | 75/144 | 255/256 | 45/112 | 34/112 | 13/256 | 0/112 | 601,356 B |
| Open Sans, `path@elsewhere@fonts/noto-set/ttf/` | 528,744 B | 2882 | 208/208 | 256/256 | 121/144 | 256/256 | 45/112 | 111/112 | 13/256 | 0/112 | 2,159,352 B |

Noto Sans lacks the minus sign U+2212, ≤ ≥ ≠ ≈ ∞ and every arrow; the `noto-set` Open Sans lacks
the arrows only, among those.

**The licences of the dependencies (V).** `cargo metadata --format-version 1` over the workspace,
counting the `license` field of the 513 packages that come from a registry: 230
`MIT OR Apache-2.0`, 108 `MIT`, 55 `Apache-2.0 OR MIT`, 22 `Unicode-3.0`, 18 `MIT/Apache-2.0`, 16
`Apache-2.0`, and the rest other permissive licences (Zlib, BSD, ISC, BSL-1.0, CC0, Unlicense, OFL
for a font); three offer a GPL or LGPL branch, each as an alternative to MIT or Apache-2.0. No
dependency forces a copyleft licence.
