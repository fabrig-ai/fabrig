# reconciling-start: execution

2026-11-20. How [reconciling-start.md](reconciling-start.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| S1, a start reads and doesn't write: a making's read in read-afresh, its write after the settle in the room line's order; every read-afresh term checked for a wait on a write or on what the settle frees; tests 1 and 2 of `reconciling-start.md`, 7 | 2, 7 | none | dev.fabrig.ai's next start opens with a full outbox and a missing goal branch | not started |
| S2, a stop in every phase, and waits that say what they wait for: the stop before the mark, at the phase's end, and settled after the opening; the stop and report bounds printed and derived, the help's units taking them; a start phase, a reconciliation and a full table naming their waiters past their report bounds; tests 6 and 7 | 4, 5, 7 | none | a stop during dev.fabrig.ai's next start ends it within its bound | not started |
| S3, the engine opens once its own records are in: the audit of every procedure relying on the start's barrier, listed in the status first; the script opening after adopt runs, read-afresh and the settle as the forge's reconciliation, reads per resource, writes fenced per resource; `upgrade --check` on to the opening with no network; tests 3 to 5 and the fuzzy sweep, its faults in the shared vocabulary | 3, 6, 7 | S1 | dev.fabrig.ai's next redeploy opens before its forge reconciles, and its rehearsal passes on the opening | not started |

## 2. Order

S1 first: it removes the cycle that held dev.fabrig.ai closed on 11-20, and it is small. S2 can run beside it. S3 follows S1, since its reads per resource build on reads that hold no place.
