# landing-history: status

2026-11-15. Where temper's code lags or departs from [landing-history.md](landing-history.md).

- **Not yet:** 2.2, `job-ended` and *A job's attempt*: an attempt's end reaches history only through its task's end, a retry's claim closes the attempt before it `lost`, a cancel writes `lost`, and no `timeout`, `cancelled` or failure class is stored or shown (`temper-engine-domain-forge/src/job_history.rs`, `heard`; `temper-engine-domain/src/route/forge_route/job_history.rs`, `saved`) [in flight: change 3487].
- **Not yet:** 2.2, *After its end*: `change_history::pushed` and the other writers append to an ended change's history (`temper-engine-domain-forge/src/history.rs`, `write_event`) [in flight: change 3486].

