# worker-restarts: execution

2026-11-20. How [worker-restarts.md](worker-restarts.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| W1, the stop is the product's: the drain asked over the worker's link and a ctl verb for it, no new work without a stop, the shipped unit's `KillMode=mixed` and stop timeout from the worker's printed bound, an agent with a live host leaving its stop to it; tests 6 and 8 of section 9 | 2, 9 | none | a drained dev.fabrig.ai worker settles its turns in flight before it stops | not started |
| W2, suspended, not failed: the suspended end, no clock over the gap, no repair asked for cut work, a resumed attempt sent back to its worker within the window; test 5 | 4 (placing), 5, 9 | none | a run cut by a worker's restart on dev.fabrig.ai resumes with no try and no clock spent | not started |
| W3, settle at the answer and resume the pending calls: the answer committed on arrival on the turns' wire, calls cut after a grace, the cut run's workspace and results directory kept, the agent running pending calls again or handing them to the model; tests 1 to 3 | 3, 4, 6, 9 | W1, W2 | a delivery check cut by a restart on dev.fabrig.ai runs again from its passed tests | not started |
| W4, a crash takes the same path: the kept workspace saved at the next start, a worker back as a new process counting its missing attempts as suspended at once; test 4 | 7, 9 | W3 | a worker killed on dev.fabrig.ai loses only what wasn't committed | not started |
| W5, settings live where that's safe: timeouts, deadlines and bounds applied at the next operation, each setting saying whether it needs a restart; test 7 | 8, 9 | none | a timeout changed on dev.fabrig.ai applies with no worker restart | not started |

## 2. Order

W1 first: with its unit and the agent's signal handling, the drain already built starts working, and most of today's loss goes. W2 and W5 can run beside it. W3 follows W1 and W2, and W4 follows W3.
