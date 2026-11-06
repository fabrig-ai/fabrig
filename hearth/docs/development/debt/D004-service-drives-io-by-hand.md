# D004: The service drives io and files by hand, doubling every operation token

- **Kind:** primitives
- **Where:** `hearth-service-builder/src/lib.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, new, met while checking D001
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** medium (one less private routing rule; the shared stage's ordering)
- **Size:** S
- **Status:** planned (hearth-memory-6)
- **Paid when:** `hearth-service-builder` routes completions with `file_layer::owns`, calls `skein_io::stage::{up, down}`, and has no `tag` and no `io::resume`.
- **Measure:** `git grep -E '\bfn tag\b|io::resume' -- hearth/crates/hearth-service-builder/src/lib.rs | wc -l`: 2

## What

- `io_up` and `io_down` write out the order of
  `skein/crates/skein-io/src/stage.rs` by hand: ready work, completions,
  deadlines, each behind `io_room`.
- `tag` multiplies each submission's token by two and adds a file bit, for
  `Cancel`'s target too, and `io_up` shifts it back. skein already tells the
  two apart: `FileIo` mints its tokens with `file_layer::LANE`, and
  `file_layer::owns(op)` reads it (`skein/docs/design/io.md`, 5).

## Why it matters

The private tag is a rule the service and its tests must keep alike, for a
fact the token already holds. A change of order in skein's stage doesn't reach hearth, and hearth's
hand-kept order isn't the one `skein/crates/skein-io/src/stage.rs` tests.

## What pays it back

Take off `tag` and split completions with `file_layer::owns`, as
`skein/testing/skein-world/src/files.rs` does, then call the stage for
`Io`'s and `file_layer::up` for the rest. `Io`'s `MAX_OUT_*` then sit inside
the stage alone. Do it with D001, which rewrites the rest of `iterate`.
A second `Io` on one kernel is the service's to tell apart (`skein/docs/design/io.md`, 5),
so it doesn't block this.
