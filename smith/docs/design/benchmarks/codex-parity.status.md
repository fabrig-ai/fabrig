# codex-parity: status

2026-11-10. Where the tree's code lags or departs from [codex-parity.md](codex-parity.md).

- **Not yet:**
  -  calibrations. The probes `budget`, `cache-reuse`,
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
  - 4: `shell-cwd` is at version 3, its first grade having
    changed (`README.md`, 5.2), and both `benchmarks/baselines/parity-*.json` and
    every committed summary hold it at version 2. So a run's drift says
    "baseline inputs changed; retake baseline"
    (`benchmarks/src/summary.rs`, `drift`), and parity has not been
    shown on version 3.
  -  the `mid-work` and `build-environment` probes.
  -  the `affinity`, `cache-lifetime`, `delegation` and
    `edit-format` experiments; only `compaction` has a suite.
  -  `shell-cwd` at version 3 has not been seen on its bundle.
    Only `smith-bench validate repository/shell-cwd` on a machine
    holding it shows that the first grade finds the frozen workspace's
    two lines (`README.md`, section 14), that the grade keeps its 80 s, that the
    oracle compiles against the frozen tree (its `tools::worst_case`
    is read from the blob `reference/solution.patch` names and from
    the rewritten commits holding it) and that the seed fails and the
    reference passes. Not validated: the bundle is absent, and the
    source rebuild could not start because the archive host's name
    could not be resolved.
  - 6: a projection on this change's ratio alone:
    `benchmarks/src/parity/change.rs`'s `project_report` still multiplies
    in the ratios of the changes landed before it (`--landed`).
