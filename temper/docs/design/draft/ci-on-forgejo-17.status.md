# ci-on-forgejo-17: status

2026-11-11. Where the design lags or departs from [ci-on-forgejo-17.md](ci-on-forgejo-17.md).

skein's forge kit has the re-run (skein's `forge/README.md`: `Write::Rerun { attempt }` and its facts), and a landing's gates re-run the runner's failures (`../domain/landing.md`, 7.4, bounded by `gate_retries`). Section 2 is designed for a change's CI (`../domain/forge/change/README.md`, 2, step 3; `../domain/forge/README.md`, section 4, kind 17; `../protocol/configuration/limits.md`, `forge.change_policy.reruns`), by the request `queue-smooth.md`.

- **Design lags:**
  - 3: `../domain/forge/change/repairs.md` holds a change whose CI never reports with no reason from its pending jobs, and `../protocol/forge/README.md` has no read of a pending job's state.
  - 4: `../domain/agent/tools.md`'s `read_job` takes `head`, `run`, `job`, `attempt` and `max_bytes`, with no step and no search, and `../protocol/forge/README.md` 2.2's job read has neither.
- **Design departs:**
  - 2.2, *Counted at the head*: a change's re-runs are counted per job by the forge's attempts (a job at attempt *a* ran again *a* − 1 times, by temper or by anyone), not kept by the change, so `reruns` bounds each job's re-runs at a head and no stored count is added; *What people see*: each re-run is said in its write's account, a repair's brief and the landed result's checks (the attempt a check concluded at), not in a list kept on the change. `queue-smooth.md` asks that a stored format change be a question first, and these keep the draft's bound and its words without one. Decided by `queue-smooth`'s design, 2026-11-11.
  - 2.2, *Its effect*: the re-run's recovery class is skein's, unrecoverable, as the landing's re-run is (`../domain/landing.md`, 11), not conditional; it is found again by the job's attempt as the draft says.
