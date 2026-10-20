---
name: knowledge-architect-issue-tracking
description: MUST use before diagnosing any behaviour as a new problem or defect; when parking anything for later (an unexplained observation, an open question, deferred work, a suspicion); when a recorded tripwire fires; and when work closes any of these. Covers where the issue and tripwire registers live, the entry file shape, the entry kinds and their required subsections, the cold-reader standard, and the movement instruction between the two kinds.
---

# Tracking open issues and tripwires

| register | home | holds | it needs doing |
| --- | --- | --- | --- |
| `issue` | the directory `open-issues/`, one file per entry | what is **outstanding**: a defect, an observation nobody has explained, a question nobody has answered, work known to be missing | yes |
| `tripwire` | `tripwires.md`, or a `tripwires/` directory, one heading per entry | evidence that would **flip a recorded decision** | no: it is a hypothesis about a future failure, plus the response |

Keeping them apart is the point. A tripwire mixed into open issues inflates the count of things
that look actionable, and that count is what makes an issue listing rankable.

**Where they live is the project's manifest, `knowledge-architect.toml`, and it is the only
answer.** Every Component carries both registers under its `docs/`: `register@*@issue` and
`register@*@tripwire`. The project's root is a Component too, so its pair is like any other. A
**location** is a directory the manifest declares to carry registers outside every Component,
with the homes directly under its own path, for instance an `open-issues/` directory for what is
outstanding about the agent configuration. `cargo klarch check` asserts that every home exists and
that every entry has the shape below. A tracker nobody declared is not a tracker: nothing reports
it and nothing counts it.

**A tripwire goes in the same anchor as the decision it guards**, because that is where the
decision is recorded and the tripwire is unreadable apart from it.

**A check that can only be applied once unbuilt work is built is not a tripwire.** It is what that
work must prove, and it belongs with the work's planning, under
`skill@knowledge-architect-planning`. A tripwire is for a decision whose work exists, or that no
work implements.

## What is outstanding, across every register `##outstanding-across-registers`

```sh
cargo klarch issues                          # every issue entry, one row each
cargo klarch issues <anchor>                 # the entries of one Component or location
cargo klarch issues <text>                   # only the rows whose id or title holds that text
cargo klarch issues --kind <kind>            # only the entries of one kind
cargo klarch issues --group <group>          # only the entries in one group subdirectory
cargo klarch tripwires                       # every tripwire entry, and the decisions each guards
cargo klarch tripwires --guarding <ref>      # only the entries guarding that decision or goal
cargo klarch show <kind>@<anchor>@<id>       # one entry in full, and every reference to it
```

All three read the entity table at run time, the same table `cargo klarch check` resolves against,
so no count is stored and none can go stale. The first positional argument is an anchor when
something declares that name, and search text otherwise. An `issues` row is kind, anchor, id,
title and the date of the entry's last commit. A `tripwires` row is anchor, id, title and every
reference the entry carries to a decision, a goal or a declared register's entry. No row is exit 1.
The listing exists because the registers are spread over every Component and location, so grepping
the one you happen to think of is not the check.

## Read before you diagnose `##read-before-diagnosing`

**Before concluding that a behaviour is a new problem, list the issues and the tripwires of every
anchor**, `cargo klarch issues` and `cargo klarch tripwires`, and read closely the entries of the anchor
it appears in, `cargo klarch issues <anchor>`, and those whose title comes near. Diagnosing a
recorded issue again costs a session and produces nothing. The same applies before attributing a
measurement to a change.

## An issue entry `##issue-entry`

One file, `<id>.md`, in the anchor's `open-issues/` directory or in one of its declared group
subdirectories. The id is the file's basename, matches `[a-z0-9]+(-[a-z0-9]+)+`, says what the
entry is about, and is unique in the directory. It is what every `issue@<anchor>@<id>` reference
names. The file's shape, which `cargo klarch check` asserts:

```markdown
---
kind: defect
---
# The title, one line, plain text

## Summary

What the listing and a reader skimming need. Any length.

## Details

### What

### Why it matters

### What would close it
```

- The frontmatter carries `kind` and nothing else, with a value from the closed list below. A key
  nobody declared is a finding, so a typo cannot pass as an absent optional.
- The title is the level-one heading, and it is what `issues` prints.
- `## Summary` then `## Details`, in that order.
- Under `## Details`, the three level-three subsections in that order. A `deferred` entry carries
  `### Trigger` in place of `### What would close it`. Further level-three subsections may follow
  them.

The three subsections are the three fields every kind owes:

- **What**: the observation, question or missing work, stated so a reader who was not there knows
  what is being claimed.
- **Why it matters**: the consequence of leaving it. **Without this the register only grows**:
  nothing can be ranked and nothing can be deleted with confidence.
- **What would close it**: the measurement, experiment or work that retires the entry. An issue
  with no named way to close it is a wish. Write the wish down as a question with an experiment
  attached, or leave it out.

**`Why it matters` names what the entry stands against**: the decision it strains, as
`design@<anchor>@<id>`, and the goal it threatens, as `goal@<anchor>@<id>`, when it does directly.
That is what lets `cargo klarch show` on the decision list what is outstanding against it before it
is reopened, and on the goal what stands between the project and it. A reference is a claim that
the text is revisited when the entry it names changes, per `primer@where-knowledge-goes`. An entry
that strains no recorded decision names none rather than the nearest one. The inverse holds in
code: a guard, a workaround, a stub or a test that exists because of an entry names it,
`issue@<anchor>@<id>`, in the comment at the site. Closing the entry then dangles the comment, and
the site is revisited.

**Every entry states its kind**, in the frontmatter. Without it, a missing section is ambiguous
between "this kind has none" and "the author did not write one", and that is exactly what a reader
needs in order to decide whether they can act.

| kind | what it adds to the three fields |
| --- | --- |
| `defect` | the **full cold-reader standard** below |
| `observation` | something seen whose status as a defect is **not established**: it may be correct behaviour. The exact conditions it was seen under, and explicitly whether it reproduces: if it does, the reproduction as for a `defect`; if not, what would make it so |
| `question` | the question, and the experiment, derivation or reading that would answer it |
| `todo` | work known to be missing that nobody has scheduled and no event gates. Its closing condition is doing it. **If a spec or a milestone document schedules it, it is not a `todo`**, per `skill@knowledge-architect-planning@spec-or-milestone`: the document is the schedule, and two schedules drift. A row of the roadmap, docs/roadmap.md, orders a `todo` and schedules nothing, so the entry stays a `todo` |
| `deferred` | what is missing, the consequence of leaving it, and the **trigger** that should make someone do it. The axis against `todo` is scheduling, not urgency: `deferred` waits for a named event, `todo` waits for someone |
| `design` | the instances seen, the suspected mechanism in one sentence, and the re-entry point: the discussion at which it is raised again |

**The list is closed.** It is the compiled default of the `issue` register, and the manifest's
`[registers.issue] kinds` is the one place that replaces it. An unknown kind is a finding naming
the list. An entry fitting none of these is written under the nearest kind, with the three required
fields and whatever else a cold reader needs, and the fit is said in the entry. Adding a kind is a
reviewed manifest change.

**A plan document that leaves opens a `question`** on each other plan that cited it, a milestone's
own documents being one plan: does the citing plan
still hold now that the leaving plan is built? Its `Why it matters` cites what the leaving plan
harvested, and the entry cites the citing plan, so it cannot outlive it. When it is opened is
`skill@knowledge-architect-planning@plan-document-leaves`.

**Groups.** A directory holding many entries may declare group subdirectories in a `register.toml`
beside its README, `groups = ["a", "b"]`, and file entries under them. A group is not part of the
id, so regrouping is a `git mv` that breaks no reference. An entry may stay ungrouped.

**The index.** `index.md` beside the README is generated, one row per entry, and
`cargo klarch check` fails when it is stale. After creating, deleting, retitling, regrouping or
changing the kind of an entry, regenerate it and check in one command:

```sh
cargo klarch check --fix
```

Delete an entry with `git rm`, or stage its deletion: the check reads git's listing, and a deletion
it still lists stops the run in phase 2, before the index is written.

`cargo klarch index` writes the generated files alone, without the check's last phase: like
`cargo klarch check --fix`, it writes no generated file while the first three phases find anything.

**A commit of part of the working tree** needs the index of what it commits, which differs from
the working tree's when an unstaged change touches the same register. Stage the part, then:

```sh
cargo klarch index --staged
cargo klarch check --staged
```

`index --staged` writes the staged tree's generated files into git's index and touches no
working-tree file; `check --staged` judges what the commit will record. `cargo klarch check --fix`
refuses while a generated file it would write differs from the one the staged tree needs, and
names both repairs.

**One home does not bind an issue entry.**

- Two entries may carry the same measurement, and an entry may restate a figure that also lives in
  a head.
- Both registers hold statements that **leave**: an issue when it closes, a tripwire when it fires.
  The cost of the alternative lands in the worst place. Opening an entry is nearly always churn
  inside a session that was doing something else and found a problem, and checking that no other
  entry or head carries the same figure is a tax on the one act this register exists to make
  cheap. List the entries as `skill@knowledge-architect-issue-tracking@read-before-diagnosing` says,
  and write what the entry needs.

Only `defect` carries a mandatory checklist, and only because that checklist is already written and
already shared with the commit messages of fixes. The other kinds carry required fields, not a
template: a template imposed on a kind that cannot fill it produces empty headings, and empty
headings teach readers to skim the entries where the checklist does matter.

## The trigger test `##trigger-test`

A `deferred` entry's trigger and a tripwire's firing evidence answer the same question: what will
make someone do this. Both are subject to one test.

**A trigger must name an occasion whose own work already includes the work the trigger names.**
Otherwise it names a tax on a session doing something else. Such a session finishes its own task
and reports what it met outside it, rather than doing that work. The trigger fires, the session
correctly declines, and nothing schedules the work.

A trigger passes when it names the change that would make the missing work necessary: whoever makes
that change is already deciding what the work must do. A size threshold on a document fails it: it
fires on whoever happened to add the last line, and that is nobody in particular.

**Enforcement does not rescue a failing trigger: it sharpens the failure.** An assertion in the
checker or in the project's tests fires on the session that crosses the bound, and every commit
must pass it, so at the bound that session can neither do the work mid-task nor leave the tree
committable.

**When a trigger fails the test there are two answers, and only two:** name a different occasion,
one whose own work includes this, or do the work now, in a change of its own.

## A tripwire entry `##tripwire-entry`

A level-two heading carrying a slug at its end, in the anchor's tripwires home, and its body:

```markdown
## Guarding `design@<anchor>@<id>`: <what the guard is about> `##<slug>`

**Fires when:** …
**Response:** …
**Re-entry:** …
```

- **the decision it guards**, as a `design@<anchor>@<id>` reference in the heading or the body, so
  that a reversed decision dangles its tripwires mechanically, and `tripwires --guarding <ref>`
  finds them. One guarding an instruction, a guarantee, or a decision that earned no head names
  what it can, the last naming in words the site that records it, and stays legal with no
  reference;
- **the firing evidence**, meeting the falsifiability bar: an event, or a count crossing a bound,
  specific enough that both parties would agree it fired. "It gets slow" is not a tripwire;
- **the response**: reopen the decision by its reference, or open an issue of a named kind;
- **the re-entry point**: the checkpoint at which it is read again.

The slug is what `tripwire@<anchor>@<slug>` references name, and it follows the id grammar above.
Every level-two heading of a tripwires home is a tripwire: a heading there with no slug is
reported. A slug at another level, at the head of a plain line, or in the `README.md` of a
directory-shaped home, defines nothing, and outside a fenced block it is reported as misplaced.

When a tripwire is written, and on whose word, is
`skill@knowledge-architect-decision-recording@ruled-tripwires`.

One standing re-entry point: `agent@knowledge-architect-standing-state-reviewer` reads every
tripwire home and every deferred trigger again, on the review axis whose whole subject they are. An
entry may name a narrower one.
**A tripwire nobody reads again is a parked item with no re-entry point**, which is the thing this
register exists to avoid, not to become.

**If a script could check the firing evidence, it is not a tripwire: it is an assertion**, a test
or a check the project runs on every commit. A threshold with no enforcement is a wish. Apply
the trigger test above first, though: an assertion whose occasion fails it blocks a session rather
than scheduling anyone.

## The movement instruction `##movement-instruction`

- A tripwire that **fires** has its heading and body **deleted** from the tripwires home, and an
  issue file is **created**, or the decision is reopened. It never becomes a fired-but-still-listed
  entry. Where the reopened decision's reversal is recorded only at a plan document's harvest, the
  tripwire is narrowed to except that plan document's work, citing it, so the citation dangles when
  the document leaves and its harvest deletes the entry.
- **Unless it guards a standing guarantee, in which case it is restated and stays.** A tripwire is
  usually a one-shot hypothesis, _if this happens, that decision was wrong_, and firing consumes it.
  A tripwire guarding a guarantee that holds for the life of the project is not consumed by one
  instance of it being broken: the guarantee is still owed after the repair. Restate it so it names
  the _class_ rather than the instance, and record the instance as the issue.
- A tripwire whose decision is **reversed** is deleted outright.
- A tripwire is **absorbed** when another entry already guards the same decision: fold its firing
  condition into that entry rather than leaving one decision guarded from two places, where a
  reversal voids only half of itself.
- A tripwire is **retired** when its firing evidence names an artifact the project's own
  instructions prevent from existing. It is deleted, and the guard becomes a predicate a reader
  applies, in whichever review axis owns its subject or in the instruction itself.
- An `observation` that becomes reproducible is **promoted in place** to `defect`: its `kind`
  changes and it acquires the checklist at that moment. Do not rewrite it from scratch, and keep
  what was already ruled out.
- **An issue file is deleted when the entry closes.** A fixed defect is **deleted, not marked
  resolved**: the history of the fix belongs to its commit message and nowhere else. Regenerate the
  index in the same change.

If a closed entry still holds something live, such as an instruction about how to work in that
area or an uncertainty that survived the fix, that content is not an open issue. Move it to the home
the knowledge table of `primer@where-knowledge-goes` gives it, **then** delete the file.

Deleting or renaming an entry means fixing what points at it. Every `issue@<anchor>@<id>` and
`tripwire@<anchor>@<slug>` reference to a deleted entry becomes a dangling-reference finding of
`cargo klarch check`, a comment in Rust source included, and that list is the work list the deletion
produces. A reference in a file the checker does not read is found by grep. A commit message that names the closed entry resolves against the parent commit's tree,
so the closing commit may still name it. `cargo klarch show <ref>` prints the inbound references
before you delete.

**A sentence about the past whose reference dangles is rewritten to state the present, or
removed**: what an earlier step wrote, what an entry once said. Its history stays in the commit
messages. It is never retargeted to the new name, which would make it false, and never turned into
plain text, which takes it out of the check. A verbatim quotation of the owner that names a renamed
entry is left as it is, with a reference to the current entry beside it. A record of a deliberation
names the entry as `primer@where-knowledge-goes` says instead.

**An entry that records a missing checked form** is what a pointer the checker cannot express is
written beside, in plain text, as `primer@where-knowledge-goes` says. Its `What` names the
pointer's class and the form that would express it; its `What would close it` is that form
shipping and every site that cites the entry converted to it. `cargo klarch show` on the entry
lists those sites. A gap of the checker itself is recorded this way in the project that meets it, in the issue directory of any of
its anchors, since `cargo klarch show` lists the entry's sites wherever it sits, and is reported to
the workflow's maintainers by the retrospective.

## The cold-reader standard `##cold-reader-standard`

Anyone who did not witness the work must be able to reproduce what the entry describes and start
from the entry alone. It binds every issue entry and the commit message of every fix. Each carries,
as far as they exist:

- the types, files and **function names** involved. **Never line numbers**, which go stale at the
  next edit, and **never commit hashes**: name the change instead;
- the exact numbers measured: magnitudes, tolerances, counts, the parameter values that make the
  behaviour appear and disappear, not qualitative wording;
- paths to whatever reproduces it, written as `path` references per
  `primer@where-knowledge-goes`, and a snippet inline for anything not committed. **An entry must
  never depend on a scratch directory**;
- what was **ruled out and by what evidence**, so the next reader does not repeat the eliminations;
- an explicit `assumption` or `not established` label on anything unverified.

These entries are longer than the text around them on purpose,
and the length is not a reason to compress them.

## Reviews `##reviews-and-entries`

A review's findings that stay open become entries in the affected anchor's `open-issues/`
directory, each of the kind the table above gives it. The review document itself is a working
artifact and is not a durable home. This is deliberately one mechanism rather than a separate
review tracker with status tokens: three trackers need a script to answer "what is outstanding?",
and then the script is the mechanism.
