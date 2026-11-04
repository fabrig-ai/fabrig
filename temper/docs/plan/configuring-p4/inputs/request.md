# configuring-p4: stage 1, the golden check

Carry out phase **P4** of `temper/docs/design/draft/configuring.execution.md`. Its entry is the scope, quoted below. It runs as a reference milestone does (`temper/docs/design/draft/reference-case.execution.md`, section 2): design and code go through this plan as one pipeline, its designers writing the design with the specs that cite it (`DESIGN.md`).

P1, P2 and P3 (the configuring agent) are on main, as are every reference milestone, M1 to M10. So this plan builds on main. Its golden is `reference-code-production.md`, section 14, cut into obligations: the reference profile M10 built (`docs/examples/reference/`).

## The entry

### P4. Stage 1

- **Executes:** `configuring.md`, 8.1.
- **Design:** the golden, `reference-code-production.md`, section 14
  cut into obligations; its check in code and its judge; the box's
  prerequisites and limits; designed beside 6.4 in
  `../domain/testing.md` unless its designers find a better home.
- **Builds:** the golden, the check and the judge; the walk's brief for
  fabrig/qa.
- **Walk:** 8.1 itself, run again as the reference's milestones land.
- **Done** when 8.1 passes with every obligation of a setting met and
  every gap named, as far as the reference has landed.

## Context

- **The walk** is 8.1 itself. The brief is for fabrig/qa; mark that slice `walk = true`.
- **Budgets, timeouts and the box's limits are the user's** (skein's `docs/foundation/testing-strategy.md`, 8.1). Propose figures as Open.
- **Beside it:** `public-reading-build` and `playground-browser` are building; leave their files alone.

## Size

One cut is at most 90 minutes. Release early.
