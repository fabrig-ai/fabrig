# browser: status

2026-11-03. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.1: the shell's `playground` feature
  (`../playground.md`, 6); `temper-client-shell-web` has the features
  `record` and `loopback-navigation` alone.
  4.3: none of `temper-client-domain`, `temper-client-view`,
  `temper-client-protocol-wire` and `temper-client-shell-web` is in the gate's wasm32
  clippy (fab's gate list, `.config/gate/wasm32.txt`); `temper-client-shell-web` fails
  it at `clippy::too_many_lines` (`src/lib.rs`, `turn`). 4.6: item 12, the
  playground's story A on the built playground and its log replayed
  natively (`../playground.status.md`).
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle`, `welcome-setup-failed`,
  `welcome-ready-under-way` and `welcome-ready-adopted` are states the
  gallery holds (`tools/gallery/names.txt`, `tools/gallery/src/samples.rs`)
  that the table does not list.
