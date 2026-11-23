# README: status

2026-11-23. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 10: `temper`'s and `temper-agent`'s suites launch the
  binary for each row of their tables and each mode's refusal
  [in flight: change 14097].
- **Not yet:** 10: `temper-engine`'s and `temper-worker`'s suites launch
  the binary for each row of their tables and each document's shape
  [in flight: change 14105].
- **Not yet:** 10: `temper-engine`'s suite launches `upgrade --check`
  and `--back` for each of the check's refusals
  [in flight: change 14115].
- **Not yet:** 10: `temper-box`'s suite still launches the binary for
  each row of its table (`temper/bin/temper-box/tests/suite/table.rs`),
  rather than its crate's unit tests holding the rows beyond one launch
  per path.
