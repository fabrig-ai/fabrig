+++
id = "io-file-states"
part = "io-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/file.rs", "skein/crates/skein-io/src/file_layer.rs", "skein/crates/skein-io/src/file_layer/**", "skein/crates/skein-io/src/tests/file.rs", "skein/crates/skein-io/src/tests/store.rs", "skein/crates/skein-io/src/tests/private.rs", "skein/crates/skein-io/src/tests/directory.rs"]
effort = "high"
minutes = 75
+++

# skein: `FileIo` holds its request in a state, and answers a failure from one constructor

## Goal

Part of `skein/docs/development/debt/D018-io-pipe-and-file-states.md`. `FileIo`'s `pending`, `outstanding`, `deadline`, `cancel_op`, `cancel_target`, `stop` and `loading` become states (`skein/docs/foundation/programming-model.md`, 5.4), and `Failed { committed: false, residue: None, .. }` comes from one constructor. Its events and submissions stay the same.

```rust
enum Work { Idle, Active { pending: Pending, op: Token, deadline: Option<Time>, stop: Stop } }
enum Stop { Running, Deadline, Cancel }
enum Cancelling { Quiet, Asked { op: Token, target: Token } }  // a cancel can outlive its request: `takes` reads both
// skein-io/src/file.rs, on `impl Event`
pub(crate) fn failed(owner: Token, error: Error) -> Event;
```

`loading` moves into `LoadOpening`, `LoadReopening` and `LoadStating`. The 44 `io.pending =` sites become one `issue(io, pending, op, subs) -> Work`.

Acceptance:

- The item's **Measure** (`rg -c 'committed: false, residue: None' skein/crates/skein-io/src | awk -F: '{s+=$2} END{print s}'`) reads 1, the constructor's own line. It reads 53 at base, 32 of them production. Also convert the wrapped sites it misses (two in `file_layer.rs`, one in `tests/store.rs`).
- `rg -n '\bio\.(pending|outstanding|cancel_op|cancel_target|stop|loading)\b|match \(pending|\(_, _, _\)' skein/crates/skein-io/src/file_layer.rs skein/crates/skein-io/src/file_layer` finds nothing that belongs to `FileIo` (your report names any line of `store.rs`'s or `private.rs`'s own machines it lists).
- A new test destructures `Event::failed(..)` and asserts `committed` and `residue`, since the other tests stop spelling those fields.
- `heavy cargo nextest run -p skein-io --lib`, `-p skein-io-world --test suite` and `-p skein-kv-tests` pass, with what they assert unchanged but the spelling of that event.
- By hand: send a stopped `Rename` to cleanup (`a_rename_or_remove_answered_after_its_deadline_keeps_its_own_answer` goes red); retry a cancel without checking its target (`a_late_unsubmitted_store_cancel_never_cancels_the_cleanup_close`); make `failed` commit (the new test). Restore each.
- The net production lines are at most +70 (estimate: +40). Compact first.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `skein/docs/development/debt/D018-io-pipe-and-file-states.md`
- `skein/docs/foundation/programming-model.md`, 5.4 (it forbids the `(_, _, _)` arms too)
- `skein/docs/design/io.md`, 5, 5.2 and 5.3

## Owns and may change

- **Owns:** the files in `files`. The item stays: `io-file-limits` deletes it. `io-file-limits` follows you in these files.
- **Readers:** none outside `skein-io`: the public API doesn't change.
- **Traps:**
  - Don't reuse `terminal_failure` for the constructor. It turns `Cancelled` into `Event::Cancelled`, while one site passes a kernel `Cancelled` as `Failed`.
  - `private.rs`'s two `committed: true` literals and `store.rs`'s that carries a real fact stay literals.
  - `tests/file.rs`'s `Debug` pin (a length, no canary) holds as long as `Contents` stays.
- **`store::Store` and `private::Opening` stay as they are:** they keep phases in flags and `Option`s too, but the item names `FileIo`. `io-file-limits` records that departure.
- **27 files in other crates write `committed: false`:** that's their own record, and they stay.
- **The `Vec` buffers and the module's `#![expect]`** stay: the item's **Paid when** doesn't ask for them, and `List` has no `insert` or `pop`.
- **The constructor is `pub(crate)`,** so `skein-io/src/tests/*` use it; `skein/tests/io/**` keeps its own spelling.

## Steps

1. `Event::failed` with its test; the literals, production then tests. Commit.
2. `Work`, `Stop`, `Cancelling`, `issue`; `Pending`'s load phases; the matches' arms listed. Commit.
3. Break the rules by hand, as the acceptance says.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; the tests above; `heavy cargo check -p smith-local-service -p jig-protocol-store --tests`. The citations check. Commit and push.

## Report

The common note's items; what you broke and what failed; any line the second search lists.
