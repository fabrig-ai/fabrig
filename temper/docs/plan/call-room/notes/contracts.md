# call-room: the run's binding calls

The shapes that cross this plan's tasks. The design says what is true: smith's `protocol/limits.md`, 2.4 and 3.10; jig's `domain/hosts.md`, 8 and `protocol/configuration.md`, 6; temper's `protocol/configuration.md`, 13.8 and 13.9, `domain/agent.md`, 4.4 and `domain/forge.md`, 8.1.

## 1. The figures

| | The stock (`temper init`) | The reference | Its fallback |
|---|---|---|---|
| a model's `output`, an endpoint's | 8,192 | 98,500 | 48,000 |
| the agent's `tool_payload`, P | 10,662 | 131,072 | 63,740 |
| T | 16,803,810, unmoved | measured | measured |

skein's rule, with smith's framing: (P + 260) × 6 ≤ output × 8. The fallback is taken only if the start's checks refuse the larger; `reference-room` picks, by test, and `room.json` holds the pick.

`temper/crates/temper-service/src/stock.rs`'s `StockModel.output` is a price per unit; the stock's token output is the literal 8,192 in `models`. Neither moves.

## 2. Smith

```rust
// smith/crates/smith-domain-run/src/limits.rs, derive
run_bytes: u64::from(inputs.tool_payload).checked_mul(2)/* never overflows from u32 */.max(1 << 16),
outcome_bytes: u64::from(inputs.tool_payload).max(4096),

// smith/crates/smith-domain-run/src/host.rs
impl HostInput { pub const CAPACITY: usize = 262_144; }
impl HostAnswer { pub const CAPACITY: usize = 262_144; }
```

- **The two capacities are the first limit a declared 131,072 meets** (`valid_host_tools`, `Invalid::Grants`). 262,144 is twice the reference's payload and smith's charter sections' own bound; the channel's `host_ask_input` and `host_reply_text` ceilings, 1,048,576, stay. No generated codec moves, so it is no protocol question.
- **`large-argument` moves them,** in `host.rs`, since its acceptance needs it, and with them the literal 65,536 in `temper/crates/temper-engine-smith/src/json.rs`'s `parse`, which becomes `smith_domain_run::HostInput::CAPACITY`, one line: `a_large_settings_decoder_error_is_bounded_host_feedback` parses at the capacity. Readers that build a body at the capacity take its new size and say what it costs.
- **`smith-room` moves `run_bytes` and `outcome_bytes`,** and measures T.

## 3. jig

```rust
// jig/crates/jig-protocol-config/src/profile/standard.rs
AUTHORITY: authority::Limits { batch: 32, .. }
HUB: host::Limits { charter_bytes: 262_144, outcome_bytes: 131_072, .. }   // small keeps 65,536 and 4,096

// jig/crates/jig-protocol-charter/src/assignment.rs
pub struct Charter { /* .. */ pub words: u32 /* the run contract's words; 0 for one with none */ }
tasks::Contract::Change { words, .. } => FieldRule { name: b"body", max: words }   // title, failure, no_change: words.min(CONTRACT_TEXT_MAX)

// jig/crates/jig-protocol-charter/src/charter.rs, a report-only run that delivers
FieldRule { name: b"title", max: CONTRACT_TEXT_MAX }
FieldRule { name: b"body", max: charter.words.max(CONTRACT_TEXT_MAX) }
```

`CHANGE_BODY_MAX` goes, from `assignment.rs` and from `lib.rs`'s exports. `temper/crates/temper-engine-smith/src/room.rs` measures the largest translated contract with `words: u32::MAX` and a change `body` of `max: u32::MAX`, ten digits being the most a contract prints.

## 4. The reference's files

`temper/docs/examples/reference/room.json`, new:

```json
{
  "agent": { "tool_payload": 131072, "output": 98500 },
  "worker": { "limits": { "overrides": {
    "hub.event_bytes": 131070,
    "hub.transcript_bytes": 0,
    "hub.turn_bytes": 0,
    "hub.turn_queue_bytes": 0
  } } }
}
```

(The zeros stand for the figures 13.8's table gives from T.) `limits.json` gains, under `engine.limits.overrides`, `core.resume_bytes`, `hub.transcript_bytes`, `hub.turn_bytes`, `hub.turn_queue_bytes`, `tasks.inbox_bytes`, `tasks.message_bytes`, `tasks.result_bytes`; and under `application.limits.overrides`, `checkout.message_bytes`, `forge.client.op_bytes`, `forge_component.body_bytes`, `root.journal.transcript_bytes`, `root.loads.bytes`, `root.loads.read_bytes`, `root.loads.reply_bytes`, `root.loads.transcript_bytes`. `settings.json`'s every charter model takes the `output`, and template 4's report the `tasks.result_bytes`.

`README.md` gives the two commands:

```sh
jq --slurpfile room room.json '.profile.declared.tool_payload = $room[0].agent.tool_payload | .endpoints[].limits.output = $room[0].agent.output' agent.json
jq --slurpfile room room.json '.workers[] |= (. * $room[0].worker)' temper.json
```

## 5. temper's path

- **The start names:** both hubs' `hub.outcome_bytes` against `settings::Agent.host_limits.outcome_bytes`, in `checks::turns`, "the agent's outcome (agent.json profile.declared.tool_payload)"; `forge.client.op_bytes` against `tasks.result_bytes`, where `limits`'s validity today says only `engine.limits: past its bound`.
- **A pull request's body:** at most 65,536 bytes, in `temper-engine-domain-forge`'s `change_write`, whatever `op_bytes` allows.
- **A landing's result** leaves room in the contract's words for the label its requester reads it under (`temper-engine-domain/src/link.rs`, `inbox_words`), so a result at the bound is never refused as a live message.
