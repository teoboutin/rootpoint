# Slice: Named styles in the model and the protocol

The third slice of `milestone@plans@editor-conveniences`. Read the milestone document whole before
this spec. This slice builds the headless half of `thread@editor-conveniences@named-styles`; the
window's half is `spec@editor-conveniences@styles-window`.

## Builds

- **The style table.** `Document::styles`, a map from a style's name to its `TextStyle`, ordered by
  name. `TextContent::style` becomes a `StyleChoice`: `Named(String)`, a name the table may hold, or
  `Custom(TextStyle)`, the area's own values.
- **The predefined styles.** `Document::default()` holds four styles. The values are the defaults
  the owner approved in `thread@editor-conveniences@new-area-style`:
  - Title: size 120, bold, centred, middle;
  - Subtitle: size 64, centred;
  - Body: size 40, left, top;
  - Legend: size 28, italic, centred.

  The other fields of each are those of `TextStyle::default()`.
- **Resolution.** `Document::resolve_style` gives the `TextStyle` a `StyleChoice` draws with: the
  table's entry, the custom values, or `TextStyle::default()` for a name the table lacks.
  `render_slide` and `area_text` draw with it. The renderer's text layout takes the resolved style,
  not the `TextContent`.
- **The missing style.** `Breach::MissingStyle { area, name }` joins `Document::breaches`.
  `Report::missing_styles` lists, for each text drawn with the default style for that reason, its
  area and the missing name. The reply of `render` and `App::status` name both, as they name a
  missing image's asset.
- **The operations**, each with its inverse, in `path@rootpoint-model@src/ops.rs`:
  - `Op::AddStyle { name, style }`, refused when the name is in the table or is empty; its inverse
    is the removal of the style with no replacement;
  - `Op::SetStyle { name, style }`, which replaces a style's values;
  - `Op::RenameStyle { from, to }`, which rewrites every `Named(from)` to `Named(to)`, refused when
    `to` is in the table; its inverse renames back;
  - `Op::RemoveStyle { name, into }`: `into` names another style, or asks for a custom copy of the
    removed style for each user, or is `None`, which leaves the users naming the removed style.

  No operation refuses a reference to a missing style, per
  `thread@editor-conveniences@no-new-breach`: an undo must not be refused. With a file loaded with
  areas naming a missing "Heading", `style add "Heading"` then `undo` removes Heading again and
  leaves those areas as they were loaded. The inverse of a removal restores the style and each
  user's `Named(name)`; the shape of that inverse is the implementer's.
- **The new area's style**, `thread@editor-conveniences@new-area-style`:
  - a split of a text area gives its new area that area's `StyleChoice`, when it is `Custom` or names
    a style the table holds; a name the table lacks is not copied, so no split creates a breach;
  - every other new area takes `Named("Body")` when the table holds "Body", and
    `Custom(TextStyle::default())` otherwise. That covers a split of a split or of an image area, the
    area of `Op::AddSlide`, an image area that `clear` empties, and an image area that
    `text <area> "<text>"` with no style option makes a text, as `App::set_text_content` does.
- **The commands**, in `path@rootpoint-protocol@src/command.rs` and
  `path@rootpoint-protocol@src/session.rs`:
  - `style add "<name>" <options>`: the options apply to `TextStyle::default()`;
  - `style set "<name>" <options>`: the options apply to the style's current values;
  - `style rename "<from>" "<to>"`, refused when `to` is in the table or when an area names `to`,
    so the rename's inverse restores exactly the areas it rewrote;
  - `style remove "<name>"`, refused while an area uses the style, with a reply naming its users;
    `style remove "<name>" into="<other>"` and `style remove "<name>" into=custom`;
  - `styles`, a query, which replies with one line listing each style and its options in the order
    of the table;
  - `text <area> "<text>" style="<name>"` names a style, and is refused when the table lacks it, per
    `thread@editor-conveniences@no-new-breach`;
  - `text <area> "<text>" custom` copies the resolved values into `Custom`;
  - any other style option on an area whose style is `Named` makes it `Custom`, with the named
    style's values and that change.

  `tree` prints a text as `n2 text "rootpoint" style="Title"` or `n3 text "…" custom`.

  A style's name may hold a space, as "Style 1" does. Today `split_words` of
  `path@rootpoint-protocol@src/command.rs` reads `key="…"` as one bare word with its quotes, and a
  space ends it. The grammar gains a quoted value after `key=`, with the escapes of a quoted text,
  and `Command`'s `Display` prints it back so that the line parses to the same command.
  The socket's queries are listed twice in `path@rootpoint-app@src/control.rs`, `tree`, `view` and
  `render`; `styles` joins both lists, so a client without the authority can read the styles.
- **Fonts in styles.** A style whose font names an asset uses that asset: `asset_in_use` and
  `Breach::MissingAsset` read the style table as well as the areas. A text whose resolved font is a
  missing asset is a `MissingAsset` of that area.
- **The format.** `FORMAT_VERSION` is 3. The sample, `path@rootpoint-protocol@sample/sample.ron`,
  holds the four styles. Each of its texts whose values equal Title's or Body's names that style:
  n2 names Title, and n3, n5, n9 and n11 name Body. Its other texts are `Custom`.
- **The property test** of `path@rootpoint-protocol@tests/properties.rs`, widened with the style
  commands and with `style=` names that may be missing.

## Claims

- **A named style draws as its table entry.** Test: a render test draws a text naming Title and a
  text whose `Custom` style equals Title's values; the two images are equal. Shown to fail: against
  a resolution that ignores the table, the named text draws at size 48.
- **A style edit changes every area naming it.** Test: a model test applies `Op::SetStyle` to Body
  and finds every text of the sample naming Body resolving to the new values, and the Title text
  unchanged. Shown to fail: against a `SetStyle` that copies into areas, an undo then leaves the
  areas changed.
- **A rename rewrites the references.** Test: `style rename "Body" "Text"` leaves no `Named("Body")`,
  and its undo restores them. Shown to fail: against a rename of the table's key alone,
  `Document::breaches` names two missing styles.
- **A removal takes a replacement.** Tests:
  - `style remove "Body"` on the sample is refused, with a reply naming its users n3, n5, n9 and
    n11;
  - with `into="Title"`, every user names Title;
  - with `into=custom`, every user holds `Custom` with Body's values, and their render is unchanged;
  - an undo of each restores `Named("Body")`.

  Shown to fail: against a removal that leaves the users naming Body, `Document::breaches` is not
  empty.
- **A missing style draws with the default and is reported.** Test: a document loaded with
  `Named("Heading")` and no such style loads. Its text draws as `TextStyle::default()`, the reply of
  `render` names the area and "Heading", and `Document::breaches` holds `MissingStyle`. Shown to fail: against a
  `validate` that refuses an unknown name, the load fails.
- **No command names a missing style.** Test: `text n3 "x" style="Heading"` on a document without
  Heading replies `err` and changes nothing. Shown to fail: against a command that passes the name
  through, the document gains a breach.
- **A style option on a named area makes it custom.** Test: `text n3 "x" size=64` on a Body area
  gives `Custom` with Body's values and size 64; Body itself is unchanged. Shown to fail: against an
  option applied to the named style, Body's size becomes 64.
- **A new area takes the split area's style, or Body.** Tests: a split of the sample's Title area
  gives a new area naming Title; a split of the split n6 gives one naming Body; `Op::AddSlide` on a
  document whose table lacks Body gives `Custom(TextStyle::default())`. Shown to fail: against a
  split that always gives Body, the first test fails.
- **An undo of a style's addition is not refused.** Test: on a document loaded with n3 naming the
  missing "Heading", `style add "Heading"` then `undo` replies `ok`, Heading leaves the table, and n3
  names "Heading" again. Shown to fail: against an `Op::RemoveStyle` that refuses a style in use, the
  undo replies `err`.
- **A rename onto a name an area holds is refused.** Test: on the same document,
  `style rename "Body" "Heading"` replies `err`, and the document is unchanged. Shown to fail:
  against a rename that checks only the table, n3 is caught by the rename, and its undo turns n3
  into Body.
- **A split does not copy a missing name.** Test: on the same document, a split of n3 gives a new
  area naming Body. Shown to fail: against a split that copies every `StyleChoice`, the new area
  names "Heading", and the property test of the next claim would not see it, since it starts from a
  document with no breach.
- **A name with a space reads and prints back.** Test: `style add "Style 1" size=40` and
  `text n3 "x" style="Style 1"` parse, run, and print back through `Command`'s `Display` to lines
  that parse to the same commands. Shown to fail: against today's `split_words`, the second line
  splits `style="Style` from `1"`.
- **AC4.** The widened property test passes. Shown to fail: with the refusal of a missing `style=`
  name removed, it finds a sequence leaving a `MissingStyle`.

## Fixtures

The committed `.ron` files are rewritten at version 3. Each text's `style` becomes `Custom` with its
former values, written as `save_ron` writes it, `Custom(font: …, size: …, …)`, since the file
enables `unwrap_variant_newtypes`, except in the sample, where the texts name its styles as
"Builds" says. The fixture edited by an agent,
`path@rootpoint-model@tests/data/fixture-edited-by-agent.ron`, keeps the meaning its test states,
read at the audit.

## Audit subjects

- `TextContent`, `TextStyle` and `Document` in `path@rootpoint-model@src/document.rs`; every site of
  the mapping table below.
- `area_text`, `lay_out_area` and `render_slide` in `path@rootpoint-render@src/lib.rs`;
  `layout_text` in `path@rootpoint-render@src/text.rs`; `design@rootpoint-render@area-text-layout`.
- The grammar of `path@rootpoint-protocol@src/command.rs`, `Command`'s `Display`, and
  `design@rootpoint-protocol@protocol-transport`: a printed command parses back to itself.
- `design@rootpoint-model@text-style-per-area`, `design@rootpoint-protocol@protocol-addressing`,
  `design@rootpoint-model@stable-ids`, `rejected@rootpoint-model@content-by-reference`.
- `issue@rootpoint@rich-text-spans`, `issue@rootpoint-model@slide-templates`;
  `tripwire@rootpoint-render@render-within-bounds`, whose re-entry is this audit.

## Fails alone on

- A text naming a style draws other than the table's entry, or a style edit leaves an area unchanged.
- A command or an undo leaves an area naming a style the table lacks, which the property test finds.

## Premises that expire

- The window's style controls only fork a named area to custom here, through the `text` command.
  The style combo box and the Styles tab come with `spec@editor-conveniences@styles-window`.
  Guard: that slice's claims.
- The outline's row of an area with a missing style does not exist yet. AC1's third place is judged
  at `spec@editor-conveniences@styles-window`.

## Mapping tables

Every source site that reads or writes a text's style, by
`grep -rn "\.style\b\|style:" crates/*/src --include=*.rs` at the time of writing, and what it
becomes.

| site | today | becomes |
| --- | --- | --- |
| `path@rootpoint-model@src/document.rs`, `TextContent::style` | a `TextStyle` | a `StyleChoice` |
| `path@rootpoint-model@src/validate.rs`, `Document::validate_content` (moved into `Document::breaches` by `spec@editor-conveniences@breach-tiers`) | reads the font of `TextContent::style` | reads the font of the resolved style, and the names of `Named` |
| `path@rootpoint-model@src/ops.rs`, `asset_in_use` | the font of each text | the font of each `Custom` and of each style of the table |
| `path@rootpoint-render@src/lib.rs`, `render_slide`, the colour | `content.style.colour` | the resolved style's colour |
| `path@rootpoint-render@src/lib.rs`, `lay_out_area`, the vertical alignment | `content.style.v_align` | the resolved style's |
| `path@rootpoint-render@src/text.rs`, `layout_text`, the font and the style | `text.style` | a `&TextStyle` parameter |
| `path@rootpoint-protocol@src/session.rs`, the font loading of `render` | `t.style.font` | the resolved style's font |
| `path@rootpoint-protocol@src/session.rs`, `Command::Text` | the current `TextStyle`, or the default, from `Session::text_style` | the current `StyleChoice`; an option forks a `Named` to `Custom` |
| `path@rootpoint-protocol@src/session.rs`, `Session::text_style` | the text's `TextStyle`, or `TextStyle::default()` for an image area | the text's `StyleChoice`, or, for an image area, the style "The new area's style" gives every other new area |
| `path@rootpoint-protocol@src/session.rs`, `Command::Clear` | an empty text in `Session::text_style` | unchanged, on the `StyleChoice` that `Session::text_style` gives |
| `path@rootpoint-model@src/document.rs`, `impl Default for Content` | an empty text in `TextStyle::default()` | an empty text in `Custom(TextStyle::default())`; it reads no style table, so `apply_in` and `empty_area` of `path@rootpoint-model@src/ops.rs`, which hold the document, give a new area the style "The new area's style" decides |
| `path@rootpoint-protocol@src/session.rs`, `apply_style` | on a `TextStyle` | unchanged, applied to the values a fork copies |
| `path@rootpoint-protocol@src/command.rs`, `Command::Text::style` | `Vec<StyleOption>` | gains `StyleOption::Named` and `StyleOption::Custom` |
| `path@rootpoint-app@src/assets.rs`, the font of each text | `t.style.font` | the resolved style's font |
| `path@rootpoint-app@src/panels.rs`, `App::content` and `App::style` | the area's `TextStyle` | the resolved style; a change sends `text` with the option, which forks to custom |
| `path@rootpoint-app@src/actions.rs`, `App::set_text_content` and `App::set_style` | `text` with no option, or one | unchanged |
| `path@rootpoint-app@src/editing.rs`, the `text` command of an edit | no option | unchanged |
| `path@rootpoint-model@src/document.rs`, `Document::same_content` | compares the aspect, the slides and the assets | also compares `styles` |
| `path@rootpoint-model@src/file.rs`, `FileRef`, `File` and the construction in `load_ron` | the document's fields, field by field | gain `styles` |
| `path@rootpoint-model@tests/ops.rs`, `op_for` of the property test | reaches every `Op` variant, as its comment says | reaches the four style operations |
| `path@rootpoint-app@src/panels.rs`, `SIZES` | the sizes of the size menu | gains 28, Legend's size |

## Decided design

**A table of named text styles, and each text names one or holds its own**,
`thread@editor-conveniences@named-styles`. A text's style is `Named(name)` or `Custom(values)`.
Editing a named style changes every text that names it. That is its purpose, which the nearest
rival, a preset copied into the area with no link, cannot give
(`argument@editor-conveniences@arg-20`). The other rival, overrides per field over a named style,
needs every field optional and a cascade that the per-area design avoids. "custom" covers the case
it serves. The costs that `rejected@rootpoint-model@content-by-reference` names for shared content
are answered for styles (`argument@editor-conveniences@arg-19`):
- a style no area uses is a library entry;
- a change to a shared style is the point, and only the Styles tab of the window makes one;
- the file reads the name in place.

A style option given to a named text forks it to custom, so no command but `style set` changes a
shared style.

**Styles are named by their name**, `thread@editor-conveniences@style-key`. The file reads
`style: Named("Title")`, and a command writes `style="Title"`. A rename is one operation that
rewrites every reference, and its inverse renames back. On the owner's word: reference by name is
important to the owner, who chose it over ids from the counter, with which a rename touches nothing
else (`argument@editor-conveniences@arg-22`).

**Named styles are for text only**, `thread@editor-conveniences@style-kinds`. An image's one option,
`fit`, needs no name. A kind with text still to come, `issue@rootpoint@more-content-kinds`, decides
then whether it shares this table.

**Removing a style in use takes a replacement**, `thread@editor-conveniences@style-removal`: another
named style, or a custom copy of the removed one for each user, which changes no slide. With
neither, the removal is refused and its reply names the users. The owner proposed the replacement
(`argument@editor-conveniences@arg-46`). The rival, a removal that leaves its users drawn with the
default style, leaves warnings to clear after a deliberate act (`argument@editor-conveniences@arg-38`).

**A new area takes the style of the area it was split from, or Body**,
`thread@editor-conveniences@new-area-style`. Splitting a body text for a second column gives a body
text. Every other new area takes Body. T4 watches the case where inheriting gets in the way.

**A missing style draws with the default and is reported**,
`thread@editor-conveniences@missing-style-fallback`, a cosmetic breach under
`thread@editor-conveniences@cosmetic-fallbacks`.

## Implementation sequence

1. The table, `StyleChoice`, resolution and the breach, with every site of the mapping table; the
   fixtures rewritten; no command yet. Fails alone on the render and model tests of resolution.
2. The style operations and their inverses. Fails alone on the model tests of edit, rename and
   removal.
3. The commands, `tree`'s output, the fork to custom, and the new area's style. Fails alone on the
   protocol tests.
4. The sample, and the property test widened. Fails alone on AC4.

## Acceptance criteria

This slice judges `acceptance@editor-conveniences@ac-no-command-breach` (AC4), and reports it in its
landing commit.

## Harvest

- `design@rootpoint-model@text-style-per-area` rewritten: a text has one style for all its text,
  named or custom. `thread@editor-conveniences@style-key` and `thread@editor-conveniences@style-kinds`
  go to the heads they name. The table, the fork to custom,
  the replacement on removal and the new area's style are cut into heads by `primer@design-heads`.
- `design@rootpoint-protocol@protocol-addressing` rewritten: a command names a style by its name. Its
  argument, an id that names one thing after every edit, does not cover a name a rename changes, so
  this is a change of the decision, argued by `thread@editor-conveniences@style-key`; the title
  widens to state both.
- `design@rootpoint-model@inline-content-assets` rewritten: a text's style may name an entry of the
  document's style table, the one content shared by name, argued against
  `rejected@rootpoint-model@content-by-reference` in "Decided design" above.
- T4, recorded on the owner's word in the tripwires home of the Component that holds the head of the
  new area's style, guarding it, with its label in its text.
- A `rejected` entry for ids from the counter as style keys, `style-ids`, if a test of
  `skill@knowledge-architect-decision-recording@losing-alternatives` admits it. Test 1 is the
  candidate: the rival would change the file format.
- `issue@rootpoint@rich-text-spans` re-read: its "What" names `TextStyle` per area, which becomes
  `StyleChoice`.
