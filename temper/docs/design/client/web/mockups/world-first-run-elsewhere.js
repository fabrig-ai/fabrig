// The configured string-managed Forgejo, after the configuration's restart.
// Every stop returns to the same Details, with only the token emptied.
(function () {
  const MOCKUPS = window.MOCKUPS;
  const first = MOCKUPS.worlds['first-run'];
  function initial() {
    const w = first.initial();
    w.forges = w.forges.filter(f => f.kind === 'GitHub');
    w.forges.push({ n: 3, kind: 'Forgejo', mode: 'managed', admin: true,
      web: 'git.example.org', state: 'not connected', login: 'temper', id: 7,
      setup: { org: 'work', repo: 'hello', project: 'hello', label: 'docker', newProject: true } });
    return w;
  }
  // Each stop is a state of Details, at `?stop=`: the four repository
  // fields kept, and no token drawn.
  const STOPS = [
    ['required', 'Token required'], ['too-long', 'Token over 4 KiB'], ['label', 'Invalid runner label'],
    ['org-required', 'Organization required'], ['repository-required', 'Repository required'],
    ['not-admin', 'Not a site admin’s token'], ['401', 'Refused, 401'], ['403-bot', 'Refused 403: making the bot'],
    ['403-org', 'Refused 403: making the organization'], ['unmarked', 'The bot’s login held, unmarked'], ['other', 'Another stop: 500'],
    ['first-admin', 'The first repository stopped: checking the token'], ['first-org', 'The first repository stopped: its organization'],
    ['first-repository', 'The first repository stopped: its repository'], ['first-file', 'The first repository stopped: its starter workflow'],
  ];
  const both = (query, label) => [[`welcome.html?at=forge${query}`, label], [`connect-forge.html${query ? `?${query.slice(1)}` : ''}`, `${label}, from Forges`]];
  const steps = [
    { title: 'Choose', state: 'not connected', summary: 'The configuration was restarted: its Forgejo has its own card.', look: both('', 'Connect a forge: Forgejo at git.example.org') },
    { title: 'Details', state: 'not connected', summary: 'Sam chooses Connect as its admin: the site admin’s token, and the first repository’s fields.',
      look: [...both('&forge=3', 'Connect as its admin'), ...STOPS.map(([stop, label]) => [`welcome.html?at=forge&forge=3&stop=${stop}`, label])] },
    { title: 'Connecting', state: 'connecting', summary: 'temper makes its bot, then work/hello.', look: both('&forge=3', 'Connecting as its admin') },
    { title: 'Ready', state: 'connected', summary: 'temper made its bot, work/hello and the project hello.', look: [...both('&forge=3', 'Connected, managed'), ['welcome.html?at=model', 'Connect a model: the forge ticked']] },
  ].map(({ title, state, summary, look }, i) => ({
    id: String(i + 1), label: String(i + 1), hour: i / 60, act: 1, title, summary, look,
    apply(w) {
      w.forges.find(f => f.n === 3).state = state;
      if (state === 'connected') {
        w.project = 'hello';
        w.repos['work/hello'] = { name: 'work/hello', forge: 3, branch: 'main', tip: '5d0e9b1', project: 'hello', ci: { state: 'waiting' }, queue: [] };
      }
    },
  }));
  MOCKUPS.worlds['first-run-elsewhere'] = {
    name: 'First run, on a Forgejo elsewhere',
    about: 'the configured Forgejo after restart: its admin token, first repository, every stop and Ready',
    HOUR0: first.HOUR0, initial, steps, upcoming: [], acts: { 1: 'Connecting a Forgejo elsewhere' },
  };
})();
