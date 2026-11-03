+++
id = "relay-up"
part = "relay"
kind = "feature"
needs = ["relay-surface", "relay-policy", "questions-call"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/**", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/tests/**", "temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/relay.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/relay.rs", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/v11.rs", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/lib.rs", "temper/crates/temper-protocol-store/src/tests/**", "temper/tests/engine/domain/tests/suite/forge_routes/questions.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/forge.status.md", "temper/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 120
+++

# relay-up: a change relays its agents' questions to its requester

## Goal

A change made with questions `up` relays its agents' questions (`domain/forge.md`, 8.11):
its producing, repairs and resolutions keep the Message family and ask it; each step takes
their questions onto its record, passes the first up to its requester in the question form,
kept by its own question, and relays the requester's answer down, held while the change is
kept by a pending dependency; past its bound it holds, and released it passes the question
up. The root applies task `relay-policy`'s `relay` (contract C7) at every change step.
`change` accepts `"questions": "up"`; `"foreman"` stays refused until task `relay-foreman`.

Acceptance:

- **The record keeps the words** (contract C2, revision 2): `Open.words` after `line`, in
  the pure crate, the store (`record Open`'s `words: bytes 65536`, store v11 in place,
  regenerated as `relay-surface` did) and the translations; `relay` fills it from
  `Asked.words` whole. Every literal of `Open` gains it.
- **The call:** `change_batch`'s refusal narrows to `Foreman`, its words "change's questions
  foreman is not offered yet."; `chat_change` writes parameter 14 = 2 for `Up` (contract C3).
  A change whose requester is no task (a person, a deployment) keeps route `None` at
  `start_change`, whatever parameter 14 says.
- **Its agents may ask:** `change_delegate` keeps `domain.core.settings.tools.message` in the
  authority of a `Produce`, `Repair` or `Resolve` delegate when the row's route is not
  `None` (settings stays removed); gates and every delegate of a route-`None` change are as
  today.
- **Taken at its step:** where `change_replies` answers its inbox, a change whose route is not
  `None`:
  - takes each `Question` from the task of its `row.delegate` while that delegate is
    `Produce`, `Repair` or `Resolve` as C7's `Asked { asker, number: word.number, words }`
    instead of the canned reply; one past `QUESTIONS_OPEN` open questions gets the canned
    reply;
  - takes an `Answer { .. }` from its requester task, while the first open question is
    `Asked`, as `heard.answer` (its words); any other answer or words, and questions from
    any other task, are answered as today.
- **Heard and applied:** `heard.kept` is `landing_route::kept(domain, env, task)`;
  `heard.ended` the open questions' askers whose tasks have ended; `heard.released` true at
  the first step after a release of `Hold::Questions`; `room` the step's `tasks.batch` less
  the replies it already holds. The acts become the step's: `Answer` a `StepMessage` to the
  asker, `MessageKind::Answer { question: number }`, the words whole; `Ask` a `StepMessage`
  to the requester task, `MessageKind::Question`, its words `Open.words`, then a blank line
  and the reasons when there are any, cut from the end to `tasks.message_bytes` and marked
  `…`; `Cancel` a `StepCancel` (reason "its asker ended"); `Hold` the change held at its
  step as the pure step's `Decision::Hold(Hold::Questions)` is applied, so its row is
  `Held { was, why: Questions }` and `hold_change_decision` gives C1's words. Until task
  `relay-foreman`, a `Foreman` act is settled at once in the same step as
  `Settled::Failed(b"no foreman is offered yet.")`, so the question goes up.
- **Saved with the step:** the relay's `questions` reach the row in the step's commit (a
  connector event, `temper_engine_domain_forge::Event::Relayed { task, questions }`, applied
  as the other row events are, marking the panel's subject), so a retried step relays once
  and a restart resumes each question where it stood. `Relayed.more` makes the change due
  again at once.
- **Status:** `agent.status.md`'s entry "4.3.1, 8, 8.2: ..." becomes "8: a change's foremen:
  `change` refuses questions `foreman`"; `forge.status.md`'s "8.1, 8.11: ..." becomes
  "8.11: a change's foremen: a question is passed up, never given a foreman"; remove
  `store.status.md`'s sentence on the `Change` family's questions.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 (the paragraph "A change made with questions
  `none`" and the replies it keeps) and 8.11 (all but "A foreman per question");
  `temper/docs/design/domain/agent.md`, 8.2 (its first bullet) and section 8's table.
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C1 to C3, C6, C7,
  and C2's revision 2.
- jig's `jig/docs/design/domain/tasks.md`, 5.7 ("Its own question keeps it too") and 8.2:
  the change's own question keeps it, its credit's answerer is its requester.

## Owns and may change

Owns the files in the header.

- `route/forge_route.rs`: `change_batch`, `chat_change`, `start_change`'s route, `change_delegate`,
  `change_replies` and the step assembly that calls it (`append_change_replies` and its
  callers), each hunk to the relay; a new private module `forge_route/relay.rs` for the
  root's half is yours to make.
- Readers forced by `Open.words` or `Event::Relayed`: relay-policy's tests, the store's round
  trips, the connector's and the world's fixtures; each edited only for the field or arm.
- If `tasks.batch` is 1 where a story needs an answer down and the next question up in one
  step, the story shows them over two steps by `more`; never raise the limit.
- If jig refuses the change's question to its requester (no answer room, the inbox full),
  the change holds as a failed effect does, its words jig's refusal, and the report says
  which refusal; never drop the question.
- If `landing_route::kept` also counts the change's own question as keeping it, use the
  dependencies alone (its `waiting_on`) and say so.

## Steps

1. The record's words; the call; the Message family; taking and the heard; the acts; the
   event; the status lines.
2. Tests in a new `temper/tests/engine/domain/tests/suite/forge_routes/questions.rs`,
   declared beside `asks_no_one`, a requester task with jig's control family making the
   change by `change` with `"questions": "up"` (the world's `plan_world` or its kin), the
   producer scripted to ask with `message` in the question form (or the world's
   `restore_delegate_question` where a scripted call cannot):
   - `a_change_with_questions_up_keeps_its_agents_message_family` (and a route-`none`
     change's still lack it: `asks_no_one.rs` passes unchanged);
   - `a_producers_question_goes_up_and_the_answer_comes_down` (the requester's inbox holds
     the question's words whole; its answer reaches the producer as the answer to the
     producer's question; the producer's next run goes on);
   - `the_change_merges_nothing_while_its_question_is_up`;
   - `an_answer_with_a_new_dependency_waits_for_it_to_land` (the requester makes a change,
     amends the asked change's `after` to it, answers; the producer hears nothing until the
     new change lands, then the answer);
   - `past_its_bound_the_change_holds_naming_the_question` (a bound of 1 by the world's
     limits, the second question holds the change with C1's words, its requester hears the
     escalation) and `released_it_passes_the_question_up`;
   - `a_question_from_another_task_is_answered_as_before`;
   - `an_asker_that_ends_drops_its_question`;
   - `a_restart_keeps_an_open_question_where_it_stood`;
   - `questions_foreman_is_still_refused_at_the_call`.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::questions::/)'` (check the count), then `-E
   'test(/forge_routes::/)'`, `heavy cargo nextest run -p temper-engine-domain-forge-change`,
   `-p temper-engine-domain-forge`, `-p temper-protocol-store`, `-p temper-engine-domain
   --lib`, `-p temper-engine-forge-world`, `-p temper-codecs-world` (one at a time). This
   changes what a change's step answers, so its readers' tests run: `heavy cargo nextest run
   -p temper-service --lib -E 'test(/family/) | test(/change/)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-forge-change -p
   temper-protocol-store -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy
   cargo check --tests` for `-p temper-service`, `-p temper-fake` and `-p
   temper-engine-protocol-forge-world`. Commit ("Relay a change's questions to its
   requester") and push.

## Report

As the implementer's text says, naming how a step with `more` is made due again.

## Answers

2026-11-03. Q (run relay-up-1): extend the files to the store's `src/lib.rs` for the
runtime `open_words` binding, or leave it to the store's owner? A: extend; it is this
task's, since its acceptance (the service tests at checked store limits) cannot hold
without it. In `temper-protocol-store/src/lib.rs`, `bounds` gains one field beside
`held_answer`: `open_words: forge.client.op_bytes,` and nothing else in that file. The
schema ceiling stays 65,536 and the store stays v11. Add a store test beside
`a_held_answer_is_bounded_by_the_forge_operation_bytes` asserting `open_words` is the same
bound; set the owned size assertions to that runtime bound; then finish the remaining
checks of step 3 and 4 (the service filter whole, the forge-route group, clippy, the
`check --tests`). `change_tool.rs`'s `questions_foreman_is_not_offered_yet` and
`stored_bytes` stay outside the task: report them under `## Left` as before.
