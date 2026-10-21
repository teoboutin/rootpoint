# Goals of rootpoint-app

What rootpoint-app is for, and what would show it achieved: one level-two heading per goal. A goal
is met or unmet. It stays here while it is met, and it leaves only on the owner's word. How
rootpoint-app is built is in `register@rootpoint-app@design`.

## An author builds a presentation in the window by direct manipulation of its slides `##direct-manipulation`

The window is where an author makes a presentation: they divide areas, resize them, set their
content and reorder slides with the pointer and the keyboard on the slide itself, and see each
change at once. It refines `goal@rootpoint@no-language-no-build` and
`goal@rootpoint@free-slide-division`. It is met when the author, in the window and without typing
a command, divides areas and resizes them, selects any node, swaps and moves areas, sets each
area's content and its style, adds, removes, duplicates and reorders slides, and undoes and redoes
each of these, and each effect shows in the next frame.

## The window shows the slide as every other display does `##faithful-slide-view`

The slide the window shows, in the editor and in presenter mode, is the slide the renderer draws,
with no editor mark in it, at any window size. It refines
`goal@rootpoint-render@same-slide-everywhere`. It is met while presenter mode's image of a slide
equals the editor's at the same size, and the window names every image, font or character it
could not draw as the document asks.

## A talk is given from the window, full screen `##presents-full-screen`

The window shows a presentation full screen, slide by slide, moved by the keys a presenter uses.
It refines `goal@rootpoint@presenter-mode`. It is met when the author gives a talk from the window
alone.

## A running window shows live what an agent's commands do `##live-agent-edits`

An agent drives the open window through the text protocol, and the author sees each change as it
is made. It refines `goal@rootpoint@text-command-protocol`. It is met when an LLM session builds
and changes a presentation while the window shows the result live.

## The window runs in a web browser `##browser-window`

The same editor and presenter mode run in a browser build. It refines
`goal@rootpoint@runs-in-browser`. It is met when the browser build edits and presents the built-in
sample with the same actions as the native window.
