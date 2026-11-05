# browser: status

2026-11-05. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.5: a
  chat's document, its sheet and `Ctrl+\`. 7.2: `chat-draft-*`, `chat-new-draft`,
  `chat-start-goal*`, `chat-goal-linked`, `configure-chat-document*`
  and `rail-new-menu`.
- **Not yet:** 7.2: `goal-sending` has one pending message and "Being sent",
  but no earlier accepted message or next words in the composer.
  `goal-toast` answers Pause without delivering a paused task document, so
  its toast says "Paused" while the header and acts still read running and
  Pause (`tools/gallery/src/samples.rs`, `goal_domain`).
- **Departs:** 7.2: `start-goal` is Chats with no sheet and no filter,
  `Chats { filter: All }`, since that is where Goals' entry goes and `OpenStart`
  opens nothing without both a draft and a chat; `start-goal-draft` and
  `start-goal-refused` draw the sheet by sending it both, which no page does.
  The rulings of 2026-11-05 on the request `ui-goal-page` send the entry to a
  Drafts filter (`README.status.md`, 3.5, *Goals*, 5.14).
- **Not yet:** 7.1: a `home-*` state's title is its headline, then
  "temper", where the shell's is "Home · temper" (`README.md`, 5.1):
  `tools/gallery/src/lib.rs`, `heading`, reads the level-1 heading, not
  the view's title.
- **Departs:** 2.5: Home uses the greenfield mock's own scale, as the
  project's frame and Goals do, where the general main's scale says
  14/12: its column is 720 px, body 15 px, headline 32 px (25 px on a
  phone), facts 13.5 px and byline 12 px. The greenfield target is the
  user's look (`greenfield/README.md`); `temper.css` scopes these rules
  under `temper-home`.
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle` and `welcome-setup-failed` are
  states the gallery holds (`tools/gallery/names.txt`,
  `tools/gallery/src/samples.rs`) that the table does not list.
