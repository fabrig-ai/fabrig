# browser: status

2026-11-05. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.3, 2.5: Goals' path and look: plan `ui-goals`.
- **Not yet:** 2.3: `?state=live` on a task's address still reads. 2.5: *A goal's look*. 7.2: the `goal-*` and `start-goal*` states: plan `ui-goal-page`.
- **Not yet:** 7.2: `needs-proposal`, `needs-held`, `needs-funds`,
  `home-morning-card`, `home-answered` and
  `home-one-need`. 7.1: a `home-*` state's title is its headline, then
  "temper", where the shell's is "Home · temper" (`README.md`, 5.1):
  `tools/gallery/src/lib.rs`, `heading`, reads the level-1 heading, not
  the view's title.
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `settings`, `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way` and `welcome-ready-adopted` are states the
  gallery holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`)
  that the table does not list.
