# browser: status

2026-11-05. Where temper's code lags or departs from [browser.md](browser.md).

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
  which names none (`forge.status.md`). 7.2: `settings`, `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way`, `welcome-ready-adopted`,
  `welcome-model-declined`, `welcome-model-renewed`,
  `welcome-model-refused`, `welcome-paste-waiting`, `welcome-paste-written`,
  `welcome-paste-too-long`, `welcome-paste-sending`,
  `welcome-paste-finishing`, `welcome-paste-address`, `welcome-paste-state`,
  `welcome-paste-too-many`, `welcome-paste-ended`, `welcome-paste-expired`,
  `welcome-paste-lost` and `welcome-paste-unsent` are states the gallery
  holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`) that
  the table does not list.
