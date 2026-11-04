+++
id = "kv-phase-ready"
part = "kv-review"
kind = "feature"
needs = ["kv-snapshot-file"]
files = ["skein/crates/skein-kv/src/store.rs"]
effort = "medium"
minutes = 25
+++

# skein-kv: one `Phase::ready`, and `Phase` on one line each

## Goal

The review's Part A, findings 3 and 4. `begin_batch`, `close_requested`, `is_ready` and `fire` each
list every phase but `Ready`; one exhaustive `Phase::ready` replaces the four. The `#[expect]` on
`ReadingLog`'s `bytes` makes rustfmt break every variant of `Phase`; on `enum Phase` it does not. The
cut tried that move with rustfmt on a copy: `store.rs` falls from 1,302 to 1,259 lines. Behaviour does
not change.

Acceptance:

- `rg -c '\| (Phase|Self)::ClosingSnapshot' skein/crates/skein-kv/src/store.rs` reads 3: `next_number`,
  `available` and `Phase::ready`. Those three stay exhaustive, with no `_`.
- `rg -c '^    [A-Z][A-Za-z]+ \{$' skein/crates/skein-kv/src/store.rs` reads nothing.
- The note's `What a task runs` passes with no test changed.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/skein-kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4.

## Owns and may change

- Owns: `skein-kv/src/store.rs`.
- The review offers `bytes` as a `Box<[u8]>` instead. **Do not:** the cut read it, and a boxed `bytes`
  keeps the old suffix beside the joined one, past what `skein/tests/kv/tests/suite/recovery.rs` bounds
  and `kv.status.md`, 3, 7, 9 records. The expect goes on `enum Phase`, and those on `replay`,
  `log_opened` and `log_read` stay.

## Steps

1. `Phase::ready`:

   ```rust
   impl Phase {
       /// The open log, when no file request is outstanding and none must be.
       fn ready(&self) -> Option<Log>;   // Ready gives its log; every other phase is listed, None
   }
   ```

   Each of the four matches on it. `begin_batch`, for one:

   ```rust
   match phase.ready() {
       Some(log) => store.start_batch(dir, log, &step.env.limits, step.below),
       None => dir.at(phase),
   }
   ```

   `close_requested` keeps its two cases under `Some`. `fire` keeps its `state @ (Closed | Memory)` arm.
2. Move the `#[expect]` onto `enum Phase`, above its `#[derive]`, with its reason; `cargo fmt` folds the
   variants.
3. What the note's `What a task runs` names; the citations check. One commit for each finding. Push.

## Report

The note's line counts. No test changed, so no hand-break.
