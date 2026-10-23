# Slice: Structural and cosmetic breaches

The second slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec.

## Builds

- `Breach` and `Document::breaches` in the model: the cosmetic breaches of a document. Here two
  kinds: `Breach::MissingAsset { area, asset }`, for an image or a text font that names an asset id
  the table lacks, and `Breach::UnknownFont { area, name }`, for a `FontRef::Builtin` naming a
  family the binary does not carry, which the renderer already draws in the built-in sans. The
  model names the built-in families, "sans" alone, in one constant, `BUILTIN_FAMILIES` of
  `path@rootpoint-model@src/document.rs`, which `Document::breaches` and `FontSet::family` of
  `path@rootpoint-render@src/text.rs` both read, so the two never disagree on what is unknown.
  `spec@editor-conveniences@styles-model` adds `Breach::MissingStyle`.
- `Document::validate` checks the structural invariants only. The check of
  `Document::validate_content` moves into `Document::breaches`. So `load_ron` loads a document with
  a missing asset, and `apply` accepts an operation whose result has one. `Op::SetContent` stops
  refusing an image whose asset the table lacks, with `OpError::UnknownAsset`, in `apply_in` of
  `path@rootpoint-model@src/ops.rs`: its inverse can bring such an image back, after a file was
  loaded with one.
- The commands that name an asset refuse a missing one: `image <area> <asset> …` and the
  `font=<asset>` option of `text`, each with the reply `err no asset a<n>`, in
  `Session::operation`. `Op::RemoveAsset` keeps its refusal of an asset in use: it refuses a removal,
  no reference, and no undo meets it, since the inverse of `Op::AddAsset` removes an id the counter
  has just given, which no area loaded with can name.
- The stand-ins and the reports exist, and are checked on a document with a missing asset: the grey
  image and `missing_images`, the built-in sans and `font_fallbacks`, the status line of the window
  and the reply of `render`.
- `path@rootpoint-protocol@tests/properties.rs`, with proptest added as a dev-dependency of
  rootpoint-protocol: random sequences of commands on a headless `Session` keep `Document::validate`,
  leave `Document::breaches` empty, and save to a text that loads back equal. It is AC4's instrument.

## Claims

In these claims, a missing asset is `a99` in a document whose `next_id` is above 99, so that the id
is below the counter and only the table lacks it: a file edited by hand raises `next_id` to 100.
An id at or above the counter is the structural breach of the sixth claim.

- **A missing asset loads, and is reported.** Test: a model test loads a document whose image area
  names `a99`, absent from the table; `load_ron` returns it, and `Document::breaches` returns
  `MissingAsset { area, asset: a99 }`. Shown to fail: against today's `validate`, the load is
  refused.
- **A missing asset draws its stand-in.** Test: `render_slide` on that document draws the area grey
  and lists `a99` in `missing_images`; a text whose font names a missing asset draws in the built-in
  sans and is listed in `font_fallbacks`. Shown to fail: against a renderer that skips the area, the
  centre pixel of the area is the slide's white.
- **The window reports it.** Test: a test of `App::frame` opens that document; `App::status`
  includes a line naming `a99`. Shown to fail: against a status that drops the list of missing
  images, no line names it.
- **A structural breach is still refused.** Test: every refusal of
  `path@rootpoint-model@tests/invalid.rs` but the two about a missing asset still holds. Of those
  two, `a_file_naming_a_missing_asset_is_refused` now asserts the load, and
  `a_text_whose_font_is_a_missing_asset_is_refused` asserts that the operation applies and that its
  inverse restores the document. Shown to fail: against a `validate` that accepts a split of one child,
  its test fails.
- **An undo that brings back a missing asset is not refused.** Test: a protocol test loads a
  document whose area n3 shows `a99`, absent from the table, runs `text n3 "x"`, then `undo`; the
  undo replies `ok`, and n3 shows `a99` again. Shown to fail: against an `Op::SetContent` that still
  refuses `UnknownAsset`, the undo replies `err`.
- **A reference to an asset at or above the counter is refused.** Test: a model test loads a
  document whose image names `a<next_id>`; `load_ron` refuses it with a reason naming the id. Shown
  to fail: against a `validate` that moves the asset check to `Document::breaches` and adds no check
  of the id, it loads, and the next `Op::AddAsset` gives that id to a new asset.
- **No command names a missing asset.** Test: protocol tests run `image n<id> a3 fit` and `text
  n<id> "x" font=a3` on a session where `a3` is an asset added then removed, below the counter, and
  the same with `a99`; each replies `err no asset a<n>` and changes nothing. Shown to fail: against a
  `Session::operation` that passes the id through, the document gains a breach.
- **No sequence of commands creates a breach.** Test: the property test above, at 256 cases a run,
  proptest's default. Shown to fail: with the refusal of `image` for a missing asset removed, the
  property test finds a sequence and shrinks it to one `image` command.

## Audit subjects

- `Document::validate`, `Document::validate_node` and `Document::validate_content` in
  `path@rootpoint-model@src/validate.rs`; `apply` and `asset_in_use` in
  `path@rootpoint-model@src/ops.rs`; `load_ron` in `path@rootpoint-model@src/file.rs`.
- `Session::operation`, `apply_style` and `Session::asset_bytes` in
  `path@rootpoint-protocol@src/session.rs`.
- `render_slide` and `Report` in `path@rootpoint-render@src/lib.rs`; the font lookup in
  `path@rootpoint-render@src/text.rs`.
- The asset loading of `path@rootpoint-app@src/assets.rs`; `App::status`.
- `design@rootpoint-model@validated-documents`, `rejected@rootpoint-model@unvalidated-documents`,
  `design@rootpoint-render@caller-supplies-assets`, `goal@rootpoint-model@faithful-readable-file`.

## Fails alone on

- A document with a missing asset is refused at load, or panics when drawn.
- The property test finds a sequence of commands that leaves a breach.

## Premises that expire

- `Breach` has two kinds here. `spec@editor-conveniences@styles-model` adds the missing style, and
  widens the property test with the style commands. Guard: AC4, judged at that slice.

## Mapping tables

Every check of `Document::validate` and its helpers, in `path@rootpoint-model@src/validate.rs` at the
time of writing, and its kind.

| check | kind | why |
| --- | --- | --- |
| the aspect ratio has no zero side | structural | no area rectangle can be computed |
| an id is below `next_id` | structural | a later operation would reuse it |
| a reference to an asset names an id below `next_id`: covered today by the check of a missing asset, which leaves for `Document::breaches` | structural | a later `Op::AddAsset` would give that id to a new asset, which the area would then show, and whose removal an undo could not make; a new check of `Document::validate` |
| an id is used once | structural | a command would name two nodes |
| a split has two children or more | structural | `design@rootpoint-model@split-tree-arity` is broken; a repair would guess the layout |
| a split has one divider fewer than its children | structural | `area_rects` panics on a missing divider |
| dividers strictly increase inside (0, 1), NaN refused | structural | a part has no extent, or a negative one |
| no child split has its parent's axis | structural | one layout would have two trees |
| an image or a text font names an asset in the table | cosmetic | the grey image and the built-in sans stand in, inside the area's drawing |
| `Op::SetContent` refuses an image whose asset the table lacks, in `apply_in` of `path@rootpoint-model@src/ops.rs` | cosmetic | the same check, at the operation; it leaves, and the command `image` refuses instead |
| no check today: a `FontRef::Builtin` naming a family the binary lacks | cosmetic | the built-in sans stands in; it becomes `Breach::UnknownFont` |

## Decided design

**A structural breach is refused; a cosmetic one is drawn with a stand-in and reported**,
`thread@editor-conveniences@cosmetic-fallbacks`.

A breach is structural when, with it, the layout or the identity of the nodes is undefined: no area
rectangle can be computed, an operation could panic or reuse an id, or a repair would have to guess
the author's layout. `Document::validate` refuses it at load and after every operation, as it does
today. A breach is cosmetic when it is a reference to a missing target that has a defined stand-in,
whose effect stays inside one area's drawing. It is accepted, drawn with the stand-in, and named in
the status line, in the reply of `render`, and, once the outline exists, on the area's row.

The principle is the owner's (`argument@editor-conveniences@arg-44`); the test that tells the two
kinds apart is the session's (`argument@editor-conveniences@arg-52`). It does not bring back
`rejected@rootpoint-model@unvalidated-documents`: the three counterexamples that defeated that
rival, a missing divider, a lowered id counter and dividers rounded onto each other, are structural
and stay refused (`argument@editor-conveniences@arg-53`). The renderer already draws the stand-ins;
only validation refused what it could draw (`argument@editor-conveniences@arg-50`).

**No command creates a cosmetic breach**, `thread@editor-conveniences@no-new-breach`. The commands
that name a target refuse one the document lacks. The operations refuse no reference to a missing
target: an undo can bring back a missing target a file was loaded with, and the undo must not be
refused. A refusal of a removal in use stays where no undo meets it
(`argument@editor-conveniences@arg-55`). The two together cover both inputs: a fallback handles a file
edited by hand, which `goal@rootpoint-model@faithful-readable-file` makes a purpose, and the property
test shows the editor itself never produces one (`argument@editor-conveniences@arg-51`).

## Implementation sequence

1. Add `Breach` and `Document::breaches`, and move the asset check into it; the model's tests pass
   with the invalid test rewritten. Fails alone on the model's tests.
2. Refuse a missing asset in the commands, and check the stand-ins and the reports on a document with
   one. Fails alone on the protocol, render and app tests of the claims.
3. Add the property test. Fails alone on a sequence it finds.

## Acceptance criteria

This slice lands the instrument of `acceptance@editor-conveniences@ac-no-command-breach`, judged by
`spec@editor-conveniences@styles-model`.

## Harvest

- `design@rootpoint-model@validated-documents`: rewritten in place to state the two kinds, the
  principle attributed to the owner and the test standing on its argument. Its slug stays if it
  still names the decision; a head of its own for the cosmetic tier,
  slugged after `thread@editor-conveniences@cosmetic-fallbacks`, if `primer@design-heads` cuts them as
  two decisions.
- A protocol head for `thread@editor-conveniences@no-new-breach`: the commands refuse a missing
  target, the operations do not. It passes entry test 2, a reason respected at every command that
  names a target.
- The comments citing `design@rootpoint-model@validated-documents`, in
  `path@rootpoint-model@src/ops.rs` and `path@rootpoint-model@src/validate.rs`, re-read.
