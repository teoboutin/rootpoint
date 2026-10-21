# Slice: The text protocol and the headless binary

This slice's spec leaves the repository when the slice lands. The milestone document,
`milestone@plans@first-prototype`, holds the names, the criteria, the threads and the arguments it
cites.

## Builds

- The crate `rootpoint-protocol` at `planned@rootpoint@crates/rootpoint-protocol/`: `Command`,
  `parse_command`, `ParseError`, `Session`, which holds one document and its undo stack and executes
  commands, and `Reply`.
- The binary `rootpoint-headless`: reads one command per line from stdin, writes one reply per
  command to stdout, and exits at the end of its input.
- The crate's wasm build, added to the `wasm` gate. The binary is not built for the wasm target.
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
   document equals the document before it. Test: one failing case per command.
5. **A headless script renders.** A script piped into `rootpoint-headless` builds the fixture
   presentation and renders a slide to a PNG equal to the render crate's golden image of it.

## Fixtures

Scripts in the protocol's own grammar, under the crate's test directory, one per claim. They use
only the commands of the mapping table.

## Audit subjects

- `thread@first-prototype@protocol-addressing`, `thread@first-prototype@protocol-transport`,
  `thread@first-prototype@edit-operation-core`.
- The model crate's `Op` and `apply`, and the render crate's `render_slide`, as landed.

## Fails alone on

- A claim 2 failure with the model's claim 1 passing points at the mapping, not at the operation.
- Claim 5 failing with the render crate's golden images passing points at the session.

## Premises that expire

- The session is driven only from stdin in this slice. The slice `spec@first-prototype@live-control`
  adds the socket; the grammar does not change.

## Implementation sequence

1. **An empty session.** `Session` with the commands `new`, `save`, `load`, `tree` only, and the
   headless binary reading stdin. Fails alone on: the line handling and the reply format.
2. **The editing commands.** The rows of the mapping table, with `undo` and `redo`. Claims 1, 2, 4.
3. **Ids across files, and render.** Claim 3; the `render` command, claim 5.

## Decided design

### The grammar

One command per line: a verb, then arguments separated by spaces; a text argument is a
double-quoted string with `\"` and `\\` escapes; a named option is `key=value`. Every reply is one
line that starts with `ok` or `err`, followed by the result or the reason; `tree` replies with the
tree on one line, every node with its id (`thread@first-prototype@protocol-transport`). An
illustration of the shape, not a test:

```text
> add-slide            ok s1 root=n1
> split n1 v 0.5       ok n2 n3
> text n2 "Hello" size=64 bold
> image n3 @cat fit
> tree s1              s1 V[n2 text "Hello" | n3 image @cat fit]
> render s1 out.png    ok
```

### Addressing

Commands address slides and nodes by the ids the model stores, never by position
(`thread@first-prototype@protocol-addressing`).

## Mapping tables

Each editing command to the operation it applies. The verbs are the shape the design needs; the
audit may refine them in place.

| command | `Op` |
| --- | --- |
| `add-slide [at]` | `AddSlide { at }` |
| `remove-slide <slide>` | `RemoveSlide { slide }` |
| `move-slide <slide> <to>` | `MoveSlide { slide, to }` |
| `duplicate-slide <slide>` | `DuplicateSlide { slide }` |
| `split <node> <h\|v> <at> [before\|after]` | `Split { node, axis, at, side }` |
| `remove <node>` | `Remove { node }` |
| `divider <split> <index> <pos>` | `MoveDivider { split, index, pos }` |
| `swap <a> <b>` | `Swap { a, b }` |
| `text <area> "<text>" [style options]` | `SetContent { area, content: Text(..) }` |
| `image <area> <asset> [fit\|fill]` | `SetContent { area, content: Image(..) }` |
| `clear <area>` | `SetContent { area, content: Empty }` |
| `asset <path>` | `AddAsset { asset }`, replying with the new asset id |
| `undo`, `redo` | the inverse on the undo stack, and its inverse |

The commands `new`, `save <path>`, `load <path>`, `tree [slide]` and `render <slide> <path> [w h]`
apply no operation.

## Harvest

Judged by the entry tests of `primer@design-heads`, into the design home of the protocol crate's
Component (rootpoint's, if the owner rules D2 otherwise):
`thread@first-prototype@protocol-addressing` and `thread@first-prototype@protocol-transport`, the
grammar being an interface others consume; the losing alternatives, paths and JSON-RPC, judged for a
`rejected` entry.
