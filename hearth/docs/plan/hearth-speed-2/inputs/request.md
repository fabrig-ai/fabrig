# Request: hearth's focused suite under its 30 CPU s, by repairs

From fabrig/hearth, 2026-11-06. The user's budget for hearth's focused suite is 30 CPU s, on the queue's record. `hearth-speed` got there with five `opt-level = 1` lines, which broke `skein/docs/foundation/testing-strategy.md`, 8 and have been removed: the queue's record now reads 30.5 CPU s over 2,189 tests, fuzzy 79.8 (11-06 14:49Z). **Read that section's "A slow test is understood, never hidden" before anything else.** This plan does it the way the rule says: the cause is found with a profile and fixed where it is.

Low priority: this plan yields to temper's plans for places.

**No build setting changes:** no `opt-level` and no `[profile.dev.package.*]` line for a hearth crate. **Every assertion and every case stays.** Don't move a test out of the focused suite to escape the budget, raise the budget, shrink a scenario, cut a seed or lower a count. What a test proves doesn't change, and neither does what hearth does for a user. Aim for 27 on the queue's figure, so ordinary noise doesn't put it back over. A builder reads higher than the queue, so a task aims at 85% of its own machine's figure at the plan's base.

## 1. A profile first

No suite profile has ever been taken. `valgrind` is on the builders (`perf` isn't). `valgrind --tool=callgrind` on the costly test binaries is enough; it's slow, so sample representative tests, not the suite. Put the result by function in the plan's `figures.md`: where the CPU goes, in hearth's code, in skein's, in the harness and in process startup. What's in it decides the rest of the plan.

## 2. Repairs

`hearth/docs/design/domain/testing.md`, 11 lists what nobody has tried:

- **Work a world repeats whatever its seed,** computed once in a test's process.
- **A scratch repository's objects written without `git add` and `git commit`:** the builder's real-git binary is 183 tests.
- **What the profile shows:** a scan where a lookup would do, setup repeated, data copied or encoded again, in the code under test or the test. A real inefficiency in hearth's production code is the best kind of fix here.

Earlier measurements, for where to look, on one builder with the overrides in place: the builder suite about 10 CPU s, real-git about 6.6, the tree suite about 3.9 (much more without them: the tree suite's pure computation is what they hid), the render suite about 3.5. No single test is over about 1.3.

## 3. What to report

Before and after, on one machine, by the method of `hearth/docs/design/domain/testing.status.md`, 9.1, and the queue's record after landing. Each commit says what it saved, measured. `testing.status.md`, 9.1, `testing.md`, 11 and `STATUS.md` state the result. If this can't reach the aim by what the rule allows, say what's left and by how much, land what helps, and put the measurements in the status for the user: `skein/docs/foundation/testing-strategy.md`, 8.1 says over budget ends in a ruling with measurements.

## Rules

- `skein/`, `smith/`, `jig/` and `temper/` change only if a profile shows a shared kit is the cost. That's a question first, not a task.
- A memory ceiling doesn't move for this, unless a measurement shows the peak moved, and the commit says why.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- Keep it small: stop once the aim is met.
- All parts must be in the plan when the cut ends.
