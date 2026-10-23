---
kind: todo
---
# The undo history cannot be read, browsed or branched

## Summary

A `Session` keeps its undo history as two stacks of steps, which only `undo` and `redo` walk, one
step at a time. A new edit after an undo clears the redo stack, so the undone branch is lost. The
author cannot see what the history holds, nor fetch an earlier step and apply it again.

## Details

### What

`Session` holds `undo` and `redo`, each a `Vec<Step>`, a step being the operations that revert one
command or one group, in `path@rootpoint-protocol@src/session.rs`. Every command that applies an
operation clears `redo`, inside a group or not. So after "do A, undo, do
B", the step that would redo A is gone, and nothing lists the steps either stack holds.

The owner named three improvements, as open leads, none designed:

- a view of the history: what each step did, in words an author reads, such as "split n4 beside",
  "text of n3", "removed n6";
- fetching an earlier step and applying it again, or trying to, when the document has changed
  since, such as bringing back a removed area's text;
- keeping the branches of the history, so that "do A, undo, do B, undo, redo A" reaches A again.

The owner proposed this in place of a confirmation dialog before removing a node that holds
content. That dialog was withdrawn, because undo already restores what a removal took.

### Why it matters

Undo is what protects content from being lost by mistake, against `goal@rootpoint@real-presentations`.
A removal noticed only after another edit has cleared the redo stack, or a step the author cannot
find among many, is lost content all the same. Every step is a command of the protocol, so the
history is the session's, per `design@rootpoint-protocol@view-state`, and a view of it is a view
`goal@rootpoint-protocol@commands-drive-everything` asks to be a command.

### What would close it

A design session on the history, under `skill@knowledge-architect-design`, that decides which of
the three leads to build, the protocol commands that read and apply the history, and how a step is
described, and its work built, with tests that run headless.
