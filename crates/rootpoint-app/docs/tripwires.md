# Tripwires of rootpoint-app

Evidence that would flip a recorded decision of rootpoint-app: one level-two heading per tripwire,
each naming its decision. What is outstanding is in `register@rootpoint-app@issue`.

## Guarding `design@rootpoint-app@control-passing`: the socket's request granted at once ends a gesture the author had not finished `##control-ends-a-gesture`

T6 of the premortem of the first prototype, recorded on the owner's word.

**Fires when:** in real use, the owner reports, more than once, that an agent's `control` ended a
stay in editing or a drag the owner had not finished.
**Response:** reopen `design@rootpoint-app@control-passing`, toward the request granted only while
the window has no gesture open, `rejected@rootpoint-app@control-when-idle`.
**Re-entry:** the first real talk prepared with an agent.

## Guarding `design@rootpoint-app@control-passing`: an agent takes the authority back after the author took it `##agent-takes-control-back`

T7, which the owner proposed in answer to D6 of the first prototype, recorded on the owner's
word.

**Fires when:** in real use, an agent answers the refusal that says the author took control with
`control`, before the author gave it new instructions, more than once; the count is a default, the
owner's to reset.
**Response:** reopen `design@rootpoint-app@control-passing`, toward a button "Give control" the
author clicks to hand back, `rejected@rootpoint-app@give-control-button`.
**Re-entry:** the first real talk prepared with an agent.

## Guarding `design@rootpoint-app@edit-entry`: a second click on a new, empty area enters editing, and the keys go to its text `##empty-text-takes-the-keys`

T3 of the premortem of the milestone on the conveniences of the editor, recorded on the owner's
word.

**Fires when:** the owner reports, more than once, that a second click on a new area, whose text is
empty, entered editing, and that Delete or the node keys then went to the empty text instead of
acting on the area.
**Response:** reopen `design@rootpoint-app@edit-entry`, toward an empty text that enters editing
only when the author types.
**Re-entry:** the standing-state review before every merge.

## Guarding `design@rootpoint-app@node-selection`: the node keys still confuse the owner with the outline in view `##keys-confuse-with-the-outline`

T1 of the premortem of the milestone on the conveniences of the editor, recorded on the owner's
word.

**Fires when:** the owner reports, more than once after the outline landed, that Alt with an arrow
moved the selection where the owner did not expect.
**Response:** reopen `design@rootpoint-app@node-selection`, toward the spatial keys, Alt with an
arrow selecting the neighbouring area in that direction, as i3's `focus left`, `focus right`,
`focus up` and `focus down` do, with the parent and the child on other keys such as Alt+Page Up and
Alt+Page Down; or toward Alt with ↑ and ↓ moving one row of the outline.
**Re-entry:** the design audit of the next work on the app's keys, or earlier on the owner's word;
and the standing-state review before every merge.
