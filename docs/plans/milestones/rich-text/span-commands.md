# Slice: the commands of the text tree and of the span styles

The second slice of `milestone@plans@rich-text`. It lands the protocol's half of
`thread@rich-text@text-edit-commands`, of `thread@rich-text@text-node-ids`, of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`.

## Builds

- In `path@rootpoint-protocol@src/command.rs`: the commands of the milestone's second mapping
  table, `insert`, `delete`, `wrap`, `unwrap`, `span`, `split-span`, and `span-style add`, `set`,
  `rename` and `remove`, each one line, each printed and parsed back; `text` taking a whole tree on
  one line, its grammar fixed at this slice's audit from the milestone's illustration, with quoted
  strings and the escapes of `design@rootpoint-protocol@protocol-transport`.
- In `path@rootpoint-protocol@src/tree.rs`: `tree` printing a text as a tree in the grammar of
  `text`, each span with its `t<n>`.
- In `path@rootpoint-protocol@src/session.rs`: each command run as its operation; a command that
  names a span style or a font asset the document lacks refused, per
  `design@rootpoint-protocol@no-new-breach`; `assets uses=` and `asset-remove … into=` counting a
  span's and a span style's font; the `render` reply naming a span's missing span style.
- In `path@rootpoint-protocol@README.md`: the new commands.

## Steps

1. The span style commands, on the model's operations of #text-tree. Fails alone on: their parse
   and their refusals.
2. The commands of the tree and `tree`'s new print. Fails alone on: the parse of every printed
   command.
3. `text` with a tree, and the property test extended. Fails alone on: the property test.

## Claims

- **Every new command parses back from its printed form**: `every_command_prints_back_as_it_parses`
  and `every_scripted_line_prints_back_as_a_line_that_parses_to_it` of
  `path@rootpoint-protocol@tests/commands.rs`, extended to each new command and to `text` with
  nested spans; shown to fail by a printer that drops a span's quote escape.
- **`tree`'s output, given back to `text`, gives the same text, with new ids**: a test that builds
  a styled text with nested and named spans by commands, since the sample holds none before
  #span-styling.
- **No command sequence leaves a new breach**: the property test of
  `path@rootpoint-protocol@tests/properties.rs`, its commands extended to the new ones.
- **A span keeps its id across edits before it**: a script inserts text before a span, then styles
  the span by its `t<n>`.
- **`split-span` at a span's edge is refused**: a test at each edge.

## Audit subjects

- `path@rootpoint-protocol@src/command.rs`, `path@rootpoint-protocol@src/tree.rs`,
  `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@README.md`,
  `path@rootpoint-protocol@tests/commands.rs`, `path@rootpoint-protocol@tests/properties.rs`.
- `design@rootpoint-protocol@protocol-transport`, `design@rootpoint-protocol@protocol-addressing`,
  `design@rootpoint-protocol@no-new-breach`, `design@rootpoint-protocol@asset-commands`,
  `design@rootpoint-protocol@gesture-undo`, `rejected@rootpoint-protocol@json-rpc-commands`,
  `rejected@rootpoint-protocol@merged-commands`.

## Fails alone on

- The parse of every printed command, and the property test of no new breach.

## Premises that expire

- The editor still sends `text` with the whole text at each key, which now gives new span ids at
  each key; #tree-caret ends that, and no release is made between the slices.

## Decided design

The milestone's "Decided design" of #text-edit-commands, #text-node-ids, #split-at-caret and
#named-span-styles holds the shape.

## Acceptance criteria

None is judged at this slice.

## Harvest

- Write the protocol's head of the editor's small commands, under the slug of #text-edit-commands,
  against whole-text replacement.
- Rewrite `design@rootpoint-protocol@protocol-transport` for `text`'s tree on one line,
  `design@rootpoint-protocol@protocol-addressing` for `t<n>` and the offsets read at the moment of a
  command, and record the members of `design@rootpoint-protocol@asset-commands` and
  `design@rootpoint-protocol@no-new-breach`, which their arguments cover, directly; update the
  texts the milestone's table lists for them, the protocol's lines of
  `design@rootpoint-model@stable-ids`, and the comment of `path@rootpoint-protocol@src/session.rs`
  that cites `design@rootpoint-model@named-styles`.
- Record the name `split-span`, #split-at-caret's protocol half, at its code, or in a head if the
  entry tests give it one.
- The losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T2, guarding the protocol's head of #text-edit-commands and the rewritten
  `design@rootpoint-protocol@protocol-transport`.
