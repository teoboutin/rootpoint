---
kind: todo
---
# The editor's keys cannot be configured

## Summary

Every key binding of the window is fixed in the code. The owner wants the bindings configurable
later, the LyX math keys included.

## Details

### What

The window's shortcuts are match arms in `App`, in `path@rootpoint-app@src/lib.rs`, such as Ctrl+B
calling `App::bold`. `milestone@plans@math-support` adds the LyX math keys, Ctrl+M and the Alt+M
sequences, as a built-in table from key sequences to actions, with no file that changes it.

Leads, none decided:

- the table of the milestone is the one place a configuration would fill: a loader that reads a file
  into it, and the existing shortcuts moved into the same table;
- where the file lives, natively and in the browser build, which reads no file of its own, is open;
- LyX's bind files, `path@elsewhere@lib/bind/cua.bind` and the files it includes, are a model of
  such a file: a binding per line, a key sequence then a function. They are read in
  `path@rootpoint@docs/grounding/math-typesetting.md`, section 1.

### Why it matters

An author whose keyboard layout, platform or habits conflict with a fixed binding cannot change it:
on a Mac, Option with M types µ, which LyX itself moves to Control with M.

### What would close it

The bindings of the window read from a configuration the author can change, natively and in the
browser build, and every existing shortcut moved into the table.
