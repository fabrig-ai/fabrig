+++
id = "alert-table"
part = "replay"
kind = "feature"
needs = []
files = ["jig/tests/ops/tests/suite/fuzzy/alert_cuts.rs", "jig/tests/ops/tests/suite/stories.rs"]
effort = "medium"
minutes = 30
+++

# The alert story's cuts share one baseline

## Goal

Jig D034's third clause. Six tests each run the whole alert story again before cutting it, which is 12 runs. The item's `OnceLock` can't share across tests: nextest gives each test a process (`skein/docs/foundation/testing-strategy.md`, 8). So one table test runs the baseline once and cuts it six times, which is 7 runs. `assert_alert` is also written twice.

Acceptance:

- `fuzzy/alert_cuts.rs` holds one `#[test]` under the `fuzzy` module, with a table of six named rows (`before the lead receipt`, `at the lead receipt`, `after the lead receipt`, `at the person's acceptance`, `at the restart's entry`, `after the restart's entry`), each giving its commit from the one baseline.
- A failure names its row, through the assertion's message or a `println!` before the cut.
- `assert_alert` lives once. `fuzzy/alert_cuts.rs` reads `stories.rs`'s, which are in the same binary. Its body doesn't change.
- Each row still asserts what its test did (`crashes == vec![commit]`, then `assert_alert`).
- Net test lines fall by about 75.

**Measure it:** the merged test's time on a builder, against the 10 s guide for a fuzzy test (`skein/docs/foundation/testing-strategy.md`, 8). Under it, you're done. Over it, stop before pushing a merged test: push nothing merged, report `Question:` with the measured figures for one run and for the seven, and recommend either two tests of three rows or keeping the clause open. Moving the test or raising a guide isn't an option.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `jig/docs/development/debt/D034-world-replay-cost.md`
- `skein/docs/foundation/testing-strategy.md`, 8

## Owns and may change

- **Owns:** the files in the header. Test names change by this spec's leave; `test(/^fuzzy::/)` still selects the test.
- **Stay off** `fuzzy/joint.rs`: its six `p_acceptance` and `q_move` tests run seeds 531 to 536, one each, so a shared baseline would narrow them to one seed. They stay, and `observer-guard` writes that down in the item.

## Steps

1. The table, then the shared `assert_alert`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-ops-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-ops-world --profile fuzzy -E 'test(alert)'` and `heavy cargo nextest run -p jig-ops-world -E 'test(alert)'`. Commit and push.

## Report

The common note's items; the runs before and after; the test's time. This is a move: no rule to break by hand.
