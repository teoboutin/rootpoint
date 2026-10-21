# Goals of rootpoint-model

What rootpoint-model is for, and what would show it achieved: one level-two heading per goal. A
goal is met or unmet. It stays here while it is met, and it leaves only on the owner's word. How
rootpoint-model is built is in `register@rootpoint-model@design`.

## Every change of a presentation runs, and is undone, without a display `##headless-changes`

The model holds a presentation and applies every change the editor offers, with its undo, with no
window and no GUI dependency, so the protocol, the tests and the GUI share one implementation of
each change. It refines `goal@rootpoint@headless-core`. It is met while every change the editor
offers is an operation of this crate, and the crate's tests apply each of them with no display.

## The model builds for the browser `##browser-buildable`

The model is part of the browser build, so it uses nothing that exists only on a desktop. It
refines `goal@rootpoint@runs-in-browser`. It is met while the gate `wasm` builds this crate for the
browser's target.

## A saved presentation loads back the same, and stays readable `##faithful-readable-file`

The model writes a presentation as text a person or an agent can read and edit, and reads it back
to the same presentation. It refines `goal@rootpoint@readable-save-file`. It is met while every
presentation loads back from its saved text equal to what was saved, and a file edited by hand
within the format loads with the edits.
