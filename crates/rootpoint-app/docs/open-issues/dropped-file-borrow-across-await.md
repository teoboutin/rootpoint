---
kind: defect
---
# Reading a dropped file holds the picker inbox's borrow across an await, in the browser build

## Summary

`Web::read_dropped` of `path@rootpoint-app@src/web.rs` takes `inbox.borrow_mut()` before it awaits
`DroppedFileHandle::bytes_async`, so the `RefCell` stays mutably borrowed while the browser reads the
file. A frame that reads the inbox in that time would panic with a `BorrowMutError`, an assumption,
not run. `cargo clippy --target wasm32-unknown-unknown -p rootpoint-app` warns of it, with
`clippy::await_holding_refcell_ref`.

## Details

### What

The loop of `Web::read_dropped` writes
`inbox.borrow_mut().push(match file.bytes_async().await { … })`. Rust evaluates the receiver of
`push`, the `RefMut`, before its argument, so the borrow is taken first and held across the
`.await`. While the future waits, eframe runs frames, and `App::frame` reads the same inbox,
`Rc<RefCell<…>>`, through `Web::take` of `App::serve_pickers` on every frame; a mutable or a
shared borrow there while the `RefMut` is held panics.

Measured: the warning, from clippy 1.99 for the wasm target, at the `inbox.borrow_mut()` of
`Web::read_dropped`. The native build has no such code, the pickers of
`design@rootpoint-app@browser-pack-save` being the browser's alone, and `cargo x gates` reports its
gate `clippy` ok on the same tree, so that gate runs no clippy for the wasm target. Not established: that a frame runs during the await in practice,
which a file large enough to take more than one frame to read would show, by dropping it on the
browser build's window.

The commits that wrote the code: "Build the editor for the browser, load only, with `cargo x web`",
then "Add the Assets tab: the tree of the assets, their uses and the asset commands"; it predates
the work on math, which met it.

### Why it matters

A panic in the browser build stops the page's editor, and its edits in memory are lost unless a
download holds them, against `goal@rootpoint-app@browser-window`.

### What would close it

The bytes are awaited first and the inbox borrowed after, as
`let picked = match file.bytes_async().await { … }; inbox.borrow_mut().push(picked);`; clippy for the
wasm target gives no warning, and the gate runs clippy for that target too, or the entry says why it
does not.
