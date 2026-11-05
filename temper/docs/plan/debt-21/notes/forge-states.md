# debt-21: forge-connection's states (skein D020)

Binding on `hooked-states`, `connection-states`, `connection-deadlines` and `forge-states-close`. `C` is `skein/crates/skein-forge-connection/src`. Line numbers are at the plan's base.

## The shapes

Sketches: the variants and where the data goes bind, the inner names are yours. `skein/docs/foundation/programming-model.md`, 5.4 gives the idiom: move the state out, assign the match's result back, follow it in one place.

```rust
// C/connection.rs. `Phase` stays a private tag that `phase()` computes, so that the
// component's 22 reads and the tests' 25 change only `.phase` to `.phase()`.
enum State {
    Closed { closing: bool, ended: Option<Ended> },
    Connecting,
    Handshaking,
    Sending { exchange: Exchange },
    Reading { exchange: Exchange, draining: bool },
    Idle,
}
struct Exchange { upload: Option<Upload>, receive: Receiving, json_body: JsonBody, json_next: Option<tokenizer::Request> }
```

`connection-deadlines` then adds `until: Option<Time>` to every variant but `Closed`. It is `Option` so that it stays bit-exact (C3 below).

```rust
// C/hooks/connection.rs. `NextOwed` is today's `done`.
enum State {
    Head { until: Time, held: Option<Hint> },   // a hint the root refused, after `Next` went (H1)
    Body { until: Time, mac: Hmac, signature: [u8; 64], forge: u16,
           decoder: Option<Decoder>, decoded: Option<Delivery>, malformed: bool, ended: bool },
    Answering { until: Time, hint: Option<Hint>, offer: HintState, next: NextOwed },
    Closing(Closing),                           // Pending { until }, Sent, Aborted
}
```

- **Stay beside `Connection`'s state:** `forge`, `role`, `origin`, `membership`, `link`, `http`, `json`, the four queues, `shared`, `raw_cap`, `json_limits`. So does `written`, a fact of the last send that's read in Idle and Closed after its exchange has gone (C6); a comment says so. So does `deadline`, the value `follow` writes.
- **Stay beside `Hooked`'s state:** `socket`, `http`, `events`, `below`, `json`, `json_events`, `json_down`.
- **`follow` sets the deadline:** one function per type, called after each transition, an exhaustive match over the state. `Connection`'s demand comes from http, the tokenizer and `raw_demand`, not its phase, so `follow` sets its deadline alone. On `Hooked`, the state-derived demand is `Next` on entering Head, and it goes in `follow` if that's simple; say which.

## Where behaviour could silently change

Each of these is today's behaviour, and each stays. "Pin" means write the test on the trunk's code first and commit it apart, so that it passes before the refactor and after. A row with no pin has a test already.

**`Connection`**

| | What happens today | Pinned by, or pin |
|---|---|---|
| C1 | Idle reached from connect keeps the holder's deadline (`connection.rs:243, 395`) | `each_start_read_keeps_its_deadline_through_connect_and_starts_afresh` |
| C2 | an unheld connection that reaches Idle from connect runs out `limits.deadline`, not `idle` (`component/calls.rs:438`) | **pin** |
| C3 | Idle with `None` (`component/github.rs:951`): `None <= Some(now)` is true at `calls.rs:438`, `github.rs:921`, `connection.rs:217`, so `None` can mean "close now" | **pin** |
| C4 | `component.rs:703` can write `Some` onto a Closed connection after `Connection: close`, until `:797` clears it | **pin** |
| C5 | `calls.rs:272` writes the idle clock on any phase but Closed; `github_calls.rs:459` guards on Idle. Whether Reading reaches the first is untraced | trace first, **pin** if it does |
| C6 | `written` outlives its exchange (`component.rs:530, 760`, `calls.rs:200`, `github_calls.rs:328`) | `a_calls_deadline_distinguishes_connecting_from_a_written_request`, `github_call_keeps_one_deadline_and_first_byte_cost` |
| C7 | `connect` during Sending overwrites to Idle with `exchanging` set (`connection.rs:241`), test-only as far as the cut can see | `a_connected_notice_keeps_the_in_flight_exchange` |
| C8 | Closed with `exchanging` set, briefly (`connection.rs:460-461`, until `finish`) | raw-bound rows, `tests/connection.rs:229-245` |
| C9 | an early head during Sending, and `Up::Failed` or `End` leaving Sending with no upload (`connection.rs:568`) | **pin** |
| C10 | `ended` survives `close` (`calls.rs:233` hears it after), and `failed` keeps the first ending (`connection.rs:586`) | fault rows, `tests/dialects.rs:267` |
| C11 | `draining` is read at release (`github_calls.rs:461`), so it can outlive Reading: carry it where that needs it | `original_deadline_covers_follow_before_and_after_its_write` |
| C12 | `Debug` (`connection.rs:654`) prints the phase and deadline. Print the tag: a data-carrying state would show `Grow`'s body bytes | seven tests' redaction, of secrets only |

**`Hooked`**

| | What happens today | Pinned by, or pin |
|---|---|---|
| H1 | Head holding a returned hint is the normal path: the refusal comes after `Next`. `done == false` means `Next` already went | `a_held_hint_stops_the_next_body_until_it_is_offered`, `a_hint_the_root_refused_is_held_and_offered_again`, `runtime.rs:204` |
| H2 | a refusal during Body: `:227` drops the returned hint on a `204`, keeps it on anything else | trace; if reachable, `Body` holds it too, with a **pin**, and the report says so |
| H3 | a refused head stays `Closing::Open` until `Done`. Entering Closing early trips the gate at `:101` and could lose the answer | the 403, 404 and 405 tests |
| H4 | a `204` with the client's `Connection: close` offers the hint before Close (`:265` before `:279`) | **pin** |
| H5 | a stale `hint_state == Ready` after an unhinted delivery sends the next hint up before `Done`; the outward order is the same | trace; **pin** if a difference can show |
| H6 | **The most dangerous.** The deadline is set only at accept and at `Done`: no restart on Head to Body, Answering to Head, a refusal or an offer. `deadlines_end_each_hook_phase` runs everything from t=0, so a `follow` that restarts on entry passes every test and silently retimes | **pin:** a head that spends most of `hook_deadline` before its body still ends at accept + `hook_deadline`; `until` is carried across, never recomputed |
| H7 | `Pending` still pumps `below` and stays under the deadline (`hooks.rs:128, 168`). `Sent` to `Aborted` | `abort_after_close_aborts_a_webhook_connection_closing_gracefully_and_the_listener_closes_once` |

A pin goes in the file holding its neighbours, as a row of an existing table where one fits. Break each by hand (the common note), and name the break in your report. **If keeping any of these means a state the design doesn't have, keep the behaviour** and say so in your report: `forge-states-close` records it.

## Limits

- `Phase`, `Connection` and `Hooked` are private to the crate (`C/lib.rs:8`, `C/hooks.rs:6`). No other crate changes.
- `C/installations.rs` stays as it is: the item's **Paid when** doesn't name it.
- `skein/docs/development/debt/D030-*.md` moves these test files' harness tests, and isn't in this plan. Don't do it here.
- **Tests:** `heavy cargo nextest run -p skein-forge-connection --lib`; `-p skein-forge-machine-world --test suite`; `-p skein-forge-protocol-world --test suite`; `-p skein-fake-peers-tests`. `forgejo-conformance` and `github-conformance` stay outside your run.
