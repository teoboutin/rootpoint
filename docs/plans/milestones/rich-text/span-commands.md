# Slice: the commands of the text tree and of the span styles

The second slice of `milestone@plans@rich-text`. It lands the protocol's half of
`thread@rich-text@text-edit-commands`, of `thread@rich-text@text-node-ids`, of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`.

## Builds

- In `path@rootpoint-protocol@src/command.rs`: the commands `insert`, `delete`, `wrap`, `unwrap`,
  `span`, `split-span`, and `span-style add`, `set`, `rename` and `remove`, each one line, each
  printed and parsed back; `text` taking a whole tree on one line, its grammar fixed at this slice's
  audit from the milestone's illustration, quoted strings and the escapes of
  `design@rootpoint-protocol@protocol-transport`.
- In `path@rootpoint-protocol@src/tree.rs`: `tree` printing a text as a tree in the grammar of
  `text`, each span with its `t<n>`.
- In `path@rootpoint-protocol@src/session.rs`: each command run as its operation; a command that
  names a span style or a font asset the document lacks refused, per
  `design@rootpoint-protocol@no-new-breach`; `assets uses=` and `asset-remove … into=` counting a
  span's font.
- In `path@rootpoint-protocol@README.md`: the new commands.

## Claims

- **Every new command parses back from its printed form**: the protocol's round-trip test, extended
  to each new command and to `text` with nested spans; shown to fail by a printer that drops a
  span's quote escape.
- **`tree`'s output, given back to `text`, gives the same text, with new ids**: a test on the
  sample's styled text.
- **No command sequence leaves a new breach**: the property test of
  `design@rootpoint-protocol@no-new-breach`, its commands extended to the new ones.
- **A span keeps its id across edits before it**: a script inserts text before a span, then styles
  the span by its `t<n>`.

## Audit subjects

- `path@rootpoint-protocol@src/command.rs`, `path@rootpoint-protocol@src/tree.rs`,
  `path@rootpoint-protocol@src/session.rs`, `path@rootpoint-protocol@README.md`.
- `design@rootpoint-protocol@protocol-transport`, `design@rootpoint-protocol@protocol-addressing`,
  `design@rootpoint-protocol@no-new-breach`, `design@rootpoint-protocol@asset-commands`,
  `design@rootpoint-protocol@protocol-axis-words`, `design@rootpoint-protocol@gesture-undo`,
  `rejected@rootpoint-protocol@json-rpc-commands`, `rejected@rootpoint-protocol@merged-commands`.

## Fails alone on

- The parse of every printed command, and the property test of no new breach.

## Premises that expire

- The editor still sends `text` with the whole text at each key, which now gives new span ids at
  each key; #tree-caret ends that.

## Decided design

The milestone's "Decided design" of #text-edit-commands, #text-node-ids, #split-at-caret and
#named-span-styles holds the shape.

## Acceptance criteria

None is judged at this slice.

## Harvest

- Write the protocol's head of #text-edit-commands, and rewrite
  `design@rootpoint-protocol@protocol-addressing` for `t<n>` and
  `design@rootpoint-protocol@asset-commands` for a span's font; update the lines of
  `design@rootpoint-model@stable-ids`'s referencing texts in the protocol's design home, and the
  comment at `path@rootpoint-protocol@src/session.rs` that cites
  `design@rootpoint-model@named-styles`.
- Write T2, guarding the head of #text-edit-commands.
