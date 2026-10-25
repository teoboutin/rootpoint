# Slice: the commands of the text tree and of the span styles

The second slice of `milestone@plans@rich-text`. It lands the protocol's half of
`thread@rich-text@text-edit-commands`, of `thread@rich-text@text-node-ids`, of
`thread@rich-text@split-at-caret` and of `thread@rich-text@named-span-styles`.

## Builds

- In `path@rootpoint-protocol@src/command.rs`: the commands of the milestone's mapping table "A
  command, to the operation it runs", `insert`, `delete`, `wrap`, `unwrap`, `span`, `split-span`, and `span-style add`, `set`,
  `rename` and `remove`, each one line, each printed and parsed back; `text` taking a whole tree on
  one line, its grammar fixed at this slice's audit from the illustration in the milestone's
  "Decided design" of #text-edit-commands, with quoted
  strings and the escapes of `design@rootpoint-protocol@protocol-transport`.
- In `path@rootpoint-protocol@src/tree.rs`: `tree` printing a text as a tree in the grammar of
  `text`, each span with its `t<n>`.
- In `path@rootpoint-protocol@src/session.rs`: each command run as its operation; a command that
  names a span style or a font asset the document lacks refused, per
  `design@rootpoint-protocol@no-new-breach`; the asset commands already count a span's and a span
  style's font as uses of an asset, since #text-tree; the `render` reply naming a span's missing
  span style.
- In `path@rootpoint-protocol@README.md`: the new commands.
- In the app, which builds `Command::Text` with a string: `path@rootpoint-app@src/editing.rs`,
  `path@rootpoint-app@src/actions.rs` (`set_text_content`, `send_text_option`) and the app's tests
  that send `text` lines keep compiling and keep their behaviour, a bare quoted string being read as
  a one-run tree if the grammar keeps it; the window's status line in
  `path@rootpoint-app@src/lib.rs` and the breach list of `path@rootpoint-app@src/panels.rs` name a
  span's missing span style, as the `render` reply does.

## Steps

1. The span style commands, on the model's operations of #text-tree. Fails alone on: their parse
   and their refusals.
2. The commands of the tree and `tree`'s new print. Fails alone on: the parse of every printed
   command.
3. `text` with a tree, the `render` reply naming a missing span style, and the property test
   extended. Fails alone on: the property test.

## Claims

- **Every new command parses back from its printed form**: `every_command_prints_back_as_it_parses`
  and `every_scripted_line_prints_back_as_a_line_that_parses_to_it` of
  `path@rootpoint-protocol@tests/commands.rs`, extended to each new command and to `text` with
  nested spans; shown to fail by a printer that drops a span's quote escape.
- **`tree`'s output, given back to `text`, gives the same text, with new ids**: a test that builds
  a styled text with nested and named spans by commands, since the sample holds none before
  #span-styling, reads the text's tree from `tree`'s line and gives it to `text`; shown to fail by
  a printer that drops a named span's name.
- **No command sequence leaves a new breach**: the property test of
  `path@rootpoint-protocol@tests/properties.rs`, its commands extended to the new ones.
- **A span keeps its id across edits before it**: a script inserts text before a span, then styles
  the span by its `t<n>`; shown to fail by an `insert` that sets the text whole.
- **`split-span` at a span's edge is refused**: a test at each edge; shown to fail by a session
  that moves the caret instead of replying `err`.
- **No command sequence leaves a new breach** is shown to fail by a `span` command that accepts a
  span style the table lacks.

## Fixtures

The new commands' tests are scripts of `path@rootpoint-protocol@tests/scripts/` and tests of
`path@rootpoint-protocol@tests/commands.rs`, written in the commands this slice adds: a styled text
built by `text` with a tree, or by `wrap` over a plain text, nested and named spans, and a span style
added by `span-style add`. No fixture needs a form the slice does not add.

## Audit subjects

- Open at this audit, each applied in place where the milestone's decisions imply one answer, and
  put to the owner where two shapes survive, per `skill@knowledge-architect-planning@working-a-slice`,
  point 2: which operation carries `text`'s area-style options beside the tree, since `Op::SetText`
  keeps the area's style, and whether a change of the area's style alone gives the spans new ids;
  which options a span command takes, and whether `span` and `span-style set` merge over the current
  values, as a named span made custom must, and how a set value is cleared back to inherited; the
  grammar's open points, a `t<n>` inside braces, a named span inside braces, spaces around the
  brackets, the text tree inside a slide line of `tree`, the order of a span's options, the empty
  tree and the empty span; how a command naming `t<n>` finds its area; the replies of `wrap`,
  `split-span` and `text`, the ids they make, and the `render` reply's word for a missing span
  style; a query that lists the span style table, against
  `goal@rootpoint-protocol@commands-drive-everything`; and how `assets uses=` tells a span style
  from a text style of one name.
- The tests that assert `tree`'s one-string output, `path@rootpoint-protocol@tests/headless.rs` and
  `path@rootpoint-protocol@tests/files.rs` beside `path@rootpoint-protocol@tests/commands.rs`, and
  the hand-written lists of `path@rootpoint-protocol@tests/properties.rs`, its step kinds and its
  `ids()`, which no step lets pick a `t<n>` yet.

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

- The editor still sends `text` with the whole text at each key, which drops the spans of a text
  that holds them, or gives them new ids if the editor sends the tree; #tree-caret ends that, and no
  release is made between the slices.

## Decided design

The milestone's "Decided design" of #text-edit-commands, #text-node-ids, #split-at-caret and
#named-span-styles holds the shape.

## Acceptance criteria

None is judged at this slice.

## Harvest

- Write the protocol's head of its small commands, under the slug of #text-edit-commands, against
  whole-text replacement: the commands this slice builds; the editor's use of them is
  `design@rootpoint-app@edit-in-place`'s, rewritten at #tree-caret.
- Rewrite `design@rootpoint-protocol@protocol-transport` for `text`'s tree on one line,
  `design@rootpoint-protocol@protocol-addressing` for `t<n>` and the offsets read at the moment of a
  command, and record the members of `design@rootpoint-protocol@asset-commands` and
  `design@rootpoint-protocol@no-new-breach`, which their arguments cover, directly; update the
  texts the milestone's table lists for them, the protocol's lines of
  `design@rootpoint-model@stable-ids`, and the comment of `path@rootpoint-protocol@src/session.rs`
  that cites `design@rootpoint-model@named-styles`.
- Add to `design@rootpoint-model@named-span-styles` the rule its commands build, a value set on a
  named span alone making it custom, with the named values and that change, as
  `design@rootpoint-model@named-styles` says of a text.
- Record the name `split-span`, #split-at-caret's protocol half, at its code, or in a head if the
  entry tests give it one.
- The losing alternatives of these threads, judged under
  `skill@knowledge-architect-decision-recording@losing-alternatives`.
- Write T2, guarding the protocol's head of #text-edit-commands and the rewritten
  `design@rootpoint-protocol@protocol-transport`.
