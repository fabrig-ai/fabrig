# agents: status

2026-11-23. Where smith's code lags or departs from [agents.md](agents.md).

- **Not yet:**
  - 1: the Claude Code arm: `benchmarks/src/agent/mod.rs` refuses it.
  - 2: no `anthropic` arm (`agents/models.toml`'s `[working.anthropic]`
    is empty), and no suite runs `smith-claude-code`.
- **Departs:**
  - 2: the harness names the OpenAI tier table `codex`
    (`benchmarks/src/agent/mod.rs`, `Provider::Codex`, and every suite's
    `provider = "codex"`), where the design names smith's preset,
    `openai`; the code keeps the earlier name.
  - 1: `benchmarks/src/run.rs` names an agent once, where the user's
    request of 11-04 asked it to name none: `Request`'s `logins`, which
    each adapter reads, is stored as `codex_home`, since that request
    kept stored formats as they are.
  - 2: smith's `anthropic` endpoint runs on the `claude` program
    account (`agents/smith/accounts.toml`), not on a key; no reason is
    recorded.
