# browser: status

2026-11-05. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 7.1: a `home-*` state's title is its headline, then
  "temper", where the shell's is "Home · temper" (`README.md`, 5.1):
  `tools/gallery/src/lib.rs`, `heading`, reads the level-1 heading, not
  the view's title.
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way`, `welcome-ready-adopted`,
  `welcome-model-declined`, `welcome-model-renewed`,
  `welcome-model-refused`, `welcome-paste-waiting`, `welcome-paste-written`,
  `welcome-paste-too-long`, `welcome-paste-sending`,
  `welcome-paste-finishing`, `welcome-paste-address`, `welcome-paste-state`,
  `welcome-paste-too-many`, `welcome-paste-ended`, `welcome-paste-expired`,
  `welcome-paste-lost`, `welcome-paste-unsent`, `settings-drafts-off`,
  `settings-goals-off`, `settings-changes-off` and `settings-away-off` are states the gallery
  holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`) that
  the table does not list.
