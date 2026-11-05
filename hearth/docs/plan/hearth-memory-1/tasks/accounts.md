+++
id = "accounts"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/tests/accounts/**", "hearth/crates/hearth-protocol-accounts/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++

# The accounts world under a ceiling

## Goal

The accounts world's memory test holds a fixed scenario under a named ceiling, and `hearth_protocol_accounts::worst_case` becomes `decoder_bound`.

Acceptance:

- `a_read_at_its_limits_holds_no_more_than_its_ceiling` replaces `a_read_at_its_limits_holds_no_more_than_its_worst_case`, by
  `notes/common.md`, "A ceiling". It keeps both files, both rows, and its assertion that the fixture fills the limits.
- `decoder_bound` and its test follow `notes/contracts.md`, 1 and 5.
- `git grep -n worst_case -- hearth/tests/accounts hearth/crates/hearth-protocol-accounts` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1 and 5
- `hearth/docs/design/domain/testing.md`, 8.4
- `skein/docs/foundation/testing-strategy.md`, 6

## Owns and may change

- **Owns:** the files under `files`.
- **Option, pre-approved:** `fixtures::LIMITS` is `SHIPPED`. If the test costs over 0.2 CPU s, give it smaller limits, and change its `> 1_000`
  to a named size that still shows the limits filled.

## Steps

1. `hearth/tests/accounts/src/world.rs`: `read_metered(file, bytes, limits, ceiling)` checks against `ceiling`. It reads no formula.
2. `hearth/tests/accounts/tests/suite/memory.rs`: the test, with `const CEILING: u64`.
3. Rename the formula and its test in `hearth/crates/hearth-protocol-accounts`.
4. `cargo fmt`, clippy on `hearth-accounts-world` and `hearth-protocol-accounts`, focused tests of both. Nothing else depends on the crate.
5. `notes/common.md`, "The end of every task". Commit, with the peak, and push.

## Report

The first line, the tests run and how many, the peak and ceiling, what bit, and CPU seconds before and after.
