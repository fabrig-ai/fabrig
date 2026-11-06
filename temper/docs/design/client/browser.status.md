# browser: status

2026-11-06. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.5: `temper.css` shows the `Accounts` tab's "Sign out" at every
  width. 7.2: the gallery has no `rail-person-menu`.
- **Departs:** 2.5: `Ctrl+\` keeps Chat → Split → Draft on a phone too,
  with Split displayed as Draft. The 2026-11-05 answer to
  `ui-chat-draft/document-pane` accepts this cycle without viewport state;
  only the visible buttons omit Split (`README.md`, 3.5).
- **Design departs:** 2.5, from the request `ui-phone` of 2026-11-05, decided by
  its plan's cut and its sweep. A chat's foot stands 16 px in, where the
  request's check has the mock's 12: the look request `configure-temper-name`
  moved it for every chat, and the request keeps look requests made. A chat's
  foot and the toast take the home bar's inset, which the request asks of the
  tab bar alone: a covered page leaves them under it. "Every tab and primary
  button at least 44 tall" holds where the mock draws no shorter: the welcome's
  primaries stand 40 px, Home's acts 34, and a goal's primary outside its
  choices, the sheet's among them, 40, as the mock's `.btn` and `.btn.small` do.
  The mock's `box.html` draws a terminal, what `temper box` prints, and no page
  of the client's draws it.
- **Departs:** 7.2: the gallery's comparison pages do not hold one copy of
  the greenfield scenario. `home-morning` is Hello with no goals or Away
  switch, `home-with-goals` has the Solo world's goals, and `home-away`
  has four decisions still open to Overturn, where the mock's first is
  already landed. `needs-one` has no goal templates; `goals-reference`
  adds the large goal; chats and settings use the Reference and Solo
  worlds' parties, accounts and templates. Their counts, words, available
  tabs and resulting page lengths differ because the gallery draws those
  domains (`tools/gallery/src/samples.rs`), not the mock's stored state.
- **Not yet:** 7.2: `goal-sending` has one pending message and "Being sent",
  but no earlier accepted message or next words in the composer.
  `goal-toast` answers Pause without delivering a paused task document, so
  its toast says "Paused" while the header and acts still read running and
  Pause (`tools/gallery/src/samples.rs`, `goal_domain`).
- **Not yet:** 2.2: no test shows a head waiting for jig's mapping's room
  ended by the stall deadline: `temper-service`'s
  `a_call_past_the_mounts_room_waits_until_its_peer_closes` ends it by its
  peer's close. 4.3: the `ui-browser-red` plan has its whole tier pass within
  its budget (`tools/tiers/budgets.toml`, `browser-temper`, 85 CPU s). The
  answer on its task of 2026-11-05 judged its eight passes on everything but
  that line and takes the budget to the user as its own item, so nothing here
  shows the tier within it.
- **Departs:** 7.2: `start-goal` is Chats with no sheet and no filter,
  `Chats { filter: All }`, since that is where Goals' entry goes and `OpenStart`
  opens nothing without both a draft and a chat; `start-goal-draft` and
  `start-goal-refused` draw the same sheet a chat's "Start goal…" opens.
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
