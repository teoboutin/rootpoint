---
kind: deferred
---
# How the protocol's scripts and the crates' callers stay compatible is not decided

## Summary

`design@rootpoint@grammar-and-api-stay-free` keeps no form of an earlier version of the protocol's
grammar or of a crate's public API, until the owner rules otherwise. When the owner does, how each
of the two interfaces stays compatible is not decided.

## Details

### What

The protocol's commands and the crates' public API carry no version. An agent's script of protocol
commands, or a program built on a crate, may break at any change of either. Shapes to weigh when
the question opens: a refusal that names the conversion, versioned commands, a deprecated form
kept for a number of versions.

### Why it matters

An agent's script that drives the editor, `goal@rootpoint@text-command-protocol`, or a program
built on a crate once one is published, breaks at an update with no notice, which loses the work
the script or the program does.

### Trigger

The owner's word that the protocol or a crate is to be kept compatible, or the owner's word to
publish a crate on crates.io, `issue@rootpoint@publish-to-crates-io`: either opens a design session
on how the interface concerned stays compatible.
