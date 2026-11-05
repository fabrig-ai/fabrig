+++
id = "tree"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/tree/tests/suite/memory.rs", "hearth/tests/tree/src/**", "hearth/crates/hearth-protocol-tree/**", "hearth/crates/hearth-domain-tree/**", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 80
+++

# The tree world under ceilings

## Goal

The tree world's memory tests, reads and walks both, hold fixed scenarios under named ceilings. The protocol's and the walk's formulas are named
for what they bound. This is the largest memory file of hearth's worlds, and most of its CPU is at-the-limit walks.

Acceptance:

- Every test of `hearth/tests/tree/tests/suite/memory.rs` named `..._worst_case` follows `notes/common.md`, "A ceiling", and none reads a formula
  as a bound.
- `the_bounds_are_none_past_a_u64_for_a_resolution_only` and `the_bounds_are_none_past_the_encoders_depth` keep every row, by
  `notes/contracts.md`, 5.
- `limits::decoder_bound`, `limits::decoded_bound`, `decode::decoder_bound` and `hearth_domain_tree::limits::walk_bound` follow
  `notes/contracts.md`, 1.
- `git grep -n -E 'worst_case|limits::held' -- hearth/tests/tree hearth/crates/hearth-protocol-tree hearth/crates/hearth-domain-tree` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1 and 5
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Not yours:** `hearth/tests/tree/tests/suite/frames.rs`. `focused-budget` takes it if it must.
- **Options, each pre-approved:**
  - The walks at `SHIPPED` carry entries of 4 MiB. Where one costs over 0.2 CPU s, run it at smaller limits with the entry scaled, as
    `walking_a_layer_at_its_limits_against_a_parent...` already does for attribute values, and scale its `> 4_000_000` to a named size.
  - `reading_generated_cut_and_mutated_frames...` is a table of generated frames: one ceiling on them all.
  - A test with `shipped` and `small` rows takes two ceilings by `notes/common.md`, "A ceiling", 4.

## Steps

1. `measure_tree`, `measure` and `peak_against` take the ceilings as arguments, and read no formula. Their `assert!(step.held() <= held ...)`
   lines become `HANDED_OUT_CEILING`s by `notes/common.md`, "A ceiling", 5.
2. Each test, with its constants beside it. Keep its assertions on what the scenario holds: `touched`, `deleted`, `held`, `shipped`.
3. The comment on `walking_a_layer_at_its_limits_against_a_parent...` says `worst_case`: say what's true now.
4. Rename the formulas and their tests. `the_walk_bound_is_a_sum_that_fits_or_nothing` stays in
   `hearth/crates/hearth-domain-tree/src/tests/bounds.rs`.
5. `cargo fmt`, clippy on `hearth-tree-world`, `hearth-protocol-tree` and `hearth-domain-tree`, focused tests of all three, then
   `heavy cargo check -p hearth-protocol-builder -p hearth-builder-world --tests`.
6. `notes/common.md`, "The end of every task". Commit, with the peaks, and push.

## Report

The first line, the tests run and how many, a table of tests with peak and ceiling, what bit, what you scaled, and CPU seconds before and after.
