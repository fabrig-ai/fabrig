# codex-parity: status

2026-11-11. Where the tree's code lags or departs from [codex-parity.md](codex-parity.md).

- **Not yet:**
  -  calibrations remain `catalogue.status.md`, 5.
  -  the Claude Code arm remains `README.status.md`, 6.
  -  the missing arms remain `README.status.md`, 7.
  - 4: `shell-cwd` is at version 3, its first grade having
    changed (`README.md`, 5.2), and both `benchmarks/baselines/parity-*.json` and
    every committed summary hold it at version 2. So a run's drift says
    "baseline inputs changed; retake baseline"
    (`benchmarks/src/summary.rs`, `drift`), and parity has not been
    shown on version 3.
  -  the missing probes remain `catalogue.status.md`, 1.
  -  the missing experiments remain `catalogue.status.md`, 3.
  -  the bundle validation remains `catalogue.status.md`, 4.
  - 6: a projection on this change's ratio alone:
    `benchmarks/src/parity/change.rs`'s `project_report` still multiplies
    in the ratios of the changes landed before it (`--landed`).
