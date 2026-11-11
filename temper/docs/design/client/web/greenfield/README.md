# The greenfield mockup

Drawn 2026-11-03 and 04 with the user, as temper's client imagined from
nothing. It is one working page set, drawn from `data.js` by `app.js`,
kept in the viewer's browser; nothing is sent anywhere. The "Mockup"
button at the bottom right changes its scenario: the morning after an
overnight run, a project at work, or the first run from `temper box`.

**It is the target.** The user, 2026-11-05: the client is to look like
these pages, exactly, with the changes made since by look requests.
Where this mock and `../mockups/` differ, this mock is the design
(`../mockups/README.md`, section 1).

**The whole story's mockups lead where they draw** (`../whole-story/`,
its README; `../../../draft/surfaces.md`, section 13): the rail, Home across
projects, Needs you's list and its why, Work, Team, How work runs,
Deployment, the phone's tabs and the terminal's screens. This mock leads
where they draw nothing, a goal's page and a chat. So this mock's rail, its
Goals, Chats, Changes and Settings, is the whole story's: Work and its
filters, How work runs and Deployment (`../../view.md`, 3); and its
`goals.html` and `changes.html` are Work's Goals and Changes filters
(`../../pages/README.md`, *Work*).

- `home.html`, `needs.html`, `goals.html`, `goal.html`, `chats.html`,
  `chat.html`, `changes.html`, `settings.html`: the pages, in the rail's
  order.
- `welcome.html`, `box.html`: the first run.
- `phone.html`: the pages in a phone's frame.
- `mock.css`: the look; the client's stylesheets take its rules.

**How work runs** is the whole story's `../whole-story/how.html`
(`../../pages/settings.md`, 2, *The page*): this mock's `settings.html` leads its
How work runs tab only for the rows the page keeps, What the project shows
and Ways of working at its foot, and its question route and "Away until",
which What reaches you holds under its columns; its Status, Funds and
Telling you rows, and its checks of what Away decides, are not drawn.

Open a page from a checkout; it needs no server. Budgets are drawn here
but matter least: the flow, look and mechanics come first (the user,
2026-11-05).

## Where the client departs from it

Every departure is a **Design departs** entry in
`../../README.status.md`, with its request. Three to know when reading these
pages:

- **Stopping work** says "Pause", "Resume" and "Close" on a chat, a goal and an
  agent's task, on purpose, where a goal here says "Stop": that word meant a
  goal's end and a chat's hold. The user's word, 2026-11-06 (the request
  `ui-pause-close`; `../../pages/README.md`, *A person's pause*).
- **The rail's foot row,** the person's name, is plain text here. In the
  client it opens a small menu above it, with "Accounts" and "Sign out",
  in the look of the project switcher's menu; a phone, with no rail's
  foot, signs out on Settings' Accounts tab. The user's word, 2026-11-06
  (the request `ui-owner-menu`; `../../view.md`, 3).
- **A project's repository,** plain words inside the switcher's button
  here, is a link beside the project's name button in the client, opening
  the repository on its forge in a new tab. The user's word, 2026-11-06
  (the request `ui-repo-link`; `../../view.md`, 3).
