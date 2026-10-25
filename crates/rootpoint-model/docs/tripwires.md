# Tripwires of rootpoint-model

Evidence that would flip a recorded decision of rootpoint-model: one level-two heading per
tripwire, each naming its decision. What is outstanding is in `register@rootpoint-model@issue`.

## Guarding `design@rootpoint-model@split-tree-arity`: merging same-axis splits removes a grouping the owner wants `##grouping-lost-to-merge`

T1 of the premortem of the first prototype, recorded on the owner's word.

**Fires when:** in real use, the owner wants to move or resize a group of same-axis areas as one
unit, and the merge has made that group impossible.
**Response:** reopen `design@rootpoint-model@split-tree-arity`, for example with an explicit group
node.
**Re-entry:** the first real talk given with the prototype; and the standing-state review before
every merge.

## Guarding `design@rootpoint-model@new-area-style`: inheriting the split area's style mostly gets in the way `##inherited-style-in-the-way`

T4 of the premortem of the milestone on the conveniences of the editor, recorded on the owner's word.

**Fires when:** the owner reports changing the new area's style right after most splits of a
styled area, as when a Title is split to add a logo.
**Response:** reopen `design@rootpoint-model@new-area-style`, toward a new area that always takes
Body.
**Re-entry:** the standing-state review before every merge.

## Guarding `design@rootpoint-model@stable-ids`: a span's id or syntax gets in the way of editing a file by hand `##span-ids-in-the-way`

T3 of the premortem of the milestone on rich text, recorded on the owner's word.

**Fires when:** the owner, or an agent the owner asked, edits a saved text tree by hand and reports
once that the spans' ids or their syntax got in the way.
**Response:** reopen `design@rootpoint-model@stable-ids` for spans, toward an id a hand-written span
may leave out and the loader gives.
**Re-entry:** the next design work on the save format; and the standing-state review before every
merge.

## Guarding `design@rootpoint-model@named-span-styles`: two tables of styles confuse the author `##span-styles-confuse`

T4 of the premortem of the milestone on rich text, recorded on the owner's word.

**Fires when:** the owner reports, more than once after the Styles tab lists the span styles,
looking for a style in the wrong table, or applying a text style where a span style was meant.
**Response:** reopen `design@rootpoint-model@named-span-styles`, toward one table of styles whose
fields are all optional, `rejected@rootpoint-model@one-table-of-optional-styles`.
**Re-entry:** the design audit of the next work on styles; and the standing-state review before
every merge.

## Guarding `design@rootpoint-model@equation-fit`: the owner holds most equations at a size `##at-most-for-most-equations`

T1 of the premortem of the milestone on math, recorded on the owner's word.

**Fires when:** while preparing one real talk, the owner sets `AtMost` on more than half of its
equations.
**Response:** reopen `design@rootpoint-model@equation-fit`, toward `AtMost` as the default of a new
equation.
**Re-entry:** the first real talk given with math; and the standing-state review before every
merge.
