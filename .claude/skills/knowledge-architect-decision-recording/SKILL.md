---
name: knowledge-architect-decision-recording
description: MUST use before writing into a design home, whatever the activity, and when a design decision has been made or reversed — a choice about how the project or one of its Components is built, including anything a consumer of it may depend on — in order to decide whether it earns durable text at all (most implementation choices do not) and where that text lands. Covers when recording happens, the reversal check that comes before everything else, which Component owns it, the split between the design home, the rejected designs and history, where a head is written and how a slug is renamed, and when a tripwire the owner ruled on is written; what earns a head and how a head is shaped are the primer's section on design heads.
---

# Recording a decision

**Read `primer@design-heads` again, whole, before writing or judging a head.** The primer was
loaded when the session started, and a recording comes late in it.

Scope: turning a decision about **what we build** into durable text. Whether it earns durable text
at all, when it is written, which Component owns it, what goes in the design home, what stays in
history, and when a tripwire the owner ruled on is written.

Not covered here: **having** the discussion (`skill@knowledge-architect-design`);
**writing the spec or the milestone document** that carries a decision until its work lands
(`skill@knowledge-architect-planning`); **parking something undecided**, and writing a tripwire's
entry (`skill@knowledge-architect-issue-tracking`). A decision about the agent configuration itself
is both this and `skill@knowledge-architect-agent-configuration`: that skill owns how to write the
configuration, this one owns where the argument lands.

## When recording happens `##when-recording-happens`

**A decision is recorded when the work that implements it lands, not when it is decided.** A
design home holds built intent: the design as built and its reasons, against which the code is
checked. A head written before the code exists would report a defect in code nobody has written.
While the work is open, the decision is unbuilt intent, and the spec or the milestone document is
the only place it exists, on the main branch or on the work's. Writing it into the design homes from there is
the **harvest**, and it happens in the change that lands the work.

A decision that no work implements, such as a policy, and that is not part of any spec, is
recorded when it is made: there is no implementing change to wait for. The second entry test of
`primer@design-heads` admits it, as a decision that no code and no text of the project states.

**A default awaiting the owner is built as written, per
`skill@knowledge-architect-planning@working-a-slice`, and enters a design head once the owner rules
on it.** Until then it is no approved decision, and a head is read against the owner's approval
before it is written, per `primer@design-heads`.

## Does it reverse something already recorded? `##reversal-check`

**Ask this first, because the answer decides everything below it.** A decision routed by what it
_is_ lands in the Component that fits its subject. A decision that reverses a recorded one belongs
where the incumbent already lives, whatever the subject test would say. Get the order wrong and the
tree ends with two homes for one question, the old one still asserting what the new one denies.

**Search for the statement you are about to replace, not for a slug.** The slug you would look for
is the one you have not written yet, and the incumbent's own name is rarely the words you are
thinking in. Grep the subject's terms across every document that carries decisions (the design homes, the
`rejected` homes, the scoped `CLAUDE.md` files) and the comments of the code the subject
touches, then `git log -G'<term>'` for the argument behind them. Say in the commit message what you
searched and what it returned. A search that returned nothing is a finding worth one line there.

**If nothing holds it, carry on to the entry tests of `primer@design-heads`.** If something does, do all of the following, in the same
change:

- **The Component is the incumbent's.** The test of `skill@knowledge-architect-decision-recording@owning-component` is not asked again. If the incumbent looks misfiled, that is a separate change with its
  own argument.
- **It earns durable text whatever the entry tests of `primer@design-heads` say.** The
  incumbent's head now states something false, so the minimum is rewriting it. The entry tests
  decide whether a _new_ decision earns an entry, not whether a false one may stand.
- **Rewrite the head in place** (`skill@knowledge-architect-decision-recording@current-design`), and give the new statement its own slug when it is a new
  statement. Keep the old slug only where it still names the same decision. The slug and the
  title stay aligned with the full scope of the decision, per `primer@design-heads`. A rename is
  cheap, and the checker lists every reference it leaves dangling in the files it reads, per
  `primer@where-knowledge-goes`.
- **Move the incumbent into the Component's `register@*@rejected`** (`skill@knowledge-architect-decision-recording@losing-alternatives`) with its reason and a
  validity marker, stated as strongly as it was originally made, if it meets one of the tests of `skill@knowledge-architect-decision-recording@losing-alternatives`, and cite it from the new head.
- **Argue it under `skill@knowledge-architect-design` first**, if it was not: a reversal contradicts
  a statement of the incumbent, the second case of the backstop of `primer@design-heads`.
- **Delete the tripwires guarding the reversed decision.** `skill@knowledge-architect-issue-tracking` owns that movement.
- **Repair what pointed at the old behaviour**: skills, subagent definitions, scoped `CLAUDE.md`
  files, generated headers. Grep the behaviour's wording as well as the slug, since a pointer that
  describes the behaviour without naming it is the one a slug grep misses. A sentence about the
  past that named the old slug is rewritten to state the present, or removed, never retargeted and
  never turned into plain text, as `skill@knowledge-architect-issue-tracking` says of a deleted
  entry, a record of a deliberation naming it as `primer@where-knowledge-goes` says; a verbatim quotation of the owner is left as it is, with a reference to the current entry
  beside it.
- **Close the issue entry that asked the question**, in this commit rather than a later one, and
  rewrite whatever half of it survives rather than deleting the whole.

## Which Component owns it `##owning-component`

The project is partitioned into Components, which its manifest declares. The project's root is a
Component too.

1. Does the decision shape the layout of the project: which Components exist, and how they depend
   on each other? Then it belongs in the root Component's design home.
2. Does it shape how every Component is implemented: a design principle every Component follows?
   Then it belongs in the root Component's design home.
3. Does it shape how a single Component is implemented, or its own interface? Then it belongs in
   that Component's design home.

If it is a statement about what the project or a Component is _for_ rather than about how it is
built, it is a goal, not a decision. Goals are a register of their own, the goals home of each
Component, each entry a level-two heading carrying a slug. A decision that leans on one names it as
`goal@<component>@<slug>`.

If none of these fits, **ask the owner before writing it anywhere.**

## A decision approved with no argument `##unargued-approval`

**A decision approved with no argument is argued before it is recorded.** Before a head is written
from an approval that no argument for the decision, no rival and no cost stand behind, such as a
default approved in a batch, search for the arguments for and against it, its costs and its rivals,
and record what the search finds. When the search leaves the rivals equal, put the fork to the
owner as a tie. The owner's choice is then the head's ground, as a decision that came from the
owner, per `primer@design-heads`.

## Three homes, split by function `##three-homes`

| what | where | why there |
| --- | --- | --- |
| the current design, and its **standing argument** | the Component's design home | it is rewritten in place, so it cannot go stale |
| a design not kept, and why | the Component's `register@*@rejected`, each entry cited by the heads whose argument covers it | a reader about to propose it must meet it without knowing to look, and a reader of the head meets its citation |
| the **deliberation**: what was weighed, in which order, what evidence was built, who ruled what | the spec or milestone document while it exists, then history; the commit message when there was no spec | failing to find it costs a re-derivation, not a wrong decision |

**The standing argument**, and what stays out of the head, are `primer@design-heads`.

**The deliberation is not copied into the head**, and, where a plan document carries it, not into
the harvest's commit message either.
A plan document is deleted when its work lands, per `skill@knowledge-architect-planning`, and the
commit that deletes it cites it by its kind. A reader who needs the deliberation finds it there:

```sh
git log --diff-filter=D --name-only -- docs/plans/         # every deleted plan document, with its commit
git log -G'<slug>'                                         # every commit whose diff touches the slug
git show <commit>^:<path of the document>                  # the document as it stood before deletion
```

**`-G` matches any commit whose diff touches the slug. `-S` matches only a change in how many times
it occurs, so it misses every revision.**

When a decision was taken with no spec, on the in-change path of `skill@knowledge-architect-design`,
the deliberation is where that path puts it.

## The current design `##current-design`

In the Component's design home, `register@*@design`: one file, or, for a Component whose design
has outgrown one file, a directory. In the directory shape the decision goes in the
subdocument owning its subject, never in the directory's `README.md`, which is the head and the
index. A new subdocument is linked from that index, `[title](file.md)`, the target relative to the
README, conventionally one bullet per subdocument.

**What a head holds and how it is shaped is `primer@design-heads`**: read it whole before writing
a head. The template:

```markdown
### <The decision, stated as a sentence> `##<slug>`

<The shape, present tense. Then the standing argument, per the primer's section on design heads.>
```

The slug is an identifier. Code comments, tripwires, other documents and `git log -G` all cite it,
so **renaming one means rewriting every reference in the same change.** Grep before you rename.

## Rejected designs `##losing-alternatives`

In the Component's `register@*@rejected`: a heading register, one level-three heading per entry,
cited `rejected@<anchor>@<id>`.

**Most designs not kept earn no entry.** An entry is read by someone reasoning from first
principles who was not there, and it is reached by grep from anywhere in the tree, so it is a
description of a rejected design that must never be read as the current one.

**Which threads of a discussion are candidates.** A thread's final state does not decide it. The
question is whether **a shape lost to an argument**.

| the thread | what happens to it |
| --- | --- |
| ruled-out | judged by the tests below |
| withdrawn with a defeating reason | the same as a `ruled-out` thread: its own proposer gave it up on an argument |
| withdrawn with no defeating reason | the deliberation only. It carries no argument a later reader could test. |
| superseded, when the absorbing thread carries its shape whole | the deliberation only |
| superseded, when a distinct shape lost | judged by the tests below, as a ruled-out thread |
| a shape rejected with nothing built in its place | judged by the tests below; the decision not to build it is a head of absence, "we do not do X", which entry test 2 of `primer@design-heads` admits, and which cites the entry |
| a shape that lost while the question stays open | the deliberation only, and the question is an issue under `skill@knowledge-architect-issue-tracking` |

**A design earns an entry only if at least one of these holds:**

1. it would change an interface others consume, as entry test 1 of `primer@design-heads` defines
   it;
2. its reason rests on a reading of an external specification the project implements;
3. it was refuted by **evidence that cost work to obtain**: a measurement, a prototype, a survey of
   other projects;
4. there is still some doubt that the design kept will achieve every goal of the project or of its
   Component.

Test 3 is the load-bearing one. Proposing something again costs one discussion round, and that is
healthy. What decides an entry is whether the refutation can be derived again inside that round.
Reasoning derives again for free, so a design refuted by an argument any competent reader
reconstructs needs no entry. Evidence does not, so a design refuted by measurement does.

Test 4 gives future design sessions the designs that were not kept, so they can start from
somewhere when the one kept proves insufficient.

A design meeting none of those stays in the deliberation, where
`skill@knowledge-architect-decision-recording@three-homes` keeps it, and nowhere else.

**Write the entry so that it cannot be read as the current design.** Name the design and the reason
it was not kept. Do not describe how it would work: an entry detailed enough to build from is the
one a grep turns into a specification.

One entry per design, keyed by **the design**, not by date. Flat, unordered, and **edited in
place**. Where it was a thread of a design discussion, its slug is the thread's name.

```markdown
### <The design not kept, named in a few words> `##<slug>`

`live`. <The reason it was not kept, checkable without leaving the entry.>
```

Each entry carries: what the design was, a **validity marker** opening its body, and the reason.
It names no winner as such, and its reason may name a head where the reason needs one: **each
design head whose argument covers the design cites the entry**, where
its argument meets that design, or in a closing sentence of its body. Writing the entry and its
citation is one act: `cargo klarch check` reports an entry that no design entry cites.

- `live`: the reason still holds.
- `void`: the reason no longer holds. **Mark it; never delete it.** The design may still be wrong,
  but _this_ argument no longer supports that, and the marker prevents reopening a settled question
  on a dead argument.

**`live` records that the reason held against the arguments seen, not that the question is shut.**
A new argument or new evidence reopens any entry here at any time, and that is an ordinary move
rather than a transgression.
What counts as new is the record test of `skill@knowledge-architect-design@threads-and-states`.

**Every entry states a reason a reader can check without leaving the entry, or points at where the
reason is.** _"Refuted by measurement"_ satisfies neither: it asserts that a checkable reason exists
somewhere and does not say where.

**A rejected design that is reopened, chosen and implemented moves out of the home.** The home
must not describe as rejected a design the project now has.

**An entry also leaves when the owner rules that it fails every recording test**, as an audit may
propose, **or when no design head cites it any longer and no head of absence is written for it**,
as when the heads that cited it leave the design home for a line beside the instruction they state:
that line then names the design and the reason it was not kept, so a session that reopens the
instruction still meets it.

## Writing a tripwire the owner ruled on `##ruled-tripwires`

A design discussion that runs a premortem ends with it, and the owner rules on which of its
surviving causes become tripwires, each by the label, `T<n>`, it was put to the owner under. **A
tripwire is written only on the owner's word, and with the decision it guards**, whatever named
it, a parked thread included: in the change that records that decision, which on the full path of
`skill@knowledge-architect-design` is the harvest, or in a later change once that decision is
recorded. It goes in the tripwires home of the Component that owns the guarded decision, so the
decision's head exists before the tripwire that names it. A parked thread's tripwire whose ruling
records no decision, as when the owner defers the question, is instead the trigger of a `deferred`
issue. Its shape and its lifecycle are `skill@knowledge-architect-issue-tracking`.

## Before you finish `##before-you-finish`

```sh
cargo klarch check
```

Every reference you wrote must resolve.

Then re-read each head you wrote: present tense, as if the design had always been so (`primer@design-heads`).

Then apply the tests of `primer@design-heads` again to each decision of the change, and to each
head it writes or extends, now that every text of it is written: a site the change itself wrote counts, a skill or a restatement of the project's own
included. A decision judged while its texts were still being written misses the second site the
same change adds.

Then, for each head the change edits, whatever the edit, run `cargo klarch show` on it and read
every text that cites it, and re-read the head's whole body, its argument included, against the
change; repair in the change each text it leaves false. A reference is a claim that its text is
revisited when the entry changes, and an edit that keeps the slug dangles no reference, so no check
reports a citing text it made false.

Then, for each attribution of the owner the change writes, read the owner's words beside it, from
the plan document or the transcript, and check that it keeps their scope and every hedge, as
`primer@design-heads` asks.

Then re-read what you wrote against the head you replaced: **rewriting argued text is where
fidelity gets lost.** If you cannot restate a rejected design as strongly as it was written, you
have not understood it well enough to move it.
