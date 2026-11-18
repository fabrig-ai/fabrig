# after-landing-repairs: execution

2026-11-18. How [after-landing-repairs.md](after-landing-repairs.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| R1, a goal ends at landing: kind 4's `wait`; under `landed`, a change into a branch a tier follows ends at landed with no `Verification` row, the closer's *Land* waits for landed, and the tiers judge its landing by commits, naming its members from the landing log, where each tier's verdict is kept; switching a project ends its verifying and red changes landed and uncleared | 3, 8's first point, 9's first and third points, 10's first two | a goal of the draft's | none | after landing, on a staging deployment: a goal under `landed` ends at its landing, and the tier's next red run comments on its pull request | not started |
| R2, a run's new failures and the repair finding: new failures by test id against the unclosed red findings; one red finding per judged run with new failures, its signature and body; closed by a run that fails none of its ids; flakes filed or counted; `failing_tests` 64 and the many finding | 4, 5, 8's second and third points, 9's first point, 10's third to fifth | a goal of the draft's | R1 | after landing: two red runs, the second adding new failures, file two findings, and a run passing the first's tests closes it | not started |
| R3, repair goals: kind 4's `repairs` and its members; the root starting a goal per finding within `repair_goals`, the waiting findings taken together when a place frees; the goal's words; attempts judged at the first run covering a goal's end, then triage; ctl's finding reads naming the repair | 6, 7, 8's first and last points, 9, 10's last four | a goal of the draft's | R2 | after landing: a red run starts a repair goal, a third finding waits behind two live ones, and a repair that doesn't repair gets one more goal, then triage | not started |
| R4, the reference deployment switched: its project under `wait: landed` with `verification_bound` back to its default, `repairs` on with `repair_goals` 2 and its `requester`; the hand-made repairs stop | 6's first point | the deployment's operator, at a redeploy the person approves | R3, and its `requester` named (status, *Open*) | none: the deployment's own runs | not started |

## 2. Order

- **R1 first,** since R2 files by run where goals no longer wait. Until R1 is on, the reference deployment keeps its one-minute `verification_bound` (`after-landing-repairs.md`, section 2).
- **R2, then R3:** a repair goal is started from the finding R2 files.
- **R4 once R3 has landed** and the deployment runs it.
- **Section 11's Later items** are no milestone's.
