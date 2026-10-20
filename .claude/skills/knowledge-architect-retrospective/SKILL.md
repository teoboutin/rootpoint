---
name: knowledge-architect-retrospective
description: MUST use once per session, at the first of these moments (deferred to the end of a task under way), to offer a retrospective to the owner - a branch the session worked on has merged into the main branch, a plan document has left, or the owner says the session is ending - and run it only if the owner accepts. Covers what the retrospective examines (the installed files, the project's own instructions, how the two interact, and the checker), the standing questions it always asks, the expectation sets that bound what counts as a defect of an installed skill, the two files it writes outside the project, and where each file goes: the project's file to an intake the owner starts, the workflow's file to the workflow's repository.
---

# Retrospective

Scope: looking back at one session's use of the workflow, and turning what was unclear, missing or
wrong into findings the owner can act on.

Not covered here: **analysing the project's file and ruling on its findings**,
`skill@knowledge-architect-retrospective-intake`; **opening an issue entry in the project**,
`skill@knowledge-architect-issue-tracking`.

## When it is offered `##when-offered`

**Once per session, at the first of these moments**, and never during a task:

- a branch the session worked on has merged into the main branch;
- a plan document has left the repository;
- the owner says the session is ending.

A moment that falls during a task defers the offer to the end of that task. Offer it in one line,
and run it only if the owner accepts. A declined retrospective is not offered again in that session.

## What it examines `##what-it-examines`

Open with **what the session did**: one paragraph, at the level of the workflow ("a design
discussion and two merged branches"), not of the project's subject matter.

Then **the version the session used**, in one line:

- in both files, the version of knowledge-architect: where the project pins the checker, the exact
  version of the pin; where the project builds the checker from its own source, the main commit
  the session's tree contains, described with `git describe --tags $(git merge-base HEAD <main branch>)`,
  the branch the session worked on, and whether the tree held uncommitted changes;
- in the project's file, also the project's main commit the session's tree contains,
  `git rev-parse --short $(git merge-base HEAD <main branch>)`, the branch the session worked on,
  and whether the tree held uncommitted changes. The workflow's file does not name them, since it
  may become public; where the project builds the checker from source, the first bullet names
  that commit there.

A commit of the branch itself is not named: a merge that rewrites the branch's commits leaves it
pointing at nothing.

Then read back over the session: the instructions that were followed, where they were followed with
difficulty, where they were not followed, and what the owner corrected. Then, for each of the four
subjects below, list what was **unclear** (it could be read two ways), **missing** (the session
could not follow the workflow, or could not produce a document the checker accepts, without an
instruction that does not exist), or **wrong** (following it produced a defect or a correction). A
decision the session made by its own judgement where no instruction covers it is not missing: the
installed text leaves that room on purpose, per `primer@room-to-judge`. Each item quotes the instruction, says what happened when it was followed, and what was
done instead.

**Each finding is named by a letter and a number**, so that a fix, an issue or a commit can cite it
by its id and the stem of its file (`skill@knowledge-architect-retrospective@two-files`), as "W3 of <date>-<project>-<subject>-klarch-workflow":
**W** for the installed files, **C** for the checker, **P** for the project's own
instructions. Numbers follow `primer@labelled-items`, so an id is unique within its file, and the
stem makes the citation unique across retrospectives. An interaction
finding gets one id in each file, each naming the other, since the two files may be read apart. The standing questions below take no id: they are answers, not findings.

1. **The installed files.**
2. **The project's own instructions**: its root `CLAUDE.md`, its scoped `CLAUDE.md` files, its own
   skills and agents.
3. **How the two interact**: an installed instruction and a project instruction that contradicted
   each other, overlapped, or each assumed the other would cover something.
4. **The checker**: a defect; a blind spot, something it should have reported and did not; a false
   finding; and what would make it easier to use, such as a missing command or option, or a message
   that did not say what to do.

**Always ask these standing questions**, and answer each with what the session shows:

- Did this session need to change an installed file, and what for?
- Did this session, or one of its subagents, act as if a rule of the primer were absent? For each:
  the rule, who acted, and what they did.
- Did this session miss something a project skill adds to an installed skill?
- Did this session need to write a reference or a path without backticks, because no checked form
  expresses what it points at? For each: what it pointed at, the checked form that was missing, and
  whether it was written beside a reference to an issue entry of the project or bare. A missing
  form is also a finding on the checker, in the workflow's file. One written bare is also a finding
  on the project, in the project's file. Text outside the checker's syntax, such as a commit of
  another project, is no answer to this question.

**A ruling of the owner recorded a little wider, narrower or firmer than given, on a detail that is
not load-bearing, and caught by the transcript reviewer before the merge, is no finding.** The
reviewer exists to catch that expected rate. A misstated ruling that changed what was
built, or reached the main branch, is one.

**A finding about how the owner works is judged against the skill's expectation set** (`skill@knowledge-architect-retrospective@expectation-sets`). A
finding that describes the owner's behaviour where `skill@knowledge-architect-retrospective@expectation-sets` states the skill assumes otherwise is
reported as outside that skill's scope, with the assumption quoted, rather than as a defect of it.
A finding that two installed instructions leave no move satisfying both is always in scope,
whatever the owner did.

End with **proposals**: for each finding that has one, what to change, and where, shaped as
`skill@knowledge-architect-agent-configuration@admission-standard` says, read whole.

## Two files, sorted by whose text must change `##two-files`

Each finding goes to the file of the project whose text or code must change:

| file | holds |
| --- | --- |
| `<YYYY-MM-DD>-<project>-<subject>.md` | what the session did and the version it used, the findings on the project's own instructions, and the project's side of an interaction |
| `<YYYY-MM-DD>-<project>-<subject>-klarch-workflow.md` | what the session did and the version it used, the findings on the installed files and on the checker, the workflow's side of an interaction, and the answers to the standing questions |

`<subject>` names the session's work in a few words, in lowercase joined by hyphens, such as
`pin-move`. The session proposes it, and the owner may change it on reading the files. Before
writing, check that neither name exists in the directory: a file is never overwritten, and two
retrospectives of one project on one day differ by their subject.

An interaction finding whose fix may fall on either side goes in both files, each written from its
own side.

**The second file may become public.** It holds nothing of the project beyond what a finding needs
to be understood: no code, no names of the project's internals, no content of its documents beyond
the instruction quoted.

**Where the files go**: a directory outside the project, chosen by the owner. If the owner's
user-level agent configuration names one, use it. Otherwise ask, propose a directory under the
owner's home, and offer to record the answer in that user-level configuration, on the owner's word,
so later sessions find it. Never inside the project: committed, the files would enter its history;
ignored, they would be lost to the next clean.

## What becomes of each file `##what-becomes-of-files`

Give the owner the full path of each file, and never print either file into the conversation: the
owner reads both files at those paths, verbatim, and may edit them. Then, **on the owner's word
only, and where the owner directs**:

- **The project's file**: it stays where the files went until the owner hands it to an intake,
  usually in a later session and with other files, under
  `skill@knowledge-architect-retrospective-intake`, which puts each finding to the owner with its
  evidence and a default outcome.
- **The workflow's file**: it becomes an issue on the repository of knowledge-architect,
  <https://github.com/teoboutin/knowledge-architect>, opened with
  `gh issue create --repo teoboutin/knowledge-architect --title "<title>" --body-file <file>`.
  If `gh` is absent or fails, give the owner the path of the workflow's file, whose text is the
  issue's body, and the address of the repository's new-issue page.

Nothing leaves the machine without the owner having read it.

## What the installed skills expect of the owner `##expectation-sets`

An installed skill is built on assumptions about how the owner works. Each names what degrades when
it does not hold. They are not rules the owner is asked to follow; they bound what counts as a
defect of the skill (`skill@knowledge-architect-retrospective@what-it-examines`). A skill not listed here states none yet.

### `skill@knowledge-architect-design`

- **The owner brings a design question, or a requested change for the skill to ground, not a task
  order to execute unexamined.** The mode assumes the answer is not yet
  known by either party. Given a decision already made, it argues against it, because testing a
  proposal is what it is for. Given bounded work, it sends one proposal and builds its default; the
  owner's answer stops or redirects it, and an argument returns the work to the loop. An owner who
  neither answers nor argues leaves the default built.
- **The owner gives the word that closes a fork.** The agent closes a thread on its argument where
  one shape stands, and in the other cases the skill declares, and shows each closure for the owner
  to contest; a thread at a fork stays open until the owner's word. An owner who neither rules on a
  fork nor contests leaves the fork open, and the discussion cannot converge. There is no way to hand
  a fork to the agent: "your judgement" is an approval like any other, and a finding that defeats
  its premise still returns to the owner.
- **A closed thread is not frozen.** The owner's word reopens one at any time, and needs no new
  argument; the agent does.
- **The owner says why when rejecting something.** A ruled-out thread carries its reason into the
  record, and the reopening rule reads that reason to decide whether a later proposal is new. A
  rejection with no reason cannot do that work, so the same argument comes back.
- **The owner reads the summary tables and contests what is wrong in them.** The batch confirmation
  at a checkpoint closes everything in the table at once. Confirming without reading records
  decisions the owner did not make.
- **The owner corrects a stated misreading.** A message that could close a thread but does not
  clearly leaves it `presumed-settled`, with the agent's reading stated. Uncorrected, a wrong
  reading hardens into the record as though the owner had ruled.
- **The owner ends a discussion rather than dropping it.** "Stop, build X" and "park this" both end
  the argument at once. Abandoning silently leaves the criteria and the losing arguments unrecorded.
- **The owner is the person who can decide what is the owner's.** A fork, a goal, a contradiction
  no argument resolves, a fact only the owner holds and an action that cannot be undone route to the
  owner, per `primer@question-authority`. If the real authority is not in the conversation, the discussion converges on a verdict nobody present can
  act on.
- **A discussion runs in one session, and its memory does not outlive it.** The ledger lives in the
  conversation and in the harness's transcript of it, and what survives is what
  `skill@knowledge-architect-planning` assembled from it into the plan document, and then the
  records harvested from it. A previous discussion is not resumed in a new session: a new session
  starts a new discussion, grounded on the record. A resumed session with its full transcript
  restored is the same session.
- **The owner is trying to converge**: arguing, ruling, or saying stop. Several rules are released
  only by the owner's word, and withholding it leaves the discussion parked rather than producing a
  wrong result.
- **The owner accepts a methodology, not a script for every exchange.**
- **The project's design intent is discoverable, or the project is new.** The skill grounds itself in
  the goals, the design homes, the rejected designs, the README files, the open issues and the
  tripwires, or failing
  those in the code and its history. Intent that exists only in someone's memory is not reachable,
  and proposals will contradict decisions already made without either party noticing.

### `skill@knowledge-architect-setup`

- **The owner is present to rule.** The setup proposes and the owner rules: the Components, the
  goals, the place of each existing document that no row of the knowledge table settles, and each
  choice that changes the project's build or a file it already has. A run without an owner, such as a trial, makes those choices itself, the
  goals excepted, as `skill@knowledge-architect-setup` says, and a gap it reports in a choice it
  had to make alone is not a defect of the skill.
