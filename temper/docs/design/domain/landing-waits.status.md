# landing-waits: status

2026-11-24. Where temper's code lags or departs from [landing-waits.md](landing-waits.md).

- **Not yet:** 3 (*A wait on the forge is bounded*): the bound counts from a run of `read` waits, not from the landing's last advance, and bounds no other wait on the forge; the `read` hold's words name no read count (`temper-engine-domain-forge/src/landing_read_wait.rs`). 4: the log's held line is the run's, `held after AGE waiting for a fresh read`.
