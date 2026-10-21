# rootpoint-app

## Every action is a command

Every action of the editor goes through `App::run`, which runs a protocol `Command` on the
session, so a script of the same commands gives the same document and view. The session is
changed nowhere else.

## The slide's texture is rendered again only when something changed

`App::run` marks the texture stale, and a frame renders the slide again only when it is stale or
the view's size in pixels changed. egui sizes its panels again on the second frame, so the view's
size can change once after the first.

## The build is tied to its checkout

`path@rootpoint-app@src/lib.rs` and `path@rootpoint-app@src/main.rs` read `ROOTPOINT_CHECKOUT`. So
does every binary target and build script added to this crate, as `path@xtask@CLAUDE.md` says.
