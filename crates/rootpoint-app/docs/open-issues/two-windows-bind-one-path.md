---
kind: defect
---
# Two windows started at once on a stale socket path can both bind, and one listens at no path

## Summary

`Listener::bind` checks the path, removes a stale socket and binds, in three steps with no lock. Two
windows that start at the same moment on the same path, a stale socket there, can both pass the
check; the second removes the first's fresh socket and binds its own, and the first then listens
on a socket no path reaches, while its author believes it is driven.

## Details

### What

In `path@rootpoint-app@src/control.rs`, `Listener::bind` reads the path's metadata, connects to it
to tell another window's live socket from a stale one, removes a stale one, then calls
`UnixListener::bind`. Between the removal and the bind, another `Listener::bind` on the same path
can run the same steps. Measured by the review of the slice that built the socket, with a test not
committed:

```rust
// Two threads behind a barrier each call Listener::bind on a path that holds a stale socket,
// left by a UnixListener dropped before; the round counts when both binds return Ok.
let path = socket("race");
for _ in 0..200 {
    drop(std::os::unix::net::UnixListener::bind(&path).unwrap());
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let both = (0..2).map(|_| {
        let (path, barrier) = (path.clone(), barrier.clone());
        std::thread::spawn(move || { barrier.wait(); Listener::bind(&path, || {}) })
    }).map(|t| t.join().unwrap()).collect::<Vec<_>>();
    // count the rounds where both are Ok, then drop both and remove the path
}
```

Both binds succeeded in 4 of 200 rounds. Ruled out since: the window whose socket was replaced no
longer deletes the other's socket file when it closes, since `Listener`'s `Drop` removes the file
only while its inode is the one bound; `a_listener_removes_no_other_windows_socket` in
`path@rootpoint-app@tests/control.rs` checks it. Not established: whether two windows started on
one path at one moment happen outside a test.

### Why it matters

The window whose socket was replaced shows no error and no banner, and every command sent to the
path goes to the other window, so an author watching the first sees none of the agent's edits,
against `goal@rootpoint-app@live-agent-edits`.

### What would close it

`Listener::bind` holds an exclusive lock, such as `flock` on a lock file beside the socket's path,
from its check to its bind, so a second window waits and then finds the first's live socket and
refuses. The test above, with the lock, gives no round where both binds succeed.
