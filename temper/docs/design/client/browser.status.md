# browser: status

2026-11-03. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.1: the shell's `playground` feature
  (`../playground.md`, 6); `temper-client-shell-web` has the features
  `record` and `loopback-navigation` alone.
  4.3: `temper-client-shell-web` fails the gate's wasm32 clippy, so
  fab's gate list (`.config/gate/wasm32.txt`) leaves it out; its first
  error: `clippy::too_many_lines` at `temper/crates/temper-client-shell-web/src/lib.rs:131` (`turn`).
  4.6: item 12, the playground's story A on the built playground and its
  log replayed natively (`../playground.status.md`).
  8.4: the public browser walk's project-link checks expect
  `Project <identifier prefix>` (`temper/tests/client-browser/tests/browser_all/browser_public.rs`,
  `Walk::project_label`), where the public projects view uses the
  project's name (`temper/crates/temper-client-view/src/public/pages.rs`,
  `Draw::projects`).
- **Departs:** 8.3: public pages show a project's name and tasks' first-line
  titles (`temper/crates/temper-client-view/src/public/pages.rs` and
  `lines.rs`), as the user decided on 2026-11-03; people's names stay
  hidden (`../draft/first-release.status.md`).
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way` and `welcome-ready-adopted` are states the
  gallery holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`)
  that the table does not list.
