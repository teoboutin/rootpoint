---
kind: todo
---
# The editor's keys cannot be configured

## Summary

Every key binding of the window is fixed in the code. The owner wants the bindings configurable
later, the LyX math keys included.

## Details

### What

The window's shortcuts are tested in `App` of `path@rootpoint-app@src/lib.rs`: outside editing by
calls of egui's `consume_shortcut` and `consume_key` in `App::shortcuts`, and while a text is edited
by match arms in `App::editing_keys`, such as Ctrl+B calling `App::bold`; the browser's listener,
`keep_editor_keys` of `path@rootpoint-app@src/web.rs`, keeps a list of its own. `milestone@plans@math-support` adds the LyX math keys, Ctrl+M and the Alt+M
sequences, as a built-in table from key sequences to actions, with no file that changes it.

Leads, none decided:

- the table of the milestone is the one place a configuration would fill: a loader that reads a file
  into it, and the existing shortcuts moved into the same table;
- where the file lives, natively and in the browser build, which reads no file of its own, is open;
- LyX's bind files, `path@elsewhere@lib/bind/cua.bind` and the files it includes, are a model of
  such a file: a binding per line, a key sequence then a function. They are read in
  `path@rootpoint@docs/grounding/math-typesetting.md`, section 1.

### Why it matters

An author whose keyboard layout, platform or habits conflict with a fixed binding cannot change it.
An assumption, not measured: on a Mac, Option with M types µ, and LyX's `mac.bind` moves Alt to
Control by its comment, which the survey read and did not test.

### What would close it

The bindings of the window read from a configuration the author can change, natively and in the
browser build, and every existing shortcut moved into the table.
