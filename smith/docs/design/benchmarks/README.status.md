# benchmarks: status

2026-11-12. Where smith's code lags or departs from [README.md](README.md).

- **Not yet:**
  - 3.1, 5.2: calibrations remain `catalogue.status.md`, 5.
  - 6: the Claude Code arm: `benchmarks/src/agent/mod.rs` refuses it.
  - 7: no `anthropic` arm (`agents/models.toml`'s `[working.anthropic]`
    is empty), and no suite runs `smith-claude-code`.
  - 9: version-3 parity remains `codex-parity.status.md`, 4.
  -  the missing probes remain `catalogue.status.md`, 1.
  -  the missing experiments remain `catalogue.status.md`, 3.
  -  the bundle validation remains `catalogue.status.md`, 4.
  -  the change-only projection remains `codex-parity.status.md`, 6.
- **Departs:**
  - 5.3, 7: the harness names the OpenAI tier table `codex`
    (`benchmarks/src/agent/mod.rs`, `Provider::Codex`, and every suite's
    `provider = "codex"`), where the design names smith's preset,
    `openai`; the code keeps the earlier name.
  - 6: `benchmarks/src/run.rs` names an agent once, where the user's
    request of 11-04 asked it to name none: `Request`'s `logins`, which
    each adapter reads, is stored as `codex_home`, since that request
    kept stored formats as they are.
  - 7, 10: smith's `anthropic` endpoint runs on the `claude` program
    account (`agents/smith/accounts.toml`), not on a key; no reason is
    recorded.
  -  the uncatalogued suites remain `catalogue.status.md`, 5.
