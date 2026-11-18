# after-landing-repairs: status

2026-11-18. Where the design lags or departs from [after-landing-repairs.md](after-landing-repairs.md).

- **Design lags:**
  - 3: `../domain/after-landing.md`, sections 1, 6 and 10 keep a landing's change alive until cleared, with no `wait`; section 13 still asks whether a goal may close before the bound. `../domain/forge/branches.md`, section 2's *Land* waits for cleared. `../domain/landing-settings.md`, 1.1 has no `wait`.
  - 4 and 5: `../domain/after-landing.md`, sections 4, 7 and 10 and `../domain/findings.md`, section 4 file a red of no goal's per landing with no signature, and `failing_tests` is 16 (`../protocol/configuration/limits.md`); no many finding; `../domain/findings.md`, section 7 closes a finding by its fix's landing cleared, not by a run that passes its tests.
  - 6 and 7: nothing starts a goal from a finding; `../domain/landing-settings.md`, 1.1 has no `repairs`, and `../domain/findings.md`, section 6 has no attempts or waiting finding.
- **Design departs:** none.
- **Open:** which party requests the reference deployment's repair goals and answers their questions (6, `requester`), for R4.
