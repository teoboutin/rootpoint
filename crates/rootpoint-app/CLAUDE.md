# rootpoint-app

## Every action is a command

Every action of the editor runs protocol `Command`s through `App::run`, and the session is changed
nowhere else, per `design@rootpoint-app@actions-are-commands`.

## The slide's texture is rendered again only when something changed

`App::run` marks the texture stale, and a frame renders the slide again only when it is stale or
the view's size in pixels changed. egui sizes its panels again on the second frame, so the view's
size can change once after the first.

## Presenter mode draws the slide as the editor does

Every editor mark is drawn by `App::overlay` over the texture, never into the image `App::draw`
uploads, per `design@rootpoint-app@editor-overlay`; `App::presenter` renders through `App::draw`
and draws nothing over it, and `path@rootpoint-app@tests/presenter.rs` compares its image with the
editor's. `App::frame` asks the window for full screen whenever presenter mode starts, and to
leave it when it ends.

## The slide view's marks and hits come from one layout

`hit::Layout` places the shown slide's nodes and dividers in points through `slide_to_pixels`, so
the overlay, the hit testing and `render_slide` agree. A press is read from
`InputState::pointer.press_origin()`: egui reports a drag only once the pointer has moved.
`consume_key` ignores extra Shift and Alt, so a key tested with fewer modifiers is tested after
the one with more.

## A gesture's undo group closes before another opens

A gesture opens its group through `App::open_group`, which first closes any group still open, per
`design@rootpoint-app@actions-are-commands`. An action that does not join the gesture, as undo, an
open, a save or presenting does, closes any group still open before it acts.

## The tests find widgets by the places the last frame recorded

`App::layout`, `App::slide_rows` and `App::text_field_rect` return where the last frame put the
slide, the list's rows and the text field, so a test of `App::frame` places its synthetic
pointer on them. A selection changes the side panel, which egui sizes again on the next frame, so
a test lets two frames pass before it reads a place.

## The build is tied to its checkout

`path@rootpoint-app@src/lib.rs` and `path@rootpoint-app@src/main.rs` read `ROOTPOINT_CHECKOUT`. So
does every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
