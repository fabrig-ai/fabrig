# browser: status

2026-11-04. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 8.4: the public browser walk's project-link checks expect
  `Project <identifier prefix>` (`temper/tests/client-browser/tests/browser_all/browser_public.rs`,
  `Walk::project_label`), where the public projects view uses the
  project's name (`temper/crates/temper-client-view/src/public/pages.rs`,
  `Draw::projects`).
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way` and `welcome-ready-adopted` are states the
  gallery holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`)
  that the table does not list.
  `AGENTS.md`, 1: 4.3's entry above calls the gate's list by the
  pipeline's name, the one name this file holds
  (`tests/boundary/outward.txt`). The playground-browser plan, unlanded,
  rewrites that entry and drops the name, so its lines are left to it.
