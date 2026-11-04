+++
id = "smith-stages-machines"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-agent-service/src/service.rs",
  "smith/crates/smith-agent-service/src/service/**",
  "smith/crates/smith-agent-service/src/terminal.rs",
]
effort = "high"
minutes = 110
+++

# smith D002, `Stages`: the run's opening and end as state enums

## Goal

The agent service's `Stages` stops holding lifecycles in flags. `Teardown` holds the first
failure; the answer the last line reports, the run's handle with its waiting cancel, the Start
with its result contract, and the roots with their ordered close each become one state enum with
its own transitions and `follow`. Behaviour, order of effects, wire and stored formats do not
change. This is `Stages`' share of `smith/docs/development/debt/D002-lifecycles-in-flags.md`; **the
item stays open: leave its file.**

Acceptance:

- Measured on `pub(crate) struct Stages` with
  `awk '/^pub\(crate\) struct Stages/,/^}/' smith/crates/smith-agent-service/src/service.rs`:
  `grep -c ': '` reads 44 (55 at base), `grep -c ': bool'` reads 1 (3), `grep -c ': Option<'`
  reads 6 (13). The `bool` is `usage_unread`; the `Option`s are `events`, `product_exit`,
  `program`, `input`, `output`, `signals`.
- `grep -rn -E '\b(cancel_pending|final_failure|final_refusal|final_detail|final_answer_at|root_close_pending|root_close_next|root_index|pending_start)\b' smith/crates/smith-agent-service/src`
  reads nothing, and `Stages` has no field `admitted`, `failure`, `outcome`, `delivery` or
  `directories`. Exempt, not changed: `smith/tests/agent-process` has a `cancel_pending` of its
  own, the world's; `run::Directory`, `outcome` and `failure` as words in other types stay.
- `grep -n 'fn follow' smith/crates/smith-agent-service/src/service/{teardown,admission,opening,roots}.rs`
  shows one in each file.
- `git diff origin/main -- smith/crates/smith-agent-service/src/lib.rs` is empty. The one public
  change is the added variant `Teardown::LiveUnsent`. `Effects`, `Service`, `done`, `failure`,
  `last` and `Last` keep their signatures. No file outside this spec's `files` changes; in
  particular `profile.rs`, `profile/**` (another plan is changing them) and every design, status
  and debt file stay as they are.
- `heavy cargo nextest run -p smith-agent-service --lib` selects 154 tests (147 at base, plus the
  7 sketched here) and passes. Existing tests change only where Steps 8 lists.
- The readers' tests in Steps pass, unchanged; fmt, the old-forms check and clippy are clean.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`, and the item, whole.
- `skein/docs/foundation/programming-model.md`, 5.4, whole, and section 1.
- `smith/docs/design/protocol/agent.md`, 6: the end is one phase, `Live`, `Answered`, `Closing`,
  `Closed`. It needs no edit: `LiveUnsent` is the design's `Live` with its first cause kept
  (`Closing { step }` refines `Closing` the same way) and no behaviour moves.
- `smith/AGENTS.md` (module docs, the strict subset).
- The form: `smith/crates/smith-host-domain/src/domain/supervision.rs` (a small state enum with
  `mem::replace` transitions beside its owner), `smith/crates/smith-local-domain/src/delivery/machine.rs`
  (`follow`).

How the cut read the item: its seven named fields are four lifecycles and one duplicate.
`failure` is the phase's `answer` in waiting: from `Answered` on, `answer == Unsent(first)` exactly
when `failure == Some(first)`, kept in step by a call in `mark_failed`. `final_*` are the answer
kept for the last line. `outcome` and `delivery` are no part of the end: they are the result
contract a Start brings, and `delivery` is `outcome.change` held twice. `cancel_pending` waits
for the run's handle. `root_close_pending` is the roots' ordered close. `pending_start`,
`directories`, `root_index`, `admitted` and `root_close_next` go with them.

Stay, each meaningful in every state, so no enum: `events` and `program` (configuration for the
process's life); `input`, `output`, `signals` (the open entity's name: the `Option` is the state,
and no other field describes it); `product_exit` (the product's last word, an input; `None` until
it is given); `usage_unread` (a one-shot notice for the shell); `run_started` (the run clock's
origin: `Time::ZERO` until a Start enters, and a refused Start's `elapsed` counts from that ZERO,
which must not change); `pending_messages` (a queue).

## Owns and may change

Owned: the three `files` entries, with new `service/admission.rs`, `service/opening.rs`,
`service/roots.rs`, `service/tests/lifecycles.rs`. Bare paths in this spec are under
`smith/crates/smith-agent-service/src/`. Every edit outside the new files is listed in Step 7; a
change the compiler forces beyond it is allowed.

Pre-approved options:

- `Teardown` is exported but no public function takes or gives one, and no crate outside names
  `Teardown`, `Answer` or `Step` (`Close` is named widely, and does not change). So the new
  variant reaches nothing. If anything outside the crate stops compiling for it, stop and ask.
- `Roots` pops with `List::last` and `List::truncate`, since `List` has no `pop`. If that reads
  badly, keep `roots` whole and hold `next: u32` in `Held` and `Closing`; add nothing to skein.
- `Opening::Started` holds `outcome` inline. If `clippy::large_enum_variant` fires, box it.
- The tests live in `service/tests/lifecycles.rs` with `setup`'s helpers. If one reads better
  beside its module, move it; the count stands.
- If a test drives a state this spec calls `unreachable!`, make that cell do what the code does
  at base and name it in the report.

## Steps

One commit per step 1 to 5, each with its tests passing, in this order (tests off private state
first: step 3 moves `with_admission` as it adds the transition).

1. **The last line** (`terminal.rs`). `final_answer_at`, `final_failure`, `final_refusal`,
   `final_detail` become `pub(crate) line: Line` on `Stages`:

   ```rust
   /// The answer the settled agent's last line reports (smith/docs/design/protocol/agent.md, section 6).
   #[derive(Debug)]
   pub(crate) enum Line {
       /// No answer yet, or one that needs no line.
       Quiet,
       /// A failed answer, when it was told, with its operator detail.
       Failed { failure: run::Failure, at: Time, detail: Box<[u8]> },
       /// A refused answer, with its operator detail.
       Refused { refusal: run::Refusal, detail: Box<[u8]> },
   }
   ```

   `remember` keeps its signature and assigns `service.line` from the answer: `Accepted` and
   `Parked` give `Quiet`, and their detail, which nothing reads, is dropped. `last` keeps its
   `done` and `Unanswered` head, then matches `line` to `Last::Quiet` or `Last::Failed`, through
   `failure_record(failure.clone(), *at)` and `refusal_record(*refusal)`. A line implies nothing,
   so it has no `follow`.

2. **The first failure** (`teardown.rs`). The field `failure` goes.

   ```rust
   pub enum Teardown {
       /// The run still owes its answer.
       Live,
       /// The run still owes its answer, which stays unsent for this kept first cause.
       LiveUnsent { failure: Failure },
       // Answered, Closing and Closed are unchanged.
   }

   impl Teardown {
       pub(super) const fn failure(self) -> Option<Failure>; // Live: None; LiveUnsent: failure; else answer's
   }

   impl Answer {
       const fn failure(self) -> Option<Failure>;
       const fn failed(self, reason: Failure) -> Self;       // Sent -> Unsent(reason); Unsent(first) stays
   }
   ```

   | State | `answered` | `failed(reason)` | `shutdown` |
   |---|---|---|---|
   | `Live` | `Answered { Sent, Graceful }` | `LiveUnsent { reason }` | `admission::cancel` |
   | `LiveUnsent { failure }` | `Answered { Unsent(failure), Graceful }` | stays | `admission::cancel` |
   | `Answered`, `Closing` | ignored | `answer.failed(reason)` | as at base |
   | `Closed` | ignored | `unreachable!`, as at base | ignored |

   `Stages::mark_failed` keeps its name and calls `teardown::failed`. `stages::failure` stays a
   `const fn`, reading `service.teardown.failure()`. Every other `Live` arm gains
   `| Teardown::LiveUnsent { .. }`: `stages::done`, `teardown::work_pending`, `teardown::advance`,
   `routing::record_usage`, the `Heard::Ended` arm, `program::fire`, and both `Live` arms of
   `program::policy_deadline`. `teardown::advance` is this machine's `follow` and takes the name:
   its callers in `service.rs`, `effects.rs` and 13 lines of `service/tests.rs` move by `sed`.

3. **The handle and its cancel** (`service/admission.rs`). `admitted` and `cancel_pending` become
   `admission: Admission`, starting `Waiting`.

   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub(super) enum Admission {
       /// The domain has not named the run.
       Waiting,
       /// A cancel came before the handle and waits for it.
       Cancelled,
       /// The handle reached a waiting cancel; this pass delivers it.
       Owed { run: Token },
       /// The domain named the run.
       Admitted { run: Token },
   }

   impl Admission {
       pub(super) const fn run(self) -> Option<Token>;  // Owed and Admitted: Some(run)
       pub(super) fn admit(&mut self, run: Token);
   }
   /// The one cancel entrance: `Heard::Cancel`, and `shutdown` while the run is live.
   pub(super) fn cancel(service: &mut Stages);
   /// At the tail of `domain_down`, where the deferred cancel is pushed at base.
   pub(super) fn follow(service: &mut Stages);
   ```

   | State | `cancel` | `admit(run)` | `follow` |
   |---|---|---|---|
   | `Waiting` | `Cancelled` | `Admitted { run }` | stays |
   | `Cancelled` | stays | `Owed { run }` | stays |
   | `Owed { run }` | push `Cancel`, stays | `unreachable!("one Admitted per Start")` | push `Cancel`, `Admitted { run }` |
   | `Admitted { run }` | push `Cancel`, stays | `unreachable!("one Admitted per Start")` | stays |

   `Owed` exists so the order holds: the deferred `Cancel` is pushed after the pass's request
   loop and before the messages that waited for the run, which flush on the next pass. Do not
   push it from `admit`. In the tests, `with_admission` calls `admission.admit(run)` instead of
   writing the field, and `interrupted_before_admission` asserts
   `service.stages.admission == Admission::Cancelled`.

4. **The Start and its contract** (`service/opening.rs`). `pending_start`, `directories`,
   `root_index`, `outcome` and `delivery` become `opening: Opening`, starting `Idle`.

   ```rust
   #[derive(Debug)]
   pub(super) enum Opening {
       /// No Start waits, and no contract is known.
       Idle,
       /// A Start waits for its mounts; `directories` holds those attached, in Start order.
       Attaching { start: Box<DecodedStart>, directories: List<run::Directory> },
       /// The run's result contract is known.
       Started { outcome: run::outcome::OutcomeSpec },
   }

   impl Opening {
       pub(super) fn attach(&mut self, start: Box<DecodedStart>, roots: u32);
       pub(super) fn next_mount(&self) -> Option<&[u8]>;       // backs Service::root_to_open
       pub(super) fn attached(&mut self, host: Box<[u8]>, root: Token);
       pub(super) fn take(&mut self) -> (Box<DecodedStart>, List<run::Directory>);
       pub(super) fn refuse(&mut self);                        // any state to Idle
       pub(super) fn contract(&mut self, outcome: run::outcome::OutcomeSpec);
       pub(super) fn outcome(&self) -> &run::outcome::OutcomeSpec;
   }
   /// A Start with every mount attached enters the domain.
   pub(super) fn follow(service: &mut Stages, frame: &mut Frame);
   ```

   | State | `attach` | `attached` | `take` | `contract` | `outcome` |
   |---|---|---|---|---|---|
   | `Idle` | `Attaching` | `unreachable!` (a root belongs to a pending Start) | `unreachable!` | `Started` | `unreachable!` (an admitted Start retains its result contract) |
   | `Attaching` | `unreachable!` (one Start per channel) | push one directory | `Idle` | `unreachable!` (effects hold no Start) | `unreachable!` |
   | `Started` | `unreachable!` | `unreachable!` | `unreachable!` | replaces it | the contract |

   The index into the mounts is `directories.len()`. `follow` calls `start_ready` when the state
   is `Attaching` and `next_mount` is `None`; it replaces `if empty` in `Heard::Start` and the
   check in `Service::root_opened`. `start_ready` opens with `take`, and sets
   `Opening::Started { outcome: start.charter.outcome.clone() }` beside `run_started`, before
   pushing `domain::Event::Start`, so its refusals leave `Idle`. `delivery` is gone:
   `request.rs` binds `outcome` from `service.opening.outcome().clone()` and
   `deliver = outcome.change.clone()`. `Effects::contract` keeps its signature and calls
   `opening.contract`. `refuse_start` calls `opening.refuse`.

5. **The roots** (`service/roots.rs`). `roots`, `root_close_next` and `root_close_pending` become
   `roots: Roots`, starting `Empty`. `close_roots` and `roots_closable` move here from
   `teardown.rs`, and `close_roots` becomes `follow`, called from the same two arms.

   ```rust
   #[derive(Debug)]
   pub(super) enum Roots {
       /// No root is held: none adopted, or every close reported.
       Empty,
       /// Roots not yet asked to close, in adoption order, with no close in flight.
       Held { roots: List<Token> },
       /// The root after `roots` awaits its close report.
       Closing { roots: List<Token> },
   }

   impl Roots {
       pub(super) fn adopt(&mut self, root: Token, capacity: u32);
       pub(super) fn first(&self) -> Option<Token>;       // program::install's `roots.get(0)`
       pub(super) const fn held(&self) -> bool;           // Held only
       pub(super) const fn closed(&self) -> bool;         // Empty only: `settled`'s `roots.is_empty()`
       pub(super) fn reported(&mut self) -> bool;         // Closing: to Held, true; else false
       fn close_next(&mut self) -> Option<Token>;
   }
   pub(super) fn closable(service: &Stages) -> bool;      // roots_closable without its pending test
   pub(super) fn follow(service: &mut Stages);            // machine.close, then close_next when closable
   ```

   | State | `adopt` | `close_next` | `reported` |
   |---|---|---|---|
   | `Empty` | `Held { [root] }`, list of `capacity` | stays, `None` | false |
   | `Held`, some left | push | take the last, `Closing { rest }`, `Some(root)` | false |
   | `Held`, none left | push | `Empty`, `None` | false |
   | `Closing` | `unreachable!` (roots attach before their close) | stays, `None` | true, `Held { roots }` |

   Keep `Held { [] }` apart from `Empty`: it is base's `roots` not yet cleared, which `settled`
   and `work_pending` still see as open, so no pass moves. In `lower_events`, a
   `ROOT_CLOSE_OWNER` event with `reported()` false falls on to `program::file_event`, as it does
   at base. `teardown::work_pending`'s `!roots.is_empty() && roots_closable` becomes
   `roots.held() && roots::closable`.

6. **`Stages`** after these steps: drop the 14 fields named above; add `admission`, `opening` and
   `pub(crate) line`; `roots` changes type. `run_started` stays where it is. `parts` starts them
   `Admission::Waiting`, `Opening::Idle`, `Line::Quiet`, `Roots::Empty`. New modules are declared
   `mod admission; mod opening; mod roots;`. Module docs follow `smith/AGENTS.md`: state kept,
   what is never known, entry points, `Contract: smith/docs/design/protocol/agent.md, section 6`
   in the `//!` block (sections 4–6 for `opening.rs`), never a `/// Contract:` line.

7. **Readers** (nothing else reads these): `service.rs` (`root_to_open`, `root_opened`, `iterate`);
   `stages.rs` (`done`, `failure`, `parts`, `mark_failed`, `refuse_start`, `start_ready`);
   `teardown.rs`; `up.rs` (`lower_events`; in `channel_event`: `Start`, `Message`, `Acknowledged`,
   `Cancel`, `Ended`); `down.rs` (the flush reads `admission.run()`, the tail is
   `admission::follow`); `readiness.rs` (`channel_event_ready`, `pending_message_ready`);
   `link.rs` (`admission.admit`); `request.rs`; `effects.rs` (`contract`, `workspace`, `down`);
   `routing.rs` (`record_usage`); `program.rs` (`install`, `fire`, `policy_deadline`).

   Keep these as they are, though they look wrong:

   - `Heard::Ended` before admission calls `teardown::answered` and pushes no cancel, whether the
     Start is attaching or queued; the Start still enters the domain. `Admission` does not change
     on it.
   - A refused Start's `events.answer` reads `run_started` at `Time::ZERO`.
   - `Effects::abort` while `Live` reaches `admission::cancel`, and nothing ever reads the
     `Cancelled` it leaves.

8. **Tests.** Existing tests change only here: `setup.rs`'s `with_admission` and the assert in
   `interrupted_before_admission`, and `teardown::advance` to `teardown::follow` in `tests.rs`.
   `with_streams` stays: those fields stay. Add `mod lifecycles;` to `service/tests.rs` and these
   7 tests in `service/tests/lifecycles.rs`, on `setup`'s helpers:

   - `a_cancel_before_the_handle_is_delivered_once_after_it`: `Waiting`, `Cancelled`, `Owed`,
     `Admitted`, one `domain::Event::Cancel`; a second `domain_down` pushes none.
   - `a_waiting_cancel_goes_before_the_messages_that_waited_for_the_run`: `Heard::Cancel` and a
     `host_message` on `service()`, then `domain::Request::Admitted` pushed on `domain_requests`
     and `domain_down`: `domain_events` holds the `Cancel`, the message is still pending, and the
     next `domain_down` brings it. The channel is not started there, so `send_admitted` marks
     `Failure::Send`; assert nothing on that.
   - `the_first_failure_stands_in_every_phase`: rows for `Live`, `Answered`, `Closing`:
     `mark_failed(Send)` then `mark_failed(WriteFailed)`; `failure()` reads `Send` and the phase's
     answer is `Unsent(Send)` once answered.
   - `a_failure_before_the_answer_leaves_the_run_live`: after `mark_failed`, `follow` requests no
     close, `done` is `None`, and a `Shutdown` still cancels the admitted run.
   - `each_settled_answer_gives_its_last_line`: rows for `Parked`, `Failed` (the shape in
     `tests/accounts.rs`), `Refused`, and an unsent answer, each through `terminal::remember`,
     `teardown::answered` and `follow` to `Closed` on `without_channel()`, as
     `done_is_some_only_when_closed` does; `crate::last` reads `Quiet`, `Failed`, `Failed`,
     `Unanswered`.
   - `roots_close_last_first_and_one_at_a_time`: the table above on `Roots` alone, including
     `reported` false on `Empty` and `Held`, and `closed` only after the empty `Held` steps to
     `Empty`.
   - `a_start_attaches_its_mounts_in_order_then_enters_the_domain`: `Heard::Start` with two
     mounts through `channel_event` (the `DecodedStart` in
     `a_start_waits_at_the_watermark_before_attaching_its_roots`), then `root_to_open` and
     `root_opened(Ok(fd), host)` twice: the paths come in order, `domain_events` then holds
     `Start` with both directories in order, and `opening.outcome()` is the charter's. A second
     Service with `root_opened(Err(()), ..)` leaves `root_to_open` `None` and the phase past
     `Live`.

9. **Commands.**

   ```
   heavy cargo nextest run -p smith-agent-service --lib
   heavy cargo nextest run -p smith-agent-process-world
   heavy cargo nextest run -p smith-local-service --lib
   heavy cargo nextest run -p smith-local-world
   cargo fmt
   (cd smith && python3 -I -B scripts/old_forms.py)
   heavy cargo clippy -p smith-agent-service --all-targets -- -D warnings
   python3 -I -B tools/citations/citations.py --base origin/main
   ```

   The worlds read what must not move: the agent's exit and last line, and `Effects`' contract
   and closes. No old-form count should move; no design, golden or Markdown read by tests does.
   Commit and push.

## Report

- `Stages`' field, `bool` and `Option` counts after, measured as in Acceptance, and the test
  count selected.
- Each pre-approved option taken, and any cell you made differ from this spec's tables.
- Any change outside Step 7's list, and any existing test touched beyond Step 8.
