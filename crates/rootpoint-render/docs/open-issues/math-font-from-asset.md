---
kind: todo
---
# A presentation cannot draw its formulas in a math font of its own assets

## Summary

Every formula is laid out from the MATH table of the built-in Libertinus Math and drawn with its
glyphs. A presentation whose author wants another math font, as a font asset it holds, has no way to
name it.

## Details

### What

`design@rootpoint-render@builtin-libertinus-math` registers Libertinus Math in every `FontSet`, and
`rootpoint_render::area_formula` and the formulas of `area_text` lay out every formula from it. A
font asset, which a text style names for its text, is never read as a math font: no option of an
equation or of a style names one, and the math crate reads one font's MATH table, `MathFont`.

The design discussion of the milestone on math named a math font from a font asset as later work,
to be held under a tripwire; its premortem proposed none and the owner ruled on none, so this entry
records the work, and watches for no need of it.

Leads, none decided:

- where the font is named: an option of the equation, a value of the text style that a formula of
  the text takes, or one choice for the whole presentation;
- the table of commands, `path@rootpoint-math@src/commands.rs`, splits its commands by whether the
  built-in font has their character, `design@rootpoint-math@offered-commands-drawn`; a font of the
  presentation's own would have another split, so the palette's list would follow the font;
- a font asset with no MATH table, or a broken one, falls back to the built-in font, as a text's
  font falls back to the built-in sans.

### Why it matters

A presentation's formulas all look like Libertinus Math, whatever font its text is set in; an
author whose talks match another typeface, as a TeX document set in Latin Modern, cannot match
them. It strains `design@rootpoint-render@builtin-libertinus-math`, which makes the built-in font
the only one.

### What would close it

A design session on which math font a formula is drawn in and where it is named, and its work
built: a formula of a presentation that names a font asset with a MATH table is laid out from that
table, by a test of the render crate.
