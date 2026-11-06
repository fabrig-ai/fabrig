+++
id = "core-view"
part = "conformance"
kind = "feature"
needs = ["trait-policy"]
files = ["jig/testing/jig-conformance/src/core/evidence.rs", "jig/testing/jig-core-kit/src/conformance.rs", "jig/tests/ops/src/conformance/observations.rs"]
effort = "high"
minutes = 35
+++

# `jig_conformance::core` builds the store view

## Goal

Jig D030's first peer the kit takes over. `jig-core-kit`'s `store_view` and ops's `core_store` both translate an application's rows and pending writes into `jig_conformance::core::Store`. They differ only in how a row and a write are read, and in the effects each adds.

Acceptance:

- `jig_conformance::core::view(applied, rows, pending, row, write)` builds the view (a `row` closure gives the core row of a stored one, a `write` closure a `Write`). Both adopters call it and add what's their own. Its exact parameters are your call, keeping its callers shorter than they are.
- `store_view` keeps its name and its public signature in `jig-core-kit`, and `core_store` keeps its in ops.
- `temper`'s reader of `jig_conformance::core::snapshot` doesn't change.
- No test changes. Net lines fall by about 30.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D030-conformance-not-drop-in.md`
- `jig/docs/design/domain/testing.md`, 5

## Owns and may change

- **Owns:** the files in the header.
- **Stay off:** `Observer::durable`'s wrapper in `conformance.rs`, which `observer-guard` changes.
- **The kit's `effects` part:** take it too if both adopters' bodies are the same; leave it if they differ.

## Steps

1. The builder, then each caller.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-conformance -p jig-core-kit -p jig-ops-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-conformance -p jig-core-kit -p jig-core-world -p jig-ops-world -p jig-conformance-world`. Commit and push.

## Report

The common note's items. This is a move: no rule to break by hand.
