# after-landing-repairs: execution

2026-11-18. How [after-landing-repairs.md](after-landing-repairs.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| R1, a goal ends at landing: kind 4's `wait`; under `landed`, a change into a branch a tier follows ends at landed with no `Verification` row, the closer's *Land* waits for landed; the tiers judge its landing by commit and key, its members from the landing log, which keeps each tier's verdict and keeps a landing until judged, within `landings_pending`; a red still filed per landing until R2; switching a project ends its verifying and red changes landed and uncleared | 3, 8's first, second and fourth points, 9's first and third points, 10's first two | a goal of the draft's | none | after landing, on a staging deployment: a goal under `landed` ends at its landing, and the tier's next red run comments on its pull request | not started |
| R2, the red finding: new failures by test id against the uncleared red findings; one red finding per judged run with new failures under `landed`, its signature, title and body; cleared by a run that fails none of its ids, kept apart from dropped and never evicted while uncleared, a filing refused for room counted; flakes filed or counted; `failing_tests` 64 with the start's checks, and the many finding | 4, 5, 8's second and third points, 9's first point, 10's third to fifth | a goal of the draft's | R1 | after landing: two red runs, the second adding new failures, file two findings, and a run passing the first's tests clears it | not started |
| R3, a standing grant beneath a gathering: in jig, a gathering's maker grants an application the right to make delegates beneath it on the maker's behalf, of a given shape; each such `Delegate` checked as the maker's own, its authority, the gathering's budget and delegation, a refusal reported to the application; the grant ending with the gathering | 6's first point | a goal of the draft's | none | none: jig's own stories | not started |
| R4, repair goals: kind 4's `repairs` and its members, checked at a start; a finding's repair phases; admission in one decision, keyed, retried at filing, a goal's end, a start and a settings change; the goal's words; judging at the first run covering a goal's end or at once without a landing, then waiting again or triage, a cancel to triage; `repair_deadline`; ctl's finding reads naming the repair | 6, 7, 8's first, third and last points, 9, 10's sixth to last | a goal of the draft's | R2, R3 | after landing: a red run starts a repair goal beneath the milestone, a third finding waits behind two live goals, and a repair that doesn't repair gets one more goal, then triage | not started |
| R5, the reference deployment switched: its project under `wait: landed`, its `verification_bound` back to its default; its repairs milestone made, with its budget, by the party that will answer the repair goals, and named in `repairs` with `repair_goals` 2; the hand-made repairs stop | 6's first point | the deployment's operator and the milestone's maker, at a redeploy the person approves | R4, and the milestone's maker named (status, *Open*) | none: the deployment's own runs | not started |

## 2. Order

- **R1 first,** since R2 files by run where goals no longer wait. Until R1 is on, the reference deployment keeps its one-minute `verification_bound` (`after-landing-repairs.md`, section 2).
- **R3 alongside R1 and R2:** jig's alone, it touches nothing they do.
- **R4 once R2 and R3 have landed:** a repair goal is made from R2's finding by R3's grant.
- **R5 once R4 has landed** and the deployment runs it.
- **Section 11's Later items** are no milestone's.
