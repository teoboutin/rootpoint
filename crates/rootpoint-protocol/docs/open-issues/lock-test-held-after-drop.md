---
kind: observation
---
# A lock test once found its workspace still locked after the lock was dropped

## Summary

The test `a_second_process_is_refused_naming_the_holder` failed once during `cargo x gates`: after
the test dropped its `WorkspaceLock`, the second `open ws` of `rootpoint-headless` was refused as
if the lock were still held. Eight runs of the test's file alone passed. It is not established
whether it reproduces, nor why it happened.

## Details

### What

The test, in `path@rootpoint-protocol@tests/workspace.rs`, takes `WorkspaceLock::take` on a
workspace in the test process, checks that `rootpoint-headless` run by `headless` of
`path@rootpoint-protocol@tests/common/mod.rs` replies `err ws is open in another process, pid
<the test process's pid>`, drops the lock, and expects a second `open ws` to reply `ok`. In the one
failing run, the second reply was:

```text
err ws is open in another process, pid 2039498
```

The pid is the one the lock file holds, which `WorkspaceLock::take` wrote in the test process: the
test's first assertion, which passed, finds `std::process::id()` in the first refusal, and no
process that is refused writes the file, so the second refusal names the test process too. The
failing run was the gate `test` of `cargo x gates`, which runs every test binary of the workspace,
on the branch that added the span commands of rich text; that branch changed no file of the
workspace lock or of this test. Eight runs of
`cargo test -q -p rootpoint-protocol --test workspace` right after it each passed 5 tests, and the
next run of `cargo x gates` passed.

Assumption, not checked: the five tests of the file run in parallel threads of one process, and
`a_crash_leaves_no_stale_lock` and `headless` spawn processes. A process spawned by another thread
while the lock is held receives a copy of the lock file's descriptor, which is closed only at its
`exec`, since Rust opens files with close-on-exec. `File::try_lock` is `flock` on Linux, which is
held while any descriptor of the open file stays open, so a spawn caught between its fork and its
`exec` at the moment of the drop would keep the lock held for that interval.

### Why it matters

A gate that fails at random makes a branch rerun `cargo x gates` before it merges, and teaches a
reader to rerun a failure instead of reading it. If the assumed mechanism holds, it also touches
`design@rootpoint-protocol@workspace-lock`, whose argument is that the kernel releases the lock
when its holder ends: a host that spawns a process while it holds the lock would let the child hold
it until the child's `exec`.

### What would close it

Run `cargo test -p rootpoint-protocol --test workspace` in a loop, 500 times, under the load of
`cargo x gates`' other tests: if it fails, promote this entry to a `defect` with the count, and test
the mechanism by running the test with `--test-threads=1`, which spawns no process during its
drop. If 500 runs pass, the entry is closed as not reproduced, naming the count.
