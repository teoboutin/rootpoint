---
name: knowledge-architect-goal-setting
description: MUST use when a Component has no goal, when the owner states a new purpose or abandons one, when a recorded decision conflicts with a goal, and before writing or editing any goals home. Covers what a goal is and how it differs from a decision, how the owner's intent is drawn out and written as a draft the owner rules on goal by goal, the shape of a goal entry, how a Component's goal names the project goal it refines, where each goal sits, and why a goal leaves only on the owner's word.
---

# Setting goals

Scope: the goals homes of the project, one per Component, `register@*@goal`. Writing
a goal, changing its wording, and removing it.

Not covered here: **a decision about how something is built**, which is
`skill@knowledge-architect-decision-recording`; **setting up the rest of a Component**,
`skill@knowledge-architect-setup`.

**The goals are the only statements assumed to come from the owner**, per `primer@goals-bind`. Every design decision binds
as a presumption, and a goal binds outright: a constraint derived from a goal rules a proposal out.
That is why an agent never edits a goal outside this skill.

## A goal, and what is not one `##goal-or-not`

**A goal is met or unmet. A decision is won or lost.** A goal states an outcome: what the project or
the Component is for, and for whom. It never states a mechanism. The test, for each sentence: if it
changed, would the Component be for something else (a goal), or would it reach the same end another
way (a decision, which belongs in the design home)?

A goal stays in its home **while it is met**. One removed when it is achieved stops being checked,
and can stop being met without anyone noticing. **A goal leaves only when the owner abandons it**,
on the owner's word.

## The shape of a goal entry `##goal-entry-shape`

A level-two heading stating the goal as a sentence, its slug at the end, then one short paragraph:
what the goal means, and **what would show it is met**. The paragraph stays short because a goals
home is read whenever a decision is argued from one. Every level-two heading of a goals home is a
goal and carries a slug: the head that says what the home holds is prose under the level-one title,
never a level-two section.

```markdown
## <The goal, stated as a sentence> `##<slug>`

<What it means, in two to four sentences, and what would show it is met.>
```

**Where a goal goes.** A goal is written in the goals home of the Component whose responsibility it
is to fulfil it, even when other Components' decisions serve it too: any goal can be referenced from
anywhere. The root's goals state what the project provides to its consumers. A published Component
serves those consumers, so its goals are encouraged to be sub-goals: more specific than a root
goal, perhaps not stated by it, and still to be fulfilled. A Component that serves only the
project, such as a maintenance tool, serves every root goal at once, and its goals need not refine
one. Every Component
states at least one goal.

A Component's goal that refines a goal of the project's root names it, with a reference in its
body: `goal@<root anchor>@<slug>`. Then `cargo klarch show goal@<root anchor>@<slug>` lists every
Component goal that refines it, and removing the root goal leaves each of those references dangling,
which the check reports.

The head of the goals home says what it holds: a goal is met or unmet, it stays while it is met,
and it leaves only on the owner's word.

## Drawing out the owner's intent `##drawing-out-intent`

1. **Ask the owner to state their intent** for the Component, and any goals they already have, in
   their own words. Three questions help: what is it for, and for whom; what would show that it
   fails at that; what is it explicitly not for.
2. **Refine the wording** of what the owner stated. **Propose further goals** from the existing
   documentation, or from the code and content when the documentation does not say enough.
3. **Write a draft that goes into the goals home verbatim if it is approved**.
   **Mark each goal with its source**: "the owner's statement", or "proposed from <the document or
   the code it was read from>". The source mark is not written to the goals home.
4. **Ask the owner to read the draft in full, and to rule on each goal by its slug**: approved, or
   dropped. A goal with no ruling is neither written nor dropped: ask again for it by its slug. A
   goal the owner rewords is shown again in its new wording before it is written.
5. Write the approved goals, exactly as approved, and the home's head, and nothing else.

**A goal need not be met yet.** A goal is the owner's intent about where the project should get to,
and it constrains future work and design from the moment it is written. When nothing fulfils it yet and
no plan document schedules the work that would, open a `todo` issue for that work, under
`skill@knowledge-architect-issue-tracking`, and reference the goal from it.

## When this runs again `##when-it-runs-again`

- **The owner states a new purpose, abandons one, or rewords one.** A goal is added or reworded
  through `skill@knowledge-architect-goal-setting@drawing-out-intent`. When one is removed, every reference to it dangles, and `cargo klarch check` lists each
  one: each is a text that derived something from the goal, and is read again.
- **A decision conflicts with a goal**, and the primer's rule, `primer@goals-bind`, sends the conflict to the owner. The
  goal prevails until the owner rules; if the ruling changes the goal, it changes through `skill@knowledge-architect-goal-setting@drawing-out-intent`.

The goals of one round of rulings are written in one commit, whose message records each ruling,
attributed to the owner, as `primer@design-heads` says. A
goal's removal and the repair of every reference it leaves dangling are one commit, so the commit
passes the check.
