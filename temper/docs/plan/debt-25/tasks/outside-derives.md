+++
id = "outside-derives"
part = "replay"
kind = "interface"
needs = ["core-view"]
files = ["jig/testing/jig-conformance/src/**", "jig/testing/jig-core-kit/src/**", "jig/testing/jig-test-store-files/src/**", "jig/testing/jig-test-system/src/**", "jig/testing/jig-test-profile/src/**"]
effort = "medium"
minutes = 20
+++

# The worlds' outside state can be compared

## Goal

Jig D034's first clause needs `PartialEq` and `Eq` on what `replay::outside` compares, which today only needs `Debug`. This task adds the derives and nothing else, so `typed-outside` builds on them.

Acceptance:

- `PartialEq, Eq` are derived on `Referee` and its substates, `Observed`, `Observer`, `Intents`, `Names`, `Makes`, `SeenFence`, `Effect` and `NamedRevision` (jig-conformance); `Observer`, `Peers` and `Stand` (jig-core-kit); `Creating`, `Party`, `Worker`, `Run`, `System`, `Pending` (the fake parties, workers and system); the store's `Pending`; and what else the compiler asks for, in the kits under `jig/testing`.
- **No step crate changes.** If a type in `jig/crates/**` would need a derive, don't add it: find what holds it, and stop and report. `core::Event` has no `Eq`, and `busy_events` is `typed-outside`'s to change.
- Nothing else changes. The lines are the derives alone.

The compile check that proves this is `typed-outside`'s `Outside`. To check here, add a throwaway `const _: fn() = || { fn eq<T: Eq>() {} eq::<jig_conformance::Referee>(); };` in an untracked scratch and delete it before pushing, or just compile the derives: an impossible derive fails on its own.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`, `Outside`
- `jig/docs/development/debt/D034-world-replay-cost.md`

## Owns and may change

- **Owns:** the globs in the header, plus the fake parties' and workers' kits under `jig/testing`, for a derive line only. Find them from the compiler's errors.

## Steps

1. The derives, then `heavy cargo check -p jig-conformance -p jig-core-kit --tests`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` on the packages you changed, `--all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance -p jig-core-kit`. Commit and push.

## Report

The common note's items; the types derived; any that stopped you. Derives only: no rule to break by hand.
