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
and draws nothing over it but, while the control socket holds control, a frame in a corner with
its label and the button that takes control back, and `path@rootpoint-app@tests/presenter.rs` compares its image with the
editor's. `App::frame` asks the window for full screen whenever presenter mode starts, and to
leave it when it ends; in the browser build it asks the page, per
`design@rootpoint-app@page-full-screen`.

## The slide view's marks and hits come from one layout

`hit::Layout` places the shown slide's nodes and dividers in points through `slide_to_pixels`, so
the overlay, the hit testing and `render_slide` agree. A press is read from
`InputState::pointer.press_origin()`: egui reports a drag only once the pointer has moved.
`consume_key` ignores extra Shift and Alt, so a key tested with fewer modifiers is tested after
the one with more.

## A gesture's undo group closes before another opens

A gesture opens its group through `App::open_group`, which first ends the window's gestures and
closes their group, per `design@rootpoint-app@gesture-groups`. An action that does not join the
gesture, as undo, an open, a save or presenting does, ends them too before it acts. A group the
control socket opened is closed only by the socket or by the take-over, `App::take_control`, per
`design@rootpoint-app@take-over-at-once`.

## The tests find widgets by the places the last frame recorded

`App::layout`, `App::slide_rows`, `App::outline_rows` and `App::button_rect` return where the last
frame put the slide, the list's rows, the outline's rows and the side panel's buttons and tabs, so a
test of `App::frame` places its synthetic pointer on them; `rootpoint_render::area_text` gives where a text's characters are, in slide units, which
`App::layout` carries to the view.

## The browser build differs only where the page does

The code built for the wasm target alone is under `cfg(target_arch = "wasm32")`: the entry point in
`path@rootpoint-app@src/main.rs`, the pickers and full screen in `path@rootpoint-app@src/web.rs`
with their calls in `App::frame`, and the actions of `path@rootpoint-app@src/actions.rs` that
differ there, open, save and adding an image; the Save buttons of `path@rootpoint-app@src/panels.rs`
are under `cfg(not(target_arch = "wasm32"))`; the control socket is under `cfg(unix)`, which the
wasm target is not, per `design@rootpoint-app@control-socket`. What a picker gives goes through an
action of `actions.rs`, `App::open_directory` or `App::add_picked_image`, built on every target,
so the native tests check it. `path@rootpoint-app@tests/actions.rs` and
`path@rootpoint-app@tests/web.rs` also run in a headless browser, the gate `wasm-test` of
`path@xtask@src/main.rs`, per `design@xtask@browser-tests-gated`; a test added to either must
build and pass on the wasm target, with no file system and no thread.

## The build is tied to its checkout

`path@rootpoint-app@src/lib.rs` and `path@rootpoint-app@src/main.rs` read `ROOTPOINT_CHECKOUT`. So
does every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
