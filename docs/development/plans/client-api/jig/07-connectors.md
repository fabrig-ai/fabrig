# Session 07: connectors' topics and procedures, the party's own scope and windows, untracking

A connector's side of the engine's plumbing (connectors.md, 1, 2, 5.3 to
5.5, 6.2, 6.3, 8). A run subscribes to a topic a connector offers to
agents, under a read grant on the resources the connector says the topic
concerns, and the topic's news reaches its inbox. A procedure's step is
one decision: a batch, amendments naming the revisions its connector was
told, messages, proposals, and later cancels with a grace, refused as a
unit when one part fails, with what it made or why told back. An
application's watch and read name the party's own scope, and carry their
window to the connector; an application's request names its scope too,
and is admitted by it. A goal an amendment untracks is fed once more,
as its last. The reference root routes each step a test connector's
procedure takes; its panic on a step that delegates or proposes goes.

## Read first

- jig's `docs/design/domain/connectors.md`, 1, 2, 5.3 to 5.5, 6.2, 6.3
  and 8; `docs/design/domain/authority.md`, 8.5 and 9;
  `docs/design/domain/hosts.md`, 8 (the bullet "A connector's topics
  offered to agents"); `docs/design/domain/engine.md`, 4.4 (the rows "a
  procedure's step" and "a connector's news");
  `docs/design/domain/tasks.md`, 7 (amending, cancelling with a grace,
  tracking and untracking); `docs/design/client/domain/contract.md`, 7.5;
  `docs/design/domain/testing.md`, 3 to 7.
- `findings/engine-agents-store-config.md`, sections 1 (the rows on
  topics, procedures, the party's own scope and windows, untracking), 4
  and 5.
- skein's `programming-model.md`, sections 4.4, 4.5, 5.3 and 10;
  `testing-strategy.md`, sections 2.2, 7 and 8.

## What the design asks

- **Topics offered to agents** (connectors.md, 5.3): "A connector says
  which kinds of its topics an agent may subscribe to with `subscribe`
  ... asked, it names the resources a topic concerns, which a read grant
  of the task's must cover". authority.md, 8.5: "A subscription to a
  connector's topic is checked as a read: `subscribe` among the task's
  tools, and a read grant covering the resources the connector says the
  topic concerns." News is classified per subscriber, and reaches the
  task's inbox (engine.md, 4.4).
- **A step is one decision** (connectors.md, 6.3): effects, batches,
  amendments "each naming the revision of the spec it was told", cancels
  "with a reason, and a grace when the step gives one", proposals "while
  the rest of the step commits", messages, a hold, a result. "A step
  whose amendment names a revision that has moved is refused whole, as a
  step that fails any check other than authority is: nothing of it
  commits, its connector is told why, with what the task is now, and the
  procedure steps again on that." "The core tells the procedure's
  connector ... the number of the first task the step made, or that it
  made none, with the revisions of the specs it made or amended; or that
  the step was refused, and why."
- **When it steps** (6.2): a cancel's grace "comes as an amendment does".
- **What clients see** (5.5): a watch and a read "name a project, the
  deployment, or the party's own ... the party's own on the sign-in
  alone"; "a watch of one of its subjects, and a read, carry a window, as
  jig's do; the connector makes the document within it".
- **Untracking** (8; tasks.md, 7): "a goal an amendment untracks is fed
  its state then, as a last feed"; its state "outlives it until its last
  write, made as the goal closes or is untracked, has settled".

## Choices this plan makes

- **A topic call reaches the reference root decoded,** as an effect call
  does (`root::Event::EffectCall`, `testing/jig-test-domain/src/lib.rs:225`):
  `root::Event::TopicCall` names the test connector's topic by its
  number. The reference service declares no host tool for it, as it
  declares none for effects; the joint world sends the event, as it
  sends `EffectCall` (`tests/core/src/effects.rs:420`).
- **The core checks a topic as a read,** reusing `authority::Call::Read`
  once per resource with the `subscribe` family, as
  `Core::connector_read_admit` (`crates/jig-domain-core/src/connector.rs:633`)
  checks a read with the `read` family. Authority does not change.
- **A step's effects stay their own decisions** (`core::Event::EffectStart`
  with `EffectOrigin::Procedure`), as at the base; the other parts of a
  step are one decision. A step that asks for an effect and something
  else is two steps of the test connector's program.
- **What a step made is told within its decision,** as a connector ask
  (`Ask::StepMade`, `Ask::StepRefused`) committed with it, replacing
  `Now::ProcedureDelegateOutcome`, which followed no mutation.
- **The test connector's actions** gain `Amend`, `Message` and, in 07.5,
  `Cancel`; its `Propose` proposes a widening of its task's budget.
- **An application's window is jig's `documents::Window`**
  (`crates/jig-domain-core/src/documents.rs:102`): `rows`, and `after`
  as `Cursor::Number`, the connector's own row number; the connector
  answers `Unknown` for another cursor.
- **The test application's family wire is not changed:** its read
  passes the window its watch passes today
  (`testing/jig-test-service/src/family.rs:185`), and its scopes name no
  party's own; the wire is jig 14's (STATUS, step 3).

## Rules that bind it

- **Files over 800 lines** get at most 40 more lines each, the new code
  in a module beside them (code-organization.md, 7): here
  `testing/jig-test-domain/src/{lib.rs, routes.rs}` (1,108 and 997),
  `crates/jig-domain-core/src/{effects.rs, routing/tasks.rs}` (1,445 and
  1,449), `crates/jig-domain-tasks/src/{boundary.rs, domain.rs}` (1,794
  and 1,275), `examples/ops/domain/src/route.rs` (1,589).
- **The meeting point** (README, Rules; development.md, 5.3): each
  increment's story runs the real core with the test connector in the
  joint world `jig-core-world`, or `ops` in its conformance world; a
  connector's change tested alone, in `jig-test-connector-world`, is no
  Proof here.
- **No name of the repository downstream of jig** in any jig file: the
  check code-organization.md, 6 gives prints nothing at each increment's
  end.
  That repository's root matches `ProcedureAction`,
  `Now::ProcedureDelegateOutcome`, `core::Ask::{Document, Read}` and
  builds `SubscriptionKind::Topic`; it takes these changes at its repins
  (README, Rules).

## Increments

### 07.1 Topics offered to agents, and their news

Scope: a run's call for a topic its connector offers to agents is
checked as a read (authority.md, 8.5) and subscribes its task
(connectors.md, 5.3); unsubscribing, and the task's end, end it at the
connector; a topic's news reaches each subscriber's inbox with its class
(engine.md, 4.4). The reference root and the test connector route each
of these.
Needs: nothing.
Files:
- `crates/jig-domain-core/src/routing/{vocabulary.rs, mod.rs}`, a new
  module `routing/topics.rs`, `connector.rs` (`Core::topic_admit`),
  `effects.rs:400-405` (`connector::Event::News` routed to
  `Core::connector_news`, `connector.rs:439`, instead of dropped),
  `stopping.rs` (the new event's arm);
- `testing/jig-test-connector/src/{boundary.rs, domain.rs, tests.rs}`:
  `TopicSpec` gains `offered: bool` and `resources: Box<[Path]>`,
  `Event::DescribeTopic`, `Request::Topic` and `Request::TopicRefused`;
  the users of `TopicSpec { .. }`: `testing/jig-test-connector/src/tests.rs`,
  `tests/test-connector/src/world.rs:79`, and the core world's
  configuration (`tests/core/src/world.rs:220`, `config`);
- `testing/jig-test-domain/src/{lib.rs, routes.rs}` and a new module
  `topics.rs`: `Event::TopicCall`; `core::Ask::{SubscriptionDone,
  UnsubscriptionDone, EndTopic, DropSubscription}` (today no-ops,
  `routes.rs:264-267`) routed to the connector's `Subscribe` and
  `Unsubscribe` and answered (`core::Event::NamedAnswer` with
  `Subscribed`/`Unsubscribed`, from connector 1 only, as `ops` answers
  once, `examples/ops/domain/src/route.rs:587-615`);
  `connector::Request::News` (`routes.rs:751`, today dropped) routed
  per subscriber;
- the arms the new variants need: `root::Event` matches in
  `testing/jig-test-domain/src/admission.rs`,
  `testing/jig-test-service/src/clients/translation.rs`; `core::Event`
  matches in `crates/jig-domain-core/src/{routing/mod.rs, stopping.rs}`,
  `testing/jig-test-domain/src/admission.rs:21`,
  `testing/jig-test-service/src/clients/translation.rs:165, 237`,
  `tests/client-protocol/src/engine.rs:358`.
Shape:
- `core::Event::SubscribeTopic { to: ReplyTo, key: CallKey, connector:
  u16, topic: u64, resources: Box<[authority::Effect]> }`: the run's
  call, its topic in the connector's number, the resources as read
  descriptions (`authority::Effect`, as a read is described).
- `Core::topic_admit(&self, key: CallKey, resources: &[authority::Effect],
  findings: &mut Queue<authority::Finding>) -> authority::Answer`:
  `authority::check_call` with `family: settings.tools.subscribe` and
  `Call::Read(resource)` for each resource, the first answer other than
  `Allow` returned; no resource, `Refuse`.
- `routing/topics.rs`: not current proof, dropped as an effect call is;
  denied, `CallPart::ToolDenied { answer, findings }`; allowed, the
  present subscribe route with `SubscriptionKind::Topic { connector,
  topic }` (`routing/runs.rs:447-470`, `tasks::Event::SubscribeTopic`),
  which asks `Ask::SubscriptionDone` of the connectors as today.
- `jig_test_domain::Event::TopicCall { to: ReplyTo, key: CallKey,
  number: u16, topic: u16 }`; the root keeps `(to, key)` under a token,
  sends `connector::Event::DescribeTopic { token, topic }`; on
  `Request::Topic { token, resources }` it sends the core
  `SubscribeTopic` with each resource described for a read; on
  `Request::TopicRefused { token }` it answers the call `CallPart::
  SubscriptionRefused(Problem { why: Refusal::Subscription, .. })`
  through `core::Event::NamedAnswer`.
- News: for each `Classed { task, class }` of `Request::News { topic,
  subscribers }`, the root finds the task's subscription to `(number,
  topic)` in `domain.core.tasks.task(task)`'s `subscriptions`, and sends
  `core::Event::EffectConnector(connector::Event::News { task,
  subscription, topic, class, words })`, the words `News on topic 3`;
  a task with no such subscription gets none.
Test shape:
- `testing/jig-test-connector/src/tests.rs` (511 lines):
  `an_offered_topic_names_the_resources_it_concerns`,
  `a_topic_not_offered_to_agents_is_refused`. Under 20 ms each.
- A new file `tests/core/tests/topics.rs`, the world `World::new`
  (`tests/core/src/effects/constructors.rs:71`) with connector one's
  topic 1 offered on `jig_test_connector_world::path(1, 1)`; limits
  from the world's fixture (`tests/core/src/world.rs:17`), none set by
  hand. The reference test:

  ```rust
  #[test]
  fn a_topic_call_under_a_read_grant_subscribes_the_task() {
      let mut world = topic_world(181, true);
      world.send(root::Event::TopicCall {
          to: ReplyTo::new(Token::new(801)),
          key: world.call_key(1),
          number: 1,
          topic: 1,
      });
      assert!(matches!(
          world.answers.last(),
          Some((_, _, core::CallPart::Subscribed { .. }))
      ));
      let task = world.call_key(1).task;
      assert!(live_row(&world, task).subscriptions.iter().any(|subscription| {
          subscription.kind == tasks::SubscriptionKind::Topic { connector: 1, topic: 1 }
      }));
  }
  ```

  where `topic_world(seed, grant)` configures the topic and, with
  `grant`, the caller's read grant on the resource, and `live_row`
  moves from `tests/core/tests/stories.rs:916` into the world's
  fixtures module, `tests/core/src/fixtures.rs`, where both files use
  it; and
  `a_topic_call_without_a_read_grant_is_denied_naming_the_grant`,
  `a_topic_not_offered_to_agents_is_refused_to_the_run`,
  `news_on_a_subscribed_topic_reaches_the_tasks_inbox`,
  `dropped_news_reaches_no_inbox`,
  `unsubscribing_a_topic_ends_it_at_the_connector`. Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(news_on_a_subscribed_topic_reaches_the_tasks_inbox)'` once: a
run's topic call under its grant subscribes its task, the fake system's
news is classified by the test connector and lands in the task's inbox
as `MessageKind::News`, through the real core, tasks child and store;
under 0.5 s.
Don't: change `authority`; add a topic kind to jig's `subscribe` tool
schema; change `ops`'s routes (it keeps calling `connector_news` from
its root).
Size: at most 250 lines.
Removes: the drop of `connector::Event::News` at `effects.rs:400-405`.

### 07.2 A procedure's step is one decision, told back

Scope: a step's decision holds a batch, amendments naming the spec's
revision the connector was told, messages, and its end (wait, a hold,
a result), and commits as one decision, or nothing of it does; a step refused is
told why, with what the task is now; a step that commits is told the
first task it made and the spec revisions it made or amended
(connectors.md, 6.3; engine.md, 4.4).
Needs: nothing beyond 7.1.
Files:
- `crates/jig-domain-core/src/delegation.rs:36-44` (`ProcedureAction`
  becomes `ProcedureStep`), `routing/run_delegation.rs:359-580`,
  `routing/{vocabulary.rs, mod.rs:541}`, a new module
  `routing/procedure_steps.rs` (the checks of amendments and messages),
  `effects.rs` (`ProcedureDecision` arms, at most 40 lines);
- `crates/jig-domain-tasks/src/{boundary.rs:277-286, procedure.rs}`
  (`ProcedureDecision` becomes the step's parts; checked before any is
  applied), `control.rs` (the amendment and message checks the
  procedure route reuses), `domain.rs` (at most 40 lines);
- the users of `ProcedureAction::` and `Event::ProcedureStep`:
  `examples/ops/domain/src/route.rs` (14 and 8 lines; a new module
  `examples/ops/domain/src/steps.rs` takes the translation),
  `testing/jig-test-domain/src/routes.rs:621-660`,
  `tests/client/src/joint/role_stories/family_stories.rs`,
  `tests/core/src/{effects/constructors.rs, world.rs}`,
  `tests/core/tests/stories.rs`, `testing/jig-test-service/src/clients/translation.rs`,
  `tests/client-protocol/src/engine.rs`, `crates/jig-domain-core/src/stopping.rs`;
- the users of `tasks::ProcedureDecision::`:
  `crates/jig-domain-core/src/{effects.rs, routing/run_delegation.rs,
  tests/acts/proposals.rs}`, `crates/jig-domain-tasks/src/procedure.rs`,
  `tests/tasks/tests/{inbox.rs, lifecycle.rs, procedures.rs}`;
- the users of `Now::ProcedureDelegateOutcome` (replaced by the two asks):
  `crates/jig-domain-core/src/{routing/run_delegation.rs,
  tests/completion_retry.rs}`, `crates/jig-protocol-store/src/historical.rs:47, 107`,
  `examples/ops/domain/src/route.rs:1351`,
  `testing/jig-test-domain/src/routes.rs:507`,
  `testing/jig-test-service/src/{clients/translation.rs:86, route.rs:206}`,
  `testing/jig-test-store/src/route.rs:173`,
  `tests/client-protocol/src/engine.rs:306`,
  `tests/client/src/joint/session.rs`, `tests/core/src/client_peer.rs`,
  `tests/ops/src/joint/{engine.rs, front.rs}`;
- `testing/jig-test-connector/src/{boundary.rs, domain.rs:189-192,
  473}` (`Event::ProcedureDelegateOutcome` becomes `Event::StepMade` and
  `Event::StepRefused`); `testing/jig-test-store/schema/test-connector-v1.schema:45`
  and `src/records.rs` (`RecordProcedureDelegateOutcome` gains the
  revisions; the test application's store, edited in place).
Shape:
- `core::ProcedureStep { batch: Box<[Delegate]>, amendments:
  Box<[StepAmendment]>, messages: Box<[StepMessage]>, end: StepEnd }`
  (sketch); `core::Event::ProcedureStep { task, step, connector, code,
  decision: ProcedureStep }`.
- The route: the step's task due at that step, as today; each amendment
  admitted as a run's is (`Core::task_amend_admit`, the procedure's task
  as the amender) and its revision compared (`Core::spec_revision`);
  each message's target referenced (tasks.md, 8.5); the first failure
  refuses the step with nothing of it sent to the tasks child:
  `Ask::StepRefused { task, step, why: StepRefusal }`; then the batch's
  holdings, as today; then one `tasks::Event::Procedure { decision }`
  with the parts.
- `tasks::ProcedureDecision { batch: Box<[New]>, amendments:
  Box<[ProcedureAmendment]>, messages: Box<[ProcedureMessage]>, end:
  ProcedureEnd }` (`ProcedureEnd::{Wait, Hold(Hold), Result(TaskResult)}`;
  `tasks::ProcedureMessage { task: u64, word: Word }`, the message's
  target beside the `Word`, which has none, `boundary.rs:138`;
  `tasks::ProcedureAmendment { task: u64, message: u64, stop_run: bool,
  amendment: Amendment }`, the root-issued message number and stop that
  `control.rs`'s `amend` admits a run's amendment with; both filled by
  the core; ruling 115):
  the tasks child checks each part before it changes any, then applies
  them in that order; one failing check answers `Request::ProcedureRefused
  { task, step, problem }` and changes nothing.
- Told back, in the step's decision: `Ask::StepMade { task, step, first:
  Option<u64>, revisions: Box<[(u64, u64)]> }` (each made or amended
  task with its spec revision after the step), `Ask::StepRefused { task,
  step, why: StepRefusal }`, `StepRefusal::{Moved { task, now:
  people::Moved }, Refused(tasks::Problem), Denied { answer, findings }}`.
- `ops`: its watch's decisions become `ProcedureStep { batch, end: Wait,
  .. }` or `{ end: Hold(..), .. }`; its connector hears `StepMade` where
  it heard `ProcedureDelegateOutcome`.
- The reference root (`testing/jig-test-domain/src/steps.rs`, new):
  `StepDecision::{Finish, Wait, Stall, Hold}` as today; `Delegate {
  kinds }` becomes a batch, one agent task per kind with charter `kind`,
  words `procedure <task> step <step>`, a report of 128 words and the
  authority the procedure's task gives (as `tests/core/tests/stories.rs:67-81`'s
  `member` builds one); `Propose` stays for 7.3, refused as a held step
  (`StepEnd::Hold(Hold::Procedure)`), no panic.
Test shape:
- `tests/tasks/tests/procedures.rs` (43 lines; its `step` helper, line
  7): `a_step_amending_and_messaging_commits_as_one_decision`,
  `a_step_with_one_refused_part_changes_nothing`. Under 0.5 s each.
- A new file `tests/core/tests/procedure_steps.rs`, the world as
  `a_plan_of_reports_and_a_procedure_is_revised_midway_and_done`
  (`tests/core/tests/stories.rs:89`) makes one, its `plan_fixture`
  (`jig_core_world::faults::plan_fixture`) for limits:
  `a_procedure_hears_the_first_task_its_step_made_with_its_revision`,
  `a_step_naming_a_moved_revision_is_refused_and_told_what_it_is_now`.
  Under 0.5 s each.
Proof: conformance, `cargo nextest run -p jig-ops-world -E
'test(the_alert_storm_reaches_triage_and_the_person_accepts_the_production_restart)'`
once (it passes at the base): `ops`'s standing watch makes its triage
task through `ProcedureStep` and hears it made, through `ops`'s real
root and core; under 0.5 s. Its amendments are shown by 7.3's joint
story, the first that a connector's program can make.
Don't: put a step's effects in `ProcedureStep`; send
`Ask::ProcedureMade`, `ProcedureActivate`, `ProcedureMessage` or
`ProcedureClose` (`StartProcedure` carries activation and the inbox, as
today); let a procedure release a hold or decide a proposal or an
escalation (connectors.md, 6.3).
Size: at most 600 lines (ruling 136; 280 before it: the staged admission, its two roots and the callers' sweep).
Removes: `ProcedureAction`, `Now::ProcedureDelegateOutcome`, the
reference root's `unreachable!` at `routes.rs:646`.

### 07.3 The test connector's procedures amend, message and propose

Scope: the test connector's programs amend their delegate naming the
revision they were told and step again on a moved refusal, message their
delegate, and propose a widening of their task's budget; a step's
proposals are admitted while the rest of the step commits (connectors.md,
6.3; authority.md, 9); the reference root routes each.
Needs: nothing beyond 7.2.
Files:
- `testing/jig-test-connector/src/{boundary.rs, procedures.rs,
  domain.rs, tests.rs}`: `ProcedureAction` gains `Amend { words: u64 }`
  and `Message { words: u64 }`; `Propose { number }` becomes `Propose {
  spend: u64 }`; `ProcedureState` keeps its delegate and the spec
  revision it was told (`delegate: Option<u64>, revision: u64`);
  `testing/jig-test-store/schema/test-connector-v1.schema:7` and
  `src/records.rs` (edited in place);
- the users of `ProcedureAction::Propose` and `StepDecision::Propose`:
  `testing/jig-test-connector/src/tests.rs:90, 192`,
  `tests/test-connector/src/world.rs:95`,
  `tests/test-connector/tests/stories.rs:159`;
- `crates/jig-domain-core/src/routing/procedure_steps.rs` (proposals),
  `delegation.rs` (`ProcedureStep.proposals`);
- `testing/jig-test-domain/src/steps.rs`.
Shape:
- `ProcedureStep` gains `proposals: Box<[tasks::ProposalAction]>`
  (a batch, an amendment, a widening; an effect's proposal stays the
  effect route's); each is admitted as `named_propose` admits a run's
  (`routing/runs.rs:246-297`), the procedure's task the proposer; one
  refused as "no proposal could be accepted" refuses the step
  (`StepRefusal::Refused`), one beyond authority is proposed and the rest
  commits.
- The admitted proposals cross into the tasks child on
  `tasks::ProcedureDecision` (`crates/jig-domain-tasks/src/boundary.rs:277`),
  which gains `proposals: Box<[tasks::Proposal]>` beside `amendments` and
  `messages` (ruling 152): `tasks::procedure` checks and installs them
  with the step's other parts, so a refusal, an existing proposal or an
  inbox reservation refuses the whole step. `ProcedureDecision::new`
  (`:305`) gives `Box::new([])`, so its `..ProcedureDecision::new(..)`
  users change nothing; the whole literals gain `proposals:
  Box::new([])` or the step's: `crates/jig-domain-core/src/routing/procedure_steps.rs:87`,
  `crates/jig-domain-core/src/tests/acts/proposals.rs:80`,
  `tests/tasks/tests/inbox.rs:517`, `tests/tasks/tests/procedures.rs:126`,
  `:147`, `:176` (jig `055b8a547`); a further one gets the same, listed in
  the commit body. temper names no `ProcedureDecision`.
- The test connector: `Amend { words }` decides `StepDecision::Amend {
  task: <its delegate>, revision: <the revision it was told>, words }`;
  on `Event::StepRefused { why: Moved { revision } }` it records the
  revision and its next step amends again; on `Event::StepMade` it
  records each revision it is told. `Message { words }` decides
  `StepDecision::Message { task, words }`. `Propose { spend }` decides
  `StepDecision::Propose { spend }`.
- The reference root translates `Amend` to a `StepAmendment` whose spec
  words are `amended <words>`, `Message` to a `StepMessage` of
  `MessageKind::Words`, `Propose` to a widening of the procedure task's
  budget by `spend`.
Test shape:
- `testing/jig-test-connector/src/tests.rs`:
  `an_amend_action_names_the_revision_it_was_told`,
  `a_moved_refusal_makes_the_next_step_name_the_new_revision`. Under
  20 ms each.
- `tests/core/tests/procedure_steps.rs`, the program configured in
  `configuration.first.procedures` as `fixture` configures one
  (`tests/core/src/effects/constructors.rs:42-50`):
  `a_procedure_amends_its_delegate_naming_the_revision_it_was_told`,
  `a_procedure_steps_again_after_its_amendment_is_refused_as_moved`,
  `a_procedure_messages_its_delegate_in_its_step`,
  `a_procedure_proposes_a_widening_while_its_message_commits`. Under
  0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_procedure_steps_again_after_its_amendment_is_refused_as_moved)'`
once: a run amends the procedure's delegate first, the procedure's
amendment naming the older revision is refused with the revision now,
and its next step's amendment commits naming it; under 0.5 s.
Don't: let the test connector infer a delegate's number from the order
tasks were made in (connectors.md, 6.3: "Nothing is inferred").
Size: at most 200 lines.
Removes: nothing.

### 07.4 The party's own scope, and windows to the connector

Scope: an application's watch and read may name the party's own scope,
admitted on the sign-in alone; a watch and a read carry their window to
the connector, which makes the document within it (connectors.md, 5.5;
contract.md, 7.5).
Needs: nothing.
Files:
- `crates/jig-domain-core/src/documents.rs:168-185`
  (`ApplicationScope::Own`), `routing/views.rs` (the watch's admission
  and its `Ask::Document`), `reads.rs:261-270` (the read's admission and
  `Ask::Read`), `reads/vocabulary.rs` (`Read::Application` gains its
  window), `routing/vocabulary.rs:35-40` (`Ask::Document` and `Ask::Read`
  gain `window: documents::Window`), `routing/tasks.rs` (the asks' arms);
- `testing/jig-test-connector/src/client_documents.rs` (`ClientScope::Own`;
  documents within `window.rows`, after `Cursor::Number`);
- `testing/jig-test-domain/src/{client_documents.rs:13, lib.rs,
  routes.rs}` (`Event::ReadApplication` gains `window`);
  `testing/jig-test-service/src/family.rs:150-170` (the read passes
  `Window { rows, after: None }`, as the watch does at line 185);
- the users of `ApplicationScope` that match it: `examples/ops/domain/src/client_documents.rs:41, 47`,
  `testing/jig-scripted-engine/src/application.rs:111`,
  `testing/jig-test-domain/src/client_documents.rs:13`,
  `testing/jig-test-service/src/family.rs:200`,
  `tests/client/src/joint/{packets.rs:249, requests.rs:163}`,
  `tests/client/src/scripted.rs:212`,
  `tests/ops/src/client/transport.rs:123`,
  `tests/ops/src/joint/front.rs:347, 379` (an arm each, at most 5
  lines; `ops`'s connectors show nothing of a party's own, so its arm
  answers the document unknown);
- the users of `Ask::Document {` and `Ask::Read {`:
  `examples/ops/domain/src/route.rs` (its connectors take the window and
  keep their documents as today), `testing/jig-test-domain/src/routes.rs`,
  `crates/jig-domain-core/src/tests/application_documents.rs`.
Shape:
- `ApplicationScope::Own`: "what is the signed-in party's across
  projects"; admitted when the sign-in is live, with no role check;
  `project()` is not called for it (the people child's role check is
  skipped, not given project 0).
- `Ask::Document { watcher, scope, subject, party, window: Window }`,
  `Ask::Read { reply, scope, read, party, window: Window }`,
  `Read::Application { scope, connector, read, window: Window }`.
- The test connector: `ClientScope::Own` shows what `Deployment` shows
  (its objects are no party's; `ApplicationObject` gains no field), so
  the scope's difference is the core's admission; a document of rows
  holds at most `window.rows`, from the row after `after`.
Test shape:
- `testing/jig-test-connector/src/tests.rs`:
  `a_document_holds_at_most_its_windows_rows`,
  `a_document_starts_after_its_windows_cursor`,
  `an_own_scope_shows_what_the_deployment_scope_shows`. Under 20 ms each.
- `tests/core/tests/application_documents.rs` (172 lines), its `world`
  (line 11) and `received` (line 45):
  `an_own_scope_watch_is_admitted_on_the_sign_in_alone`,
  `a_deployment_scope_watch_without_a_deployment_role_is_refused`,
  `an_application_read_carries_its_window_to_the_connector`. Under
  0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(an_own_scope_watch_is_admitted_on_the_sign_in_alone)'` once: a
party with no deployment role watches a subject in its own scope and
receives the connector's document, through the real people child, views
and store; under 0.5 s.
Don't: change `people::Ask::Application`, which keeps naming a project
until 7.6; change the test application's
family wire or jig's client wire
(jig 14's); give `Own` a project number.
Size: at most 150 lines.
Removes: nothing.

### 07.5 Cancels with a grace in a step; untracking

Scope: a step's cancels of its delegates, with a reason and a grace
when the step gives one; a procedure asked to finish by a grace steps
as on an amendment (connectors.md, 6.2 and 6.3; tasks.md, 7); a step's
amendment of a priority alone names no revision (connectors.md, 6.3:
"each naming the revision of the spec it was told, but for a change of
priority alone"; it needs 4.2's `tracked`, so it is here rather than in
7.2); a goal an amendment untracks is fed its state once more, as its
last feed, and its projection is kept until that write settles
(connectors.md, 8).
Needs: jig 04.5 (its 4.1's `Control::Cancel { reason, grace, message }`
and the tasks child's grace bound; its 4.2's `Amendment.tracked`; its
4.5's party's tracking amendment, which untracks a goal).
Files:
- `crates/jig-domain-core/src/{delegation.rs, routing/procedure_steps.rs}`
  (`ProcedureStep.cancels`), `crates/jig-domain-tasks/src/procedure.rs`
  (the cancels, checked with the rest), and where jig 04.1 delivers a
  grace to a task's executor (the procedure's wake, as an amendment's);
- `crates/jig-domain-core/src/projections.rs:248-320`
  (`finish_decision`: a projection whose task is no longer tracked feeds
  `closing: true` once);
- `testing/jig-test-connector/src/{boundary.rs, procedures.rs}`
  (`ProcedureAction::Cancel { grace: Duration }`, its delegate);
  `testing/jig-test-domain/src/steps.rs`.
Shape:
- `ProcedureStep` gains `cancels: Box<[StepCancel]>`, `StepCancel {
  task: u64, reason: Box<[u8]>, grace: Option<Duration> }`, checked as a
  run's cancel is, the grace within the tasks child's bound; the core
  fills the cancel's finish message, as 04.5's party route does.
- A `StepAmendment` whose one change is `tracked` with a priority names
  no revision (`revision: 0`, as jig 04's compound names one) and the
  route compares none for it; the test connector's `Amend` action gains
  `priority: Option<u32>`, which decides such an amendment.
- `projections::finish_decision`: `closing` is the goal ended, or its
  task's `tracked` now `None` (`core.tasks.task(number)`); the
  projection is forgotten when its connectors settle, as an ended goal's
  is (`projections.rs:322-360`). No record changes: the untracked state
  is read from the task, so a restart finds it again.
Test shape:
- `tests/core/tests/procedure_steps.rs`:
  `a_procedure_cancels_its_delegate_with_a_grace_it_finishes_within`,
  `a_procedure_asked_to_finish_by_a_grace_steps_as_on_an_amendment`,
  `a_procedures_priority_change_names_no_revision_and_commits_after_a_spec_change`.
- `tests/core/tests/projections.rs` (335 lines):
  `an_untracked_goal_is_fed_its_state_once_as_its_last_feed`,
  `an_untracked_goals_projection_is_forgotten_once_its_write_settles`.
  Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(an_untracked_goal_is_fed_its_state_once_as_its_last_feed)'` once:
a party's amendment untracks a goal and each projecting connector gets
one feed with `closing: true`, then none; under 0.5 s.
Don't: end goals' topics (no goal topic exists at the base, connectors.md,
5.4; nothing to end); add a field to the projection's record.
Size: at most 170 lines.
Removes: nothing.

### 07.6 An application's request names its scope

Scope: the people child admits an application's request on the party's
sign-in and, by the scope the request names as a read or a subject does
(7.4), on their role in its project or in the deployment, or on nothing
more for the party's own; a refusal names their role in the request's
scope (contract.md, 3.3; people.md, 5.1: "by the scope the request
names, their role in its project, their role in the deployment, or
nothing more for the party's own"). The wire's side, the request's
record carrying the scope, is jig 14.1's.
Needs: jig 02.3's people family at version 3 on `main` (README, Rules:
this increment changes a people record).
Files:
- `crates/jig-domain-people/src/`: a new module `scope.rs`
  (`ApplicationScope`, moved from `crates/jig-domain-core/src/documents.rs:168-185`
  with its `project()`), `lib.rs`, `boundary.rs:265`
  (`Ask::Application { scope, connector, digest }`), `domain.rs` (`project`
  at :909, `admission` at :1838, `application_part` at :1765; at most 40
  lines), `capacity.rs:92`;
- `crates/jig-protocol-store/`: `schema/people-v3.schema` (edited in
  place: `AskApplication` names `scope: ApplicationScope`, an enum `{
  project: ScopeProject, deployment, own }`), `src/codec/people/asks.rs:362,
  516`, `src/tests/fixtures.rs:425`, the goldens;
- `crates/jig-domain-core/src/`: `routing/vocabulary.rs:41-49, 354-362`
  (`Event::Application` and `Ask::Application` name `scope`; the ask's
  `role` an `Option<people::Role>`), `routing/application_routes.rs:25-95`,
  `routing/mod.rs:435`, `stopping.rs:53`;
- the roots and connectors: `testing/jig-test-domain/src/{lib.rs:453,
  629, routes.rs:134}` (1,108 and 997 lines: at most 40 more each),
  `testing/jig-test-connector/src/{boundary.rs, domain.rs:173}` (its
  `Event::Application` takes its `ClientScope` and an optional role),
  `testing/jig-test-service/src/clients.rs:425` and its family's request
  (`ApplicationScope::Project(decoded.project)`, the family wire naming a
  project until jig 14.1), `examples/ops/domain/src/route.rs:121, 385`
  (`ops` names `Project` only; another scope is refused as not offered),
  `examples/ops/domain-infrastructure/src/domain.rs:808`;
- the users of `jig_domain_core::documents::ApplicationScope` (25 files
  at the base, `git grep -l ApplicationScope`, the scripted engine's own
  type of that name aside), each to `jig_domain_people::ApplicationScope`:
  in production `crates/jig-domain-core/src/{documents.rs,
  reads/vocabulary.rs, routing/views.rs, routing/vocabulary.rs}`,
  `examples/ops/domain/src/{boundary.rs, client_documents.rs}`,
  `testing/jig-test-domain/src/{client_documents.rs, lib.rs, link.rs}`,
  `testing/jig-test-service/src/family.rs`; the others in tests and
  worlds;
- the users of `people::Ask::Application { .. }` literals:
  `crates/jig-domain-people/src/tests.rs:37, 1233`,
  `crates/jig-domain-core/src/tests.rs:14`,
  `crates/jig-protocol-store/src/tests/fixtures.rs:425`.
Shape:
- `jig_domain_people::ApplicationScope { Project(u32), Deployment, Own }`,
  the type 7.4 left in `documents.rs`, moved with no alias
  (code-organization.md, 2).
- `people::Ask::Application { scope: ApplicationScope, connector: u16,
  digest: [u8; 32] }`.
- Admission (`domain.rs`, `admission`): `Project(p)` needs a role in
  `p`; `Deployment` a role in the deployment (project 0, as the views'
  `scope.project()` reads it); `Own` nothing beyond the live sign-in. A
  refusal is `Refusal::Role { held }`, `held` the role the party holds in
  the request's scope (`None` for a party with a project role asking in
  the deployment). The pending flight of an `Own` request is kept under
  project 0.
- `core::Event::Application { .., scope, .. }` in place of `project`;
  `core::Ask::Application { request, token, owner, person, scope, role:
  Option<people::Role>, retention }`: an `Own` request reaches its
  connector with no role.
Test shape:
- A new module `crates/jig-domain-people/src/tests/application_scope.rs`
  (`mod application_scope;` in `tests.rs`, 1,596 lines), using its
  parent's `Test`, `identity`, `reply` (`tests.rs:136, 247, 316`) and
  `LIMITS`; limits as the module's `LIMITS`, none set by hand. The
  reference test:

  ```rust
  #[test]
  fn a_deployment_application_request_without_a_deployment_role_is_refused() {
      let mut test = Test::new(LIMITS);
      test.signin(1, 10, identity(0, 1));
      test.send(Event::Roles { project: 1, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });
      let ask = Ask::Application { scope: ApplicationScope::Deployment, connector: 7, digest: [8; 32] };
      assert_eq!(
          reply(&test.request(10, 1, ask)),
          Reply::Outcome(Outcome::Refused(Refusal::Role { held: None }))
      );
  }
  ```

  and `a_project_application_request_is_admitted_on_a_role_in_that_project`,
  `a_project_application_request_without_a_role_there_is_refused`,
  `a_deployment_application_request_on_a_deployment_role_is_admitted`,
  `an_own_application_request_is_admitted_on_the_sign_in_alone`. Under
  20 ms each.
- `crates/jig-protocol-store/src/tests/codec.rs`:
  `an_application_ask_round_trips_each_scope_through_people_v3`.
- `tests/core/tests/application_documents.rs`:
  `an_own_scope_application_request_reaches_its_connector_with_no_role`.
  Under 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(an_own_scope_application_request_reaches_its_connector_with_no_role)'`
once: a party with no role anywhere sends an application request in its
own scope, and the test connector receives it with no role and answers
it, through the real people child, core and store; under 0.5 s.
Don't: change the wire's records (jig 14.1's); give `Own` a role check.
Size: at most 150 lines.
Removes: `ApplicationScope` in `crates/jig-domain-core/src/documents.rs`
(moved to `jig-domain-people`).

## Interface sketches

Compiled at the base in a scratch module of `jig-domain-core`
(`src/sketch_w4.rs`), bodies `unreachable!("sketch")`, under `cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the new
`Event`, `Ask` and enum variants are written there as their own types.
`StepCancel`'s grace is jig 04.1's form: checked at the ready check.

```rust
// crates/jig-domain-core/src/routing/vocabulary.rs (7.1): a run's call for
// an offered topic, decoded by the application and described by its
// connector; ends in Subscribed, SubscriptionRefused or ToolDenied.
pub enum Event {
    SubscribeTopic {
        to: ReplyTo,
        key: CallKey,
        connector: u16,
        topic: u64,
        /// What the topic concerns, as a read of each is described.
        resources: Box<[authority::Effect]>,
    },
    // ...
}

// crates/jig-domain-core/src/connector.rs (7.1).
impl Core {
    /// `subscribe` among the task's tools and a read grant on each resource.
    pub fn topic_admit(
        &self,
        key: CallKey,
        resources: &[authority::Effect],
        findings: &mut Queue<authority::Finding>,
    ) -> authority::Answer;
}

// crates/jig-domain-core/src/delegation.rs (7.2, 7.3, 7.5): one step's
// decision, from its connector through the root; ends in StepMade or
// StepRefused, asked of the connector in the same decision.
pub struct ProcedureStep {
    pub batch: Box<[Delegate]>,
    pub amendments: Box<[StepAmendment]>,
    pub cancels: Box<[StepCancel]>,
    pub messages: Box<[StepMessage]>,
    /// Beyond the task's authority; proposed while the rest commits.
    pub proposals: Box<[tasks::ProposalAction]>,
    pub end: StepEnd,
}
/// An amendment naming the spec's revision the connector was told.
pub struct StepAmendment { pub task: u64, pub revision: u64, pub amendment: tasks::Amendment }
/// A cancel of a delegate and its tree; the grace asks it to finish first.
pub struct StepCancel { pub task: u64, pub reason: Box<[u8]>, pub grace: Option<Duration> }
/// Words, a question or an answer, to a delegate or a referenced task; or up.
pub struct StepMessage { pub target: u64, pub kind: tasks::MessageKind, pub words: Box<[u8]> }
/// What the step leaves its task in.
pub enum StepEnd { Wait, Hold(tasks::Hold), Result(tasks::TaskResult) }

// crates/jig-domain-core/src/routing/vocabulary.rs (7.2): told back.
pub enum Ask {
    /// The step committed: the first task it made, and each spec revision it made or amended.
    StepMade { task: u64, step: u64, first: Option<u64>, revisions: Box<[(u64, u64)]> },
    /// Nothing of the step committed, and why.
    StepRefused { task: u64, step: u64, why: StepRefusal },
    /// (7.4) The views' window travels to the connector.
    Document { watcher: Token, scope: ApplicationScope, subject: Token, party: u64, window: Window },
    Read { reply: ReplyTo, scope: ApplicationScope, read: Token, party: u64, window: Window },
    // ...
}
pub enum StepRefusal {
    /// An amendment named a revision that has moved: what the task is now.
    Moved { task: u64, now: jig_domain_people::Moved },
    /// A part failed a check other than authority.
    Refused(tasks::Problem),
    /// A part beyond authority that may not be proposed.
    Denied { answer: authority::Answer, findings: Box<[authority::Finding]> },
}

// crates/jig-domain-core/src/documents.rs (7.4); moved by 7.6 to
// crates/jig-domain-people/src/scope.rs.
pub enum ApplicationScope {
    Project(u32),
    Deployment,
    /// What is the signed-in party's across projects; admitted on the sign-in alone.
    Own,
}

// crates/jig-domain-people/src/boundary.rs (7.6): an application's request,
// admitted by its scope; ends in the connector's outcome or a refusal.
pub enum Ask {
    Application { scope: ApplicationScope, connector: u16, digest: [u8; 32] },
    // ... the other variants unchanged.
}
```

## Questions to raise when reached

- **A step's effects in its one decision** (connectors.md, 6.3 lists
  effects first among a step's parts): this plan keeps a step's effect
  its own decision, as at the base. If a story needs an effect and an
  amendment in one commit, raise it.
- **An offered topic's description** (connectors.md, 5.3: "each with
  what a call names it by"): the reference application names a topic by
  the connector's number; whether jig's `subscribe` tool declares a
  topic kind, or each application declares its own tool (hosts.md, 8:
  "declared by the application with its connector's host tools"), is
  the designer's; this plan builds the core's side for either.
- **The news's words**: the test connector's news carries no words;
  the reference root writes `News on topic 3`. An application's words
  are its own.

## Done when

- 7.1 to 7.6 have landed on jig's `origin/main`, each with its Proof's
  test passing once in its landing's gate.
- `git -C ~/src/rust/jig grep -n ProcedureDelegateOutcome origin/main --
  crates testing examples` prints nothing, and `git -C ~/src/rust/jig
  grep -n -w ProcedureAction origin/main -- crates examples` prints
  nothing (the test connector keeps its own `ProcedureAction`, its
  program's actions).
- `git -C ~/src/rust/jig grep -n 'effect and delegation scripts enter'
  origin/main` prints nothing.
