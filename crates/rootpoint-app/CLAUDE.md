# rootpoint-app

## Every action is a command

Every action of the editor goes through `App::run`, which runs a protocol `Command` on the
session, so a script of the same commands gives the same document and view. The session is
changed nowhere else.

## The slide's texture is rendered again only when something changed

`App::run` marks the texture stale, and a frame renders the slide again only when it is stale or
the view's size in pixels changed. egui sizes its panels again on the second frame, so the view's
size can change once after the first.

## The slide view's marks and hits come from one layout

`hit::Layout` places the shown slide's nodes and dividers in points through `slide_to_pixels`, so
the overlay, the hit testing and `render_slide` agree. A press is read from
`InputState::pointer.press_origin()`: egui reports a drag only once the pointer has moved.
`consume_key` ignores extra Shift and Alt, so a key tested with fewer modifiers is tested after
the one with more.

## A gesture's undo group closes before another opens

A divider drag, a grip drag and a focus of the text field each open a group with `begin`, through
`App::open_group`, which first closes any group still open; a release, the loss of the field's
focus and the loss of the window's focus close it. Undo, redo, open, save and the sample close the
text field's group first.

## The tests find widgets by the places the last frame recorded

`App::layout`, `App::slide_rows` and `App::text_field_rect` return where the last frame put the
slide, the list's rows and the text field, so a test of `App::frame` places its synthetic
pointer on them. A selection changes the side panel, which egui sizes again on the next frame, so
a test lets two frames pass before it reads a place.

## The build is tied to its checkout

`path@rootpoint-app@src/lib.rs` and `path@rootpoint-app@src/main.rs` read `ROOTPOINT_CHECKOUT`. So
does every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
