# ci-on-forgejo-17: status

2026-11-02. Where the design lags or departs from [ci-on-forgejo-17.md](ci-on-forgejo-17.md).

skein's forge kit has the re-run (skein's `forge.md`: `Write::Rerun { attempt }` and its facts), and a landing's gates re-run the runner's failures (`../domain/landing.md`, 7.4, bounded by `gate_retries`).

- **Design lags:**
  - 2: `../domain/forge.md` repairs a change's CI failure the runner caused like any other, and leaves its re-run to this draft (section 20); `../protocol/configuration.md` has no `reruns` bound; `../protocol/forge.md` 3.2 names no re-run.
  - 3: `../domain/forge.md` 8.4 holds a change whose CI never reports with no reason from its pending jobs, and `../protocol/forge.md` has no read of a pending job's state.
  - 4: `../domain/agent.md` 4.3's `read_job` takes `head`, `run`, `job`, `attempt` and `max_bytes`, with no step and no search, and `../protocol/forge.md` 3.2's job read has neither.
