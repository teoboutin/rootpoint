export const meta = {
  name: 'knowledge-architect-agentic-workflow-audit',
  description: 'One pass of the agentic-workflow axis of a project audit, from its preflight to a report the session verifies',
  phases: [
    { title: 'preflight' }, { title: 'drafts' }, { title: 'verify' }, { title: 'dedup' },
    { title: 'cluster' }, { title: 'merge' }, { title: 'repairs' }, { title: 'check' },
    { title: 'cross-check' }, { title: 'report' },
  ],
}
// args: {
//   commit      the commit audited
//   scratch     the run's scratch directory; each step and each agent writes under it
//   worktrees   the directory under which the run creates the worktrees
//   history     a git range whose commit messages and added report files record the earlier passes
//   previous    optional: for a re-check, the commit the previous pass audited
//   kept        optional: the path of the kept list, for a re-check
//   counts      optional: { L1 … L9, clusters, repairsPerCause }, each missing key taking its default
//   from        optional: 'verify', 'cluster' or 'repairs', to restart from the files a run wrote
// }
// The lens table. What each lens looks for is in the workflow auditor's definition, its one home.
const LENSES = [
  { id: 'L1', name: 'contradiction', count: 3 },
  {
    id: 'L2', name: 'activity walk', groups: [
      'design, planning, decision recording and issue tracking',
      'development, review and merge',
      'audit and retrospective',
      'setup, pin move, goal setting and agent configuration',
    ],
  },
  { id: 'L3', name: 'two readings', count: 2 },
  { id: 'L4', name: 'provable gap', count: 2 },
  { id: 'L5', name: 'restatement', count: 1 },
  { id: 'L6', name: 'local against installed', count: 2 },
  { id: 'L7', name: 'checker rules', count: 2 },
  { id: 'L8', name: 'verdict alignment', count: 1 },
  { id: 'L9', name: 'need test', count: 2 },
]
const counts = args.counts || {}
const at = (...parts) => parts.join('/')
const lines = (xs) => xs.filter(Boolean).join('\n')
const PATH = { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] }
const IDS = {
  type: 'object',
  properties: { path: { type: 'string' }, ids: { type: 'array', items: { type: 'string' } } },
  required: ['path', 'ids'],
}
const VERDICTS = {
  type: 'object',
  properties: {
    path: { type: 'string' }, verified: { type: 'number' }, refuted: { type: 'number' },
    plausible: { type: 'number' }, bearsOnRuling: { type: 'number' },
    bearsOnTripwire: { type: 'number' }, known: { type: 'number' }, kept: { type: 'number' },
  },
  required: ['path', 'verified', 'refuted', 'plausible', 'bearsOnRuling', 'bearsOnTripwire', 'known', 'kept'],
}
const DONE = {
  type: 'object',
  properties: { done: { type: 'boolean' }, notes: { type: 'string' } },
  required: ['done', 'notes'],
}

// The auditors: one per lens and count, L2's walkers taking its groups in turn.
const auditors = []
const unwalked = []
for (const lens of LENSES) {
  const n = counts[lens.id] === undefined ? (lens.groups ? lens.groups.length : lens.count) : counts[lens.id]
  if (!(n >= 1)) throw new Error(lens.id + ': each lens is dispatched at least once')
  for (let i = 0; i < n; i++) {
    const label = lens.id + '-' + String.fromCharCode(97 + i)
    const group = lens.groups ? lens.groups[i % lens.groups.length] : null
    auditors.push({ label, lens: lens.id + ' ' + lens.name, group })
  }
  if (lens.groups && n < lens.groups.length) unwalked.push(...lens.groups.slice(n))
}
const clusters = counts.clusters === undefined ? 2 : counts.clusters
const perCause = counts.repairsPerCause === undefined ? 1 : counts.repairsPerCause
const reading = at(args.worktrees, 'reading')
const preflightDir = at(args.scratch, 'preflight')
if (unwalked.length) log('L2 groups left unwalked: ' + unwalked.join('; '))

// The corpus is defined in the workflow auditor's section the-corpus, its one home.
const CORPUS = 'The corpus is the one the section the-corpus of the agent definition ' +
  'knowledge-architect-workflow-auditor defines: read that section whole first; the commands ' +
  'that section gives an auditor are not yours. Read the corpus in the worktree ' + reading +
  ', detached at commit ' + args.commit + ', every file whole. You edit nothing in the project, ' +
  'run no command of the checker, and write only in the directory your brief names.'
const SECTION = 'the section agentic-workflow-axis of the skill knowledge-architect-project-audit'
// The cause, its kinds and the repair are stated in the standard of the agent-configuration skill, their one home.
const KINDS = 'What a cause is, its four kinds, what becomes of a finding that fits none, and ' +
  'how a repair is shaped are stated in the section admission-standard of the skill ' +
  'knowledge-architect-agent-configuration; how the repair of a cause is proposed in this run is in ' +
  SECTION + '.'
// The home of this line is the bullet of review-invariants, in the skill knowledge-architect-review,
// that names the files a subagent's copy may predate: the definition, the root CLAUDE.md and the
// primer it imports.
const FROM_DISK = 'Your copies of your definition, of the root CLAUDE.md and of the primer it ' +
  'imports may predate this session: read each in the worktree this brief names, and follow the ' +
  'worktree where your copy differs.'

// Preflight: the worktrees, the kept lists, the issues and tripwires listings and the history.
if (!args.from) {
  phase('preflight')
  const pre = await agent(lines([
    'You set up one pass of the agentic-workflow axis of a project audit. Run these, from the ' +
      'project root, and report what failed:',
    '- for each of these labels, git worktree add --detach ' + at(args.worktrees, '<label>') + ' ' +
      args.commit + ': ' + auditors.map((a) => a.label).join(', ') + ', reading;',
    args.kept ? '- copy the kept list ' + args.kept + ' to kept.md in the directory ' +
      at(args.scratch, '<label>') + ' of each auditor label above, creating it;' : '',
    '- create ' + preflightDir + '; in the worktree ' + reading + ', run the checker command ' +
      'cargo klarch issues and save its output to ' + at(preflightDir, 'issues.txt') + ', and ' +
      'cargo klarch tripwires to ' + at(preflightDir, 'tripwires.txt') + ';',
    '- save the messages of the commits of the range ' + args.history + ', with git log ' +
      '--format="=== %h %s%n%n%b", to ' + at(preflightDir, 'history.txt') + ';',
    '- save the report files of earlier passes that the range added, with git log -p ' +
      '--diff-filter=A --format="=== %h %s" ' + args.history + ' -- docs/audit-reports/, to ' +
      at(preflightDir, 'reports.txt') + '.',
    'Edit nothing else. Return done true only if every command succeeded.',
  ]), { schema: DONE, phase: 'preflight', label: 'preflight' })
  if (!pre || !pre.done) throw new Error('preflight failed: ' + (pre ? pre.notes : 'no return'))
}

const draftBrief = (a) => lines([
  'Commit audited: ' + args.commit,
  'Your worktree, detached at that commit: ' + at(args.worktrees, a.label),
  'Lens: ' + a.lens + (a.group ? ', group of activities: ' + a.group : ''),
  'Scratch directory: ' + at(args.scratch, a.label),
  args.kept ? 'Kept list, for this re-check: ' + at(args.scratch, a.label, 'kept.md') : '',
  'Issues listing at that commit: ' + at(preflightDir, 'issues.txt'),
  'Also write your return, in its shape, to return.md in your scratch directory.',
  FROM_DISK,
])
const verifyBrief = (a) => lines([
  'Verify brief.',
  'Commit audited: ' + args.commit,
  'Your worktree, detached at that commit: ' + at(args.worktrees, a.label),
  'The drafts to verify: the auditor ' + a.label + ', lens ' + a.lens +
    (a.group ? ', group of activities: ' + a.group : '') + ', in ' + at(args.scratch, a.label),
  'Your directory: ' + at(args.scratch, 'verify', a.label),
  args.kept ? 'Kept list: ' + at(args.scratch, a.label, 'kept.md') : '',
  'Issues listing: ' + at(preflightDir, 'issues.txt'),
  'Tripwires listing: ' + at(preflightDir, 'tripwires.txt'),
  'History of the audit earlier passes: ' + at(preflightDir, 'history.txt') + ', and their ' +
    'report files, with the owner\'s rulings: ' + at(preflightDir, 'reports.txt'),
  FROM_DISK,
])
const auditorOpts = (a, step) => ({ agentType: 'knowledge-architect-workflow-auditor', phase: step, label: step + ' ' + a.label })

// Drafts, each auditor's drafts going to its verifier as soon as it returns.
let verdicts = []
if (!args.from || args.from === 'verify') {
  if (!args.from) phase('drafts')
  verdicts = await pipeline(auditors,
    (a) => (args.from ? Promise.resolve(a) : agent(draftBrief(a), auditorOpts(a, 'drafts')).then(() => a)),
    (a) => agent(verifyBrief(a), Object.assign(auditorOpts(a, 'verify'), { schema: VERDICTS })))
  const ok = verdicts.filter(Boolean)
  const sum = (k) => ok.reduce((t, v) => t + v[k], 0)
  log('verify: ' + ok.length + ' of ' + auditors.length + ' auditors verified; ' + sum('verified') +
    ' verified, ' + sum('refuted') + ' refuted, ' + sum('plausible') + ' plausible, ' +
    sum('bearsOnRuling') + ' bearing on a ruling, ' + sum('bearsOnTripwire') + ' bearing on a tripwire, ' +
    sum('known') + ' known, ' + sum('kept') + ' kept')
}

// Dedup: the confirmed-findings file.
const confirmed = at(args.scratch, 'dedup', 'confirmed.md')
if (!args.from || args.from === 'verify') {
  phase('dedup')
  await agent(lines([
    'You merge the verified drafts of one pass of the agentic-workflow axis of a project audit.',
    'Read every verdicts.md under ' + at(args.scratch, 'verify') + ' and the drafts they name.',
    'Write ' + confirmed + ': one line per finding, for every draft the verifier did not refute, ' +
      'merging the drafts that report one finding: its id, the id of the draft kept, as ' +
      '<auditor label>/<draft file name>; its tag; its verdict and marks; the path of its draft; ' +
      'the ids merged into it, and the shared defect in a few words.',
    args.previous ? 'Classify each finding as on old or new text, as ' + SECTION + ' says under ' +
      '"Old and new text", against the commit ' + args.previous + ', in the worktree ' + reading +
      ', and add to its line its class and the commit and the command that decided it.' : '',
    'Return its path.',
  ]), { schema: PATH, phase: 'dedup', label: 'dedup' })
}

// Cluster and merge: the causes file.
const causesFile = at(args.scratch, 'merge', 'causes.md')
let causeIds = []
if (args.from !== 'repairs') {
  phase('cluster')
  const groupings = (await parallel(Array.from({ length: clusters }, (_, i) => () => agent(lines([
    'You group the confirmed findings of one pass of the agentic-workflow axis of a project ' +
      'audit by the cause that produced them.',
    CORPUS,
    'Read the confirmed-findings file ' + confirmed + ', every draft it names, and the corpus.',
    KINDS,
    'For each cause write: an id, its kind, its mechanism in one sentence, the ids of its ' +
      'findings, and every text, with its path and section, that states an instruction about ' +
      'its moment or its act.',
    'Write the grouping to ' + at(args.scratch, 'cluster-' + (i + 1), 'grouping.md') + ', and ' +
      'return its path.',
  ]), { schema: PATH, phase: 'cluster', label: 'cluster ' + (i + 1) })))).filter(Boolean)
  phase('merge')
  const merged = await agent(lines([
    'You merge the groupings of the confirmed findings of one pass of the agentic-workflow axis ' +
      'of a project audit into one file of causes.',
    CORPUS,
    'Read the groupings ' + groupings.map((g) => g.path).join(', ') + ', the confirmed-findings ' +
      'file ' + confirmed + ' and the drafts the findings name.',
    KINDS,
    'Write ' + causesFile + ': one section per cause, its id K1, K2 and on, its kind, its findings ' +
      'and the sections of the groupings it comes from. Where the groupings differ, read the ' +
      'findings concerned and choose; state every choice and its reason in a last section, ' +
      '"Choices". You may merge or split causes. Return its path and the cause ids.',
  ]), { schema: IDS, phase: 'merge', label: 'merge' })
  causeIds = merged ? merged.ids : []
} else {
  phase('merge')
  const read = await agent('Read ' + causesFile + ' and return its path and the ids of its causes, ' +
    'in order.', { schema: IDS, phase: 'merge', label: 'causes' })
  causeIds = read ? read.ids : []
}
log('causes: ' + causeIds.length)

// Repairs: repairsPerCause agents per cause, their proposals alternatives for the cause's one repair.
phase('repairs')
const jobs = []
for (const id of causeIds) for (let k = 1; k <= perCause; k++) jobs.push({ id, dir: at(args.scratch, 'repairs', perCause > 1 ? id + '-' + k : id) })
// A check agent runs the checker, so each gets a worktree of its own, per review-invariants. The
// preflight cannot create them: the jobs are known only now.
const checkTree = (j) => at(args.worktrees, 'check-' + j.dir.split('/').pop())
if (jobs.length) {
  const trees = await agent(lines([
    'You set up the worktrees of the check step of one pass of the agentic-workflow axis of a ' +
      'project audit. From the project root, for each of these paths that is not already a ' +
      'worktree detached at ' + args.commit + ', run git worktree add --detach <path> ' + args.commit +
      ', and report what failed: ' +
      jobs.map(checkTree).join(', ') + '.',
    'Edit nothing else. Return done true only if every command succeeded.',
  ]), { schema: DONE, phase: 'repairs', label: 'check worktrees' })
  if (!trees || !trees.done) throw new Error('check worktrees failed: ' + (trees ? trees.notes : 'no return'))
}
const repairPrompt = (j) => lines([
  'You propose the repair of one cause of findings of the agentic-workflow axis of a project ' +
    'audit, at commit ' + args.commit + '.',
  CORPUS,
  'Read the cause ' + j.id + ' in the causes file ' + causesFile + ', every draft its findings ' +
    'name, and the corpus. ' + KINDS,
  'Propose the repair of the cause as those sections say. Where you add an instruction, say why ' +
    'none of the forms of repair that standard names closes the cause. Name the nearest rival to ' +
    'your repair and the fact that decides between them.',
  'Write proposal.md in ' + j.dir + ': each edit as its file, the text before and the text after; ' +
    'the findings each edit closes; every instruction added and removed, quoted, and their two ' +
    'counts; the nearest rival; and any finding of the cause the repair does not close, with the ' +
    'reason. Return its path.',
])
const checkBrief = (j, p) => lines([
  'Check brief.',
  'Commit audited: ' + args.commit,
  'Your worktree, detached at that commit: ' + checkTree(j),
  'The proposal: ' + p.path + ', for the cause ' + j.id + ' of the causes file ' + causesFile,
  'Your directory: ' + j.dir,
  FROM_DISK,
])
// Each proposal goes to its check as soon as it is written.
const proposals = (await pipeline(jobs,
  (j) => agent(repairPrompt(j), { schema: PATH, phase: 'repairs', label: j.id })
    .then((r) => (r ? Object.assign({ id: j.id }, r) : null)),
  (p, j) => (p ? agent(checkBrief(j, p), { agentType: 'knowledge-architect-workflow-auditor', schema: PATH,
    phase: 'check', label: 'check ' + j.id }).then((c) => Object.assign({ check: c ? c.path : null }, p)) : null),
)).filter(Boolean)

// Cross-check.
phase('cross-check')
const cross = at(args.scratch, 'cross-check', 'cross-check.md')
await agent(lines([
  'You cross-check the repair proposals of one pass of the agentic-workflow axis of a project audit.',
  CORPUS,
  'Read the causes file ' + causesFile + ', every proposal (' + proposals.map((p) => p.path).join(', ') +
    '), the check beside each, the confirmed-findings file ' + confirmed + ', and the axis outcome ' +
    'table in ' + SECTION + '.',
  'Write ' + cross + ' with: every paragraph that the proposals of two or more causes edit, with ' +
    'the causes concerned; for each cause, the ' +
    'outcome you propose under the axis outcome table, with its reason, for the session to decide; ' +
    'and every finding marked bears-on-ruling, with the ruling quoted from its source and the ' +
    'premise that may be false; and every finding marked bears-on-tripwire, with the tripwire ' +
    'and its firing evidence. Return its path.',
]), { schema: PATH, phase: 'cross-check', label: 'cross-check' })

// Report.
phase('report')
const report = await agent(lines([
  'You write the report of one pass of the agentic-workflow axis of a project audit, for the ' +
    'session that verifies it and discusses it with the owner. Quote; do not summarise evidence.',
  'Read: every return.md and verdicts.md under ' + args.scratch + ', the confirmed-findings file ' +
    confirmed + ', the causes file ' + causesFile + ', every proposal and check under ' +
    at(args.scratch, 'repairs') + ', and the cross-check ' + cross + '.',
  'Write ' + at(args.scratch, 'report', 'report.md') + ' in six parts:',
  '1. the header: the commit audited, ' + args.commit + '; the agents of each step; the counts of ' +
    'drafts, verified, plausible, bearing on a ruling, bearing on a tripwire, known, kept, after ' +
    'dedup and refuted; for a re-check, the counts the section record-of-an-audit of the skill ' +
    'knowledge-architect-project-audit asks of a pass, the owner-list one as the cross-check ' +
    'proposes it; each ' +
    'refuted draft with its reason; each auditor return that says a corpus file was not read whole;',
  '2. the figures: the run counts of findings and causes, and the instructions each proposal adds ' +
    'and removes;',
  '3. one section per cause: its kind and its one-sentence mechanism; its findings, each with its ' +
    'quoted evidence, the verifier verdict and note, and its tag; the repair as edits before and ' +
    'after, per file; the instructions it adds and removes, quoted; the nearest rival; its check, ' +
    'quoted; the outcome the cross-check proposes, and the default; each finding marked ' +
    'bears-on-ruling, with the ruling quoted and the premise that may be false, and each marked ' +
    'bears-on-tripwire, with the tripwire;',
  '4. the overlap map: each paragraph the proposals of several causes edit, and the causes concerned;',
  '5. the draft of the owner list, its labels left for the session to number;',
  '6. what the run did not decide: the merge choices, the plausible verdicts, the "Outside the ' +
    'axis" and "Met outside the task" notes of the returns, and every step that failed or returned ' +
    'nothing' + (unwalked.length ? ', and the L2 groups left unwalked: ' + unwalked.join('; ') : '') + '.',
  'Return its path.',
]), { schema: PATH, phase: 'report', label: 'report' })

return {
  report: report ? report.path : null,
  auditors: auditors.length,
  verified: verdicts.filter(Boolean).length,
  causes: causeIds.length,
  proposals: proposals.length,
  checks: proposals.filter((p) => p.check).length,
  unwalked,
}
