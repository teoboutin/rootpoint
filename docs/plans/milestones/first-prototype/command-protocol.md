# Slice: The text protocol and the headless binary

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-protocol` at `path@rootpoint@crates/rootpoint-protocol/`: `Command`,
  `parse_command`, `ParseError`, `Session`, which holds one document, its undo stack and a view
  state, and executes commands, and `Reply`.
- The binary `rootpoint-headless`: reads one command per line from stdin, writes one reply per
  command to stdout, and exits at the end of its input.
- The library root and the binary read `ROOTPOINT_CHECKOUT`, as `path@xtask@CLAUDE.md` requires.
- The library added to the `wasm` gate: the browser build links `Session`. The binary is not built
  for the wasm target.
- `Files`, the trait through which the session reads and writes files: the presentation, its
  assets and rendered images. The session opens no file itself, since the browser build links it
  and `std::fs` fails there at run time, as `path@rootpoint-model@CLAUDE.md` says; the headless
  binary gives it a `Files` on the native file system. Asset paths are relative to the
  presentation file's directory, or to the working directory of a presentation not yet saved. What
  a save into another directory does to them is open: `issue@rootpoint-protocol@asset-paths-follow-save`.
- The renaming of the model's axes to `Axis::SideBySide` and `Axis::Stacked`, built in this slice
  and recorded in `design@rootpoint-model@split-tree-arity`: a fresh agent read `Vertical` as
  stacked. The protocol's words are `beside` and `stacked`. A default awaiting the owner, Q11.
- Per D2, the crate's Component.

## Claims

1. **Every command parses and prints back.** For every `Command` variant, printing then parsing
   gives the same command. Test: one case per variant. Shown to fail: a printer that drops a
   quoted string's escaping fails on a text holding a quote.
2. **A command and its operation agree.** Each editing command applies the operation the mapping
   table gives. Test: per row, the document after the command equals the document after `apply`
   of that operation. Shown to fail: swapping the axes of `split` fails the comparison.
3. **Ids survive save and load.** A script that adds areas, saves, loads, and then addresses the
   areas by the ids printed before the save succeeds. Shown to fail: allocating fresh ids at load
   fails the script.
4. **A failed command changes nothing.** A command that fails replies `err <reason>`, and the
   document and the view equal those before it. Test: one failing case per command.
5. **A headless script renders.** A script piped into `rootpoint-headless` builds the fixture
   presentation, saves it into a subdirectory, reads its asset beside it, and renders a slide to a
   PNG equal to the render crate's golden image of it.
6. **The view commands run headless.** A script selects a node, enters presenter mode, moves to the
   next and previous slide and leaves it, and `view` reports each state.

## Fixtures

Scripts in the protocol's own grammar, in `path@rootpoint-protocol@tests/scripts/`, for the
claims a script states: 3, in two halves, 5 and 6, with the replies of claim 6 beside its script.
They use only the commands of the mapping table and of the view commands. Claims 1, 2 and 4 are
tables of lines and commands in the tests, since each row is one line and its expected outcome.

## Audit subjects

- `design@rootpoint-model@split-tree-arity`, for the names of the axes in `split`.
- `thread@first-prototype@protocol-addressing`, `thread@first-prototype@protocol-transport`,
  `thread@first-prototype@edit-operation-core`, `goal@rootpoint@headless-core`.
- The model crate's `Op` and `apply`, and the render crate's `render_slide`, as landed.

## Fails alone on

- A claim 2 failure with the model's claim 1 passing points at the mapping, not at the operation.
- Claim 5 failing with the render crate's golden images passing points at the session.

## Premises that expire

- The session is driven only from stdin in this slice. Guard: `spec@first-prototype@live-control`
  adds the socket and runs the same scripts through it; the grammar does not change.

## Implementation sequence

1. **An empty session.** `Session` with the commands `new`, `save`, `load`, `tree` only, and the
   headless binary reading stdin. Fails alone on: the line handling and the reply format.
2. **The editing commands.** The rows of the mapping table, with `undo` and `redo`. Claims 1, 2, 4.
3. **The view commands.** Claim 6.
4. **Ids across files, and render.** Claim 3; the `render` command, claim 5.

## Decided design

### Replies, undo and rendering

- A command that creates ids replies with them: `add-slide` with `s<n> root=n<n>`, `split` with
  `new=n<n>`, and `split=n<n>` when it wrapped the node in a new split, `duplicate-slide` with the
  copy's id and its root, `s<n> root=n<n>`, as `add-slide` does, `asset` with `a<n>`. `present`,
  `next` and `prev` reply with the slide presented.
- Each accepted editing command pushes its inverse on the undo stack and clears the redo stack;
  `undo` applies the top inverse and pushes its own inverse on the redo stack. `new` and `load`
  clear both. View commands are not undone.
- `render <slide> <path> [w h]` draws with the renderer's portable precision, 1920×1080 by
  default, so a headless image is the same on every machine; it decodes the slide's image and font
  assets through `Files`, and its reply carries the renderer's report: `fallback=`, `missing=` and
  `glyphs=` lists, which the tripwire on missing glyphs reads.
- Blank lines and lines starting with `#` give no reply, so a script can carry comments.
- A text's style options: `size=<slide units>`, above 0, `bold` or `bold=off`, `italic` or
  `italic=off`, `colour=#rrggbb` or `#rrggbbaa`, `align=left|center|right`,
  `valign=top|middle|bottom`, `font=sans|a<n>`. A text keeps its area's current style, so the `off`
  forms are what undoes a `bold` by a later `text`.
- `tree` writes a split as `n<n>:<axis>(<dividers>)[<child> | <child> ...]`, with `beside` or
  `stacked` and the dividers' positions, and an area as `n<n>` followed by `empty`,
  `text "<text>"` or `image a<n> fit|fill`; `tree` with no slide joins every slide's tree with
  `; `. `view` replies `selected=n<n>|- presenting=s<n>|-`.
- `next` and `prev` at the last or the first slide reply `err` and change nothing.

### The grammar

One command per line: a verb, then arguments separated by spaces; a text argument is a
double-quoted string with `\"`, `\\` and `\n` escapes, the last so that a text holding a new line
stays on one line; a path is bare or quoted; a quoted word is only ever a text or a path, never a
verb, an id or an option; an id or a number of pixels is written as the printer writes it, with no
sign and no leading zero; a named option is `key=value`. Every reply is one line that starts with
`ok` or `err`, followed by the result or the reason, a new line or a carriage return in which
prints as `\n` or `\r`; `tree` replies with the slide's tree on one line, every split and area with
its id
(`thread@first-prototype@protocol-transport`). An illustration of the shape, not a test:

```text
> add-slide                ok s1 root=n2
> split n2 beside 0.5 after  ok new=n3 split=n4
> text n2 "Hello" size=64 bold
                           ok
> asset cat.png            ok a5
> image n3 a5 fit          ok
> tree s1                  ok s1 n4:beside(0.5)[n2 text "Hello" | n3 image a5 fit]
> render s1 out.png        ok fallback=- missing=- glyphs=-
```

Nearest rival: JSON-RPC; see "Losing alternatives" of the milestone document.

### Addressing

Commands address slides, nodes and assets by the ids the model stores, never by position
(`thread@first-prototype@protocol-addressing`).

Nearest rival: addressing by paths; see "Losing alternatives" of the milestone document.

### The view state

The session holds, besides the document, a view state: the selected node, if any, and whether
presenter mode is on, with its current slide. The view commands change it and apply no operation,
so they are not undone. The GUI shows this state and changes it only through these commands, so
every operation the GUI offers runs headless, per `goal@rootpoint@headless-core`.

## Mapping tables

Each editing command to the operation it applies. The verbs are the shape the design needs; the
audit may refine them in place.

| command | `Op` |
| --- | --- |
| `add-slide [at]` | `AddSlide { at }` |
| `remove-slide <slide>` | `RemoveSlide { slide }` |
| `move-slide <slide> <to>` | `MoveSlide { slide, to }` |
| `duplicate-slide <slide>` | `DuplicateSlide { slide }` |
| `split <node> <beside\|stacked> <at> [before\|after]` | `Split { node, axis, at, side }` |
| `remove <node>` | `Remove { node }` |
| `divider <split> <index> <pos>` | `MoveDivider { split, index, pos }` |
| `swap <a> <b>` | `Swap { a, b }` |
| `text <area> "<text>" [style options]` | `SetContent { area, content: Text(..) }` |
| `image <area> <asset> [fit\|fill]` | `SetContent { area, content: Image(..) }` |
| `clear <area>` | `SetContent { area, content: Empty }` |
| `asset <path>` | `AddAsset { asset }`, replying with the new asset id |
| `remove-asset <asset>` | `RemoveAsset { id }` |
| `undo`, `redo` | the inverse on the undo stack, and its inverse |

The commands `new`, `save <path>`, `load <path>`, `tree [slide]` and `render <slide> <path> [w h]`
apply no operation. The view commands apply none either: `select <node>`, `deselect`,
`present [slide]`, `next`, `prev`, `stop`, and `view`, which replies with the view state.

## Harvest

Judged by the entry tests of `primer@design-heads`, into the design home of the protocol crate's
Component: addressing by id, citing `design@rootpoint-model@stable-ids`, from
`thread@first-prototype@protocol-addressing`; the line grammar, the stdin binary and the view state,
from `thread@first-prototype@protocol-transport` and `goal@rootpoint@headless-core`, the grammar
being an interface others consume; the losing alternatives, paths and JSON-RPC, judged for a
`rejected` entry.
