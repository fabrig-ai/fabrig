// A deployment reached at a name of its own (draft sign-in.md): temper
// serve behind a TLS proxy, its people signing in with Google, and its
// managed Forgejo signing them in through temper. The names are examples.
// Times are minutes from Friday 23 October, 09:00 UTC.
(function () {
  const MOCKUPS = window.MOCKUPS = window.MOCKUPS || { worlds: {} };

  function initial() {
    return {
      now: 0,
      person: 'Sam',
      project: 'hello',
      signedIn: false,

      // The configuration's public address (draft sign-in.md, 2.2), and
      // who tries to sign in at each ending.
      site: {
        origin: 'https://temper.example.com',
        member: 'sam@example.com',
        login: 'sam',
        stranger: 'alex@example.org',
        forge: 1,
      },

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
        hello: {
          name: 'work/hello', forge: 1, branch: 'main',
          tip: 'a41c9e2',
          ci: { state: 'passing', at: -55 },
          after: null,
          queue: [],
        },
      },

      // Managed: run by its owner as a service, temper set up on it with
      // a site admin's token. `through`: its people sign in through
      // temper, its OpenID provider (draft sign-in.md, section 5).
      forges: [
        { n: 1, kind: 'Forgejo', name: 'Forgejo at git.example.com', web: 'https://git.example.com', mode: 'managed', state: 'connected', login: 'temper', id: 3, through: true,
          load: { used: 22, rate: 600, window: 'minute', reserve: 60, uses: [['fresh reads', 12], ['writes', 4], ['keeping up', 6], ['the slow pass', 0]], outbox: { waiting: 0, uncertain: 0 }, hooks: [['work/hello', -2]] } },
      ],

      landings: [],
      findings: [],

      workers: [
        {
          name: 'server', kind: 'server', what: 'temper.example.com', slots: 4,
          logins: [['claude-sam', 'signed in']],
        },
      ],

      accounts: [
        {
          name: 'Claude', provider: 'Anthropic', holder: 'Sam', plan: 'Claude subscription',
          sources: [
            { name: 'claude-sam', kind: 'program', program: 'Claude Code', hosts: ['server'], use: 'Idle', usage: null },
          ],
        },
      ],

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

  const steps = [
    {
      id: '1', label: '1', hour: 0, act: 1, title: 'Signed out',
      summary: 'Sam opens https://temper.example.com with no sign-in, and the stock front page offers Continue with Google. Opening git.example.com first, Forgejo’s sign-in bounces through temper’s, which says where it continues. Alex, who is not a member, is turned away after Google.',
      look: [
        ['sign-in.html', 'The front page: Continue with Google'],
        ['sign-in.html?from=forgejo', 'From Forgejo, signed out: sign in to continue to Forgejo'],
        ['sign-in.html?state=not-member', 'Back from Google, not a member'],
        ['sign-in.html?state=cancelled', 'Back from Google, cancelled'],
        ['sign-in.html#other-states', 'Every other ending, for reference'],
      ],
      apply() {},
    },
    {
      id: '2', label: '2', hour: 1 / 60, act: 1, title: 'Signed in',
      summary: 'Back from Google, temper minted Sam’s sign-in and opened the chats page. Open Forgejo ↗ on the Forges page signs Sam in to git.example.com through temper, with no page of temper’s in between. Signing out says that Forgejo keeps its own sign-in.',
      look: [
        ['chats.html', 'Chats: signed in, no chat yet'],
        ['forges.html', 'Forges: Forgejo at git.example.com, opened signed in through temper'],
        ['sign-in.html?from=forgejo', 'From Forgejo, signed in: continuing to Forgejo'],
        ['sign-in.html?state=signed-out', 'Signed out: Forgejo keeps its own sign-in'],
      ],
      apply(w) { w.signedIn = true; },
    },
  ];

  MOCKUPS.worlds['google-sign-in'] = {
    name: 'Google sign-in',
    about: 'a deployment at its own name behind a TLS proxy: people sign in with Google, and its managed Forgejo signs them in through temper (draft sign-in.md); drawn ahead of temper',
    HOUR0: Date.UTC(2026, 9, 23, 9, 0),
    initial, steps, upcoming: [], acts: { 1: 'Signing in' },
  };
})();
