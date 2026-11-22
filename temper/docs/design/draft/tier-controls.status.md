# tier-controls: status

2026-11-21. Where the design lags or departs from [tier-controls.md](tier-controls.md).

The design carries items 1 to 3 (`../domain/after-landing-runs.md`, section 2, and `../domain/after-landing.md`, sections 5.2 and 9, and `../domain/after-landing-storage.md`, section 1, and `../domain/after-landing.md`, section 12; `../client/ctl/tools.md`, section 3; `../client/forge/records.md`, `TierActRequest`) and item 4 (`../domain/forge/change/stalls.md`, 4; `../domain/landing.md`, 4.2, and `../domain/landing-proof.md`, 2.1; `../domain/after-landing-runs.md`, section 2; skein's `forge/job-state.md`).

- **Open:** the whole document, item 4's last sentence: a tier's run is held past the runner wait, `forge.change_policy.runner_wait_ms`, the bound a change's checks use, and on its faults, each hold naming its pending jobs (`../domain/after-landing-runs.md`, section 2); a run that runs and never ends is held by no bound, which would be a new figure, the user's (`../domain/after-landing.md`, section 13; the request of 2026-11-21).
- **Design departs:** the whole document, items 1 and 3: a person's hold posts no pending `temper/verdict/<tier>` status, since no run of the tier's is there to link and no task to post it; the hold is said on each verifying change's line for the tier instead (`../domain/after-landing-runs.md`, section 2, *A person's hold*). The pending status is a run's, held on its runner faults or its wait for a runner (`../domain/after-landing.md`, section 9). Decided by the goal's design, 2026-11-14.
