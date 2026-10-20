---
name: knowledge-architect-review
description: MUST use before merging anything to the main branch, and whenever an activity's own skill says a unit of its work is ready for review — how to send independent subagent reviewers at it, how to write a brief that cannot mislead them, and where their findings land. The axes to send are every axis of its table whose condition holds, and those the skill of each activity whose work it reviews adds.
---

# Dispatching a review

Scope: sending independent reviewers at work you produced, whatever kind of work it is. One
sub-activity, required as a prerequisite by every activity that produces something reviewable.

**When to dispatch is the activity's**, because the moment differs and so does the vocabulary that
names it: the project's development procedure says when a piece of code is ready,
`skill@knowledge-architect-planning` says when a plan document is, and
`skill@knowledge-architect-agent-configuration` says when a configuration change is. Read your
activity's skill for its moment. The one moment that belongs to no activity is **before merging
anything to the main branch**: `skill@knowledge-architect-review@review-axes` says what that
review owes.

Not covered here: **being** any of the reviewers (the record reviewers
`agent@knowledge-architect-routing-reviewer`, `agent@knowledge-architect-decision-record-reviewer`
and `agent@knowledge-architect-standing-state-reviewer`; the plan-document reviewers
`agent@knowledge-architect-cold-implementer-reviewer`,
`agent@knowledge-architect-code-claims-reviewer` and
`agent@knowledge-architect-design-conformance-reviewer`; and
`agent@knowledge-architect-transcript-reviewer`), **recording**
what a review changes (`skill@knowledge-architect-decision-recording`,
`skill@knowledge-architect-issue-tracking`), and **auditing** one aspect of the whole project
rather than a diff (`skill@knowledge-architect-project-audit`).

## The axes `##review-axes`

| axis | what it does | applicable when |
| --- | --- | --- |
| spec conformity | does the work implement what was decided, item by item. If deviations happened during implementation, are they justified? | a spec or a milestone slice's spec was written before the work |
| self-consistency | does the result contradict itself. Two instructions a reader cannot both obey, a pointer into content that is not there, a statement no longer true | nearly all the time |
| fidelity of relocation | where content moved, was anything lost? A reason dropped, a number changed, an argument compressed to an assertion | content was relocated, or forked from another source |
| routing of knowledge | `agent@knowledge-architect-routing-reviewer` | a durable statement was added or moved |
| agent-facing text | `agent@knowledge-architect-agent-text-reviewer` | the diff adds to or changes agent-facing text, installed or the project's own |
| decision recording | `agent@knowledge-architect-decision-record-reviewer`. If a plan document was written, hand it to this reviewer too | a decision was made, reversed or harvested |
| standing state | `agent@knowledge-architect-standing-state-reviewer` | at the review before a merge, since it is the standing re-entry point of every tripwire and every deferred trigger |
| cold implementer | `agent@knowledge-architect-cold-implementer-reviewer`: can a session that did not see the discussion act on the plan document | a spec or a milestone was written under `skill@knowledge-architect-planning`, or a decided shape in one revised; that skill names the moment |
| code claims | `agent@knowledge-architect-code-claims-reviewer`: is every statement the plan document makes about existing code true of the tree | the same moment |
| design conformance | `agent@knowledge-architect-design-conformance-reviewer`: does the plan document fit the project's goals, design heads and rejected designs | the same moment |
| transcript | `agent@knowledge-architect-transcript-reviewer`: has everything the work's sessions established that must outlive them a durable outcome, and is no ruling of the owner misstated | the transcripts of the sessions that produced the work are available; at the review before a merge, alone and last (below), not also with the other axes; at an earlier review of the branch, only where the work is a plan document assembled from a transcript, in the batch |

**A review owes every axis of this table whose condition holds, and every axis that the skill of an
activity whose work it reviews adds, where that axis's condition holds.** The review before a merge is the last review of a branch, after
which only its own repairs land, whichever activity's skill dispatched it, such as the review of a
plan document on a branch of its own; the work it reviews is the whole branch. **An axis named by
an agent is dispatched as that agent. An axis with no agent is dispatched as a fresh
general-purpose subagent**, briefed with the axis's question as this table states it, the commit range, and the files that carry the standard. **An axis that applies and was not run is said,
with the reason, in the commit that records the review.**

**Last, at the review before a merge, the transcript reviewer runs alone**:
after every other axis has run, its repairs are committed and every item of their reports has its
outcome under `skill@knowledge-architect-review@what-review-leaves`, since an answer of the owner
given after its extraction reads to it as missing. Its range is the whole branch,
`<main branch>..<head>`, and its brief names the transcripts of every session that worked on the
branch, with the message where the branch's work begins in each. It is the one axis that can see whether the findings of the others were acted on, since
their reports reach the session as messages of its transcript.

**Find each transcript file by the message where the work begins, never by a session
identifier.** Search the harness's transcript directory for the opening words of that message, and
name a file only once it is seen to hold the message as one the owner sent, not as a quotation of
it in a later session: a session that was cleared or compacted can leave an earlier session's
identifier in the harness's paths.

## The invariants `##review-invariants`

- **Each axis is a separate subagent, and none sees another's findings.** Reviewers given a shared
  findings list converge on it. The transcript reviewer is the one exception: its subject is the
  session, and the session received the other axes' reports, so it reads them as items to find
  an outcome for, never as a list to agree with.
- **Each reviewer is fresh, never a fork.** A fork inherits the session's discussion and reads the
  work as its author.
- **At least one reviewer is briefed blind**: given the task and no list of what anyone else found,
  and no summary of what is believed to be true. The finding that changes a design is usually the
  one nobody was looking for.
- **Reproduce before claiming.** Every reviewer is told to reproduce anything it asserts, and to
  drop what it cannot. It is the whole difference between a finding you act on and a finding you
  re-check.
- **A brief must not assert what the tree contains.** Give the task and the standard, and tell the
  reviewer to establish the state itself. A brief that describes the implementation sends a reviewer
  to test something that may not be there, and it cannot tell a wrong brief from a broken artifact
  unless it was told to look.
- **Name what is reviewed as a commit range**, `<main branch>..<commit under review>` for a branch
  about to merge, unless the axis names its own range: the diff and every commit message in it
  are the subject, and a reviewer given a branch name alone guesses the base.
- **Name the files that carry the standard rather than restating it.** A subagent inherits the
  session's snapshot of the root `CLAUDE.md` and of the primer it imports, and runs on its agent
  definition as the harness registered it when the session started, so once any of them changes
  on disk, by an edit or by an install, the session is briefing from a copy that no longer matches
  disk. Point at the file; do not paraphrase what it says. Where the file changed on disk after
  the session started, tell the subagent to read it from disk and to follow the disk where its
  copy differs.
- **The dispatcher does not defend the work.** Findings arrive as claims to check, not as attacks to
  answer. Verify each against the tree before relaying or acting on it; a reviewer can be wrong, and
  saying so requires the same reproduction the reviewer owed.
- **Every subagent that judges a commit, and runs tests, a mutation, or any command of the checker
  or of a program the project builds, gets its own worktree, detached at that commit, never the
  live tree**: a reviewer at the commit under review, an auditor at the commit audited. A run in
  the live tree races the dispatcher's own edits, and a worktree that shares the branch ref moves
  under the subagent at the dispatcher's next commit. The shape is
  `git worktree add --detach <path> <commit>`, at a path outside the project, or under a path its
  ignore rules cover, since the checker walks untracked files and stops on a nested repository,
  and `git worktree remove <path>` once the subagent's findings have their outcomes. **The
  dispatcher names each subagent's path in its brief, distinct for each subagent**, so subagents
  dispatched together never build inside one another's worktree. **It names a scratch directory
  too, distinct for each subagent dispatched together**, and a subagent that writes working files
  writes them there only: subagents sharing one scratch directory overwrite one another's files.
  **The subagent's build output stays inside its worktree.** Unless every target is tied to its
  checkout, as the setup skill shows, a build directory two checkouts share lets the live checkout
  run the subagent's build, which judges the live tree with the subagent's code. Where every
  target is tied, a shared directory costs a rebuild at each switch between the two checkouts
  instead. In a Rust project, the subagent sets `CARGO_TARGET_DIR` to a directory inside its
  worktree that the project's ignore rules cover, such as its `target/`: a build directory git does
  not ignore is walked by the checker, and a finding in any of its files fails the run.
- **No reviewer edits the live tree, and none runs there an operation that can lose content**: no
  stash, no reset, no checkout of a path. After the reviewers return, the dispatcher compares
  `git status --short` of the live tree with its own changes, and puts anything else to the
  reviewer before touching it.

## What a review leaves behind `##what-review-leaves`

**Findings** become one of:

- repairs, done on the branch before merge;
- issues, one file each in the affected anchor's issue directory
  (`skill@knowledge-architect-issue-tracking`);
- nothing, where the finding is judged to need nothing, with the reason.

**A finding that makes several claims gets an outcome for each claim**: repaired, opened as an
issue, or judged to need nothing, with the reason. A claim left without an outcome of its own is
lost unless a later review finds it.

**Every item of a report gets an outcome, not only its numbered findings**: a note outside the
reviewer's axis, an observation it declined to call a finding, a remark that a defect predates the
change. Each is a claim about the work like any finding. So is a repair the dispatcher promised the
owner while answering a review. The record of the review lists each item with its outcome. The
dispatcher saves each report in a file outside the project, such as a scratch directory, as it
arrives, and builds the record from those files: a compaction of the conversation keeps neither
the reports nor their items.

**A repair that adds, removes or reverses a design head is reviewed by the decision-record axis**,
and by any other axis whose condition it meets, whichever axis's finding it answers. A repair made before the last
transcript review is reviewed before it, and that transcript review then runs as `skill@knowledge-architect-review@review-axes` says; a repair of the last transcript review
is reviewed after it, and the review ends with that review's repairs. Any other repair, a rewording
or an argument, a mention or a reference added inside an existing head included, and a title
brought to what its head's body already argues, is reviewed by no axis again; one made before the
last transcript review is still read by it.

**An item whose outcome `primer@question-authority` gives the owner is put to the owner** before its
outcome is recorded; an item whose outcome the record, a measurement or a source settles takes it
from the session, and the record of the review and the turn's "Decided" section report it. Either
way, its outcome is then one of the three above. Several items put to the owner go in one
message, each under a label, `Q<n>`, so the owner rules on each by its label; the record of the
review names each item by its content, not by the label. Where the dispatching skill gives such an
item a label of its own, as the planning skill gives a plan document's defaults `D<n>`, that label
is used instead. Where that skill writes such an item into its document as a default awaiting
the owner, as planning does, that listing puts it to the owner and is its outcome, a repair, and
the record of the review names it as awaiting the owner's word. **A defect that predates the
change** is routed by `primer@met-outside-the-task`: a fix checkable from the
diff alone lands in a commit of its own, and the record of the review names that commit by its
subject.

A transcript reviewer's finding that something **has no durable outcome** is acted on by the
dispatcher without waiting for the owner: it is recorded in its home, repaired, opened as an issue,
or judged to need nothing, with the reason. The exception is a decision that creates a design head,
contradicts a statement of one, or adds a member its argument does not cover, and was not argued:
it goes to `skill@knowledge-architect-design` first, per the backstop of `primer@design-heads`. Each
outcome is reported to the owner, in the record of the review and in the turn's "Decided" section,
per `primer@decided-section`. A ruling
the reviewer finds misstated is the owner's, and is put to the owner.

**Where the branch's commits reach the main branch as they are** (a fast-forward, or a rebase merge):

- **A repair of the branch's tree is a new commit, appended**, which edits no history, except one
  that `primer@branch-commits` folds into an earlier commit. Either way, the paragraph recording
  the review says what was repaired, and what was folded.
- **A commit message carrying a mistake is repaired by amending** while it is the newest commit, and
  by a history edit of the branch after that. Either only with no uncommitted work in the tree, and
  each confirmed afterwards to have lost no content: for an amend, that it changed no file. A
  correction written into a later commit's message is not a repair: the mistaken message still
  reads as it did. A message is judged against its own commit: a plan it states that a later commit
  of the branch changes is no mistake when the later commit's message names the change.
- **A paragraph in the commit message** records the review: the message of the last repair
  commit, or, where no repair commit is left, the message of the branch's last commit, reworded
  with no uncommitted work in the tree. A record carried by a message rides on a commit that
  changes a file: a commit of its own changes none, and the rebase merge drops it. Where no
  commit of the branch changes a file, the record is the report to the owner.

**Where the project squashes a branch into one commit on merge**, the record of the review goes
where the project keeps what survives the merge: the squashed commit's message, or the pull
request.

The record tells what was reviewed, on which axes, how consequential the
findings were, which were repaired, which were left and why, and which axis was not run and why.
It names each reviewed commit by its subject, not by its SHA: a SHA of the branch may name nothing
once the branch reaches the main branch, since a rebase before the merge or at it gives the
branch's commits new SHAs.
Every claim that paragraph makes about the tree is checked before it is written, like any other.
