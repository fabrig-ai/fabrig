+++
id = "observer-guard"
part = "replay"
kind = "feature"
needs = ["typed-outside", "alert-table", "core-view"]
files = ["jig/testing/jig-core-kit/src/conformance.rs", "jig/testing/jig-conformance/src/core.rs", "jig/tests/ops/src/conformance/mod.rs", "jig/docs/development/debt/D034-world-replay-cost.md"]
effort = "high"
minutes = 35
+++

# The core observer builds no view when nothing committed; D034 says what's left

## Goal

Jig D034's second clause, in the part this plan pays. `Observer::durable` builds `store_view` on every inspect and every core tick, though `durable_at` returns at once if nothing committed. A check on `applied` before the view removes that cost.

The clause itself isn't paid. Feeding the observer each commit's rows would still leave `snapshot(&store.rows)` and the referee's walk of every task per commit. That's a larger change, and with its equivalence test it would add test lines. So the item stays open for that alone, with figures.

Acceptance:

- `jig-core-kit`'s `Observer::durable` returns before `store_view` when `store.applied` equals what the inner observer has applied, through a small accessor on `jig_conformance::core`'s observer. Ops's copy of the wrapper does the same.
- **Measured:** the CPU of `jig-core-world` (focused) and `jig-conformance-world` and `jig-ops-world` (fuzzy) before and after this change, one run each, from nextest's summary on the same builder. Report the figures. Also report the share of `durable_at` in the same suites after the guard, with a throwaway timer you remove before pushing.
- No test's assertion changes.
- `jig/docs/development/debt/D034-world-replay-cost.md` is rewritten as it stands:
  - **Paid when** is "the core world's observer updates from each commit's rows". The other two clauses go, paid by this plan.
  - **Where** names `jig/testing/jig-conformance/src/core.rs` and `jig/testing/jig-core-kit/src/conformance.rs`.
  - **What** gives your measured share, and what stands in the row feed's way: `snapshot` rebuilt whole, its unkeyed `Vec`s and summed spend, the referee's walk per commit, `discarded`, `reached` and the generation bump not being idempotent under a feed, and `Got` changing rows outside commits.
  - **What pays it back** drops the `OnceLock`, and says nextest gives each test a process.
  - A line says `jig/tests/ops/tests/suite/fuzzy/joint.rs`'s six acceptance and move tests keep their own baselines on purpose: each runs its own seed.
- `python3 -I -B tools/citations/citations.py --all` reads 0 after you commit.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D034-world-replay-cost.md`

## Owns and may change

- **Owns:** the files in the header. `inbound`, `submitted` and `fresh_mark` also build the view. Take one only if it reads nothing when `applied` hasn't moved, and name it in your report. `cold` stays: `Got` changes rows outside commits.

## Steps

1. Measure first, then the guard, then measure again. Remove the timer.
2. The item's text.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-core-kit -p jig-conformance -p jig-ops-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-core-kit -p jig-core-world -p jig-conformance -p jig-conformance-world -p jig-ops-world`. Commit, run the citations tool and push.

## Report

The common note's items; the figures; net test lines across D034's tasks. The guard is a rule: make it return when `applied` has moved, name the test that fails, then restore.
