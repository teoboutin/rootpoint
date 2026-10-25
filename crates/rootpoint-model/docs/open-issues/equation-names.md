---
kind: todo
---
# An equation cannot be given a name that references read in place

## Summary

A reference to an equation names the equation's area by its id, `n<id>`, in the file and in
commands, so a reader of the file looks the id up to know which equation is meant. The owner wants
equations to be nameable, as LyX labels them, with the name set in the side panel.

## Details

### What

`milestone@plans@math-support` builds references to equations by area id: a text's reference item
reads `Ref(7)` in the file, where an id is a plain number, and `{ref n7}` in commands, and draws as
the equation's number, `design@rootpoint-model@equation-reference-target`. The design
discussion of that milestone weighed names, as LyX's `\label{eq:energy}` and `reference "eq:red"`
are, against ids, and the owner chose ids for a first shape, and asked that naming stay open as
this entry.

Leads, none decided:

- on the owner's word, the name is a property of the equation kind, set in the side panel's
  Selection tab, where a text's style and an image's fit are set;
- a name would need a uniqueness rule among equations, and a rename operation that rewrites every
  reference naming it, as `design@rootpoint-model@style-key` does for styles, where the owner chose
  names over ids, judging that referring by name is important;
- a name could be optional, a reference naming the equation by its name where it has one and by its
  id otherwise, or every reference could keep the id and the name only label the equation in the
  file and in the picker;
- LyX's labels and references are read in the survey
  `path@rootpoint@docs/grounding/math-typesetting.md`, section 1.

### Why it matters

A presentation edited by hand, or read by an agent, shows `Ref(7)` where `Ref("energy")` would say
which equation is meant, which bears on `goal@rootpoint@readable-save-file`.

### What would close it

A design session on equation names, and its work built, or the owner's word that ids are enough.
