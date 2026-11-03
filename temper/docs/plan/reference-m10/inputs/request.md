# reference-m10: the case

Carry out milestone **M10** of `temper/docs/design/draft/reference-case.execution.md`: what is left of the reference drafts once M1 to M9 are on main. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline (`DESIGN.md`).

In short:
- `reference-code-production.md`, section 14 whole, as one reference profile that a box can be set up with: its charters, its templates, its rules, its tiers, `questions`, `merge_job`, and its limits sized for a goal of a hundred changes (N21);
- `.temper/pre-pr` revisited with numbers.

## The entry

### M10. The case

- **Executes:** what is left: `reference-code-production.md`, section 14
  whole, as one reference profile, its limits sized for a goal of a
  hundred changes (N21: temper's `../protocol/configuration.md`, 13.4
  and section 7, `agent.md`, 4.3.1, jig's `domain/engine.md`, 17.9).
  It revisits `.temper/pre-pr` with numbers (the user, 2026-11-02): M1's
  first edition runs the format check and the old-form counts, with no
  build, and the coding agent's charter asks for clippy and the changed
  packages' focused tests. Whether the script runs them too is decided
  from the reds that reach a goal's branch, their repairs' cost and the
  check's time at each push.
- **Old code:** jig `github-pilot/t28-land-r4` (a batch refused for room
  waits for it); temper `configuring-revisions/settings-file` and
  `docs-reference/*`, background for the profile's sizing.
- **Walk:** rung C (`v0.1.0.md`, 2.1 and section 7): `reference-case.md`,
  acts 1 to 5, act by act, then whole; both plans; opened once rung B2
  passes.
- **Done** when C passes and the three drafts' statuses are empty; the
  drafts' own close-out follows (`DESIGN.md`, section 4).

## What the earlier milestones and walks leave for it

- **The stock limits are too small for the reference.** M8's walk met a one-line change that spent its 6 tasks under `change`. N21 sizes a change's tasks at about twelve, a goal's tree at about eight hundred, and a result's bytes for a designer's report. Size the reference profile from N21.
- **Each milestone configured its own parts of section 14** (the coordinator's, designer's, coding agent's and foreman's charters, the goal template for drafts, the landing rules, the tiers). Gather them into one profile, consistent with each other, as the settings M3 and configuring P2 made them. Write no second copy where a setting already exists.
- **`.temper/pre-pr`'s decision** needs numbers: reds that reach a goal's branch, their repairs' cost, and the check's time at each push. Where the numbers don't exist yet, record what the case walk must measure, and leave the first edition as it is.
- **The drafts' statuses** (`reference-case.status.md`, `reference-code-production.status.md`, `reference-landing.status.md`) should then hold nothing but what this plan can't close. Close them as the work lands. The drafts' own close-out, folding them into the design, follows the case's walk.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one. Profile limits are sizes, not time or CPU budgets.

## Size

One cut is at most 90 minutes. Release early.

## Walk

The case itself, rung C, by fabrig/qa: `reference-case.md`, acts 1 to 5, act by act, then whole, on real backends. Rung B2 must pass first (`v0.1.0.md`, 2.1). Mark the slice that makes the profile usable by a box `walk = true`.
