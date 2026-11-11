// A second world, with a workflow unlike the reference case's: one
// developer on one repository, their chats and a goal they set by hand,
// its agent delegating directly. No coordinator, no timers, no pools, no
// ops application. The same pages draw it; a page that reads wrong here
// assumed the reference case's workflow. Times are minutes from Tuesday
// 27 October, 09:30 UTC.
(function () {
  const MOCKUPS = window.MOCKUPS = window.MOCKUPS || { worlds: {} };

  function initial() {
    return {
      now: 0,
      person: 'Robin',
      project: 'tally',

      config: {
        templates: { chat: 'chat', goal: 'goal' },
        charters: ['Assistant', 'Implementer'],
        period: 'month',
        allotment: 50,
        ops: false,
        next: { account: 'Claude', model: 'Opus · high' },
        restart: 'temper serve',
      },

      repos: {
        tally: {
          name: 'robin/tally', forge: 1, branch: 'main',
          tip: 'a81f03',
          ci: { state: 'passing', at: -40 },
          after: null,
          queue: [],
        },
      },

      forges: [
        { n: 1, kind: 'GitHub', web: 'https://github.com', mode: 'connected', state: 'connected', login: 'temper-robin[bot]', id: 190311402,
          app: { slug: 'temper-robin', installs: [{ account: 'robin', id: 55311706, all: false, names: ['tally'] }] },
          load: { used: 210, rate: 5000, window: 'hour', reserve: 500, uses: [['fresh reads', 96], ['writes', 12], ['keeping up', 88], ['the slow pass', 14]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['robin/tally', -3]] } },
      ],

      landings: [
        // T2, a change the chat T1 handed its branch to, a small fix.
        { task: 'T2', repo: 'tally', at: -1080, title: 'sum: stream the input' },
      ],

      findings: [],

      workers: [
        {
          name: 'laptop', kind: 'laptop', what: "Robin's laptop", slots: 2,
          logins: [['claude-robin', 'signed in']],
        },
      ],

      accounts: [
        {
          name: 'Claude', provider: 'Anthropic', holder: 'Robin', plan: 'Claude subscription',
          sources: [
            { name: 'claude-robin', kind: 'program', program: 'Claude Code', hosts: ['laptop'], use: 'In use',
              usage: { window: 12, windowReset: 200, week: 30, weekReset: 'Sun 00:00', pace: 29 } },
            { name: 'robin-key', kind: 'key', use: 'Idle', usage: null },
          ],
        },
      ],

      tasks: {
        T3: {
          title: 'CSV import', by: 'Robin', template: 'goal', charter: 'Implementer', requester: 'Robin',
          model: 'Opus · high', account: 'Claude', hostKind: 'laptop',
          phase: 'idle', waitsOn: 'party', waitsFor: 'Robin',
          tracked: true, priority: 1, spend: 4.8, budget: 20, made: -1500,
          wakesOn: ["its delegates' questions and results", 'failures in its tree', 'your answers'],
          brief: 'Import CSV with quoted fields (RFC 4180), keeping the old parser’s behaviour where tests pin it.',
          input: 'tally · docs/csv.md @ 5d20c4e',
          writes: 'tally: src/csv/, tests/csv/',
        },
        // Each change is temper's change procedure; under it, the agent it
        // made to produce its branch.
        T4: {
          title: 'CSV import: the quoted-field parser', by: 'T3', kind: 'change', parent: 'T3', repo: 'tally',
          stage: 'checking', made: -1480, pr: 41,
        },
        T5: {
          title: "Produce T4's branch", by: 'T4', briefBy: 'T3', charter: 'Implementer', parent: 'T4',
          model: 'Sonnet · medium', account: 'Claude', hostKind: 'laptop',
          phase: 'done', made: -1480, ended: -40, spend: 2.1,
          brief: 'Replace the CSV splitter with a parser for quoted fields, escaped quotes and embedded newlines (docs/csv.md), with tests for each.',
          result: 'Pushed temper/3/csv-import-the-quoted-field-parser at 3c9e1f0.',
        },
        T6: {
          title: 'Why is tally sum slow on big files?', by: 'Robin', template: 'chat', charter: 'Assistant',
          model: 'Opus · high', account: 'Claude', hostKind: 'laptop', requester: 'Robin',
          phase: 'idle', waitsOn: 'news', wake: { words: true },
          spend: 0.8, budget: null, made: -130,
        },
        // A chat whose agent proposed a goal: the proposal is an entry in
        // its subtree, so it waits on Robin's answer, not on their words.
        T10: {
          title: 'Add a --json flag to tally report', by: 'Robin', template: 'chat', charter: 'Assistant',
          model: 'Opus · high', account: 'Claude', hostKind: 'laptop', requester: 'Robin',
          phase: 'idle', waitsOn: 'party', waitsFor: 'Robin', wake: { words: true },
          spend: 0.3, budget: null, made: -14,
        },
        T7: {
          title: 'Profile tally sum on a 2 GB file', by: 'T6', charter: 'Implementer', parent: 'T6',
          model: 'Sonnet · medium', account: 'Claude', hostKind: 'laptop',
          phase: 'done', ended: -90, spend: 0.5, made: -117,
          brief: 'Profile `tally sum` on a generated 2 GB CSV and name where the time goes.',
          result: '92% of the time is the CSV splitter allocating a string per line. A reused buffer cuts the run from 40 s to 1.8 s.',
        },
        T8: {
          title: 'CSV import: errors with line and column', by: 'T3', kind: 'change', parent: 'T3', repo: 'tally',
          stage: 'producing', made: -23,
        },
        T9: {
          title: "Produce T8's branch", by: 'T8', briefBy: 'T3', charter: 'Implementer', parent: 'T8',
          model: 'Sonnet · medium', account: 'Claude', hostKind: 'laptop', worker: 'laptop',
          phase: 'running', run: { n: 1, since: -22, doing: 'writing' },
          spend: 0.6, made: -23,
          brief: 'Give CSV import’s errors the line and column where they occur, counting embedded newlines, with tests.',
        },
      },

      children: {
        T3: ['T4', 'T8'],
        T4: ['T5'],
        T6: ['T7'],
        T8: ['T9'],
      },

      ended: [
        { task: 'T1', title: 'Make tally sum stream its input', by: 'Robin', charter: 'Assistant', template: 'chat', ended: -1590, spend: 0.9 },
      ],

      inbox: [
        { kind: 'proposal', id: 'P1', task: 'T10', at: -12,
          title: 'JSON output for tally report',
          text: 'A goal of yours: `tally report --json`, one object per row, with tests, landed into main.',
          why: 'It changes report’s output and its tests, and lands into main: more than a chat does alone.',
          batch: [{ title: 'JSON output for tally report', template: 'goal', charter: 'Implementer', budget: 6, landing: 'main in robin/tally', priority: 2 }],
          own: true },
        { kind: 'question', task: 'T3', at: -18,
          text: 'The old parser accepts a trailing comma as an empty last field. RFC 4180 does not. Keep it, or drop it in 0.5?',
          options: ['Keep it', 'Drop it in 0.5'] },
      ],

      transcripts: {
        T6: [
          { day: 'Today · Tuesday 27 October' },
          { who: 'you', at: -130, text: 'Why is tally sum slow on big files?\n2 GB takes 40 s.' },
          { who: 'agent', at: -118, read: true, text: "I'll profile it on a generated 2 GB file first." },
          { who: 'tools', at: -117, calls: [['delegate', 1]], made: ['T7'] },
          { who: 'agent', at: -88, text: 'T7 found it: **92%** of the time is the CSV splitter allocating a string per line.\n\nA reused buffer cuts the run from 40 s to 1.8 s. Shall I make that a change?' },
        ],
        T10: [
          { day: 'Today · Tuesday 27 October' },
          { who: 'you', at: -14, text: 'Add a --json flag to tally report\nOne object per row: my dashboard script reads it.' },
          { who: 'agent', at: -12, text: "That changes `report`'s output and its tests, and lands into main, so I've proposed it as a goal of yours, with a budget of its own." },
          { who: 'tools', at: -12, calls: [['read', 2], ['propose', 1]], proposes: 'P1' },
        ],
        // An agent's task: its run's turns, the last still streaming.
        T9: [
          { who: 'tools', at: -21, calls: [['read', 4], ['shell', 1]] },
          { who: 'agent', at: -15, text: 'Errors carry their line now, and an embedded newline moves it on. The column next: counted from the start of its line, quotes included.' },
          { who: 'tools', at: -14, calls: [['edit', 3], ['shell', 2]] },
          { who: 'agent', live: true, at: -1, text: 'The column counts bytes so far. A field is text, so I’ll count characters, and test a field with an accent before the bad quote' },
        ],
      },

      notes: [
        { scope: 'goal', goal: 'T3', by: 'T3', at: -1490, name: 'fixtures', text: 'The old parser’s tests in tests/legacy pin its quirks; read them before changing behaviour.' },
        { scope: 'goal', goal: 'T3', by: 'T3', at: -18, name: 'trailing comma', text: 'Open: the old parser’s trailing comma. Asked Robin.' },
      ],

      activity: {
        T3: [
          { at: -18, what: 'asked', task: 'T3', text: 'Robin a question: the trailing comma' },
          { at: -22, what: 'started', task: 'T9', text: "producing T8's branch, on laptop" },
          { at: -35, what: 'checking', task: 'T4', text: 'PR #41 opened; CI running at its head' },
          { at: -40, what: 'ended', task: 'T5', text: "done: pushed T4's branch at 3c9e1f0" },
        ],
      },
      history: {
        T3: [
          { at: -23, by: 'T3', what: 'delegated T8, a change', reason: 'Errors with line and column, beside the parser.' },
          { at: -1480, by: 'T3', what: 'delegated T4, a change', reason: 'The quoted-field parser, as one change.' },
          { at: -1500, by: 'Robin', what: 'made T3: tracked, priority 1, budget 20, host kind laptop' },
        ],
      },
    };
  }

  const steps = [
    {
      id: '0', label: 'Now', hour: 0, act: 1, title: 'A Tuesday morning',
      summary: 'Robin works alone on tally: a chat about a slow command, another whose agent proposed a goal, a goal they set by hand for CSV import, one repository and one laptop. The goal’s agent asks them a question.',
      look: [
        ['inbox.html', 'Inbox: a chat’s proposal, and the goal’s question'],
        ['chat.html?task=T6', 'A chat at rest, and its delegate’s card'],
        ['chat.html?task=T10', 'A chat waiting on its proposal'],
        ['board.html', 'Board: one goal, set by hand'],
        ['tasks.html', 'Tasks: two trees; each change with the agent producing it'],
        ['task.html?task=T3', 'CSV import: two changes, each with the agent producing it'],
        ['task.html?task=T9', 'An agent at work: its transcript, streaming'],
        ['changes.html', 'Changes: one repository, a change checking and one producing'],
        ['workers.html', 'Workers: one laptop, no ops application'],
        ['accounts.html', 'Accounts: a login, and a key'],
      ],
      apply() {},
    },
  ];

  MOCKUPS.worlds.solo = {
    name: 'Solo developer',
    about: 'one developer, one repository, one laptop: chats and a goal set by hand, no coordinator, timers, pools or ops application',
    HOUR0: Date.UTC(2026, 9, 27, 9, 30),
    initial, steps, upcoming: [], acts: { 1: 'One step' },
  };
})();
