# Goals of rootpoint

What rootpoint is for, and what would show it achieved: one level-two heading per goal. A goal is
met or unmet. It stays here while it is met, and it leaves only on the owner's word. How rootpoint
is built is in `register@rootpoint@design`.

## The owner gives real presentations with it `##real-presentations`

The tool is first for the owner's own talks, and may be published later. It is met when the owner
uses it for real presentations, including ones that need mathematics, rather than another tool.
Whether it is met is the owner's judgement: a prototype the owner finds less convenient than
hoped, or a path to real use the owner finds too hard, means it is unmet.

## A presentation is made in a GUI, with no language to learn and no build step `##no-language-no-build`

The author builds a presentation by direct manipulation in an interactive editor. No programming
or markup language and no build procedure is involved. It is met when an author goes from an empty
file to a presentation shown in presenter mode without writing code or markup and without running
a build command.

## Any area of a slide can be divided freely and conveniently, each part holding one piece of content `##free-slide-division`

The author places content by dividing the slide into areas, and each area holds one piece of
content. It is met when the layouts the author needs are reached by dividing and resizing areas in
the editor, faster than with a general-purpose slide editor and without placing coordinates by
hand.

## The editor stays minimal: a small set of content kinds, each with only the options that make sense for it `##minimal-content-set`

The editor offers a short list of content kinds: wrapped text, bullet list, image, equation, code
snippet, and a few more, such as video. Each kind is edited in the ways that make sense for it,
with the options that belong to it, such as bold and colour for text or the scaling method for an
image. It is met while every feature the editor has serves one of these kinds or the division of a
slide. Collaborative editing is not in scope.

## A presentation is shown from the tool in presenter mode `##presenter-mode`

The main way to give a talk is the tool itself: it displays the presentation full screen, slide by
slide, with no export step. An export, such as images of the slides or a PDF, may come later and
stays secondary. It is met when the author can give a talk from the tool alone.

## A presentation is saved as a text file that a person or an agent can read `##readable-save-file`

A saved presentation is one human-readable text file. It is met when a saved file can be read and
edited by hand or by an agent, and loads back to the same presentation.

## Every editing operation works without a display `##headless-core`

The model of a presentation and every operation on it run with no GUI. It is met when every
operation the GUI offers can be performed and its result checked in a test or a script that opens
no window.

## A running editor, or a headless session, is driven by text commands `##text-command-protocol`

An agent or a script drives the editor through a text protocol of commands: it performs the common
operations and reads back the state. It is met when an LLM session can build and change a
presentation while the GUI shows the result live, and when the same commands drive a headless
session in tests.

## The editor and presenter mode run in a web browser `##runs-in-browser`

The tool also builds to WebAssembly, so a presentation can be edited and shown in a browser with no
install. It is met when the same editing operations and presenter mode work in a browser build as
in the native build. The owner may abandon this goal if it constrains the design more than its use
justifies.
