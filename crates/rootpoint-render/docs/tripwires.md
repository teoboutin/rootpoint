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
