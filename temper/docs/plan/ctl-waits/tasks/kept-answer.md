+++
id = "kept-answer"
part = "kept-answer"
kind = "feature"
needs = ["deadline-words"]
files = [
  "temper/crates/temper-client-tools/src/limits.rs",
  "temper/crates/temper-client-tools/src/root.rs",
  "temper/crates/temper-client-tools/src/tests/**",
  "temper/crates/temper-client-shell-cli/src/waited.rs",
  "temper/crates/temper-client-shell-cli/src/host.rs",
  "temper/crates/temper-client-shell-cli/src/tests.rs",
  "temper/crates/temper-client-shell-cli/src/tests/deadline.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures.rs",
  "temper/tests/client/tests/suite/kept_message.rs",
  "temper/tests/client/tests/suite/main.rs",
]
effort = "high"
minutes = 150
+++

# A kept message reaches its answer

## Goal

`temper ctl task message` on a long chat took 23.2 s to succeed, or timed out at 30 s with its link live, the engine heard, one kept
request, and no HTTP call in flight. Find out which side held the answer, by running it. Then fix it there. **A retained act ends
in its terminal answer or an explicit pending result, and no invocation is stranded with no request in flight.** Why a message
takes 23 s even when it succeeds is part of this, and the report says.

Acceptance:

- **Reproduced first,** in the lowest tier that shows it, seen red before the fix. The terminal outcome is captured at both sides
  of the boundary: each attempt's `client.pending(key)` progress, and what the engine answered it.
- **The capture answers the request's question** in the report: did the service never release the answer, or did the client not
  hand it on? If the reading under *What this cut read* is wrong, say so and say what's true.
- **The property holds:** an act the engine keeps answering *busy* has a request in flight or due within the span until the
  deadline, then ends in 6.3's line; one whose cause clears gets its answer within the span of its clearing; a same-key rerun gets
  the first answer.
- **The spans:** `temper/docs/design/client/tools.md`, 12, a request's and a watch's retry at most 2 s, and a step test holds it.
- **The words:** 6.3's three rows naming a retry's why, word for word, within 240 bytes at a 64-byte ADDRESS.
- **The 30 s stays,** and no format or protocol changes.

## Inputs

- `temper/docs/plan/ctl-waits/notes/common.md`.
- `temper/docs/plan/ctl-waits/inputs/request.md`: the table of runs, and item 1.
- `temper/docs/design/client/tools.md`, 6.3 (the tables), 7, 8.2 (**A call is never left waiting with nothing to wait for**), 12.
- `jig/docs/design/client/domain/client.md`, 6 (*Progress, for showing*), 8.2.
- `temper/docs/design/domain/testing.md`, 2.3 and 2.5, for the worlds.

## What this cut read, and did not run

Treat this as a hypothesis to break.

- **Only an HTTP reply settles an act.** The stream feeds only the link and `me`. So "no call in flight" is `Retrying` or `Parked`.
- **The spans fit the times.** `temper-client-tools`' `limits.rs` has `first` 1 s and `most` 30 s; `skein-lib`'s `requests.rs` jitters
  each span to between half and all of it, doubling. Five spans put the sixth attempt at 15.5 to 31 s, midpoint 23.25 s, against the
  23.2 s run. The next span is 15 to 30 s, which passes the deadline with nothing in flight. A rerun starts its backoff afresh: the
  0.22 s retry. `backoff` in the same file spaces a refused `me`, the same way: the walk's read of 17 s may be this too.
- **Who answered *again* is not known.** Candidates: a full inbox, words piling up because no run takes them
  (`jig/crates/jig-domain-tasks/src/inbox.rs`, `room_staged`, `Refusal::Busy`, to `Again::Busy` in
  `jig/crates/jig-domain-core/src/routing/refusals.rs`); or two exchanges already active on the sign-in under concurrent reads
  (`jig/crates/jig-protocol-engine-clients/src/request.rs`, `exchanges_per_client`, 2 in the small and standard profiles).
- **Other shapes it could take:** parked with `me` not live ("link live" doesn't cover that watch); a final dropped as `Stale` or
  undecodable (`skein/crates/skein-lib/src/requests.rs`; `jig/crates/jig-client-protocol-link/src/component.rs`); a journal hold in
  `temper-engine-domain`. No model, worker or host hold was found on the message's success path.

## Owns and may change

**Owns:** the `files` above.

**Survey, on the trunk this spec was cut from:**

- `requests.most` and `backoff.most` are read by `jig-client-domain` (`limits.rs`'s validity, `first` at most `most`, which holds;
  `watches.rs`), `jig-client`'s `client.rs` and `skein-lib`'s `requests.rs`. Nothing in the tools or the shell reads `.most`.
- `.first` readers, unchanged: `temper-client-tools`' `change.rs` and `tests/change.rs`; `temper-client-shell-cli`'s `tests.rs` and
  `tests/fixtures/changes.rs`. Tests that run to the deadline with retries (`a_change_wait_*`, `a_stopping_engine_*`) may see more
  attempts: an edit there is to a count or a span only, never to the expected line or exit.
- The tools' preset alone changes: the terminal and the browser have their own limits.

**This changes what a shared item means,** when a retry goes out, so its readers' tests run (Steps, 6).

**Worlds, lowest first:**

- `temper/tests/client` (`temper-client-world`): the tools' real root and `jig-client`, the real engine root and clients' component,
  a scripted worker. `fixtures::held_chat`, `fixtures::agent`, `fixtures::give`, `fixtures::answered`, `fixtures::read`;
  `tests/suite/agent_recovery.rs` is the nearest test.
- `temper/tests/one-process` (`temper-one-process-world`): the real service and real `temper ctl` (`src/ctl.rs`, `Caller`) on the
  simulator, with the fake model. A resume refused goes through `skein_fake_llm_domain::api::Line::Fail`. `temper/tests/system`'s
  `a_long_chat_recovers_from_unusable_resume_history` and `World::long_chat` show the long chat's shape.

**Pre-approved options, by what the capture shows:**

- **a. The engine answered *again* or the answer was lost, and the client was retrying (expected):** the fix is the spans, Steps 3.
  Say who answered and why. If that cause is a bound doing its job, it stays: changing a bound is a question.
- **b. The answer arrived and its handle stayed pending:** reduce it to a client step test, in `jig-client-domain` or `skein-lib`,
  skein first where it's generic, and fix it there; those files are open to this task. The spans change too.
- **c. The service never released it:** reduce it to the core or journal step (`jig/tests/core`, `temper/tests/engine/domain`), and
  fix it there; those are open too. The spans change too.
- **d. A fix needs a wire or stored change, a bound's change, or the engine's meaning of *busy* to change:** commit the failing test
  behind `#[ignore = "..."]`, push, and ask, with the capture. That's the stored-format and protocol rule.
- **The fake model has no 413 row** (`skein/crates/skein-fake-llm-protocol/src/provider/failure.rs`). Use its 400
  `context_length_exceeded`, which classifies like a 413 with a context kind (`skein/crates/skein-llm/src/wire/failure.rs`). If that
  doesn't show the fault and 413 might, add the row there, in skein, and say so.
- **The story's home:** `temper/tests/client` if it shows the fault. If only `one-process` does, the story goes in
  `temper/tests/one-process/tests/suite/kept_message.rs` with its `mod` line, and those files are open. Mind its budget: over is red.
- **Words:** `Retry::Full` keeps the plain outcome row, since *full* has its own line and patience. `Retry::Undecodable` takes the
  *lost* row.

## Steps

**1. The story, failing first:** `kept_message.rs`, in the shape:

```rust
#[test]
fn a_kept_message_reaches_its_answer_while_its_chat_cannot_take_it() { /* a chat that takes nothing, its inbox at its bound or
    its sign-in's exchanges busy, as the capture finds; concurrent reads; `task message --key long-1`; each attempt's progress and
    answer collected; room returns; the answer prints, status 0, within the span */ }
#[test]
fn a_kept_message_asked_again_under_its_key_gets_the_first_answer() { /* the call ends at its deadline; room returns; the rerun */ }
```

A throwaway run in `one-process` to find the cause is fine: keep only what's lowest. Report the capture as a table: attempt, time,
progress, the engine's answer, against the request's four rows.

**2. Reduce it,** by the options: a step test that holds the fault without the world.

**3. The spans:** in `temper-client-tools`' `limits.rs`, `client()`: `requests.most` and `backoff.most` are `Duration::from_secs(2)`,
`first` staying 1 s; 2 s is the most a sign-in's link waits (`temper-client-shell-cli`'s `signing.rs`, `MOST`). A step test in
`temper-client-tools`' `src/tests/` holds them against `limits::DEADLINE`.

**4. The why,** for 6.3's rows:

```rust
impl Root { pub fn last_retry(&self) -> Option<jig_client::Retry>; } // the call's own act, or the kept request it joined
```

Noted in `route` on `Told::Pending`, kept through a later `InFlight`, so it says the last answer. In `src/waited.rs`, beside
`deadline-words`' items: `Waited::Again(Again)`, `enum Again { Busy, NotReady, Lost }`, chosen for a *live* acting call with a
`last_retry`, ahead of `Waited::Outcome`. `link_progress` passes it. `deadline-words`' rows and tests stay as they are.

**5. Tests,** in `temper-client-shell-cli`'s `src/tests/deadline.rs`, on `fixtures::Engine`, failing first:

- `an_act_answered_busy_is_tried_within_its_span_up_to_the_deadline`: no gap between attempts past 2 s, the last within 2 s of the
  deadline, then the busy row and the kept words, exit 1;
- `an_act_answered_busy_then_done_prints_within_a_span_of_its_clearing`;
- `a_read_whose_me_is_answered_busy_is_answered_within_a_span_of_its_clearing`;
- the rows' words and bound, in the table `deadline-words` left.

Each bites: with `most` at 30 s, the first three fail.

**6. Checks:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools -p
temper-client-shell-cli -p temper-client-world --all-targets -- -D warnings`, naming any other package you changed. Then the readers'
tests, since a retry's timing changes, one command at a time: `heavy cargo nextest run -p temper-client-tools`; `-p
temper-client-shell-cli`; `-p temper-client-world`; `-p temper-one-process-world`; `-p temper-real-loop-world`. Commit and push.

## Report

**Which side, and why 23 s:** the capture's table, and one plain paragraph a walker can read. Who answered *again*, and what would
clear it. Which option you took. Any reader test whose count or span you edited. Each new test's bite.
