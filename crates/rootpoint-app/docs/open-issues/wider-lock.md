---
kind: todo
---
# The authority to edit is one lock over the whole document

## Summary

The window and its control socket hold the authority to edit one at a time, over the whole
document, `design@rootpoint-app@one-controller-at-a-time`. Several clients each with control of
their own, or a lock per slide or per node, are not designed.

## Details

### What

A wider lock lets an agent edit one slide while the author edits another, or several agents work
at once. `goal@rootpoint@minimal-content-set` neither asks for it nor forbids it, and keeps out of
scope any editing that reconciles two changes made at once to one part. In the owner's view, the
slide's tree of splits gives a clear hierarchy of which content and which changes can be applied
with no risk of conflict, from which such a design starts; the owner thought a survey of prior art
would become worth running once the scope expands so.

Such a lock is inside one window: the lock of a workspace,
`design@rootpoint-protocol@workspace-lock`, stays one per process whatever it becomes.

### Why it matters

`goal@rootpoint-app@live-agent-edits` shares the window with one agent at a time; an author who
wants to keep editing while an agent works waits for the agent's `release`.

### What would close it

A design session on a wider lock, and its work built; or the owner's word that one lock is
enough.
