+++
id = "template"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/template/**", "hearth/crates/hearth-protocol-template/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++

# The template world under ceilings

## Goal

The template world's two memory tests hold fixed scenarios under named ceilings, and its formulas become `decoder_bound`.

Acceptance:

- `compiling_at_the_limits_holds_no_more_than_its_ceiling` and `compiling_templates_and_strings_at_the_limits_holds_no_more_than_their_ceilings`
  replace the two tests named `..._the_worst_case`, by `notes/common.md`, "A ceiling". Every row of `fixtures::at_the_limits` and
  `fixtures::templates_at_the_limits` stays.
- The second test has a ceiling each for templates and strings, `TEMPLATE_CEILING` and `STRING_CEILING`.
- `limits::decoder_bound` and `expression::decoder_bound` follow `notes/contracts.md`, 1, with a test of the value for each `Decoder`.
- `git grep -n worst_case -- hearth/tests/template hearth/crates/hearth-protocol-template` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Option, pre-approved:** `fixtures::LIMITS` is `SHIPPED`, and other tests read it. If `compiling_at_the_limits...` costs over 0.2 CPU s, add a smaller
  limits constant for it alone, and scale its rows.

## Steps

1. `hearth/tests/template/tests/suite/memory.rs`: the two tests.
2. Rename the formulas in `hearth/crates/hearth-protocol-template/src/limits.rs` and `expression.rs`. Add the value test to
   `hearth/crates/hearth-protocol-template/src/tests.rs`.
3. `cargo fmt`, clippy on `hearth-template-world` and `hearth-protocol-template`, focused tests of both, then
   `heavy cargo check -p hearth-protocol-builder -p hearth-builder-world --tests`.
4. `notes/common.md`, "The end of every task". Commit, with the peaks, and push.

## Report

The first line, the tests run and how many, the peaks and ceilings, what bit, and CPU seconds before and after.
