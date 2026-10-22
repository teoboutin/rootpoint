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

T7 of the premortem of the first prototype, recorded on the owner's word.

**Fires when:** in real use, an agent answers the refusal that says the author took control with
`control`, before the author gave it new instructions, more than once; the count is a default, the
owner's to reset.
**Response:** reopen `design@rootpoint-app@control-passing`, toward a button "Give control" the
author clicks to hand back, `rejected@rootpoint-app@give-control-button`.
**Re-entry:** the first real talk prepared with an agent.
