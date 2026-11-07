# forge-read-answer: the binding calls

## 1. The cause, as read

`temper serve` panicked at `temper-engine-smith/src/answers.rs`, `forge_answer`, on an agent's `read_file` answered with an Actions or tree kind. Nothing in `ui-qa-small`'s merge (`aaa951702a`) touched forge reads or the client's admission: it changed client pages, the configuring stock and `agent.md`, 8.1, so the agents' traffic changed, not the code that carries it.

The cause is two counters naming one range of client owners:

- the root names an agent's read `serial | 1 << 61`, its serial `domain.next_effect_owner` (`temper-engine-domain/src/route/forge_route/reads.rs`, `read_call`);
- the connector names a change step's reads and a landing's reads `sequence | 1 << 61`, from its own `d.sequence` (`temper-engine-domain-forge/src/domain.rs`, the step's `Token::new(sequence | (1_u64 << 61))`; `landing.rs`, `Token::new(d.sequence | (1_u64 << 61))`);
- the connector routes a client answer (`domain.rs`, `skein_forge_client::Request::Read`) to its own maps first, `d.steps` and `d.landing.pending` among them, and passes on to the root what has bit 61 set and bit 62 clear.

When the serial and the sequence meet, the client holds two reads under one owner. Under load the client refuses the agent's read `Busy` at once (`skein-forge-client/src/domain.rs`, `Event::Read`); the connector hands that `Busy` to the step (`step_read`, `retry_step`), which drops its owner; the step's own answer (`Commits` for containment, forge.md 5.3; or a landing's `Tree`) then finds no step, goes to the root, and the root answers the agent's waiting `read_file` with it. That attempt's `Busy` is never noted, which fits the live log: two refusals noted, then the panic. The client itself never answers a kind its call did not ask for (`skein-forge-client/src/calls.rs`, `api::accepts`), so a foreign kind can only come from this crossing.

## 2. The root's read owners (forge.md, 5.3, *Each answer reaches the read that asked*)

In `temper-engine-domain-forge`, `src/boundary.rs`, beside `Request::Read`, exported from `lib.rs`:

```rust
/// The owner of the root's fresh read `serial`: bit 61 alone of the top five
/// bits, the serial below 2^59; `None` past that (forge.md, 5.3).
#[must_use]
pub fn root_read(serial: u64) -> Option<Token>;

/// Whether a client owner is in the root's read range: bits 63 to 59 are
/// 0, 0, 1, 0, 0 (forge.md, 5.3).
#[must_use]
pub fn is_root_read(owner: Token) -> bool;
```

- The root mints its reads by `root_read` alone; its exhaustion refusal ("read correlation exhausted") answers where `root_read` gives `None`.
- The connector's router passes an unclaimed `Request::Read` to the root where `is_root_read(owner)`, else to `lost_read`, as today.
- The connector mints none of its own read owners in that range. Its change steps and its landings take `sequence | 1 << 62`, beside the lost-writer reads that already use it: `d.sequence` is one counter, so their low bits never meet. Brief reads (`1 << 61 | 1 << 59`) are outside the range already.

## 3. An answer no agent read asks for (agent.md, 8)

In `temper-engine-smith`, `src/answers.rs`, exported from `lib.rs`:

```rust
/// The kind of a forge read's answer no agent read asks for, as the log's
/// reason: `Some("answer of kind Commits")` for `Commits`, and the same for
/// `Runs`, `Run`, `Tree` and `Queued`; `None` for every other answer
/// (temper/docs/design/domain/agent.md, 8).
#[must_use]
pub fn unasked_kind(answer: &CallAnswer) -> Option<&'static str>;
```

- The rendering refuses such an answer before any bounded path, as an error, in these words, the tool being the call's tool or `the read` where none is given:
  `Forge read refused: read_file got an answer of kind Commits, which it never asks for. Nothing was read; read again.`
- `Problem::Unasked(&'static str)` is the rendering's problem for it, carrying `unasked_kind`'s phrase.
- The service notes it where it renders a host call (`temper-service/src/route.rs`, `render_host`): `Note::ReadRefused { task, attempt, tool, layer: "host-render", reason: <unasked_kind's phrase>, findings: [], error: None }`, so `temper serve` prints `temper serve: read refused task=4 attempt=3 tool=read_file layer=host-render reason=answer of kind Commits error=None findings=[]`.
