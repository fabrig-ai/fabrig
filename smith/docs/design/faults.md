# Faults in smith's worlds

Provisional, 2026-11-21. The providers' and programs' faults that smith's
worlds make fall, the ledger they keep of them, what each fault may do to
a run's retries and how the run ends, the sweeps that draw them, and the
ignored scenarios of the problems still open. Written from
`temper/docs/design/draft/failure-worlds.md`: 3.3 for runs, 4.2, 4.3's
smith faults, 5, 6 and section 7's provider rows. It builds on skein's
fault ledger and sweeps (`skein/docs/design/testing/sweeps.md`) and on
skein's fake LLM's faults (`skein/docs/design/testing/fakes/llm.md`, 2.7,
2.8 and 5). It is a part of smith's testing design (`testing.md`), kept
apart since that document is at its size mark: when that splits, this
moves into its directory.

## 1. In one page

- **The faults** (section 2): skein's fake LLM strikes an account, cuts a
  stream or hangs a request on the direct backend; smith's fake program
  answers as Claude Code reports a billing refusal, an edge's refusal, a
  model the account doesn't serve, a limit or a lost login; a program
  starts without its tools; a delivery check waits for a place; a saved
  window resumes on another endpoint.
- **The ledger** (section 3): each world that injects them keeps skein's
  fault ledger per run, copies each fake's report into it, and counts the
  session's and the program's retries against the faults that strike
  them.
- **Blame for runs** (section 4): a table, per fault, of the retries it
  allows and the ends it allows a run it strikes, held by
  `smith_faults::Runs`. The task tries and the holds are jig's, judged
  where jig runs, in temper's system world
  (`temper/docs/design/testing/checks.md`, 6.3).
- **Sweeps** (section 5): one per family in the cheapest smith world that
  reaches it, each asserting that its faults fell and that every run's
  blame held.
- **Open problems** (section 6) are ignored scenarios, named as the
  draft's catalog names them.

## 2. The faults

| Fault | Kind (family `provider` unless said) | Made by | World |
|---|---|---|---|
| a rate window shared by an account's tasks, 429 with reset headers and no `retry-after` | `rate window` | the fake LLM's `Strike::Window` (fakes/llm.md, 2.7); the program's rejected `rate_limit_event` (protocol/programs-claude-code.md, 1.6) | protocol-llm; agent-process |
| spent credits, never returning | `spent credits` | `Strike::Spent` (`insufficient_quota`) | protocol-llm |
| an expired sign-in | `expired sign-in` | `Strike::SignedOut`; the program's lost login (`Step::LoseLogin`) | protocol-llm; agent-process |
| a model the account doesn't serve | `unserved model` | `Strike::Unserved`; the program's envelope (2.1) | protocol-llm; agent-process |
| a request that hangs after its head, or goes idle | `hang after head`, `idle mid-stream` | a hold never released (fakes/llm.md, 2.8); the program's `Step::Hang` | protocol-llm, host; agent-process |
| a 200 stream ended before its completion event | `cut before output`, `cut mid-text`, `cut mid-call` | `Line::Cut`, `Config.cut` (fakes/llm.md, 2.8) | protocol-llm |
| a billing refusal | `billing refusal` | the program's envelope (2.1) | agent-process |
| an edge's or a proxy's refusal, the billing refusal's control | `edge refusal` | the program's envelope (2.1) | agent-process |
| a saved window bound to another endpoint [composition] | `window elsewhere` | a run resumed with its transcript on another endpoint (domain/session-context.md, 1.2) | agent |
| a program started without its tools | family `worker`, `untooled program` | the program's `system/init` without the agent's server (protocol/programs-claude-code.md, 1.6) | agent-process |
| a delivery check waiting for a build place [assertion] | family `worker`, `check waits for a place` | the check's `smith-check: waiting` (domain/run-delivery.md, 1.1) | run |

The fake LLM's kinds are `skein_llm_kit::faults`' constants (fakes/llm.md,
5); the fake program names the same faults by the same constants, and
smith's own by `smith_faults`' (section 4).

### 2.1 The program's envelopes

smith's fake `claude` (protocol/programs.md, section 8) answers a
billing refusal, an edge's refusal and an unserved model with three
documented synthetic envelopes, a `result` text each, built by
`End::billing_refusal()`, `End::edge_refusal()` and
`End::unserved_model(model)`, and reports each fault step or end it
plays as an observation with its moment: temper's
`temper/docs/design/testing/fakes.md`, section 1, gives their bytes and
the report, beside the fake LLM's faults. Today smith classes them
*invalid*, *unavailable* (retried) and *invalid* (protocol/programs-claude-code.md,
1.6).

## 3. The ledger and what smith's worlds count

- **One ledger per run** of a world that injects faults
  (`skein/docs/design/testing/sweeps.md`, 3), returned with its outcome.
  The world records a fault as it strikes an account, reaches a hold,
  scripts a resume elsewhere or a program without its tools, and copies
  the fakes' failures from their reports as it takes them: the fake LLM's
  failed replies, holds and observations (fakes/llm.md, 5), the fake
  program's observations (2.1), the check's markers.
- **Identities:** the account, by the name the world's host gives it; the
  request, by the call's name across its attempts (`run 1 call 2`), so
  that a call's retries name it as its fault does. A strike's entry
  strikes its account; a call's (a cut, a window's refusal) strikes its
  account and its call.
- **Intervals,** as fakes/llm.md, 5 gives them: a strike from its strike to
  its clear; a window's refusal from the refusal to its reset; a cut at
  its instant; a hang from the hold reached to its release, or never; a
  program's fault at the moment it played, to the run's end; a program's
  rejected rate limit is its window's refusal of its call, to the reset it
  names, or to the run's end where it names none.
- **The counters smith's worlds count,** from the event stream's response
  records (protocol/events.md: `attempt`, `retry_ms`), against each fault
  `striking` names at the attempt's moment:
  - `SessionRetries`: each attempt after the first that the session makes
    of a call;
  - `ProviderRetries`: each `system/api_retry` a program reports, its own
    retry.

  `TaskTries`, `Asks`, gates and repairs are not smith's: no task, party or
  gate runs in smith's worlds.
- **Streams:** a sweeping world draws its faults from
  `Streams::rng(Stream::Faults, "llm")` and `"program"`, its runs' work
  from `Workload`, and its simulated world from `Scheduling`
  (`skein/docs/design/testing/sweeps.md`, 2).

## 4. Blame for runs

The draft's 3.3 table, for what smith's worlds see. A row is the target:
where today differs, the difference is section 6's scenario.

| Fault | Session or program retries within its interval | A run it strikes ends |
|---|---|---|
| rate window | none: the next attempt comes after the window's reset | completed; limited, naming the account, if the window outlasts the run |
| spent credits | none after the first | limited, naming the account, with no reset |
| expired sign-in | one, on a refreshed credential | completed after the refresh; unauthorized, naming the account |
| unserved model | none after the first | refused, naming the account and the model |
| billing refusal | none after the first | refused, naming the account |
| edge refusal | as any transient failure, to the session's bound | completed; an outage past the bound |
| hang after head, idle mid-stream | as a transient failure once the request's own head or idle deadline fires, to the session's bound | completed; an outage past the bound. That no watchdog cuts it first is its scenario's (section 6), not `judge`'s |
| a cut, at each place | with backoff, to the session's bound; nothing of the cut reply acted on, so a cut call never runs, nor runs twice | completed |
| window elsewhere | none | completed, its window carried over |
| untooled program | none | untooled, naming the program |

A cut drawn with more `times` than the session's retries is no longer a
cut but an outage: its run ends as one, and the host's outage retries are
jig's (`jig/docs/design/domain/tasks/lifecycle-holds.md`, 1). A check waiting
for a place is judged by the run world's own check of the held bound
(domain/run-delivery.md, 1.1): its wait is not the check's time, and spends no
retry of the check.

```rust
// smith/testing/smith-faults, package smith-faults
pub const BILLING_REFUSAL: Kind;   // provider, "billing refusal"
pub const EDGE_REFUSAL: Kind;      // provider, "edge refusal"
pub const WINDOW_ELSEWHERE: Kind;  // provider, "window elsewhere"
pub const UNTOOLED_PROGRAM: Kind;  // worker, "untooled program"
pub const CHECK_WAITS: Kind;       // worker, "check waits for a place"

pub enum Retried { Session, Program }
pub enum Ended { Completed, Limited, Refused, Unauthorized, Untooled, Outage, Failed }

pub struct Runs { /* the attempts and ends a world saw */ }
impl Runs {
    pub fn new() -> Runs;
    pub fn started(&mut self, at: Time, run: &str, account: &str);
    /// An attempt after a call's first, from its response record; `call`
    /// names the call across its attempts.
    pub fn retried(&mut self, at: Time, account: &str, call: &str, by: Retried);
    /// A run's end, with what its answer names.
    pub fn ended(&mut self, at: Time, run: &str, account: &str, ended: Ended);
    /// Counts each retry against the faults striking it, then judges each
    /// fault's row: the first finding names the fault, its interval, and
    /// the run or call.
    pub fn judge(&self, ledger: &mut FaultLedger) -> Result<(), String>;
}

/// The end a run's answer names, as its host reads it.
pub fn ended(answer: &smith_domain_run::Answer) -> Ended;
/// The same end, as a hosted program world's host reads the channel's answer.
pub fn ended_channel(answer: &smith_channel::Answer) -> Ended;
```

- **`ended_channel`:** classifies `smith_channel::Answer` by the rule
  below, its `ModelFault` and `CompletionFailure` taken as `ended` takes
  the domain's; a focused parity test holds that each pair the channel
  carries ends as its domain twin.

- **`ended`:** `Answer::Accepted` and `Answer::Parked` are `Completed`;
  `Answer::Failed` names `Limited`, `Refused`, `Unauthorized` or
  `Untooled` by its `Fault` (domain/run.md, section 9); a
  `Fault::Completion` whose failure is `CompletionFailure::Unauthorized`
  is `Unauthorized` too, the direct backend's expired sign-in (section
  4's row), as `Fault::Unauthorized` is a program's; a
  `Fault::Completion` whose failure is of a class the session retries
  (domain/session.md, 4), past its retries, is `Outage`; any other
  failure, and `Answer::Refused`, is `Failed`.
- **Attribution:** a retry is counted against each fault in force on
  its account at its moment (`FaultLedger::striking`). A fault strikes a
  run when they share the account and the fault's interval meets the
  run's span, its start to its end; where several struck it, the latest
  to fall before its end decides the ends allowed: blame follows the
  latest cause. A rate window's strike judges no retry of its own: the
  window is the account's limit, and its refusals are the faults the row
  judges. A retry within a refusal's interval, refusal to reset, is
  refused by the row, and one after that reset is allowed. The strike's
  interval, strike to clear or ongoing, still decides which runs it
  struck and so their ends: completed, or limited naming the account
  where a refusal's reset comes after the run's end, or it names none.
  `check waits for a place` and today's classes
  (`overloaded`, `unavailable`, ...) are counted, not judged.
- **A finding is a red** of the sweep or scenario that called `judge`,
  with its seed, as any (`skein/docs/foundation/testing-strategy.md`, 6).
- **One table, two halves:** these rows are smith's half of the draft's
  3.3 for runs; temper's system world judges the task tries and the holds
  with jig's facts, and calls `Runs::judge` for this half
  (`temper/docs/design/testing/checks.md`, 6.3).

## 5. Sweeps

One per family, each a fuzzy test over its seeds, in the cheapest world
that reaches its faults (`skein/docs/design/testing/sweeps.md`, 4):

| Sweep | World | Configures |
|---|---|---|
| `provider_faults_fall_on_the_direct_backend` | `smith-protocol-llm-world` (`fuzzy/faults.rs`): the root domain, the LLM component and skein's client against the byte peer | rate window, spent credits, expired sign-in, hang after head, idle mid-stream |
| `provider_faults_fall_on_a_program` | `smith-agent-process-world` (`fuzzy/faults.rs`), the fake program hosted | rate window, expired sign-in, edge refusal |
| `untooled_programs_fall` | `smith-agent-process-world` (`fuzzy/faults.rs`) | untooled program |
| `checks_waiting_for_a_place_fall` | `smith-run-world` (`fuzzy/checks.rs`, beside its markers' sweep) | check waits for a place |
| `resumed_windows_carry_over` | `smith-agent-world` (`fuzzy/faults.rs`) | window elsewhere |

Each sweep draws the runs it strikes from `Workload`, part `runs`, and
its faults from their own parts.

The provider family takes two sweeps, since no one world runs both
backends; the worker family's two faults fall in different worlds.

- **Each seed:** the world draws its faults at any step from its
  `Faults` stream, runs its runs to their ends, folds its ledger
  (`Sweep::ran`), and calls `Runs::judge`; after the last seed,
  `Sweep::check_fell`.
- **Exclusions** name the ignored scenarios of section 6 and leave out
  exactly their combinations, read from the world's state at the moment
  the fault would fall: a window longer than the session's shortest
  first backoff (domain/session.md, 4), whose reset may come after the
  next attempt (`a_rate_window_is_waited_out`). Hangs take none: the
  request's own deadlines cut them before a watchdog acts (section 6).
- **A fault every draw of which is its open scenario's combination is not
  configured** until its fix: the cuts, the billing refusal and the
  unserved model today. `Sweep::exclude` refuses an exclusion with no
  condition, and `check_fell` fails a configured fault that never fell,
  so a fault whose every draw is excluded can't be configured: its
  ignored scenario holds it, and the work that fixes it configures it
  in its sweep as it removes the `#[ignore]`. Each such fault is named,
  with its scenario, in a comment beside its sweep's `configure` calls.
- **Room:** each within its tier's budget
  (`skein/docs/foundation/testing-strategy.md`, 8.1); its seeds are as
  few as make every configured fault fall in the seeds folded.

## 6. Open problems

Ignored scenarios (the draft's catalog), each `#[ignore = "<reason>"]`,
its reason naming the work that fixes it, *provider failures*:

| Scenario | Problem | World | It asserts |
|---|---|---|---|
| `a_rate_window_is_waited_out` | P127 | `smith-protocol-llm-world` | a 429 with reset headers and no `retry-after` is waited out: the next attempt after the reset, the run not failed |
| `a_cut_stream_is_retried_not_failed` | the 11-20 cut streams | `smith-protocol-llm-world` | at each of the three places, the cut call is retried with backoff, nothing of the cut reply is acted on (a cut call runs once, from the completed reply), and the run completes with no failure answered, so no task try and no person |
| `a_billing_refusal_marks_the_account` | P72 | `smith-agent-process-world` | the billing envelope ends the run refused for its account, with the refused notice, no program restart and no retry, and the edge's refusal is still retried |

The draft's P72 row names smith's session world, which runs no program:
the fake program runs in the agent-process world, so its scenario is
there. The task's move after the refusal is jig's, on the run's refused
end. The rows P26, S1 and P126 need placement and an account's models,
which are jig's: their scenarios are in temper's system world
(`temper/docs/design/testing/checks.md`, 6.3), where S1's passes and is
enabled.

P71's scenario, `a_silent_run_is_cut_at_its_derived_bound` in
`smith-host-world`, passes and is enabled, an active regression: it
composes the protocol world's client with the host's watchdog, and on
both dialects a run silent while its request hangs is cut by the
request's own idle deadline, retried with backoff, and ended by its typed
idle failure before the watchdog acts, its first piece never committed.

## 7. Testing

- **The fake program's envelopes and report:** its step tests
  (`tests/fake-claude/tests/simulated/`): each envelope's bytes, and
  smith's classification of each as today's (*invalid*, *unavailable*,
  *invalid*); each fault step's observation with its moment.
- **`smith_faults`:** step tests over hand-made ledgers: a retry inside a
  rate window's interval found, one after it passing; each row's allowed
  and refused ends; `ended` over each fault of domain/run.md, section 9.
- **Each sweep** shows that its faults fell and its runs' blame held;
  **each ignored scenario** compiles, fails today for its problem, and
  names its work.

## 8. Later

- Configuring the cuts, the billing refusal and the unserved model in
  their sweeps, each with its fix.
