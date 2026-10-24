// A third world: a person's first run, after `temper box` (setup.md, 8.2
// to 8.4). The default document's deployment: one project, the chat and
// coding charters, one worker in temper's process, three forges (the
// demo's fake, GitHub, an owned Forgejo not set up). The welcome offers
// the playground or real setup; neither has started yet. The scripted model plays
// its scripts, so every word an agent writes here is setup.md 8.4's. Times are minutes from Thursday
// 22 October, 21:00 UTC.
(function () {
  const MOCKUPS = window.MOCKUPS = window.MOCKUPS || { worlds: {} };

  const CSV = 'Add CSV export to widget';
  const DARK = 'Add a dark theme to widget';

  function initial() {
    return {
      now: 0,
      person: 'Sam',
      // The project `temper init` seeds, unnamed, so "Project 1"
      // (setup.md, 8.3); with none until the playground's start or the
      // first repository makes one (draft first-run.md, 2.4), its place
      // reads "No project yet", under View → Future.
      project: 'Project 1',
      seeded: 'Project 1',

      config: {
        templates: { chat: 'chat', goal: 'goal' },
        charters: ['chat', 'coding'],
        period: 'month',
        allotment: 100,
        ops: false,
        // The default document's providers: OpenAI, by its ChatGPT sign-in
        // and by keys (setup.md, 8.1), and the playground's. The ways are
        // client README 5.5's, the playground's provider left out.
        ways: ['Continue with ChatGPT', 'Use an API key'],
        providers: ['OpenAI'],
        restart: 'temper box',
      },

      // The playground, as the forge's side reads it (client README, 4.8), and
      // its cues, copied in (setup.md, 8.4). Each cue the mockup plays is
      // drawn at a step.
      demo: {
        state: 'not started', provider: 'Playground', forge: 1, repo: 'widget',
        cues: [CSV, DARK, 'Fix widget’s empty search state'],
        plays: { [CSV]: ['2', 'T1'], [DARK]: ['4', 'T4'] },
      },

      repos: {},

      // The default document's forges, by number (setup.md, 8.3).
      forges: [
        { n: 1, kind: 'Forgejo', name: 'Playground forge', web: 'http://127.0.0.1:8369', mode: 'fake', state: 'not started' },
        { n: 2, kind: 'GitHub', web: 'https://github.com', mode: 'connected', state: 'not connected' },
        { n: 3, kind: 'Forgejo', web: 'http://127.0.0.1:8368', mode: 'owned', state: 'not set up', box: true, versions: { forgejo: '17.0.0', runner: '13.2.0' }, label: 'temper-host', person: 'sam', setup: { org: 'work', repo: 'hello', project: 'hello', newProject: true } },
      ],

      landings: [],
      findings: [],

      workers: [
        { name: 'worker', kind: 'workers', what: 'In temper’s process', slots: 2, logins: [] },
      ],

      accounts: [],

      tasks: {},
      children: {},
      ended: [],
      inbox: [],
      transcripts: {},
      notes: [],
      activity: {},
      history: {},
    };
  }

  const DAY = { day: 'Today · Thursday 22 October' };

  const steps = [
    {
      id: '0', label: '0', hour: 0, act: 1, title: 'The link opened',
      summary: 'temper box started temper serve alone: the playground is configured but not started, and no forge or model is Sam’s. The link opens the welcome: try the playground, or set temper up.',
      look: [
        ['welcome.html', 'The welcome: the playground, or set up'],
        ['chats.html', 'Chats behind it: Project 1, no chat yet; the setup strip'],
      ],
      apply() {},
    },
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'The playground chosen',
      summary: 'Sam chooses Try the playground. The engine asks temper box for the playground’s fakes, makes the project Playground, adds the scripted model’s account and adopts playground/widget into it itself, and the link’s page is the playground’s start.',
      look: [
        ['chats.html', 'Chats: the playground’s start'],
        ['accounts.html', 'Accounts: none of Sam’s yet; the scripted model, the playground’s'],
        ['forges.html', 'Forges: no forge of yours yet, and the playground’s, folded'],
        ['workers.html', 'Workers: one, in temper’s process'],
        ['changes.html', 'Changes: playground/widget on Playground forge, nothing in flight'],
      ],
      apply(w) {
        w.project = 'Playground';
        w.demo.state = 'running';
        w.accounts.push({ name: 'Playground', display: 'Scripted model', provider: 'Playground', holder: 'the deployment',
          sources: [{ name: 'playground', kind: 'key', use: 'Idle', usage: null }] });
        Object.assign(w.forges[0], { n: 1, kind: 'Forgejo', name: 'Playground forge', web: 'http://127.0.0.1:8369', mode: 'fake', state: 'connected', login: 'temper', id: 2,
          load: { used: 9, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 5], ['writes', 0], ['keeping up', 4], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['playground/widget', 0]] } });
        w.repos.widget = {
          name: 'playground/widget', forge: 1, branch: 'main', tip: '9f2c0a7',
          ci: { state: 'passing', at: 0 }, after: null, queue: [],
        };
      },
    },
    {
      id: '2', label: '2', hour: 2 / 60, act: 1, title: 'A request sent',
      summary: 'Sam picks “Add CSV export to widget”. The chat starts with those words, T1. Its agent delegates the change T2, whose agent, T3, edits src/main.rs. The card says Running, with no step ticked: the steps are a stand-in until the change’s live stages.',
      look: [
        ['chat.html?task=T1', 'T1: the announcement, and T2’s card, Running'],
        ['task.html?task=T2', 'T2: the change, producing its branch'],
        ['task.html?task=T3', 'T3: the coding agent, calling its tools'],
        ['tasks.html', 'Tasks: the chat, its change and the change’s agent'],
        ['chats.html', 'Chats: the playground’s start, and the chat under it'],
      ],
      apply(w, id) {
        w.tasks.T1 = {
          title: CSV, by: 'Sam', template: 'chat', charter: 'chat', requester: 'Sam',
          model: 'Scripted model', account: 'Playground', hostKind: 'workers',
          phase: 'idle', waitsOn: 'delegates', spend: 0, budget: null, made: 1, at: id,
        };
        w.tasks.T2 = {
          title: "Export widget's table as CSV", by: 'T1', kind: 'change', parent: 'T1', repo: 'widget',
          stage: 'producing', made: 1, at: id,
        };
        // Its brief is the change's title, the delegate's words (setup.md,
        // 8.4), so its chip is that title too.
        w.tasks.T3 = {
          title: "Export widget's table as CSV", by: 'T2', briefBy: 'T1', charter: 'coding', parent: 'T2',
          model: 'Scripted model', account: 'Playground', hostKind: 'workers', worker: 'worker',
          phase: 'running', run: { n: 1, since: 1, doing: 'calling edit' },
          spend: 0, made: 1, at: id,
          brief: "Export widget's table as CSV",
        };
        w.children = { T1: ['T2'], T2: ['T3'] };
        w.accounts[0].sources[0].use = 'In use';
        w.transcripts.T1 = [
          DAY,
          { who: 'you', at: 1, text: CSV },
          { who: 'agent', at: 1, text: 'I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.' },
          { who: 'tools', at: 1, calls: [['delegate', 1]], made: ['T2'] },
          { who: 'tools', at: 1, calls: [['wait', 1]] },
        ];
        w.transcripts.T3 = [
          { who: 'tools', at: 1, calls: [['read', 1]] },
        ];
      },
    },
    {
      id: '3', label: '3', hour: 4 / 60, act: 1, title: 'The change landed',
      summary: 'T3 finished with the change, the worker pushed its branch, and T2 opened its pull request, passed the fake’s CI and landed into main. Its result woke T1, which says so in its script’s words. The card says Landed, every step ticked, and the merge commit.',
      look: [
        ['chat.html?task=T1', 'T1: T2’s card, Landed, and the chat’s last words'],
        ['task.html?task=T2', 'T2: landed'],
        ['task.html?task=T3', 'T3: its result'],
        ['changes.html', 'Changes: the landing on playground/widget'],
        ['inbox.html', 'Inbox: nothing needs Sam; T1 replied'],
      ],
      apply(w, id) {
        Object.assign(w.tasks.T3, { phase: 'done', run: null, worker: null, ended: 3, at: id,
          result: '**Add CSV export to widget**\n\n`widget --csv` prints the table as CSV: a header line, `name,count`, then a line for each row.' });
        Object.assign(w.tasks.T2, { stage: 'landed', pr: 1, merge: 'e3b8d4192c0f5a7e', ended: 4, at: id });
        Object.assign(w.tasks.T1, { waitsOn: 'news', wake: { words: true }, at: id });
        w.accounts[0].sources[0].use = 'Idle';
        Object.assign(w.repos.widget, { tip: 'e3b8d41', ci: { state: 'passing', at: 4 } });
        w.landings.unshift({ task: 'T2', repo: 'widget', at: 4, title: "Export widget's table as CSV" });
        w.transcripts.T1[2].read = true;
        w.transcripts.T1.push(
          { who: 'agent', at: 4, text: 'CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.' },
          { who: 'tools', at: 4, calls: [['wait', 1]] },
        );
        w.transcripts.T3.push(
          { who: 'tools', at: 2, calls: [['edit', 1]] },
          { who: 'tools', at: 3, calls: [['finish', 1]] },
        );
      },
    },
    {
      id: '4', label: '4', hour: 6 / 60, act: 1, title: 'A text-only request',
      summary: 'Back on the playground’s start, Sam picks “Add a dark theme to widget”. Its script is one turn: an answer in words, then finish, as setup.md 8.4 has it. A chat is ended by its requester, so finish ends only the turn: T4 waits at rest for Sam’s next words, its composer with it, as T1 does.',
      look: [
        ['chat.html?task=T4', 'T4: answered, at rest, its composer open'],
        ['chats.html', 'Chats: T1 and T4 at rest'],
      ],
      apply(w, id) {
        w.tasks.T4 = {
          title: DARK, by: 'Sam', template: 'chat', charter: 'chat', requester: 'Sam',
          model: 'Scripted model', account: 'Playground', hostKind: 'workers',
          phase: 'idle', waitsOn: 'news', wake: { words: true }, spend: 0, budget: null, made: 5, at: id,
        };
        w.transcripts.T1[w.transcripts.T1.length - 2].read = true;
        w.transcripts.T4 = [
          DAY,
          { who: 'you', at: 5, text: DARK },
          { who: 'agent', at: 5, text: 'I’d give widget’s table a second set of colours, chosen when your system prefers a dark theme, so that widget follows your system. The playground plays only the CSV export as a change: connect a model to make this one.' },
          { who: 'tools', at: 5, calls: [['finish', 1]] },
        ];
      },
    },
    {
      id: 'model-choose', label: '4a', hour: 10 / 60, act: 2, title: 'Connect a model, from another computer',
      summary: "Sam’s browser is on another computer. Choose offers Continue with ChatGPT and the secondary button This browser isn't on temper's computer ›.",
      look: [['connect-model.html', 'Choose: the two sign-in buttons']],
      apply() {},
    },
    {
      id: 'model-elsewhere', label: '4b', hour: 11 / 60, act: 2, title: 'Signing in, with the browser elsewhere',
      summary: 'The secondary button keeps temper at Signing in. Sam opens the sign-in in another tab; ChatGPT’s return ends at a page that cannot be reached on Sam’s computer. The paste is unfolded in temper. The mockup’s external link is a fake provider’s.',
      look: [['connect-model.html?way=chatgpt', 'Signing in: open the sign-in, then paste'], ['accounts.html', 'The same sign-in under way; the paste folded']],
      apply(w) { w.modelSignIn = { state: 'waiting', elsewhere: true }; },
    },
    {
      id: 'model-pasted', label: '4c', hour: 12 / 60, act: 2, title: 'The whole address pasted',
      summary: 'Sam copies the whole address of the page that could not be reached, then pastes it into temper. The secret field shows only that it was written, and Finish connecting is enabled. Press it to see Ready at step 5; nothing is sent.',
      look: [['connect-model.html?way=chatgpt', 'The paste written, hidden; Finish connecting'], ['accounts.html', 'The paste in the sign-in under way, folded']],
      apply(w) { w.modelSignIn = { state: 'written', elsewhere: true }; },
    },
    {
      id: '5', label: '5', hour: 15 / 60, act: 2, title: 'A model connected',
      summary: 'Sam follows the banner’s “Connect a model”, chooses ChatGPT, and finishes the sign-in by pasting from the browser on another computer. Another provider’s account is usable, so the model’s half of the playground ends: the banner says a playground forge alone, the chats page is a chats page, and T1’s next run takes the first of its charter’s models with an account, gpt-6-luna on the ChatGPT plan (setup.md, 8.3). The scripted model steps aside: it is gone from Accounts.',
      look: [
        ['connect-model.html?way=chatgpt', 'Connect a model: signed in to ChatGPT'],
        ['accounts.html', 'Accounts: ChatGPT · sam@example.com, folded; the scripted model gone'],
        ['chats.html', 'Chats: no playground’s start; the banner, a playground forge alone'],
        ['chat.html?task=T1', 'T1: its composer’s line, Using ChatGPT plan'],
        ['connect-model.html#other-states', 'Connect a model, below: the sign-in’s other endings, a key and Claude'],
        ['accounts.html#other-states', 'Accounts, below: the ChatGPT row in each state, and a key’s and Claude’s rows'],
        ['chats.html#other-states', 'Chats, below: a plan’s first use, and a limited plan'],
        ['connect-forge.html', 'Connect a forge: GitHub, Forgejo on this machine, a Forgejo elsewhere'],
        ['connect-forge.html?forge=3', 'Forgejo on this machine: the set-up’s details'],
        ['forges.html', 'Forges: still no forge of yours'],
      ],
      apply(w, id) {
        delete w.modelSignIn;
        w.accounts = w.accounts.filter(a => a.provider !== w.demo.provider);
        w.accounts.unshift({ name: 'ChatGPT', plan: 'ChatGPT plan', provider: 'OpenAI', holder: 'Sam', models: ['gpt-6-luna'], at: id,
          sources: [{ name: 'sam@example.com', kind: 'sign-in', since: 14, use: 'Idle' }] });
        Object.assign(w.tasks.T1, { model: 'gpt-6-luna', account: 'ChatGPT' });
        w.config.next = { account: 'ChatGPT', model: 'gpt-6-luna' };
      },
    },
    {
      id: '6', label: '6', hour: 16 / 60, act: 2, title: 'Setting up a Forgejo',
      summary: 'Sam follows “Connect a forge”, chooses Forgejo on this machine and keeps its defaults: the organization work, the repository hello, and a new project named after it. temper box has fetched Forgejo and its runner and is preparing them; the flow shows each step, and the banner says where it is.',
      look: [
        ['connect-forge.html?forge=3', 'Connect a forge: setting up, at its preparation'],
        ['chats.html', 'Chats: the banner’s status'],
        ['forges.html', 'Forges: forge 3, setting up'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        f.state = 'setting up';
        f.setup.step = 'preparing';
      },
    },
    {
      id: '7', label: '7', hour: 19 / 60, act: 2, title: 'A Forgejo connected',
      summary: 'Forgejo and its runner started. temper connected as its bot, made work/hello with a README and a CI workflow, and adopted it into the new project hello; the workflow’s first run passed. The flow ends on the work: Sam opens hello, a project with no chat yet. A real model and a real forge are connected, so the playground’s banner is gone. Sam’s first forge is connected, so the playground ends: its forge stops and leaves the Forges page, its open chats end, and playground/widget leaves Playground; what the playground did stays readable there.',
      look: [
        ['connect-forge.html?forge=3', 'Connect a forge: ready, and Open hello'],
        ['chats.html?project=hello', 'Chats in hello: no chat yet, no banner'],
        ['forges.html', 'Forges: forge 3 alone'],
        ['chats.html', 'Chats in Playground: the playground’s chats, ended'],
        ['chat.html?task=T1', 'T1: ended with the playground'],
        ['changes.html', 'Changes in Playground: the playground’s landing kept'],
        ['changes.html?project=hello', 'Changes in hello: work/hello, CI passing'],
        ['connect-forge.html', 'Connect a forge: GitHub and a Forgejo elsewhere left'],
        ['connect-forge.html#other-states', 'Connect a forge, below: its other paths'],
        ['forges.html#other-states', 'Forges, below: other rows, and a managed and a connected Forgejo'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        Object.assign(f, { state: 'connected', login: 'temper', id: 2,
          load: { used: 14, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 8], ['writes', 3], ['keeping up', 3], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['work/hello', 18]] } });
        delete f.setup.step;
        w.repos.hello = { name: 'work/hello', forge: 3, branch: 'main', tip: 'a41c9e2', ci: { state: 'passing', at: 18 }, after: null, queue: [], project: 'hello' };
        w.demo.state = 'retired';
        w.forges[0].state = 'retired';
        w.repos.widget.retired = true;
        for (const id of ['T1', 'T4']) {
          Object.assign(w.tasks[id], { phase: 'done', ended: 19,
            endedWhy: 'the playground ended when Forgejo on this machine connected' });
        }
      },
    },
  ];

  MOCKUPS.worlds['first-run'] = {
    name: 'First run',
    about: 'the owner’s first run after temper box: the welcome, the playground chosen and its end once a forge connects, a suggested request landed on the playground forge by the scripted model’s script, a text-only one, then a ChatGPT plan signed in to by pasting from another computer, and a Forgejo set up here',
    HOUR0: Date.UTC(2026, 9, 22, 21, 0),
    initial, steps, upcoming: [], acts: { 1: 'The playground', 2: 'Going real' },
  };
})();
