+++
id = "scale-restart"
part = "scale"
kind = "feature"
needs = []
files = [
  "temper/tests/engine/domain/tests/suite/fabrig_scale.rs",
  "temper/crates/temper-box/src/tests/restart.rs",
  "temper/docs/plan/limits-heavy/figures/scale-restart.md",
]
effort = "high"
minutes = 90
+++

# A goal tree's 384 tasks, and a box's restart in every ending order

## Goal

Two rows, in two packages. `temper-engine-domain-world::suite`'s `fabrig_scale` is 2 tests and 3.1 CPU s, `a_goal_tree_takes_384_tasks` 2.7. `temper-box`'s `tests::restart` is 8 tests and 2.6. They belong where they are (`plan.md`, *Belonging*). Find why each costs what it does, and fix the cause.

Acceptance:

- `a_goal_tree_takes_384_tasks` still takes a tree of 384 tasks, and `eighty_chats_are_funded_at_once` still funds eighty, asserting what they assert today. `temper/docs/design/protocol/setup.status.md`, 8.3, leans on both.
- `an_aborted_serve_restarts_in_every_ending_order` and its three neighbours still run all 720 orders: no sampling. `ORDERS` stays.
- `temper/docs/plan/limits-heavy/figures/scale-restart.md` holds the tests you profiled or changed, before and after in instructions, with the cause of each.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.
- A product change has its own focused test, and changes no behaviour.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*, and 8.1.

## Owns and may change

Owns `fabrig_scale.rs`, `restart.rs` and its figures. No nextest filter or document names their tests; `setup.status.md` cites `fabrig_scale.rs` by path, which stays.

May change, at the size the cause needs:

- For `fabrig_scale`: `temper/tests/engine/domain/src/**` (`limits-fixtures` is working on this world's fixtures, so keep hunks small), `temper-engine-domain`, and `jig-domain-tasks` or `jig-domain-core` where small and clearly theirs (`ui-model-next-turn` and `model-catalogue-2` are in both).
- For `restart`: `temper/crates/temper-box/src/tests/fixtures.rs` and `temper-box/src/**`, and `skein-sim` or `skein-fake-machine` where small and clearly theirs.

Options, each decided:

- **Something in the engine is more than linear in the tree** (a scan of live tasks or a ledger walk per delegate): that's a product cause, and a person's box pays it at scale. Fix it where a lookup would do, if it's small. If it isn't, it goes under *Found and not fixed*, with the shape of the fix and what it'd save.
- **The cost is the world's checking after each step:** a check stays, but may be made cheaper. A world shared with `limits-worlds` or `limits-fixtures` takes the smallest fix that pays here, and you say so in your report.
- **The test's own helpers are the cost** (`live`, `pool`, `assignment`, or a delegate cloned per child): fix them in the test.
- **`probing_serve` is built for each of the 720 orders:** the strategy's *a fixture built once per test, not per case* is the first thing to try: run it once and copy its state per order, if the harness and simulator can be cloned at a small cost. If they can't, profile what `Harness::with_stubborn` and `probing_serve` spend (the trace walked for the probe's connection, say) and make that cheaper. A new `Clone` on a skein type is a skein change: fine if small.
- **The four tests are 720 cases each and the cost is linear in them:** that is what they hold. What's left goes under *Found and not fixed* with its count.

## Steps

1. Profile `a_goal_tree_takes_384_tasks` and `eighty_chats_are_funded_at_once` with `-p temper-engine-domain-world --test suite`, filters reading `fabrig_scale::NAME`. Compare with a smaller tree by hand, if that shows the shape: a local change, gone before you push.
2. Profile `an_aborted_serve_restarts_in_every_ending_order` and `early_reply_pipe_failure_and_busy_logs_allow_restarting` with `-p temper-box --lib`, filters reading `tests::restart::NAME`. Check full names with `cargo nextest list`.
3. Fix the causes, largest first, re-measuring after each. The two rows are independent: one's stop isn't the other's.
4. Write the figures.
5. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper-box -p temper-engine-domain-world --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages and their direct dependents. Commit and push.

## Report

What a task of the tree costs as the tree grows, whether it's linear, and what a restart order costs, before and after. The causes found and not fixed, and any product change with the test that holds it. Your implementation and check minutes apart.
