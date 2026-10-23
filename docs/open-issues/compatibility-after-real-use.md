---
kind: deferred
---
# How files, scripts and callers stay compatible once the project is in real use is not decided

## Summary

`design@rootpoint@no-compat-in-initial-dev` keeps nothing compatible until the owner rules that the
project is in real use. From that ruling, a change of the save format, of the protocol's grammar or
of a crate's public API must keep what came before working, and how is not decided.

## Details

### What

A saved file names its format's version first, and a file of another version is refused,
`design@rootpoint-model@save-format`. The protocol's commands and the crates' public API carry no version. Nothing reads an older format,
accepts an older command or keeps an older function, by that decision.

### Why it matters

Once the owner prepares real talks, `goal@rootpoint@real-presentations`, a change that refuses the
owner's own files, or breaks an agent's script, loses work the project exists to keep.

### Trigger

The owner rules that the project is in real use. The response is a design session on how each of
the three interfaces stays compatible: a migration of old files on load, a refusal that names the
conversion, or versioned commands, among the shapes it weighs.
