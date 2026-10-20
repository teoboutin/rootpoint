---
name: knowledge-architect-setup
description: MUST use when a project adopts the knowledge-architect workflow, after the first install has run, and when a project moves its pin of the checker to another version. Covers how the checker is pinned and run, the declared command, the manifest, its Components and the registers it declares, the documents and register homes each Component carries, the primer's import line, the project's rows of the knowledge table, the project's skill prefix, the gates convention, what happens to the documentation the project already has, and, in a Rust project, the maintenance crate, its aliases, its gates and its continuous integration.
---

# Setting up

Scope: making a project conformant to the workflow, from a manifest the first install accepted to
a tree over which `cargo klarch check` passes, with the owner's goals stated. And moving the pin of
the checker to another version.

Not covered here: **stating the goals**, `skill@knowledge-architect-goal-setting`; **moving the
existing documentation into the new homes**, which is planned work of its own
(`skill@knowledge-architect-setup@existing-documentation`); **writing the project's own skills**,
`skill@knowledge-architect-agent-configuration`.

**Reaching this skill.** The smallest manifest the install accepts is the file
`knowledge-architect.toml` at the project's root, where `<version>` is the version of the checker
the project runs, as `skill@knowledge-architect-setup@pin-the-checker` says:

```toml
[project]
name = "<project name>"
checker-version = "<version>"
components = []

[walk]
skip-dirs = []
skip-files = []
exclude = []
```

After the first install, `cargo klarch check` lists what the project still owes, each with its
repair. That list is this skill's work list. A run reports the first phase that finds anything and
judges no later phase, so the check runs again after each repair until it passes.

The checker needs `git` 2.36 or newer, and a project inside a git repository. Building it needs a
Rust toolchain, whatever the project's own language.

**Setting up is done with the owner.** Every step that writes a durable statement on the owner's
behalf is shown to the owner first: the Components, the goals, the place of each existing document
that no row of the knowledge table settles,
and each choice below that changes the project's build or a file it already has. A run without
the owner, such as a trial, makes those choices itself, except the goals, which only the owner
states: such a run writes no goal, and leaves each goals home with no entry, awaiting the owner's.

## Pin the checker, and decide how it runs `##pin-the-checker`

A project runs the checker at the version it chose, and moves to another version by an explicit
edit. The installed skills move with it, since one version of the checker ships one version of
them.

- **A Rust project** runs the checker from its maintenance crate, which depends on
  `knowledge-architect = "=<version>"` and serves the checker's commands through the cargo alias
  `cargo klarch`, as `skill@knowledge-architect-setup@rust-project` below shows. The `=` pins exactly; without it,
  `"<version>"` accepts every later version below the next breaking one. `Cargo.lock` records the
  exact version, and `--locked` turns any change to it into a failure. A dependency alone builds
  no executable for the project: `cargo run -p` runs only the project's own packages, which is why
  the crate serves the checker's commands itself. A Rust project that is one package with no
  workspace gains one, through a `[workspace]` table in its `Cargo.toml` whose `members` names the
  maintenance crate: this is the shape to recommend, since it keeps the gates of "In a Rust
  project". The alternative is a local install, as the next point says, which leaves the project
  with no maintenance crate and no `cargo x gates`, so the project builds its own gates command,
  per `skill@knowledge-architect-setup@setup-gates`. The first changes the project's build, so the owner rules between the two.
- **Any other project**, and a Rust project that runs a local install, installs the binary into a
  directory of its own, ignored by git:
  `cargo install --locked --root <dir> knowledge-architect --version =<version>`. The binary is
  `<dir>/bin/klarch`. A plain `cargo install` is machine-wide, and two projects on one machine
  would then share one version.
- **A project with an extension** runs its own binary, which registers the extension, through a
  cargo alias. It does not install that binary under the plain name `klarch`: with two binaries of
  that name on the path, the first one found runs.

**The manifest declares the pin**, in `[project] checker-version = "<version>"`, the version of the
checker the project runs, exactly. Every command refuses to run when the binary is another version,
names the two versions and says which side is older, so an install or a build left behind is
caught at its first run. A value that is not a version serves a project's own tests: `"fixture"`
in a mock project that lies inside the directory of a library the binary links, such as the
project's extension crate. It is accepted only where the binary's build confirms it; anywhere else
it is refused. **A test that copies a mock project out
of its library's directory**, for instance into a temporary directory where it runs `git init`,
writes the version of the checker it links into the copy's manifest: the library's constant
`CHECKER_VERSION`. It never spells the version in the test, so the pin stays written in one place.
In an extension's crate, `CARGO_PKG_VERSION` is the extension's version, not the checker's.

## Declare the command `##declare-the-command`

The command the project runs is declared in the manifest, and the checker prints it in its
messages, in the header of every generated listing and in every installed file:

```toml
[project]
command = "<the command, for instance cargo klarch or tools/bin/klarch>"
```

When the key is absent, the command is `klarch`. After declaring it, run the install again, so the
installed files carry it.

## The Components and the locations `##components-and-locations`

A **Component** is a directory the manifest names under `[project] components`, by its path; its
name is the basename of that path, and the project's root is a Component named by
`[project] name`. Propose the Components to the owner: one per unit that has its own goals and its
own design, such as a published package, a service, or a maintenance tool. A directory named after
its role, not after its package, avoids a Component whose name equals the project's.

A **location** is a directory that carries only the registers it declares, for content that belongs
to no Component, such as the agent configuration:

```toml
[locations.agent-config]
path = ".claude"
registers = ["issue"]
```

**A kind of record the built-in registers do not hold is a register the project declares**, under
`[registers.<name>]`, its name the kind its references spell. `scope = "component"` makes every
Component carry it, and `scope = "opt-in"` only the Components its `components = [...]` names and
the locations that name it. Its home sits under its carrier, under `docs/` for a Component, named
by `dir`, which defaults to the register's name; an `opt-in` register declared with `dir = "."`
has its home at the location's own directory, and that location carries it alone. How each shape
keeps its entries, and how they are cited, is `primer@checker-model`. Pick the shape from how the
project's existing records are cut, and declare it with its own keys:

- **`shape = "heading"`**, one record per slugged heading, with `level`, the heading level:

  ```toml
  [registers.note]
  scope = "opt-in"
  shape = "heading"
  level = 3
  ```

- **`shape = "file"`**, one record per file, with `sections`, the level-two headings every record
  carries, and `metadata.<key>.values`, its frontmatter keys and their values:

  ```toml
  [registers.card]
  scope = "opt-in"
  shape = "file"
  sections = ["Summary", "Details"]
  ```

- **`shape = "directory"`**, one directory per subject, with `carries`, the one `opt-in` heading or
  file register each subject carries:

  ```toml
  [registers.subject]
  scope = "opt-in"
  shape = "directory"
  carries = ["note"]
  components = ["<component>"]   # the subjects sit in <component>/docs/subject/
  ```

  A location carrying it with `dir = "."` puts the subjects directly under the location's path
  instead, and the subject as a whole is then cited under the location's name.

What the walk must not read is declared under `[walk]`, each row with its reason in a comment
beside it: `skip-dirs` and `skip-files` for paths, `exclude` for another project kept inside this
one. A row names a path git tracks, spelled as git lists it: a path the ignore rules cover is
already outside the walk, and a row naming it is a finding. `[agents] harness` is absent for the
default, the `claude` harness.

## The documents each Component carries `##component-documents`

Every Component carries the same documents. `cargo klarch check` names each one missing:

| document | holds |
| --- | --- |
| `README.md` | how a user uses the Component |
| `CLAUDE.md` | the contracts and traps a developer needs, true of the code as it stands |
| `register@*@goal`, one file, or a directory with a `README.md` | what the Component is for |
| `register@*@design`, one file, or a directory with a `README.md` | how it is built and why; it may hold no entry yet |
| `register@*@rejected`, one file, or a directory with a `README.md` | the designs not kept, and why, each cited by a design head; it may hold no entry yet |
| `register@*@tripwire`, one file, or a directory with a `README.md` | evidence that would flip a decision; it may hold no entry yet |
| `register@*@issue` | one file per outstanding item, a hand-written `README.md` and an `index.md` that `cargo klarch index` generates |

**The root Component also carries the plans directory**, docs/plans/, whose path the checker fixes:

| document | holds |
| --- | --- |
| `path@plans@README.md` | what the plans directory holds |
| `register@plans@spec`, with a `README.md` and an `index.md` | one file per spec; none until work is planned |
| `register@plans@milestone`, with a `README.md` and an `index.md` | one directory per milestone; none until work is planned |

`cargo klarch index` writes each `index.md`. A roadmap, docs/roadmap.md at the root, is optional:
it is written when the owner wants known work ordered, under `skill@knowledge-architect-planning`.

Each document opens with a short introduction saying what it holds and what it does not. An empty
register home says it holds no entry yet. **Every Component states at least one goal**: run
`skill@knowledge-architect-goal-setting` with the owner for each one.

## The root CLAUDE.md `##root-claude-md`

The project's root `CLAUDE.md` holds, besides what the project already keeps there:

- **the primer's import line**, alone on its own line of prose, outside any code block:
  `@.claude/knowledge-architect/PRIMER.md`. The install never writes it, because the root
  `CLAUDE.md` belongs to the project; the check reports it missing until it is there;
- **the project's rows of the knowledge table**, under a heading of their own: each kind of
  statement the project keeps somewhere the primer's table does not name, such as its changelog, a
  register it declares, a directory with a convention of its own. The plans directory and the
  roadmap are the primer's rows, not the project's;
- **the project's skill prefix**: its name and a hyphen, which names everything of the project's
  own in the directories the install writes into. A name that begins with `knowledge-architect-`
  there is the installer's: the install deletes it. A project whose name gives that prefix takes
  another.

How the project's rows are written is
`skill@knowledge-architect-agent-configuration@project-knowledge-rows`; the prefix and a project
skill are `skill@knowledge-architect-agent-configuration@shaping-a-skill`.

**Every level-two heading of the root `CLAUDE.md`, and of each project skill and agent, ends with a
slug**: two hashes and the id in backticks, the id naming the section's subject. The check reports a
heading without one. A skill's directory and an agent's file are named in two or more lower-case
words joined by hyphens, and a frontmatter `name`, where one is set, equals that name. The slug is what a reference to the section
cites: `instructions@<slug>` for the root `CLAUDE.md`, `skill@<name>@<slug>` for a skill and
`agent@<name>@<slug>` for an agent.

## The gates `##setup-gates`

Recommend one command that runs every check the project owes before a merge (formatting, the
document check, the commit messages, the linters, the tests), runs them all even when one fails,
and exits non-zero when any fails. A verdict is then one exit code, and nothing is read from output
filtered through a pipe. In a Rust project, the shape is the maintenance crate of
`skill@knowledge-architect-setup@rust-project` below, run through the cargo alias `cargo x gates`, whose gates command hands the project's gate
list to the published library knowledge-architect-gates.

A maintenance tool of that kind is a Component of its own, which serves the project rather than its
consumers. Propose these two goals for it, under `skill@knowledge-architect-goal-setting`, for the
owner's ruling like any draft:

```markdown
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
```

## Finish `##finish-setup`

- `cargo klarch check --fix` until it passes: it writes the installed files, then, once the first
  three phases of the check find nothing, the generated `index.md` files the check would report,
  then runs every check.
- Commit the manifest, the documents, the installed files and the root `CLAUDE.md` together,
  ordered on the branch as `primer@branch-commits` says, read whole.

## Existing documentation `##existing-documentation`

A project that already has documentation keeps it until its move is planned:

1. **Inventory** every document that records decisions, goals, open work or conventions: design
   documents, decision records, notes, an issue tracker, a `CLAUDE.md` or another agent's
   configuration.
2. **Propose a destination for each**, by the primer's knowledge table: a goal, a design entry, a
   rejected design, an issue, a tripwire, a scoped `CLAUDE.md`, a skill, a comment at the code
   for a decision that earns no design entry, or nothing, for history that records no current
   decision. Say what is uncertain: which recorded decisions still hold, and
   which describe a design the code has left.
3. **A destination a row of the knowledge table fits is the session's**, reported with its row, per
   `primer@question-authority`. **The owner rules** on a document no row fits, as the table's last
   row asks, each by a label, `Q<n>`, given to it in the
   proposal. The issue below names each document by its path, not by the label.
4. **Open one `todo` issue for the move**, in the root Component, holding the inventory and the
   rulings. The move is then planned work: when the owner schedules it, it starts under
   `skill@knowledge-architect-design` as a requested change, with the issue as its grounding, and
   its full path hands it to `skill@knowledge-architect-planning`. Until then, the
   old documents and the new homes both exist, and the issue is what records that. The move's
   work ends by running the design-record axis of
   `skill@knowledge-architect-project-audit@design-record-axis` over its whole corpus, which then
   holds what the move wrote.

**A finding the check reports in an existing file is the owner's to rule**, since the setup does
not move or rewrite that file. Show the owner the finding and its repair, each finding under a
label, `Q<n>`, when there are several, such as a bare path
rewritten as a reference. The owner rules between that repair and a
`[walk] skip-files` row, with its reason beside it, that keeps the file out of the walk until its
move. A file at the path of a register home, such as `register@*@design`, and the root
`CLAUDE.md`, whose import of the primer the check reads, have no such row: the check refuses a walk
row over either, so their findings are repaired.

Setting up stops at a conformant structure, the goals, and that issue.

## Moving the pin `##moving-the-pin`

To move to another version:

1. Edit the pin: both versions in the maintenance crate's `Cargo.toml`, the checker's and the
   gates library's, which move together, or the version of the local install; and
   `[project] checker-version` in the manifest, which moves with them. Nothing compares the gates
   library's version, so it is edited in the same change.
2. Read the changelog of every version crossed. Any version but a patch may make a check
   stricter, a version may make one looser, and either changes what a test that pins findings
   sees; a major version may ask for a change to the project's layout. Each published crate ships its
   CHANGELOG.md. Fetch the new version first, with `cargo fetch` after editing the pin in a Rust
   project, or by the install; then read the file in the source cargo downloaded, under its
   registry directory: `$CARGO_HOME/registry/src/<index>/knowledge-architect-<version>/CHANGELOG.md`.
   List each Migration entry that cites an axis's section of
   `skill@knowledge-architect-project-audit`, for step 5.
3. Run `cargo klarch install-agent-skills`, then follow
   `skill@knowledge-architect-agent-configuration` for what an upgrade owes the project's own
   configuration.
4. Run the project's gates command of `skill@knowledge-architect-setup@setup-gates`, or, where it has none, `cargo klarch check` and the
   project's tests, and commit the pin, the installed files and the repairs together, ordered on
   the branch as `primer@branch-commits` says. A new version can change what a command prints,
   which only the tests see.
5. For each axis that step 2 listed, run it under `skill@knowledge-architect-project-audit`, in a
   branch of its own started from the main branch once the pin move has merged into it, so that
   the audit's range holds the audit's work alone. Such an entry changes the rules the project's
   existing record must meet, and the audit brings that record to them.

## In a Rust project `##rust-project`

A Rust project gets its gates and its pinned checker from one maintenance crate, a package named
`xtask` in a directory of its own, `<xtask-dir>/`, such as `xtask/` at the root, a member of its
workspace that is never
published. The shapes below are illustrations, to adapt.

**The crate.** It depends on the checker and on the gates library, both pinned exactly to the
version the project uses, and on clap:

```toml
[package]
name = "xtask"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
clap = { version = "4", features = ["derive"] }
knowledge-architect = "=<version>"
knowledge-architect-gates = "=<version>"
```

**The aliases**, in `<workspace-root>/.cargo/config.toml`: `cargo x` runs the crate's own commands, and
`cargo klarch` runs the checker it pins, in release mode. Declare `cargo klarch` as the project's
command (`skill@knowledge-architect-setup@declare-the-command`).

```toml
[alias]
x = "run -q -p xtask --"
klarch = "run -q --release -p xtask -- klarch"

[env]
<PROJECT>_CHECKOUT = { value = ".", relative = true, force = true }
```

**Tie every build to its checkout**, with the `[env]` entry above. Two checkouts of the project
that build into one target directory otherwise run each other's build: cargo keys a workspace
member's build by its path relative to the workspace root, judges it fresh by modification times,
and does not track `CARGO_MANIFEST_DIR`. It does track the value of a variable a crate reads, and
`relative = true` gives each checkout its own root as the value. So:

- every library root of the workspace reads it, `const _: Option<&str> = option_env!("<PROJECT>_CHECKOUT");`;
- a target of a package with no library, such as the crate's main and its integration tests,
  reads it itself;
- every build script prints `cargo:rerun-if-env-changed=<PROJECT>_CHECKOUT`, since its run is
  cached apart from the crate it builds, and a tied crate otherwise compiles over the output of a
  run made in the other checkout.

`force = true` is required: `cargo run` exports the entry to the program it starts, and a cargo
that program starts from inside another checkout, such as a copy of the tree, otherwise inherits
the outer checkout's value.
Cargo reads the entry from the directory it is started in, not from the manifest it builds, so
the tie holds for a cargo started inside the checkout it builds: one started elsewhere with
`--manifest-path` takes the other checkout's value, or none. `option_env!` rather than `env!` keeps
such a build compiling. The cost is
a rebuild at each switch between checkouts that share a target directory, and nothing in a single
checkout. The refusal in the main below stays, and detects a binary built before the tie.

**Its main**: a `gates` command over the recommended list of the gates library, and a `klarch`
command carrying the checker's own commands. **Every binary that dispatches a command of the
checker calls `cli::refuse_another_version` before it dispatches**, as the `klarch` arm below does;
without the call, that binary enforces no pin. Where the `cargo klarch` alias runs an extension's
binary instead, that binary calls it, and a maintenance crate that dispatches no command of the
checker, using the library only for `MANIFEST_NAME`, does not.

```rust
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use knowledge_architect::cli;
use knowledge_architect_gates::{project_root, rust_project, Checker, GatesArgs};

// Ties this crate's build to its checkout.
const _: Option<&str> = option_env!("<PROJECT>_CHECKOUT");

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run every gate a branch must pass before it merges.
    Gates(GatesArgs),
    /// The knowledge-architect checker, at the version this crate pins.
    Klarch {
        #[command(subcommand)]
        command: cli::Command,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Gates(args) => {
            let cwd = std::env::current_dir().expect("a working directory");
            let Some(root) = project_root(&cwd, knowledge_architect::MANIFEST_NAME) else {
                eprintln!("xtask: not inside the project");
                return ExitCode::FAILURE;
            };
            let checker = Checker {
                package: "xtask",
                prefix: &["klarch"],
            };
            knowledge_architect_gates::run(&root, &rust_project(checker, "origin/main"), &args)
        }
        Command::Klarch { command } => {
            let own = Path::new(env!("CARGO_MANIFEST_DIR"));
            let outcome = cli::locate().and_then(|manifest| {
                cli::refuse_a_foreign_build(
                    manifest.root(),
                    &[
                        cli::this_library(),
                        cli::Library {
                            crate_dir: own.to_path_buf(),
                            package: env!("CARGO_PKG_NAME"),
                        },
                    ],
                    &[env!("CARGO_PKG_NAME")],
                )?;
                cli::refuse_another_version(&manifest, &[own])?;
                cli::run(command, &manifest, &[own], &mut [])
            });
            outcome.unwrap_or_else(|e| {
                eprintln!("error: {e}");
                ExitCode::from(2)
            })
        }
    }
}
```

**The gates.** `rust_project` gives, in cost order: `rebased` behind `--require-rebased`, `fmt`,
`check`, `commits` over the branch's own commits, `clippy` with warnings denied, and `test`. Add
a gate the project owes by pushing a `Gate` to that list, with the distiller that shows its failure;
remove one only on the owner's word. Pass the main branch as git names it, such as `origin/main`.
The tool writes every gate's complete output under `target/gates/`, refuses to write its report
into a pipe outside GitHub Actions, and exits non-zero when any gate fails: read a run in the terminal, or redirect it to
a file.

**A project with an extension** depends on the crate that holds its extension instead of the
checker alone, and its `klarch` command registers the extension, as that crate's documentation
shows. Its gates' `Checker` names the same package and prefix. Before writing the extension's tests,
read the section "Testing an extension" of the checker's crate documentation: it says which findings
a mock plants, and what a test pins.

**Continuous integration** runs the same command on every pull request that is ready, with a
history deep enough for the range and the rebase check:

```yaml
name: ci
on:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]
permissions:
  contents: read
jobs:
  check:
    if: ${{ !github.event.pull_request.draft }}
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v5
        with:
          ref: ${{ github.event.pull_request.head.sha }}
          fetch-depth: 0
      - name: gates
        run: cargo --locked x gates --locked --fail-fast --require-rebased --full
      - name: gate logs
        if: failure()
        uses: actions/upload-artifact@v7
        with:
          name: gate-logs
          path: target/gates/
```

`cargo --locked x` keeps cargo from rewriting a drifted `Cargo.lock` before the tool starts, and
the tool's own `--locked` reaches every gate that resolves dependencies. The head commit is checked
out rather than a merge commit, so the range the commits gate judges is the branch's own. A draft
is skipped, and GitHub counts a skipped job as passing: whoever merges reads the draft flag, and
checks that the job ran on the pull request's current head. A hung job stops at its time limit
rather than the default of six hours.

**A crate the project publishes** keeps its `README.md` for the project's own readers, with the
references the checker resolves, and gets a separate, short `CRATES-IO.md` as its crates.io page,
named by `readme = "CRATES-IO.md"` in its `Cargo.toml`; cargo packages that file even when
`include` does not name it. Propose this to the owner. The page says what the crate is and how to
install or use it, and links, as absolute URLs, to the README in the repository and to docs.rs. It
restates no contract, so the two texts cannot drift, and it holds no backticked reference: a
reference that resolves passes `cargo klarch check`, and crates.io renders it as dead code.
