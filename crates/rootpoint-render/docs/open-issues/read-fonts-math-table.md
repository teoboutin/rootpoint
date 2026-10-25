---
kind: deferred
---
# The MATH table is parsed by ttf-parser, beside the read-fonts that parley links

## Summary

The renderer reads a math font's OpenType MATH table with ttf-parser, because read-fonts, which
parley already links, parses that table only from version 0.45. Once parley moves to such a
read-fonts, one font parser could serve both.

## Details

### What

`milestone@plans@math-support` adds ttf-parser 0.25 to `path@rootpoint-render@Cargo.toml` and parses
the MATH table's bytes with `ttf_parser::math::Table::parse`, the bytes being those read-fonts 0.41
returns from `FontRef::table_data`. read-fonts 0.45.0, released 2026-10-03, has a MATH module of
its own; parley 0.12.0, through fontique 0.12.0, and harfrust 0.12.0 require read-fonts `^0.41.0`,
so using read-fonts 0.45 now would link two copies of it. Read in the survey
`path@rootpoint@docs/grounding/math-typesetting.md`, section 3.

### Why it matters

Two font parsers stay in every build, the browser build included, while one would do. The cost is
small: ttf-parser has no required dependency.

### Trigger

The change that moves the render crate to a parley whose dependencies require read-fonts 0.45 or
later: that change already settles which read-fonts the crate links, and moving the MATH parsing
to it is then a change of the same dependency.
