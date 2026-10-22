# Tripwires of rootpoint-render

Evidence that would flip a recorded decision of rootpoint-render: one level-two heading per
tripwire, each naming its decision. What is outstanding is in `register@rootpoint-render@issue`.

## Guarding `design@rootpoint-render@font-sources`: a needed glyph is missing from every available font `##glyph-missing-everywhere`

T2 of the premortem of the first prototype, recorded on the owner's word.

**Fires when:** a real slide needs a glyph that no built-in font covers, and adding a font file to
the presentation is not an acceptable fix for the owner. `Report::missing_glyphs` names the areas.
**Response:** reopen `design@rootpoint-render@font-sources`.
**Re-entry:** the first real talk given with the prototype; and the standing-state review before
every merge.

## Guarding `design@rootpoint-render@slide-renderer`: a frame of a divider drag or a 4K render takes longer than its bound `##render-within-bounds`

AC2 of the first prototype, which did not fire when its slices measured it, recorded as a tripwire
on the owner's word when the milestone closed, put to the owner as T8.

**Fires when:** in a release build on the owner's machine, any frame of a divider drag in a 1600×900
editor view takes more than 33 ms, by
`cargo test --release -p rootpoint-app --test presenter -- --ignored --nocapture`, or any render of
one 3840×2160 slide with text and an image takes more than 200 ms, by
`cargo test --release -p rootpoint-render --test text -- --ignored --nocapture`. The bounds are
defaults the session proposed and the owner kept, the owner's to reset.
**Response:** reopen `design@rootpoint-render@slide-renderer`, with GPU vello or vello_hybrid as the
candidate.
**Re-entry:** the design audit of any work that adds a content kind or changes the renderer; and
the standing-state review before every merge.
