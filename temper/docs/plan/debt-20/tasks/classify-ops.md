+++
id = "classify-ops"
part = "classifiers"
kind = "feature"
needs = []
files = ["jig/examples/ops/client-domain/src/parts.rs", "jig/examples/ops/client-domain/src/observability.rs", "jig/examples/ops/client-domain/src/infrastructure.rs", "jig/examples/ops/client-view/src/observability.rs", "jig/examples/ops/client-view/src/infrastructure.rs"]
effort = "medium"
minutes = 25
+++

# jig: `ops` asks `ApplicationPart` once

## Goal

Part of `jig/docs/development/debt/D015-exhaustive-lists.md`, and the only real repetition `ops` has. `part_fits` is written in both `client-domain/src/observability.rs` and `infrastructure.rs`, and an `ApplicationPart` list in both `client-view/src/observability.rs` and `infrastructure.rs`. Each goes once. `ApplicationPart` is `jig-client-domain`'s, in a file another plan is changing, so these are `ops`' own functions: this one departs from the note's "method on the type" for that reason.

Acceptance:

- `rg -n 'fn part_fits' jig/examples/ops/client-domain/src` lists one line, in `parts.rs`, and the two views' lists read one function, in `client-view/src/drawing.rs` or wherever the view's shared drawing reads best.
- The item's **Measure** on `jig/examples/ops` reads at most 565 (579 at base).
- `heavy cargo nextest run -p jig-ops-client-domain -p jig-ops-client-view --lib` and `heavy cargo nextest run -p jig-ops-world --test suite` pass, with no assertion changed. `tests.rs`' resource bounds on both connectors pin `part_fits`: break it by hand, and it goes red. Restore.
- The branch's net production lines are at most 0 (estimate: -25).

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/classifiers.md`: the rule. **This task departs from its "pin first":** `part_fits`' pins exist, and the views' list is one partition moved, so no new table.
- `jig/docs/development/debt/D015-exhaustive-lists.md`

## Owns and may change

- **Owns:** the files in `files`, and `client-view/src/drawing.rs` if the function goes there. **Stay off** `jig/crates/jig-client-domain/src/contract/common.rs`.
- **`client-view/src/infrastructure.rs`' `Kind` lists and `client-domain/src/infrastructure/values.rs`:** one partition each if they prove the same, in which case they go too. Otherwise they stay.
- **What stays:**
  - `domain/src/client_documents.rs` and `route.rs`'s 38 `core::Held` variants are routing or pass-through, and a method on `Held` would move them into `vocabulary.rs`, another plan's file, and remove nothing.
  - `domain/src/admission.rs`: one list per type, so a method only moves them; and its third list is a re-narrowing that needs a redesign.

## Steps

1. The two moves.
2. Break `part_fits` by hand.
3. The note's run, with `-p jig-ops-client-domain -p jig-ops-client-view`. Commit and push.

## Report

The note's items.
