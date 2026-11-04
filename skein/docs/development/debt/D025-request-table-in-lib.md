# D025: lib's `RequestTable` is a component with one consumer

- **Kind:** coupling
- **Where:** `skein-lib/src/requests.rs`, `skein/tests/lib/src/request_table.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (themes T4 and phase 4, the `RequestTable` row; A13) and `reviews/2026-10-31-skein-areas.md` (A13)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** `RequestTable` lives with its consumer or in a component crate of its own, not in skein-lib, and its state follows `skein/docs/foundation/programming-model.md`, 5.4; or lib.md says why lib keeps it.

## What

`skein-lib/src/requests.rs` (734 lines, with its world in
`skein/tests/lib/src/request_table.rs`) is used outside skein only by
`jig-client-domain` (`jig/crates/jig-client-domain/src/domain.rs`). It
keeps flags beside its `state`. `skein/docs/design/lib.md`, 12, designs
it.

## Why it matters

lib holds the generic containers every crate builds on; a component
with one user makes lib larger to read and its tests slower, and its
flags are the shape D018 to D024 remove.

## What pays it back

Decide its home first (`skein/docs/design/lib.md`, 12, places it in lib, so moving it is a
design change), then bring it to 5.4.
