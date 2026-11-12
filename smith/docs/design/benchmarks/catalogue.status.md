# catalogue: status

2026-11-11. Where the tree's code lags or departs from [catalogue.md](catalogue.md).

- **Not yet:**
  - 5: calibrations. The probes `budget`, `cache-reuse`,
    `credentials`, `failures`, `follow-up`, `large-write`,
    `parallel-children`, `parallel-read`, `paths`, `slow-completion` and
    `smoke-tools` hold no `[calibration]` block, nor does one of
    `tool-output`'s five variants, so they gate nothing
    (`benchmarks/src/probe/gating.rs`), the smoke suite's among them.
    `cache-reuse/idle-gap` has no variant, and `budget/spend` is missing
    from `suites/probes.toml`.
  -  the Claude Code arm remains `README.status.md`, 6.
  -  the missing arms remain `README.status.md`, 7.
  -  version-3 parity remains `codex-parity.status.md`, 4.
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
  -  the change-only projection remains `codex-parity.status.md`, 6.
- **Departs:**
  -  the tier-name departure remains `README.status.md`, 5.3 and 7.
  -  the stored-agent-name departure remains `README.status.md`, 6.
  -  the Anthropic account departure remains `README.status.md`, 7 and 10.
  - 5: the suites `role` and `repository-check` exist outside the
    catalogue; no reason is recorded.
