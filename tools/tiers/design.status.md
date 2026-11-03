# design: status

2026-11-03. Where the slow tier runner's code lags or departs from [design.md](design.md).

- **Not yet:** 7.3: a starved run's JUnit abort by SIGTERM or SIGKILL with no record is
  unobserved; `cpu.py` `summary` still refuses it as missing coverage (exit 2), and the
  summary has no `unobserved` list or count. 7.4: the suite does not cover cancellation
  aborts without records.
