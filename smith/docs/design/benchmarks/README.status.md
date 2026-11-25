# benchmarks: status

2026-11-23. Where smith's code lags or departs from [README.md](README.md).

- **Not yet:**
  - 3.1, 5.2: calibrations remain `catalogue.status.md`, 5.
  -  the missing probes remain `catalogue.status.md`, 1.
  -  the missing experiments remain `catalogue.status.md`, 3.
  -  the bundle validation remains `catalogue.status.md`, 4.
  -  the change-only projection remains `codex-parity.status.md`, 6.
  - 13: receipts from supplied completions, the scratch root over the
    test tree and smith's check from a supplied receipt. [in flight:
    change 14074]
- **Departs:**
  - 5.3: the harness names the OpenAI tier table `codex`
    (`benchmarks/src/agent/mod.rs`, `Provider::Codex`, and every suite's
    `provider = "codex"`), where the design names smith's preset,
    `openai`; the code keeps the earlier name.
  -  the uncatalogued suites remain `catalogue.status.md`, 5.
