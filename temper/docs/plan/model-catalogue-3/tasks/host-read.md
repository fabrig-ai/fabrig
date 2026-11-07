+++
id = "host-read"
part = "worker-read"
kind = "feature"
needs = ["read-wire", "host-list"]
files = ["jig/crates/jig-domain-host/src/sources.rs", "jig/crates/jig-domain-host/src/domain.rs", "jig/crates/jig-domain-host/src/boundary.rs", "jig/crates/jig-protocol-worker-agents/src/sources.rs", "jig/crates/jig-protocol-worker-agents/src/route.rs", "jig/crates/jig-protocol-worker-link/src/component.rs", "jig/testing/jig-test-worker/src/**", "jig/testing/jig-test-service/src/worker.rs", "jig/testing/jig-channel-kit/src/**", "jig/tests/host/tests/suite/sources.rs", "jig/tests/channel/tests/suite/list_read.rs", "jig/tests/channel/tests/suite/main.rs", "jig/docs/design/domain/hosts.status.md"]
effort = "high"
minutes = 130
+++

# A worker reads a list

## Goal

The worker's half of `jig/docs/design/domain/hosts.md`, 5.1: `LinkDown::ReadList` takes the hub's source queue, the protocol layer runs smith's `ListRead` with the worker's table's value, and `LinkUp::ListRead` goes up. This is the first story to run both real sides: a jig worker, a real agent's `--list`, and a fake provider.

Acceptance:

- In `jig/tests/host/tests/suite/sources.rs`, rows of the existing table: a read runs while every slot is taken and takes none; a read waits behind a check and a sign-out, serialized as they are; a check waits behind a read; a link lost cancels a read and sends nothing up (`Request::CancelRead`); the hub hears `Grant` alone.
- A story in `jig/tests/channel/tests/suite/list_read.rs`, on `jig-channel-kit`'s real worker and agent over `skein-fake-peers`' fake: an engine's `read list` gives `list read` with the rows the fake's list said; a fake's status answers `Failed(Refused { status })`; and `Observation::List` shows exactly one request, with that key, on the endpoint's route. Each is a row.
- A key taken for a read leaves the worker's table when the read ends, unless an attempt holds it (`jig/docs/design/protocol/channel.md`, section 8): a row checks the table's count after. A bearer stays until it expires, as `channel.md` says.
- The referee at `jig/tests/channel/tests/suite/referee.rs` still holds: no value reaches a hub or a root.
- `hosts.status.md`'s 5.1 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 4 and 5.
- `jig/docs/design/domain/hosts.md`, 5.1 and 6.8; `jig/docs/design/protocol/channel.md`, 5.1, 5.2 and section 8; `smith/docs/design/protocol/hosts.md`, section 2.

## Owns and may change

Owns the header's files.

- `jig-domain-host/src/sources.rs:8,13,27,35,130,193,333` is the machine. A read is a queue of one, with `Busy` naming what runs. A second read arriving while one is queued answers `Failed(Agent)` at once, since the fleet sends one at a time and a reconnect cancels first.
- `jig-protocol-worker-agents/src/sources.rs:20,70,82,119,223,268`: `Resources` gains the agent's launch (`ListLaunch`'s fields) and `Component` a `ListRead` beside its `Command`, taken by the same `owns`, `from_io`, `fire`. Its deadline is the host's check deadline, `limits.hub.source_timeout` or whatever `Request::Source` carries. `Resources::new` takes the launch as an `Option`: none, and a read answers `Failed(Agent)`.
- `jig-test-worker`'s `domain.rs:144,297` and `boundary.rs:10,29`, `jig-test-service/src/worker.rs:368,561`, `values.rs:29`, `workers.rs:129`: the reference root's route, and its launch `smith-agent --list CONFIG.json` from its agent's own `program` and configuration.
- **temper, by the required-field exception, an arm or argument only:** `temper-service/src/program_sources.rs`, `worker.rs`, `worker_route.rs`. Temper passes no launch here, so its reads answer `Failed(Agent)` until `temper-worker`.

Options, each pre-approved:

- The value is read from the worker's table at the moment `ListRead::new` is called, as `process_send`'s `grant_value` does at `temper-service/src/worker.rs:698`. A missing or lapsed entry answers `Failed(Agent)` with no spawn.
- If a slot-free spawn needs an owner range, take it as `Request::Source`'s does at `jig-test-service/src/worker.rs:370`.
- `jig-channel-kit` may gain a helper to build an agent's `--list` launch.

## Steps

```rust
// jig-domain-host/src/sources.rs
pub enum Operation { Check, SignOut }             // unchanged
// beside them
struct Reading { read: u64, endpoint: u32, grant: Grant }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-host -p jig-protocol-worker-agents -p jig-protocol-worker-link -p jig-test-worker -p jig-test-service -p jig-channel-kit -p jig-host-world -p jig-channel-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-host -p jig-protocol-worker-agents -p jig-protocol-worker-link -p jig-host-world`; `heavy cargo nextest run -p jig-channel-world -E 'test(/list_read|referee|program_sources/)'`; `heavy cargo check -p temper-service --tests`. Commit and push.

## Report

As the implementer's text says.
