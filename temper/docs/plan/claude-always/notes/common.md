# claude-always: what every task keeps to

- **The design:** `temper/docs/design/draft/claude-provider.md`, and the sections each task's `## Inputs` names. Those sections are written and pushed with the specs.
- **Contracts:** `temper/docs/plan/claude-always/notes/contracts.md` is binding on names, members and paths.
- **No real network in any test:** a fake download, as Forgejo's fetch has (`temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs`; `temper/crates/temper/tests/support/fetch.rs`). No test reads `downloads.claude.ai`, `api.anthropic.com` or a real `claude`.
- **Leave alone:** `temper/docs/examples/owned.json` and `owned/agent.json`: the owned example stays as it is, Anthropic being init's addition (`temper/docs/design/protocol/setup.md`, 8.3). `temper/crates/temper-client-view/src/pages/home.rs`, `frame.rs`, `root.rs` and `temper/crates/temper-client-shell-web/temper.css`: `look-round-4`'s.
- **Fakes stay fake:** `temper box --fake-backends` still runs the `fake-claude` sibling, its model `fake-claude`. No product code branches on a test.
- **No `claude` in a test's `PATH` by accident:** the process tier's boxes run on an empty environment (`support/boxed.rs`), which is how a story has no `claude` on `PATH`. Keep it so.
