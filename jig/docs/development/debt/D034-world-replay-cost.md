# D034: The conformance and core worlds redo whole runs to check a cut

- **Kind:** tests
- **Where:** `tests/conformance/src/replay.rs`, `tests/conformance/tests/suite/snapshot.rs`, `tests/conformance/tests/suite/fuzzy/snapshot.rs`, `tests/core/src/conformance.rs`, `tests/ops/tests/suite/fuzzy/alert_cuts.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Test cost": conformance and `ops`'s alert story; the core world's observer)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Ops (A12)": `alert_cuts` replays its baseline; "Tech-debt cross-checks still open": the core observer rebuilds its snapshot per commit)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (the fuzzy suites' CPU, and comparisons that name the difference)
- **Size:** M
- **Status:** open
- **Paid when:** a resumed cut is compared with a fresh run as typed state, not as a `Debug` dump; the core world's observer updates from each commit's rows; a story's cuts share one baseline run.

## What

- `replay::outside` (`replay.rs`, line 190) formats systems, peers, store, referee and trace with `{:?}`, and both snapshot tests compare those strings (`snapshot.rs`, lines 16 and 22);
- the core world's store view collects every stored row on each call (`store_view`, `tests/core/src/conformance.rs`, line 317), and `durable` calls it on every commit (line 245);
- `ops`'s six alert cut tests each run the whole baseline story again before cutting it (`alert_at`, `alert_cuts.rs`).

## Why it matters

The cost grows with the square of a story's length, and a `Debug` mismatch says only that
two long strings differ.

## What pays it back

Typed state with an equality that reports the first difference; the observer fed each
commit's rows; the baseline computed once per binary (a `OnceLock`) and shared by the cuts.
