+++
id = "trial-root"
part = "trial-root"
kind = "feature"
needs = ["person-task", "trial-machine", "trial-wire", "trial-charter"]
files = [
  "temper/crates/temper-engine-domain/src/route/trials.rs",
  "temper/tests/engine/domain/tests/suite/trials.rs",
]
effort = "high"
minutes = 150
+++

# The root runs a trial

## Goal

An admitted `Application::Try` runs one task of charter 10, naming no model, in no project, and each thing that ends it
moves its row and is seen on the wire's document. This is where the far side answers: `trial-start` is the meeting point
from the client.

Acceptance, stories in `temper/tests/engine/domain/tests/suite/trials.rs`, failing first, no real host:

- `a_try_runs_one_task_of_charter_ten_naming_no_llm`: no project in the store; the claim's assignment is charter 10, its
  main the first model with a usable account of the person's, its task's `llm` none.
- `its_first_turn_answers_and_ends_the_task`: `Answered { seconds }`; the task ends; no second turn or run is claimed.
- `a_failed_run_fails_it_and_no_retry_starts`: each `Class`; no second attempt.
- `a_minute_fails_it`: for a run waiting with no account and for a run hanging past "Starting", through the world's clock,
  at 60 s and not before. The world advances time; no test sleeps.
- `removing_the_account_cancels_it`, whichever of `account` or `took`.
- `a_restart_cancels_it_and_claims_no_run`, and `a_restart_starts_nothing` with an ended row and no live task: no row, no
  task, no spend.
- `a_second_try_spends_nothing`, `again_runs_another`.
- `a_replayed_key_has_the_same_answer_and_spends_nothing`: the same key again, within its retention, answers `Tried` or
  `NotTheirs` as it did, with no second row or task.
- `another_persons_account_is_not_theirs`: `NotTheirs`, no row.
- `it_is_in_no_list`: the person's chats query, inbox, results and Home's counts hold nothing, with a project present.
- `past_starting_moves_its_step`, `took_is_the_account_the_run_keeps`.
- `the_calendar_takes_no_period_of_project_zero`.
- `each_person_reads_their_own_rows`: the document, over the family's watch, newest first.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `temper/docs/plan/model-check/notes/contracts.md`, *The child*, *The
  wire*, *The root*.
- `temper/docs/design/domain/trial.md`, 2 to 6.
- `jig/docs/design/domain/authority.md`, 6.1.
- `temper/docs/design/domain/root.md`: how the root routes; precedents `route/calendar.rs`, `route/forge_route/status_route.rs`.
- `jig/docs/design/domain/engine.md`, 7 and its restart section.

## Owns and may change

Owns, as `files` lists them.

May also change, each by a named hunk:

- `temper/crates/temper-engine-domain/Cargo.toml`: the dependency.
- `temper/crates/temper-engine-domain/src/domain.rs`: the field, its `new`, and `fire_routed`'s `Due`, as the calendar's;
  the `try_answers` table and its `new`.
- `temper/crates/temper-engine-domain/src/deadline.rs`: `next_deadline` takes `trials.next_due()`, while `ready()`.
- `temper/crates/temper-engine-domain/src/route.rs`: `route_core_requests` calls `trials::saved`; `route_into` calls
  `trials::observe` after its work; `Stored` period routing skips project 0; `Work::StartDone`'s cancel.
- `temper/crates/temper-engine-domain/src/route/applications.rs`: `trial-wire`'s stub and its test go; `admitted`'s
  `Application::Try` arm; `answer`'s lookup; one private sweep of expired answers, which `Application::Return`'s arm may
  share.
- `temper/crates/temper-engine-domain/src/route/panels.rs`: the subject, and `Delivery::Trials`.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: `run_workspace`'s one early return.
- `temper/crates/temper-engine-domain/src/boundary.rs`: `Delivery::Trials`, and `TRIALS_SUBJECT` going to the child's.
- `temper/crates/temper-engine-domain/src/limits.rs`, `room.rs`: a bound for the rows, by the deployment's accounts limit.
- `temper/crates/temper-service/src/family.rs`, `tests/family.rs`: encoding the delivery; `Cargo.toml` if needed.
- `temper/tests/engine/domain/tests/suite/main.rs`: a `mod` line; `temper/tests/engine/domain/src/**`: helpers the stories
  need.
- `temper/docs/design/domain/trial.status.md`: from **Not yet**, "no root's part", and only that.

Options, each pre-approved:

- **A start's order:** if `Cancel` is refused during a start, keep to the last point before a claim can be made and test
  that no claim is. If no such point exists, add one `Work` after `Phase::Open` and before the first claim; if a claim comes
  first by the core's order, give `person-task`'s surface a `Core::cancel_before_claims`-style line and say so.
- **The rows' bound** past what the room check takes: use the smaller, say so.

## Steps

```rust
// route/trials.rs: joins the child to the core; holds nothing (domain/trial.md, 4).
pub(crate) const TRIAL_CHARTER: u32 = 10;
pub(crate) fn asked(domain: &mut Domain, env: &Env<Limits>, person: u64, account: u32, again: bool) -> TryAnswer;
pub(crate) fn route(domain: &mut Domain, env: &Env<Limits>, event: trials::Event);
pub(crate) fn saved(domain: &mut Domain, env: &Env<Limits>, record: &core::Record);
pub(crate) fn observe(domain: &mut Domain, env: &Env<Limits>);
pub(crate) fn restarted(domain: &mut Domain);
```

- **`asked`:** `core.accounts.account(account)`'s holder is `Holder::Party(person)`, else `NotTheirs`. Otherwise the
  child's `Try`, and `Tried`, whichever it decides. It returns the answer: the token and its retention stay in
  `route/applications.rs`.
- **`start`:** a `Try` takes the common path to the core, with `carried: None`. A value it carried is delivered
  `CarriedUnused` there, as the stub does.
- **`admitted`'s arm:** expired answers swept, `trials::asked`, the answer kept under `token` until `retention` on both
  clocks, then `finished(domain, request, Ok(token))`.

  ```rust
  // domain.rs, beside `returns` and bounded as it is: `Map::with_capacity(limits.people.requests)`.
  pub(crate) try_answers: Map<Token, (TryAnswer, Time, Wall)>,
  ```

  The insert is `.expect("one marker per retained key")`, as `returns`' is: a key outlives its marker, so the sweep leaves
  room and admission has no refusal of its own. `answer` reads the table beside `returns`, as `ApplicationAnswer::Try`. A
  restart empties it, and a replayed key then has no outcome, as a forge setup's has.
- **`route`'s `Make`:** `core.person_seed(person)`; `None`, `Refused { account }`. Otherwise the seed's `number()`,
  `Work::Core(Event::PersonTask { .. })` with `Executor::Agent { charter: TRIAL_CHARTER }`, the spec `words: b"Say one
  word."`, no job, parameters or inputs, `asks: false`, the contract `Report { words: 64 }`; then `Made`.
- **`route`'s `Cancel`:** `Work::Tasks(tasks::Event::Cancel { reply_to: internal, by: Party::Deployment { project: 0 }, operation })`,
  as `route/forge_route.rs` has it.
- **`route`'s `Changed`:** marks `TRIALS_SUBJECT`, the way `forge::Request::Marked` does.
- **`saved`:** `core::Record::Tasks(tasks::Stored::Activity(row))` for a task the child holds. `Line::Turn` is `Turn`.
  `Line::RunEnded`: `Finished` is `Finished`; `Failed { class }` and `Refused` are `RunFailed`; `Parked` and `Cancelled`
  nothing. An account's `Write::Erase` is `AccountGone`: find its key's form in `jig-domain-accounts/src/records.rs`.
- **`observe`:** for each of `trials.running()`, `core.run_past_starting(task)` and the task's `kept_account`; an `Observed`
  only if either moved. Nothing when none run: this runs on every step.
- **`restarted`:** each live task with `Executor::Agent { charter: TRIAL_CHARTER }`, by `core.tasks.view_tasks()`, `Cancel`.

The workspace's early return: a run of charter 10 answers as `status_route::workspace` does.

**This changes what a charter number means to a shared path:** `run_workspace`'s callers. Run `heavy cargo nextest run -p temper-engine-domain --lib`,
`-p temper-engine-domain-world --test suite`, `-p temper-service --lib`. A new `Delivery` variant forces arms in `temper-service`: those
only.

The checks: fmt, `old_forms.py` for `temper`, clippy on what you changed, the tests above, `heavy cargo check -p temper --tests`.
Citations. Push.

## Report

As the implementer's text asks: each story's bite, where the restart's cancel sits and why, the bound you took.

## Answers

2026-11-06. **`person-task`'s pushed work lacks `Core::run_past_starting` and the arms for `Event::PersonTask` in
`jig-test-service`: finish them there, or widen this task into jig?** There. `person-task` owns both, and its spec now says
so; this task stays temper's. Resume from the pushed `11fd1ef4bb` once `person-task` has merged, and keep its draft story.

- **No stand-in:** no copy of `run_past_starting`'s logic in temper, and no `observe` without it.
- **If an arm is still missing once `person-task` has merged:** add that arm, as `Event::ConnectorRepair`'s beside it, only
  that, and name it in your report. `run_past_starting` still absent then is a new question.

2026-11-06. **May trial-root keep `Try`'s answers under their tokens in a bounded volatile `Domain` table, or does a jig
prerequisite provide a generic mechanism?** The table, in temper. jig's contract already leaves the answer with whoever
owns the application (`finished` checks the owner kept the token), and `Domain::returns` and the forge each keep theirs. No
jig change. Its shape is under *Steps*, and it differs from the run's proposal in two ways:

- **`asked` takes neither the token nor the retention, and returns the answer.** `route/trials.rs` still holds nothing
  (`temper/docs/design/domain/trial.md`, 4); the table is `route/applications.rs`'s.
- **No refusal for room.** The table has one entry per retained key, as `returns` has, so its insert is an `.expect`.

Resume from the pushed `34a77b9020`, and keep its draft story and fixture.
