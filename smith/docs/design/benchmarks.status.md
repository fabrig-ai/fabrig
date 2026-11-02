# benchmarks: status

2026-11-02. Where smith's code lags or departs from [benchmarks.md](benchmarks.md).

- **Not yet:**
  - 3.1, 5.2, 11.5: calibrations. The probes `budget`, `cache-reuse`,
    `credentials`, `failures`, `follow-up`, `large-write`,
    `parallel-children`, `parallel-read`, `paths`, `slow-completion` and
    `smoke-tools` hold no `[calibration]` block, nor does one of
    `tool-output`'s five variants, so they gate nothing
    (`benchmarks/src/probe/gating.rs`), the smoke suite's among them.
    `cache-reuse/idle-gap` has no variant, and `budget/spend` is missing
    from `suites/probes.toml`.
  - 6: the Claude Code arm: `benchmarks/src/run.rs` refuses it.
  - 7: no `anthropic` arm (`agents/models.toml`'s `[working.anthropic]`
    is empty), and no suite runs `smith-claude-code`.
  - 11.1: the `mid-work` and `build-environment` probes.
  - 11.3: the `affinity`, `cache-lifetime`, `delegation` and
    `edit-format` experiments; only `compaction` has a suite.
  - 16.6: a projection on this change's ratio alone:
    `benchmarks/src/parity/change.rs`'s `project_report` still multiplies
    in the ratios of the changes landed before it (`--landed`).
- **Departs:**
  - 5.3, 7: the harness names the OpenAI tier table `codex`
    (`benchmarks/src/agent/mod.rs`, `Provider::Codex`, and every suite's
    `provider = "codex"`), where the design names smith's preset,
    `openai`; the code keeps the earlier name.
  - 7, 10: smith's `anthropic` endpoint runs on the `claude` program
    account (`agents/smith/accounts.toml`), not on a key; no reason is
    recorded.
  - 11.5: the suites `role` and `repository-check` exist outside the
    catalogue; no reason is recorded.
