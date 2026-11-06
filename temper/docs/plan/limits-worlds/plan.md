# limits-worlds: the real loop and the worlds on it, within the suites' CPU limits

The request: `inputs/request.md`. This is the repair of `skein/docs/foundation/testing-strategy.md`, 8.1, for the worlds that run whole deployments. No budget, timeout, deadline or build setting changes, and no test moves into the focused suite.

This file grows with each push: the belonging table, and the tasks that follow from it, come next.

## Parts and tasks so far

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `figures` | `tasks/figures-before.md` | | The five stories' *before* by instruction, and one ranked list of causes across the worlds |
| `cuts` | `tasks/cuts-fork.md` | | Every live commit still crashed, each forked from one baseline |
| `engine` | `tasks/engine-causes.md` | `figures-before` | The causes the worlds share, fixed in the crates under them |
| `public` | `tasks/public-causes.md` | `figures-before` | The largest focused test, at its cause in its world |
| `conformance` | `tasks/conformance-causes.md` | `cuts-fork` | What a child's replay still costs, at the harness and the adapter |

Critical path so far: `figures-before`, then `engine-causes`. `cuts-fork`, then `conformance-causes`, run beside it.

## What this plan decided

- **Temper's sweeps fork; they don't take jig's `ColdSnapshot`.** jig has a snapshot that resumes a story at a cut, and it needs each story written as a resumable state machine (`jig/tests/conformance/src/replay.rs`, `Continuation`), and `Clone` on every peer and fake. Temper's fourteen stories are straight-line functions with about forty nested drains, on a fake forge that isn't `Clone`. Rewriting them would cost more than the repair and make them harder to read. A forked process holds the position with no rewrite, and its child goes through the same `crash()` a fresh cut does. A child's `Outcome` is compared to a fresh cut's in a focused test, and the focused cuts stay fresh. The kit is `unsafe`, in one small module on the `ring` profile (`notes/contracts.md`).
- **Valgrind's copy is private.** The request names callgrind, and a builder was found without it today (`hearth/docs/plan/hearth-speed/figures.md`). `notes/measuring.md` has each task keep a copy in its slot's `target/` with `apt-get download` and `dpkg -x`, so no machine changes. If that fails, the task asks.
- **The real loop's patience is raised locally while profiling,** and never committed. Callgrind is 30 to 60 times slower, and the 20 s patience runs on the real clock. Polls look bigger under it, so `notes/measuring.md` has them read by call count.

- **Cause tasks are cut by crate, not by cause.** The causes aren't known until a profile runs, and a lead doesn't wait for one. So `figures-before` ranks them across the worlds, and each cause task owns a layer, fixes what the list gives it, and stops at the request's 2%. A cause in another's layer goes under `## Left`.

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, lists them. The early tasks stay off them: `figures-before` changes no code, and `cuts-fork` and `conformance-causes` are in the kits and the conformance world, which no other draft holds. `engine-causes` can't avoid them: a cause may sit in `temper-engine-domain/src/route/**` or `jig-protocol-store`, which `goal-delivery` and `model-catalogue-2` hold, so it keeps its hunks small and keeps their changes at a merge.
