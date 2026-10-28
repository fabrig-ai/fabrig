// The welcome's real setup on GitHub: the same fresh deployment as the
// playground path, with no playground started, connecting GitHub by the
// manifest flow, then a ChatGPT plan. `github` is GitHub's side, which
// github.html draws: who Sam is there, their organizations and their
// repositories. Times are minutes from Thursday 22 October, 21:00.
(function () {
  const MOCKUPS = window.MOCKUPS;
  const first = MOCKUPS.worlds['first-run'];
  const SLUG = 'temper-7f3k2q';

  function initial() {
    return Object.assign(first.initial(), {
      github: {
        login: 'sam', name: 'Sam', app: SLUG,
        accounts: [
          { login: 'sam', kind: 'user', role: 'owner' },
          { login: 'sams-tools', kind: 'org', role: 'owner' },
          { login: 'brightline', kind: 'org', role: 'member' },
        ],
        repos: {
          sam: [['widget', false], ['notes', true], ['dotfiles', false], ['site', false]],
          'sams-tools': [['tally', false], ['tally-docs', false]],
          brightline: [['api', true], ['web', true]],
        },
      },
    });
  }

  // GitHub's choices, carried in the address as the person clicks
  // through: `granted`, the account the installation is on and the
  // repositories Sam gave it (`sam:all`, `sam:widget,notes`), and `pick`,
  // the one adopted at Ready.
  function choices(w, params) {
    const f = w.forges.find(f => f.n === 2);
    const install = f.app?.installs[0];
    const [account, names] = (params.get('granted') || '').split(':');
    const owned = w.github.accounts.some(a => a.login === account && a.role === 'owner');
    if (install && owned && names) {
      const all = w.github.repos[account].map(([name]) => name);
      install.account = account;
      install.all = names === 'all';
      install.names = install.all ? all : names.split(',').filter(n => all.includes(n));
      if (!install.names.length) install.names = [all[0]];
    }
    const repo = w.repos['sam/widget'];
    if (!repo || !install) return;
    const pick = install.names.includes(params.get('pick')) ? params.get('pick') : install.names[0];
    delete w.repos['sam/widget'];
    const name = `${install.account}/${pick}`;
    w.repos[name] = Object.assign(repo, { name, project: pick });
    Object.assign(f.setup, { org: install.account, repo: pick, project: pick });
    w.project = pick;
  }

  const steps = [
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'The link opened',
      summary: 'temper box started temper serve alone. The link opens the welcome; Sam chooses Set up temper, then GitHub’s Connect GitHub. The same choice is on Connect a forge, from the Forges page.',
      look: [
        ['welcome.html', 'The welcome: the playground, or set up'],
        ['welcome.html?at=forge', 'Connect a forge, in the wizard: Connect GitHub'],
        ['chats.html', 'Chats behind it: Project 1, no chat yet; the setup strip'],
        ['connect-forge.html', 'Connect a forge, from Forges: the same choice'],
        ['connect-forge.html?forge=2', 'An App of your own: its details, behind its link'],
      ],
      apply() {},
    },
    {
      id: '2', label: '2', hour: 1 / 60, act: 1, title: 'On GitHub: its App',
      summary: 'Connect GitHub sent the tab to temper’s manifest page, which posted temper’s manifest to GitHub as it loaded. GitHub’s form makes the App under Sam’s own account, always: nothing asks for an organization. Nothing in temper shows a connect under way.',
      look: [
        ['github.html?at=create&from=welcome', 'GitHub: Create GitHub App for sam'],
        ['forges.html', 'Forges: still no forge of yours'],
      ],
      apply() {},
    },
    {
      id: '3', label: '3', hour: 2 / 60, act: 1, title: 'On GitHub: where to install',
      summary: 'Sam created the App. GitHub sent the tab to temper’s redirect, which kept the App’s key and sent it on to the App’s install page. GitHub lists Sam’s account and organizations: the organization menu is GitHub’s. Had Sam come back to temper here, Connect a forge would offer Install on GitHub.',
      look: [
        ['github.html?at=install&from=welcome', 'GitHub: where to install temper’s App'],
        ['github.html?at=account&account=sam&from=welcome', 'GitHub: on sam, all its repositories or those selected'],
        ['github.html?at=account&account=brightline&from=welcome', 'GitHub: an organization Sam doesn’t own, a request'],
        ['github.html?at=refused&why=request&from=welcome', 'Back from a request: temper’s own page'],
        ['github.html?at=refused&why=ended&from=welcome', 'A link that has ended: temper’s own page'],
        ['connect-forge.html?forge=2', 'Connect a forge, had Sam come back: Install on GitHub'],
        ['forges.html', 'Forges: GitHub, connecting'],
      ],
      apply(w) {
        Object.assign(w.forges.find(f => f.n === 2), {
          mode: 'managed', state: 'connecting', login: `${SLUG}[bot]`, id: 248113907,
          app: { slug: SLUG, installs: [] },
        });
      },
    },
    {
      id: '4', label: '4', hour: 3 / 60, act: 1, title: 'Installed',
      summary: 'Sam installed temper’s App on sam, keeping GitHub’s default, all its repositories. GitHub sent the tab to temper’s setup route, which took the installation and sent the tab back to the wizard: choose the repository to start in. A forge of Sam’s is connected, so the playground, never started, is retired.',
      look: [
        ['welcome.html?at=forge&forge=2', 'The wizard: choose a repository to start in'],
        ['connect-forge.html?forge=2', 'Connect a forge: the same Ready'],
        ['github.html?at=installation&from=welcome', 'GitHub: the installation’s repositories, from “Choose them on GitHub”'],
        ['forges.html', 'Forges: GitHub, connected, its App and installation'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 2);
        f.state = 'connected';
        f.app.installs.push({ account: 'sam', id: 61022417, all: true, names: w.github.repos.sam.map(([name]) => name) });
        w.demo.state = 'retired';
        w.forges[0].state = 'retired';
      },
    },
    {
      id: '5', label: '5', hour: 4 / 60, act: 1, title: 'A repository adopted',
      summary: 'Sam chose sam/widget, and temper adopted it into the new project widget, named after it. The wizard moves on to the model.',
      look: [
        ['welcome.html?at=model', 'Connect a model: ChatGPT, or an API key'],
        ['connect-forge.html?forge=2', 'Connect a forge: sam/widget is ready'],
        ['forges.html', 'Forges: GitHub, with sam/widget in widget'],
      ],
      apply(w) {
        const f = w.forges.find(f => f.n === 2);
        f.setup = { org: 'sam', repo: 'widget', project: 'widget', newProject: true };
        w.project = 'widget';
        w.repos['sam/widget'] = { name: 'sam/widget', forge: 2, branch: 'main', tip: '5d0e9b1', ci: { state: 'passing', at: 4 }, after: null, queue: [], project: 'widget' };
      },
    },
    {
      id: '6', label: '6', hour: 6 / 60, act: 1, title: 'Ready',
      summary: 'Sam chose Continue with ChatGPT and signed in as sam@example.com. The wizard’s Ready: sam/widget is ready, in widget.',
      look: [
        ['welcome.html?at=ready', 'sam/widget is ready'],
        ['chats.html?project=widget', 'Chats in widget: no chat yet'],
        ['forges.html', 'Forges: GitHub alone'],
      ],
      apply(w, id) {
        w.accounts.unshift({ name: 'ChatGPT', plan: 'ChatGPT plan', provider: 'OpenAI', holder: 'Sam', models: ['gpt-6-luna'], at: id,
          sources: [{ name: 'sam@example.com', kind: 'sign-in', since: 6, use: 'Idle' }] });
        w.config.next = { account: 'ChatGPT', model: 'gpt-6-luna' };
      },
    },
  ];

  MOCKUPS.worlds['first-run-github'] = {
    name: 'First run, on GitHub',
    about: 'the owner’s first run after temper box, setting up temper on GitHub: Connect GitHub, GitHub’s own pages, a repository adopted, then a ChatGPT plan; the playground never starts',
    HOUR0: first.HOUR0, initial, choices,
    steps, upcoming: [], acts: { 1: 'Setting up' },
  };
})();
