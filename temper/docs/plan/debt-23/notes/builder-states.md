# debt-23: hearth's builder states (hearth D006)

Every `builder-*` task and `store-states` name this note. It comes from a read-only survey at this plan's base; nothing was run.

## What's binding

- **Public signatures stay still.** `hearth-service-builder/src/lib.rs` calls `Protocol::new`, `with_file_read`, `with_proxy`, `output_bound`, `bind_output`, `error_bound`, `bind_error`, `take_startup`, `code`, `is_empty`, `work_pending`, `takes`, `protocol::{down,up,resume,fire,next_deadline}`, `MAX_OUT_*`, and it isn't yours: fabrig/hearth is migrating it. `hearth-domain-builder` calls `Store::new`, `store::{step,fire,next_deadline}`, `store::MAX_OUT`, `store::vocabulary`. `hearth/tests/builder/src/protocol.rs` drives the same surface.
- **The memory forms stay as they are:** `MaxOut` and its constants in `hearth-protocol-builder/src/lib.rs`, `up`'s `!above.is_empty()` predicate, `hearth-domain-store`'s `MAX_OUT`, `client.rs`'s two `Queue::with_capacity` declarations, `revision_stdout`'s `stdout.room()`. D001's rounds will rewrite them. If this work needs one, stop and ask: that's fabrig/debt's. `scripts/old-forms.txt`'s `max_out` doesn't move.
- **Facts stay flags.** `Child.failed`, `Client.refused`, `Client.failed`, `PipeReader.ended`, `Read.failed`, `Read.ended` and `Store.staged` are what a lifecycle reports or carries, read after it, so **Paid when** doesn't ask for them. `Working`'s inner `Option`s, `Scan`'s flags, the protocol's `store.rs` `closing` and `keep.rs` are outside the item's **Where**.
- **`Held`** (`builder-child` creates it, in a new `hearth-protocol-builder/src/held.rs`; `builder-resolve` reads it):

  ```rust
  pub(crate) enum Held { Awaited, Open(Token), Closing(Token), Closed }
  // token(): Some in Open and Closing.
  // request_close(): Open to Closing, answering the token.
  ```

- **Heap:** fields moving into enums can't grow the inline size, and only a new `Box` adds heap. If `clippy::large_enum_variant` fires, a scoped `#[expect]` with a reason, not a `Box`. Ceilings are in `hearth/tests/builder/tests/suite/memory.rs` and `fuzzy/memory.rs`. One that grows stops and asks. No `size_of` pin exists here.
- **Private reads, all in fixtures, no assertion:** `protocol.resolving` (`tests/fixtures/resolve.rs`), `protocol.mode` (`tests/fixtures.rs`), `store.store.usable` (domain-store's `tests/fixtures.rs`). They change how they reach the code.

## Traps across tasks

- **A process closes by owner, a pipe by token.** `Spawned` gives `id.token()`, but `Closed` for the process carries `child.owner` (`skein-io/src/process.rs`). A shared "close if the token matches" stalls forever. Keep `owner == self.owner`, and `owner == OWNER` in `Resolve`.
- **`Closing(token)` still has a token.** `stream()` expects `output` and can run while closing, as `lib.rs`'s drain arm has no `!closing` guard. `client.owns` stays true while closing. `Awaited` counts as gone in `done()`, since three of the four commands have no stdin.
- **Close order is visible.** `Child` closes output, error, input, process, one per pass. `Resolve` closes output, error, process, then waits. `the_scratch_is_walked_away_before_the_terminal` drives that sequence.

## Tests

`heavy cargo nextest run -p hearth-protocol-builder --lib` and `-p hearth-builder-world`, unless a task names others.
