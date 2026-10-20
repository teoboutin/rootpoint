# Goals of xtask

What xtask is for, and what would show it achieved: one level-two heading per goal. A goal is met
or unmet. It stays here while it is met, and it leaves only on the owner's word. How xtask is
built is in `register@xtask@design`.

## Every check the project owes before a merge runs from one command `##one-command-runs-every-gate`

One command runs every check a branch must pass before it merges: formatting, the document check,
the commit messages, the linter where the project has one, and the tests. It runs them all when one
fails, and gives one verdict as its exit code. It is met while a session needs no other command to
know whether a branch may merge, and, where the project has continuous integration, it runs the
same command.

## A task performed repeatedly in the project is a command of the tool `##repeated-tasks-are-automated`

A task that sessions perform in this repository more than once, by hand and in the same steps,
becomes a command of the tool, so it runs the same way each time. It is met while no such task is
left to be done by hand.
