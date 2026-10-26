# Tripwires of rootpoint

Evidence that would flip a recorded decision of rootpoint: one level-two heading per tripwire, each
naming its decision. What is outstanding is in `register@rootpoint@issue`.

## Guarding `design@rootpoint@saved-presentations-stay-loadable`: a presentation saved before an update fails to load after it, or loads as another `##saved-file-breaks-on-update`

T1 of the premortem on compatibility, recorded on the owner's word.

**Fires when:** a presentation the owner saved with one version of rootpoint, as a workspace or as
a pack, of format version 7 or later, is refused by a later version, or loads to a presentation
that differs from the one saved: an area, a text, a style, an equation or an asset missing or
changed.
**Response:** open a defect for the file, and reopen
`design@rootpoint@saved-presentations-stay-loadable` on how its frozen files guard it: what the
failing file held that the frozen file of its version does not.
**Re-entry:** at its firing; and the standing-state review before every merge.
