# catalogue: status

2026-11-10. Where the tree's code lags or departs from [catalogue.md](catalogue.md).

- **Not yet:**
  - 5: calibrations. The probes `budget`, `cache-reuse`,
    `credentials`, `failures`, `follow-up`, `large-write`,
    `parallel-children`, `parallel-read`, `paths`, `slow-completion` and
    `smoke-tools` hold no `[calibration]` block, nor does one of
    `tool-output`'s five variants, so they gate nothing
    (`benchmarks/src/probe/gating.rs`), the smoke suite's among them.
    `cache-reuse/idle-gap` has no variant, and `budget/spend` is missing
    from `suites/probes.toml`.
  -  the Claude Code arm: `benchmarks/src/agent/mod.rs` refuses it.
  -  no `anthropic` arm (`agents/models.toml`'s `[working.anthropic]`
    is empty), and no suite runs `smith-claude-code`.
  -  `shell-cwd` is at version 3, its first grade having
    changed (`README.md`, 5.2), and both `benchmarks/baselines/parity-*.json` and
    every committed summary hold it at version 2. So a run's drift says
    "baseline inputs changed; retake baseline"
    (`benchmarks/src/summary.rs`, `drift`), and parity has not been
    shown on version 3.
  - 1: the `mid-work` and `build-environment` probes.
  - 3: the `affinity`, `cache-lifetime`, `delegation` and
    `edit-format` experiments; only `compaction` has a suite.
  - 4: `shell-cwd` at version 3 has not been seen on its bundle.
    Only `smith-bench validate repository/shell-cwd` on a machine
    holding it shows that the first grade finds the frozen workspace's
    two lines (`README.md`, section 14), that the grade keeps its 80 s, that the
    oracle compiles against the frozen tree (its `tools::worst_case`
    is read from the blob `reference/solution.patch` names and from
    the rewritten commits holding it) and that the seed fails and the
    reference passes. Not validated: the bundle is absent, and the
    source rebuild could not start because the archive host's name
    could not be resolved.
  -  a projection on this change's ratio alone:
    `benchmarks/src/parity/change.rs`'s `project_report` still multiplies
    in the ratios of the changes landed before it (`--landed`).
- **Departs:**
  -  the harness names the OpenAI tier table `codex`
    (`benchmarks/src/agent/mod.rs`, `Provider::Codex`, and every suite's
    `provider = "codex"`), where the design names smith's preset,
    `openai`; the code keeps the earlier name.
  -  `benchmarks/src/run.rs` names an agent once, where the user's
    request of 11-04 asked it to name none: `Request`'s `logins`, which
    each adapter reads, is stored as `codex_home`, since that request
    kept stored formats as they are.
  -  smith's `anthropic` endpoint runs on the `claude` program
    account (`agents/smith/accounts.toml`), not on a key; no reason is
    recorded.
  - 5: the suites `role` and `repository-check` exist outside the
    catalogue; no reason is recorded.
