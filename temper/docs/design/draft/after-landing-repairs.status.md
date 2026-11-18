# after-landing-repairs: status

2026-11-18. Where the design lags or departs from [after-landing-repairs.md](after-landing-repairs.md).

- **Design lags:**
  - 3: `../domain/after-landing.md`, sections 1, 6 and 10 keep a landing's change alive until cleared, with no `wait`, and its landing log keeps no verdict and evicts a landing whatever its judging; section 13 still asks whether a goal may close before the bound. `../domain/forge/branches.md`, section 2's *Land* waits for cleared. `../domain/landing-settings.md`, 1.1 has no `wait`.
  - 4 and 5: `../domain/after-landing.md`, sections 4, 7 and 10 and `../domain/findings.md`, section 4 file a red of no goal's per landing with no signature, and `failing_tests` is 16 (`../protocol/configuration/limits.md`); no many finding. `../domain/findings.md`, section 4 holds a finding open until dropped, section 7 closes one by its fix's landing cleared, not by a run that passes its tests, and section 9 may evict any routed or dropped finding.
  - 6 and 7: nothing makes a goal from a finding. `../domain/landing-settings.md`, 1.1 has no `repairs`; `../domain/findings.md`, section 6 routes only to a live goal and has no repair phase or attempts; `../domain/areas.md`, section 6 leaves all grouping of findings to the triager. jig's `domain/tasks/gatherings.md`, section 1 lets only a party who may act on a gathering make its delegates, with no standing grant to an application.
- **Design departs:** none.
- **Open:** which party makes the reference deployment's repairs milestone, with what budget, and so answers its repair goals' questions (6, for R5).
