# after-landing-repairs: status

2026-11-18. Where the design lags or departs from [after-landing-repairs.md](after-landing-repairs.md).

- **Design lags:**
  - 3: `../domain/after-landing.md`, sections 1, 6 and 10 keep a landing's change alive until cleared, with no `wait`. Its landing log names no member's goal or milestone, and evicts a landing whether or not it has been judged. Its section 13 still asks whether a goal may close before the bound. `../domain/forge/branches.md`, section 2's *Land* waits for cleared. `../domain/landing-settings.md`, 1.1 has no `wait`.
  - 4 and 5: `../domain/after-landing.md`, sections 4, 7 and 10 and `../domain/findings.md`, section 4 file a red of no goal's per landing, with no blame, take a failure as new by the previous run alone, keep `failing_tests` at 16 (`../protocol/configuration/limits.md`), and have no many finding. `../domain/findings.md`, section 4 holds a finding open until dropped; its section 7 closes one by its fix's landing cleared, not by a run that passes its tests; and its section 9 may evict any routed or dropped finding.
  - 6: `../domain/milestones.md`, sections 7 and 9, `../client/pages/milestone.md`, section 2, `../client/pages/goal.md`, section 2 and `../client/forge/records.md` show no reds on a goal or a milestone, and the `findings` subject has no filter by goal or milestone.
  - 7 and 8: `../domain/findings.md`, section 6 and `../domain/areas.md`, section 6 give the triager no sorts for a red and no `repair` or `blame` act, and a finding no repair or attempts. `../domain/landing-settings.md`, 1.1 has no `repairs`. `../domain/forge/attention.md`, section 6's stock tables have no item for routine repairs, and charter 14 (`../protocol/configuration/stock.md`) has no sorts.
- **Design departs:** none.
- **Open:** none.
