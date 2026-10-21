# Goals of rootpoint-render

What rootpoint-render is for, and what would show it achieved: one level-two heading per goal. A
goal is met or unmet. It stays here while it is met, and it leaves only on the owner's word. How
rootpoint-render is built is in `register@rootpoint-render@design`.

## A slide looks the same at every size and on every machine `##same-slide-everywhere`

The renderer draws a slide from one layout in slide units, so the editor, presenter mode and a
saved image show the same slide, with the same line breaks, at any window size and on any computer
the presentation is opened on. It refines `goal@rootpoint@presenter-mode`. It is met while a slide's
text breaks at the same places at every size, and its fonts come only from the program and the
presentation.

## The renderer builds for the browser `##browser-buildable`

The renderer is part of the browser build, so it uses nothing that exists only on a desktop and
reads no file. It refines `goal@rootpoint@runs-in-browser`. It is met while the gate `wasm` builds
this crate for the browser's target.
