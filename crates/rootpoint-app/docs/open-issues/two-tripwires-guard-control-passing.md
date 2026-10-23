---
kind: question
---
# Two tripwires guard the decision on passing control, where one entry per decision is asked

## Summary

`tripwire@rootpoint-app@control-ends-a-gesture` and `tripwire@rootpoint-app@agent-takes-control-back`
both guard `design@rootpoint-app@control-passing`. The issue-tracking skill asks that the tripwires
of one decision be absorbed into one entry. Whether to merge them is the owner's, since both are
recorded on the owner's word.

## Details

### What

The first fires when an agent's `control` ends a gesture the owner had not finished, and reopens the
decision toward `rejected@rootpoint-app@control-when-idle`; the second fires when an agent takes the
authority back after the author took it, and reopens it toward
`rejected@rootpoint-app@give-control-button`. They fire on different evidence and point at
different rivals, so a merged entry would hold two firing clauses and two responses.

### Why it matters

The project's tripwires are read by the standing-state review before every merge; two entries for
one decision cost one more entry to read, and disagree with the skill's shape for a tripwire.

### What would close it

The owner's word: merge the two into one entry with both clauses, or keep both, the skill's rule
then judged not to fit tripwires that fire on different evidence.
