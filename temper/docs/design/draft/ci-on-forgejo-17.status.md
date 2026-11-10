# ci-on-forgejo-17: status

2026-11-02. Where the design lags or departs from [ci-on-forgejo-17.md](ci-on-forgejo-17.md).

skein's forge kit has the re-run (skein's `forge/README.md`: `Write::Rerun { attempt }` and its facts), and a landing's gates re-run the runner's failures (`../domain/landing.md`, 7.4, bounded by `gate_retries`).

- **Design lags:**
  - 2: `../domain/forge/README.md` repairs a change's CI failure the runner caused like any other, and leaves its re-run to this draft (`../domain/forge/README.md`, section 11); `../protocol/configuration/README.md` has no `reruns` bound; `../protocol/forge/README.md` 2.2 names no re-run.
  - 3: `../domain/forge/change/repairs.md` holds a change whose CI never reports with no reason from its pending jobs, and `../protocol/forge/README.md` has no read of a pending job's state.
  - 4: `../domain/agent/tools.md`'s `read_job` takes `head`, `run`, `job`, `attempt` and `max_bytes`, with no step and no search, and `../protocol/forge/README.md` 2.2's job read has neither.
