---
name: knowledge-architect-project-audit
description: MUST use when the owner asks to audit one aspect of the whole project rather than the work of one branch; when a pin move crosses a version whose changelog has a Migration entry citing an axis of this skill; and when an adopting project's existing documents have been moved into the workflow's homes. Covers what a project audit is, its method (an axis's saved workflow, whose one run per pass delivers a report the session verifies; then one list for the owner, the application, the re-check, the review), the outcome of each kind of finding, where the record of an audit goes, and one section per axis: the design-record axis, which re-applies the rules on design heads and rejected designs to every entry, and the agentic-workflow axis, which reads every instruction the harness delivers as one whole for instructions a session cannot follow and statements no act needs.
---

# Auditing a project

Scope: reading one aspect of a whole project, every entry or text of a corpus, against the rules
that apply to it, and bringing what fails into line. A project audit is the reading of the whole
record on one axis; a review reads a diff. One activity: its artifact is a repaired record across the
project, not a verdict on one branch.

**Not covered here**, each named where it lives:

- **A review of a diff**, the work of one branch or one plan document:
  `skill@knowledge-architect-review`.
- **The design audit** of a slice or a spec before its work is built:
  `skill@knowledge-architect-planning`.
- **A retrospective's findings**: `skill@knowledge-architect-retrospective` writes them, and
  `skill@knowledge-architect-retrospective-intake` analyses a project's own.
- **Recording** what an audit changes: `skill@knowledge-architect-decision-recording` for a design
  head and a rejected design, `skill@knowledge-architect-issue-tracking` for an issue and a
  tripwire, and `skill@knowledge-architect-goal-setting` for a conflict with a goal. Read the
  first before any write into a design home.

## Terms `##audit-terms`

| word | meaning |
| --- | --- |
| **axis** | one aspect an audit reads, with its corpus, its rules, its agent and the outcome of each kind of finding. Each axis has a section below |
| **corpus** | the entries or the texts an axis reads, all of them |
| **dispatch** | how an axis shares its corpus out among agents, how many, what each brief names, and how they are calibrated, if they are. Each axis's saved workflow holds its own |
| **draft** | one agent's file for one entry or for one finding, as the axis says: the verdict or the finding, the evidence, a proposed edit and a proposed outcome |
| **the owner list** | every finding of one pass that needs the owner's word, in the pass's report file, and the one message that points the owner to it |
| **the re-check** | fresh agents of the axis, dispatched as its first dispatch was, after the edits |
| **pass** | one round of the method. One run of the axis's saved workflow, the session's verification of its report, the owner list and the application, with the judgement of the application where the axis's section asks one. The re-check starts the next pass |
| **run** | one invocation of an axis's saved workflow |
| **audit** | the passes on one axis, from the first until the owner stops them |

## When an audit runs `##when-an-audit-runs`

- **The owner asks for one**, naming the axis.
- **A pin move crosses a version whose changelog has a Migration entry citing an axis's section of
  this skill.** The axis runs in a branch of its own started from the main branch once the pin
  move has merged into it, as `skill@knowledge-architect-setup@moving-the-pin` says. A version
  that changes the rules an axis applies names the axis in its Migration entry, so a project
  brings its existing record to the new rules.
- **An adopting project's existing documents have moved** into the workflow's homes, as
  `skill@knowledge-architect-setup@existing-documentation` says: the move's work ends by
  running the design-record axis over its whole corpus.

An audit runs in a branch of its own, unless the owner gives it to another work's branch.

## The method `##audit-method`

An axis's section adjusts each point where its corpus needs.

Each axis runs each pass as **one run of its saved workflow**, named in its section, with the
`args` its section lists, and one more run on the application where its section says so. The run dispatches every agent of the pass, verifies their drafts and
delivers a report; the session reads nothing between its steps. The session then verifies the
report as the axis's section says, and nothing an agent proposes is applied before a reading
confirms it. Before a run, the session writes the kept list, for a re-check, and names the
history. A harness may list saved workflows only when a session starts, so a session that ran the
install later may not see the workflow listed; a session whose harness does not list the workflow
dispatches its steps by hand, in the script's order, with the script's prompts, read from the
script. The run's worktrees are removed once the pass's findings have their outcomes, as
`skill@knowledge-architect-review@review-invariants` says.

1. **Pre-commitments.** An axis that measures, one whose audit yields a figure a conclusion rests
   on, states before the audit which figure would show which conclusion, and the owner sees them
   before the audit. An axis that only applies rules owes none, and the count of violations left
   per pass, which point 4 reads, makes no axis one that measures.
2. **The owner list.** The items of one pass, in the pass's report file of
   `skill@knowledge-architect-project-audit@record-of-an-audit`, grouped by kind, each under a
   label, `F<n>`, numbered as `primer@labelled-items` says with that file as its record, with a
   default and the words of the entry or the text the item turns on, and ending with a `Default`
   line and an `Outcome` line. One message gives the owner the file's full path and one line per
   item, and proposes that the owner rules by writing each `Outcome` line; a ruling given in the
   conversation is copied into the file verbatim. Apply nothing on the list before the owner
   answers.
3. **Application.** Apply the edits, yourself or through agents dispatched on disjoint files. Read
   the whole diff before each commit.
4. **The re-check.** After the edits, run the axis's saved workflow again, given the entries, the
   texts and the rules the owner ruled to keep, as a file, and the commit the previous pass
   audited. A finding of the re-check that a
   reading confirms, that is not on the kept list, and that no issue entry records, is a violation
   left, whether a draft missed it or an edit made it: repair it or put it to the owner. The
   re-check gets its findings as the axis's section says. The audit stops when
   the owner judges what is left small enough. The violations left then take their outcomes, as
   any pass's findings do, and no re-check follows; one the owner does not rule on becomes an issue
   entry, as a deferred item does. **Each violation left is on old text or on new text**: old text
   is text the previous pass also read, whose words no commit changed after the commit that pass
   audited, a move, a reflow or a rename leaving it old; new text is text a later commit wrote, the
   previous pass's repairs among them. The axis's section says how its findings are classified. The
   count on old text is expected to fall from one pass to the next; a pass where it does not goes to
   the owner as a question, with the counts, and starts no new pass on its own. A violation left on
   new text takes its outcome like any other, and decides nothing about convergence.
5. **Review.** Once the audit stops, the audit's branch is reviewed before its merge under
   `skill@knowledge-architect-review`, on the axes `skill@knowledge-architect-review@review-axes`
   says that review owes. Its briefs name each report file the audit's branch added and deleted,
   with the commit that holds it, since a reviewer of the branch's tip does not see it.

## Outcomes `##finding-outcomes`

Every finding takes exactly one outcome.

| outcome | the findings |
| --- | --- |
| applied in the branch | the edits the axis's section lists as applied |
| put to the owner, on the owner list | the edits the axis's section lists as the owner's; a verdict where a rule reads two ways and no record states the reading, per `primer@question-authority`; a conflict with a goal, which then follows `skill@knowledge-architect-goal-setting`; any finding no row names |
| sent upstream | a finding on the installed text that the row "nothing" does not name; the project never edits the installed text: the axis's section says how |
| an issue entry | a finding outside the axis that the primer's table routes to an issue, such as a statement false of the code when the axis does not read the code; work the owner approved that is too large for the branch; a finding the owner defers. The kind follows `skill@knowledge-architect-issue-tracking` |
| nothing | the entry or the text conforms; a finding an issue entry already records; a finding on the kept list |

A finding outside the axis takes the first case of `primer@met-outside-the-task` that applies.

## The record of an audit `##record-of-an-audit`

**Each pass's report file holds its evidence and the owner's rulings, and the pass's commit
message holds its counts.** The file is `docs/audit-reports/<YYYY-MM-DD>-<axis>-pass-<n>.md` at
the project's root, the axis named by its section's slug. The session writes it after its reading
of the pass: the run's report as the session's verification left it. The owner list closes it, as
point 2 says. A span the checker would refuse is quoted in words, since the file is checked like
any other, and an entry the pass removes, renames or creates is named as
`primer@where-knowledge-goes` says. The file and every commit
message of the audit name a commit of the audit's branch by its subject, never by its SHA, since a
rebase merge gives the branch's commits new SHAs. The file is committed once every `Outcome` line
holds the owner's ruling, or, at the audit's stop, the issue entry point 4 gives an item the owner
does not rule on, before any edit of the pass is applied; the commit that carries out its last
outcome, or a later commit of the same branch, deletes it, and its message names it by its path.

The pass's commit is the one that applies its last edit, or the one that adds its report file
where the pass applies none. Its message says what was audited, at which commit, how it was
dispatched, the calibration reading settled where the axis has one, the outcome of each finding in
a line, and the path of the pass's report file. A figure the audit measured goes there too, and the
count of violations left on old text and on new text, as point 4 defines them, and of those on old
text that went on the owner list, so that the passes show whether the audit converges. It counts
too how many of the owner's rulings on the owner list went against the default.

## The design-record axis `##design-record-axis`

**The corpus**: every design head of every Component, and every entry of its `rejected`
home.

**The rules**: every rule of `primer@design-heads` for a head, the questions of
`skill@knowledge-architect-decision-recording@owning-component` for the Component it sits in, and
every rule of `skill@knowledge-architect-decision-recording@losing-alternatives` for a `rejected`
entry.

**Not in this axis**: whether a head is true of the code, and the restatements of a head in other
texts.

**The agent**: `agent@knowledge-architect-design-record-auditor`.

**The saved workflow**: `knowledge-architect-design-record-audit`. In one run, a
preflight lists every entry of every Component, in the order the auditor's section
`agent@knowledge-architect-design-record-auditor@take-your-group` gives, and the listings the
agents read; the script cuts the groups, a Component larger than the group size alone and the
smaller ones packed whole, and a setup agent creates their worktrees; one auditor per group drafts
one verdict per entry, its brief listing its entries and a shared calibration sample; once every
group has returned, one auditor compares the verdicts on the sample and proposes a reading where
they differ; one verifier per group tries to refute its drafts, under that reading; a checker judges
alone each head that a verified split, merge, rename or move creates, and each head a verified
retitle gives a new title; a cross-check maps the
findings that touch one paragraph or one slug, the tripwires and issues an edit would leave false,
and proposes each finding's outcome; the report gathers all of it. Its `args`:

| arg | what it is |
| --- | --- |
| `commit` | the commit audited |
| `scratch` | the run's scratch directory, under which every step and agent writes |
| `worktrees` | the directory under which the run creates the worktrees, outside the project or under a path its ignore rules cover |
| `history` | a git range whose commit messages, and the report files it added, record the audit's earlier passes, for the verifiers |
| `previous` | for a re-check, the commit the previous pass audited, against which each finding is classified as on old or new text |
| `kept` | for a re-check, the path of the file of entries and rules the owner ruled to keep |
| `sample` | optional: the calibration sample, each entry a head's or a `rejected` entry's reference; the preflight chooses one when it is left out, and a re-check may keep the entries whose readings differed |
| `counts` | optional: `groupSize` and `checkersPerGroup`; a key left out takes its default, written in the script |
| `from` | optional: `calibration` or `verify`, to restart a run from the files it already wrote; or `application`, to judge the session's application of the pass |
| `applied` | with `from` set to `application`: the commit that holds the application |

**The verification of a report.** The session reads the report whole; each reading the
calibration proposes, against the sample's drafts; every item of the owner-list draft, and every
finding the report marks as plausible or as bearing on a ruling of the owner or on a tripwire,
against its draft; every check; and a sample of the refuted drafts and a sample of the conforming
drafts, of its choosing. A reading of the calibration it rejects is corrected in the calibration
file, and the workflow runs again from `verify`. Where the rule's text admits both readings and no
record states the reading, the difference goes on the owner list.

**Application.** A split, a merge or a move of a head is drafted first, like any other finding: a
fold judged alone is the edit most often undone. The session commits its application, then runs the
workflow again with `from` set to `application` and `applied` to that commit: an auditor per group
judges every entry whose text the application changed, a retitle and a body edit included and a
re-pointed reference alone left out, against every rule, the failure modes of an edit and the
owner's rulings, with the tripwires and issues that cite it, since moving text unchanged does not
make a head conform and the run's checks read only the drafts. The session repairs each failure of
the report that a reading confirms and folds the repairs into the pass's commit, with no
uncommitted work in the tree.

**Old and new text**, for a re-check: the verifier classifies each finding.

**The outcomes of this axis:**

| outcome | the findings |
| --- | --- |
| applied in the branch | a rewording; the tense; a missing or wrong reference; an approval cited as a head's ground, removed when the head stands on its argument; a split that touches no word of the owner; a rejected design repaired in place, its reason made checkable or its marker set; a tripwire or an issue made stale by an edit of the audit, repaired in place |
| put to the owner, on the owner list | a widening of what the owner's words approved, such as a title moved from the members the owner named to the rule; the removal of a head that fails every entry test; the removal of a rejected design that fails every recording test; a ground that is the owner's words; a head in the wrong Component |
| an issue entry | a head false of the code, kind `defect`; a split or a move the owner approved that is too large for the branch, kind `design` |

A tripwire or an issue is edited only where an edit of the audit makes it stale. An issue whose
"What would close it" an edit of the audit does is closed in that commit.

## The agentic-workflow axis `##agentic-workflow-axis`

**The corpus**: the one `agent@knowledge-architect-workflow-auditor@the-corpus` defines, read
whole. The workflow functions as a whole, and its main defect is two texts a session cannot both
obey, so every agent reads the whole corpus.

**The rules**: a session can follow every instruction, and needs every statement. A finding is
admitted when a reading confirms it, in the classes `agent@knowledge-architect-workflow-auditor`
defines: a contradiction, a broken trigger, a factual error, two readings, a provable gap, divergent
verdicts, a restatement or a pointer its home does not allow, an unneeded statement. A gap that only predicts
what an agent would do is a predicted gap, and is never repaired by an edit, since an instruction about how an agent
behaves is added on evidence from real use. A case no instruction covers and judgement can decide
is no finding, per `primer@room-to-judge`.

**Not in this axis**: how the owner works, which the retrospective judges against the expectation
sets; the checker's own behaviour, whose code is not in the corpus; the scoped `CLAUDE.md` files.

**The agent**: `agent@knowledge-architect-workflow-auditor`.

**The saved workflow**: `knowledge-architect-agentic-workflow-audit`. In one run,
a preflight sets up a worktree per auditor and the listings the agents read; the auditors draft,
one per lens and count; one verifier per auditor tries to refute each draft; a dedup writes the
confirmed-findings file, each finding classified as on old or new text; cluster agents group the
findings by cause and a merge writes the causes file; repair agents propose each cause's repair, as
"The repair by cause" says; a check agent reads each proposal; a cross-check reads the proposals
and their checks together; the report gathers all of it. Its `args`:

| arg | what it is |
| --- | --- |
| `commit` | the commit audited |
| `scratch` | the run's scratch directory, under which every step and agent writes |
| `worktrees` | the directory under which the run creates the worktrees, outside the project or under a path its ignore rules cover |
| `history` | a git range whose commit messages, and the report files it added, record the audit's earlier passes, for the verifiers |
| `previous` | for a re-check, the commit the previous pass audited, against which each finding is classified as on old or new text |
| `kept` | for a re-check, the path of the file of entries, texts and rules the owner ruled to keep |
| `counts` | optional: the auditors per lens, keyed L1 to L9, `clusters` and `repairsPerCause`; a key left out takes its default, written in the script |
| `from` | optional: `verify`, `cluster` or `repairs`, to restart a run from the files it already wrote |

**The lenses**, each described in `agent@knowledge-architect-workflow-auditor@the-lenses`: L1
contradiction, L2 activity walk, L3 two readings, L4 provable gap, L5 restatement, L6 local against
installed, L7 checker rules, L8 verdict alignment, L9 need test. Each lens is dispatched at least once; L2's walkers
take its groups of activities in turn, and a group no walker takes is named in the report. Agents
with the same lens find different things, so a duplicate adds coverage. A project's own skill is
walked in the group of the activity it serves.

**The verification of a report.** The session reads the report whole, every repair proposal in
full, every finding the report marks as bearing on a ruling of the owner, on a tripwire or as
plausible, against its draft, and a sample of the refuted drafts it chooses.

**The repair by cause**, as `skill@knowledge-architect-agent-configuration@admission-standard` says, read whole. The
repair step proposes one repair per cause, never one per finding, and a draft's proposed repair is
one of its inputs; where `counts` gives a cause several repair agents, their proposals are
alternatives for that one repair. **The check**: one workflow auditor per proposal, under its check
brief, `agent@knowledge-architect-workflow-auditor@check-a-proposal`, reads the proposal's new text
in the corpus through L1 and L3, makes the record check of `skill@knowledge-architect-agent-configuration@admission-standard`, and writes its check
beside the proposal. The session's verification reads each proposal with its check against its
cause and the corpus, and corrects it or sends it back to a repair agent of its cause, briefed with
the script's repair prompt and with the check.

**Old and new text**, for a re-check: for each line that carries a confirmed finding's defect, the
dedup runs `git blame -w -C` at the commit audited. A line whose blamed commit is not an ancestor
of `previous` is on new text only where the words on it that carry the defect,
whitespace-normalised, are absent from every file of the corpus at `previous`, so an edit elsewhere
in a line the previous pass read leaves the finding on old text; a finding on two texts takes the
newer.

**The tripwires**: the preflight saves the output of `cargo klarch tripwires` too, and a verifier
marks `bears-on-tripwire` a finding that carries a tripwire's firing evidence, as
`agent@knowledge-architect-workflow-auditor@verify-drafts` says, without settling it. The session
judges whether each fired, and a fired tripwire moves as
`skill@knowledge-architect-issue-tracking@movement-instruction` says.

A cause's findings on the installed text are sent upstream, as below. Its findings on the
project's own text take the outcome its repair's edits on that text take in the table below:
applied in the branch when every such edit falls in the applied row, the owner list otherwise. A
finding marked known or kept, and a predicted gap, take instead the outcome
`skill@knowledge-architect-project-audit@finding-outcomes` and the table below give them. The
cross-check proposes each cause's outcome, and the session decides it. An item of the owner list is
one cause, with the ids of its findings, the repair, its count and a default. The owner list states
the net growth of the pass's repairs in the project first, where they add more instructions than
they remove, as `skill@knowledge-architect-agent-configuration@admission-standard` says. Each pass's commit
message also gives the corpus's word count before and after the pass, `wc -w` over the corpus's
files.

**The outcomes of this axis**, for a finding on the project's own text:

| outcome | the findings |
| --- | --- |
| applied in the branch | a contradiction that a design head, a goal or a source settles, repaired to the side it settles; a rewording that changes no instruction, a two-reading instruction among them where a design head or another instruction states the reading meant; a broken trigger whose target exists under another name; a pointer added to a restatement without one, or to a project text that repeats an installed one; a factual error; a restatement whose content drifted from its home, brought back to it, becoming a pointer when it is longer than one sentence, since the repair edits what it says; an unneeded statement that carries no instruction, deleted, or that is maintainer-only, moved to a comment of the source, as the table of `skill@knowledge-architect-agent-configuration@admission-standard` gives |
| put to the owner, on the owner list | which side of a contradiction wins where no design head, goal or source settles it, a loop with no exit among them; which standard wins, for divergent verdicts; a broken trigger or a two-reading instruction whose repair changes what agents are told, which a two-reading instruction is when nothing recorded states the reading meant; a provable gap, since filling it adds an instruction; an unneeded statement that is an instruction whose act no agent performs |
| an issue entry | a predicted gap; a finding the owner defers |

**Sent upstream**: every finding whose outcome is sent upstream, over every pass of the audit, a
predicted gap included, goes into one file per audit, written when the audit stops from the report
files of its passes, which history keeps. That file is the
workflow's file of `skill@knowledge-architect-retrospective@two-files`, named, written and handled
as that section and `skill@knowledge-architect-retrospective@what-becomes-of-files` say, its
subject naming the audit. It differs in its content only: what was audited at which commit in
place of what the session did, each finding numbered `W<n>`, and no standing answers.
