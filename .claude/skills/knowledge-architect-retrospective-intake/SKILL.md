---
name: knowledge-architect-retrospective-intake
description: MUST use when the owner hands this session one or more of the project's own retrospective files to act on — a `<YYYY-MM-DD>-<project>-<subject>.md` file, the findings of a retrospective on the project's own instructions — or asks what to do with the findings of such a file; and while an outcome of an analysis in docs/retrospective-reports/ is carried out, for what the analysis owes until it leaves. It runs on the owner's word. Covers checking each finding against the text as it stands, against the registers and the history, against the goals and the design, looking for a better fix than the one proposed, writing the analysis to a file, offering the owner an outcome per finding, and the life of the analysis after the ruling. Not for running a retrospective, nor for the retrospective's workflow file, which goes to the workflow's repository, nor for the work an outcome rules, which its route's skill covers.
---

# Retrospective intake

Scope: the analysis of the findings of the project's own retrospective files, the owner's ruling
on each, and the life of the analysis file until it leaves.

Not covered here:
- **running a retrospective**, and sending its workflow file to the workflow's repository:
  `skill@knowledge-architect-retrospective`;
- **the work an outcome rules**: an issue is opened under
  `skill@knowledge-architect-issue-tracking`; a finding handled now is handled under the skill that
  item g of `skill@knowledge-architect-retrospective-intake@what-to-establish` names for it. What
  the analysis file owes meanwhile is `skill@knowledge-architect-retrospective-intake@after-the-ruling`.

**This skill decides nothing.** It establishes the facts about each finding, proposes an action
with its argument, and the owner rules.

## What it takes `##what-it-takes`

- **In scope: the project's file of a retrospective**, `<YYYY-MM-DD>-<project>-<subject>.md`, any
  number of them at once. It holds the findings on the project's own instructions (P) and the
  project's side of an interaction finding. What each of the retrospective's two files holds is
  `skill@knowledge-architect-retrospective@two-files`: read it whole before reading a file.
- **For grounding only: the workflow file of the same retrospective**,
  `<YYYY-MM-DD>-<project>-<subject>-klarch-workflow.md`, when it is present. An interaction finding
  has an id in each file, each naming the other; the workflow file gives the other side. Nothing of
  it is resolved here: its findings are the workflow's. It is written to only under the outcome
  "sent upstream".

A finding is cited by its id and its file's stem, in the form
`skill@knowledge-architect-retrospective@what-it-examines` gives.

## Ground before judging any finding `##ground-before-judging`

1. **Read each file whole.** The session usually did not write it, so it knows of the reported
   session only what the file says.
2. **Note the version each file used**, from its version line: the main commit the session's tree
   contained, and the checker's version. Note the main commit you analyse against,
   `git rev-parse --short <main branch>`. A finding against an older commit or an older version of
   the checker may already be repaired.
3. **Search the standing entries with the received files as the work.** One search covers every
   file received together. Dispatch `agent@knowledge-architect-standing-entry-searcher` as its
   description says, with the files' paths and their subjects as the work, and as the seeds the
   design heads and goals the findings name, or none.

   Beside it, search the goals, the design entries and the `rejected` entries, with the same work
   and seeds: dispatch `agent@knowledge-architect-design-entry-searcher` as its description says, to
   each Component that owns a blamed text, the project's root always among them. A proposed fix
   that touches a Component this search was not given sends it again, for that Component, before
   the proposal goes to the owner.

   Read every entry returned, and every entry graded `bears` or `may-bear`, whole with
   `cargo klarch show <ref>`, never from its reason line. The searches find what a grep per finding
   does not: a finding already recorded, a deferred issue whose trigger the file meets, a tripwire
   whose firing evidence the file carries, a design issue whose re-entry it meets, and a head or a
   goal a proposed fix would contradict or strain.
4. **Cluster the findings.** One defect is often reported in several files, or as two findings of
   one file. A cluster, and a standing entry that several files meet, is analysed once, in the
   analysis of the first file by stem that holds it, and cites every id in it; the analysis of each
   other file holds a section that points there. That a
   finding recurs across sessions is evidence: say how many times.

## What to establish for each finding or cluster `##what-to-establish`

The order is a suggestion; the content is not.

- **a. What it says.** What happened in the session, which text it blames, quoted, and the fix it
  proposes.
- **b. Whether it still applies.** Read the blamed text at the main commit the version line names,
  `git show <commit>:<path>`, then on the main branch as it stands, which holds the reported
  session's branch once it merged, and quote it with its `file:line`. A version line whose commit
  the repository does not hold is read at the commit its `git describe` count gives on the main
  branch, and a file whose version line names no commit is matched by `git log -S '<phrase>'` on
  the instruction it quotes. The report is a claim about the text at its
  version, so it is checked like any claim about the code. A finding may be already repaired: name
  the commit, found with `git log -S '<phrase>'` or `git log --grep`. A report's own "repaired in
  the session" is a claim like any other, and is often wrong: find when the credited wording
  entered, with `git log -L` or `git log -G` on it, and compare with the session's work. A phrase
  searched in a commit message or in wrapped prose is matched across a line break.
- **c. Whether it is already recorded.** An issue or a tripwire, a design head, or a rejected
  design, from the searches of `skill@knowledge-architect-retrospective-intake@ground-before-judging`. A proposed fix that already lost is not proposed again: an alternative counts
  as argued and lost only when a recorded reason covers it, per the record test of
  `skill@knowledge-architect-design@threads-and-states`.
- **d. Its kind.**
  - **a defect of the text**: unclear, missing or wrong, as
    `skill@knowledge-architect-retrospective@what-it-examines` defines the three, read whole;
  - **a lapse of the session**: the instruction exists, is clear, and was not followed. What
    repairs it is the lapse rule of `skill@knowledge-architect-agent-configuration@admission-standard`;
  - **about the owner's behaviour under an installed skill**: judged against that skill's
    expectation set, in `skill@knowledge-architect-retrospective@expectation-sets`, read whole at
    this check. Behaviour the set says the skill assumes otherwise is outside that skill's scope.
    An installed skill with no set has no such test yet, and a finding on the project's own skill
    has none. **A finding that two installed instructions leave no move satisfying both is always
    in scope**, and is never closed by citing an expectation set;
  - **an observation**: a use that went well, or a behaviour not reproduced. It needs no action.
- **e. Its scope.** The goal it serves or threatens, from the goals homes, `register@*@goal`. The
  owner of the fix: the Component or the location that owns the blamed text; or the installed
  text, which the project never edits, per
  `skill@knowledge-architect-agent-configuration@installed-files-never-edited`; or the checker.
  Whether the fix reverses a recorded decision, per
  `skill@knowledge-architect-decision-recording@reversal-check`: if it does, the fix is a reversal
  and goes through that skill. The record check of item f names the heads it would reverse.
- **f. A better fix.** Name the nearest rival to the fix the report proposes, and the fact that
  decides between them. Rivals worth looking for:
  - the same defect fixed at another site: the checker rather than the text, the installed text
    rather than the project's, or the reverse;
  - one fix for a whole cluster;
  - a fix where the instruction is delivered, rather than more instruction;
  - no change.

  The fix is shaped, and read against the record, as
  `skill@knowledge-architect-agent-configuration@admission-standard` says, read whole: it is the
  standard of every actor that writes or judges agent-facing text.
- **g. Its route.** What handling it needs, which says what "now" would mean:
  - a text edit of the project's own agent configuration whose design is settled:
    `skill@knowledge-architect-agent-configuration`;
  - a decision, or one whose design is not settled: `skill@knowledge-architect-design`;
  - a change to the project's code: the project's own development procedure;
  - an issue only, when the work is not to be done in this session;
  - a fix in the installed text or in the checker: sent upstream;
  - nothing.
- **h. The proposal.** The action, its argument, and the default outcome
  (`skill@knowledge-architect-retrospective-intake@analysis-and-ruling`).

**Every verdict carries its evidence**: a quotation with its file and line, a command and its
output, a commit. A verdict without evidence is labelled an assumption.

## Subagents `##intake-subagents`

The checks a to c read text, and fresh subagents may run them, one per file or per cluster. Brief
each with the findings, the files to read, the entries the search of
`skill@knowledge-architect-retrospective-intake@ground-before-judging` returned, and a scratch
directory of its own, never shared. The agent checks c against those entries, and does not
search the registers again. Each returns, per finding: the quotation,
its file and line, and its verdict. The session itself establishes d to h, and re-checks every
verdict a proposal rests on. No subagent replays the reported session or builds a scenario of agent
behaviour.

## The analysis file and the owner's ruling `##analysis-and-ruling`

Write the analysis in docs/retrospective-reports/ at the project's root, as `<stem>.md`, where
`<stem>` is the received file's stem.

- A head: the received file's name, the version it used, the main commit analysed against, and the
  standing entries the search returned that bear on it.
- **One section per finding or cluster**, headed by its ids and a short title, with a bullet per
  item of `skill@knowledge-architect-retrospective-intake@what-to-establish`, in prose. No table:
  the content of each item is too long to read in a cell.
- **Every section ends with a `Default` line and an `Outcome` line**, whatever it proposes, no
  change included, the latter left pending: a finding, a noted item, and a judgement on a standing
  entry. A standing entry the file may meet gets a section of its own: a tripwire that may have
  fired, a deferred trigger, a design issue's re-entry. Whether it fired, and its response, are the
  owner's to rule; the analysis proposes the judgement and its evidence.
  A section that points to a cluster analysed in another file has neither line: the ruling is given
  where the cluster is analysed, once, so that it cannot be given twice and differ.
- **A span the checker would refuse is quoted in words**, and an entry an outcome removes, renames
  or creates is named as `primer@where-knowledge-goes` says. A finding about a refused form tempts a
  quotation of that form, and the analysis then fails the check it reports on. The project's paths
  and entries are written in the checked forms, since the file is checked like any other.

Then give the owner the full path of each file, and in the conversation one line per section:
its ids, the proposal in a few words, and the default outcome, or, for a section that points to a
cluster, the file that analyses it. **Propose that the owner rules by
writing each `Outcome` line in the file**: with many findings, a ruling written beside its
evidence is easier to give than one in the conversation, and it lands in the record as written. A
ruling given in the conversation instead, such as "now: P1, P5; issue: P3; no change as proposed
for the rest", is copied into the `Outcome` lines verbatim. Before committing, list every section
whose `Outcome` line is still pending, and ask the owner for those. **Four outcomes**:

- **handle now**: the finding is handled after the ruling, on the same branch or on the branches
  that follow it, under the skill its route names. It needs no issue entry, and the proposal does
  not suggest one;
- **open an issue**: name its anchor and its kind;
- **sent upstream**: the fix lies in the installed text or in the checker. Ask the owner whether
  the workflow file of the same retrospective was sent, and how:
  - not sent yet: add the finding to it, as a W finding for the installed text or a C finding for
    the checker, with the next number of its letter in that file, the two ids each naming the
    other, as an interaction finding's do;
  - sent as an issue on the workflow's repository, which
    `skill@knowledge-architect-retrospective@what-becomes-of-files` names: edit the issue's body
    with `gh issue edit <n> --repo <that repository> --body-file <file>`, and post a one-line
    comment naming the added finding with `gh issue comment <n> --repo <that repository>`, so that
    those who follow the issue see it;
  - sent with no issue, or an issue that cannot be edited: open a new issue for the finding, as the
    retrospective opens one.

  Nothing leaves the machine before the owner has read it. Where `gh` is absent or fails, give the
  owner the text and the address. The outcome is carried out once the owner has sent it, which the
  message of the commit that carries out the analysis's last outcome says;
- **no change, with the reason**: a lapse, already repaired, out of scope, or an observation.

A question tool is not the way to collect the ruling: it shows a label, not the evidence and the
rival fix the ruling depends on.

## After the ruling `##after-the-ruling`

Each section's `Outcome` line holds the owner's choice and the owner's words, verbatim. The
analysis is committed, under the project's own branch rules, once every `Outcome` line holds the
owner's ruling, and before any outcome is carried out. The issues it rules are opened, and the
findings handled now are handled, in the commits their routes need. The commit that carries out its
last outcome, or a later commit of the same branch, deletes it, and its message names the analysis
by its path. History keeps it: `git log -- docs/retrospective-reports/` lists every analysis.
Then mark each received project file the analysis carried out: a line at its head saying that it
was analysed and every outcome carried out, and that `git log -- docs/retrospective-reports/<stem>.md`
in the project finds the analysis. A file the owner keeps is then not handed to an intake again.

A finding handled now can turn out to need a design discussion and a plan. Its `Outcome` line is
then rewritten, on the owner's word, to name the plan document or the issue that now carries it,
so the analysis can still leave.

A file whose cluster other files point to leaves after them, or the commit that deletes it rewrites
the pointing sections, since their references would dangle.
