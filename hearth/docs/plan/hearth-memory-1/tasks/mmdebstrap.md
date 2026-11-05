+++
id = "mmdebstrap"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/mmdebstrap/**", "hearth/crates/hearth-protocol-mmdebstrap/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++

# The mmdebstrap world under ceilings

## Goal

The mmdebstrap world's two memory tests hold fixed scenarios under named ceilings, and its three formulas become `decoder_bound`.

Acceptance:

- `an_output_reader_at_its_limits_holds_no_more_than_its_ceiling` and `a_stanza_reader_at_its_limits_holds_no_more_than_its_ceiling` replace
  the two tests named `..._its_worst_case`, by `notes/common.md`, "A ceiling".
- `output::decoder_bound`, `packages::decoder_bound` and `tail::Tail::decoder_bound` follow `notes/contracts.md`, 1, with tests of their values.
- `git grep -n worst_case -- hearth/tests/mmdebstrap hearth/crates/hearth-protocol-mmdebstrap` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Option, pre-approved:** both tests run at `limits::SHIPPED`. If one costs over 0.2 CPU s, add `fixtures::SMALL`, as `hearth/tests/git/src/fixtures.rs` has one, and
  run both fixtures at it.

## Steps

1. `hearth/tests/mmdebstrap/src/world.rs`: `read_output_metered` and `read_index_metered` take `ceiling: u64`, and read no formula.
2. `hearth/tests/mmdebstrap/tests/suite/memory.rs`: the two tests, each with its `const CEILING: u64`.
3. Rename the formulas. Add the value tests to `hearth/crates/hearth-protocol-mmdebstrap/src/tests/output.rs` and `packages.rs`.
4. `cargo fmt`, clippy on `hearth-mmdebstrap-world` and `hearth-protocol-mmdebstrap`, focused tests of both, then
   `heavy cargo check -p hearth-protocol-builder -p hearth-builder-world --tests`.
5. `notes/common.md`, "The end of every task". Commit, with the peaks, and push.

## Report

The first line, the tests run and how many, the peaks and ceilings, what bit, and CPU seconds before and after.
