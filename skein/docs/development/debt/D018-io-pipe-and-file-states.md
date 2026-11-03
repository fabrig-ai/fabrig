# D018: io's pipe, signals and file machines keep state in flags and literals

- **Kind:** complexity
- **Where:** `skein-io/src/pipe.rs`, `skein-io/src/signals.rs`, `skein-io/src/file_layer.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the skein-io rows; A6) and `reviews/2026-10-31-skein-areas.md` (A6)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** `Pipe`, `Signals` and `FileIo` follow `skein/docs/foundation/programming-model.md`, 5.4, as io's stream and listener do (data-carrying states, `mem::replace` then follow); `FileIo` takes `Env` and `Limits` like its siblings and answers a failure from one constructor, not a literal written at each site.
- **Measure:** `rg -c 'committed: false, residue: None' skein/crates/skein-io/src | awk -F: '{s+=$2} END{print s}'`: 53 at base

## What

- `pipe.rs` keeps 8 `bool` and 9 `Option` fields; `signals.rs` 2 and 5.
- `file_layer.rs` (1,261 lines) writes the
  `Failed { .., committed: false, residue: None }` literal 53 times,
  and `FileIo` has no `Limits` or `Env` and uses `Vec` buffers under a
  module-wide expect.
- io's `stream.rs` and `listener.rs` are the tree's model of 5.4; these
  machines sit beside them in another shape.

## Why it matters

Fields that mean something in some states only, and a failure literal
whose 53 copies must agree. These machines hold every service's pipes
and files.

## What pays it back

One machine per task: the pipe and signals onto data-carrying states,
then `FileIo` onto `Env`/`Limits` with a failure constructor. The
operation-token question (D044) touches `FileIo` too.
