# Session 06: the agents' tools name revisions

The engine's host tools become a party's operations, as hosts.md, 8 and
engine.md, 7.3 give them: `amend` names the spec's revision the run was
shown, and `release` and `decide` the hold's; a call naming a revision
that has moved is answered moved, with what the task is now, and a done
answer names the revision it made. The revisions a run names are shown
to it first: in its brief's delegates, in the escalations and held
notices relayed to it, and in `delegate`'s answer. `propose` takes an
amendment and a widening, `delegate` and `propose` name what a party's
`Create` names, `decide` is one tool, `withdraw` answers "already
decided, and how", and `cancel` takes a grace. 06.1 moves the store's
core family to version 3 (README, Rules, "Format versions").

## Read first

- jig's `docs/design/domain/hosts.md`, 1, 8 and 9;
  `docs/design/domain/engine.md`, 4.4, 7.2 to 7.4 and 9;
  `docs/design/domain/authority.md`, 8.5 and 9;
  `docs/design/domain/tasks.md`, 7 and 9 (what an amendment, a release
  and a proposal are; read only these two sections);
  `docs/design/protocol/store.md`, 4.4, 5.1 and 5.3;
  `docs/design/domain/testing.md`, 4 and 6.
- `findings/engine-agents-store-config.md`, sections 1, 2 and 4 (the
  rows on `amend`, `release`, `withdraw`, cancel's grace, the brief and
  `delegate`).
- `decisions.md`, 9 (one bump per family).
- skein's `programming-model.md`, sections 4.4, 4.5 and 10;
  `testing-strategy.md`, sections 2.1, 2.2 and 7.

## What the design asks

- **The tools are a party's operations** (engine.md, 7.3; hosts.md, 8).
  The table's answers, quoted: `amend`, `release`: "done, with the
  revision it made; refused, and why; moved, with what it is now";
  `decide`: "done; beyond its authority, and passed up; moved, with what
  it is now"; `withdraw`: "withdrawn; already decided, and how";
  `delegate`: "their numbers and their specs' revisions, and what a
  budget given `AsAllowed` was given".
- **A call names what it saw** (engine.md, 7.3): "An amendment, a
  release and a decision about an escalation name the revision the run
  was shown ... One whose object has moved since is answered moved, with
  what it is now, and the run decides again on that; a change of
  priority alone names none".
- **A revision is shown before it is named** (hosts.md, 8): in "its
  brief's section on its delegates (engine.md, section 9), the messages
  relayed to it (section 9), and the answers of `delegate` and `amend`".
  hosts.md, 9: "a notice of a delegate held, and an escalation, carry
  the hold's revision".
- **The inputs** (hosts.md, 8): `release`: "the task, its hold's
  revision, and, for an effect held uncertain, whether to make it
  again"; `cancel`: "the task, a reason, and a grace, if the run gives
  one"; `decide`: "a proposal by its number, or an escalation with its
  hold's revision, and the choice, with a reason"; `propose`: "a batch;
  an amendment, naming the revision as `amend` does; an effect of a kind
  a connector offers to agents; or a widening of the task's own
  authority"; `delegate`: each task "with what a party's `Create`
  names".
- **The core keeps the answers** (store.md, 4.4): a call's part is in the
  core family, whose version moves once (store.md, 5.3; decisions.md, 9).

## Choices this plan makes

- **One "moved" vocabulary.** A run's moved answer carries
  `jig_domain_people::Moved`, the value a party's refusal carries
  (`crates/jig-domain-people/src/decisions.rs:7`): its spec's revision,
  or its hold's revision and reason. No second type for it.
- **"The revision it made", for a release,** is the task's spec revision
  after the commit: the one a following `amend` names (a release moves
  no spec revision; the question below asks the designer).
- **`decide` keeps two vocabularies of choice:** `accept`, `reject`,
  `pass` for a proposal, and `release`, `reject`, `pass` for an
  escalation, as the parties' and the tasks child's do; a proposal is
  still named with its proposer, as the relayed proposal shows both.
- **`introduce` stays a tool.** tasks.md, 8.5 asks that a task introduce
  two tasks it references; engine.md, 7.3's table omits it (a question
  for the designer below). This session neither removes nor changes it.
- **The brief's Lineage, Calls and Waiting sections are not built
  here.** The start's messages already carry the proposals and
  escalations waiting for the run, with their revisions once 06.3
  renders them (`crates/jig-domain-tasks/src/domain.rs:776-800`); the
  other two name no revision. They are a row for `later.md`, which the
  planner adds.
- **Messages to runs are rendered in one place,**
  `jig_protocol_tools::words`, replacing the three copies of
  `inbox_words` (06.3).

## Rules that bind it

- **Format versions** (README, Rules): 06.1 owns the core family's move
  to version 3 and makes it once; 06.2 to 06.5 edit `core-v3.schema` in
  place. 06.3 and 06.4 change tasks records and need jig 02.1's tasks
  v3 (06.4 through jig 03, which needs jig 02).
- **jig 06's stories join one scenario family** (testing.md, 9): 06.1
  makes `agents_tools_scenario_family` in `tests/core/tests/host_tools.rs`
  (a few chosen cuts per story, focused suite) and its sweep over each
  commit in `tests/core/tests/fuzzy_stories.rs` (fuzzy suite); each later
  increment adds its Proof's story to both. Aim: the sweep under 1.5 s;
  past 2 s, it sweeps 06.1's and 06.2's stories only, and the rest keep
  their chosen cuts.
- **Edits outside an increment's Files** (README, Rules): the arm each
  exhaustive match needs for an added `CallPart` or `NamedAction` variant
  (at most 5 lines per match) and the users of a changed type, in their
  new form with each assertion, input, expected value and name kept.
- **No name of the repository downstream of jig** in any jig file: the
  check code-organization.md, 6 gives prints nothing at each increment's
  end.
  That repository builds its own tool decoders, translates `CallPart` in
  its store and matches `NamedAction`; it takes these changes at its
  repins (README, Rules).

## Increments

### 06.1 `amend` names its spec's revision; core v3

Scope: the `amend` tool's input names the spec's revision the run was
shown; the core answers moved, with the task's revision now, when it has
moved, and done, with the spec revision the amendment made (engine.md,
7.3; hosts.md, 8); the store's core family moves to version 3 with the
two new call parts (store.md, 5.1 and 5.3).
Needs: nothing.
Files:
- `crates/jig-protocol-tools/src/{declarations.rs, decode.rs, render.rs}`;
- `crates/jig-domain-core/src/routing/{vocabulary.rs, runs.rs,
  tasks.rs}` (`routing/tasks.rs` is 1,449 lines: at most 40 more,
  code-organization.md, 7), `crates/jig-domain-core/src/{stored.rs,
  lib.rs, effects.rs, sibling_routes.rs}` (the arms of `CallPart` and
  `RoutedCall`; `effects.rs` is 1,445 lines: arms only);
- `crates/jig-protocol-store`: `schema/core-v2.schema` moved to
  `schema/core-v3.schema` (`git mv`, then `family jig-store-core 3`),
  `src/generated/core_v2.rs` moved to `src/generated/core_v3.rs` and
  regenerated, the goldens moved to `golden/core/v3/` and the v2 ones
  gone, as README, Rules ("Format versions") has the owner do,
  `src/{lib.rs, format.rs, codec.rs, sizes.rs,
  transcripts.rs, transcripts/read.rs, codec/core.rs,
  codec/core/calls.rs}`, `src/tests/{codec.rs, commits.rs}`; shared
  hotspots `format.rs` and `codec/` (README, Rules, "Shared files");
- `tests/store-codecs/tests/{drift.rs, memory.rs, fuzzy_decode.rs}`
  (`check("core", 3)`; the version each of `memory.rs:10` and
  `fuzzy_decode.rs:42` picks for "core"; `core_v2` imports);
- the users of `NamedAction::Amend`: `crates/jig-protocol-tools/src/tests.rs:85`,
  `tests/core/tests/stories.rs:99` (the field added, the test unchanged
  otherwise);
- the users of `RoutedCall::Control`: `routing/runs.rs:481, 532`,
  `routing/tasks.rs:440, 564, 610, 1099`, `sibling_routes.rs:451`;
- the exhaustive matches of `CallPart` (search `CallPart::Unavailable`):
  `effects.rs:1314`, `lib.rs:460`, `stored.rs:86`,
  `codec/core/calls.rs:158, 256`, `render.rs:92`;
- the race (testing.md, 5): `testing/jig-conformance/src/scenarios.rs`
  (`Scenario`, `ALL`, `Action`, `program`; shared with any jig 04
  scenario: both add rows), `tests/conformance/src/{scenarios.rs,
  actions.rs}` (the test application's binding),
  `tests/conformance/tests/scenarios.rs`;
- the scenario family: `tests/core/tests/{host_tools.rs, fuzzy_stories.rs}`.
Shape:
- `jig_protocol_tools`: the `amend` schema gains `"revision"` (integer,
  required); `decode` builds `NamedAction::Amend { target, revision,
  amendment }`, a missing revision answered `Problem { field:
  b"revision", why: Why::Missing }`.
- `jig_domain_core::NamedAction::Amend { target: u64, revision: u64,
  amendment: tasks::Amendment }`.
- `routing/runs.rs`, the `Amend` arm: after `task_amend_admit` admits it,
  `Core::spec_revision(target, revision)`
  (`crates/jig-domain-core/src/revisions.rs:51`); `Err(Refusal::Moved {
  now })` answers `CallPart::Moved { task: target, now }`, committing
  nothing but the call's record; other refusals answer as today.
- `RoutedCall::Control(CallKey)` becomes `RoutedCall::Control { key:
  CallKey, target: u64, revised: bool }`; on the tasks child's success,
  `revised` answers `CallPart::Revised { task: target, revision: <the
  task's spec revision after the step> }`, else `CallPart::Controlled`
  (cancel).
- `CallPart` gains `Revised { task: u64, revision: u64 }` and `Moved {
  task: u64, now: jig_domain_people::Moved }` (sketch below); `valid`
  admits `task` in `1..=deployment.tasks`.
- `core-v3.schema` gains, mirroring them field for field:
  `record CallPartRevised { task: u64 revision: u64 }`,
  `record CallPartMoved { task: u64 now: Moved }`, and `Moved`,
  `HoldReason`, `FailureClass` copied from the people schema on main,
  with their records; `codec/core/calls.rs` translates both ways,
  exhaustively.
- `format::current` names `core: 3`.
- `jig_conformance::scenarios::Scenario::RacingDecisions` ("a party's
  and a run's decisions about one task racing", testing.md, 5), in
  `ALL`, with `Action::PartyAmend` and `Action::RunAmend`, each naming
  the spec revision both read before either was sent; its check: one of
  the two committed, the other answered moved (a party's
  `Refusal::Moved`, a run's `CallPart::Moved`), and the task's spec is
  the committed one's.
- `render`: `Revised` reads `Done: task 7 is at spec revision 2`;
  `Moved` with a spec reads `Moved: task 7 is at spec revision 2 now`,
  with a hold `Moved: task 7 is held at hold revision 3 now, for its
  budget` (one phrase per `HoldReason`, as `revisions.rs:72-94` lists
  them), `error: true`.
Test shape:
- `crates/jig-protocol-tools/src/tests.rs`, fixtures in
  `src/tests/fixtures.rs` (`AMEND` gains `"revision":1`; `FIELDS`' amend
  row gains `b"revision"`): `an_amend_call_names_the_spec_revision_it_saw`,
  `an_amend_call_without_a_revision_is_answered_missing`,
  `a_revised_answer_names_the_spec_revision_made`,
  `a_moved_answer_names_the_hold_and_its_reason`. Under 20 ms each.
- `crates/jig-protocol-store/src/tests/codec.rs`, fixtures
  `src/tests/fixtures/calls.rs` (`calls::call`):
  `a_moved_call_part_round_trips_through_core_v3`,
  `a_revised_call_part_round_trips_through_core_v3`; the expected row of
  `a_first_commit_puts_the_format_row_with_its_header`
  (`src/tests/commits.rs:86`) names `core: 3`, its name kept.
- `tests/core/tests/host_tools.rs` (91 lines), the world
  `jig_core_world::effects::World::new` and its helpers `delegate`
  (`tests/core/src/effects.rs:507`), `tool_call`
  (`tests/core/src/protocol.rs:98`) and the file's `answer` (line 23);
  its limits are the world's fixture, `tests/core/src/world.rs:17`,
  which the core world's tests use; this increment sets none by hand.
  The reference test:

  ```rust
  #[test]
  fn an_amend_naming_an_old_spec_revision_is_answered_moved() {
      let mut world = World::new(176, false);
      world.hold_procedures = true;
      world.delegate();
      let task = world.procedure.expect("delegated child");
      let amend = format!(
          r#"{{"target":{task},"revision":0,"amendment":{{"spec":{{"words":"check again"}},"reason":"new evidence"}}}}"#
      );
      world.tool_call(b"amend", amend.as_bytes(), true);
      world.tool_call(b"amend", amend.as_bytes(), true);
      assert_eq!(
          answer(&world),
          &core::SettledAnswer::Host { error: true, body: b"Moved: task 2 is at spec revision 1 now".as_slice().into() }
      );
  }
  ```

  and `an_amend_naming_its_spec_revision_is_done_with_the_next`
  (expects `Done: task 2 is at spec revision 1`). Under 0.5 s each.
- The family, in `tests/core/tests/host_tools.rs`:
  `agents_tools_scenario_family` runs each story of jig 06 uncut and at
  2 chosen cuts, `Cut::Submitted` and `Cut::Durable` of the call's
  commit (`tests/core/src/effects.rs:26`), restarting with
  `World::restart` (`tests/core/src/effects/store.rs:138`) and asking
  the call again under its name: its answer is the uncut one. Under
  0.5 s. Its sweep, in `tests/core/tests/fuzzy_stories.rs`, a new kind
  of test written out in full:

  ```rust
  #[test]
  fn each_commit_cut_of_an_agents_tool_story_answers_its_call_once() {
      for story in jig_core_world::agents_tools::STORIES {
          let commits = (story.run)(&mut World::new(190, false), None);
          for at in 1..=commits {
              for cut in [Cut::Submitted(at), Cut::Durable(at)] {
                  let mut world = World::new(190, false);
                  (story.run)(&mut world, Some(cut));
                  assert_eq!(world.reached_cut, Some(cut), "{} reaches its cut", story.name);
              }
          }
      }
  }
  ```

  where `jig_core_world::agents_tools` (`tests/core/src/agents_tools.rs`,
  new) holds `STORIES`, each a name and a `run(&mut World, Option<Cut>)
  -> u64` that plays the story, asks its call again after a cut, asserts
  the answer the uncut story gets, and returns the commits it made.
  About 5 s at most, aimed under 1.5 s (Rules).
- The race, in `tests/conformance/tests/scenarios.rs`:
  `racing_decisions_chosen_effect_admission_and_completion_cut_ends_the_story`
  (the file's `chosen_cut` helper); the fuzzy
  `every_shared_story_survives_every_commit_and_seeded_random_cuts`
  (`tests/conformance/tests/fuzzy_scenarios.rs:10`) and the focused
  `every_shared_adversarial_story_runs_on_the_testing_application` take
  it through `ALL`, their names kept. Under 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(an_amend_naming_an_old_spec_revision_is_answered_moved)'` once:
a run's second amendment naming revision 0 is answered moved at
revision 1 through the real core, store and protocol stand-in; under
0.5 s.
Don't: change a party's amend route (`routing/party_controls.rs`); add a
revision to `tasks::Amendment`; accept a missing revision for an
amendment (06.5 admits one for a change of priority alone, after jig
04.5).
Size: at most 205 lines (the codec's share within code-organization.md,
7's 2.5 times; 25 of them the race's scenario in
`testing/jig-conformance`).
Removes: `schema/core-v2.schema`, `src/generated/core_v2.rs` and the v2
goldens (moved to version 3).

### 06.2 `release` names its hold's revision and again; one `decide`

Scope: `release` names the hold's revision and, for an effect held
uncertain, whether to make it again, checked as a party's release is
(tasks.md, 5.5 and 7); `decide` is one tool for a proposal or an
escalation, an escalation naming its hold's revision; a revision that
has moved is answered moved; a release of an escalation beyond the run's
authority is passed up and answered so (engine.md, 7.3).
Needs: nothing beyond 6.1.
Files:
- `crates/jig-protocol-tools/src/{declarations.rs, decode.rs,
  render.rs, tests.rs, tests/fixtures.rs}`;
- `crates/jig-domain-core/src/routing/{vocabulary.rs, runs.rs,
  party_controls.rs}`, `crates/jig-domain-core/src/revisions.rs` (the
  release check, moved out of `party_controls.rs:251-282` into
  `Core::release_admit`, used by both), `stored.rs`;
- `crates/jig-protocol-store/schema/core-v3.schema`,
  `src/codec/core/calls.rs`;
- `testing/jig-test-service/src/tools.rs:413` (`b"decide_escalation"`
  leaves the family table);
- the users of `NamedAction::Control` with a release:
  `crates/jig-protocol-tools/src/tests.rs:22` (now `NamedAction::Release`);
- the exhaustive matches of `CallPart`, as 6.1 lists them.
Shape:
- `release` schema: `{"target": u64, "revision": u64, "again":
  boolean?}`; decode: `NamedAction::Release { target, revision, again:
  Option<bool> }`.
- `decide` schema: `oneOf` `{"kind":"proposal","proposer":u64,
  "proposal":u64,"decision":"accept"|"reject"|"pass","reason"?}` and
  `{"kind":"escalation","task":u64,"revision":u64,
  "decision":"release"|"reject"|"pass","reason"?}`; decode builds the
  present `NamedAction::DecideProposal` and `DecideEscalation`. The
  `decide_escalation` declaration and decoder go.
- `Core::release_admit(&mut self, task: u64, revision: u64, again:
  Option<bool>) -> Result<Option<u64>, people::Refusal>` (sketch): the
  checks of `party_controls.rs:251-282` in that order; the party's route
  calls it and keeps its answers.
- `routing/runs.rs`: `NamedAction::Release` calls it; `Moved { now }`
  answers `CallPart::Moved`, `Authority { lacking: Lack::Again {
  required }, .. }` answers `CallPart::Again { task, required }`, other
  refusals `ControlRefused` as the cancel arm does; on success
  `tasks::Event::Control { control: Release { again, message } }` with
  `RoutedCall::Control { revised: true, .. }`.
- `named_escalation` (`routing/runs.rs:300`): a revision other than the
  waiting one answers `CallPart::Moved { task, now: Moved::Hold { .. } }`
  from `Core::hold_revision`; a release whose recipient is not the run
  (today `Refusal::Funding`, `runs.rs:325-327`) passes the escalation to
  `escalation_recipient_after` and answers `CallPart::PassedUp { task }`;
  with no next holder, `EscalationRefused` as today.
- `CallPart` gains `Again { task: u64, required: bool }` and `PassedUp {
  task: u64 }`; `core-v3.schema` gains `CallPartAgain { task: u64
  required: bool }` and `CallPartPassedUp { task: u64 }`.
- `render`: `Again` reads `Refused: task 7 holds an effect whose outcome
  is uncertain; say again: true or false` (required) or `Refused: task 7
  holds no uncertain effect; leave again out`; `PassedUp` reads
  `Beyond your authority: task 7's escalation was passed up`.
Test shape:
- `crates/jig-protocol-tools/src/tests.rs`:
  `a_release_call_names_the_hold_revision_and_again`,
  `a_decide_call_names_an_escalation_by_its_hold_revision`,
  `a_decide_call_names_a_proposal_by_its_proposer_and_number`,
  `decide_escalation_is_no_longer_a_declared_tool`; the schema tests
  `the_release_schema_parses_and_names_its_decoders_fields` and
  `the_decide_schema_parses_and_names_its_decoders_fields` keep their
  names with the new `FIELDS` rows.
- `tests/core/tests/host_tools.rs`, a delegate held by
  `world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold {
  task, why: tasks::Hold::Budget })))`, as
  `a_release_naming_an_old_hold_is_refused_as_moved_saying_what_holds_it_now`
  (`tests/core/tests/stories.rs:936`) holds one:
  `a_release_naming_an_old_hold_revision_is_answered_moved` (expects
  `Moved: task 2 is held at hold revision 2 now, for its budget` after a
  second hold),
  `a_release_naming_its_hold_revision_lifts_the_hold`,
  `a_release_of_an_uncertain_effect_without_again_is_refused`,
  `a_decide_naming_an_old_escalation_revision_is_answered_moved`.
  Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_release_naming_an_old_hold_revision_is_answered_moved)'` once:
a run releasing its delegate with the hold's first revision, after a
second hold, is answered moved and the delegate stays held; under
0.5 s.
Don't: change the parties' release answers or their order of checks;
remove `introduce`; send a release without the tasks child's `Release {
again, message }` as the party's route sends it.
Size: at most 170 lines.
Removes: the `decide_escalation` tool (declaration, decoder, its tests'
rows), and the release check's body in `party_controls.rs` (moved to
`Core::release_admit`).

### 06.3 Revisions shown: notices, escalations, the brief

Scope: a notice of a delegate held carries the hold's revision (hosts.md,
9), and a run's `subscribe` to a task's state may ask to hear its holds
(tasks.md, 8.4: "done, held, or its result"; the tool sets `held: false`
at the base, `decode.rs:77`); the escalations and held notices
relayed to a run, and in its start's messages, read with the task and
the hold's revision; the brief's delegates show each spec's revision and
a held one's hold revision (engine.md, 9); one function renders a
message for a run.
Needs: jig 02.1's tasks family at version 3 (README, Rules: this
increment changes a tasks record).
Files:
- `crates/jig-domain-tasks/src/{boundary.rs, subscriptions.rs,
  domain.rs}` (`boundary.rs` 1,794 and `domain.rs` 1,275 lines: at most
  40 more each);
- `crates/jig-protocol-store/schema/tasks-v3.schema` (edited in place),
  `src/codec/tasks/activity.rs:7-25`, `src/tests/codec/record.rs:114, 144`;
- `crates/jig-domain-core/src/brief_text.rs:280-318`;
- `crates/jig-protocol-tools/src/{lib.rs, words.rs}` (new module),
  `src/{declarations.rs, decode.rs}` (`subscribe`'s `held`), and its
  tests;
- the three copies of `inbox_words`, replaced by the new function:
  `testing/jig-test-domain/src/link.rs:347-380` (and its
  `Cargo.toml`, which gains `jig-protocol-tools`),
  `testing/jig-test-service/src/tools.rs:343-376`,
  `examples/ops/domain/src/assemble.rs:210` (and `route.rs:309`, its
  caller; `route.rs` is 1,589 lines: one line changes);
- the users of `tasks::NoticeState::Held`:
  `crates/jig-protocol-engine-clients/src/translate/objects/question_answer.rs:56`,
  `tests/client/src/joint/wire/activity.rs:86`,
  `tests/tasks/tests/holds.rs:137`;
- the users of `tasks::DelegateState`: `domain.rs:810`,
  `brief_text.rs`.
Shape:
- `subscribe`'s task kind gains `"held"` (boolean, optional, false when
  left out): `SubscriptionKind::Task { target, held, result: true }`.
- `tasks::NoticeState::Held { revision: u64 }`, filled from the held
  task's `revisions.hold` where `notify_state` is called with it
  (`domain.rs:971`, `subscriptions.rs:156`).
- `tasks::DelegateState { task: u64, phase: Phase, revisions: Revisions }`.
- tasks v3: `record NoticeHeld { revision: u64 }`, `enum NoticeState {
  held: NoticeHeld, done, failed, cancelled }`.
- `jig_protocol_tools::words(word: &tasks::Word) -> Box<[u8]>`: the
  label and words a run reads; the labels of `inbox_words` kept
  byte for byte (`Result from task 3:\n`, `Proposal accepted 9:\n`,
  ...), and two added: an escalation reads `Escalation: task 7 is held
  at hold revision 2:\n<words>`, a held notice `Task 7 is held at hold
  revision 2:\n<words>`.
- `brief_text::delegates_read` writes `delegate 7: held, spec revision
  1, hold revision 2` (a held delegate) or `delegate 7: running, spec
  revision 1`.
- The client wire's notice keeps no revision: the translation at
  `question_answer.rs:56` matches `t::NoticeState::Held { .. }` (the
  wire's records are jig 14's; a party reads the hold's revision from
  the task, contract.md, 5.2).
Test shape:
- `crates/jig-protocol-tools/src/tests.rs` (a module `words` beside it
  if the file would pass 800 lines):
  `an_escalation_reads_with_its_task_and_hold_revision`,
  `a_held_notice_reads_with_its_hold_revision`,
  `a_proposal_decision_reads_as_inbox_words_did`. Under 20 ms each.
- `testing/jig-test-domain/src/tests/link.rs`:
  `channel_records_leave_as_the_hub_s_vocabulary` keeps its name and its
  expected `Proposal accepted 9:\nHello`.
- `tests/tasks/tests/holds.rs` (312 lines):
  `a_held_notice_carries_the_holds_revision`.
- `crates/jig-domain-core/src/tests/` (a new module `brief_revisions.rs`;
  `tests.rs` is 2,987 lines):
  `the_briefs_delegates_show_their_spec_and_hold_revisions`.
- `crates/jig-protocol-tools/src/tests.rs`:
  `a_subscribe_call_may_ask_to_hear_a_tasks_holds`.
- `tests/core/tests/host_tools.rs`:
  `a_held_delegates_notice_reaches_its_requester_with_its_hold_revision`:
  the run subscribes to its delegate with `"held":true`, the delegate is
  held as 6.2's tests hold it, and the requester's live row (read from
  `world.store.rows`, as `a_message_call_is_decoded_decided_and_answered_in_words`
  reads its child's) has in its inbox a `MessageKind::Notice { state:
  NoticeState::Held { revision: 1 }, .. }`. Under 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_held_delegates_notice_reaches_its_requester_with_its_hold_revision)'`
once: the requester's inbox holds the notice with the hold's revision
through the real tasks child, core and store; under 0.5 s. The rendered
words are a function nothing above the unit tests reads before the
process tier; the unit tests are their check.
Don't: change the wire's `OwnerNoticeState`; add an inbox section to
the brief; build the Lineage, Calls or Waiting sections.
Size: at most 160 lines added (the three copies removed are not
subtracted).
Removes: `inbox_words` in `testing/jig-test-domain/src/link.rs`,
`testing/jig-test-service/src/tools.rs` and
`examples/ops/domain/src/assemble.rs`.

### 06.4 `propose` amends and widens; `delegate` names what `Create` names

Scope: `propose` takes an amendment naming its spec's revision and a
widening of the run's own authority (hosts.md, 8; authority.md, 9),
and answers "proposed, and to whom" (engine.md, 7.3); a proposed
amendment keeps the revision it named and is checked again at its
acceptance (tasks.md, 9: withdrawn when "it no longer applies");
`delegate` and `propose`'s batch name each task's label, flags,
whether it is tracked and its priority, the resources it will write
(`symbolic_grants`, as today), and a budget as an amount or `AsAllowed`
(hosts.md, 8), `core::Delegate` gaining them here (jig 03 builds its
own member type and leaves `core::Delegate` as it is); `AsAllowed` for
a task's batch resolves to what its creator task has left (authority.md,
7); `delegate` answers each task's number, its spec's revision and what
an `AsAllowed` budget was given (engine.md, 7.3).
Needs: jig 03 (`people::Budget` and `Flags` mirrored here, its proposals'
routes, and its party's amendment proposal, which passes the revision it
names to `ProposalAction::Amend`); jig 02.1's tasks v3 through it.
Files:
- `crates/jig-protocol-tools/src/{declarations.rs, decode.rs,
  nested.rs, render.rs, tests.rs, tests/fixtures.rs}`;
- `crates/jig-domain-core/src/routing/{runs.rs, tasks.rs:411, 561}`,
  `stored.rs:92, 112` (the `Proposed` and `Delegated` checks),
  `delegation.rs:26-35` (`Delegate`), `routing/run_delegation.rs`
  (`tasks::New` from a member's label, flags, tracked and budget, at
  `delegate_validated` and `procedure_step`), a new module
  `crates/jig-domain-core/src/as_allowed.rs` (a task's `AsAllowed`),
  `routing/party_decisions.rs:238` and `routing/runs.rs:123` (the
  acceptance's revision check);
- `crates/jig-domain-tasks/src/proposals.rs` (856 lines: at most 40
  more; `ProposalAction::Amend { task, revision, amendment }`), and the
  users that build it or bind its fields: `stored.rs:199`, `owned.rs:332`,
  `crates/jig-domain-core/src/{documents/objects.rs:395, 856,
  proposals.rs:51, tests.rs:881, 960}`,
  `crates/jig-protocol-store/src/{codec/tasks/proposals.rs:186, 232,
  tests/codec/phase.rs:178}`, jig 03.5's party amendment proposal
  (`tasks::PersonAction::Amend`, built at `routing/propose.rs:217` and
  accepted in `routing/party_decisions.rs:68`'s `goal_decide`, ruling
  203; `routing/party_controls.rs:80` is gone), and the matches that
  bind it with `..` unchanged; for the delegate reply's given amount,
  `lib.rs:260` (`delegating`), `routing/mod.rs:146` and
  `sibling_routes.rs:48` (ruling 203);
- `crates/jig-protocol-store/schema/{core-v3.schema, tasks-v3.schema}`
  (edited in place), `src/codec/core/calls.rs:93, 134, 190, 229`;
- the users of `core::Delegate { .. }` literals: `nested.rs:46`, and in
  the worlds `tests/client/src/joint/{engine_stories.rs:173,
  role_stories/card_stories.rs:44}`, `tests/conformance/src/scenarios.rs:246`,
  `tests/core/src/{charters.rs:65, effects.rs:516,
  effects/credentials.rs:29, peers.rs:510, stories/closing_chat.rs:64,
  world/deliveries.rs:91}`, `tests/core/tests/{activity.rs:62,
  fuzzy_stories.rs:69, projections.rs:123, reads.rs:538, stories.rs:72,
  537}` (the new fields as a plain task's: label 0, `Flags::PLAIN`,
  untracked, `Budget::Named`);
- the users of `CallPart::Proposed`: `effects.rs:1296`, `lib.rs:442`,
  `render.rs:50`, `tests/client/src/joint/role_stories/family_stories.rs:192`,
  `tests/conformance/src/scenarios.rs:337`, `tests/core/src/effects.rs:466`;
- the users of `CallPart::Delegated`: `effects.rs:1308`, `lib.rs:454`,
  `render.rs:56`, `crates/jig-protocol-tools/src/tests.rs:217`,
  `tests/client/src/joint/engine_stories.rs:190`,
  `tests/conformance/src/scenarios.rs:275`,
  `tests/core/src/observations.rs:714`,
  `tests/core/src/stories/closing_chat.rs:86`,
  `tests/core/src/world/deliveries.rs:131`,
  `tests/core/tests/{activity.rs:77, fuzzy_stories.rs:84, stories.rs:61}`,
  `tests/ops/src/conformance/observations.rs:186` (each reads `.task`
  where it read the number).
Shape:
- `propose` schema: `action` is `"batch"`, `"amend"` (`target`,
  `revision`, `amendment` as `amend` takes them) or `"widen"`
  (`authority`, as `amend`'s amendment names one); decode builds
  `NamedAction::Propose { action: ProposalAction::Amend { .. } | Widen {
  task: <the caller>, authority }, reason, as_holder: false }` in the
  form jig 03 leaves `ProposalAction`.
- `named_propose` (`routing/runs.rs:246`): an amendment's revision is
  checked by `Core::spec_revision` before the proposal is admitted;
  moved answers `CallPart::Moved`. `tasks::ProposalAction::Amend { task,
  revision, amendment }` keeps it; an acceptance (a run's,
  `runs.rs:123`, or a party's, `party_decisions.rs:238`) compares it
  first, and a moved one ends the proposal withdrawn, with the existing
  `WithdrawProposal` or `WithdrawPersonProposal` (tasks.md: "Withdrawn
  ... when the state it was made against has changed so that it no
  longer applies"; no new tasks input, ruling 203), and answers the
  accepter moved (`CallPart::Moved`, or the party's `Refusal::Moved`)
  once the child's reply returns. tasks v3: `ProposedAmendment` gains
  `revision: u64`, so a party's `PersonAction::Amend` carries it too,
  checked by `party_amend_revision` at `goal_decide` (ruling 203).
- `CallPart::Proposed { proposal: u64, holder: tasks::ProposalHolder }`,
  the holder it waits for when admitted; core v3: `record
  CallPartProposed { proposal: u64 holder: ProposalHolder }` with
  `ProposalHolder` and `ProposalKind` mirrored. `render`: `Proposed 9,
  to task 3` / `to person 4` / `to the project's role for amendments`.
- `core::Delegate` gains `tracked: Option<u32>`, `label: u32`, `flags:
  tasks::Flags`, `budget: jig_domain_people::Budget` (sketch);
  `Budget::AsAllowed { cap }` resolves, before the batch's check, to
  what the creator task has left (its numbers' budget less spent and
  reserved, less the batch's named amounts), at most `cap`; giving
  nothing refuses the batch as lacking budget (tasks.md, 4); the task
  keeps the amount.
- `nested::delegates`: each member's `label` (integer), `flags`
  (`{"conversation":bool,"listed":bool,"retention":"kept"|"forgotten_once_read"}`),
  `tracked` (priority, optional), and `budget`: absent for the
  authority's spend, or `{"as_allowed":{"cap":u64?}}`.
- `CallPart::Delegated(Box<[Made]>)`, `Made { task: u64, revision: u64,
  given: Option<u64> }` (sketch); `core-v3.schema`: `record Made { task:
  u64 revision: u64 given: option u64 }`, `CallPartDelegated { value:
  list 1024 Made }`.
- `render`: `Delegated tasks: 7 at spec revision 0; 8 at spec revision
  0, given 40`.
Test shape:
- `crates/jig-protocol-tools/src/tests.rs`:
  `a_propose_call_amends_naming_the_spec_revision`,
  `a_propose_call_widens_its_own_authority`,
  `a_delegate_call_names_label_flags_and_as_allowed`,
  `a_delegated_answer_names_each_spec_revision_and_what_was_given`,
  `a_proposed_answer_names_the_holder_it_waits_for`. Under 20 ms each.
- `tests/core/tests/host_tools.rs`:
  `a_proposed_amendment_naming_an_old_revision_is_answered_moved`,
  `an_amendment_proposal_accepted_after_its_spec_moved_is_stale`,
  `a_propose_call_answers_its_number_and_to_whom`,
  `a_delegate_given_as_allowed_answers_what_it_was_given`,
  `a_delegate_call_answers_its_tasks_spec_revisions`. Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_delegate_given_as_allowed_answers_what_it_was_given)'` once: a
run's batch with a budget `AsAllowed` is made and answered with the
amount its task had left, through the real authority and tasks children;
under 0.5 s.
Don't: let a call become a proposal on its own (authority.md, 9); add an
`effect` action to `propose` (an effect's proposal stays the
connector's tool, with its `proposal` reason, as today); resolve a
task's `AsAllowed` in `jig-domain-authority` (jig 02.4's `as_allowed` is
a party's).
Size: at most 300 lines; by ruling 203, aim for at most 480 (the three
answers of 01:37Z to 01:43Z included); past it, report why.
Removes: nothing.

### 06.5 `withdraw` answers already decided; `cancel` takes a grace

Scope: `withdraw` of a proposal that is no longer pending answers
"already decided, and how" (engine.md, 7.3); `cancel` takes a grace,
within the tasks child's bound (hosts.md, 8; tasks.md, 7); `amend`
tracks, untracks and sets a priority, and an amendment of the priority
alone names no revision (engine.md, 7.3 and hosts.md, 8: "a change of
priority alone names none"; tasks.md, 7: "the first commit decides").
Needs: jig 03's proposals with a task or a party as proposer, the
decided ones kept, a party's under `tasks::Key::DecidedProposal(number)`,
a task's in its proposer's history row (store.md, 7.3: "a task's: the
row of its proposer's history that keeps it"; ruling 214); jig 04.5 (its
4.1's `Control::Cancel { reason, grace, message }` and the tasks child's
grace bound; its 4.2's `Amendment.tracked: Option<Tracking>`; its 4.5's
projections' room for a tracking amendment).
Files:
- `crates/jig-protocol-tools/src/{declarations.rs, decode.rs,
  render.rs, tests.rs, tests/fixtures.rs}`;
- `crates/jig-domain-core/src/routing/{runs.rs:545-556, vocabulary.rs,
  historical.rs, mod.rs}`;
- `crates/jig-protocol-store/src/historical.rs:7-48` (the get's key),
  and `reads/point.rs:61-90` (the task arm's history lookup, ruling 214)
  and the roots that route `core::Now` to the store's get:
  `testing/jig-test-domain/src/routes.rs:520`,
  `examples/ops/domain/src/route.rs:1346`,
  `testing/jig-test-service/src/{route.rs:158, clients/translation.rs:78}`,
  `testing/jig-test-store/src/route.rs:177`,
  `tests/client-protocol/src/engine.rs:298`,
  `tests/client/src/joint/session.rs:776`,
  `tests/conformance/src/lib.rs:426` (an arm each, beside
  `Now::HistoricalProposal`);
- the users of `Control::Cancel` built by the tool:
  `crates/jig-protocol-tools/src/tests.rs:9`,
  `testing/jig-test-service/src/tests/engine.rs:217`;
- `crates/jig-protocol-tools/src/nested.rs` (the amendment's `tracked`),
  `crates/jig-domain-core/src/routing/runs.rs` (the `Amend` arm).
Shape:
- `cancel` schema gains `"grace"` (integer nanoseconds, optional);
  decode builds jig 04.1's cancel with `grace: Option<Duration>`; the
  core fills `message` with `fresh(Family::Message)`, as 04.5's party
  route does.
- `amend`'s amendment gains `"tracked"`: `{"priority":u32}` or
  `"untracked"`, into `Amendment.tracked`; `"revision"` becomes
  optional: left out, decode admits it only for an amendment whose one
  change is `tracked` with a priority, and builds `revision: 0`, as jig
  04's compound names a priority-only amendment; the `Amend` arm
  compares no revision for it, and its answer `Revised` names the spec
  revision unchanged.
- `routing/runs.rs`, `WithdrawProposal`: a proposal not pending for the
  caller asks `Now::DecidedProposal { to: ReplyTo, key: CallKey,
  proposal: u64 }`; the store answers `Event::DecidedProposal { to,
  key, proposal, row: Option<tasks::Proposal>, busy: bool }`; a row
  whose proposer is the caller answers `CallPart::ProposalDecided {
  proposal, outcome }` from its state, other rows and none answer
  `ProposalRefused(Unknown)`, busy answers `Unavailable`. A task
  proposer's row is the `proposal` of its `Key::History { task, revision }`
  record, the revision found in the owner's `proposal_rows` by
  `kept.proposal == proposal`, as `reads/point.rs:79-88` finds it (ruling
  214); shared with that read, not copied. A proposal no longer in
  `proposal_rows` is the row `None`. No `Stored`, schema, codec, generated
  or golden change.
- `render`: a withdraw's `ProposalDecided` reads `Already decided:
  proposal 9 was accepted` (`rejected`, `withdrawn`, `stale`).
Test shape:
- `crates/jig-protocol-tools/src/tests.rs`:
  `a_cancel_call_gives_a_grace`, `a_cancel_call_without_a_grace_decodes_as_before`,
  `an_amend_of_a_priority_alone_needs_no_revision`,
  `an_amend_changing_the_spec_without_a_revision_is_answered_missing`.
  Under 20 ms each.
- `tests/core/tests/host_tools.rs`:
  `withdrawing_an_accepted_proposal_answers_already_accepted`,
  `withdrawing_an_unknown_proposal_is_refused_unknown`,
  `a_cancel_with_a_grace_asks_the_delegate_to_finish_first`,
  `a_runs_priority_change_without_a_revision_commits_after_a_spec_change`,
  `a_runs_amendment_untracks_its_delegate`. Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(withdrawing_an_accepted_proposal_answers_already_accepted)'`
once: a run withdrawing its proposal after a holder accepted it hears
"Already decided: proposal 3 was accepted", read from the store; under
0.5 s.
Don't: keep decided proposals in memory to answer this; change a party's
`Withdraw`.
Size: at most 150 lines.
Removes: nothing.

## Interface sketches

Compiled at the base in a scratch module of `jig-domain-core`
(`src/sketch_w4.rs`), bodies `unreachable!("sketch")` (the subset denies
`todo!` and `unimplemented!`), under `cargo clippy -p
jig-domain-core --all-targets -- -D warnings`. `Made`, the new
`NamedAction` and `CallPart` variants are written as their own types
there; in the code they are variants of the existing enums. The member
type of 6.4's batch is jig 03's: checked at the ready check.

```rust
// crates/jig-domain-core/src/routing/vocabulary.rs: NamedAction, changed
// and added variants (6.1, 6.2). Sent by the engine's protocol layer;
// each ends in one CallPart.
pub enum NamedAction {
    /// A run's amendment of a task below it, naming its spec's revision
    /// as the run was shown it (engine.md, 7.3).
    Amend { target: u64, revision: u64, amendment: tasks::Amendment },
    /// A run's release of a held task below it, naming its hold's
    /// revision; for an effect held uncertain, whether to make it again.
    Release { target: u64, revision: u64, again: Option<bool> },
    // ... the other variants unchanged.
}

// crates/jig-domain-core/src/stored.rs: CallPart, added variants (core v3).
// Kept with the call's record until a turn carrying it commits.
pub enum CallPart {
    /// An amendment or a release committed: the task's spec revision after it.
    Revised { task: u64, revision: u64 },
    /// The call named a revision that has moved: what the task is now.
    Moved { task: u64, now: jig_domain_people::Moved },
    /// A release named `again` for a task not held uncertain, or left it out for one that is.
    Again { task: u64, required: bool },
    /// A release beyond the run's authority: the escalation went to the next holder.
    PassedUp { task: u64 },
    /// The batch's tasks, in call order (6.4; replaces `Delegated(Box<[u64]>)`).
    Delegated(Box<[Made]>),
    /// The proposal admitted, and the holder it waits for (6.4; replaces `Proposed { proposal }`).
    Proposed { proposal: u64, holder: tasks::ProposalHolder },
    // ... the other variants unchanged.
}

/// One task a `delegate` call made: its number, its spec's revision, and
/// what a budget given `AsAllowed` was given.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Made {
    pub task: u64,
    pub revision: u64,
    pub given: Option<u64>,
}

// crates/jig-domain-tasks/src/proposals.rs (6.4): the amendment proposed keeps
// the spec revision its proposer named, compared when it is accepted.
pub enum ProposalAction {
    Amend { task: u64, revision: u64, amendment: Amendment },
    // ... the other variants unchanged.
}

// crates/jig-domain-core/src/delegation.rs (6.4): a batch member as a
// party's `Create` names one; `budget` and `flags` are jig 03's and jig 02's
// types (checked at the ready check).
pub struct Delegate {
    pub executor: tasks::Executor,
    pub spec: tasks::Spec,
    pub contract: tasks::Contract,
    pub authority: tasks::Authority,
    pub symbolic_grants: Box<[tasks::Grant]>,
    pub dependencies: Box<[Dependency]>,
    pub wake: tasks::WakePolicy,
    /// Its priority among the project's goals when tracked.
    pub tracked: Option<u32>,
    /// The application's opaque label.
    pub label: u32,
    pub flags: tasks::Flags,
    /// `authority.budget.spend` as named, or as much as its creator has left.
    pub budget: jig_domain_people::Budget,
}

// crates/jig-domain-core/src/revisions.rs (6.2): the release check shared by
// a party's route and a run's; the message number when `again` is false.
impl Core {
    pub(crate) fn release_admit(
        &mut self,
        task: u64,
        revision: u64,
        again: Option<bool>,
    ) -> Result<Option<u64>, jig_domain_people::Refusal>;
}

// crates/jig-domain-tasks/src/boundary.rs (6.3).
pub enum NoticeState {
    /// The watched task is held; the hold's revision a decision about it names.
    Held { revision: u64 },
    Done,
    Failed,
    Cancelled,
}
pub struct DelegateState {
    pub task: u64,
    pub phase: Phase,
    /// Its spec's and hold's revisions, as the brief shows them.
    pub revisions: Revisions,
}

// crates/jig-protocol-tools/src/words.rs (6.3): what a run reads of one
// message, its label then its words; the start's messages and the relayed
// ones alike.
pub fn words(word: &jig_domain_tasks::Word) -> Box<[u8]>;
```

## Questions to raise when reached

- **The revision a release made** (engine.md, 7.3: "done, with the
  revision it made"): a release moves no spec revision and leaves the
  hold. This plan answers the spec's revision after the commit; if the
  designer means another revision, 6.2's `Revised` changes before it
  lands.
- **`introduce`** is a tool at the base and tasks.md, 8.5 asks for the
  act, but engine.md, 7.3's table lists no `introduce`: the designer
  adds its row, or says it goes.
- **A proposal's acceptance beyond the holder's authority** (engine.md,
  7.3, `decide`: "beyond its authority, and passed up"): 6.2 passes an
  escalation's release up; the acceptance of a proposal is refused today
  when its funding fails at acceptance (`named_proposal_accept`). Raise it
  if 6.2's tests reach that path.

## Done when

- 6.1 to 6.5 have landed on jig's `origin/main`, each with its Proof's
  test passing once in its landing's gate.
- `git -C ~/src/rust/jig show origin/main:crates/jig-protocol-store/src/format.rs
  | grep -c 'core: 3'` prints 1.
- `git -C ~/src/rust/jig grep -n decide_escalation origin/main --
  crates/jig-protocol-tools` prints nothing.
- `git -C ~/src/rust/jig grep -n 'fn inbox_words' origin/main` prints
  nothing.
