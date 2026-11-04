# D012: Test shapes the render run left

- **Kind:** tests
- **Where:** `hearth-domain-render/src/tests/fixtures/hearth.rs`, `hearth-domain-render/src/tests/fixtures/debian.rs`, `hearth-domain-render/src/tests/compose/hearth.rs`, `hearth-domain-render/src/tests/fixtures/collisions.rs`, `hearth-domain-render/src/tests/fixtures/entries.rs`, `tests/render/tests/suite/oracle.rs`, `tests/render/tests/suite/oracle/engine.rs`, `tests/builder/tests/suite/properties.rs`
- **Seen:** 2026-10-18 at `56096145d0`, in render's retrospective (its list of debts, items 2 to 5)
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** low (tables that read as tables)
- **Size:** S
- **Status:** open
- **Paid when:** `target_with` and `debian_layers` take named fields, one `RANGE_BASE` and one `RANGE_COUNT` exist or the two have different names, the oracle's rows hold plain values, and the cut-pipe tests are one table.
- **Measure:** `git grep -E 'Outcome::Text\(Cow::Borrowed' -- hearth/tests/render | wc -l`: 63

## What

- `target_with` and `debian_layers` take `[Option<&str>; 6]`, six unnamed
  slots, and the tables in `tests/compose/hearth.rs` fill them by position.
- `RANGE_BASE` and `RANGE_COUNT` are each defined twice with different
  values: 42 and 2 in `fixtures/entries.rs`, `kinds::MIN_SUBORDINATE` and
  that again in `fixtures/collisions.rs`.
- The oracle's rows wrap 63 expected values in
  `Outcome::Text(Cow::Borrowed(..))`.
- `tests/builder/tests/suite/properties.rs` has six numbered tests,
  `..._through_cut_pipes` and `_1` to `_5`, each calling
  `cut_pipe_digests` with its number.

## Why it matters

Each makes a table harder to read or extend. One name with two values in
one fixture module invites the wrong import.

## What pays it back

One small pass: a named struct, a rename, a constructor for the oracle's
rows, and a table of batches. The six cut-pipe tests may be split so they
run apart: check that before folding them into one test.
