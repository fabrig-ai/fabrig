+++
id = "settings-wake-pace"
part = "settings-wake"
kind = "feature"
needs = ["goal-jig-store", "goal-jig-client"]
files = ["jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-domain-tasks/src/wake.rs", "jig/crates/jig-domain-tasks/src/lib.rs", "jig/crates/jig-domain-tasks/src/control.rs", "jig/crates/jig-domain-tasks/src/tests/deadlines.rs", "jig/crates/jig-domain-people/src/amendment.rs", "jig/crates/jig-domain-core/src/admission/amendments.rs", "jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/src/generated/**", "jig/crates/jig-protocol-store/src/codec/tasks/phase.rs", "jig/crates/jig-protocol-store/src/codec/people/policy.rs", "jig/crates/jig-protocol-store/src/tests/fixtures.rs", "jig/crates/jig-protocol-store/golden/tasks/**", "jig/crates/jig-protocol-store/golden/people/**", "jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/src/tests/fixtures.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-protocol-engine-clients/src/translate/owner.rs", "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs", "jig/crates/jig-client-domain/src/contract/wake.rs", "jig/crates/jig-client-protocol-link/src/translate/asks.rs", "jig/crates/jig-client-protocol-link/src/translate/facts_part1.rs", "jig/crates/jig-client-view/src/tree/fragments/wake.rs", "jig/crates/jig-client-view/src/words/wake_words.rs", "jig/crates/jig-client-view/src/tree/confirmation/amendment.rs", "jig/crates/jig-client-view/src/tests/tree/fragments/wake.rs", "jig/testing/jig-client-scripted-engine/src/codec/values/wake.rs", "jig/tests/tasks/tests/suite/pace.rs", "jig/tests/tasks/tests/suite/main.rs", "jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs", "temper/crates/temper-client-tools/src/jig/render/document/templates.rs", "temper/crates/temper-client-view/src/pages/chats.rs", "jig/docs/design/domain/tasks.status.md"]
effort = "high"
minutes = 120
+++

# settings-wake-pace: a pace in the wake vocabulary

## Goal

jig's closed wake vocabulary gains a pace: `WakeRule::Paced { span }`, "at most once in a
span". A message whose rule is paced wakes an idle agent task at once when its last run
started at least `span` before (or it has never run), and otherwise once `span` since that
start has passed, everything that arrived meanwhile waking it once. The rule crosses every
layer a `WakeRule` crosses today (the tasks and people stores, the client wire, the client's
contract and views), so a task holding it is stored, restored and shown. It is the coordinator's
news rule (contract C7); a charter's default policy is the next task's.

Acceptance:

- `jig_domain_tasks::WakeRule::Paced { span: Duration }`, and the same variant in
  `jig_domain_people::WakeRule` and `jig_client::contract::WakeRule` (C7's names).
- `wake::valid` refuses `Paced` with a zero span (as it refuses a zero batch age):
  `Refusal::Spec` at the batch and the amendment, as for `Batch`. The crate exports it as
  `pub fn wake_valid(policy: &WakePolicy) -> bool` from its root (a charter's policy is
  checked by it in `settings-wake-charter`).
- An idle agent task whose news is `Paced { span }`: waking news wakes it at once when
  `run_since` is `None` or `wall >= run_since + span`; otherwise it stays idle, a wake alarm
  is armed at `run_since + span`, and when it fires the task becomes due once, its inbox's
  paced messages all eligible. A message the pace holds is not relayed to a live run. After a
  restart the alarm is re-armed (`wake::restore`).
- Kinds other than news keep their own rules: a delegate's result under `ResultsWake::Each`
  wakes the task while its news is held, and that run carries the held news.
- The stores (tasks-v9, people-v14) and the client wire (client-v19) carry the variant,
  in place (versions stay fixed until the first release: jig's `protocol/store.md`, section
  5; `protocol/client.md`, "Until the first release, versions stay fixed"), with goldens.
- A paced rule is shown in words: "Wakes at most once in N ms" (the view's wake fragment and
  ctl's template render).

## Inputs

- `jig/docs/design/domain/tasks.md`, 8.3 (the bullet **a pace**, and the paragraph on each
  charter's default): the design this builds.
- `jig/docs/design/domain/tasks.status.md`: its **Not yet** words "8.3: a pace (no
  `WakeRule` paces a kind)", which this task removes (leave the rest of that entry, the
  charter's default, for `settings-wake-charter`).
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C7 (and its line
  under "Names the families write here").

## Owns and may change

Owns the files in the header. The edits there:

- `jig-domain-tasks/src/boundary.rs`: the variant, with a doc line ("At most once in
  `span` from the task's last run's start (domain/tasks.md, 8.3)."). `WakePolicy::DEFAULT`
  is unchanged.
- `jig-domain-tasks/src/wake.rs`: `valid`; in `after_message`, a `WakeRule::Paced { span }`
  arm: ready when `row.run_since` is `None` or `env.wall >= run_since + span`; when ready,
  mark every inbox item of the word's `category` eligible (as the batch arm does) and go on
  as for a ready batch; when not ready, return after `schedule`. In `schedule`, a paced item
  not yet eligible arms the task's alarm at `run_since + span` (the earliest of its batch and
  pace deadlines, the wall-to-now conversion as the batch arm has it); in `fire`, a paced
  item not eligible whose deadline has passed is the candidate, as a batch's is. A procedure
  has no `run_since`: its paced news wakes it at once (say so in a comment).
- Compiler-forced arms, each mapping `Paced` to its counterpart: `jig-domain-people`
  (`amendment.rs`, the variant), `jig-domain-core/src/admission/amendments.rs`
  (`wake_rule`), `jig-domain-tasks/src/tests/deadlines.rs`, the store codecs
  (`codec/tasks/phase.rs`, `codec/people/policy.rs`), the engine-clients translations
  (`owner.rs`, `asks.rs`), the link's (`asks.rs`, `facts_part1.rs`), the view's confirmation
  (`tree/confirmation/amendment.rs`), the scripted engine's codec, the client-wire fuzzy
  test's generator, and temper's `temper-client-tools/.../templates.rs` render and
  `temper-client-view/src/pages/chats.rs` (a paced rule wakes, as `Batch` does there).
- Schemas, in place: `tasks-v9.schema`, `record WakePace { span: duration }` and
  `enum WakeRule { never, immediate, batch: WakeBatch, paced: WakePace }`;
  `people-v14.schema`, `record WakeRulePaced { span: u64 }` beside its `WakeRuleBatch`
  (whose `age` is a `u64` there) and the enum's `paced: WakeRulePaced`; `client-v19.schema`,
  `record WakeRulePaced { span: duration }` and `paced: WakeRulePaced`. Regenerate with the
  drift tests (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world
  --test drift`; the client wire's `drift` in `jig-client-wire-world`'s suite, run the same
  way), which also rewrite `layouts.rs`; add the goldens the generator asks for
  (`enum_wake_rule_paced_*`, `record_wake_pace_*` / `record_wake_rule_paced_*`), and update
  the store's and the wire's fixtures that list every variant.
- `jig-client-view`: a phrase `WakePhrase::Paced`, the next unused id in the wake phrases'
  range, its words "Wakes at most once in " then the span as a number hole then " ns"
  (the batch phrase's unit), its hole kinds checked as the batch phrase's are; the fragment's
  arm; a fragment test. If the phrase table is checked against a catalogue file elsewhere,
  add the phrase there too and name the file in the report.
- Not here: the `delegate` and `amend` tools' JSON (`jig-protocol-tools`,
  `temper-engine-smith`) keep `never`, `immediate` and `batch`: a tool offers no pace in this
  task. If a tool's decoder matches `WakeRule` exhaustively, give it the arm and refuse
  nothing new.
- Other plans: reference-m7a changes `jig-domain-tasks`, `jig-domain-core`,
  `jig-protocol-store` and the client crates on its trunk; keep every hunk to the variant.
  This plan's `goal-jig-store` and `goal-jig-client` regenerate the same generated files
  (`people_v14.rs`, `v19.rs`, `layouts.rs`) and change the same translations, so this task
  starts from their merged work: regenerate from the trunk's schemas, never by hand-merging
  generated code.

## Steps

1. The variant and `valid`; the arms the compiler asks for, crate by crate
   (`heavy cargo check -p jig-domain-tasks`, then each dependent with `--tests`).
2. The schemas and regeneration; goldens and fixtures; the store's and the wire's round
   trips (`heavy cargo nextest run -p jig-protocol-store --lib`, `-p jig-client-protocol-wire
   --lib`).
3. The semantics in `wake.rs`.
4. Stories in a new `jig/tests/tasks/tests/suite/pace.rs` (declared in `main.rs`), each an
   agent task made by `Party::Person(9)` with `wake.news = WakeRule::Paced { span }`
   (`span` 1,000 ns), subscribed to a topic as `procedure_watch` in `inbox.rs` does, claimed
   (`world.claim`, which reports the start) and parked (`world.terminal(1, End::Parked)`):
   - `paced_news_wakes_at_once_after_a_quiet_span`: elapse `span`, send waking news: the
     task is activated once.
   - `paced_news_within_the_span_waits_for_its_end`: send waking news at once: no
     activation; send a second; elapse to `span` since the start: one activation, the inbox
     holding both, eligible.
   - `a_result_wakes_through_a_pace_and_carries_the_news`: the coordinator (task 1, its
     `results` `ResultsWake::Each`) makes a delegate, task 2, as
     `a_coordinator_is_woken_once_by_a_burst` in `inbox.rs` does; news held by the pace,
     then task 2's result: one activation, at the result, the news in task 1's inbox.
   - `a_held_pace_survives_a_restart`: news held, `world.restart()`, elapse to the span's
     end: one activation.
   - `a_zero_span_is_refused`: a batch naming `Paced { span: 0 }` is refused with
     `Refusal::Spec` and makes nothing.
   Narrow: `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(/^pace::/)'`
   (check the count: five).
5. The implementer's text's checks: fmt, old forms, clippy on every package changed, the
   focused tests of `jig-domain-tasks`, `jig-domain-people`, `jig-domain-core`,
   `jig-protocol-store`, `jig-client-protocol-wire`, `jig-protocol-engine-clients`,
   `jig-client-domain`, `jig-client-protocol-link`, `jig-client-view`, `jig-tasks-world`,
   `jig-store-codecs-world`, `jig-client-wire-world`, `temper-client-tools` and
   `temper-client-view`; `heavy cargo check --tests` of the other temper packages that name
   `WakeRule` (`temper-engine-smith`, `temper-service`, `temper-system-world`).
6. Commit ("Add a pace to the wake vocabulary") and push.

## Report

The implementer's text's report, plus: each compiler-forced arm by file, the goldens added,
and the phrase id given to `Paced`.

## Answers

2026-11-02. May this task also change `jig-domain-tasks/src/lib.rs` and `src/control.rs`?
Yes, both added to the header, each to the one edit its acceptance needs and no more:

- `lib.rs`: `pub use wake::valid as wake_valid;` beside the crate's other root exports
  (`settings-wake-charter` calls `jig_domain_tasks::wake_valid`). `bound-verdict-fields` also
  owns `lib.rs`: keep the hunk to that line so the two merge.
- `control.rs`, the amendment's wake check: `Refusal::Read` becomes `Refusal::Spec`, as
  `batch.rs` refuses an invalid wake at the batch. It changes the refusal for an invalid
  `Batch` amendment too, which is intended; no existing test pins `Read` there. Re-enable
  `tests::deadlines::a_zero_pace_amendment_is_refused_without_changing_the_task` in the
  focused run.

The rest of the work stands as pushed in `789f023aee`; rerun the narrow tests of the two
edits, the deadlines tests and the pace stories, then the focused run without the amendment
exclusion. The settings size failure
(`settings::a_revision_item_past_the_single_read_budget_answers_its_encoded_size`) is not
this task's: report it under `## Left` with its log.

