+++
id = "sums"
part = "sums"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-service-builder/**", "hearth/crates/hearth-domain-builder/src/lib.rs", "hearth/crates/hearth-protocol-builder/src/limits.rs", "hearth/crates/hearth-domain-store/src/limits.rs", "hearth/crates/hearth-domain-build/src/limits.rs", "hearth/crates/hearth-domain-build/src/tests.rs", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++

# The builder's sums go

## Goal

`hearth_service_builder::worst_case`, its two tests, and the sums that fed only it are deleted. Every other family's task builds on this: a
formula that no sum reads can be renamed in its own crate alone.

Acceptance:

- The five formulas of `notes/contracts.md`, 1, "Goes, in `sums`", and the tests the table names, are gone.
- `git grep -n worst_case -- hearth/crates/hearth-service-builder hearth/crates/hearth-domain-builder hearth/crates/hearth-domain-store hearth/crates/hearth-domain-build hearth/crates/hearth-protocol-builder/src/limits.rs`
  prints nothing.
- The `worst_case` count in `hearth/scripts/old-forms.txt` is lowered. So is `max_out`, by the one line that goes with
  `hearth_domain_build::limits::worst_case`.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1
- `hearth/docs/design/domain/render.md`, 8.4
- `skein/docs/foundation/programming-model.md`, 6.3
- `hearth/docs/development/debt/D002-process-worst-case.md`

## Owns and may change

- **Owns:** the files under `files`.
- **May change, on a helper that nothing reads any more:** a `use` line, a test module's `mod` line, and a `price` function or constant whose
  only reader was a deleted sum. If one of these has another reader, leave it and say so.
- **In `hearth-service-builder/src/tests/fixtures.rs`:** remove `LARGER_QUEUE` and `BUILDER_BYTES`. Remove `tests/memory.rs` and its `mod` line.
- **Not yours:** the formulas under them. `render::limits::worst_case`, `git::limits::worst_case`, `tree::limits::worst_case` and the rest stay
  for their families' tasks, and lose a reader here.

## Steps

1. Delete, in this order: `hearth_service_builder::worst_case` with `src/tests/memory.rs`; `hearth_domain_builder::worst_case`;
   `hearth_protocol_builder::limits::worst_case`; `hearth_domain_store::limits::worst_case`; and `hearth_domain_build::limits::worst_case`
   with `the_plan_fits_its_worst_case` in `hearth/crates/hearth-domain-build/src/tests.rs`.
2. Keep every other test of these crates as it is. `Limits`, `SHIPPED` and `Walk` in `hearth-protocol-builder/src/limits.rs` stay.
3. `cargo fmt`, then clippy on `hearth-service-builder`, `hearth-domain-builder`, `hearth-protocol-builder`, `hearth-domain-store` and
   `hearth-domain-build`.
4. Focused tests of those packages, then `heavy cargo check -p hearth -p hearth-builder-world --tests`.
5. Lower the counts (`notes/common.md`, "The end of every task"). Commit and push.

## Report

The first line, the tests run and how many, every helper you removed beyond the table, and the old-form counts before and after.
