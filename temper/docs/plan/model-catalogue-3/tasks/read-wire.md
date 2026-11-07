+++
id = "read-wire"
part = "worker-read"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-link/src/lib.rs", "jig/crates/jig-protocol-worker-wire/schema/**", "jig/crates/jig-protocol-worker-wire/src/**", "jig/crates/jig-protocol-worker-wire/golden/**", "jig/crates/jig-protocol-engine-workers/src/translate.rs", "jig/crates/jig-protocol-engine-workers/src/component.rs", "jig/crates/jig-protocol-worker-link/src/translate.rs", "jig/crates/jig-protocol-worker-link/src/component.rs", "jig/crates/jig-domain-hub-link/src/down.rs", "jig/crates/jig-domain-hub-link/src/payload.rs", "jig/crates/jig-domain-host/src/link.rs", "jig/crates/jig-domain-host/src/boundary.rs", "jig/crates/jig-domain-host/src/domain.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/routing/children.rs", "jig/crates/jig-domain-fleet/src/boundary.rs", "jig/crates/jig-domain-fleet/src/domain.rs", "jig/docs/design/protocol/channel.status.md"]
effort = "high"
minutes = 110
+++

# A read crosses the worker's channel

## Goal

The surface a read of a list crosses, from the core's `Held` to the host's boundary and back, with every exhaustive match taking its arm and the two wire records encoded. It is inert: nothing sends one. `fleet-reads`, `host-read` and `core-reads` build on it, so it stays small.

Acceptance:

- `contracts.md`, 5's types, variants and kinds exist with those names.
- `read list` and `list read` round-trip through the worker wire's records with their goldens at smallest and full, at the bounds of `jig/docs/design/protocol/channel.md`, 5.1 and 5.2. A row past 256, or a name past 128, is refused as the schema refuses.
- The engine's side fills `read list`'s grant from the accounts' table, as it does `grant`'s, and the worker's side takes the value out into the worker's table, the hub hearing `Grant` alone: a row of the existing referee at `jig/tests/channel/tests/suite/referee.rs` or its unit neighbour holds a secret's bytes out of everything the hub sees.
- A form other than a credential, or a generation the table no longer holds, isn't sent: the engine's side answers it upward as `ListRead { Failed(Agent) }` at once (`contracts.md`, 5). A table test row holds each.
- `LinkDown::ReadList` reaching the host answers `LinkUp::ListRead { Failed(Agent) }` at once, since `host-read` hasn't given it a machine; the fleet's `Event::ReadList` does nothing yet and its `Event::ListRead` is dropped.
- `heavy cargo check --workspace --tests` compiles; the focused tests of every package here pass.
- `channel.status.md`'s 5.1, 5.2 and 8 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 5.
- `jig/docs/design/protocol/channel.md`, 5.1, 5.2 and section 8; `jig/docs/design/domain/hosts.md`, 5.1.

## Owns and may change

Owns the header's files. The survey's list of what a sibling of `check` touches is your checklist:

- **The wire:** `schema/worker-wire-v9.schema:166-177` beside `Check` and `Logins`, in place; `src/kinds.rs:7,55,64,74,90`; generated `src/generated/v9.rs`, by its tool; `src/sizes.rs:58,128,162,184`; `src/lib.rs:10,18,50`.
- **The engine's side:** `jig-protocol-engine-workers/src/component.rs:136,161`; `translate.rs:70,91,127,143,494,510,526,567,705,757`; `host.rs:317`, the grants' enumeration.
- **The worker's side:** `jig-protocol-worker-link/src/translate.rs:90,527,591,616,634,635,771,795`; `component.rs:333,350,553`. `limits.rs` where a frame's size needs its room.
- **The hub's link:** `jig-domain-hub-link/src/down.rs:15,76`; `payload.rs:64`.
- **The host:** `jig-domain-host/src/link.rs:9,90`; `boundary.rs:27,88,556`; `domain.rs:240,311`; `live.rs:64`.
- **The core, an arm alone:** `vocabulary.rs:330,1218`; `routing/children.rs:143,556,661,746`.
- **The fleet, an arm alone:** `boundary.rs:142,360`; `domain.rs:471,476`.
- **Harnesses, by the required-field exception, the arm alone:** `jig/testing/jig-test-worker/src/{admission.rs:22,137,169,boundary.rs:10,29,domain.rs:144,297}`, `jig/testing/jig-test-service/src/{worker.rs:367,561,workers.rs:129,route.rs:292}`, `jig/testing/jig-test-domain/src/routes.rs:95`, `jig/testing/jig-core-kit/src/world/deliveries.rs:234`, `jig/testing/jig-conformance/src/core.rs:916`, `jig/testing/jig-client-kit/src/joint/engine.rs:216`, `jig/testing/jig-protocol-accounts-kit/src/owner.rs:664`; `temper/crates/temper-service/src/{worker.rs,worker_route.rs,route.rs}` and `temper-engine-domain`'s route, the reference root's way.

Options, each pre-approved:

- The worker's table holds at most two generations; a read's key gets its entry there as a run's does. The sweep `jig/docs/design/protocol/channel.md`, section 8 says ends it is `host-read`'s, not yours.
- If `Held`'s channel is named differently from `contracts.md`'s `channel`, take the file's own word.
- If a wire record's size pushes the channel's largest frame up, raise the derived frame bound where `sizes.rs` derives it and name it in the report: it's a derived limit, not a budget.
- Nothing about `Event::ListRead` needs a slot or an attempt: no hello field is added.

## Steps

```text
# worker-wire-v9.schema
record ReadList { read: u64  endpoint: u32  grant: Grant }
record ListRow  { name: bytes 128  shown: option bytes 64  window: option u32  output: option u32  efforts: list 8 bytes 16 }
enum   ListFailed { refused: u16, unreachable, unreadable, deadline, agent }
enum   ListOutcome { listed: list 256 ListRow, failed: ListFailed }
record ListRead { read: u64  outcome: ListOutcome }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` naming every package you changed, `--all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-worker-wire -p jig-protocol-engine-workers -p jig-protocol-worker-link -p jig-domain-hub-link -p jig-domain-host -p jig-domain-fleet`; `heavy cargo nextest run -p jig-worker-wire-world -E 'test(/drift/)'`; `heavy cargo check -p temper-service -p temper-engine-domain --tests`. Commit and push.

## Report

As the implementer's text says.
