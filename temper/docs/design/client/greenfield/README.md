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

- `home.html`, `needs.html`, `goals.html`, `goal.html`, `chats.html`,
  `chat.html`, `changes.html`, `settings.html`: the pages, in the rail's
  order.
- `welcome.html`, `box.html`: the first run.
- `phone.html`: the pages in a phone's frame.
- `mock.css`: the look; the client's stylesheets take its rules.

Open a page from a checkout; it needs no server. Budgets are drawn here
but matter least: the flow, look and mechanics come first (the user,
2026-11-05).

## Where the client departs from it

Every departure is a **Design departs** entry in
`../README.status.md`, with its request. Three to know when reading these
pages:

- **Stopping work** says "Pause", "Resume" and "Close" on a chat, a goal and an
  agent's task, on purpose, where a goal here says "Stop": that word meant a
  goal's end and a chat's hold. The user's word, 2026-11-06 (the request
  `ui-pause-close`; `../README.md`, 3.5, *A person's pause*).
- **The rail's foot row,** the person's name, is plain text here. In the
  client it opens a small menu above it, with "Accounts" and "Sign out",
  in the look of the project switcher's menu; a phone, with no rail's
  foot, signs out on Settings' Accounts tab. The user's word, 2026-11-06
  (the request `ui-owner-menu`; `../README.md`, 5.3).
- **A project's repository,** plain words inside the switcher's button
  here, is a link beside the project's name button in the client, opening
  the repository on its forge in a new tab. The user's word, 2026-11-06
  (the request `ui-repo-link`; `../README.md`, 5.3).
