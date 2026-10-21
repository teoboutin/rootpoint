---
kind: deferred
---
# The gates compare the branch against the local main, not origin/main

## Summary

`cargo x gates` builds its commit range from the local branch `main`. The repository now has a
remote, `origin`, but it holds no branch yet, so `origin/main` does not resolve and cannot be the
base.

## Details

### What

In `path@xtask@src/main.rs`, the `Gates` arm calls `rust_project(checker, "main")`. The `commits`
gate then judges `main..HEAD`, and the `rebased` gate, behind `--require-rebased`, checks that HEAD
contains `main`. `git ls-remote origin` printed nothing on 2026-10-21, so `origin/main` names
nothing; with it as the base, the `commits` gate fails to resolve its range.

### Why it matters

Once work is pushed and merged through the remote, a local `main` that lags `origin/main` makes the
`commits` gate judge commits already merged, and the `rebased` gate pass a branch that is behind
the remote. That strains `goal@xtask@one-command-runs-every-gate`, whose one command is to say
whether a branch may merge.

### Trigger

The first push of `main` to `origin`. The session that pushes it switches the base in the same
change: `rust_project(checker, "origin/main")`, then `cargo x gates` passes against it.
