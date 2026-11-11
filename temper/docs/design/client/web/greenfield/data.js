// The mockup's world: two projects at two scales. Everything a page
// shows comes from here, changed by what the viewer does (app.js keeps
// that in localStorage). Nothing is sent anywhere.
window.MOCK = {
  person: 'Owner',
  projects: {
    fabrig: {
      name: 'fabrig',
      repo: 'fabrig-ai/fabrig',
      forge: 'GitHub',
      parts: { drafts: true, goals: true, changes: true, away: true },
      awayUntil: '08:00',
      ways: [
        { id: 'whole', name: 'Plan, tasks, land whole', text: 'A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.', used: 11 },
        { id: 'each', name: 'Task by task', text: 'The same plan; each task lands on its own once checked.', used: 2 },
        { id: 'review', name: 'Plan, then wait for me', text: 'Proposes the plan and its tasks, and starts nothing until you approve.', used: 1 },
        { id: 'one', name: 'One agent, one branch', text: 'No plan: one agent works the draft through and opens one change.', used: 0 },
      ],
      defaultWay: 'whole',
      morning: {
        kicker: 'Tuesday, 08:04 · you last looked 9 hours ago',
        headline: '41 changes landed overnight.',
        text: 'first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two. welcome-stall found its fault in jig’s store. Nothing was decided for you.',
        by: 'Written by the status agent at 07:42',
      },
      away: {
        kicker: 'While you were away · 23:10 to now',
        headline: '57 changes landed.',
        text: 'first-run-projects stopped at 05:50 for funds, which always wait for you. welcome-stall landed its fix in temper serve.',
        by: 'Written by the status agent at 07:55',
      },
      landing: { line: '3 queued · last landed 6 min ago · main green at 07:36' },
      needs: [
        {
          id: 'n-stall', goal: 'welcome-stall', kind: 'question', asked: '40 min', scenario: 'morning',
          short: 'Fix the dropped watch in jig’s store, or in temper serve?',
          question: 'Fix the dropped watch in jig’s store, or keep it alive in temper serve?',
          context: 'MakeProject answers after the client’s watch drops. jig’s fix changes a stored record; serve’s fix is local but leaves jig’s behaviour as it is.',
          saw: 'The box test reproduces it every time: the store’s write lands, then the watch closes before the answer comes. The test and its log are on the goal.',
          choices: [
            { label: 'In jig’s store', rec: true, effect: { state: 'running', line: 'Fixing it in jig’s store: the record change and its migration are planned as one task.' } },
            { label: 'In temper serve', effect: { state: 'running', line: 'Fixing it in temper serve: the watch stays open until its answer arrives.' } },
          ],
        },
        {
          id: 'n-funds', goal: 'first-run-projects', kind: 'funds', asked: '2 h', scenario: 'morning',
          short: 'needs $8 more to finish',
          question: '$31 of $40 used. The split store task needs about $8 more to finish.',
          context: 'Both halves of the store task run, about $4 each. Without funds the goal stops after they land, with the Ready page undone.',
          choices: [
            { label: 'Add $10', rec: true, effect: { state: 'running', budget: 50, line: 'Funded to $50. Both halves of the store task run; the Ready page follows.' } },
            { label: 'Stop the goal', quiet: true, effect: { state: 'stopped', line: 'Stopped by you. What landed on the goal branch stays there.' } },
          ],
        },
        {
          id: 'n-link', goal: 'forge-cards', kind: 'question', asked: '3 h', scenario: 'morning',
          short: 'Link the GitHub help page, or hide it?',
          question: 'The “connect your deployment” help page isn’t written. Link the server help on GitHub for now, or hide the link until the page exists?',
          context: 'The welcome’s design names the link a placeholder.',
          choices: [
            { label: 'Link the GitHub page', rec: true, effect: { line: 'Linking the GitHub help page for now; three tasks run, one is queued.' } },
            { label: 'Hide the link', effect: { line: 'Hiding the link until the page exists; three tasks run, one is queued.' } },
          ],
        },
        {
          id: 'n-funds-away', goal: 'first-run-projects', kind: 'funds', asked: 'since 05:50', scenario: 'away',
          short: 'stopped for funds',
          question: '$40 of $40 used, with the store task’s halves done and two small tasks left, about $6.',
          context: 'The goal stopped at 05:50. Funds always wait for you, away or not.',
          choices: [
            { label: 'Add $10', rec: true, effect: { state: 'running', budget: 50, line: 'Funded to $50. The Ready page and the journey print are running.' } },
            { label: 'Land what’s done', effect: { state: 'running', line: 'Landing the eight checked tasks without the Ready page; a goal for it can follow.' } },
          ],
        },
      ],
      decided: [
        { id: 'd-stall', goal: 'welcome-stall', at: '01:05', question: 'Fix in jig’s store or in temper serve?', took: 'In temper serve', why: 'jig’s fix changes a stored record, which always waits for you; serve’s fix doesn’t. The jig fix can follow.', landed: '03:12', options: ['In jig’s store', 'In temper serve'] },
        { id: 'd-link', goal: 'forge-cards', at: '02:40', question: 'Link the GitHub help page, or hide the link?', took: 'Link the GitHub page', why: 'The agent’s own recommendation; the welcome’s design names the link a placeholder.', options: ['Link the GitHub page', 'Hide the link'] },
        { id: 'd-split', goal: 'first-run-projects', at: '05:40', question: 'The store task failed twice on size. Retry, or split it?', took: 'Split it in two', why: 'Splitting a failing task is allowed while you’re away.', options: ['Retry it as it is', 'Split it in two'] },
        { id: 'd-words', goal: 'forge-cards', at: '06:15', question: 'The welcome card says “Run on this machine”. Keep it, or “Set up here”?', took: 'Keep “Run on this machine”', why: 'A wording choice inside the task’s spec.', options: ['Keep “Run on this machine”', 'Use “Set up here”'] },
      ],
      goals: [
        {
          id: 'first-run-projects', state: 'running', way: 'whole', draft: 'first-run-projects.md',
          done: 6, total: 9, spent: 31, budget: 40, since: 'Mon 19:10',
          line: 'Six of nine tasks landed. The store task is split in two after two size failures, and needs about $8 more to finish.',
          short: 'Six of nine tasks landed; the store task is split in two after two size failures.',
          by: 'Written by its agent at 07:58, when the second half started',
          report: [
            'The project records, the project page’s empty state and the welcome’s Default project landed on the goal branch overnight, each checked. The Ready page waits on the store task.',
            'The store task adds the project policy record to jig’s store. It failed twice on the wasm client’s size limit, so I split it: the record and its migration first, the client’s lazy loading second. Both run now, about $4 each.',
          ],
          left: 'the two halves, the Ready page’s Open Default, and the welcome journey’s print.',
          risk: 'lazy loading touches every page that lists projects; its check runs the client’s journeys.',
          tasks: [
            { name: 'Project records', state: 'landed', when: 'Mon 21:40' },
            { name: 'Project page, empty state', state: 'landed', when: '00:15' },
            { name: 'Welcome’s Default project', state: 'landed', when: '02:02' },
            { name: 'Playground project', state: 'landed', when: '03:30' },
            { name: 'Banner’s set-up link', state: 'landed', when: '04:10' },
            { name: 'Pane’s playground line', state: 'landed', when: '05:01' },
            { name: 'Store: policy record', state: 'running', when: '22 min' },
            { name: 'Client: lazy project loading', state: 'running', when: '12 min' },
            { name: 'Ready page’s Open Default', state: 'waits', when: 'on the store' },
          ],
          landing: 'Lands whole once all nine are checked. The goal branch was last updated from main at 07:30, clean.',
          decided: ['Split the store task, 05:40', 'Retried it once, 05:12', 'Kept the Default project’s name in English, 01:20'],
          awayPatch: { state: 'stopped', done: 8, spent: 40, line: 'Stopped at 05:50 for funds: $40 of $40 used, with two small tasks left.', short: 'Stopped for funds at 05:50; two small tasks left.' },
        },
        {
          id: 'welcome-stall', state: 'running', way: 'each', chat: 'T2',
          done: 1, total: 3, spent: 4, budget: 15, since: '18:20',
          line: 'Reproduced in a box test; the fault is in jig’s store. Waiting on where to fix it.',
          short: 'Reproduced in a box test; the fault is in jig’s store.',
          by: 'Written by its agent at 07:24',
          report: [
            'The box test walks the welcome to Ready and presses Open Default. MakeProject’s write lands in the store, then the client’s watch closes before the answer arrives, so the page waits forever.',
            'Two fixes work in a spike: jig’s store answers before it closes the watch, which changes a stored record; or temper serve keeps the watch open until the answer. Both pass the box test.',
          ],
          left: 'the fix, wherever it goes, and the box test landing with it.',
          risk: 'none known beyond the choice itself.',
          tasks: [
            { name: 'Box test: Open Default', state: 'landed', when: '22:40' },
            { name: 'The fix', state: 'waits', when: 'on you' },
            { name: 'Welcome-set-up journey', state: 'waits', when: 'on the fix' },
          ],
          landing: 'Each task lands on its own once checked.',
          decided: ['Wrote the box test before the fix, 19:05'],
          awayPatch: { state: 'landed', done: 3, line: 'Fixed in temper serve and landed at 03:12; the jig fix is left for a later goal.', short: 'Fixed in temper serve, landed 03:12.' },
        },
        {
          id: 'forge-cards', state: 'running', way: 'whole', draft: 'forge-cards.md',
          done: 0, total: 4, spent: 3, budget: 20, since: '04:05',
          line: 'Four tasks planned from the draft; two are running and one waits on a question.',
          short: 'Four tasks planned from the draft; two are running.',
          by: 'Written by its agent at 07:10',
          report: ['The plan has four tasks: the two welcome cards, the owned Forgejo set up at once, the help link, and the tests. The cards and the set up run; the help link waits on whether to link GitHub’s page.'],
          left: 'all four tasks.', risk: 'the owned set up hard-codes work/hello until projects can make repositories.',
          tasks: [
            { name: 'Two welcome cards', state: 'running', when: '1 h' },
            { name: 'Owned Forgejo, set up at once', state: 'running', when: '40 min' },
            { name: 'The help link', state: 'waits', when: 'on you' },
            { name: 'Tests', state: 'waits', when: 'on the cards' },
          ],
          landing: 'Lands whole once all four are checked.',
          decided: ['Kept the GitHub card first, 04:30'],
          awayPatch: { done: 2, line: 'Two of four tasks checked; the help link is in and the tests run.', short: 'Two of four checked; the tests run.' },
        },
        {
          id: 'pane-chat-line', state: 'landed', way: 'one', chat: 'T3',
          done: 2, total: 2, spent: 5, budget: 10, since: 'Mon 22:30', ended: '01:12',
          line: 'Landed whole at 01:12: two tasks and one journey print.',
          short: 'Landed whole: two tasks and one journey print.',
          by: 'Written by its agent at 01:12',
          report: ['A chat in the side pane is one compact line: its title and when, then its status and T number, the whole line its link.'],
          left: 'nothing.', risk: 'none.',
          tasks: [{ name: 'Pane chat line', state: 'landed', when: '00:50' }, { name: 'Journey print', state: 'landed', when: '01:12' }],
          landing: 'Landed at 01:12.', decided: [],
        },
        {
          id: 'welcome-model-cards', state: 'landed', way: 'whole', draft: 'welcome-model.md',
          done: 3, total: 3, spent: 9, budget: 15, since: 'Mon 14:00', ended: 'Mon 18:40',
          line: 'Landed whole Monday at 18:40: the Model step is two cards, ChatGPT and Claude.',
          short: 'Landed Monday: the Model step is two cards.',
          by: 'Written by its agent on Monday at 18:40',
          report: ['The Model step offers ChatGPT, with its subscription or a key, and Claude, with its subscription.'],
          left: 'nothing.', risk: 'none.',
          tasks: [{ name: 'Model cards', state: 'landed', when: 'Mon 17:10' }, { name: 'Key way', state: 'landed', when: 'Mon 18:00' }, { name: 'Tests', state: 'landed', when: 'Mon 18:40' }],
          landing: 'Landed Monday at 18:40.', decided: [],
        },
      ],
      chats: [
        {
          id: 'T4', title: 'Away mode, draft', status: 'waiting on your words', live: true, when: '1 min', writes: 'away-mode.md',
          model: 'Claude, through your subscription · Opus',
          thread: [
            { who: 'you', text: 'Let’s draft away mode from the brainstorm: a setting, limits I set, and “decided for you” with an overturn.' },
            { who: 'agent', text: 'Outline: in one page, the setting, what agents may decide, decided for you, coming back. Writing it now.' },
            { who: 'tool', text: 'Wrote away-mode.md', detail: '64 lines', ref: 'one' },
            { who: 'you', text: 'Budgets, sign-ins, machines and anything hard to reverse always wait for me. Make the limits a list of kinds, not levels.' },
            { who: 'agent', text: 'Done: section 3 is a list of kinds, each on or off per project, and the four that always wait are fixed. One question stays open: how long a decision can be overturned once its change has landed.' },
            { who: 'tool', text: 'Edited away-mode.md', detail: '+18 −6', ref: 'kinds' },
            { who: 'agent', text: 'It reads complete apart from that question. If you start a goal from it now, the question comes back to you as the goal’s first.' },
          ],
          replies: [
            'Noted. I’ll fold that into section 4 and say what changed.',
            'Done; the change is highlighted in the draft.',
            'Good point. That belongs under open questions until you decide it.',
          ],
        },
        {
          id: 'T3', title: 'Pane chat line', status: 'closed', goal: 'pane-chat-line', live: false, when: '9 h',
          model: 'ChatGPT, through your plan · GPT-5',
          thread: [
            { who: 'you', text: 'The chats in the side pane lost their layout: title, time and status run together.' },
            { who: 'agent', text: 'The pane reuses the list’s grid rows. I’d make each chat one compact line: title and when, then status and its T number, the whole line its link. That touches temper.css and one journey print, so I’ve started it as a goal.' },
            { who: 'tool', text: 'Started goal pane-chat-line', detail: 'one agent, one branch' },
          ],
          replies: ['It landed at 01:12; the goal has the details.'],
        },
        {
          id: 'T2', title: 'Welcome stall', status: 'closed', goal: 'welcome-stall', live: false, when: '14 h',
          model: 'Claude, through your subscription · Opus',
          thread: [
            { who: 'you', text: 'The welcome stops at “Making Default…”. Find out why, then fix it with a box test.' },
            { who: 'agent', text: 'The store writes the project, then the link drops. That needs a box test first, then a fix in the lowest tier that shows it. Starting it as a goal, task by task.' },
            { who: 'tool', text: 'Started goal welcome-stall', detail: 'task by task' },
          ],
          replies: ['The goal is waiting on where to fix it; it’s in Needs you.'],
        },
        {
          id: 'T5', title: 'What changed in jig’s store this week?', status: 'closed', live: false, when: 'Mon',
          model: 'ChatGPT, through your plan · GPT-5',
          thread: [
            { who: 'you', text: 'What changed in jig’s store this week?' },
            { who: 'agent', text: 'Three things: project policy records, size limits on watches, and a faster compaction. The first is what first-run-projects is building on.' },
          ],
          replies: ['Anything else about the store?'],
        },
      ],
      drafts: [
        {
          id: 'away-mode.md', chat: 'T4', status: 'writing', updated: '1 min ago', path: 'docs/design/draft/away-mode.md',
          doc: [
            { id: 'top', html: '<h1>Away mode</h1><p class="doc-muted">Draft, 2026-11-03, with the user.</p>' },
            { id: 'one', html: '<h2>1. In one page</h2><ul><li>A setting, per project or for all: away until a time, or until turned off.</li><li>While away, agents decide what they would ask, within the kinds the project allows (section 3).</li><li>Each decision shows as decided for you, with Overturn while it is cheap (section 4).</li><li>Coming back opens on what was decided (section 5).</li></ul>' },
            { id: 'setting', html: '<h2>2. The setting</h2><p>The switch sits beside the person’s name, on the laptop and on the phone. On, it reads “Away until 08:00”; the time comes from the quiet hours, or one the person picks.</p>' },
            { id: 'kinds', html: '<h2>3. What agents may decide</h2><ul><li>Implementation choices within a task’s spec.</li><li>Retries, and splitting a failing task.</li><li>A design document’s wording, not its decisions.</li><li>Adding a dependency: off unless turned on.</li><li class="doc-muted">Always wait, fixed: budgets, sign-ins, machines, what is hard to reverse.</li></ul>' },
            { id: 'decided', html: '<h2>4. Decided for you</h2><p>Each decision keeps the question, the choice taken and why. It shows in Needs you as decided, with Overturn while overturning is cheap: before its change lands. Overturning asks the question again.</p>' },
            { id: 'back', html: '<h2>5. Coming back</h2><p>Home opens on what was decided, then what still needs you. Turning away off is the person’s; it never turns itself off.</p>' },
            { id: 'open', html: '<h2>Open questions</h2><ol><li>How long can a decision be overturned after its change landed?</li></ol>' },
          ],
          changed: 'kinds',
          openQuestion: { short: 'How long can a decision be overturned after it landed?', question: 'The draft leaves one question open: how long can a decision be overturned once its change has landed?', choices: [{ label: 'Until it lands, then only by a new goal', rec: true }, { label: 'For 24 hours, by reverting' }] },
        },
        {
          id: 'first-run-projects.md', goal: 'first-run-projects', status: 'goal running', updated: 'Mon', path: 'docs/design/draft/first-run-projects.md',
          doc: [
            { id: 'top', html: '<h1>First run: projects</h1><p class="doc-muted">Draft, 2026-11-02, with the user. Landed as the goal first-run-projects.</p>' },
            { id: 'one', html: '<h2>1. In one page</h2><ul><li>The welcome makes a Default project; the playground has its own.</li><li>A project page with no repository yet says so, and offers to create one.</li><li>Project records carry a policy, kept in jig’s store.</li></ul>' },
          ],
        },
        {
          id: 'forge-cards.md', goal: 'forge-cards', status: 'goal running', updated: '04:00', path: 'docs/design/draft/forge-cards.md',
          doc: [
            { id: 'top', html: '<h1>Forge cards</h1><p class="doc-muted">Draft, 2026-11-03, with the user.</p>' },
            { id: 'one', html: '<h2>1. In one page</h2><ul><li>The welcome’s Forge step is two cards: GitHub, and Forgejo.</li><li>Forgejo runs on this machine at once, or connects to a deployment (a help link).</li></ul>' },
          ],
        },
        {
          id: 'dashboards.md', status: 'idea', updated: 'Oct 28', path: 'docs/design/draft/dashboards.md',
          doc: [
            { id: 'top', html: '<h1>Dashboards</h1><p class="doc-muted">An idea, 2026-10-28. Not discussed yet.</p>' },
            { id: 'one', html: '<h2>1. In one page</h2><ul><li>Each person gets a page of what they follow across projects.</li><li>Written status from agents, beside counts temper keeps.</li><li>Builders and their capacity, for those who run them.</li></ul>' },
          ],
        },
      ],
      changes: {
        queue: [
          { title: 'Store: policy record', from: 'first-run-projects', state: 'checking', note: 'checks started 6 min ago' },
          { title: 'Two welcome cards', from: 'forge-cards', state: 'queued', note: '2nd' },
          { title: 'Help typo in server.md', from: 'chat T6', state: 'queued', note: '3rd' },
        ],
        landed: [
          { title: 'Pane’s playground line', from: 'first-run-projects', when: '05:01' },
          { title: 'Banner’s set-up link', from: 'first-run-projects', when: '04:10' },
          { title: 'Playground project', from: 'first-run-projects', when: '03:30' },
          { title: 'Journey print', from: 'pane-chat-line', when: '01:12' },
        ],
      },
    },

    hello: {
      name: 'hello',
      repo: 'work/hello',
      forge: 'this machine’s Forgejo',
      parts: { drafts: false, goals: false, changes: false, away: false },
      awayUntil: '08:00',
      ways: [
        { id: 'whole', name: 'Plan, tasks, land whole', text: 'A plan from the draft, its tasks on a goal branch, landed as one change.', used: 0 },
        { id: 'one', name: 'One agent, one branch', text: 'No plan: one agent works the draft through and opens one change.', used: 0 },
      ],
      defaultWay: 'one',
      morning: {
        kicker: 'Tuesday, 08:04 · you last looked an hour ago',
        headline: '“Hello in Rust” landed on main.',
        text: '“Add a README” is written and asks which license to name.',
        by: 'Written by T3’s agent, 4 min ago',
      },
      landing: { line: 'main last moved 1 h ago · nothing queued' },
      needs: [
        {
          id: 'n-license', chat: 'T3', kind: 'question', asked: '4 min',
          short: 'Which license should the README name?',
          question: 'The repository has no license. Name one in the README?',
          context: 'The README is written apart from this line.',
          choices: [
            { label: 'MIT', rec: true, effect: { status: 'landing', line: 'MIT it is; the README is landing.' } },
            { label: 'Apache-2.0', effect: { status: 'landing', line: 'Apache-2.0 it is; the README is landing.' } },
            { label: 'No license line', effect: { status: 'landing', line: 'No license line; the README is landing.' } },
          ],
        },
      ],
      decided: [],
      goals: [],
      chats: [
        {
          id: 'T3', title: 'Add a README', status: 'waiting on you', live: true, when: '4 min', model: 'ChatGPT, through your plan · GPT-5',
          thread: [
            { who: 'you', text: 'Add a README: what this is, how to build it, how to run it.' },
            { who: 'agent', text: 'Written: a one-line summary, cargo build and cargo run, and an example run. One thing is left: the repository has no license. Which should the README name?' },
            { who: 'tool', text: 'Wrote README.md', detail: '24 lines' },
          ],
          replies: ['Done; it’s on its way to main.'],
        },
        {
          id: 'T2', title: 'Hello in Rust', status: 'landed', live: false, when: '1 h', model: 'ChatGPT, through your plan · GPT-5',
          thread: [
            { who: 'you', text: 'Make this a hello world in Rust.' },
            { who: 'agent', text: 'Added Cargo.toml and src/main.rs; cargo run prints “Hello, world!”. It landed on main.' },
          ],
          replies: ['Anything else?'],
        },
        {
          id: 'T1', title: 'What is in this repository?', status: 'closed', live: false, when: '1 h', model: 'ChatGPT, through your plan · GPT-5',
          thread: [
            { who: 'you', text: 'What is in this repository?' },
            { who: 'agent', text: 'Only an empty initial commit so far.' },
          ],
          replies: ['Anything else?'],
        },
      ],
      drafts: [],
      changes: { queue: [], landed: [{ title: 'Hello in Rust', from: 'chat T2', when: '07:02' }] },
    },
  },
};

// The first run: a fresh temper box with real backends, from the terminal
// through the welcome's wizard to the project Ready makes. Times are the
// mockup's, shortened: the Forgejo set-up takes a minute, not fifteen
// seconds.
window.MOCK.firstRun = {
  link: 'http://127.0.0.1:8367/jig/v1/sign-in/link?secret=mock-q7RvT2xLwE5nZp9aYc3HsB7uFgJ1oVi4NtXe6Wk',
  forgejo: {
    n: 3, name: 'Forgejo on this machine', web: 'http://127.0.0.1:8368', org: 'work', repo: 'hello', login: 'temper',
    versions: { forgejo: '17.0.0', runner: '13.2.0' },
    steps: [
      { ms: 4000, doing: 'Fetching Forgejo 17.0.0 and its runner 13.2.0', done: 'Fetched Forgejo 17.0.0 and its runner 13.2.0' },
      { ms: 3000, doing: 'Preparing its database, temper’s admin and your account', done: 'Prepared its database, temper’s admin and your account' },
      { ms: 2500, doing: 'Starting Forgejo and its runner', done: 'Started Forgejo and its runner' },
      { ms: 2000, doing: 'Connecting as temper', done: 'Connected as temper' },
    ],
    // What it starts with, chosen in the sheet "Run on this machine" opens:
    // an empty repository (the default), copies of public repositories, or
    // nothing. Each adds its own steps after the connect.
    makeMs: 2000,
    copyMs: 2500,
    copyHint: 'github.com/owner/name',
  },
  github: { ms: 2200, account: 'owner-dev', repos: ['owner-dev/website', 'owner-dev/notes', 'owner-dev/budget-cli', 'owner-dev/dotfiles'] },
  models: {
    chatgpt: { name: 'ChatGPT plan', who: 'owner@example.com', model: 'gpt-6.1-sol · high effort', line: 'ChatGPT, through your plan · gpt-6.1-sol · high effort' },
    key: { name: 'OpenAI API key', who: 'billed to the key’s organization', model: 'gpt-6.1-sol · high effort', line: 'OpenAI API key · gpt-6.1-sol · high effort' },
    claude: { name: 'Claude subscription', who: 'claude-owner, signed in on this machine', model: 'Opus', line: 'Claude, through your subscription · Opus' },
  },
  // What Ready makes: an ordinary project on the stock policy, with no
  // repository, showing only what any way of working needs until the
  // person says how they work.
  project: {
    name: 'Default', repo: null, forge: null, fresh: true, chatBase: 0,
    parts: { drafts: false, goals: false, changes: false, away: false },
    awayUntil: '08:00',
    ways: [
      { id: 'whole', name: 'Plan, tasks, land whole', text: 'A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.', used: 0 },
      { id: 'each', name: 'Task by task', text: 'The same plan; each task lands on its own once checked.', used: 0 },
      { id: 'one', name: 'One agent, one branch', text: 'No plan: one agent works the request through and opens one change.', used: 0 },
    ],
    defaultWay: 'one',
    morning: { kicker: '', headline: '', text: '', by: '' },
    away: { kicker: '', headline: '', text: '', by: '' },
    landing: { line: 'nothing has landed yet' },
    needs: [], decided: [], goals: [], chats: [], drafts: [],
    changes: { queue: [], landed: [] },
  },
  // The configuring chat's answers: what it proposes depends on what the
  // person says, so that the same temper fits other ways of working.
  configure: {
    explain: 'temper can run work in a few ways. A chat per change: it works on a branch and lands once your checks pass. Or a draft first: a chat writes a design document with you, and a goal plans its tasks and lands them. And it can decide small things while you’re away, within limits you set. Which sounds like you? Your own words are fine.',
    proposals: {
      small: {
        intro: 'Here’s how I’ll run that. Nothing changes until you accept.',
        lines: [
          ['Each change is one chat: its agent works on a branch and opens one change.', 'Way: one agent, one branch'],
          ['A change lands once its CI passes, with no review by you first.', 'Landing: when checked'],
          ['Changes on their way to main get a page of their own.', 'Changes page: shown'],
          ['Questions wait for you; nothing is decided while you’re away.', 'Away switch: hidden'],
        ],
        enforced: 'branches, landing once checked, budgets',
        followed: 'how an agent writes and tests its change',
        parts: { drafts: false, goals: false, changes: true, away: false }, way: 'one',
      },
      drafts: {
        intro: 'Here’s how I’ll run that. Nothing changes until you accept.',
        lines: [
          ['A chat can write a design draft beside it, in docs/design/draft/.', 'Drafts: on'],
          ['A draft becomes a goal: a plan, its tasks on a goal branch, landed whole once all are checked.', 'Goals: shown · way: plan, tasks, land whole'],
          ['Changes on their way to main get a page of their own.', 'Changes page: shown'],
          ['While you’re away, agents may decide small things within limits, and tell you after.', 'Away switch: shown'],
        ],
        enforced: 'the goal branch, landing once checked, budgets, what waits for you',
        followed: 'how designers write drafts, and how producers work a task',
        parts: { drafts: true, goals: true, changes: true, away: true }, way: 'whole',
      },
    },
    starters: [
      'I work alone on small changes: one chat per change, landed once CI passes.',
      'I write a design draft first; agents plan its tasks and land them together once checked.',
      'Not sure yet. What can temper do?',
    ],
  },
};
