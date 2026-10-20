export const meta = {
  name: 'knowledge-architect-design-record-audit',
  description: 'One pass of the design-record axis of a project audit, from its preflight to a report the session verifies',
  phases: [
    { title: 'preflight' }, { title: 'setup' }, { title: 'drafts' }, { title: 'calibration' },
    { title: 'verify' }, { title: 'check' }, { title: 'cross-check' }, { title: 'report' },
    { title: 'judge' },
  ],
}
// args: {
//   commit      the commit audited
//   scratch     the run's scratch directory; each step and each agent writes under it
//   worktrees   the directory under which the run creates the worktrees
//   history     a git range whose commit messages and added report files record the earlier passes
//   previous    optional: for a re-check, the commit the previous pass audited
//   kept        optional: the path of the kept list, for a re-check
//   sample      optional: the calibration sample, an array of entries, each { ref } for a design
//               head or a rejected entry; the preflight chooses one if absent
//   counts      optional: { groupSize, checkersPerGroup }, each missing key taking its default
//   from        optional: 'calibration' or 'verify', to restart from the files a run wrote; or
//               'application', to judge the session's application of a pass instead of auditing
//   applied     with from 'application': the commit that holds the application
// }
const counts = args.counts || {}
const groupSize = counts.groupSize === undefined ? 60 : counts.groupSize
const checkersPerGroup = counts.checkersPerGroup === undefined ? 1 : counts.checkersPerGroup
if (!(groupSize >= 1)) throw new Error('groupSize: at least 1')
if (!(checkersPerGroup >= 1)) throw new Error('checkersPerGroup: at least 1')
if (args.from && args.from !== 'calibration' && args.from !== 'verify' && args.from !== 'application') {
  throw new Error('from: calibration, verify or application')
}
if (args.from === 'application' && !(args.commit && args.applied)) {
  throw new Error('commit and applied: the commit audited and the commit of the application')
}
const at = (...parts) => parts.join('/')
const lines = (xs) => xs.filter(Boolean).join('\n')
const AUDITOR = 'knowledge-architect-design-record-auditor'
const SECTION = 'the section design-record-axis of the skill knowledge-architect-project-audit'
const reading = at(args.worktrees, 'reading')
const preflightDir = at(args.scratch, 'preflight')
const groupsFile = at(args.scratch, 'setup', 'groups.json')
const calibrationFile = at(args.scratch, 'calibration', 'calibration.md')
// The home of this line is the bullet of review-invariants, in the skill knowledge-architect-review,
// that names the files a subagent's copy may predate: the definition, the root CLAUDE.md and the
// primer it imports.
const FROM_DISK = 'Your copies of your definition, of the root CLAUDE.md and of the primer it ' +
  'imports may predate this session: read each in the worktree this brief names, and follow the ' +
  'worktree where your copy differs.'

const ENTRY = {
  type: 'object',
  properties: { ref: { type: 'string' }, name: { type: 'string' }, file: { type: 'string' }, line: { type: 'number' } },
}
const ENTRIES = {
  type: 'object',
  properties: {
    components: {
      type: 'array',
      items: { type: 'object', properties: { name: { type: 'string' }, entries: { type: 'array', items: ENTRY } }, required: ['name', 'entries'] },
    },
    sample: { type: 'array', items: ENTRY },
  },
  required: ['components', 'sample'],
}
const GROUPS = {
  type: 'object',
  properties: {
    groups: {
      type: 'array',
      items: {
        type: 'object',
        properties: { label: { type: 'string' }, components: { type: 'array', items: { type: 'string' } }, entries: { type: 'array', items: ENTRY } },
        required: ['label', 'components', 'entries'],
      },
    },
    sample: { type: 'array', items: ENTRY },
  },
  required: ['groups', 'sample'],
}
const JUDGED = {
  type: 'object',
  properties: {
    judged: {
      type: 'array',
      items: { type: 'object', properties: { entry: { type: 'string' }, verdict: { type: 'string', enum: ['conforms', 'fails'] } }, required: ['entry', 'verdict'] },
    },
  },
  required: ['judged'],
}
const CALIBRATED = {
  type: 'object',
  properties: { path: { type: 'string' }, proposed: { type: 'number' }, bothReadings: { type: 'number' } },
  required: ['path', 'proposed', 'bothReadings'],
}
const VERDICTS = {
  type: 'object',
  properties: {
    path: { type: 'string' }, verified: { type: 'number' }, refuted: { type: 'number' },
    plausible: { type: 'number' }, bearsOnRuling: { type: 'number' },
    bearsOnTripwire: { type: 'number' }, known: { type: 'number' }, kept: { type: 'number' },
    createdHeads: { type: 'number' },
  },
  required: ['path', 'verified', 'refuted', 'plausible', 'bearsOnRuling', 'bearsOnTripwire', 'known', 'kept', 'createdHeads'],
}
const PATH = { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] }
const DONE = {
  type: 'object',
  properties: { done: { type: 'boolean' }, notes: { type: 'string' } },
  required: ['done', 'notes'],
}

// An entry is named by its reference, a design head's or a rejected entry's.
const id = (e) => e.ref
const show = (e) => e.ref

// The cut: a Component above groupSize is cut alone into groups of consecutive entries whose sizes
// differ by at most one; the others are packed whole, in the manifest's order, each into the first
// packed group it fits in. A packed group holding one Component takes that Component's name.
const cut = (components) => {
  const out = []
  for (const c of components) {
    const n = c.entries.length
    if (n === 0) continue
    if (n > groupSize) {
      const k = Math.ceil(n / groupSize)
      const base = Math.floor(n / k)
      let start = 0
      for (let g = 0; g < k; g++) {
        const size = base + (g < n % k ? 1 : 0)
        out.push({ label: c.name + '-' + (g + 1), components: [c.name], entries: c.entries.slice(start, start + size) })
        start += size
      }
    } else {
      let target = out.find((g) => g.packed && g.entries.length + n <= groupSize)
      if (!target) {
        target = { packed: true, components: [], entries: [] }
        out.push(target)
      }
      target.components.push(c.name)
      target.entries = target.entries.concat(c.entries)
    }
  }
  let p = 0
  for (const g of out) {
    if (!g.packed) continue
    g.label = g.components.length === 1 ? g.components[0] + '-1' : 'packed-' + ++p
    delete g.packed
  }
  return out
}

// The judgement of an application: every entry whose text the session's application of a pass
// changed, judged after the owner's ruling, as a run of its own; it ends at its report.
if (args.from === 'application') {
  const dir = at(args.scratch, 'application')
  const judging = at(args.worktrees, 'application')
  phase('preflight')
  const pre = await agent(lines([
    'You set up the judgement of the application of one pass of the design-record axis of a ' +
      'project audit. From the project root:',
    '- run git worktree add --detach ' + judging + ' ' + args.applied + ', and work in that ' +
      'worktree for every step below;',
    '- list every design head and every rejected entry whose text differs between ' +
      args.commit + ' and ' + args.applied + ', a head the application created included and one it ' +
      'removed left out, and an entry whose only change is the target of a reference left out too, ' +
      'its words unchanged; read git diff ' + args.commit + ' ' + args.applied + ' over every ' +
      'Component\'s design home and rejected home whole. Order them as the section ' +
      'take-your-group of the agent definition ' + AUDITOR + ' states, which you read whole, each ' +
      'as { ref, file, line }, a head\'s ref design@<component>@<slug>, a rejected entry\'s ' +
      'rejected@<component>@<slug>;',
    '- write that list, as JSON, to ' + at(dir, 'entries.json') + ', creating the directory;',
    '- run cargo klarch issues and save its output to ' + at(dir, 'issues.txt') + ', and ' +
      'cargo klarch tripwires to ' + at(dir, 'tripwires.txt') + '.',
    'Edit nothing else. Return the Components with their entries, and the sample as an empty array.',
  ]), { schema: ENTRIES, phase: 'preflight', label: 'preflight' })
  if (!pre) throw new Error('preflight returned nothing')
  const jgroups = cut(pre.components)
  if (!jgroups.length) {
    log('the application changed no entry')
    return { report: null, groups: [], failing: 0, gaps: [] }
  }
  phase('setup')
  const setup = await agent(lines([
    'You set up the groups of the judgement of an application. From the project root:',
    '- write this JSON, exactly, to ' + at(dir, 'groups.json') + ':',
    JSON.stringify({ groups: jgroups, sample: [] }),
    '- for each of these paths, run git worktree add --detach <path> ' + args.applied + ': ' +
      jgroups.map((g) => at(args.worktrees, 'application-' + g.label)).join(', ') + '.',
    'Edit nothing else. Return done true only if every command succeeded.',
  ]), { schema: DONE, phase: 'setup', label: 'setup' })
  if (!setup || !setup.done) throw new Error('setup failed: ' + (setup ? setup.notes : 'no return'))
  phase('judge')
  const jfailures = []
  const judged = await parallel(jgroups.map((g) => () => agent(lines([
    'Application brief.',
    'Commit audited: ' + args.commit,
    'Commit of the application: ' + args.applied,
    'Your worktree, detached at the application: ' + at(args.worktrees, 'application-' + g.label),
    'Your entries, ' + g.entries.length + ', each changed by the application:',
    g.entries.map((e, i) => (i + 1) + '. ' + show(e)).join('\n'),
    'Issues listing: ' + at(dir, 'issues.txt'),
    'Tripwires listing: ' + at(dir, 'tripwires.txt'),
    'Your directory: ' + at(dir, g.label),
    'In the return, name each entry by its reference, as listed here.',
    FROM_DISK,
  ]), { agentType: AUDITOR, schema: JUDGED, phase: 'judge', label: 'judge ' + g.label })
    .then((r) => {
      if (!r) jfailures.push('the judge of group ' + g.label + ' returned nothing')
      return { g, r }
    })))
  // An entry a judge leaves out of its return is judged by nobody, so it is named, as the drafts'
  // gaps are, rather than read as conforming.
  const jgaps = []
  for (const x of judged) {
    if (!x || !x.r) {
      jgaps.push((x ? x.g.label : 'a group') + ': no return')
      continue
    }
    const got = new Set(x.r.judged.map((j) => j.entry))
    for (const e of x.g.entries) if (!got.has(id(e))) jgaps.push(x.g.label + ': ' + show(e))
  }
  const failing = judged.filter((x) => x && x.r).reduce((t, x) => t + x.r.judged.filter((j) => j.verdict === 'fails').length, 0)
  log('judge: ' + jgroups.length + ' groups, ' + failing + ' entries failing' +
    (jgaps.length ? '; entries missing from a return: ' + jgaps.join('; ') : ''))
  phase('report')
  const jreport = await agent(lines([
    'You write the report of the judgement of an application of one pass of the design-record ' +
      'axis, for the session that repairs it. Quote; do not summarise evidence. You edit nothing in ' +
      'the project.',
    'Read every file under ' + dir + ' that the judges wrote.',
    'Write ' + at(dir, 'report.md') + ': per failing entry, the rule or the failure mode of an edit ' +
      'it fails, the words quoted, the tripwire or issue it leaves false, and the repair the judge ' +
      'proposes; then the counts of entries judged and failing, each entry missing from a return, ' +
      'which nobody judged' + (jgaps.length ? ': ' + jgaps.join('; ') : '') + ', and every step that ' +
      'returned nothing' + (jfailures.length ? ', among them: ' + jfailures.join('; ') : '') + '.',
    'Return its path.',
  ]), { schema: PATH, phase: 'report', label: 'report' })
  return { report: jreport ? jreport.path : null, groups: jgroups.map((g) => g.label), failing, gaps: jgaps }
}

let groups = []
let sample = []
if (!args.from) {
  phase('preflight')
  const pre = await agent(lines([
    'You set up one pass of the design-record axis of a project audit. From the project root:',
    '- run git worktree add --detach ' + reading + ' ' + args.commit + ', and work in that worktree ' +
      'for every step below;',
    '- list the entries of every Component: the Components in the order of the manifest, ' +
      'knowledge-architect.toml, the root Component first, then its components list in order; each ' +
      'Component\'s entries in the order the section take-your-group of the agent definition ' +
      AUDITOR + ' states, which you read whole in the worktree. Run the checker command cargo klarch ' +
      'model for the level-three headings that end with a slug, in each Component\'s design home ' +
      'and rejected home. Give each as { ref, file, line }, a head\'s ref ' +
      'design@<component>@<slug>, a rejected entry\'s rejected@<component>@<slug>;',
    '- write that list, as JSON, to ' + at(preflightDir, 'entries.json') + ', creating the directory;',
    args.sample ? '' : '- choose the calibration sample: about five entries, from different ' +
      'Components where there are several, in the same shape;',
    '- run cargo klarch issues and save its output to ' + at(preflightDir, 'issues.txt') + ', and ' +
      'cargo klarch tripwires to ' + at(preflightDir, 'tripwires.txt') + ';',
    '- save the messages of the commits of the range ' + args.history + ', with git log ' +
      '--format="=== %h %s%n%n%b", to ' + at(preflightDir, 'history.txt') + ';',
    '- save the report files of earlier passes that the range added, with git log -p ' +
      '--diff-filter=A --format="=== %h %s" ' + args.history + ' -- docs/audit-reports/, to ' +
      at(preflightDir, 'reports.txt') + '.',
    'Edit nothing else. Return the Components with their entries, and the sample' +
      (args.sample ? ' as an empty array' : '') + '.',
  ]), { schema: ENTRIES, phase: 'preflight', label: 'preflight' })
  if (!pre) throw new Error('preflight returned nothing')
  sample = args.sample || pre.sample
  groups = cut(pre.components)
  if (!groups.length) throw new Error('preflight listed no entry')

  phase('setup')
  const checkTrees = []
  for (const g of groups) for (let k = 1; k <= checkersPerGroup; k++) checkTrees.push(at(args.worktrees, 'check-' + g.label + '-' + k))
  const setup = await agent(lines([
    'You set up the groups of one pass of the design-record axis of a project audit. From the ' +
      'project root:',
    '- write this JSON, exactly, to ' + groupsFile + ', creating the directory:',
    JSON.stringify({ groups, sample }),
    '- for each of these paths, run git worktree add --detach <path> ' + args.commit + ': ' +
      groups.map((g) => at(args.worktrees, g.label)).concat(checkTrees).join(', ') + ';',
    args.kept ? '- copy the kept list ' + args.kept + ' to kept.md in each of these directories, ' +
      'creating it: ' + groups.map((g) => at(args.scratch, g.label)).join(', ') + '.' : '',
    'Edit nothing else. Return done true only if every command succeeded.',
  ]), { schema: DONE, phase: 'setup', label: 'setup' })
  if (!setup || !setup.done) throw new Error('setup failed: ' + (setup ? setup.notes : 'no return'))
} else {
  phase('setup')
  const back = await agent('Read ' + groupsFile + ' and return its groups and its sample exactly as ' +
    'written.', { schema: GROUPS, phase: 'setup', label: 'groups' })
  if (!back) throw new Error('could not read ' + groupsFile)
  groups = back.groups
  sample = back.sample
}
log('groups: ' + groups.map((g) => g.label + ' (' + g.entries.length + ')').join(', ') + '; sample: ' + sample.length)

// A sample entry's draft of record is that of the group whose listed entries hold it.
const ownerOf = (e) => {
  const g = groups.find((x) => x.entries.some((y) => id(y) === id(e)))
  return g ? g.label : 'none'
}
const sampleLines = sample.map((e) => '- ' + show(e) + ' (draft of record: group ' + ownerOf(e) + ')')

// Drafts, all groups in parallel; the calibration waits for every one.
const gaps = []
// Every step that fails or returns nothing, for the report's part 6.
const failures = []
if (!args.from) {
  phase('drafts')
  const draftBrief = (g) => lines([
    'Drafting brief.',
    'Commit audited: ' + args.commit,
    'Your worktree, detached at that commit: ' + at(args.worktrees, g.label),
    'Your group: ' + g.label + ', ' + g.entries.length + ' entries, in this order:',
    g.entries.map((e, i) => (i + 1) + '. ' + show(e)).join('\n'),
    'The calibration sample, which you judge exactly as your own:',
    sample.map((e) => '- ' + show(e)).join('\n'),
    'Scratch directory: ' + at(args.scratch, g.label) + '. Name each draft <nn>-<slug>.md: nn its ' +
      'position, two digits, the sample following your group in its order; slug the entry\'s slug.',
    args.kept ? 'Kept list, for this re-check: ' + at(args.scratch, g.label, 'kept.md') : '',
    'In the return, name each entry by its reference, as listed here.',
    FROM_DISK,
  ])
  const drafted = await parallel(groups.map((g) => () => agent(draftBrief(g),
    { agentType: AUDITOR, schema: JUDGED, phase: 'drafts', label: 'drafts ' + g.label })
    .then((r) => ({ g, r }))))
  for (const x of drafted) {
    if (!x || !x.r) {
      failures.push('the drafts of group ' + (x ? x.g.label : '(unknown)') + ' returned nothing')
      gaps.push((x ? x.g.label : 'a group') + ': no return')
      continue
    }
    const got = new Set(x.r.judged.map((j) => j.entry))
    for (const e of x.g.entries) if (!got.has(id(e))) gaps.push(x.g.label + ': ' + show(e))
  }
  if (gaps.length) log('entries missing from a return: ' + gaps.join('; '))
}

// Calibration: one auditor compares the sample's verdicts across the groups.
if (!args.from || args.from === 'calibration') {
  phase('calibration')
  const cal = await agent(lines([
    'Calibration brief.',
    'Commit audited: ' + args.commit,
    'Your worktree, detached at that commit: ' + reading,
    'The drafters\' scratch directories: ' + groups.map((g) => at(args.scratch, g.label)).join(', '),
    'The sample:',
    sampleLines.join('\n'),
    'Your directory: ' + at(args.scratch, 'calibration') + '; write calibration.md there.',
    FROM_DISK,
  ]), { agentType: AUDITOR, schema: CALIBRATED, phase: 'calibration', label: 'calibration' })
  if (!cal) {
    failures.push('the calibration returned nothing, so the verifiers had no calibration file')
    log('calibration returned nothing')
  }
}

// Verify: one auditor per group, each given the calibration.
phase('verify')
const verifyBrief = (g) => lines([
  'Verify brief.',
  'Commit audited: ' + args.commit,
  'Your worktree, detached at that commit: ' + at(args.worktrees, g.label),
  'The drafts to verify: the group ' + g.label + ', in ' + at(args.scratch, g.label) + ', its ' +
    'entries ' + show(g.entries[0]) + ' to ' + show(g.entries[g.entries.length - 1]) + '.',
  'The sample, each entry with the group whose draft is of record:',
  sampleLines.join('\n'),
  'The calibration file: ' + calibrationFile,
  'Your directory: ' + at(args.scratch, 'verify', g.label),
  args.kept ? 'Kept list: ' + at(args.scratch, g.label, 'kept.md') : '',
  'Issues listing: ' + at(preflightDir, 'issues.txt'),
  'Tripwires listing: ' + at(preflightDir, 'tripwires.txt'),
  'History of the audit earlier passes: ' + at(preflightDir, 'history.txt') + ', and their ' +
    'report files, with the owner\'s rulings: ' + at(preflightDir, 'reports.txt'),
  args.previous ? 'For this re-check, the commit the previous pass audited: ' + args.previous : '',
  FROM_DISK,
])
const checkBrief = (label, k) => lines([
  'Check brief.',
  'Commit audited: ' + args.commit,
  'Your worktree, detached at that commit: ' + at(args.worktrees, 'check-' + label + '-' + k),
  'The group ' + label + ': its drafts in ' + at(args.scratch, label) + ', its verdicts in ' +
    at(args.scratch, 'verify', label, 'verdicts.md'),
  'Your directory: ' + at(args.scratch, 'check', label) + '; write check-' + k + '.md there.',
  checkersPerGroup > 1 ? 'Other checkers judge the same heads; your check is one alternative.' : '',
  FROM_DISK,
])
// Each group's verifier hands to its checkers as soon as it returns: for a group with a verified
// finding that creates a head, checkersPerGroup auditors, whose checks are alternatives.
const ks = Array.from({ length: checkersPerGroup }, (_, i) => i + 1)
const results = (await pipeline(groups,
  (g) => agent(verifyBrief(g), { agentType: AUDITOR, schema: VERDICTS, phase: 'verify', label: 'verify ' + g.label })
    .then((v) => {
    if (!v) failures.push('the verifier of group ' + g.label + ' returned nothing')
    return v ? Object.assign({ label: g.label }, v) : null
  }),
  (v, g) => (v && v.createdHeads > 0
    ? parallel(ks.map((k) => () => agent(checkBrief(g.label, k),
      { agentType: AUDITOR, schema: PATH, phase: 'check', label: 'check ' + g.label + '-' + k })))
      .then((cs) => {
        if (cs.some((c) => !c)) failures.push('a checker of group ' + g.label + ' returned nothing')
        return Object.assign({ checks: cs.filter(Boolean).length }, v)
      })
    : (v ? Object.assign({ checks: 0 }, v) : null)),
)).filter(Boolean)
const verdicts = results
const sum = (k) => verdicts.reduce((t, v) => t + v[k], 0)
log('verify: ' + verdicts.length + ' of ' + groups.length + ' groups verified; ' + sum('verified') +
  ' verified, ' + sum('refuted') + ' refuted, ' + sum('plausible') + ' plausible, ' +
  sum('bearsOnRuling') + ' bearing on a ruling, ' + sum('bearsOnTripwire') + ' bearing on a tripwire, ' +
  sum('known') + ' known, ' + sum('kept') + ' kept, ' + sum('createdHeads') + ' creating heads; ' +
  sum('checks') + ' checks')

// Cross-check.
phase('cross-check')
const cross = at(args.scratch, 'cross-check', 'cross-check.md')
await agent(lines([
  'You cross-check the verified findings of one pass of the design-record axis of a project audit, ' +
    'at commit ' + args.commit + ', in the worktree ' + reading + '. You edit nothing in the ' +
    'project, and write only ' + cross + '.',
  'Read every verdicts.md under ' + at(args.scratch, 'verify') + ' and the drafts they name, every ' +
    'check under ' + at(args.scratch, 'check') + ', the calibration file ' + calibrationFile + ', and ' +
    'the outcome table of ' + SECTION + '. A sample entry\'s finding is its draft of record only:',
  sampleLines.join('\n'),
  'Write ' + cross + ' in three parts:',
  '1. the overlap map: every pair of verified findings that edit one paragraph or one slug, every ' +
    'rename whose citing sites another finding edits, and every split, merge or move whose ' +
    'rejected entry or citing head another group drafted;',
  '2. the stale entries: each tripwire and each issue that cites a head some verified finding ' +
    'edits, read whole with the checker command cargo klarch show, and whether the edit leaves it ' +
    'false;',
  '3. for each verified or plausible finding, the outcome you propose under the outcome table, ' +
    'with its reason, for the session to decide.',
  'Return its path.',
]), { schema: PATH, phase: 'cross-check', label: 'cross-check' })
  .then((c) => {
    if (!c) failures.push('the cross-check returned nothing')
    return c
  })

// Report.
phase('report')
const report = await agent(lines([
  'You write the report of one pass of the design-record axis of a project audit, for the session ' +
    'that verifies it and discusses it with the owner. Quote; do not summarise evidence. You edit ' +
    'nothing in the project.',
  'Read: every return.md under ' + args.scratch + ', the calibration file ' + calibrationFile +
    ', every verdicts.md under ' + at(args.scratch, 'verify') + ' and the drafts they name, every ' +
    'check under ' + at(args.scratch, 'check') + ', and the cross-check ' + cross + '.',
  'The groups: ' + groups.map((g) => g.label + ', ' + show(g.entries[0]) + ' to ' +
    show(g.entries[g.entries.length - 1]) + ', ' + g.entries.length + ' entries').join('; ') + '.',
  'The sample, each entry with the group whose draft is of record:',
  sampleLines.join('\n'),
  gaps.length ? 'Entries missing from a return: ' + gaps.join('; ') + '.' : '',
  'Write ' + at(args.scratch, 'report', 'report.md') + ' in six parts:',
  '1. the header: the commit audited, ' + args.commit + '; the agents of each step; each group ' +
    'with its first and last entry; the counts of entries, of drafts conforming and failing, ' +
    'verified, refuted, plausible, bearing on a ruling, bearing on a tripwire, known and kept; for ' +
    'a re-check, the counts the section record-of-an-audit of the skill ' +
    'knowledge-architect-project-audit asks of a pass, the owner-list one as the cross-check ' +
    'proposes it; each entry missing from a return; each refuted draft with its reason;',
  '2. the calibration: per sample entry, each group\'s verdict, the proposed reading and the ' +
    'sentence of the rule it rests on, quoted, or the two readings the rule admits; and the drafts ' +
    'the verifiers judged again under a proposed reading;',
  '3. the findings, grouped by the outcome the cross-check proposes, those applied in the branch by ' +
    'Component and file: per finding, the entry, the rule failed, the quoted evidence, the edit ' +
    'before and after, the verifier verdict and note, old or new text for a re-check, and the ' +
    'checks beside each head it creates;',
  '4. the overlap map and the stale entries;',
  '5. the draft of the owner list, grouped by kind, its labels left for the session to number;',
  '6. what the run did not decide: the plausible verdicts, the notes the drafts give of a head ' +
    'false of the code, the "Met outside the task" notes of the returns, and every step that ' +
    'failed or returned nothing' + (failures.length ? ', among them: ' + failures.join('; ') : '') + '.',
  'Return its path.',
]), { schema: PATH, phase: 'report', label: 'report' })

return {
  report: report ? report.path : null,
  groups: groups.map((g) => g.label),
  verified: verdicts.length,
  checks: sum('checks'),
  gaps,
}
