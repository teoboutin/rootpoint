---
kind: todo
---
# Copy and paste carry a text's characters alone, and drop its spans

## Summary

While a text is edited in place, a copy or a cut puts the selection's characters on the clipboard,
and a paste inserts plain text into the caret's text node. A span copied with the selection, its
style and its named span style, is not carried: pasted elsewhere, the text takes the style of the
node it lands in.

## Details

### What

`App::cut`, and the `Copy` event of `App::editing_keys` through `App::selected_text`, give the
clipboard `editing::range_text`, the characters of the selected range, a span's included, with
nothing of its style; `Event::Paste`
reaches `App::type_str`, which sends one `insert` of the pasted string. So a bold word copied from
one area and pasted into another arrives plain. The milestone on rich text chose this for its slice
of the tree caret, its default D3, which the owner accepted, and named styled copy and paste a
later consequence.

Leads, none decided:

- the clipboard carries the selection as the one-line tree of `text`,
  `design@rootpoint-protocol@protocol-transport`, beside its plain text, and a paste inside the
  editor reads the tree back; a paste from another program keeps reading plain text;
- a paste of a tree into a text node is a new operation, or a sequence of `insert` and `wrap`,
  each span given a new id, as `Op::SetText` gives them;
- a span naming a span style the target document lacks needs a rule, as
  `design@rootpoint-model@cosmetic-fallbacks` gives a missing style.

### Why it matters

An author who restyles a phrase and moves it by cut and paste loses the styling, and makes it
again by hand, against `goal@rootpoint-app@direct-manipulation`.

### What would close it

A design of styled copy and paste under `skill@knowledge-architect-design`, built, with a test in
`path@rootpoint-app@tests/editing.rs` that cuts a selection holding a span and pastes it into
another text, and finds the span with its style there.
