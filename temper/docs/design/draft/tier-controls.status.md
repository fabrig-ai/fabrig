# tier-controls: status

2026-11-14. Where the design lags or departs from [tier-controls.md](tier-controls.md).

The design carries items 1 to 3 (`../domain/after-landing.md`, sections 3, 5.2, 9, 10 and 12; `../client/ctl/tools.md`, section 3; `../client/forge/records.md`, `TierActRequest`) and item 4 for a change and a landing (`../domain/forge/change/repairs.md`, *A stall on CI says why*; `../domain/landing.md`, 4.2 and 7.1; skein's `forge/job-state.md`).

- **Design lags:** the whole document, item 4's last sentence: no bound holds a tier's run that was found and never ends (`../domain/after-landing.md`, section 3 holds a run on its runner faults alone), so none names its pending jobs. A tier's bound and its hold would be a fifth task; the draft drops from the bottom past four (the goal's design, 2026-11-14).
- **Design departs:** the whole document, items 1 and 3: a person's hold posts no pending `temper/verdict/<tier>` status, since no run of the tier's is there to link and no task to post it; the hold is said on each verifying change's line for the tier instead (`../domain/after-landing.md`, section 3, *A person's hold*). The pending status is a run's held on its runner faults (`../domain/after-landing.md`, section 9). Decided by the goal's design, 2026-11-14.
