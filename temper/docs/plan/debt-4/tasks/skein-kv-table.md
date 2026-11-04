+++
id = "skein-kv-table"
part = "skein-kv-states"
kind = "feature"
needs = ["skein-kv-admission"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/store/**"]
effort = "high"
minutes = 80
+++

# skein D017, the table: `State`, `Dir`, `Done`, and `up` by state, then event

## Goal

`up` matches `(phase, event)` tuples with a catch-all, and `root` and `close_owner` stand as
`Option`s beside `phase`. The store takes the note's outer shape: `State`, with `Dir` holding what
an open directory holds, and `up` a match on the state and in each arm on `Done`, with no
catch-all. Flags stay flags here, moved as they are into `Dir`; the next two tasks remove them.
The item stays open.

Acceptance:

- `Store` has the fields `state`, `repair_required` (the flag, until `skein-kv-recovery`), `last`,
  `map`, `queue`, `queued_bytes`, `admitted`; `Dir` holds `root` as an `Fd`, and every other field
  that means something only while open. `Phase` has no `Closed`.
- `rg -n '\(phase, event\)|\(_, _\)|match \(|if let Phase|let Phase::|if let State|let State::' skein/crates/skein-kv/src/store*`
  reads only `snapshot_up`'s tuple, which `skein-kv-compaction` takes.
- `root()`'s `expect` is gone, and so is every `self.root.is_some()`-style check.
- `#![expect(clippy::too_many_lines, ..)]` stays only if `fire` or `down` still needs it, and
  says so in your report.
- The note's `What a task runs` passes, with no test changed.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/kv-store.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 5.4, whole; `skein/docs/design/kv.md`, 2 to 5.
- The form: `smith-protocol-program/src/backend/process.rs`, a small machine and its handlers.

## Owns and may change

- Owns: `skein-kv/src/store.rs`, and `src/store/*.rs` if you split it.
- `down`, `up`, `fire`, `is_ready` and `Store`'s public methods keep their signatures.
- Pre-approved: `Phase::Ready`, `Writing` and `Syncing` keep `file` and `offset` here or take the
  note's `Log`, your choice; `active` stays on `Dir` until `skein-kv-subset`.

## Steps

1. Add `Done` and `done`, as the note sketches. `Failed`'s `error` is read only by `snapshot_up`.
2. Add `State` and `Dir`. `Store::memory` is `State::Memory`, and `recovered` stays a `Dir` flag
   for the other states. `down`'s `Open` matches on the state: `Closed` and `Memory` open, as at
   base; `Open` refuses. `Commit`, `Get`, `Load` and `Close` match on the state first.
3. Turn `up` into the note's match, one handler per live cell, taking `dir` and the source's data
   by value and returning `State`; a `Dir::at(self, phase: Phase) -> State` keeps those lines
   short. `Ready` in `up` is `unreachable!("no file request is outstanding")`, where the base's
   `Failed` arm for it was dead. Cancellation is not a request the store sends: `done`'s
   `unreachable!` covers `Cancelled`.
4. `open_failed` and `Closing`'s end return `State::Closed`. `snapshot_up` keeps its body and its
   tuple, with `dir` and the `Done` passed in.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

`store.rs`'s lines before and after, and each `#![expect]` left and why.
