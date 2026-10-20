# The knowledge-architect workflow

This project keeps its design record with knowledge-architect. Its documents are checked by
`cargo klarch check`: every reference between records resolves, every record has the shape its
register declares, and every generated listing is current. A register is one kind of record, such
as the design decisions or the issues, and its home is the file or the directory that stores its
entries; the two words stay apart because a rule can need both, as "a slug is defined in its
register's home". This primer is installed, and the project's root `CLAUDE.md` imports it. The
project's own rules are in its root `CLAUDE.md`, beside this primer, and add to it.

## Goals bind; decisions bind as a presumption `##goals-bind`

**The goals are the only statements assumed to come from the owner.** Each Component states them
in its goals home, `register@*@goal`, one heading per goal. A recorded decision was reviewed, but its review can
miss a detail or an implication, more often as the volume of agentic work grows. A decision binds
as a presumption, which a better argument may rebut, and reversing one is an ordinary move with a
procedure (`skill@knowledge-architect-decision-recording`). **When a decision conflicts with a goal,
the likely cause is that the owner missed the conflict: the goal prevails, and the conflict goes to
the owner.**

## A question goes to the authority that answers it `##question-authority`

What an external specification the project implements says is answered by its text. What the code
or a tool does is answered by a measurement. What follows from the goals and the recorded decisions
is answered by the argument from them. A question one of these answers is answered by the session,
with its source cited, and the owner is told, not asked.

**The owner answers what none of them settles**: a goal to set or change; a fact only the owner
holds, such as an intent, a plan or a weighing the owner made; a contradiction between the owner's
words, the goals and the record that no argument resolves; an action that cannot be undone; and a
fork, where two or more shapes meet every binding constraint and no fact defeats
either, so that choosing between them weighs a cost or a scope that no goal or decision ranks. A
cost alone makes no fork: a shape that fails a goal, or loses to a fact, is out, whatever it would
save. A question put to the owner names the authorities consulted and, for a fork, each surviving
shape and what it trades. **A default put to the owner is built, and the branch that builds it
merges once the owner has ruled on it**: the work goes on with no wait, and the owner rules before
the default reaches the main branch, at its merge at the latest. Which further actions a project
reserves to its owner, such as a merge, is the project's to state. **Where an installed instruction sends to the owner a question this
section gives the session, this section prevails**.

## The owner's word and its premise `##owner-word-premise`

**A word of the owner holds only as far as its premise.** A ruling given on a premise, stated by
the owner or supplied by the session, does not rule on the case where that premise is false. A
premise the session supplied is the session's to check before the owner rules on it.

**An answer of the owner on a reading or a fact gives way to its source.** Where the owner answered
what a specification's text says, or what the code or a tool does, and the text or a measurement
says otherwise, the source applies at once: the work and the records follow it, and the owner is
told at the top of the turn, with the source cited, and in the turn's "Decided" section, per
`primer@decided-section`. A part of the work that cannot be undone still waits for the owner's
answer. Whether the project follows a specification, and how closely, is the project's own goal or
decision: a departure from a specification is a scope decision the project states, never inferred
from an answer to a reading question.

**A ruling on intent whose premise proves false is a contradiction.** A ruling on intent is a word
of the owner on a question `primer@question-authority` reserves to the owner. Put the corrected
premise to the owner at the top of the turn, quoting the word it defeats; the part of the work that
rests on it waits for the answer, and the rest goes on.

## Items put to the owner under a label `##labelled-items`

**An item put to the owner under a label**, `Q<n>` or the prefix a skill gives it, **takes the
next number of its prefix in the record the label reaches, from 1, and that number is never reused
there.** That record is a plan document, a milestone's documents counting as one; a file; or the
commit messages of one branch, counting as one. A label carried later with its item into another
record, as a tripwire's into a tripwires home, keeps its number, and that record numbers nothing.
A label that reaches none is numbered within the message that puts it to the owner, wherever the
item's ruling is kept.

**A label numbers what was put to the owner; it never names what decided.** An answer on a question
another authority answers, per `primer@question-authority`, is recorded as a reading or a fact with
its source, never as the owner's ruling. Where the source leaves the point open and the owner chose,
the record says that the owner chose among the readings the source permits.

## Room to judge `##room-to-judge`

**Where the installed text is silent, judge.** It leaves that room on purpose: it states the
instructions the workflow needs and the reasons for them, and leaves the rest to the session. Room
to judge is never room to act against an instruction.

## Intent and claims `##intent-and-claims`

- **A design home is built intent**: the design as built and its reasons, and the decisions that
  no work implements and that are not part of any spec, recorded when made. Design that is decided and not built is in a plan
  document until it lands. Check the code against a design home, never the other way. A divergence
  is a defect in one of them: say which. Where the head stands, its argument or the owner's ground
  undefeated, the code is the defect: fix it, or open an issue. Where the session finds the head's
  argument defeated, the head's
  reversal goes to `skill@knowledge-architect-design`, and to the owner where reversing it is a
  fork or the head's ground is the owner's; the part of the task that rests on the point in
  divergence waits for that answer, and the rest of the task goes on. A design home can be wrong,
  and it still prevails over the code until the divergence closes. It closes when
  the code changes to meet the head, or when the head is reversed under
  `skill@knowledge-architect-decision-recording`; it never closes by following the code. Work on
  that point that goes on meanwhile builds on the head.
- **A reason recorded at the code** (an inline comment saying why the code is shaped so, or the
  message of the commit that argued it) is intent at the scale of that code. It binds as a
  presumption, below the design home: a change that defeats it argues against that reason in its
  own commit message, and where it conflicts with a design home, the design home prevails, as in
  any divergence. Between a comment and a commit message, the comment is the current statement.
  **Before removing or reshaping code as unneeded, read its comment and the message of the commit
  that introduced it** (`git log -L`, `git blame`): absence from the design home is not evidence
  that code is superfluous.

## Verify before relying `##verify-before-relying`

A document can describe the code as it was committed rather than as it is. Confidence is not
correlated with correctness: the more obvious a behaviour looks, the less likely anyone has
checked it. An agent tends to assume that the project and its tools behave as their names, their
documents and its own expectations suggest, and to act on that assumption without checking it.

Before relying on a claim about how anything behaves (an API, a tool, a script, a file format, a
test mechanism, an issue's diagnosis; the list is illustrative, not a boundary):

- **Establish it from the implementation, or from a run you observe.** Names, doc comments and
  prose are leads to check, never evidence. A scoped `CLAUDE.md` invariant, a doc comment and an
  issue's diagnosis go stale.
- **Follow the whole path, not the entry point**: what calls it, what it returns on success _and_
  on failure, and what the caller does with that.
- **One confirming case does not establish a convention.** Widen the check whenever the answer
  generalises.
- **Report the evidence with the conclusion**, and label anything unverified as an assumption. A
  plausible mechanism is not a finding.
- **Re-verify a conclusion before acting on it**, and correct it in place the moment evidence
  contradicts it.
- **A transfer from another project names the property it depends on, and checks that the property
  holds here.** Naming the pattern is not enough. A claim taken from another project's record,
  such as a count of occurrences, is checked against that project's registers as well as its
  history: its rejected designs and its issues often hold what its commit log does not.

**Evidence is textual and cheap**: a verbatim reading of a file, a measurement over a tree, a
failing test. Build it rather than trading intuitions.

### Verify a claim before writing it

**A checkable claim is checked before it is written down**, against the tree at the moment the
sentence is written:

- a number comes from a measurement you can point to, with the machine named; a figure whose
  instrument the same change altered is taken again;
- a count is re-counted: files, lines, entries, references, occurrences;
- "a search returns nothing" and "the only hit in the tree" are run, against the revision the
  sentence describes, so that they match across a line break, since wrapped prose splits a phrase
  over two lines;
- "X already says this" and "the skill requires that": open X and read it;
- a slug, a path, a section or a run identifier is resolved;
- "this is fixed" and "that entry is closed" are diffed.

**A commit message is a document for this purpose.** It is durable, and `cargo klarch commits`
reads only its pointers, its references, its paths, its skill names and its SHAs: a measurement, a count or a claim about the tree in a message is checked by nobody but
its writer. Before each commit, list every factual claim its message makes: a count, a search and
its result, a section or line number, the name of a file, a slug or a commit, a quotation or an
attribution of the owner, or a statement that something exists, was searched, was found or
changed. Beside each, write the command or the file that checks it; run or open each against the
tree the commit holds, and check a quotation or an attribution against the conversation. A claim no
check supports is removed, or labelled as an assumption. Writing the message from memory of the
work is where the false claims come from.

**The same checks apply to an answer given to the owner in the conversation.**

### Check whether it is already known

Before diagnosing anything as a problem, or reporting one met while doing something else, find
out whether it is already recorded: `cargo klarch issues`, `cargo klarch tripwires`, and
`cargo klarch show <kind>@<anchor>@<id>` for one entry and every reference to it. A recorded entry
usually says more than a fresh diagnosis: the measurement already taken, what was ruled out, and
often why the work was left undone on purpose.

### Precedent is not authority

The instructions that apply are the installed ones, the project's root `CLAUDE.md`, its skills,
and the scoped `CLAUDE.md` next to what is edited. What already exists in the project is evidence
of what was done, not of what should be done.

- **"The existing ones do it this way" is not an argument, and neither is how many of them do.**
  Where an instruction and the surrounding practice disagree, the instruction wins. Where you
  think an instruction is wrong, say so and argue it; changing an instruction is a normal move, and
  silently following the older practice is not.
- **Match local idiom only where the instructions are silent.** Naming and the shape of the
  surrounding prose are worth matching, so a diff stays readable.
- **Do not spread non-conformance while working around it.** Starting from an existing document or
  entry copies its defects with it: read what you copied against the instructions before extending
  it.

## Working prose `##working-prose`

These rules govern working prose: every text the project's contributors read, such as a
discussion with the owner, a document of the project, a commit message and a code comment. Text
the product's end users read follows the product's own style contract, where literary value can be
a deliberate goal. The test is who reads the
text, not where it is stored.

- Write plain, direct technical English, in short sentences. Readers might be non-native
  speakers: where a simpler word or a shorter sentence says the same thing, use it.
- Use precise technical terms. No idioms, colloquialisms or aphorisms: they read differently to
  different readers, and they carry a qualitative judgement past argument.
- No undefined metaphor: the meaning is carried explicitly. A metaphor that names a defined
  concept is vocabulary once defined.
- Use em dashes and semicolons sparingly. They usually join two sentences that read better apart.
- Prefer explicit quantities, units and invariants over qualitative wording. A qualitative claim
  may state a goal or an observation. Offered as an argument, it is replaced by its checkable form:
  a count, a bound, an invariant, a named failure it prevents, or a test either party could run.
  The test: delete the sentence; if the argument falls, the sentence was the argument, and it is
  replaced by its checkable form, not reworded in other qualitative terms. An owner's ruling needs
  no such form.

## Something met outside the task `##met-outside-the-task`

Something met while doing other work takes the first of these that applies:

| # | test | outcome |
| --- | --- | --- |
| 1 | it bears on the current work: the work's result, or a decision it rests on, is wrong or incomplete without it | route it as `primer@question-authority` says: what a source, a measurement or the record answers, the session decides and reports; a fork, a goal to set or change, or a fact only the owner holds, goes to the owner at the top of the turn, with a default the work proceeds on unless the owner answers otherwise, a goal standing as it is meanwhile; a contradiction, or an action that cannot be undone, holds the part of the work that rests on it until the owner's answer; a divergence between a design home and the code is routed as `primer@intent-and-claims` says |
| 2 | its fix is checkable from the diff alone: it changes no behaviour, no decision and no test outcome (a typo, a stale pointer, wording that is now false, a broken link) | fix it, in a commit of its own |
| 3 | its `Why it matters` and its `What would close it` can be written | open an issue entry (`skill@knowledge-architect-issue-tracking`) |
| 4 | none of the above | name it, with why it is dropped |

**A turn that met anything outside its task ends with a section titled "Met outside the task"**,
listing every item with its outcome: fixed (with the commit), decided (with its ground, also
listed under "Decided"), issue opened (with its id), waiting for the owner's ruling, or dropped (with
the reason). A mention inside other prose does not count.
A subagent that edits nothing reports each item at the end of its report, under a heading "Met
outside the task", and the session that dispatched it routes each item by this table.

## Decisions reported at the end of a turn `##decided-section`

**A turn in which the session decided something the owner could have decided ends with a section
titled "Decided"**, before "Met outside the task" where both are owed: one line per decision, each
with its ground, a reference or a source. A design thread closed on its argument, with the rivals
it rules out, a default the
record answered, a source applied over an answer of the owner, and a rewording of a tripwire's
clauses are such decisions; an implementation choice that a commit message records is not. A
subagent reports its decisions at the end of its report, under the same title, and the session that
dispatched it carries them into its own section.

## The project's documents and how they are checked `##checker-model`

**The manifest**, knowledge-architect.toml at the project's root, declares the Components, the
locations, the registers, and what is exempt from the checks. `cargo klarch anchors` lists what it
resolves to: every anchor with its kind and its path, and, under every anchor but a spec or a
milestone, the home of each register it carries.

**An anchor** is a named place that carries registers. A **Component** is a directory the manifest
declares, named by its basename, or the project's root, named by the manifest's project name; it
carries the required documents,
every component register, and each `opt-in` register whose `components` names it. A **location** carries only the registers it declares, their homes
directly under its path, or at its path for a register declared with `dir = "."`. The checker
constructs the anchor `plans` at docs/plans/ and one anchor per spec file, and, under the home of
every directory register, wherever that home sits, one anchor per entry directory, each milestone
included. The anchor words `*` (every Component), `elsewhere` (a
path the tree does not hold) and `plans` are reserved.

**Every Component carries** `path@*@README.md`, `path@*@CLAUDE.md` while the project serves the
`claude` harness, and the home of each component register: `register@*@goal`,
`register@*@design`, `register@*@rejected`, `register@*@tripwire` and `register@*@issue`.

**A register's entries**:

- a **heading register**, such as `design`, `rejected`, `goal` and `tripwire`, keeps its home in
  one file, or in a directory behind a `README.md` that links every subdocument. An entry is a
  heading at the register's level, its statement first and its slug last: level 3 for `design` and
  `rejected`, level 2 for `goal` and `tripwire`;
- a **file register**, such as `issue`, keeps its home in a directory: one file per entry, whose
  basename is the entry's id, beside a hand-written `README.md` and a generated `index.md`;
- a **directory register**, such as `milestone`, keeps its home in a directory: one directory per
  entry, holding a `README.md`, beside a hand-written `README.md` and a generated `index.md`. Each
  entry is an anchor named by its basename, carrying at its own directory the register that a
  declared directory register's `carries` names, so an entry of it is cited `<kind>@<entry>@<id>`;
- a **plan document** is an entry of `spec` or of `milestone`, and a plan's items, of `thread`,
  `argument`, `criterion` and `acceptance`, are level-three headings under the section of their
  kind.

An id matches `[a-z0-9]+(-[a-z0-9]+)+`: two segments or more.

**A reference** is one backticked span, `<kind>@<anchor>@<id>`. The kind is a register's name, a built-in
one or one the manifest declares, and the reference names one entry of it; or `register`, which names a register's home; or `path`, which
names a file or a directory under the anchor; or `planned`, which names, in a plan document, a
path its work will create. The harness kinds take no anchor: `skill@<name>` and `agent@<name>`,
each with an optional section slug after a second `@`, `primer@<slug>`, and `instructions@<slug>`
for a section of the root `CLAUDE.md`. When a reference is written, and how a `path` reference is
shaped, are in `primer@where-knowledge-goes`.

**Reading**: `cargo klarch show <reference>` prints the entry whole and every reference to it;
`cargo klarch issues` and `cargo klarch tripwires` list the standing entries.

**Running the checker**:

- `cargo klarch check` reads every document in four phases (the manifest resolved, the tree read
  against it, the entity table built, then every check) and stops at the first phase that finds
  anything, saying which phases it did not judge: a finding computed over an incomplete model is
  unreliable in both directions. Its exit code is the verdict: 0 for no finding, 1 for findings, 2 when it could not
  run. Its last line states it, `PASSED: no findings` or `FAILED: n findings above`; its summary
  block prints whether or not it found anything, so the middle of its output does not tell whether
  it passed. A finding names its repair.
- `cargo klarch check --staged` judges the tree a commit will record, HEAD's with the staged
  changes. Run it after staging and before each commit, and repair a failing one before the
  commit: it catches a file left unstaged, or a change staged in part, before the commit exists,
  where the gates catch it only before the merge.
- `cargo klarch commits <range>` judges each commit of a range, its message and its tree: a commit
  message is a document, read long after it is written, and every reference in it must resolve,
  against its own commit's tree or its first parent's.
- Never chain a command on a verdict that went through a pipe: in `check | tail && git commit`,
  the shell takes the exit status of `tail`, so the commit runs whatever the check found. Run the
  checker bare, or redirect its output to a file, read its last line, and chain nothing on the
  redirection's command.

## Where knowledge goes `##where-knowledge-goes`

**Every durable decision has exactly one home.** A second mention of a decision is a reference to
it, never a copy, because a copy starts drifting the moment it is written. **A directive is
restated where it has to be delivered only when the restatement is no longer than a pointer to
it**: a path, a file name, a command, a value, or one sentence. A directive sentence carries its
pointer beside it, and where the two disagree the restatement is the defect. A directive longer
than one sentence is delivered by a pointer to its home, with an instruction to read the home whole
at that moment, or by one of its sentences restated with its pointer beside it, where that one
sentence is what the point of delivery needs. More than one sentence of it is never restated: a
partial copy drifts, and its reader takes it for the whole. For installed text, the home a pointer
names is installed text, a skill's section or the primer's, since installed text cites no entry of
the project. An existing longer restatement is converted when a change edits what it says;
re-pointing a reference in it does not count. Until then, its length alone is no defect, whenever
it was written.

| the statement is about | home | it leaves when |
| --- | --- | --- |
| what the project, or one Component, is for, and what would show it achieved | that Component's goals home, `register@*@goal` (the project's root is a Component) | the owner abandons the goal |
| how the project or a Component is built, and why | that Component's design home, `register@*@design` | the design changes: the entry is rewritten in place |
| a design not kept, and why | that Component's `register@*@rejected`, each entry cited by the design heads whose argument covers it | it is reopened, chosen and built; the owner rules it fails every recording test; or no design head cites it any longer and no head of absence is written for it; a reversal moves the reversed decision into it if it meets a recording test of `skill@knowledge-architect-decision-recording` |
| what is outstanding: a defect, an unexplained observation, an open question, missing work | one file in the owning anchor's issue directory, `register@*@issue` in a Component | the issue closes |
| evidence that would flip a recorded decision whose work exists, or that no work implements | the owning Component's tripwires home, `register@*@tripwire` | it fires, or its decision is gone |
| a contract or a trap that only a developer needs, true of the code as it stands | the scoped `CLAUDE.md` nearest the code | the contract changes or the trap is removed |
| how a user can use a Component, and what to respect | its `README.md` | the contract changes |
| directions about what to find where in a directory | a `README.md` in that directory | the directory's content changes |
| what a caller must respect to use a type or a function | that item's doc comment | its contract changes |
| why a piece of code is shaped the way it is, and where that holds | an inline comment at that code | that code changes |
| work that is designed and not built: a spec or a milestone | the plans directory, docs/plans/ at the project's root: a spec in docs/plans/specs/, a milestone in docs/plans/milestones/ | the work lands |
| the evidence of one pass of a project audit, and the owner's ruling on each of its items | one file per pass in docs/audit-reports/ at the project's root, as `skill@knowledge-architect-project-audit@record-of-an-audit` says | the commit that carries out its last outcome, or a later commit of the same branch, deletes it |
| the analysis of a retrospective's project file, and the owner's ruling on each of its findings | one file per analysed file in docs/retrospective-reports/ at the project's root, as `skill@knowledge-architect-retrospective-intake@analysis-and-ruling` says | the commit that carries out its last outcome, or a later commit of the same branch, deletes it |
| the order in which the owner wants known work done | docs/roadmap.md at the project's root, optional: each row cites an issue entry or a plan document | a row leaves when its plan document leaves, or when its issue closes with no plan document scheduling the work |
| how to perform an activity | the owning skill | the procedure changes |
| **none of these, nor a row of the project's own** | **ask the owner before writing it anywhere** | the table gains the row |

The project's root `CLAUDE.md` adds its own rows. **A measurement is routed by what it serves**:
the head of the decision it supports, the issue entry it characterises, the tripwire whose
threshold it is, or else the commit message that took it.

**A reference is written wherever the text would have to be revisited if the entry it names
changed**: a design head names the goal it derives from, a decision of another Component it
depends on, and the `rejected` entries its argument covers; an issue names the decision it strains and the goal it threatens, when it does
directly; a guard, a
workaround, a stub or a test that exists because of an issue names it in the comment at the site; a
tripwire names its decision; a commit message
names every entry it opens, closes, reverses or argues from. A reference whose entry's change would
leave the text unaffected is not written. A reference, whose kinds
`primer@checker-model` lists, names the anchor that defines the entry, and it is live wherever it is
prose, a fenced block included; an illustration that must not resolve writes a placeholder in angle
brackets. A span never crosses a line break: a line breaks before the span, not inside it. A skill
or an agent is named by its reference, `skill@<name>` or `agent@<name>`, never by its bare name in
backticks. A `path` reference names the deepest anchor that holds its target, with a plain path
under it, no `..`, no `./` and no leading `/`, and ends with `/` exactly when the target is a
directory; an anchor's own directory is named from the anchor above it, as
`path@<parent-anchor>@<dir>/`. A relative markdown link, in any of markdown's link shapes, an
image `![alt](<path>)` included, is legal in a `README.md` or an `index.md` only, to a target under
its own directory, its path under the same constraints as a `path` reference's; any other pointer is
a backticked reference.
A heading is a line markdown reads as an ATX heading: one to six `#` marks, at most three spaces
in, then a space or a tab and text, with no list or block-quote marker before them; a line
markdown reads as a heading in another shape, as a `---` right under a paragraph is, stops the
check. The checker reads Markdown, and in Rust source the comments and every string
literal that a `let`, a `const` or a `static` does not bind to a name, a function call's argument
being never bound and a macro's body bound like the rest of the value; it reads each commit's
message and tree through `cargo klarch commits`. A reference anywhere else is found by grep.

**A finding is repaired in a form the checker judges, never by moving the pointer into plain
text**: the right anchor, `path@elsewhere@<path>` for a path the tree does not hold, a placeholder,
or a rewritten sentence. A record of a deliberation (a plan document, a committed file that leaves
once its outcomes are carried out, or a commit message) names by its bare slug in backticks an
entry that the work it records removes, renames or is still to create, and by its bare name in
backticks a skill or an agent that it removes or is still to create, where its tree does not hold it, nor, for a commit message, its first parent's: a name, not a reference, since there is nothing
for the checker to judge. A renamed entry's old slug carries beside it the reference to the entry
under its new slug, so a reader reaches the live entry by a checked pointer. A reference or a path that no checked form expresses is
written without backticks only beside a reference to an issue entry of this project that records
the missing form.
A gap of the checker itself gets that entry in this project's own register, since a reference
cannot reach another project. Text that is not in the checker's syntax, such as a commit named by
its subject, a commit of another project or a description in words, is outside this rule.

## Every commit of a branch is judged by the checker at its tip `##branch-commits`

Where the project's gates run `cargo klarch commits` over a branch, as
`skill@knowledge-architect-setup@setup-gates` recommends, that run is the **per-commit gate**: it
judges every commit of the range, its message and its tree, with the checker the branch's tip
runs, and a tree that holds no manifest or has a finding under that checker fails it. **A change
that sets what the tip's checker requires of every tree, such as adding the manifest, moving the
pin, or, in a project that builds the checker, a change to it under which a tree that passed
before can fail, goes in the branch's first commit, with every fix the tree needs to pass it, or
the branch is squashed to one commit before its review. A repair that would leave an earlier
commit failing is folded into the earliest commit it repairs**,
by a history edit with no uncommitted work in the tree, confirmed afterwards to have lost no
content.

## Design heads `##design-heads`

A design head is an entry of a design home: a level-three heading that states a decision, and its
body. Read this section to read a head, to tell in advance whether a change bears on one, and
before writing or judging one; `skill@knowledge-architect-decision-recording` holds the procedures
of recording.

### What earns a head

Most implementation choices earn no head. A unit of work produces dozens of them, and a design home that
records dozens per unit of work stops being readable and stops being ranked.

**A decision earns an entry in a design home only if at least one of these holds:**

1. reversing it would change **an interface others consume**: a type or a signature that crosses
   the boundary of a separately built unit (a crate, a package, a library, a module others import),
   a file format, a document grammar, a command line;
2. **the same reason must be respected at more than one site, or at none.** A reason is an argument
   against a rival someone could plausibly propose; a name, a path or a value repeated for
   consistency is not one, since nothing argues for changing it and a stale copy is found by a
   search. At more than one site, the reason needs a home each site can cite, and a comment cannot
   be cited. At none: a decision about an absence ("we do not do X"), or a policy that no code and
   no text of the project states. For a decision about agent-facing text, a skill, an agent or a
   `CLAUDE.md`, each text that states the instruction is one site, a design head of the project
   that restates it included, other than the decision's own: a decision stated by one text
   has one site, and its reason lives in that text, beside the instruction; or
3. **its argument turns on the behaviour of something outside the project**: an external
   specification the project implements (a standard, a protocol, a rule set), or an external tool's
   behaviour, read in its documentation or measured; or
4. **the owner confirms that it records the owner's own intent**: a ruling the owner gave that an
   agent could judge superfluous and reverse as a small fix, or an argument the owner made and wants
   kept so as not to restate it. The agent does not judge this. When a decision would earn a head
   by this test alone, ask the owner, one question per decision, each under a label, `Q<n>`,
   numbered as `primer@labelled-items` says, several in one message, each quoting the owner's words
   the decision rests on: do they record the owner's intent, or were they an answer to the agent's
   proposal, or a hedged statement ("I think", "maybe"), which is a position to argue under
   `skill@knowledge-architect-design`? The head attributes the decision to the owner, with the
   owner's answer, as the next section says.
   With no answer, the decision earns no head by this test.

A choice that turns on what the specification means, or on how the tool behaves, is expensive to
get wrong and expensive to derive again, and it is visible: the argument quotes the specification
or the documentation, or names the measurement.

Otherwise it belongs in an **inline comment at the code it explains, plus the commit message**.
The comment is read by every session that touches the code, which a head is not, and the commit
carries the argument. A reason that fits in one comment at one site, about that site's own code,
fails test 2.

**The backstop. A decision that creates a head, contradicts a statement of one, its argument
included, or adds a member the head's argument does not cover, goes to the design skill before its
text is written**, unless it was argued there, under `skill@knowledge-architect-design`. This is the case of
a decision met during another task and settled there, by the owner's word or by the session's own
choice. The design skill's in-change path keeps the deliberation in the commit message, so the task
needs no plan document and no new session. An addition that answers another question than the
head's takes a head of its own, by the test of one decision per head below. A member the head's
argument covers is recorded directly, routed as the next paragraph says; where the owner approved
it, the approval is recorded in the commit, attributed to the owner, not in the head. A change that relocates or rewords
recorded decisions, a split of a head included, and adds or removes none, is not a decision: it
needs no design skill, and the routing and fidelity-of-relocation review axes judge that it adds or
removes none.

**A new member of a head is routed by the head's ground.** A member is what a head's rule admits
and its body does not yet name. Where the part of the head that admits it stands on its argument,
and the argument covers the member, it is recorded directly. Where that part's ground is the
owner's words and they state the rule, the member is within the ruling and is recorded directly,
the ground attributed to the owner as before. Where the owner's words name the members, it goes to the owner as one
proposal: one message stating the rule, the member and a default, and nothing of it is written
before the owner's word; it is no change of the decision, so it owes no reversal. A member the rule
does not admit, or that the argument does not cover, is a change of the decision, and goes to the
design skill, as the backstop says.

### What a head holds

**The standing argument** is every premise whose failure would reopen the decision: the goal it
derives a constraint from and a decision of another Component it depends on, as references; the
measurement it rests on, with the command that takes it again; the fact that defeated its nearest
rival. A constraint from a goal binds outright, where one from a decision is a presumption, and the
reference is what tells a reader which. **The test: if this premise turned false, would the
decision have to be argued again? If yes, it is in the head.** If no, it is deliberation.

**A head stands on its argument, and cites the owner only for what came from the owner.** Most
decisions are proposed, argued over their costs and rivals, and approved by the owner. The head
records such a decision on that argument, with its costs and rivals in its body or in the
`rejected` entries it cites. An approval is not a ground, whatever its words ("approved", "agreed", "all defaults
approved", "accepted the cost"). It stays in the deliberation: the plan document or the commit
message. The owner's words are a ground, attributed to the owner, only where the decision came from
the owner: the owner proposed it, the owner chose where the argued rivals did not settle it, or it
rests on a premise only the owner can state, such as an intent, a plan or a weighing the owner
made. Test 4 is such a case. In a head that holds a part of each, the part that came from the owner says so, and
the rest stands on its argument.

**A durable record attributes a decision to the owner; it does not quote the owner.** A head, a
commit message, a tripwire, a `rejected` entry and an issue entry state what the owner decided and how far the
decision reaches, as "the owner chose X over Y" or "on the owner's word", never the owner's words
verbatim. The verbatim words stay where the deliberation is kept: a plan document's items, and a
committed file that leaves once its outcomes are carried out. An attribution keeps the scope and the
hedges of the owner's words, and is checked, like a quotation, against the question it answered.

**What the ground means to a later session.** A head that stands on its argument is reversed by a
better argument, which the owner approves where the reversal is a fork, per
`primer@question-authority`. A head whose ground is the owner's words is
reversed by the owner; a defeated premise of those words goes to the owner, as
`primer@owner-word-premise` says. An argument against it goes to the
owner as a question about their intent.

**The deliberation is not copied into the head.** Where it is kept, and how a reader finds it,
is `skill@knowledge-architect-decision-recording@three-homes`.

A head is written **as if the design had always been so**. Present tense, no dates, no "formerly", no
account of the change. If you find yourself writing "we used to…", that sentence belongs in the
commit, and so does an opening that motivates a decision by describing the state before it.

**A head carries intent, shape and the standing argument, not implementation.** What the project is
for, and how it is arranged in order to get there, belongs here. How a particular function does its
work belongs in a comment at that function. The test: **if changing a piece of code would force a
change to the design home, it is design; if the design home would be unaffected, it is a comment.**

Never list what cites a head: `cargo klarch show design@<component>@<slug>` computes it.

A decision that relies on the checker of this workflow states that it relies on the checker
working as intended. It cannot reference the checker's own decisions: a reference resolves only
against the project that holds it.

### How a head is shaped

Give the decision a **slug anchor**: a short hyphenated name in backticks, prefixed with `##`. It
goes at the **end of a level-three heading**, the level the design register declares, so that the
outline reads as decisions under level-two subjects. **Every level-three heading in a design home
is an entry** and carries a slug: one without is a finding, so a heading that is section text sits
at level two or four. Nowhere else: a slug at another heading level, in a table cell, at the head
of a plain line, in the middle of a line or in a file that is not the design home defines nothing.
Outside a fenced block, `cargo klarch check` reports it as a misplaced definition; a reference to it
resolves to nothing.
The slug is an id, unique in the design home.

**When the decision was a thread of a design discussion, its slug is the thread's name**, unless
that name misdescribes the decision as approved, as a name for the change it proposed does. The
discussion minted it in the same grammar and checked it for a collision with the Component's
entries before using it, for that reason. When the name misdescribes the decision, the entry takes
a slug that names the decision, by the alignment rule below, and the text that keeps the
deliberation states the pair, `#<thread> → <entry slug>`: the plan document's harvest row, or the
commit message on the in-change path. `git log --grep` on either name then finds a commit message
that states the pair, and `git log -G` a plan document's diff that does.

**A list item is not a definition site either**, so a decision written as one bullet among several
carries no anchor and cannot be cited or found by `git log -G`. This is a constraint on the
document rather than a gap in the checker: **a decision worth a slug is worth its own level-three
heading**. When a section of bulleted arguments produces one, break it out of the list.

The statement comes first and the slug last, with no bold and no em dash between them, so that a
document outline reads as a list of decisions rather than a list of identifiers. The body follows as
ordinary prose.

**A title states a decision only while it is false of the nearest rival it beat.** A title that the
losing alternative would make true names a subject, not a decision: "The configuration is read
once, at start-up" is false of a configuration read again on every request, and "The configuration
is read with care" is true of nearly any rival.

**A title states the rule that decided, not the list of what it admits today.** When a decision
admits members (fixes, kinds, verbs, sections, consumers), or picks a mechanism to meet a
requirement, the title states the property that admits a member, in the argument's own terms, and
the body names the members built as what the rule admits today. A count or a list in a title goes
stale at the next member, and every later member then reads as a change of the decision. A set
closed on purpose keeps its list in the title, with the sentence that argues the closure in the
body. A title states no more than its argument argues.

**A head holds one decision.** Two statements are one decision when they answer one question and
lose to the same nearest rival. A rule's exception, its parameter and its delivery belong to its
head, since none means anything without the rule. Two statements that lose to different nearest
rivals, or whose arguments share no premise, are two decisions. Each takes a head of its own when
it passes an entry test, and a comment at its code when it passes none. A title joined by "and"
over two decisions shows a bundle. Rewording the title to state both does not make them one. A
member of a set the title's rule admits is no decision, so a member missing from a title is no
finding.

**A head written from a decision the owner approved is read against that approval before it is
written.** The title and the body state the position as the owner saw it, in a plan document's
thread or in the discussion of the in-change path. A clause that widens it, narrows it, or drops
part of it can still be false of the nearest rival, and is still not what was approved: it goes to
the owner, as a change of the decision.

**The slug and the title stay aligned with the full scope of the decision**: the slug is often the
only part a reader sees, in a citing document or in code, and the title is what a document outline
shows. A slug or a title that misdescribes its decision misinforms every reader, or undermines the
decision it names, so rename it even when that means rewriting every reference in the project.

**A head that a change touches is brought to these rules in that change**: its title to the rule it
argues, each decision it bundles to a head of its own, its ground to its argument. Re-pointing a
reference in a head, to a renamed entry or a migrated form, does not touch it; adding or removing
one does. Where bringing it
to the rule would widen what the owner's words in it approved, as a title moved from the members
the owner named to the rule, the change goes to the owner as one proposal, as a new member does.

## The installed skills `##installed-skills`

- `skill@knowledge-architect-decision-recording`: before writing into a design home; a design
  decision has been made or reversed.
- `skill@knowledge-architect-issue-tracking`: before diagnosing a problem; parking anything; a
  tripwire fires; work closes an entry.
- `skill@knowledge-architect-design`: a design question has an open solution space; a requested
  change whose design is not settled, to ground it and find out whether it is bounded work;
  whether to keep or change an existing design; a bug trend suggests the design is the problem; a
  discussion is reverting to multiple-choice menus, minimal-solution defaults, or approval-seeking
  closers.
- `skill@knowledge-architect-planning`: a design discussion converged on its full path; a slice of a
  milestone starts or lands; the work of a spec starts or lands; before editing the roadmap.
- `skill@knowledge-architect-review`: before merging to the main branch, or when an
  activity's skill says its work is ready.
- `skill@knowledge-architect-agent-configuration`: before editing a `CLAUDE.md`, a skill or an
  agent; after an install that changes the installed files.
- `skill@knowledge-architect-setup`: the project adopts the workflow, or moves its pin of the
  checker to another version.
- `skill@knowledge-architect-goal-setting`: before writing or editing any goals home; a Component
  has no goal; the owner states or abandons a purpose; a decision conflicts with a goal.
- `skill@knowledge-architect-project-audit`: the owner asks to audit one aspect of the whole
  project; a pin move crosses a version whose Migration entry cites an audit axis; an adopting
  project's existing documents have moved.
- `skill@knowledge-architect-retrospective`: once per session, offered when a branch the session
  worked on merges, a plan document leaves, or the session ends.
- `skill@knowledge-architect-retrospective-intake`: the owner hands the session the project's own
  retrospective files; an outcome of an analysis in docs/retrospective-reports/ is carried out.

A project's own skills add to these, and never replace them.
