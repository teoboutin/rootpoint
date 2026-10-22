---
kind: todo
---
# Slides cannot be exported as images

## Summary

The protocol's `render` command writes one slide as a PNG, but the editor has no export of a
presentation's slides, and no PDF export exists.

## Details

### What

An image export reuses `render_slide` of the render crate, as the protocol's `render` does, for
every slide of a presentation, from the editor or a command.

### Why it matters

`goal@rootpoint@presenter-mode` keeps an export secondary but admits one, images of the slides or a
PDF, for sharing a talk with people who do not run the tool.

### What would close it

An export of every slide as images, by the protocol and the editor, with a test that each image
equals `render_slide`'s.
