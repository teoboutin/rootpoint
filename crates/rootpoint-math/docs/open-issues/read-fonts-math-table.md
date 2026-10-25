---
kind: deferred
---
# The MATH table is parsed by ttf-parser, beside the read-fonts that parley links

## Summary

The math crate reads a math font's OpenType MATH table with ttf-parser,
`design@rootpoint-math@math-layout-engine`, because read-fonts, which parley already links, parses
that table only from version 0.45. Once parley moves to such a
read-fonts, one font parser could serve both.

## Details

### What

`MathFont` of `path@rootpoint-math@src/font.rs` reads the whole math font with ttf-parser 0.25.1:
its cmap, its glyphs' advances and bounds, its MATH table by `ttf_parser::math`, and its `ssty`
lookups by `ttf_parser::gsub`. The renderer links read-fonts 0.41 through parley, which has
`FontRef::table_data` and no MATH module. read-fonts 0.45.0, released 2026-10-03, has a MATH module
of its own; parley 0.12.0, through fontique 0.12.0, and harfrust 0.12.0 require read-fonts
`^0.41.0`, so using read-fonts 0.45 now would link two copies of it. Read in the survey
`path@rootpoint@docs/grounding/math-typesetting.md`, section 2.

### Why it matters

Two font parsers are in every build, the browser build included, while one would do. The cost is
small: ttf-parser has no required dependency.

### Trigger

The change that moves the render crate to a parley whose dependencies require read-fonts 0.45 or
later: that change already settles which read-fonts the crate links, and moving the MATH parsing
to it is then a change of the same dependency.
