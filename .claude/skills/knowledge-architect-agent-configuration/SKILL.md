---
name: knowledge-architect-agent-configuration
description: MUST use before adding or editing the project's root CLAUDE.md, a scoped CLAUDE.md next to code, a project skill, or a project subagent definition, and after an install that changes the installed files. Covers choosing between root, a skill, a scoped file and an agent; how a skill's scope and description are shaped; the standard for what enters agent-facing text and how a change to it is shaped; the project's rows of the knowledge table; and why an installed file is never edited.
---

# Maintaining the agent configuration

Scope: the project's own agent configuration, the text an agent reads to work in this project.
Its root `CLAUDE.md`, its scoped `CLAUDE.md` files, its own skills and its own subagent definitions.
Its standard for what enters agent-facing text,
`skill@knowledge-architect-agent-configuration@admission-standard`, also governs the installed text
wherever a session judges it or proposes a change to it.

Not covered here: **editing the installed files**, every file that `cargo klarch install-agent-skills`
writes. They are never edited by hand (`skill@knowledge-architect-agent-configuration@installed-files-never-edited`). **Where the argument
for a decision about the configuration lands**: `skill@knowledge-architect-decision-recording`.
**Setting the configuration up the first time**: `skill@knowledge-architect-setup`.

## What enters agent-facing text, and how a change to it is shaped `##admission-standard`

This standard binds every statement of agent-facing text, the primer, a `CLAUDE.md`, a skill, a
subagent definition and a saved workflow's prompts included, and every actor that writes or judges
one: a session writing it, a review, an audit and its repairs, a retrospective's proposals, an
intake of a retrospective's findings, `skill@knowledge-architect-retrospective-intake`, and a skill
that moves text into agent-facing text.

### The need test

Every statement in agent-facing text passes one test: **name the act of an agent following the
workflow that would differ without it**: what the agent does, when, where, or a fact it must use.
A statement for which no such act can be named is not written, and an existing one leaves as the
table below gives. The
unit is a sentence, or a clause when only part of a sentence fails.

- **Whose act.** Any agent that reads the text in its work counts, an agent that maintains the text
  or writes a brief from it included. A statement whose only reader is the text's maintainer is at
  most one sentence; in installed text it is a comment of the workflow's source, removed by the
  build, since no installing project maintains the installed text.
- **A reason** passes when it states the mechanism an instruction relies on or the failure it
  prevents, even when the instruction has no exception. A second reason for the same instruction
  passes only when it names another mechanism or failure. A sentence that narrates where an
  instruction came from or how it turned out fails: "learned by getting them wrong", "and it
  worked".
- **A duplicate inside one text fails while its twin stays.** A text is read whole once it is
  loaded, so a second statement of the same act adds none. The finding names the copy that stays:
  the one in the section whose subject it is. The primer and the root `CLAUDE.md` that imports it
  are one text, since both are loaded together, and the primer's copy is the one that stays. Across
  other texts, `primer@where-knowledge-goes` decides.
- **A lead, a transition or an emphasis** fails when the statements it introduces or stresses
  already carry every act. A heading is structure, not a statement.
- **A remark about the text itself** fails unless it tells a reader where to look or what to edit:
  what the text is classified as, or that something is not restated here.
- **A definition** passes when an instruction anywhere in the workflow uses the term. **An example**
  passes when it shows how a definition or an instruction applies.
- **A pointer is needed** when its target is needed to act, or when it names the home of a
  restatement.
- **A statement that can turn false with no change to the project** (the state of another project,
  a tool's version, a person's plan) is suspect: it is written only when an act needs it. "This
  issue was reported upstream" fails, since no act differs.

Wording and style follow `primer@working-prose`; placement across files is judged by other rules.

Current reality only: no history, no dates, no commit hashes, no session records. Examples are kept
minimal.

### Evidence and kind

**A defect provable by reading needs no other evidence**: a contradiction, a broken trigger, a
factual error, an instruction whose words allow two acts, a provable gap, divergent verdicts that a
review standard and an audit standard give over one text, a statement that fails the need test.

**An instruction about how an agent behaves is added only on evidence from real use**, in the
installed text and in a project's own text alike: a behaviour seen in a real session, produced
unprompted or asked for by the owner, with the owner naming what the session would have lacked
without it, and one sentence stating the mechanism that produced the need. A gap that only
predicts what an agent would do is no edit: it is an issue entry stating what a real session would
have to show. A provable gap is an input an act needs that no text supplies and judgement cannot
derive, a fact fixed elsewhere such as a format the checker enforces, a name or a path; its fill
adds an instruction, so it goes to the owner.

**A lapse is not repaired by adding text.** An instruction that exists, is clear, and was not
followed stays as it is. Where the lapse recurs at one point of delivery, because the instruction
is read at one moment and needed at another, the fix moves where it is delivered, and the
recurrence is its evidence.

**The kind of a gap weighs it.** A missing capability, a move the agent does not have, is worth
text. A conformance rule, for a move the agent has and might not make, is worth text only where the
default behaviour is systematically wrong. A structure imposed on the work, a fixed sequence, count
or mapping between a unit of the work and a unit of its record, only where the work needs it; a
contradiction a structure causes is answered by removing structure. For an installed skill, a
finding about the owner's behaviour outside the skill's expectation set,
`skill@knowledge-architect-retrospective@expectation-sets`, is no gap; a finding that two installed
instructions leave no move satisfying both is always in scope. An installed agent has no expectation set, since it never works with the owner.

### The shape of a repair

**A finding is repaired at the cause that produced it, not at its site**: a repair written for one
finding adds a rule, and added rules interact at the next pass. A cause is one of four kinds:

- **two homes for one moment**: instructions about one moment or one kind of act stated in more
  than one text;
- **a structure the work did not need**: a fixed sequence, count or mapping that another
  instruction needs to vary;
- **a restatement that drifted from its home**;
- **a rule written for one interaction**: a narrow rule where judgement already decides.

A finding that fits none is a cause of its own, with no kind. A repair is the homes merged into one
and the others pointing to it, the structure removed, the restatement brought back to a pointer, or
the narrow rule deleted and the case left to judgement. It adds an instruction only where none of
these closes the cause. **An instruction** is a sentence that tells an agent to do, or not to do,
an act at a moment; a rewording that makes an existing instruction's intent clearer adds none,
whatever its length. Each repair counts the instructions it adds and removes, and names the nearest
rival to itself and the fact that decides between them. **When the repairs applied together add
more instructions than they remove, the owner is told so first, with the net figure**, and rules on
that growth before any of them is applied.

### The record check

**An edit that changes what an agent is told to do is a decision.** Before it is written, and before
the owner rules on a proposed repair, it is read against the record: every goals home; the design
heads that `cargo klarch show` and a search of the design homes return for the behaviour the edited
text describes; the rejected designs of each Component that owns an edited text; and the
scoped `CLAUDE.md` files and code comments that `skill@knowledge-architect-decision-recording@reversal-check`
searches. The check names each head the edit would reverse, each rejected design it brings
back and each goal it strains, with its quotation. An edit that writes into a design home, or edits
a text whose behaviour a head describes, loads `skill@knowledge-architect-decision-recording` before
it is written; that skill, with `primer@design-heads`, judges whether it contradicts a head, adds a
member its argument does not cover, or earns text at all. One that strains a goal goes to the
owner, under `skill@knowledge-architect-goal-setting`. A rewording that changes no instruction is
not a decision.

A project's skills and agents are walked by `cargo klarch check`, and a live reference in them must
resolve. Run it after editing one.

### The outcomes of a need-test failure

| the failing statement | outcome |
| --- | --- |
| it carries no instruction: narration, a remark about the text, outside state no act uses, a duplicate whose twin stays, a lead or an emphasis whose statements carry every act | deleted in the branch: by the session, which applies an audit's outcomes, or as a review's repair |
| it is maintainer-only, in installed text | moved to a comment of the workflow's source |
| it is maintainer-only, in a project's own text, and longer than one sentence | cut to one sentence |
| it is an instruction whose act no agent performs | put to the owner, since deleting it changes what agents are told: on an audit's owner list, as a default awaiting the owner in a plan document, or as a question in the turn elsewhere |
| a suspect statement that passes | no finding; a suspect statement found false is a factual error |

In a project that installs the workflow, a finding on installed text, of any row, is sent upstream,
as every finding on installed text of an audit is.

## Where a piece of agent-facing text goes `##where-text-goes`

An agent needs two kinds of text: **knowledge** about the project (what exists, how it works) and
**instructions** about the tasks it performs. They are split first by kind, then by **when the place
reaches a session**.

### Knowledge

| place | when it reaches a session | what it holds |
| --- | --- | --- |
| root `CLAUDE.md` | every session, unconditionally | the knowledge every task needs |
| a scoped `CLAUDE.md` | when a session reads, writes or edits a file in that directory, or opens it | the contracts and traps of one Component |
| an issue directory | when a session searches for open issues: before recording a new one, or when asked to fix one | what is outstanding in that Component or location |

### Instructions

| place | when it reaches a session | what it holds |
| --- | --- | --- |
| root `CLAUDE.md` | every session, unconditionally | instructions that span all activities, the project's rows of the knowledge table |
| a skill | its description is listed every session; its body loads only when invoked | a procedure, and the knowledge shaped by a task. Length is nearly free, so a skill can be thorough |
| a subagent definition | only when a subagent of that type is spawned, and then from its first token | a task only ever done by a subagent, such as a reviewer following a fixed standard |

**The primer is an extension of the root `CLAUDE.md`**, which imports it: the part every adopting
project's root would say alike. The root `CLAUDE.md` never restates an instruction of the primer; it holds what is the project's own, and a statement it shares with the
primer is a duplicate inside one text, which the need test deletes.

#### Dispatching between them

**The unit is an activity**: the scope over which a complete set of procedures makes sense.
Decided in order; the first match wins.

1. **Is it needed by every activity?** Then it is **baseline**: root `CLAUDE.md`. **Reading is
   always baseline**, because a session cannot know what it will read before reading it, and so
   cannot know which skill it would have needed. **Producing is an activity, unless the producing
   is itself universal**, as committing is: every session commits.
2. **Otherwise, does it produce a nameable artifact?** Then it is an **activity**, and it owns a
   skill. The discriminator is artifact versus property, not locality: work that produces one work
   list across every file it touches is still an activity, while a property that must hold in
   every file produces no artifact and is baseline.
3. **Is it shared by two or more activities but not all?** Then it is a **sub-activity**: its own
   skill, named as a prerequisite by each activity that needs it. The installed decision-recording,
   issue-tracking and review skills are this.
4. **Is it read only by a fresh subagent?** Then it is a subagent definition. A definition sets a
   **standard**, not a one-off task, because it must be reusable. A review axis whose content
   depends too much on the task to be standardized stays as a line in the activity's skill.

## Shaping a skill `##shaping-a-skill`

- **Name it by its activity, as a noun of one or two words in common usage** (planning, review,
  issue-tracking), never as an artifact the activity writes: the text names both, and a skill named
  like its artifact cannot be told from it. Give it the project's prefix: the project's name and a
  hyphen, as in `<project>-development`; a name that begins with `knowledge-architect-` is the
  installer's, and the install deletes it, so a project whose name gives that prefix takes another.
  The directory name and the frontmatter `name` are equal. The same prefix names the project's
  other files in the directories the install writes into.
- **End every level-two heading of a skill, a subagent definition and the root `CLAUDE.md` with a
  slug**: two hashes and the id in backticks, the id naming the section's subject. It is what a
  reference such as `skill@<name>@<slug>` cites, and the check reports a heading without one. A
  subagent definition's file name and its frontmatter `name` are equal, as a skill's directory and
  its `name` are where a skill sets one.
- **Begin the description with MUST**, and make the trigger **symptom-shaped, not request-shaped**.
  A session rarely asks to "track open issues"; it does meet a behaviour that looks wrong.
- **State what the skill does not cover**, naming the skill that does. A seam left unnamed becomes a
  dead zone where neither skill loads.
- When two skills legitimately both apply to one change, say so, and in what order.

**One skill covers one activity.** A description naming several unrelated scopes is matched by no
task, so it loads for none. The cause looks like the wording of the description, but the content
was scoped wrong before the description was written: fix the scope, not the wording. Enumerating the facets of one activity is the opposite move and is
correct. The test is whether a single task can want all of them at once.

**A skill is self-sufficient.** It holds every piece of knowledge its activity needs, such as a
build command, a dispatch table or a common trap, even where it is stated elsewhere, delivered as
`primer@where-knowledge-goes` says. **The test, per pointer: could a session complete this
activity correctly without opening it?** If not, the content is delivered in the skill as that
section says. A pointer passes when its target is not needed to act, as an argument is; when it
varies per instance, as task material does; or when it is one complete instruction the session
reads whole: a named prerequisite skill, a home the pointer asks to read whole, the primer or the
root `CLAUDE.md`, which reach every session. What this guards against is dilution rather than
length: each pointer is an extra read a session must remember, and the more there are, the less
likely all are followed.

## The project's rows of the knowledge table `##project-knowledge-rows`

The installed primer carries the workflow's own rows:
where a goal, a decision, a design not kept, an issue, a tripwire, a contract or a plan document
goes. The project's root `CLAUDE.md` carries the rows that are the project's alone, under a heading
of its own: its changelog, a register it declares, a directory with a convention of its own. The
last row of the combined table is **ask the owner before writing it anywhere**. It is for a
statement with no home, not for a choice between two: when two rows could fit, pick one, say which,
and carry on. A genuine gap means the table is incomplete, and what the table holds is a decision
about the shape of the configuration, which is the owner's. **Each answer ends as a new project
row**, so the fallback limits itself: if it fires often, the table is what needs changing, rather
than the entry. A project row may refine a row of the primer with what is the project's own, such as
a README that is also its package's page on a registry; it never contradicts one, and a row that
only repeats one is removed.

## Installed files are never edited `##installed-files-never-edited`

An installed file is compared byte for byte with the version the project pins, and the install
overwrites it. **A change the project needs is a project skill or agent of its own**,
with its own name. It adds to the installed text: an extra step,
an extra review axis, a convention of the project. It never contradicts it. Where it would have to
contradict it, the installed text is wrong for this project: say so to the owner.

## After an install that changes the installed files `##after-installing`

`cargo klarch install-agent-skills` writes the files the checker ships and removes the ones it no
longer ships. When it changes any of them, in the same commit:

- read the new primer's table against the project's rows, per
  `skill@knowledge-architect-agent-configuration@project-knowledge-rows`, read whole;
- read each project skill and agent against the installed text the install changed, for an
  instruction that now contradicts it;
- stage the files the install removed, since a deletion git still lists stops the check;
- run `cargo klarch check`.

## Reviewing a configuration change `##reviewing-a-change`

Dispatch a review when an instruction is written and a mechanism is in place to deliver it, per
`skill@knowledge-architect-review`. Its axis on agent-facing text,
`agent@knowledge-architect-agent-text-reviewer`, judges the change against
`skill@knowledge-architect-agent-configuration@admission-standard`.
