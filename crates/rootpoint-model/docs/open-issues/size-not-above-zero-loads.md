---
kind: defect
---
# A file with a size not above 0 loads, and the session then prints a line that does not parse

## Summary

The grammar refuses a text size, a span's size and an equation's size that is not a number above 0,
and the session refuses one built through the API. A file edited by hand that holds one loads, and
`styles`, `span-styles` or `tree` then print it as an option the grammar refuses.

## Details

### What

`Document::validate` of `path@rootpoint-model@src/validate.rs` checks no size, so `load_ron` loads
a style of the table, a text's own values, a span's values or an equation's
`EquationSize::AtMost` whose size is 0, below 0, or not finite. Reproduced in a probe test of the
protocol crate, not committed: a document whose Body style has size 0, saved and opened, gives
`styles` the line `"Body" size=0 ...`, and `parse_command("style set \"Body\" size=0")` replies
`` `size=0` is not a style option ``. The review of the slice #equation-kind of
`milestone@plans@math-support` found the same for an equation: a file holding
`size: AtMost(0.0)` loads, and `tree` prints `size=0`, which `parse_command` refuses for
`equation`.

The renderer does not panic on such a size: a probe rendered an equation held at sizes 0, -5, NaN
and infinity, and returned with no panic.

### Why it matters

`design@rootpoint-protocol@protocol-transport` says a printed `Command` is a line that parses back
to it, so a session can be written out as a script and run again; the lines `styles` and `tree`
print are what an agent copies into its next command. A file edited by hand is a purpose,
`goal@rootpoint-model@faithful-readable-file`, so such a size is reachable.

### What would close it

A decision, by `design@rootpoint-model@validated-documents` and
`design@rootpoint-model@cosmetic-fallbacks`, of whether a size not above 0 in a file is structural,
refused by `load_ron`, or cosmetic, drawn with a stand-in and named by `Document::breaches`; built,
with a test that loads a file holding such a size in each of these four places and finds it
refused, or finds every line the session prints for it parsing.
