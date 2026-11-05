# D049: FileIo's readers use thin forms and its private machines keep flags

- **Kind:** complexity
- **Where:** `skein-io/src/file_layer.rs`, `skein-io/src/file_layer/store.rs`, `skein-io/src/file_layer/private.rs`, every tree's readers
- **Seen:** 2026-11-05 at `d4b22abddf`, in the task io-file-limits of the plan debt-20, read at that commit
- **Checked:** 2026-11-05 at `d4b22abddf`
- **Value:** low
- **Size:** M
- **Status:** open
- **Paid when:** no reader calls `file_layer::{down, down_until, up, expire}` or `FileIo::{new, with_whole_limit}`, those thin forms are gone, and `Store` and `Opening` follow `skein/docs/foundation/programming-model.md`, 5.4.
- **Measure:** `rg --type rust 'file_layer::(down|down_until|up|expire)\b' skein smith jig temper hearth | wc -l`: 115 after the file limits migration

## What

`FileIo` takes `Env<Limits>` through `file_layer::step`, but readers use
its thin entry points and constructors. `Store` and `Opening` keep phases
in flags and `Option`s beside their state.

## Why it matters

Moving the readers crosses every tree, some readers keep no clock, and
`hearth-service-builder/src/lib.rs` is being migrated apart. Keeping the
thin forms lets those readers move together when their owners are ready
(`skein/docs/design/io.md`, 5; `skein/docs/design/io.status.md`).

## What pays it back

Move the readers to `FileIo::with_limits` and `file_layer::step`, then
remove the thin forms. Give each lifecycle in `Store` and `Opening` an
explicit data-carrying state, following programming-model.md, 5.4.
