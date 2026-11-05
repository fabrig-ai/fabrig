+++
id = "render"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/render/**", "hearth/crates/hearth-domain-render/**", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 90
+++

# The render world under ceilings

## Goal

The render world's ten memory tests hold fixed scenarios under named ceilings, `parent::checked` drops its `Memory` argument, render's own sums
go, and what stays is named for what it bounds.

Acceptance:

- Every test of `hearth/tests/render/tests/suite/memory.rs` follows `notes/common.md`, "A ceiling", and none reads a formula.
- `parent::checked` and `checked_diff` have the signatures of `notes/contracts.md`, 3.
- `fixtures::decoded_bound`, `fixtures::reuse::bound` and `fixtures::specs::bound` are gone.
- The formulas of `notes/contracts.md`, 1, "Goes, in `render`", are gone, and `evaluation_bound` and `run_bound` stand, by sections 1 and 5.
- `git grep -n -E 'worst_case|Memory::Checked' -- hearth/tests/render hearth/crates/hearth-domain-render` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1, 3 and 5
- `hearth/docs/design/domain/render.md`, 8.4
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Names:** `rendering_every_target_holds_one_targets_settings` keeps its name. `templates_keep_expanded_entries_within_the_bound` becomes
  `templates_keep_expanded_entries_within_their_ceiling`.
- **Options, each pre-approved:**
  - Several scenarios run at `SHIPPED`, with 16 MiB of blobs or values. Where one costs over 0.2 CPU s, give it smaller limits by the shape of
    `fixtures::settings::LIMITS`, and scale its rows. `fixtures::memory::TEMPLATE_LIMITS`, `fixtures::reuse::LIMITS` and `SHIPPED` in
    `hearth/tests/render/src/fixtures/decoded.rs` are the places.
  - A test's row may scale, but no row goes.
  - `hearth-domain-render`'s `limits` module keeps `Limits`, `SHIPPED` and `FOUR_MADE`.

## Steps

1. `hearth/tests/render/src/parent.rs`, by `notes/contracts.md`, 3. `drive` reads its ceiling with `if let Memory::Within(ceiling) = memory`, so that
   `Memory::Checked` isn't named.
2. `hearth/tests/render/src/fixtures/memory.rs` and `decoded.rs`: `each`, `peak`, `checked_decoding` and `decoded_step` name their argument
   `ceiling`.
3. Each test of `hearth/tests/render/tests/suite/memory.rs`, with its `const CEILING: u64`. Keep `run.outcome`, `run.answers`, `second_reads` and the
   rest. `collecting_errors_at_the_limits...` keeps its three kinds of step.
4. Delete the formulas that go, and whatever they alone read in `hearth-domain-render`. Rename `evaluate::worst_case` and
   `evaluate::template::worst_case`, and the test in `hearth/crates/hearth-domain-render/src/tests/templates.rs`. Add the `Some(..)` row it lacks
   for `run_bound`, as a test of its own beside it, and one for `evaluation_bound`.
5. `cargo fmt`, clippy on `hearth-render-world` and `hearth-domain-render`, focused tests of both, then
   `heavy cargo check -p hearth-domain-builder -p hearth-builder-world --tests`.
6. `notes/common.md`, "The end of every task". Commit, with the peaks, and push.

## Report

The first line, the tests run and how many, a table of tests with peak and ceiling, what bit, what you scaled, every helper you removed, and CPU
seconds before and after.
