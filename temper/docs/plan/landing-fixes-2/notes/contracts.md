# landing-fixes-2: the shapes that cross a task

This run's binding calls. `refusal-reason` builds section 1, and `refusal-words` builds on it.

## 1. A refused hello's reason (jig, then temper)

In `jig/crates/jig-domain-fleet`, beside `HelloRefusal` in `src/history.rs`, exported from `lib.rs`:

```rust
/// What each side holds of an agent's shortfall (domain/engine.md, section 8):
/// in the bound's own unit, a duration in milliseconds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Figures {
    pub required: u64,
    pub offered: u64,
}

/// Why the fleet refuses a hello, for the refusal's text (protocol/channel.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HelloRefused {
    pub why: HelloRefusal,
    /// Some for an agent's shortfall that has amounts; None for every other reason.
    pub figures: Option<Figures>,
}
```

In `src/needs.rs`, beside `shortfall`, exported as `shortfall` is:

```rust
/// The amounts of a shortfall `shortfall` selected: None for `Model`, which
/// has none, or a charter or model those needs and bounds do not hold.
#[must_use]
pub fn figures(needs: &Needs, bounds: &Bounds, why: AgentShortfall) -> Option<Figures>
```

| `why` | `required` | `offered` |
|---|---|---|
| `Window` | the need's `window` | the declared model's `window` |
| `Output` | the need's `output` | the declared model's `output` |
| `Turns` | the charter's `turns` | `bounds.turns` |
| `Time` | the charter's `time`, ms | `bounds.time`, ms |
| `Waiting` | the charter's `waiting`, ms | `bounds.waiting`, ms |
| `Checks` | the charter's `change_reserve`, ms | `bounds.checks`, ms |
| `Spend` | `needs.spend` | `bounds.spend` |
| `Message` | `needs.message_bytes` | `bounds.message_bytes` |
| `Inbox` | `needs.start_messages` | `bounds.inbox` |
| `Unacknowledged` | `needs.unacknowledged` | `bounds.unacknowledged` |

A charter is found by `version.charter`, as `shortfall` reports it. A model is found by its index in the charter's `models`, then by name in `bounds.models`.

The carriers:

- `jig_domain_fleet::Request::Refuse { channel: Token, hello: Option<HelloRefused> }`
- `jig_domain_core::Held::Refuse { channel: Token, hello: Option<fleet::HelloRefused> }`
- `temper_engine_domain::Delivery::Refuse { channel: Token, hello: Option<fleet::HelloRefused> }`

`hello` is Some from `channel.rs`, `refuse`, the one place a hello's reason is recorded. It is None for a hello's malformed metadata (`channel.rs`), a later record's fault (`sources.rs`, `fault`) and a hello to a stopping core (`jig-domain-core/src/routing/stopping.rs`).

`HelloRefusal`, `HelloRefusalRecord` and every document, wire or stored form of them stay as they are. The figures cross no wire and aren't stored.

## 2. The refusal's words (temper)

`refusal-words` writes them in `temper/crates/temper-service/src/route/refusal.rs`:

```rust
/// The refusal's text and the engine's line (protocol/README.md, 15.3): at most 256 bytes.
pub(super) fn text(hello: Option<jig_domain_fleet::HelloRefused>) -> Box<[u8]>
```

| `hello` | Words |
|---|---|
| None | `not admitted` |
| `Agent`, `Model` | `charter <n>, model <m>: ` then a start's words for it, `not in the agent's configuration under the same name` |
| `Agent`, `Window`, `Output` | `charter <n>, model <m>: ` then a start's words with the figures |
| `Agent`, `Turns`, `Time`, `Waiting`, `Checks` | `charter <n>: ` then a start's words with the figures |
| `Agent`, `Spend`, `Message`, `Inbox`, `Unacknowledged` | a start's words with the figures |
| `Agent` with no figures, other than `Model` | its prefix as above, then `not admitted` |
| `JobKinds` | `its job kinds are not ascending and distinct` |
| `ReservedChannel` | `channel zero is the engine's own host` |
| `Workers { limit }` | `no room: fleet.workers is <limit>` |
| `Position { worker }` | `worker position <worker> is not configured or is held by a live channel` |
| `StopBound { grace }` | `its stop bound is not shorter than the engine's grace of <ms> ms` |

"A start's words" are `temper-service/src/startup/problems.rs`, `admission`. There is one copy of them.

The engine's line, printed by `temper/crates/temper/src/host.rs` from a new `temper_service::Note::HelloRefused { channel: u64, text: Box<[u8]> }`:

```
temper serve: hello refused channel=<n>: <words>
```
