+++
id = "fleet-reads"
part = "reads"
kind = "feature"
needs = ["read-wire"]
files = ["jig/crates/jig-domain-fleet/src/reads.rs", "jig/crates/jig-domain-fleet/src/domain.rs", "jig/crates/jig-domain-fleet/src/lib.rs", "jig/crates/jig-domain-fleet/src/grants.rs", "jig/crates/jig-domain-fleet/src/limits.rs", "jig/crates/jig-domain-fleet/src/channel.rs", "jig/tests/fleet/tests/suite/reads.rs", "jig/tests/fleet/tests/suite/main.rs"]
effort = "high"
minutes = 110
+++

# The fleet holds and places a read

## Goal

The fleet's half of `jig/docs/design/domain/engine.md`, 17.12's *Placing a read*: it holds each read until a connected worker can take it, sends it, keeps it through a lost channel, and hands up the answer. It takes no slot and makes no attempt.

Acceptance:

- `Event::ReadList` queues a read; `Request::ReadList` sends it to the connected worker, not draining, with `channel.sources.ready()`, whose declared bounds hold the endpoint (`EndpointBound`), with the most free slots, and with none of this fleet's reads in flight. A worker with none free is still taken: a read takes no slot.
- `Event::ListRead` from the channel holding that read gives `Request::ListRead { read, outcome }`, frees the worker and places the next. One from a channel not holding it is dropped.
- A channel lost while holding a read puts it back at the head of the queue, and no answer is said. A worker connecting, or a hello that adds an endpoint, places what's held.
- A renewal (`grants.rs:69`) updates a read still queued, which is then sent with the newest generation. An account removed while its read is held needs nothing here: its generation is gone from the table when it's sent, the engine's side answers `Failed(Agent)` (`read-wire`), and the accounts child drops an answer nobody holds. A read in flight isn't cancelled: it ends by the host's deadline.
- The queue's capacity is the accounts' bound, `Limits::reads`, sized at configuration; past it is a link that can't be made and panics, since an account has one read at a time.
- Stories in `jig/tests/fleet/tests/suite/reads.rs`, in the fleet's world, one table of rows: no worker connected, then one; the worker with most slots chosen; none holding the endpoint; two reads and one worker, in order; a lost channel and its re-send; a stale answer; a renewal while held.
- A read makes no attempt: `attempts`, `hosts`, `slots`, the fleet's deployment document and its capacity history are unmoved, a row checking them.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 5.
- `jig/docs/design/domain/engine.md`, 17.12.

## Owns and may change

Owns the header's files. `jig-domain-fleet/src/boundary.rs`'s variants are `read-wire`'s and don't change here.

`channel.rs:340-372`'s `probe` is the shape to copy: `channel.sources.ready()`, `channel.draining`, `channel.slots`. Reads are its own function, since `probe` filters by free slots.

Options, each pre-approved:

- `Limits::reads` is derived where the fleet's limits are built, from the accounts' bound, in `jig-protocol-config`, a line; every `Limits { .. }` literal takes it by the required-field exception.
- An engine-kind channel is never chosen: `HostKind::Worker` alone.
- A read in flight on a channel that hears `draining` stays there until it answers.
- The fleet's `next_deadline` doesn't move: a read has no deadline here. The host's command has one, and a lost channel is what the fleet sees.

## Steps

```rust
// jig-domain-fleet/src/reads.rs
//! A read of a provider's list, held until a worker can take it (domain/engine.md, 17.12).
//! Keeps the queue and which channel holds each. It never holds a value: a grant is a name.
pub(crate) struct Reads { .. }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-fleet -p jig-fleet-world -p jig-protocol-config --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-fleet -p jig-fleet-world -p jig-protocol-config`; `heavy cargo check -p jig-domain-core --tests`. Commit and push.

## Report

As the implementer's text says.
