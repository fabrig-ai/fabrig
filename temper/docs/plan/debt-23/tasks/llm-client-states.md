+++
id = "llm-client-states"
part = "llm-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm/src/client.rs", "skein/crates/skein-llm/src/client/tests.rs", "skein/tests/llm/tests/suite/client.rs"]
effort = "high"
minutes = 120
+++

# skein-llm: `Client`'s states hold their data

## Goal

Half of skein D023. `Client` has 35 fields and a data-less `State` tested with `state == State::` 12 times. Each state takes what it holds, two flags that are states in disguise go, and no comparison decides a transition. Nothing outward moves.

The sketch binds at this level:

```rust
enum State {                       // no Debug: http::Call holds the credential header
    Prepared { call: http::Call, receiving: Receiving },
    Head { receiving: Receiving },
    Streaming { answer: Answer, receiving: Receiving },
    ErrorBody { answer: Answer, error: List<u8> },
    Draining, Idle, Closing, Closed,
}
struct Receiving { decoder: dialect::Decoder, content: List<Block>, content_bytes: u64, asked: bool }
struct Answer { status: u16, request: Box<[u8]>, rate: openai::RateLimit }
enum Upload { Sending { body: Box<[u8]>, offset: usize }, Done }  // a lifecycle of its own, beside State
```

These stay on `Client`, each for a reason:

| Field | Why |
|---|---|
| `http`, `sse`, `collector` and their queues | `close_sse` closes them from any state |
| `evidence`, `activity`, `upload_end`, `kept_body` | read after a terminal, or at any time |
| `outcome` | orthogonal in ErrorBody (below), and `fail_report` gates on it |
| `event_data` | already a data-carrying enum of its own |

`close_sent` (true exactly in Closing and Closed) and `sse_closed` (true exactly in Draining, Idle, Closing and Closed) go. `fail_report`'s write to `client.status` goes: nothing reads it after.

**"Then follow" has nothing to do here.** The client mirrors nothing: `resume` derives the read demand from the state after every entry, and entry pushes (`Close`, `Discard`) stay in their transitions. So there's no `follow`; the cut reads the item so, and `llm-connection-phase` has the one that exists.

Acceptance:

- **A pin first, on today's code.** No test holds that a rejected head still reports its request id: `response_head` sets `request` before its encoding and media checks, and `fail` runs still in Head. One row in `skein/tests/llm/tests/suite/client.rs`, or in a table there: a response head with an unsupported `Content-Encoding` and a request-id header ends with that id in its failure. It passes first. Break it by removing `client.request = request_id(..)`, see it fail, restore. Report that. This row stays after the refactor.
- `grep -c 'state == State::' skein/crates/skein-llm/src/client.rs` reads 0, and `rg -n 'state (==|!=) State::|close_sent|sse_closed|upload_offset|(client|self)\.(decoder|content|call)\b|client\.(status|request|rate|error)\b' skein/crates/skein-llm/src/client.rs` reads nothing. `self.request` in `Limits::native` isn't matched by that spelling, and is another thing.
- `pub` signatures don't move, and `Waiting` keeps its variants. `client::Client`'s readers in `skein-llm-connection`, `smith-agent-service`, `smith-protocol-llm` and the worlds don't change.
- `heavy cargo nextest run -p skein-llm --lib`, `-p skein-llm-world --test suite`, `-p skein-llm-connection-world --test suite`, `-p smith-protocol-llm-world --test suite` and `-p smith-agent-process-world --test suite` pass, with no assertion changed but the new row's. `heavy cargo check -p smith-agent-service --tests`.
- Memory pins hold: `skein/tests/llm/tests/suite/memory.rs`, `bounded_peer_memory.rs`, `smith/tests/protocol-llm`'s `component_boundary.rs`, `smith/tests/agent-process`'s `memory.rs`. One that grows stops and asks. Moving data into variants should free earlier, and these are ceilings, so they needn't change.
- +20 to +50 net production lines; that's the survey's guess, unmeasured. The six `expect`s that become compile-time facts go.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `skein/docs/development/debt/D023-llm-client-states.md`
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/llm.md`, 3

## Owns and may change

- **Owns:** the files in the header. `client/tests.rs` only in how it reaches the code: `:123`'s `state ==` in `settle`, which isn't an assertion, becomes a small `stopped()` helper; `client.asked` becomes `.asked()`; `prepared.upload` needs a reach.
- **Heap:** build `Decoder` at prepare, as now. Don't `Box` `Receiving`: that adds an allocation at prepare. If `clippy::large_enum_variant` fires, a scoped `#[expect]` with a reason (`boundary.rs` has one). `List::with_capacity` allocates nothing, so making `error` on entering ErrorBody moves no allocation. In `closing`, drop the old state before `http_down(Close)`, as today, so the cancellation peak doesn't rise.
- **If it overruns,** the split point is after the flags go: each side stays green on its own. Push what's green, and ask.

## Traps

These were read, not run. Each changes behaviour if missed.

1. **The placeholder hides the source state.** `closing` and `fail_report` read the state. A handler that takes the state, then calls `closing`, finds the placeholder, does nothing and loses the `Close` event. Take only inside the transition function, not around the twenty-odd `fail; closing` handlers.
2. **ErrorBody stays after `Up::End`.** `error_end` emits the failure with no `closing`, and the client waits for `Done`, which always goes to `closing`, never Idle. `Next` is inert in that window, and `waiting()` still says `Response`.
3. **A queued head's scan.** `fail_report`'s loop over `http_events` overrides the request id and evidence from a head not yet popped, and doesn't observe `rate`. Its `report` param is the natural way to pass a local `Answer`'s id at a rejected head.
4. **The upload isn't Head's data.** A Room event pushes pieces in any state, and an early response leaves the upload live until `Upload(Failed)` is popped in Streaming or ErrorBody. `tests/suite/client.rs` pins the early response, and `skein-http` panics on an `Upload` with no exchange. Keep the order of events; don't drop the body on entering Streaming.
5. **`outputs` stays on the struct.** `resume` and `has_work` check it before the state, with `asked`. Keep that order: a leftover output in Draining would wedge the client.
6. **`close_sse` leaves a `Closed` in `sse_events`,** so `has_work` is true once in Draining, which uses one route. Keeping the machines on the struct keeps that exactly.
7. **Start from anything but Prepared** puts the state back. **`closed`** matches first, to fail with the state still live, then closes machines unless the source was Closing, then pushes Cancelled if owed, then Closed.

## Steps

Commit in this order, each green:

1. The pin.
2. `Upload`.
3. `close_sent` and `sse_closed` go, for matches on the state.
4. The states, with `Receiving` and `Answer`.
5. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; what you broke for the pin and what failed; any trap the reading got wrong; which memory pins moved.
