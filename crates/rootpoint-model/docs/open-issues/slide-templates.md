---
kind: deferred
---
# A background, a footer or a layout shared by several slides is set up on each by hand

## Summary

The prototype has no templates: a background, a reserved area such as a footer, or a predefined
layout is repeated by duplicating a slide. T4 of the premortem of the first prototype, recorded on
the owner's word, watches for the cost.

## Details

### What

The owner listed three cases of consistency across slides: a common background, a reserved area on
every slide, and reusable layouts. One mechanism covers all three: a template is a slide holding a
background and a split tree, some of whose areas are marked as the slide's body; a slide based on
it fills the body areas, and editing the template changes every slide using it. The model admits
it later by an optional template id on a slide. Today a slide is duplicated
(`DuplicateSlide`), which copies and does not link.

### Why it matters

Without templates, a change of the footer or the background is made on every slide, which costs
the author time in proportion to the slide count, against `goal@rootpoint@real-presentations`.

### Trigger

Preparing a real talk with the prototype needs the same background, footer or layout on several
slides beyond what duplicating a slide gives. The owner's work on that talk meets the need, and
the response is a design session on templates, which this entry opens with the shape above.
