+++
id = "config"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/config/**", "hearth/crates/hearth-protocol-config/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++

# The config world under ceilings

## Goal

The config world's two memory tests hold fixed scenarios under named ceilings, and its formulas are named for what they bound.

Acceptance:

- `each_step_holds_no_more_than_its_ceiling_under_tiny_limits` and `declarations_hold_no_more_than_their_ceiling_at_the_limits` replace the two
  tests named `..._worst_case...`, by `notes/common.md`, "A ceiling". Each keeps its row count and its `> 0`.
- `json::decoder_bound`, `json::decoded_bound`, `schema::decoder_bound` and `declaration::decoder_bound` follow `notes/contracts.md`, 1. `json::decoder_bound`
  and `declaration::decoder_bound` have tests of their values.
- `git grep -n -E 'worst_case|json::held' -- hearth/tests/config hearth/crates/hearth-protocol-config` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Keep the tiny limits:** `memory::JSON` and `memory::DECLARATION` already make these scenarios small.

## Steps

1. `hearth/tests/config/src/fixtures/metered.rs`: `metered`, `metered_declaration` and `measure` name their `bound` argument `ceiling`.
2. `hearth/tests/config/tests/suite/memory.rs`: the two tests, each with its `const CEILING: u64`.
3. Rename the formulas in `json.rs`, `schema.rs` and `declaration.rs`. Add the value tests to
   `hearth/crates/hearth-protocol-config/src/tests/json.rs` and `declaration.rs`.
4. `cargo fmt`, clippy on `hearth-config-world` and `hearth-protocol-config`, focused tests of both, then
   `heavy cargo check -p hearth-protocol-builder -p hearth-builder-world --tests`.
5. `notes/common.md`, "The end of every task". Commit, with the peaks, and push.

## Report

The first line, the tests run and how many, the peaks and ceilings, what bit, and CPU seconds before and after.
