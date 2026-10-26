# rootpoint

@.claude/knowledge-architect/PRIMER.md

## Merge gates `##merge-gates`

`cargo x gates` runs every check a branch owes before it merges, and its exit code is the verdict.
`cargo klarch` runs the document checker at the pinned version.

## Skill prefix `##skill-prefix`

The project's own skills, agents and workflows are named with the prefix `rootpoint-`.

## The project's homes `##project-homes`

| the statement is about | home | it leaves when |
| --- | --- | --- |
| a survey of prior art or of outside tools, run as grounding for a design discussion | one file per survey in `path@rootpoint@docs/grounding/`, linked from its `README.md` | never: the owner keeps them as material for later design |

## Compatibility of a change `##change-compatibility`

A breaking change of the save format raises `FORMAT_VERSION`, adds the step that migrates the
version before, and freezes the files of the new version as `path@rootpoint-model@tests/formats.rs`
says, per `design@rootpoint@saved-presentations-stay-loadable`. Until the owner rules otherwise, a
change of the protocol's grammar or of a crate's public API keeps no old form, per
`design@rootpoint@grammar-and-api-stay-free`.

## Git workflow `##git-workflow`

- Run the gates before merge `cargo x gates`

Preserve linear history:
- fetch origin before merge, update local `main` if needed.
- check that the branch can be fast-forwarded on `origin/main`
- rebase if that is not the case, then rerun the gates.
- merge to `main` locally, using `--fast-forward`
- then push to origin


