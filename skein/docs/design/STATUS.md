# Status

2026-11-03. Where skein's code lags or departs from its design, and its
design from its drafts, from the status files below. The kernel's
records of a tree's metadata, the filesystem's status and btrfs's
subvolumes, and cgroups with contained spawn, are not built, nor their
simulation, faults and fake machine (`kernel.md`, `io.md`,
`simulator.md`, `shell.md`, `testing.md`). The forge kit lacks the calls
no dialect has yet (the labels, `UpdateRef`) and GitHub's Actions
routes, `Artifact` and `Commits` among them (`forge.md`,
`forge-github.md`); the fake GitHub refuses no push as `GH013`
(`fake-forge.md`). `skein-playground` is not built
(`playground.md`, and `ui-web.md` for the worker's platform). json, tls
and http's collector have no fuzz target (`json.md`, `tls.md`,
`http.md`), and the withdrawn LLM routes still stand while smith uses
them (`llm.md`). A few items lag in `lib.md`, `examples.md`, `ui.md`
and `ui-web.md` (Escape in a form). The code departs in three crates' `tests/` directories
(`testing.md`). `README.md`, `channel.md`,
`codec.md`, `kv.md`, `world.md`, `fake-checkout.md`, `fake-llm.md`,
`forge-client.md`, `forge-connection.md`, `forge-setup.md`,
`llm-connection.md`, `oauth.md`, `oauth-issuer.md`, `ui-text.md` and
`browser.md` have no gap. The foundation's rules
(`../foundation/`) have no status file: of `testing-strategy.md`, 6, no
tree runs transition coverage (`cargo llvm-cov`) or fuzzes a machine or a
decoder (`cargo fuzz`). Of the drafts, `component-kit.md` waits on
`lib.md` and `io.md` to name its items, and the design departs from
`redb.md`: the store is `kv.md`'s own machine.

Design documents:
- [`README.md`](README.status.md)
- [`lib.md`](lib.status.md)
- [`kernel.md`](kernel.status.md)
- [`io.md`](io.status.md)
- [`shell.md`](shell.status.md)
- [`simulator.md`](simulator.status.md)
- [`world.md`](world.status.md)
- [`testing.md`](testing.status.md)
- [`examples.md`](examples.status.md)
- [`codec.md`](codec.status.md)
- [`json.md`](json.status.md)
- [`kv.md`](kv.status.md)
- [`http.md`](http.status.md)
- [`tls.md`](tls.status.md)
- [`channel.md`](channel.status.md)
- [`llm.md`](llm.status.md)
- [`llm-connection.md`](llm-connection.status.md)
- [`fake-llm.md`](fake-llm.status.md)
- [`oauth.md`](oauth.status.md)
- [`oauth-issuer.md`](oauth-issuer.status.md)
- [`forge.md`](forge.status.md)
- [`forge-client.md`](forge-client.status.md)
- [`forge-connection.md`](forge-connection.status.md)
- [`forge-github.md`](forge-github.status.md)
- [`forge-setup.md`](forge-setup.status.md)
- [`fake-forge.md`](fake-forge.status.md)
- [`fake-checkout.md`](fake-checkout.status.md)
- [`ui.md`](ui.status.md)
- [`ui-text.md`](ui-text.status.md)
- [`ui-web.md`](ui-web.status.md)
- [`browser.md`](browser.status.md)
- [`playground.md`](playground.status.md)

Drafts:
- [`draft/component-kit.md`](draft/component-kit.status.md)
- [`draft/redb.md`](draft/redb.status.md)
