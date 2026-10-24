# Slice: the window as the host of a workspace

The third slice of `milestone@plans@asset-storage`. It lands `thread@asset-storage@workspace-lock`,
`thread@asset-storage@socket-place`, the `.rootpoint/` of `thread@asset-storage@workspace-layout`,
the native half of `thread@asset-storage@unsaved-close-warning`, and the state label of
`criterion@asset-storage@c-visible-mode`.

## Builds

- `planned@rootpoint-app@src/workspace.rs`: `WorkspaceLock`, which creates `.rootpoint/` with its
  `.gitignore` holding `*`, takes `File::try_lock` on `.rootpoint/lock`, writes its pid there, and
  holds the file until dropped; the runtime directory and the socket's name; the pointer file
  `.rootpoint/socket`.
- `open <dir>` in the window and in `rootpoint-headless` takes the lock before the session runs the
  command, and refuses the open, naming the pid, when another process holds it. The lock is the
  host's: the session stays free of it, #c-no-host-in-core.
- `Listener::bind` of `path@rootpoint-app@src/control.rs` runs only under the lock, and removes any
  file at its path before the bind. The window binds at start: `<name>-<hash>.sock` in a workspace,
  `memory-<pid>.sock` in memory; on `unpack`, the socket file is renamed, AC5. `--control` leaves
  `path@rootpoint-app@src/main.rs`; the command line becomes `rootpoint [<dir> | <file>.rpk]`.
- `rootpoint-ctl`, `path@rootpoint-protocol@src/bin/rootpoint-ctl.rs`: a first argument that is a
  directory reads its `.rootpoint/socket`; `list` prints each socket of the runtime directory that
  accepts a connection, with its name.
- The state label, in the window's top bar, and the names of the writing actions, by
  #c-visible-mode's form; the question on close in memory, by `ViewportCommand::CancelClose`.

## Claims

- **The bind race is gone.** Carried from the issue this milestone closed: two threads behind a
  barrier each open the same workspace and bind, 200 rounds, on a path that holds a stale socket;
  before the lock, both binds succeeded in 4 of 200 rounds on the owner's Intel Core Ultra 7 165H.
  With the lock, no round has two binds. Shown to fail by removing the lock.
- **A second process is refused**: a test opens a workspace, spawns `rootpoint-headless` on it and
  reads its refusal, naming the pid.
- **A crash leaves no stale lock**: a test kills a child that holds the lock, and opens.
- **A renamed socket answers**, AC5, `acceptance@asset-storage@ac-renamed-socket`.
- **`rootpoint-ctl <dir>` reaches the window**, and `rootpoint-ctl list` lists it, in
  `path@rootpoint-protocol@tests/ctl.rs`.
- **The state label reads each of the three states**, a test of `App::frame`, #c-visible-mode.
- **A close in memory with an edit asks, and a close with none or in a workspace does not**, a test
  of `App::frame`.

## Audit subjects

- `path@rootpoint-app@src/control.rs`, `path@rootpoint-app@src/main.rs`,
  `path@rootpoint-app@tests/control.rs`, `path@rootpoint-app@tests/listener_drop.rs`,
  `path@rootpoint-app@tests/presenter.rs`, `path@rootpoint-protocol@src/bin/rootpoint-ctl.rs`,
  `path@rootpoint-protocol@tests/ctl.rs`.
- `design@rootpoint-app@control-socket`, `design@rootpoint-app@only-a-stale-socket-is-replaced`,
  `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch`, `design@rootpoint-app@one-controller-at-a-time`.
- `issue@rootpoint-app@socket-waits-for-a-frame`, which cites the socket's head.
- macOS's `sun_path` and `$TMPDIR`, the milestone's "Readings", before the fallback is written.
- The rejected entries of the socket's authority, which keep their state: this slice changes where
  the socket is, not who holds the authority.

## Fails alone on

- The lock's refusal of a second process, and the bind race test.

## Premises that expire

- `flock` holds on the owner's local disks; T1 watches a workspace shared by a sync tool.

## Decided design

The milestone's sections on `thread@asset-storage@workspace-layout`,
`thread@asset-storage@workspace-lock`, `thread@asset-storage@socket-place` and
`thread@asset-storage@unsaved-close-warning` hold the shape.

## Acceptance criteria

This slice judges `acceptance@asset-storage@ac-renamed-socket`.

## Harvest

- A head of the app for the workspace lock; rewrite `design@rootpoint-app@control-socket` and
  `design@rootpoint-app@only-a-stale-socket-is-replaced` to the runtime directory and the lock;
  rewrite `design@rootpoint-protocol@ctl-sends-a-line-or-a-batch` for a workspace and `list`; a
  `rejected` entry for the socket in `.rootpoint/`, #arg-7, and one for the abstract namespace and
  TCP, #arg-19, by the tests of `skill@knowledge-architect-decision-recording`.
- Rewrite `design@rootpoint-model@image-storage` for the workspace's layout.
- Write T1, guarding the lock's head, as
  `skill@knowledge-architect-decision-recording@ruled-tripwires` says.
- Update `path@rootpoint-app@CLAUDE.md`, `issue@rootpoint-app@socket-waits-for-a-frame`, the
  comments of `path@rootpoint-app@src/control.rs` and the tests the milestone's table lists, and
  `path@rootpoint-app@README.md` and `path@rootpoint-protocol@README.md` for the command lines.
