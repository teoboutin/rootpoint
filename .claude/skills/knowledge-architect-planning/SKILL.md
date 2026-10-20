---
name: knowledge-architect-planning
description: MUST use when a design discussion has converged and its work needs a spec or a milestone, in the same session as the convergence; whenever a slice of a milestone is about to be implemented or lands; when the work of a spec starts or lands; and before editing the roadmap. Covers the plan document vocabulary (spec, milestone, plans directory, roadmap), choosing between a spec and a milestone, the roadmap, the file layout and the fixed sections, plan items and their citations, assembly from the discussion's transcript, cutting the work into steps and slices, acceptance criteria, the design audit, the reviews of a plan document, the harvest, and the deletion that ends a plan document's life.
---

# Planning

Scope: turning the converged design of a discussion into a plan document that a later session
implements, then carrying that document through each landing until it leaves the repository. It
begins in the session where the discussion converged, and it re-enters at each slice's audit and at
each landing.

**Not covered here**, each named where it lives:

- **The design discussion** that produces the converged design:
  `skill@knowledge-architect-design`. On its full path, it ends at convergence, the premortem and
  the owner's rulings on its tripwires and acceptance criteria, and hands off to this skill; its
  in-change path writes no plan document.
- **Writing the code** of a step, its claims and its tests: the project's own development
  procedure. This workflow installs none.
- **Dispatching the reviewers**: `skill@knowledge-architect-review`.
- **Auditing one aspect of the whole project**, which the design audit of a slice or a spec is
  not: `skill@knowledge-architect-project-audit`.
- **Recording** what a landing establishes. The procedure is
  `skill@knowledge-architect-decision-recording` for the decisions and the rejected designs, and
  `skill@knowledge-architect-issue-tracking` for the tripwires and the issues.

## Terms `##planning-terms`

| word | meaning |
| --- | --- |
| **plan document** | a spec, a milestone document, or the spec of a slice. Temporary: it leaves when its work lands |
| **plans directory** | docs/plans/ at the project's root, the one directory where a project keeps its plan documents. The checker fixes the path (`skill@knowledge-architect-planning@plans-layout`) |
| **spec** | the plan document of work done in one branch and one PR: the converged design and a concise implementation sequence. Cited `spec@plans@<id>` |
| **step** | one item of an implementation sequence, in a spec or in a slice |
| **slice** | a part of a milestone that is one branch and one PR, with its own spec, cited `spec@<milestone>@<slice>` |
| **milestone** | work across several PRs, with design sessions between them. Its plan documents are one directory: the **milestone document**, its `README.md`, and one spec per slice. Cited `milestone@plans@<id>` |
| **item** | a thread, an argument, a criterion or an acceptance criterion, defined in a plan document as a level-three heading ending with its slug, under the section of its kind (`skill@knowledge-architect-planning@spec-contents`) |
| **roadmap** | docs/roadmap.md at the project's root, optional: the order in which the owner wants known work done (`skill@knowledge-architect-planning@spec-or-milestone`) |
| **transcript** | the harness's log of a session, on disk. It keeps the records from before a compaction |
| **assembly** | writing a plan document from the discussion's transcript (`skill@knowledge-architect-planning@spec-contents`) |
| **converged** | convergence holds, as loop step 6 of `skill@knowledge-architect-design@discussion-loop` defines it |
| **thread** | one proposal of the discussion, carrying a state; a thread closed `approved` or `argued` is a decision |
| **criterion** | what proposals were judged against: **binding** rules a proposal out, **weighed** makes failing it a cost the owner rules on |
| **acceptance criterion** | a check on a decision of the plan, a thread closed `approved` or `argued`, that only the work's built code can apply |
| **material** | a finding made after a thread closed that defeats a reason the closure gave, a premise it rested on, or a criterion it claimed to meet. A material finding is presented once, with a default named; presenting it reopens the thread, which closes again on its argument or, at a fork, on the owner's word, per `skill@knowledge-architect-design@decision-authority` |
| **audit** | the reading of a slice's entry, or of a spec, against the tree and the design homes before its work is implemented |
| **harvest** | the recording of what a landing established into the project's durable homes |

**The owner** is the person whose word closes a fork, and every question `primer@question-authority` reserves to the owner.
The word "plan" alone never names a document: say spec, milestone document, or plan document.

## When this skill starts `##when-planning-starts`

**In the session where the discussion converged, before that session ends.** The discussion's
ledger, every thread with its state and its argument, lives in that conversation and in the
harness's transcript of it, which the assembly of `skill@knowledge-architect-planning@spec-contents` reads. A plan document written from memory in a
later session is written from a summary, and a summary loses exactly the losing arguments and the
conditions of each closure.

## A spec or a milestone `##spec-or-milestone`

| the work | its plan document |
| --- | --- |
| one branch, one PR | a spec |
| several PRs, with design sessions between them | a milestone |

Propose which one, with the reason, and let the owner rule. A spec whose work turns out to need
several PRs becomes a milestone: at that moment the spec is split into the milestone document and
one spec per slice, by the rule of `skill@knowledge-architect-planning@spec-contents`.

**Work that is known but not designed is not a plan document.** It is a `todo` issue, or a
`deferred` one if an event gates it (the default kinds; a project that declares its own kinds uses
the nearest), in the owning anchor's issue register, with its leads in the
entry. The plan document that schedules it closes that issue in the commit that adds the plan
document, so the work is listed in one place at a time. `cargo klarch issues --kind todo` lists
what is known and undesigned. There is no record of landed work: history lists what landed.

**The roadmap orders known work, and holds nothing else.** docs/roadmap.md at the project's root
is optional. Each row cites an issue entry, `issue@<anchor>@<id>`, or a whole plan document,
`spec@plans@<id>` or `milestone@plans@<id>`, in the order the owner wants the work done. An
unordered section may follow. **The order is the owner's**: a row is added or moved on the owner's
word. Three edits need no word, because the row keeps pointing at the same work, or that work is
done:

- the commit that adds a plan document closes the issue it schedules, and rewrites that issue's row
  to cite the plan document;
- the commit that deletes a plan document removes its row: the work landed;
- the commit that closes an issue with no plan document scheduling it removes its row: the work is
  done.

A row whose entry left without its edit above dangles, and `cargo klarch check` reports it, so the order
cannot go stale unnoticed. An illustration of the shape:

```markdown
# Roadmap

## In order

1. `milestone@plans@<id>`
2. `issue@<anchor>@<id>`

## Unordered

- `issue@<anchor>@<id>`
```

## Layout `##plans-layout`

```
docs/plans/
├─ README.md               what the directory holds
├─ specs/
│  ├─ README.md
│  ├─ index.md             generated
│  └─ <id>.md              a spec
└─ milestones/
   ├─ README.md
   ├─ index.md             generated
   └─ <id>/                a milestone
      ├─ README.md         the milestone document
      ├─ index.md          generated: the slice specs
      └─ <slice>.md        the spec of one slice
```

- The checker constructs the anchor `plans` at docs/plans/, one anchor per milestone directory and
  one per spec file. A file or a directory directly under docs/plans/ outside this layout is a
  finding, and so is a directory under milestones/ with no `README.md`. `cargo klarch index` writes
  every `index.md`, and `cargo klarch check --fix` writes them once the first three phases of the
  check find nothing, then runs every check. For a commit of part of the working tree,
  `cargo klarch index --staged` writes them into git's index from what is staged, and
  `cargo klarch check --staged` checks what the commit will record.
- The milestone document links each slice's spec as a navigation row, `[<slice title>](<slice>.md)`.
  The checker resolves a relative link only in a `README.md` or an `index.md`.
- **The plans directory holds plan documents and nothing else**, except its `README.md` files and
  generated indexes, which keep its homes in the tree while no plan is open. A document with
  another lifetime, such as a record of how far a subject is implemented or a survey that outlives
  its work, has its own home. If none fits, ask the owner before writing it anywhere.
- **A plan document is cited by its kind**, never by its path: `spec@plans@<id>`,
  `milestone@plans@<id>`, `spec@<milestone>@<slice>`. A `path` citation of one is refused. A whole
  plan document may be cited from anywhere, and its citations dangle when it leaves (`skill@knowledge-architect-planning@plan-document-leaves`).
- **A plan document defines items, and no design entry** (`skill@knowledge-architect-planning@spec-contents`). A plan's name is the anchor of its
  items, so it is not the name of a Component, of a location or of a reserved anchor, `plans`,
  `elsewhere` or `*`, and one name is not used under both homes.

## What a spec holds `##spec-contents`

**Written for a session that did not witness the discussion.** That is the standard every section
is held to, and the reviews of `skill@knowledge-architect-planning@plan-reviews` check it.

**The sections of a spec of specs/ and of a milestone document are fixed: level-two headings with
these titles, in this order.** A slice's spec owes the five sections of its entry instead, and
carries of the sections below only those its slice fills. `cargo klarch check` reports a section a
document owes as missing or out of order. A section a document owes and has nothing to hold says
so in one line rather than being omitted, so a reader can tell an empty section from a missing one.

| section | holds |
| --- | --- |
| Status and audience | what the document is for; that it leaves when its work lands; that where it and a design home disagree on a point its section "What is already decided" does not list as reversed or rewritten, the design home wins; that every name it uses is defined in it or exists in the code; that where the owner's word is awaited, `skill@knowledge-architect-planning@working-a-slice` says what waits |
| How the work is done | in a milestone document: a pointer to `skill@knowledge-architect-planning@working-a-slice`, with an instruction to read it whole before each slice, and any addition the milestone's work needs. In a spec: one line naming this skill |
| Names | every project shorthand the document uses, expanded to the file, function or command it names |
| What the work is | what exists today at each site the work touches; what is outside the work and why, each exclusion naming the work or the decision that owns it |
| What is already decided | the recorded decisions the design rests on and does not argue again, as references; and each recorded decision the work reverses or rewrites, with every text that `cargo klarch show` lists as referencing it (a tripwire, an issue, a restatement in a `CLAUDE.md` or a skill, a README, a comment), and the step, slice or harvest that judges or updates each; and each standing entry the discussion's search returned, and each design or `rejected` entry its design-entry search graded `bears`, with its outcome: how it bears on the work, or that it bears on nothing |
| Criteria | one item per criterion, ``### <criterion> `##<id>` ``: its kind, its source and its satisfaction |
| Threads | one item per thread, ``### <resolution> `##<id>` ``: who proposed it and in which round, its final state, the arguments that moved it, the section that carries its shape, the durable home that will harvest it, and the owner's words that closed it, verbatim, with their round, or, for a thread closed `argued`, the argument that closed it |
| Arguments | one item per argument of the discussion, ``### <argument> `##arg-<n>` ``: its round, who gave it, the threads it bears on, and its key words verbatim |
| New names, in one place | every new name the design uses (a type, a function, a field, an event, a bound, a counter) in one fenced block with the file it goes in; a name that exists in the code is listed as existing |
| Decided design | one subsection per thread closed `approved` or `argued`: the shape, the argument, the nearest rival and the fact that defeated it |
| Mapping tables | one table per total function the code will need, over its whole domain: which existing thing becomes which new thing. Empty when the work needs none |
| Losing alternatives | every ruled-out thread, every thread withdrawn with its defeating reason, and every superseded thread under the thread that absorbed it, each with the thread it lost to and the fact that decided it |
| Readings | where the work reads an external specification the project implements: each reading it makes, and where it is recorded. Empty for work that reads none |
| Premortem | each cause, the thread it stresses, and its verdict: survives into a named claim, criterion or guard; converted into a named clause of the design; becomes a tripwire or an acceptance criterion, on the owner's word; or fired and the thread reopened. Each tripwire and acceptance criterion carries the label it was put to the owner under, `T<n>` or `AC<n>` |
| Acceptance criteria | one item per criterion, ``### <criterion> `##<id>` ``, as `skill@knowledge-architect-planning@acceptance-criteria` says |
| Implementation sequence | in a spec, its steps; in a milestone document, its slices, each linked to its spec, whose own steps it holds (`skill@knowledge-architect-planning@cutting-steps-and-slices`). Concise: what each builds and what it fails alone on, and, in a spec, each step's Claims, as a slice's entry states them |
| Order rationale | one sentence per pair of adjacent steps or slices |
| Defaults awaiting the owner | each default a reviewer's finding or the author's judgement produced that `primer@question-authority` gives the owner, labelled `D<n>`, with the thread it bears on, marked as ruled once the owner rules; and the label of each acceptance criterion awaiting the owner's word |
| Harvest | what lands where and when. In a spec, its rows; in a milestone, each slice's row in that slice's spec, and the row of the milestone document itself in it |
| Later consequences | what each later piece of work adds or replaces, so a later reader knows what was deliberately left |

**A milestone's design is split across its documents from the start, by lifetime.** The milestone
document and one spec per slice are written together, in the session that converged. A milestone
document that held every slice's design would be read whole at every slice's grounding, and grows
with every slice the milestone has.

- **A slice's spec leaves when its slice lands, so it holds what only that slice builds**: the slice's
  entry (`skill@knowledge-architect-planning@cutting-steps-and-slices`) and its steps, then, in the sections above, the items of its own threads and the arguments only
  they use, its decided design, its mapping tables, its losing alternatives, the acceptance
  criteria it judges and its harvest row. These sections are not ordered against the entry's.
- **The milestone document holds what crosses slices or outlives one**: the status, how the work is
  done, the names, what the work is, the criteria, every item that more than one slice's
  document cites, the premortem, the implementation sequence with each slice linked to its spec,
  the order rationale, the defaults awaiting the owner, the harvest row of the document itself and
  the later consequences. A section whose content lives in the slice specs says so in one line.
- **An item cited from the milestone document, or from more than one slice's spec, lives in the
  milestone document.** A slice's spec that held it would dangle those citations when it leaves; the
  check reports any that remain at that deletion.
- A design session held at a slice's audit writes its design into that slice's spec, and what it
  decides for later slices into theirs.

**Content rules.**

- **No untested code snippet is presented as authority.** An implementer forces an authoritative
  snippet into the code at any cost and copies its comments verbatim. A snippet in a plan document
  is labelled as an illustration of a shape. A detailed step-by-step implementation plan, if one is
  ever written for a less capable implementer, covers a bounded amount of work and opens by saying
  that its content rests on assumptions and may be wrong.
- **A statement about existing code is a claim, not a hypothesis.** It is checked against the tree
  before it is written, as `primer@verify-before-relying` asks, in its subsection "Verify a claim
  before writing it". Only the unbuilt half of a plan document is a
  bet.
- **A correction is applied in place**, with no correction log and no issue: the document leaves
  when its work lands, so a log would be history in a file that keeps none. The commit says what
  was corrected.

**Items.** A thread, an argument, a criterion and an acceptance criterion are items: a
level-three heading under the section of its kind, the statement first and the slug last,
``### <statement> `##<id>` ``. The section gives the kind: Threads `thread`, Arguments `argument`,
Criteria `criterion`, Acceptance criteria `acceptance`. Every level-three heading of those four
sections is an item, and a slug anywhere else in a plan document defines nothing.

- An id is in the grammar `[a-z0-9]+(-[a-z0-9]+)+`. A thread keeps the slug the discussion minted.
  An argument is numbered `arg-1`, `arg-2`, …, in order of appearance and never reused, in one sequence
  across a milestone's README and its slice specs, which share one namespace.
- An item is cited `<kind>@<plan>@<id>`, where the plan is a spec's id, or the milestone's name for
  an item of the milestone document or of any of its slice specs, and only from inside its own plan: the spec file, or the milestone's directory. From anywhere
  else, a commit message and a design head included, the plan is cited whole, and an item is named
  in plain text with a hash sign, as #<id>. A slice is named the same way.

**A path the plan's work will create** is cited `planned@<anchor>@<path>`, under the rules of a
`path` reference: the anchor that will hold it, a trailing `/` for a directory. The form is legal
in the plans directory only. `cargo klarch check` reports it once the target exists, and the commit
that creates the file converts it to `path@<anchor>@<path>` in the same change.

**Assembly from the transcript.** A plan document records the whole discussion: every thread with
its proposer, its final state, the arguments on each side, the owner's rulings verbatim with their
round, and its relations. An answer of the owner on a reading or a fact is recorded with its source,
as `primer@labelled-items` says. It is assembled from the transcript, not from memory. Dispatch a
subagent that reads every transcript file the discussion spans, a resumed session included, and
extracts the delta tables, the owner's messages verbatim and the arguments; the design skill's
per-round delta is the draft it reads. Its brief names
`agent@knowledge-architect-transcript-reviewer@extract-transcripts`, read whole, for which lines
it keeps and which it labels as the owner's: the transcript reviewer then checks the document
against the same lines. Where one argument ends and the next begins is decided at
assembly, and the transcript reviewer of `skill@knowledge-architect-planning@plan-reviews` checks that no argument was lost. The status section quotes, for
each session the discussion spans, the message where the discussion begins in it, so that a
reviewer reads the same transcripts: each file is found by that message, never by a session
identifier, as `skill@knowledge-architect-review` says, and the document names no file of the
transcript directory, whose path holds the session identifier and the machine's home directory. Where the harness keeps no
transcript, assemble from the conversation, and say so, and why, in the commit that adds the
document.

## Cutting the steps and the slices `##cutting-steps-and-slices`

The tests below hold for the steps of a spec and of a slice, and for the slices of a milestone,
unless one names the slice alone.

**Each step must be able to fail without another step's new machinery entangled in the
failure.** Every other test here serves that one.

- **The first step lands the mechanism empty and measures it.** No content that uses it, no
  observable change against the instruments the Component already has, and its cost measured against a recorded baseline where the Component has one. Every later failure is
  then a failure of the content and not of the mechanism.
- **Order by dependency.** A step that needs a mechanism comes after the step that proves it.
- **A step touches at most one subsystem it did not build.** Two subsystems in one step are two
  failure surfaces with one name.
- **The last step consumes**: the measurements, the report, the harvest.
- **A step whose trees cannot pass the tip's checker alone** lands in the commit of the step that
  makes them pass, per `primer@branch-commits`. The tests above still decide where the steps are
  cut; only their commits are joined.
- **One branch and one merge per slice**, each merged before the next slice begins, and reviewed
  before the merge. A step is no unit of the history: how many commits it takes, and what other
  commits the branch carries, such as the commit of its own a fix met outside the task takes, is the
  implementing session's to judge.

A slice's entry is five level-two sections, with these titles, in this order, and
`cargo klarch check` reports one missing or out of order. Fixtures, where it applies, sits between
Claims and Audit subjects, and is not checked:

- **Builds**: what the slice lands, by name.
- **Claims**: each with the test that could refute it, and how the test is shown to fail against a
  wrong implementation. A claim whose instrument does not exist names the instrument the slice
  lands first.
- **Fixtures**, where the Component drives its tests with authored content: each written in the
  vocabulary that exists, or naming the addition it makes. A fixture that needs something the
  vocabulary cannot express is a finding against the design, not a note for the implementer.
- **Audit subjects**: the files, decisions and specification passages the slice's audit is already
  known to have to read.
- **Fails alone on**: the one or two observations that tell this slice's failure from every other
  slice's.
- **Premises that expire**: where the slice relies on something a later slice or change ends, what
  ends it, and the criterion or guard that watches it.

## Acceptance criteria `##acceptance-criteria`

An acceptance criterion is a bet about a decision the work is built on, where a claim is a bet about
the work itself: firing a criterion reopens the decision, not the step. It lives in the plan
document of the work that can judge it, in its acceptance criteria section, and in no file of its
own. One of the in-change path, which has no plan document, is
`skill@knowledge-architect-design@discussion-loop` step 7's. Each acceptance criterion of a plan
document names:

- the decision it guards, as a reference to its thread;
- the step or the slice that judges it;
- the observable that fires it;
- the response.

**A check proposed before the work has a plan document is a lead, not a criterion.** It stays in
the work's `todo` or `deferred` issue, among its leads, and the design session judges it: it
becomes a criterion of the plan document, or it is dropped. It fires nothing while it is a lead.
Evidence that bears on it before the design session is an issue entry of its own, an
`observation` or a `defect` by what it establishes, which the work's issue names.

**The result a scheduled review is expected to give is not a criterion.** Passing the reviews that
every plan document and every slice owes is the baseline. A criterion names an observable specific to
the decision it guards; listing "the reviews pass" in every plan document would stand in for the
specific criterion that is harder to find.

**An acceptance criterion stands on the owner's word**, as a tripwire does. The owner rules on
whether it is applied and on the decision its firing reopens; its observable is worded by the agent,
and a rewording is listed in the turn's "Decided" section, per `primer@decided-section`. It is put to the owner under a label,
`AC<n>`, numbered as `primer@labelled-items` says, so the owner's ruling in the transcript finds it.
A criterion first proposed after the premortem, at the assembly, by a review or at an audit, is
written as an item of the acceptance criteria section marked as awaiting the owner, its label at the
head of the item's text, and its label is listed under the defaults awaiting the owner. It is not
judged before the ruling, and the work goes on without it; the branch that carries it merges once
the owner has ruled on it, per `primer@question-authority`. One the owner declines is deleted with
the document. A criterion that gains its slug keeps its label beside it, in
the item's text.

A number in a criterion is a threshold the owner sets. Until the owner has, it is written as a
default marked as the owner's to reset.

- **At each landing**, the messages of the commits that land it report on every criterion judged
  there, one line each, naming it as #<id> beside a citation of the whole plan, since a commit
  message cites a plan only whole: the decision guarded, fired or not, the evidence, the response
  taken.
- **A criterion that fires** leaves the document at once, as an issue entry or a reopened decision,
  under `skill@knowledge-architect-issue-tracking`. A criterion is judged by its observable,
  wherever the observation was made: one that a later review or test shows firing after a landing
  reported it as not fired is reported as fired in the next landing commit, which names the earlier
  report as superseded.
- **When the document leaves**, the landing that deletes it reports on every criterion once more.
  One that did not fire and recurs at later work is proposed to the owner as a tripwire, and written
  on the owner's word as `skill@knowledge-architect-decision-recording@ruled-tripwires` says, or,
  where an entry already guards the same decision, folded into it as an absorbed tripwire is. One
  that is spent, or that the owner declines, is deleted with the document.

## Working a slice, and the work of a spec `##working-a-slice`

The work of a spec follows this procedure too, as its last paragraph says.

**Where the owner's word is awaited.** A default the plan documents list as awaiting the owner is
built as they write it until the owner rules on it, and its branch merges once the owner has, per
`primer@question-authority`. A scope change listed there does not start
before its ruling. A load-bearing gap of point 2 is not such a default, even where the message
that puts it to the owner names one: it stops the slice, the whole of which rests on the decided
design the gap defeats.

1. **Ground**: the Component's `CLAUDE.md`, its design home, its `rejected` home; then the
   milestone document entire, then the slice's spec. The issues and the tripwires, of every anchor,
   are read at the audit.
2. **The design audit.** Read the slice's entry and every decided shape it depends on against the
   code as it stands and against the design homes. List every gap: a shape the code refutes, a
   passage of a specification the entry did not read, a name the entry uses that the code does not
   have, a consequence the entry did not see, and **a standing entry the slice's planned code bears on**:
   a tripwire whose firing condition, or a `deferred` issue whose trigger, the planned code meets, a
   tripwire of the milestone document's premortem included, and an issue of any kind the slice's code touches, closes, makes worse or depends on. Dispatch the
   search as the description of `agent@knowledge-architect-standing-entry-searcher` says, with the
   slice's spec and the milestone document as the work and, as seeds, the decisions the milestone
   document lists under "What is already decided" and the decisions and goals the slice's spec
   cites; for a spec, the spec is the work and its own "What is already decided" gives the seeds.
   Read whole, with `cargo klarch show`, every entry the search returns, and judge it against the
   slice; never act on an entry from the reason the search gave. The audit's record gives one line
   to each entry the search returned, with its outcome, an entry judged to bear on nothing in the
   slice included: in the message of the audit's commit, or, where the audit applies nothing in
   place, in the message of the slice's first commit. A firing found at the audit is
   ruled before the code is written, where one found only by the review of the harvest reopens the
   harvest. Sort each gap:
   - **Applied in place.** The gap has one answer the document's decisions already imply, or is a
     choice among shapes the document rules out all but one of. Apply the answer in place, in the
     milestone document or the slice's spec, wherever the shape it changes is written. Commit the
     amended documents alone, with a subject that names the slice's design audit in those words,
     and a message listing each finding: the gap, the answer and the decision it follows from, and
     the line of each returned entry. Where the slice's first change sets what the per-commit gate
     requires, per `primer@branch-commits`, the amended documents land in that change's commit
     instead, and its message carries the audit's subject and list. Earlier audits are found with
     `git log --grep='design audit'`.
     **An answer that widens or narrows a ruling of the owner, or adds an obligation to one, is a
     scope change even when it is the one answer the document implies.** It is applied with the
     others, and also listed in the milestone document as a default awaiting the owner.
   - **A change to the slices.** An audit that finds the work needs another slice, or a slice split
     or reordered, writes each new slice's spec with its harvest row, adds the slice to the
     implementation sequence and the order rationale, and moves into its spec the design it takes
     from other slices, so that each shape keeps one home. It changes a sequence the owner ruled on, so it is a scope
     change, listed and ruled as above.
   - **Load-bearing.** The gap is material, or is a choice between two shapes neither of which the
     document rules out, or needs a ruling the document marks as the owner's. Record it in the
     slice's spec as open at the audit, with the discriminating fact, stop the slice, and open a
     design session with the owner under `skill@knowledge-architect-design`.
     - A choice among shapes that can be stated in full, each with its consequence, is put to the
       owner in one message, with a default; several such gaps go in one question, each under a
       label, `Q<n>`, since each is written in place as a thread with its slug. The slice's
       documents exist, so the ruling is not left to a commit message as the in-change path would
       leave it: it is written in place in the
       milestone's documents by the rule of `skill@knowledge-architect-planning@spec-contents`, as a thread with the owner's words verbatim, like
       the audit's other answers, and the audit's commit lists it among its gaps. It is no
       converged design session, so it owes no new review of
       `skill@knowledge-architect-planning@plan-reviews`.
     - A gap that defeats a reason, a premise or a criterion a thread closed `approved` or `argued` rests on needs the
       full session. Its converged design goes into the milestone's documents by the rule of `skill@knowledge-architect-planning@spec-contents`,
       and owes the reviews of `skill@knowledge-architect-planning@plan-reviews`.

     The slice resumes from the ruling or the converged design.
3. **Claims, tests, implementation, gates, commits**, per the project's development procedure. The
   commits name each claim of the spec with its test and its outcome, how the test was shown to fail
   against a wrong implementation, and of any claim whose test cannot yet do so, why not. A claim
   that holds over every site of a corpus is tested by reading the corpus: a phrase search is a lead,
   which misses a phrase split over a line break and a site written in other words.
4. **Review of the work**, per `skill@knowledge-architect-review`. A repair lands as
   `skill@knowledge-architect-review@what-review-leaves` says.
5. **The report** on each acceptance criterion judged at this slice, as
   `skill@knowledge-architect-planning@acceptance-criteria` says.
6. **The harvest**, per the harvest row of the slice's spec: the decisions and the losing
   alternatives under `skill@knowledge-architect-decision-recording`, then the tripwires and the
   issues under `skill@knowledge-architect-issue-tracking`. Read `primer@design-heads` again, whole,
   before writing a head: it holds what earns one and how it is shaped, the thread's slug and its
   pair included. The row names what is judged; the entry tests and the tests for a losing
   alternative decide whether each decision and each alternative earns an entry, the rules on a head's shape
   decide how the decisions are cut into heads, and they govern where the two disagree: an item of the row the tests exclude is named in the harvest's commit,
   with the test it fails. Where a design home is a directory, a new subdocument is
   linked from its README. **The harvest's review is the review before the merge**, as
   `skill@knowledge-architect-review@review-axes` defines it and says what it owes: the harvest
   writes the record the decision-record, routing and standing-state axes judge, so the review of
   point 4 cannot see it.
7. **The slice's spec leaves** in the commit that completes its harvest, as in `skill@knowledge-architect-planning@plan-document-leaves`. What crosses slices stays in the
   milestone document, amended in place where the landing changed it.

**The work of a spec** takes the same points once, for the whole spec, on its one branch:

- point 1, the grounding;
- point 2, the design audit, only when the work does not start in the session where the discussion
  converged, or when commits other than the spec's own have landed on the main branch since the
  spec was written. Otherwise the session goes straight to the work, since the design was read
  against the tree as it stands;
- point 3, the claims and checks of each step;
- point 4, the review of the work, once;
- points 5 and 6, the report and the harvest, in the commits that land the work;
- point 7, the deletion of the spec, in the commit that completes its harvest, as in `skill@knowledge-architect-planning@plan-document-leaves`.

Where a point names the milestone document or the slice's spec, the work of a spec reads the spec:
its defaults, its threads, its harvest row, and the design audit its commit subject names.

## Reviews of a plan document `##plan-reviews`

A plan document is committed first, on a branch of its own or on its work's branch, and that
commit is what the reviewers read; a repair lands as
`skill@knowledge-architect-review@what-review-leaves` says. **It may be merged on its own,
whatever the time of its work**: a plan document on the main branch keeps the work done meanwhile
from drifting from it.
**One whose work's first commit changes what the project's per-commit gate checks is merged
before that work**, a spec as well as a milestone document: on one branch,
`primer@branch-commits` would put that commit before the document that plans it, or squash the
two into one. A change that only a check of the working tree sees does not count. It is read
again after its decided design changes, by a design session that converges or by a ruling of the
owner, as at a load-bearing gap of a design audit. A change made in answer to its review, a default written in included, is a
repair, reviewed again only as `skill@knowledge-architect-review@what-review-leaves` says, and an
audit applied in place is no revision. Its reviewers are **fresh, never a fork**: a fork inherits the discussion and reads the document as its
author. Dispatch them through `skill@knowledge-architect-review`, with the invariants that
skill lists, the blind brief included, on every axis of its table whose condition holds, among them
these four:

- `agent@knowledge-architect-cold-implementer-reviewer` reads the document as the implementer of its
  work, or of a milestone's first slice, and reports every place where it cannot act: undefined names,
  shapes without enough detail to write, procedure gaps, ambiguities, and what it would have to
  reconstruct from a conversation it did not see. It also applies the readiness checks below.
- `agent@knowledge-architect-code-claims-reviewer` verifies every statement the document makes about
  the code as it stands, and reports each as confirmed, wrong or imprecise, with the evidence.
- `agent@knowledge-architect-design-conformance-reviewer` reads the document against the project's
  record: the goals, the design heads and the rejected designs of every Component it touches.
  It reports a shape, an acceptance criterion, a default, a step or a harvest row that contradicts
  a goal, that contradicts or widens a head the document does not list as reversed or rewritten,
  or that brings back a rejected design. A conflict with a goal goes to the owner.
- `agent@knowledge-architect-transcript-reviewer` reads the discussion's transcripts and checks that
  everything the discussion established that must outlive it is in the document or has another
  durable outcome, and that no ruling of the owner is recorded wider, narrower or in another state
  than the owner gave it. Dispatch it on every assembled document, in the batch or alone and last
  as `skill@knowledge-architect-review@review-axes` says, and name in its brief the transcript
  files the assembly read, with the message where the discussion begins in each.

**What their findings become.** Check each finding against the tree, or against the transcript,
before acting on it. A material finding is answered with a default, written into the sections it
touches. **A default the record, a measurement or a source settles is the session's decision**: it
is written into the sections it touches with its ground, takes no label, is not listed under the
defaults awaiting the owner, and is reported in the turn's "Decided" section, per
`primer@decided-section`, where the owner can contest it. A
default that `primer@question-authority` gives the owner, a fork among them, is listed under the
defaults awaiting the owner, with the thread it bears on and a label, `D<n>`, numbered as
`primer@labelled-items` says. A ruled default stays in that list,
marked as ruled, with its label, since the sections it was written into cite it by its label. A
finding that is a gap with one answer is applied in place, except that **a gap whose one answer widens
or narrows a ruling of the owner, or adds an obligation to one, is a scope change**, as at the
design audit. A finding that is wrong is dropped, with
the reproduction that showed it wrong kept in the commit message. A scope change, a clause that
widens or narrows a ruling or adds an obligation to it, is put to the owner, listed under the
defaults awaiting the owner. Detail the author added
inside the scope of a ruling, and a wording better than the one the owner was shown, are no
findings.

**The readiness checks**, applied by the author before dispatch and by the cold implementer after.
This list is their one home; the reviewer reads it here.

- every name the document uses is defined in the names sections or exists in the code;
- every total function the code will need is a mapping table;
- every fixture is expressible in the vocabulary that exists, or names its addition;
- a milestone document points to `skill@knowledge-architect-planning@working-a-slice`;
- every number is a measurement with its instrument, or a default marked as the owner's;
- every premise a later step or change ends is stated with its guard;
- every statement about the code names the file and the function, and its truth is the code-claims
  reviewer's to establish;
- every acceptance criterion names the decision it guards, the step or slice judging it, the observable that
  fires it and the response;
- every thread maps to a section and to a harvest home;
- every text referencing a decision the work reverses or rewrites is named, with the step, slice or
  harvest that judges or updates it;
- every section the document owes by `skill@knowledge-architect-planning@spec-contents` is
  present, and an empty one says so.

## When a plan document leaves `##plan-document-leaves`

**A plan document is deleted in the commit that completes its last harvest, and that commit's message cites
it by its kind**, `spec@plans@<id>`, `milestone@plans@<id>` or `spec@<milestone>@<slice>`, which
resolves against the commit's parent. A milestone's slice spec leaves when its slice lands; the
milestone document leaves with the last slice. Before deleting:

- every harvest row of the document is done;
- every acceptance criterion has been reported on, and has become a tripwire or left (`skill@knowledge-architect-planning@acceptance-criteria`);
- what the document established is in the design homes and the registers; what stayed a guess
  leaves with it;
- each of its later consequences is in the issue entry or the plan document of the later work it
  names, or in a `todo` issue opened for it where that work has neither;
- every default the document lists as awaiting the owner is ruled, since a design head records such
  a default only once the owner has ruled on it;
- its row of the roadmap, if it has one, is removed (`skill@knowledge-architect-planning@spec-or-milestone`).

**A citation of the leaving document from another plan document dangles.** The session that meets
it, the one deleting the document or the one rebasing the citing document's branch onto that
deletion, does two things in one commit:

- it removes the citation from the citing document;
- it opens a `question` issue in the root's issue register, under
  `skill@knowledge-architect-issue-tracking`: does the citing document still hold now that the
  leaving document's work is built, accounting for deviations or other unplanned happenings? It is
  answered by reading the citing document against what the leaving document harvested. The
  issue cites the citing document, so it cannot outlive it, and its `Why it matters` cites the
  leaving document's harvested design entries, since the leaving document no longer exists. For a
  milestone, its next slice's audit reads the issue.

A milestone's own documents are one plan: where a slice spec that leaves is cited by its milestone
document or its sibling specs, they re-point or remove the citation with no
`question` issue, since the next slice's audit already reads the milestone against what landed.

Any other citation of the leaving document, from an issue, a design head or a `CLAUDE.md`, dangles
too, and `cargo klarch check` reports it: the deleting commit repairs each, as
`skill@knowledge-architect-issue-tracking` says of a sentence whose reference dangles, and an issue whose
subject was the leaving document closes with it.

A reader who needs the deliberation later finds it in history:

```sh
git log --diff-filter=D --name-only -- docs/plans/         # every deleted plan document, with its commit
git show <commit>^:<path of the document>                   # the document as it stood before deletion
```
