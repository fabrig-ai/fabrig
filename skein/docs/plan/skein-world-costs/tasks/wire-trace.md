+++
id = "wire-trace"
part = "wire-trace"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-world/src/stream.rs",
  "skein/testing/skein-world/src/domain/trace.rs",
  "skein/tests/world/tests/suite/wire_trace.rs",
  "skein/tests/llm-connection/src/world.rs",
  "skein/tests/llm-connection/tests/suite/protocol.rs",
  "skein/tests/llm-connection/tests/suite/fake_protocol.rs",
  "skein/tests/llm-connection/tests/suite/failures.rs",
  "skein/tests/oauth-accounts/src/streams.rs",
  "skein/tests/oauth-accounts/src/world.rs",
  "skein/tests/oauth-accounts/src/sign_in/protocol.rs",
  "skein/tests/oauth-accounts/src/sign_out.rs",
  "skein/tests/oauth-accounts/src/shared.rs",
  "skein/tests/oauth-accounts/src/accounts.rs",
  "hearth/tests/config/src/world.rs",
  "hearth/tests/mmdebstrap/src/world.rs",
  "skein/docs/plan/skein-world-costs/figures/wire-trace.md",
]
effort = "medium"
minutes = 75
+++

# `Wire`'s trace, read only when something reads it

## Goal

`Wire::take` and `Wire::answer` push a formatted `String` at every step, including `format!("fragment {bytes:?}")` of the raw bytes: 13.67% of 153M instructions, over 1,292 calls, in `client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store` of `jig-client-protocol-world`. Keep what a failure needs in a cheap form, and format it when something reads it.

Acceptance:

- `take` and `answer` format nothing: under callgrind, `alloc::fmt::format` is no longer called from either, in that test.
- `Wire`'s public `trace` field has the type `skein_world::stream::Trace`, below.
- `len()` counts what `Vec<String>::len()` counted, step for step, as worlds read it for progress.
- `lines()` gives the same strings the `Vec<String>` held, and `{:?}` of a `Trace` prints what `{:?}` of that `Vec<String>` printed, so what a failure prints is as useful as it is today.
- A seeded wire makes the same `Rng` draws in the same order, so every pinned seed's course stays the same.
- The new tests in `skein/tests/world/tests/suite/wire_trace.rs` pass, and each bites.
- `figures/wire-trace.md` holds that one test's reading before and after, as `skein/docs/plan/skein-world-costs/notes/measuring.md` says, with `Wire::take`, `Wire::answer` and `alloc::fmt::format` under each.

## Inputs

- `skein/docs/plan/skein-world-costs/inputs/request.md`, item 2.
- `skein/docs/foundation/testing-strategy.md`, 2.5 and 8.
- `skein/testing/skein-heap/src/lib.rs`, its "A step's own" section: why `take` must still free a `Send`.
- `skein/docs/plan/skein-world-costs/notes/common.md`, `skein/docs/plan/skein-world-costs/notes/measuring.md`.

## Owns and may change

Owns `skein/testing/skein-world/src/stream.rs`, `skein/tests/world/tests/suite/wire_trace.rs` and the figures.

Neighbours, each for the edit named and no more:

| File | The edit |
|---|---|
| `skein/testing/skein-world/src/domain/trace.rs` | `assert_replays` takes any comparable trace, below, and its doc says so; `domain::Trace` stays as it is |
| `skein/tests/world/tests/suite/main.rs` | `mod wire_trace;` |
| `skein/tests/llm-connection/src/world.rs` | `pub trace: Trace`, its `Trace::new()`, and `run`'s return type |
| `skein/tests/llm-connection/tests/suite/protocol.rs`, `fake_protocol.rs` | `run`'s return type |
| `skein/tests/llm-connection/tests/suite/failures.rs` | `report`'s return type |
| `skein/tests/oauth-accounts/src/streams.rs` | `drive`'s return type, its `Trace::new()` |
| `skein/tests/oauth-accounts/src/world.rs`, `sign_out.rs`, `shared.rs`, `accounts.rs` | the return types of `protocol` and `protocol_client` |
| `skein/tests/oauth-accounts/src/sign_in/protocol.rs` | `protocol`'s return type, and `close`'s `trace: &mut Trace` |
| `hearth/tests/config/src/world.rs`, `hearth/tests/mmdebstrap/src/world.rs` | the comment above `wire.trace.clear()` drops the word "formatted"; the calls stay |

These read `trace` and need no edit, because `Trace` keeps `Vec`'s names: `skein/testing/skein-forge-kit/src/hooks.rs` and `skein/tests/forge/machine/src/hand.rs` (`len`, `clear`, as progress), `jig/testing/jig-client-protocol-kit/src/net.rs` and `jig/testing/jig-engine-clients-kit/src/peer.rs` (`len`, as progress). `wire.trace.push(format!("owner {event:?}"))` in `skein/tests/llm-connection` stays as it is, and so does that world's own `events`: they're the test's fixture, and another plan's.

Options, each decided:

- **Other readers:** a line the compiler stops at, in any tree, takes the same edit: a type path, or `.lines()` where it reads text. Name each in your report.
- **`answer`'s own cost:** `self.pending.drain(..count).collect()` is a loop over bytes in an unoptimized build, and `write`'s `extend` may be too. If the profile shows them, take the bytes through the slices (`VecDeque::as_slices`) or straight into the trace's store. The same bytes reach the same places, and the same draws are made.
- **`clear`:** frees what it holds, capacity included. `hand.rs` clears so that a scripted peer's transcript isn't charged to the component's heap, and today's `clear` frees every line. Leave it so rather than keeping a store as large as the largest answer.
- **The private form of a step** is yours, provided that a step holds no `Box` per step.

## Steps

### 1. Before anything changes

Read the one test on your base, as `notes/measuring.md` says, and write it to `figures/wire-trace.md`.

### 2. The shape

The public surface binds. `Trace` keeps `Vec<String>`'s method names, so that readers here and in plans in flight compile unchanged where they only count, clear, push or append.

```rust
/// What a wire did, a step to a line. A step keeps what its line needs, and
/// the line is formatted when something reads it: a test that passes prints
/// nothing, and a replay compares the steps.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Trace {
    steps: Vec<Step>,
    /// The bytes of the steps that carry some, one after another.
    bytes: Vec<u8>,
}

impl Trace {
    pub fn new() -> Trace;
    /// The steps: each take, fragment, delayed room, answer and pushed line.
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    /// A line of the world's own, among the wire's steps.
    pub fn push(&mut self, line: String);
    /// Moves `other`'s steps after these, leaving it empty.
    pub fn append(&mut self, other: &mut Trace);
    /// Forgets every step, and frees what they held.
    pub fn clear(&mut self);
    /// Each step's line, formatted now.
    pub fn lines(&self) -> Vec<String>;
}

/// Prints as the list of its lines.
impl Debug for Trace { /* f.debug_list().entries(self.lines()).finish() */ }

/// A step, private to the module. Those with bytes hold how many, in
/// `Trace::bytes`, so that `append` is two `Vec::append`s.
#[derive(Clone, PartialEq, Eq)]
enum Step {
    Demand { read: Read, room: u32 }, // down Demand { read: .., room: .. }
    Send(usize),                      // down Send([..])
    Finish,                           // down Finish
    Fragment(usize),                  // fragment [..]
    RoomDelayed,                      // room delayed
    Bytes(usize),                     // up Bytes([..])
    Room,                             // up Room
    End,                              // up End
    Line(String),                     // the line pushed
}
```

In `Wire`:

- `pub trace: Trace`, still a public field, and `Wire` still derives `Debug`. The module's doc says what it keeps.
- `take` records the step from `&down` first, as it does today, then runs its `match` as it does. **A `Send`'s bytes are copied into the store, and its box is freed where it is today:** a `Meter` counts that free as handed out (`skein-heap`, "A step's own"). A trace that kept the box would charge what a step handed out to the step.
- `answer` pushes the same steps at the same places. Its `Rng` draws don't move.

In `skein/testing/skein-world/src/domain/trace.rs`:

```rust
pub fn assert_replays<R: PartialEq, T: PartialEq + Debug, F: Fn(u64) -> (R, T)>(
    seed: u64,
    other: u64,
    run: F,
) -> R
```

No caller names its type parameters, so its other callers compile unchanged. A replay compares steps and doesn't format them.

### 3. The tests

`skein/tests/world/tests/suite/wire_trace.rs`, whose binary declares the counting allocator:

```rust
use skein_heap::Meter;
use skein_lib::stream::{Down, Read, Up};
use skein_world::stream::Wire;

#[test]
fn a_wire_prints_each_step_as_it_went() {
    // One seed, pinned, that shows a delayed room. A read met in fragments,
    // room granted, a send, a finish, the end, and a line of the world's own.
    // Asserts `wire.trace.lines()` against the whole list, literally:
    // "down Demand { read: Fill(3), room: 0 }", "fragment [97, 98]", ...,
    // "up Bytes([97, 98, 99])", "down Demand { read: Nothing, room: 4 }",
    // "room delayed", "up Room", "down Send([1, 2])", "down Finish",
    // "down Demand { read: Arrived { max: 1 }, room: 0 }", "up End",
    // "owner closed".
    // And `format!("{:?}", wire.trace)` == `format!("{:?}", wire.trace.lines())`.
}

#[test]
fn a_trace_counts_compares_and_appends_its_steps() {
    // `len` is `lines().len()` after every step above. Two wires of one seed
    // given the same bytes compare equal; given bytes that differ in one value
    // at the same lengths, unequal. `append` leaves its argument empty, and
    // its lines follow in order. `clear` leaves `is_empty`.
}

#[test]
fn a_cleared_trace_holds_no_heap() {
    // A wire, then a `Meter`: room demanded and granted, a `Send`, then
    // `wire.trace.clear()`. `meter.held()` is `wire.received.capacity()`, no more.
}
```

For each, break its rule once by hand and report what failed: the text of one step; `len` skipping `room delayed`; a comparison that ignores the store; `clear` keeping capacity.

### 4. After

Read the test again, and fill in the figures' after arm.

### 5. Checks

Once, in order:

- `cargo fmt`
- `python3 -I -B skein/scripts/old_forms.py`, `python3 -I -B hearth/scripts/old_forms.py`
- `heavy cargo clippy -p skein-world -p skein-world-tests -p skein-llm-connection-world -p skein-oauth-accounts-world -p hearth-config-world -p hearth-mmdebstrap-world --all-targets -- -D warnings`
- `heavy cargo nextest run -p skein-world-tests -p skein-llm-connection-world -p skein-oauth-accounts-world -p skein-forge-machine-world -p skein-forge-client-world -p skein-forge-protocol-world -p hearth-config-world -p hearth-mmdebstrap-world -p jig-client-protocol-world -p jig-engine-clients-world`. These are the readers of `len` as progress, of a replayed trace and of a wire's heap: **a change of meaning here would be theirs to catch.**
- The rest compile unchanged, which this holds: `heavy cargo check --tests -p skein-forge-kit -p skein-forge-connection -p skein-echo-world -p jig-client-protocol-kit -p jig-engine-clients-kit -p jig-client-shell-native -p jig-test-service -p jig-fleet-world -p jig-host-world -p jig-notes-world -p jig-people-world -p hearth-test-git -p hearth-builder-world -p smith-agent-process-world -p smith-agent-world -p smith-channel-world -p smith-hosts-world -p smith-host-world -p smith-local-process-world -p smith-local-world -p smith-machine-world -p smith-run-world -p smith-session-world -p smith-tools-world -p temper-service -p temper-worker-checkout-world`

A memory test's failure here means the trace moved a charge: read it against "`take` frees a `Send`" and `clear` above before you touch its figure. No ceiling moves. Delete `target/world-costs/`, commit and push.

## Report

The test's total before and after, with `format`'s share; each reader you changed beyond the table; what you did about `answer`'s own cost, with the numbers; each rule you broke by hand, and the failure seen.
