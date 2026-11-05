# D032: tests/io runs its worlds on hand loops, beside `skein_world` and io's stage

- **Kind:** tests
- **Where:** `skein/tests/io/src/world.rs`, `skein/tests/io/src/output.rs`, `skein/tests/io/src/joined.rs`, `skein/testing/skein-world/src/world.rs`, `skein-io/src/stage.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, copies between worlds) and `reviews/2026-10-31-skein-areas.md` (A7, A12)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-05 at `654928ff4a`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** tests/io's socket, output and either worlds run on `skein_world` and io's stage, keeping the per-call `MAX_OUT` check and the ledger.
- **Measure:** `rg -c 'advance_to' skein/tests/io/src | awk -F: '{s+=$2} END{print s}'`: 3 at base

## What

`world.rs` over `Sim`, `output.rs`'s conversation and `joined.rs`'s `either` each run their own loop; nine `impl Host` modules
already run on `skein_world`. Of those, only `append.rs` and `usage.rs` use `skein_io::stage`.

## Why it matters

A service runs on the stage, so these worlds hold its machines to a drive no service uses.

## What pays it back

Two steps, and the second is a decision, not a refactor.

- **The loop:** `skein_world` needs a `Referee::idle` that can change the referee at idle, and the worlds a shared directory
  that doesn't delay a dial by an iteration. By reading it keeps each schedule; a per-seed digest of the trace and logs, before and after,
  proves it. About 30 lines out.
- **The stage:** not the same schedule by construction. These worlds fire deadlines with completions waiting, and `stage.rs`
  returns first; it has no per-stage resume bound here; and it has no place for the per-call `MAX_OUT` check or for the ledger's
  `request` on each request taken, since `Request` isn't `Clone`. It needs a per-entry witness in `skein_io::stage`, a step crate,
  or a panic path in shipped code.
