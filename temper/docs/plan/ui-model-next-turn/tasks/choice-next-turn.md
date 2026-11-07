+++
id = "choice-next-turn"
part = "next-turn"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-tasks/src/boundary.rs",
  "jig/crates/jig-domain-tasks/src/control.rs",
  "jig/crates/jig-domain-tasks/src/compound/staged.rs",
  "jig/crates/jig-domain-tasks/src/wake.rs",
  "jig/crates/jig-domain-tasks/src/waiting.rs",
  "jig/crates/jig-domain-tasks/src/run.rs",
  "jig/crates/jig-domain-tasks/src/domain.rs",
  "jig/crates/jig-protocol-store/schema/tasks-v9.schema",
  "jig/crates/jig-protocol-store/src/generated/**",
  "jig/crates/jig-protocol-store/src/codec/tasks/record.rs",
  "jig/crates/jig-protocol-store/src/tests/**",
  "jig/crates/jig-protocol-store/golden/tasks/**",
  "jig/tests/tasks/tests/suite/open.rs",
  "jig/tests/tasks/tests/open/llm_change.rs",
  "jig/docs/design/domain/tasks.status.md",
]
effort = "high"
minutes = 110
+++

# A chat's new LLM choice answers the person's next words

## Goal

A change of model or effort on a started chat says it applies from the next turn, and the next turn still runs on the old choice: the amendment and the words after it reach the run that is still listening. Build the rule in jig's tasks child, so that the next words stop that run and start one on the new choice, and the amendment itself wakes nothing and is heard by nobody.

Acceptance, each held by a test in `jig/tests/tasks` (the world `jig-tasks-world`), written first and seen to fail for the rule's reason:

- **An amendment that changes only the LLM choice of a task its requester ends adds no word:** on an idle task the phase stays `Active(Idle)`, nothing is activated, the inbox is unchanged, and the task's history still records `Change::Amended`.
- **On a live run it reaches nothing:** no `Request::Relay` and no `Request::Stop`, the record's `llm` is the new choice, and `llm_changed` is true.
- **The next message that would reach the run stops it instead:** no relay, `Request::Stop` for the attempt once, however many messages follow, and the message stays in the inbox. The run's terminal leaves the task `Active(Due)`, due for that message (`Why::Message`), `tries` none, the activity line `RunEnded` with `EndKind::Parked`, and `llm_changed` false. The next claim's assignment carries the message.
- **A run left alone parks as ever:** its `End::Parked` terminal leaves the task `Active(Idle)`, nothing due, `llm_changed` false; the next message wakes it (`Active(Due)`).
- **Effort alone counts,** and so do a sub-agent's model and the account preference: any change of the `LlmChoice`. An amendment naming the choice the task already holds marks nothing.
- **An amendment that changes the choice and something else** keeps its word and is heard by the next run: the word's wake stops the live run, by the same rule.
- **A task its result ends** is unchanged: its LLM amendment adds its word, relays and wakes as today, and `llm_changed` stays false.
- **An authority narrowing** is unchanged: `an_open_raw_finish_while_narrowing_goes_due` and its `EndKind::Cancelled` line hold.
- **`llm_changed` round-trips the store** and survives a restart: `jig-protocol-store`'s record fixture sets it true.
- The **Not yet** clause in `jig/docs/design/domain/tasks.status.md`, section 7, *a new LLM choice on a task its requester ends*, is gone, its dated line and the other clauses kept. **Departure from your text:** that one clause is yours. `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new broken.

## Inputs

- `temper/docs/plan/ui-model-next-turn/notes/common.md`: the cause as read, and the standing rules.
- `jig/docs/design/domain/tasks.md`, section 7, the bullets *A new LLM choice is for the next run* and *On a task its requester ends, the next run is the next turn*; 5.8 on a task its requester ends; 8.3 on wake rules.
- `jig/docs/design/domain/hosts.md`, 6.2, on a cancel taking the run's tail.
- `jig/crates/jig-protocol-store/schema/tasks-v9.schema`, `TaskRun`, and commit `8097dbddc0`, which extended tasks-v9 in place and shows what regenerates: the codecs through `skein/testing/skein-codegen` (`skein/docs/design/codec.md`), `generated/layouts.rs`, the goldens, the exact size fixtures.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- A `TaskRecord` or `TaskRun` literal, for the new field, `llm_changed: false`: `jig/testing/jig-test-store-files/src/tests/routes/fixtures.rs`, `jig/tests/store/src/root.rs`, `temper/crates/temper-service/src/tests/fixtures.rs`. This is a required field: the temper line is edited only for it.
- `jig/testing/jig-tasks-kit/src/world.rs`: it records stops (`stops`) and drops `Request::Relay`, so add `pub relays: BTreeSet<(u64, u64, u64)>` beside `stops`, by task, attempt and word number, and its one arm. It holds only for the new tests' sight of a relay.
- `jig/testing/jig-tasks-kit/src/referee.rs` or `world.rs`, if the referee or the world assumes every amendment adds a word: only that assumption, for this rule.
- A test in `jig-domain-core`, `jig-core-world`, `temper-engine-domain-world` or `temper-client-world` that says an LLM-only amendment on a task its requester ends wakes it or is heard: its assertion only, to what the design now says. Name each in the report.

**Pre-approved options:**

- **`AmendPlan.prepared`:** default, make it `Option<crate::inbox::Prepared>`, `None` for the word-less amendment, and let the compiler lead you through `control.rs` and `compound/staged.rs`. If a staged path can't take `None` without a wider change, keep it `Some` and carry a bool the apply and staged paths check, and say so.
- **`last_message`:** the root issued the amendment's message number; still set `last_message` to it, word or no word.
- **A compound's amendment:** the rule is the same; if a compound's staged accounting can't drop the word cheaply, keep the word for an amendment in a `Compound` and name it in the report. The picker sends a plain `Amend`.
- **A relay in flight when the amendment commits:** it stays the old run's, as words said before the change.
- **`temper-service`'s or the world's byte or size bounds** that the larger record trips: regenerate or refresh as `8097dbddc0` did; never raise a budget.

**Not yours:** smith; `temper/tests/system` (`next-turn-story`); the picker's words and `temper/crates/temper-client-*`, including its reason `Change next run`; the design's text, but the status clause above.

## Steps

**The tests first.** A new file `jig/tests/tasks/tests/open/llm_change.rs`, joined by `#[path = "../open/llm_change.rs"] mod llm_change;` in `suite/open.rs`, beside `resume`, using `claimed_task`, `commit_turn`, `say` and `raw_finish` from `super` (widen one to `pub(super)` where it needs). A chat is `Flags { conversation: true, ended_by: EndedBy::Requester, ..Flags::PLAIN }`, with `authority.models = Models::Named(0b11)` as `an_open_raw_finish_while_narrowing_goes_due` has it. Amend with `Event::Amend` as that test does, `stop_run: false`, `llm: Some(LlmChoice { account: None, main: ModelEffort { model, effort }, subagents: Box::new([]) })`. Names, for the shape:

```rust
fn a_new_choice_on_an_idle_chat_adds_no_word_and_wakes_nothing()
fn a_new_choice_on_a_listening_chat_reaches_no_run()
fn the_words_after_a_new_choice_stop_the_run_and_wait_for_the_next()
fn more_words_while_the_run_stops_ask_no_second_stop()
fn a_run_left_alone_after_a_new_choice_parks_and_the_next_words_wake_it()
fn a_new_effort_alone_is_a_new_choice()
fn an_amendment_naming_the_held_choice_marks_nothing()
fn a_new_choice_with_new_instructions_is_heard_by_the_next_run()
fn a_finite_tasks_new_choice_is_heard_as_ever()
fn a_run_behind_its_choice_is_still_behind_after_a_restart() // w.restart(), then a message stops it
```

**The field.** In `TaskRecord`, beside `narrowing`, and in `TaskRun` in the schema, beside `narrowing`, carried by `codec/tasks/record.rs` both ways, false in `domain.rs`'s fresh record:

```rust
/// The task's LLM choice changed while this Open run was live: the next message for it
/// stops the run rather than reach it (domain/tasks.md, section 7).
pub llm_changed: bool,
```

**The rule, the outer shape.** Inside it the choices are yours.

- `control.rs`, the amend plan, where `old` is the record before: the amendment **marks the run** when `old.flags.ended_by == EndedBy::Requester`, `crate::run::run_attempt(&old.phase).is_some()`, and `amendment.llm` is `Some(llm)` with `old.llm.as_ref() != Some(llm)`. It is **word-less** when `ended_by == Requester` and `llm` is its only change: every other field of `Amendment` `None`, no release.
- `control.rs`, the apply: set `llm_changed` when marked. Word-less: skip the inbox, the activity's message line and `wake::after_message`; `save_history` and `publish` stay. The `stop_run` path is untouched.
- `wake.rs`, `after_message`, the arm `Phase::Active(Active::Claimed { attempt } | Active::Running { attempt })`:

  ```rust
  // A run behind its choice hears nothing more: the message starts the next run (domain/tasks.md, section 7).
  if row.llm_changed {
      if !row.narrowing {
          // set narrowing, due_reason = crate::Why::Message { message: word.number }, publish
          out.push(Request::Stop { task: number, attempt });
      }
  } else {
      out.push(Request::Relay { task: number, attempt, previous, word });
  }
  ```

- `waiting.rs`, `next`: `None` while the record's `llm_changed`, so that core's `relays::next` offers such a run nothing more.
- `run.rs`, the terminal: `llm_changed = false` beside `narrowing = false`, before `wake::after_run`, so a natural park wakes on any word left unread. `end_kind` is `Cancelled` only for a narrowing that isn't this rule's: `(narrowed && !old.llm_changed) || closing.is_some()`. Keep the rest of the narrowed path: `End::Parked`, `Active::Due`, the `activate`.

**Checks.** The new tests fail first, then pass. Break each rule once by hand as your text says. Then, since this changes what an amendment means to its readers (a change of meaning runs its readers' tests), the focused tests of `-p jig-domain-tasks -p jig-tasks-world -p jig-protocol-store -p jig-store-world -p jig-domain-core -p jig-core-world -p temper-service -p temper-engine-domain-world -p temper-system-world -p temper-client-world`, through `heavy`, plus `--profile fuzzy` for `-p jig-tasks-world`. `cargo fmt`, the old forms check, clippy on what you changed. Commit and push.

## Report

As your text says, and: what the failing tests showed, each reader you edited, which option you took for `AmendPlan.prepared` and a compound's amendment, and anything the tests showed that differs from `notes/common.md`'s reading of the cause.
