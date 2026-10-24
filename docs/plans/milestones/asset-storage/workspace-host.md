# Slice: the hosts of a workspace

The third slice of `milestone@plans@asset-storage`. It lands `thread@asset-storage@workspace-lock`,
`thread@asset-storage@socket-place`, the `.rootpoint/` of `thread@asset-storage@workspace-layout`,
the native half of `thread@asset-storage@unsaved-close-warning`, and the state label of
`criterion@asset-storage@c-visible-mode`.

## Builds

- `planned@rootpoint-protocol@src/workspace.rs`, native only, `cfg(not(target_arch = "wasm32"))`:
  `WorkspaceLock`, which creates `.rootpoint/` with its `.gitignore` holding `*`, takes
  `File::try_lock` on `.rootpoint/lock`, writes its pid there, and holds the file until dropped;
  `runtime_dir` and `socket_path`, by the milestone's "Decided design" of #socket-place; and the
  pointer file `.rootpoint/socket`. It is the third part of the protocol crate that touches the file
  system, beside the headless binary and `path@rootpoint-protocol@src/control.rs`.
  - `socket_name(root: Option<&str>) -> String`, the file name alone, so a test places it in a
    runtime directory of its own and gives that directory to a child by `XDG_RUNTIME_DIR`.
  - The workspace's name is truncated to 40 bytes, at a character's boundary, and further where the
    whole path would pass 103 bytes: `sun_path` holds 108 bytes on Linux and 104 on macOS, read at
    the audit in xnu's `bsd/sys/un.h`, `char sun_path[104]`, less the closing NUL. Where no byte of
    the name fits, the name is the hash alone.
  - `/tmp/rootpoint-<uid>/` takes the user id from `libc::getuid`, `libc` 0.2 a dependency of the
    protocol crate on Unix only, already in `Cargo.lock`.
  - The lock's refusal reads `err <dir> is open in another process, pid <pid>`.
- `workspace::run(session, command, files, lock: &mut Option<WorkspaceLock>) -> Reply`, the one
  function both hosts run every command through: it takes the lock of `<dir>` before `open <dir>`,
  where `<dir>` exists, and before `unpack <dir>` in memory; it keeps the lock it holds when `<dir>`
  is the workspace it holds already, since a second lock of one file in one process is refused like
  another process's; and after an `ok` that changed `Session::root` it drops the old lock.
  `unpack` counts a directory holding only `.rootpoint/` as empty, since the lock is taken before.
- One function of each host that runs a command, by "Decided design" of #workspace-lock: the
  headless binary's loop, and `App::run` with the control socket's dispatch of
  `path@rootpoint-app@src/control.rs`. Before `open <dir>` or `unpack <dir>`, it takes the lock of
  `<dir>`, or replies `err` naming the pid; after an `ok` that changed `Session::store`, it drops the
  old lock and, in the window, moves the socket by the milestone's table "The store's change to the
  socket's name".
- `Listener::bind` of `path@rootpoint-app@src/control.rs` binds at `socket_path`, removing a socket
  file at its path and refusing any other file; `Drop for Listener` removes the file at the path it
  last renamed to. The window binds at start; `--control` leaves `path@rootpoint-app@src/main.rs`,
  whose command line becomes `rootpoint [<dir> | <file>.rpk]`.
  - `Listener::rename_to(path)` moves the socket's file, by the rules of a bind at the new path.
  - `App::native(files)` makes the window that holds workspaces: its `App::run` takes the locks and
    moves the socket. `App::new` takes no lock and binds nothing, so a test over files held in memory
    writes nothing on disk.
  - A workspace or a pack the command line names and that cannot be opened leaves the window in
    memory, the status line saying why, as a file that did not load does.
- `rootpoint-ctl`, `path@rootpoint-protocol@src/bin/rootpoint-ctl.rs`: a first argument that is a
  directory reads its `.rootpoint/socket`; `list` prints each socket of the runtime directory that
  accepts a connection, with its name.
- `App::state_label` and its text at the right end of the toolbar, by the milestone's table "The
  store's state to the state label" (D5); the names of the writing actions, by #c-visible-mode; the
  question on close in memory while `Session::unsaved` is not 0, by `ViewportCommand::CancelClose`:
  a window "Close without saving?" naming the changes, with "Close" and "Keep open". A count of 1
  reads "1 change".
  A workspace never asks, as the owner accepted it; until #autosave-and-watch, its state label
  says how many changes Ctrl+S has not saved.

## Claims

- **The bind race is gone.** Carried from the closed `two-windows-bind-one-path`, whose text, with
  its test's code, is read as the milestone's "Names" says: two threads behind a barrier each call
  `Listener::bind` on a path that holds a stale socket, left by a `UnixListener` dropped before; a
  round counts when both binds return `Ok`. Measured by the review of the slice that built the
  socket, with that test, not committed: both binds succeeded in 4 of 200 rounds on the owner's
  Intel Core Ultra 7 165H. Now two threads behind a barrier each open the same workspace through
  the host's function and bind, 200 rounds: no round has two binds. Shown to fail by skipping the
  lock. Whether two windows start on one workspace at one moment outside a test is not
  established; the race mattered because the replaced window showed no error while every command
  went to the other, against `goal@rootpoint-app@live-agent-edits`.
- **A second process is refused**: a test opens a workspace, spawns `rootpoint-headless` on it and
  reads its refusal, naming the pid.
- **A crash leaves no stale lock**: a test kills a child that holds the lock, and opens.
- **A renamed socket answers**, AC5, `acceptance@asset-storage@ac-renamed-socket`; and the listener
  dropped after the rename removes the renamed file.
- **`rootpoint-ctl <dir>` reaches a listener, and `rootpoint-ctl list` lists it**, in
  `path@rootpoint-protocol@tests/ctl.rs`, on a listener of that suite's own bound at
  `socket_name` in a runtime directory of the test's, given to the child, with a pointer file, as
  the suite runs with no window.
- **The state label reads each state of the table**, a test of `App::frame` through
  `App::state_label`, #c-visible-mode.
- **A close in memory with an edit asks, and a close with none does not**, a test of `App::frame`.

## Audit subjects

- `path@rootpoint-app@src/control.rs`, `path@rootpoint-app@src/main.rs`,
  `path@rootpoint-app@src/lib.rs`, `path@rootpoint-app@tests/control.rs`,
  `path@rootpoint-app@tests/listener_drop.rs`, `path@rootpoint-app@tests/presenter.rs`,
  `path@rootpoint-protocol@src/bin/rootpoint-ctl.rs`, `path@rootpoint-protocol@src/bin/rootpoint-headless.rs`,
  `path@rootpoint-protocol@tests/ctl.rs`, `path@rootpoint-protocol@CLAUDE.md`.
- `design@rootpoint-app@control-socket`, `design@rootpoint-app@only-a-stale-socket-is-replaced`,
  `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`, `design@rootpoint-app@one-controller-at-a-time`,
  `design@rootpoint-app@no-discard-confirmation` with D3, `design@rootpoint@gui-only-in-app`.
- `issue@rootpoint-app@socket-waits-for-a-frame`, which cites the socket's head.
- macOS's `sun_path` and `$TMPDIR`, the milestone's "Readings", before the fallback is written.
- The rejected entries of the socket's authority, which keep their state: this slice changes where
  the socket is, not who holds the authority.

## Fails alone on

- The lock's refusal of a second process, and the bind race test.

## Premises that expire

- `flock` holds on the owner's local disks; T1 watches a workspace shared by a sync tool.
- A workspace is saved only by Ctrl+S until #autosave-and-watch, which ends it; the state label's
  two states "before #autosave-and-watch" leave with it, and that slice's claims check their
  removal.

## Decided design

The milestone's sections on `thread@asset-storage@workspace-layout`,
`thread@asset-storage@workspace-lock`, `thread@asset-storage@socket-place` and
`thread@asset-storage@unsaved-close-warning`, and its tables of the state label and of the socket's
name, hold the shape.

## Acceptance criteria

This slice judges `acceptance@asset-storage@ac-renamed-socket`.

## Harvest

- A head of the protocol for the workspace lock; rewrite `design@rootpoint-app@control-socket` and
  `design@rootpoint-app@only-a-stale-socket-is-replaced` to the runtime directory and the lock,
  its last sentence, which cites this spec, included; rewrite
  `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch` for a workspace and `list`; a `rejected`
  entry for the socket in `.rootpoint/`, #arg-7, and one for the abstract namespace and TCP,
  #arg-19, by the tests of `skill@knowledge-architect-decision-recording`; and one for the lock in
  the app, if it meets one.
- Rewrite `design@rootpoint-app@no-discard-confirmation` as D3 rules, with the warning on close of
  `thread@asset-storage@unsaved-close-warning`, and update `register@rootpoint-app@rejected` line 93
  if the rewrite changes what it cites.
- Rewrite `design@rootpoint-model@image-storage` for the workspace's layout, and the lines of
  `register@rootpoint-app@design` and `register@rootpoint-protocol@design` that cite it.
- A head of the app for the state label and for the writing actions named by where they write,
  `criterion@asset-storage@c-visible-mode`, the owner's requirement, built natively here and in the
  browser at #browser-packs, by the tests of `primer@design-heads`.
- Write T1, guarding the lock's head, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- Update `path@rootpoint-app@CLAUDE.md`, `path@rootpoint-protocol@CLAUDE.md` (its list of the parts
  that touch the file system), `issue@rootpoint-app@socket-waits-for-a-frame`,
  `rejected@rootpoint-app@socket-holds-control-at-start` (its `--control`), the comments of
  `path@rootpoint-app@src/control.rs` and the tests the milestone's table lists, and
  `path@rootpoint-app@README.md` and `path@rootpoint-protocol@README.md` for the command lines.
