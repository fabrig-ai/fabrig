+++
id = "relay-foreman"
part = "relay"
kind = "feature"
needs = ["relay-up", "foreman-charter"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/**", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/tests/**", "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs", "temper/tests/engine/domain/tests/suite/forge_routes/foremen.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/forge.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 100
+++

# relay-foreman: a foreman per question

## Goal

A change made with questions `foreman` gives each of its agents' questions a foreman of its
own first (`domain/forge.md`, 8.11): a read-only delegate under the foreman's charter
(contract C4), briefed with the change's words, the question and the change's pushed head,
its verdict `answer`, `finish` or `escalate` (C5). `answer` and `finish` answer the asker;
`escalate`, a failed or cancelled foreman, pass the question up as task `relay-up` does.
`change` accepts `"questions": "foreman"`.

Acceptance:

- **The call:** `change_batch` no longer refuses `Foreman`; `chat_change` writes parameter
  14 = 1, gives the change `env.limits.forge.change_policy.questions` more tasks beside its
  six, and grants its procedure delegation to `Charter(FOREMAN_CHARTER)` (C4, C6); a change
  of any other route is granted none.
- **The foreman delegate**, on `RelayAct::Foreman { asker, number }` (replacing `relay-up`'s
  settling at once): a `super::Delegate` in the step's batch, executor `Agent { charter:
  row.foreman_charter }`, its spec's words C5's brief (the change's spec words, the head
  question's `Open.words`, the change's `branch` and `last_head` or "pushed nothing yet"),
  parameters 1 and 2 as C5 says, its authority the review's (`gate_authority`), its contract
  C5's verdict of three choices. Made under the change's authority as its other delegates
  are: one its authority no longer covers settles the question as `Settled::Failed` with
  the authority's words, so it goes up (never a hold of the change for a foreman).
- **Its number** reaches the head question as `Stands::Foreman { task: Some(n) }`, learned
  where the root learns a delegate's (`route/asks.rs`'s `step_made`), saved by the relay's
  event.
- **Its end:** the foreman's result in the change's inbox maps to C7's `Settled`: `Verdict {
  code: 1 }` `Answer(words)`, code 2 `Finish(words)`, code 3 `Escalate(words)`; `Failed` or
  `Cancelled` `Failed(words)`, the failure's words or "the foreman was cancelled". It never
  reaches `change_delegate_status` or `row.delegate`, whose one delegate stays the
  producing, repair or resolution task.
- **Cancelled** with `RelayAct::Cancel`, and with the change as its other delegates are.
- **Status:** remove `agent.status.md`'s "8: a change's foremen ..." entry and
  `forge.status.md`'s "8.11: a change's foremen ..." entry; from `configuration.status.md`'s
  "13.2, 13.4: kind 3's number 3 ..." remove "kind 3's number 3" (its words keeping kind 6's
  clause).

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 ("A foreman per question") and 8.4 ("A step
  its authority no longer covers"); `temper/docs/design/domain/agent.md`, section 8 (the
  foreman's row and paragraph) and 4.3.1 (`questions`, the tasks for foremen);
  `temper/docs/design/protocol/configuration.md`, 13.2 ("The forge's charters are read
  when a change is made").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C4 to C7.

## Owns and may change

Owns the files in the header.

- `route/forge_route.rs` and its modules: `change_batch`, `chat_change`, the relay's root
  half `relay-up` made, and a foreman's delegate builder beside `change_delegate`;
  `route/asks.rs`, `step_made` alone.
- If a `Verdict` contract of three choices is refused by a limit at the change's start (the
  deployment's `tasks.contract_choices`), the world's limits name 3 and the report says
  where; never lower the choices.
- If the brief's change words must be cut to fit, cut from their end and mark them, as
  C5 says; the question's words are never cut.

## Steps

1. The call; the delegate builder; its number; its end; cancels; status lines.
2. Tests in a new `temper/tests/engine/domain/tests/suite/forge_routes/foremen.rs`, declared
  beside `questions`, a requester with jig's control family making the change with
  `"questions": "foreman"`:
   - `a_question_gets_a_foreman_reading_only` (its executor is charter 6, its authority
     holds the read family and the forge's reads and no Message family, its contract has
     the three choices, its brief holds the change's words, the question whole and the
     pushed head);
   - `a_foremans_answer_reaches_the_producer`, and nothing reaches the requester;
   - `a_foremans_finish_tells_the_producer_to_finish`;
   - `a_foremans_escalation_goes_up_with_its_reasons` (the requester's question holds the
     question's words, then `Foreman:` and the foreman's text);
   - `a_failed_foreman_goes_up`;
   - `foremen_count_against_the_bound_and_the_changes_tasks` (four questions, four foremen,
     the fifth holds the change);
   - `an_ended_asker_cancels_its_foreman`;
   - `questions.rs`'s `questions_foreman_is_still_refused_at_the_call` is replaced by
     `questions_foreman_is_offered_with_control`.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::foremen::/)'` (check the count), then `-E
   'test(/forge_routes::/)'`, `heavy cargo nextest run -p temper-engine-domain --lib`, `-p
   temper-engine-domain-forge`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-world
   --all-targets -- -D warnings`; `heavy cargo check -p temper-service --tests`. Commit
   ("Give each relayed question a foreman") and push.

## Report

As the implementer's text says.

## Rebase (2026-11-03, step merge, foreman reference-m5/relay-foreman/foreman-1)

resolve the merge conflicts by preserving held-branch parameters alongside foreman routing and retaining current status text.
The merge failed in two task-owned paths against `origin/reference-m5/integration` (`f5bb3892cc`).

In `temper/crates/temper-engine-domain/src/route/forge_route.rs`, resolve `chat_change` by combining both changes:
- Keep the trunk’s complete `if let Some(branch)` block: parameters 3 (branch), 11 (requester-held branch), and 10 (first-line title).
- Immediately afterward, keep this task’s route match and parameter 14 insertion: Foreman = 1, Up = 2, None = absent.
- Remove the superseded Up-only condition; preserve each block’s refusal handling.
The overlapping trunk changes originate in `87c57fa5fa` (held branch) and `f537ee0c72` (title), integrated by `995ce1601e`.

In `temper/docs/design/protocol/configuration.status.md`, retain the trunk’s current paragraphs and remove only `kind 3's number 3; ` from the “13.2, 13.4” entry.
Keep kind 6’s clause and all newer status wording, including the `.temper/pre-pr` departure; do not restore the branch’s older paragraphs.

Preserve the automatically merged branch/control checks and foreman implementation.
After resolving, run the spec’s foremen tests, full forge_routes tests and required checks; verify that tests select a nonzero count.
