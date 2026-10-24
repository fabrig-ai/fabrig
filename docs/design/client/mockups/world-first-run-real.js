// The welcome's real setup: the same fresh deployment as the playground path,
// with no playground started. Times are minutes from Thursday 22 October, 21:00.
(function () {
  const MOCKUPS = window.MOCKUPS;
  const first = MOCKUPS.worlds['first-run'];
  const steps = [
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'The link opened',
      summary: 'temper box started temper serve alone. The link opens the welcome; Sam chooses Set up temper, then Forgejo on this machine, and keeps its defaults.',
      look: [
        ['welcome.html', 'The welcome: the playground, or set up'],
        ['welcome.html?at=forge', 'Connect a forge'],
        ['welcome.html?at=forge&forge=3', 'Forgejo on this machine: its defaults'],
        ['welcome.html?at=forge&forge=elsewhere', 'A Forgejo elsewhere: how to add it'],
        ['welcome.html?at=ready', 'Ready, opened by its address: no forge, no model yet'],
        ['chats.html', 'Chats behind it: Project 1, no chat yet; the setup strip'],
      ],
      apply() {},
    },
    {
      id: '2', label: '2', hour: 1 / 60, act: 1, title: 'Setting up a Forgejo',
      summary: 'temper box fetches Forgejo and its runner. The wizard moves on to the model while it does: ChatGPT, or an OpenAI API key.',
      look: [
        ['welcome.html?at=model', 'Connect a model, while Forgejo sets up: ChatGPT, or an API key'],
        ['welcome.html?at=model&way=key', 'An OpenAI API key: its Details'],
        ['welcome.html?at=model&way=chatgpt&paste=waiting', 'Signing in, come back before its end: the paste folded'],
        ['welcome.html?at=model&way=chatgpt&paste=waiting&elsewhere=1', 'This browser isn’t on temper’s computer: the paste'],
        ['welcome.html?at=model&way=chatgpt&paste=written&elsewhere=1', 'The paste: the whole address pasted, hidden'],
        ['welcome.html?at=model&way=chatgpt&paste=finishing&elsewhere=1', 'The paste: finishing'],
        ['welcome.html?at=model&way=chatgpt&paste=done&elsewhere=1', 'The paste: signed in, in the wizard'],
        ['welcome.html?at=ready&end=declined', 'Back from ChatGPT: signed in without plan usage'],
        ['welcome.html?at=ready&end=held', 'Back from ChatGPT: another person holds that account'],
        ['welcome.html?at=ready&end=misconfigured', 'Back from ChatGPT: OpenAI refused temper’s sign-in'],
        ['welcome.html?at=ready&end=port-taken', 'Back from ChatGPT: its listener’s port taken'],
        ['welcome.html?at=ready&end=not-eligible', 'Back from ChatGPT: not eligible for plan usage (future)'],
        ['forges.html', 'Forges: forge 3, setting up; no playground'],
        ['chats.html', 'Chats: Project 1, no chat yet; the setup strip’s status'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        f.state = 'setting up';
        f.setup.step = 'fetching';
      },
    },
    {
      id: '3', label: '3', hour: 3 / 60, act: 1, title: 'Signed in',
      summary: 'Sam chooses Continue with ChatGPT: ChatGPT’s sign-in opens in the same tab, in the browser on this machine. Sam approves, and temper’s listener on 127.0.0.1:1455 sends the browser back to the wizard: signed in as sam@example.com. The forge is still starting, so the wizard is at Almost ready.',
      look: [
        ['welcome.html?at=ready', 'Almost ready: the model, ChatGPT; waiting for Forgejo'],
        ['accounts.html', 'Accounts: ChatGPT'],
        ['chats.html', 'Chats: Project 1, no chat yet; the strip, no forge yet'],
      ],
      apply(w, id) {
        w.accounts.unshift({ name: 'ChatGPT', plan: 'ChatGPT plan', provider: 'OpenAI', holder: 'Sam', models: ['gpt-6-luna'], at: id,
          sources: [{ name: 'sam@example.com', kind: 'sign-in', since: 3, use: 'Idle' }] });
        w.config.next = { account: 'ChatGPT', model: 'gpt-6-luna' };
        w.forges.find(f => f.n === 3).setup.step = 'starting';
      },
    },
    {
      id: '4', label: '4', hour: 4 / 60, act: 1, title: 'Ready',
      summary: 'temper connected as the forge’s bot, made work/hello and adopted it into the new project hello. A forge of Sam’s is connected, so the playground, never started, is retired: nothing of it shows.',
      look: [
        ['welcome.html?at=ready', 'work/hello is ready'],
        ['welcome.html', 'The link, the first run over: Chats in its place'],
        ['chats.html?project=hello', 'Chats in hello: no chat yet, no strip'],
        ['forges.html', 'Forges: forge 3 alone'],
        ['accounts.html', 'Accounts: ChatGPT alone'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 3);
        Object.assign(f, { state: 'connected', login: 'temper', id: 2,
          load: { used: 14, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 8], ['writes', 3], ['keeping up', 3], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['work/hello', 18]] } });
        delete f.setup.step;
        w.project = 'hello';
        w.repos.hello = { name: 'work/hello', forge: 3, branch: 'main', tip: 'a41c9e2', ci: { state: 'passing', at: 18 }, after: null, queue: [], project: 'hello' };
        w.demo.state = 'retired';
        w.forges[0].state = 'retired';
      },
    },
  ];

  MOCKUPS.worlds['first-run-real'] = {
    name: 'First run, set up',
    about: 'the owner’s first run after temper box, choosing to set up temper: a Forgejo on this machine and a ChatGPT plan, through the welcome’s wizard; the playground never starts',
    HOUR0: first.HOUR0, initial: first.initial,
    steps, upcoming: [], acts: { 1: 'Setting up' },
  };
})();
