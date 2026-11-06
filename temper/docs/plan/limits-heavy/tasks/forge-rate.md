+++
id = "forge-rate"
part = "rate"
kind = "feature"
needs = []
files = [
  "temper/tests/engine/protocol-forge/tests/suite/rate.rs",
  "temper/tests/engine/protocol-forge/tests/suite/github_land.rs",
  "temper/docs/plan/limits-heavy/figures/forge-rate.md",
]
effort = "high"
minutes = 90
+++

# The forge's protocol world: `rate` and `github_land`

## Goal

`temper-engine-protocol-forge-world::suite`'s `rate` is 3 tests and 3.5 CPU s, two of them at 1.6, and its `github_land` is 8 tests and 2.4. They belong where they are (`temper/docs/design/domain/testing.md`, 6.1, *rate scenarios run here*). Find why simulating these requests costs what it does, and fix the cause.

Acceptance:

- All eleven tests still exist and assert what they assert today: the same per-hour totals and allowances in `rate`, on the same shapes, measured over a window that still shows the cycle its module comment says it shows.
- `temper/docs/plan/limits-heavy/figures/forge-rate.md` holds each `rate` test and the `github_land` tests you profiled or touched, before and after in instructions, with the cause of each.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.
- A product change has its own focused test, and changes no behaviour. Since request counts are what `rate` reads, `rate` itself holds that the traffic didn't change.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `temper/docs/design/domain/testing.md`, 2.2 and 6.1.
- `temper/docs/design/domain/forge.md`, 8.2: the one-update budget `rate.rs` names.
- `skein/docs/foundation/testing-strategy.md`, 8, *A slow test is understood, never hidden*.

## Owns and may change

Owns `rate.rs`, `github_land.rs` and its figures.

May change, at the size the cause needs: `temper/tests/engine/protocol-forge/src/**`, `temper/tests/engine/forge/src/**`, `temper-engine-domain-forge`, `temper-engine-domain-forge-change`, and `skein-forge-kit` or `skein-fake-forge-domain` where small and clearly theirs. `goal-delivery` and `look-round-13` hold parts of `temper-engine-domain-forge`. Name in your report every package outside the world you changed.

Options, each decided:

- **The request asks whether an hour is simulated where fewer show the rate.** At the cut's reading, it isn't: `pending_changes_installation_requests_per_hour` warms 120 s and measures one 40 s cycle scaled by ninety, and the busy shapes warm 300 s and measure 600 s scaled by six, stepping a second at a time. Read it again and say what you find. A window goes shorter only if the cycle it shows still fits whole in it, and the same allowance still holds, by the comment's own arithmetic; if that changes the allowance, leave the window alone.
- **The cost is per step of the world or its fake** (a snapshot, a scan, an invariant, encoding per step): fix it there. A world's check of an invariant stays; it may be made cheaper, never dropped.
- **The cost is the product's per-wake work:** fix it where a scan could be a lookup, keeping every request the same. If it needs a mechanism rather than a fix, it goes under *Found and not fixed*, with what it would take.
- **The cost is shared with the real loop or `limits-worlds`' worlds:** make the smallest fix that pays here, and say in your report that it's shared. If a merge meets `limits-worlds`, keep theirs.
- **A fix changes what a shared item means:** run the tests of every package that reads it, and say which.

## Steps

1. Profile `iterating_changes_installation_requests_per_hour`, `moving_main_installation_requests_per_hour` and `pending_changes_installation_requests_per_hour`, then `direct_landing_finish_and_two_writes` and one other of `github_land`, with `-p temper-engine-protocol-forge-world --test suite` and filters reading `rate::NAME` or `github_land::NAME`. Check the full names with `cargo nextest list`.
2. Starting points, to check and not findings: `busy_run` calls `scheduler.run(world, 1)` for each second, and `Scheduler::run` drains `take_seen` and walks every task's `due` and `until`. What does one world step cost, and what share of it is the check rather than the code? What's in `github_land`'s shared setup?
3. Fix the causes, largest first, re-measuring after each.
4. Write the figures.
5. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper-engine-protocol-forge-world --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages and their direct dependents. Commit and push.

## Report

What a simulated second costs and where it goes, before and after, the causes found and not fixed, and any product change with the test that holds it. Your implementation and check minutes apart.
