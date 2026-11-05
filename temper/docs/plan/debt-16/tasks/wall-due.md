+++
id = "wall-due"
part = "wall-due"
kind = "feature"
needs = []
files = ["skein/crates/skein-lib/src/env.rs", "skein/crates/skein-lib/src/tests/env.rs", "jig/testing/jig-test-domain/src/lib.rs", "jig/examples/ops/domain/src/domain.rs"]
effort = "medium"
minutes = 15
+++

# skein: `Env::wall_due` goes for a plain comparison

## Goal

`Env::wall_due` compares two walls by way of the monotonic clock, and its test pins a wrong answer: a future expiry reads
as due once `now` saturates. `Wall` is `Ord`, so a wall expiry is `expiry <= env.wall` at the call site. Finding 5 of the
review.

Acceptance:

- `rg -n 'wall_due' skein jig temper smith hearth --glob '*.rs'` finds nothing.
- `skein/crates/skein-lib/src/env.rs` no longer imports `Duration`; `Env` keeps `now`, `wall`, `limits` and `with`.
- `a_child_environment_keeps_the_clocks_and_owns_its_limits` is untouched and passes.
- Net about -19 lines, production and tests together.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `skein/docs/design/lib.md`, 9: this cut has already taken `wall_due` out of it.

## Owns and may change

- **Owns:** `skein/crates/skein-lib/src/env.rs`; `skein/crates/skein-lib/src/tests/env.rs`.
- **May change, this comparison only:** `jig/testing/jig-test-domain/src/lib.rs` (two uses in `timers_due`),
  `jig/examples/ops/domain/src/domain.rs` (one use in its `timers_due`).
- **The one approved test change:** `wall_expiry_uses_the_steps_clocks_and_saturates` is deleted. No other assertion
  moves.
- `jig/docs/development/debt/reviews/2026-10-31-jig.md` names `wall_due`: a dated review, left as it is. The citations
  check doesn't read it as a code name.

## Steps

1. `env.rs`: delete `wall_due` and the import it alone used.
2. `tests/env.rs`: delete the test; leave the imports the other test still uses.
3. The three call sites read `due <= env.wall`.
4. `heavy cargo nextest run -p skein-lib --lib -E 'test(env)'`, then `heavy cargo nextest run -p jig-test-domain -p
   jig-ops-domain`.
5. `cargo fmt`; `heavy cargo clippy -p skein-lib -p jig-test-domain -p jig-ops-domain --all-targets -- -D warnings`; the
   citations check. Commit and push.

## Report

The common note's items. No new test: nothing to break by hand.
