# after-landing-repairs: status

2026-11-19. Where the design lags or departs from [after-landing-repairs.md](after-landing-repairs.md).

- **Design lags:**
  - 9, `failing_tests`: `../protocol/configuration/limits.md` doesn't name `forge.tiers.failing_tests` or its stock 64.
  - 6: `../domain/milestones.md`, sections 7 and 9, `../client/pages/milestone.md`, section 2, `../client/pages/goal.md`, section 2 and `../client/forge/records.md` show no reds on a goal or a milestone, and the `findings` subject has no filter by goal or milestone.
  - 7 and 8: `../domain/findings.md`, section 6 and `../domain/areas.md`, section 6 give the triager no sorts for a red and no `repair` or `blame` act, and a finding no repair or attempts. `../domain/landing-settings.md`, 1.1 has no `repairs`. `../domain/forge/attention.md`, section 6's stock tables have no item for routine repairs, and charter 14 (`../protocol/configuration/stock.md`) has no sorts.
- **Design departs:**
  - 5, *Its repair is named*: `Fixes #n` in the home names the finding whose issue is n, so it names no red finding until findings have issues (`../domain/findings.md`, section 2), and a landing names one by a member's pull request body's `Fixes-finding: n`, not yet by its commits' trailers, which the landing log doesn't keep (`../domain/red-findings.md`, sections 6 and 11). The goal's designer, 2026-11-19: the mechanisms the draft's letter needs aren't built, and the run's comments still name the repair.
  - 5, *Never evicted*: at most 8 filings wait for room per tier; one past them is counted, not kept, and a later run files its failures again, without its blame (`../domain/red-findings.md`, section 7). The goal's designer, 2026-11-19: the tier's record is bounded, and the findings full of uncleared work is rare.
- **Open:** none.
