---
kind: design
---
# Alt and the arrows move through the tree, where the owner expects them to move on the slide

## Summary

Alt with an arrow selects the parent, a child or a sibling of the selection, whatever way the
split runs, per `design@rootpoint-app@node-selection`. The owner, using the editor, expected Alt+↓
to move down on the slide, and found it sometimes did and sometimes did not. The owner chose to
keep the tree navigation for now and to discuss the keys again.

## Details

### What

On the sample's first slide, a vertical split n4 holding the title n2 above a row n6, which holds
n3 and n5 side by side, the keys do this, measured with no window by driving `App::frame`:

- from n2, Alt+↓ does nothing, since an area has no child, and Alt+→ selects n6, the whole bottom
  row, which is n2's next sibling in a split that stacks;
- from n6, Alt+↓ selects n3, its first child or the one selected last in it;
- from n4, Alt+↓ selects the child selected last in it, n2 or n6 by the history.

So Alt+↓ reaches the bottom row from the root when the row was the last child selected, and never
from the title itself. The owner, thinking of the arrows as directions on the slide, found that it
moved down most of the time. The owner first suspected a link with leaving a text with
Esc; the measurement shows none: Esc leaves the selection where it was.

Instances seen: the owner's report while judging AC1 of the slice that edits text in place, on the
sample's first slide. Suspected mechanism: the arrows name directions, and the tree's parent,
children and siblings only match those directions in a split that runs the same way.

### Why it matters

A key that moves the selection somewhere the author did not expect costs a look and another key
each time, against `goal@rootpoint-app@direct-manipulation` and `goal@rootpoint@free-slide-division`,
which ask that areas be divided and reached conveniently. It strains
`design@rootpoint-app@node-selection`, whose keys AC6 checked reach the app, but not that they
move where an author expects.

### What would close it

A design session on the selection keys, under `skill@knowledge-architect-design`, that weighs the
tree navigation against a spatial one: Alt with an arrow selecting the neighbouring area in that
direction, as i3's `focus left`, `focus right`, `focus up` and `focus down` do, with the parent and
the child on other keys, such as Alt+Page Up and Alt+Page Down. It ends by keeping
`design@rootpoint-app@node-selection` with this case argued in it, or by replacing it, with its
code and tests.

### Re-entry

The design audit of the next work that touches the app's keys, or earlier on the owner's word.
