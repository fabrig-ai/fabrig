+++
id = "setup-steps"
part = "service"
kind = "feature"
needs = ["service-fixture"]
files = [
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_*.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/**",
  "temper/docs/plan/limits-fixtures/figures/setup-steps.md",
]
effort = "high"
minutes = 90
+++

# setup-steps: what each `setup` test does beyond its fixture

## Goal

`setup` is one of the suite's two largest single sets: 238 tests, 24.9 CPU s at a 77 ms median. `service-fixture` has made what they build cheaper. Profile what most of them still repeat after that, and fix it (request, item 2).

**Acceptance:**

- `figures/setup-steps.md` gives `Ir` before and after, by test name, for the cheapest and a median test of `setup::`, and for a test that drives a request through `Face`, each saying what it spends beyond its fixture. Use `figures/service-fixture.md`'s `setup::` tests where they fit, so the two tables read together.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take.
- Every `temper-service` test passes, with the base's count plus any you add. A test checks what it checked: a loop that stops early stops only once nothing more can happen, and every assertion still runs.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 2 and 3 and `For every task of this plan`
- `temper/docs/plan/limits-fixtures/figures/service-fixture.md`, what the fixture's task found
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1

## Owns and may change

- **Owns:** `temper/crates/temper-service/src/tests/setup.rs` and `tests/setup_*.rs`, including `Face`, `start`, `terminal` and each file's helpers; `temper/crates/temper-service/src/setup.rs` and `src/setup/**`, where the profile puts a cause, with its own focused test; `figures/setup-steps.md`.
- **Leave to `limits-heavy`:** `setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests`, which that plan owns as a heavy test. Its helpers may change, if its body doesn't.
- **May change, where the profile puts the cause, small, with a focused test:** `temper/crates/temper-forge-setup/src/**`, `skein/crates/skein-forge-setup/src/**`, `skein/crates/skein-fake-forge-forgejo/src/**`, `skein/crates/skein-fake-forge-github/src/**`, `skein/testing/skein-world/src/**`.
- **`tests/fixtures.rs` stays as `service-fixture` left it.** If its fixture is still a large cost, a line in your causes table says how much, and what it would take. One edit is allowed if it's a call-site or a private helper that saves over the stop rule, and your report names it.
- **Options, each pre-approved:**
  - **If `Face`'s fixed loops are the cost** (below), turn until nothing more can happen: nothing queued on `setup.io`, nothing the wire would answer, `engine.setup.above` drained as the test expects, and no deadline due at this time. Keep a bound on the loop, which panics if it's passed. Add a small helper and use it where the loops are alike. If a test's wait depends on the count itself, leave that test's loop as it was, and say so.
  - **If `advance` is the cost per turn** (`src/setup.rs:623`), find which part: a scan where a lookup would do is the product's, and the box pays it too.
  - **If `fake::Server::new` or `forge_reply`'s 4,096 turns are the cost** (`tests/setup_scopes.rs:19`), the same: run until closed, with a bound.
  - **If encoding or rendering a page is the cost,** that's what a test checks, so leave it unless the profile shows it done twice.

## Steps

**From reading, not found (`notes/common.md`):**

- `Face::reply_status` (`tests/setup.rs:330`) and `Face::sent` (`:335`) each run a fixed 1,024 `Face::turn`s, and each turn runs `advance` (`src/setup.rs:623`). A test that exchanges a few requests may so run thousands of `advance`s, most with nothing to do.
- There are more loops of that kind: 4,096 turns in `tests/setup_admin.rs` (`:452`, `:468`, `:484`, `:517`), `tests/setup_reachable.rs:44`, `tests/setup_scopes.rs:19` and `tests/setup.rs:909`, and 1,024 in `tests/setup_lifecycle.rs` (`:566` and five more) and `tests/setup.rs` (`:1178`, `:1481`, `:1630`).

**The work:**

1. Choose the tests and take the before figures, at your branch's start.
2. Read the inclusive annotations. Split each test's `Ir` into its fixture and what follows, by the fixture's call in the inclusive view.
3. Fix the largest cause. Run the narrowest test (`heavy cargo nextest run -p temper-service --lib -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-service`. A product change also runs the focused tests of what reads it, which `cargo tree -i PKG --depth 1` lists, with `heavy cargo check -p PKG --tests` for those in another tree.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, any loop you left as it was and why, and what `setup`'s median reads now against 77 ms by `Ir`.
