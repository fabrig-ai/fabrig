# Session 04: cancel with a grace, tracked and priority amended, `Move`'s check, the compound request

jig's step 2, the parties' and tasks' half that changes live tasks: the
grace that only a chat's close had becomes any cancel's, kept with the
task on a timer of the tasks child's and bounded by a new tasks limit; a
maintainer amends whether a task is tracked and its priority, within a
new tasks limit, goals per project; `Move` checks the escalation holder;
each control on a live task is split into a check and an apply, so that
a party's compound request admits its operations in order and makes them
in one decision or none; and the compound request reaches the people
child and the core, bounded by a new people limit, operations per
request; a move's conversation read position, and the compound and the
grace as conformance scenarios with a referee rule. The wire is not
changed here: jig 14 carries these requests to the client. The session
starts once jig 02.1 has landed (04.1 and 04.2 change the tasks family
at v3); 04.3 to 04.7 need jig 02.3; no increment needs jig 03, since a
compound holds no `Create`, `Propose`, `Withdraw` or `Decide`. 04.1 is
"jig 04.1's cancel with a grace", which jig 09 uses, and jig 06.5 and
07.5 through 04.5; 04.2 is "jig 04.2's tracked made amendable", which
jig 07.5 uses through 04.5, for untracking.

## Read first

- jig's `docs/design/domain/tasks.md`: 4 ("Checked whole", "A party's
  root has someone to escalate to"), 7, 9 ("Proposing is asked for"),
  11.
- jig's `docs/design/domain/people.md`: 5.1 (the table's `Amend`,
  `Cancel`, `Move` and compound rows, "A compound request", "A party's
  root has someone to escalate to"), 6 (a conversation's read position
  moved by a `Move`).
- jig's `docs/design/domain/core.md`: 6.2 (tracked and untracked), 11
  ("Once, as each kind allows": "every operation of a compound request
  is made, or none is").
- jig's `docs/design/domain/engine.md`: 4.1, 4.4 (the compound's
  route), 13 (its worst case).
- jig's `docs/design/client/domain/contract.md`: 3.4.
- jig's `docs/design/protocol/configuration.md`: 6 ("Operations per
  request", "A template's grace": jig 09's checks, read for the limits'
  names only).
- jig's `docs/design/domain/testing.md`: 3 (tasks and people), 5 (the
  compound and the grace scenarios), 6 ("Once").
- This plan's `findings/engine-parties-and-tasks.md`, 1 ("Cancel with a
  grace", "Move", "The compound request", "Goals per project"), 3, 4;
  `findings/engine-agents-store-config.md`, 3 (the worst case).
- skein's `programming-model.md`: 4.5, 5.4, 6.3, 9; `testing-strategy.md`:
  2.1, 2.2, 3, 7.

## What the design asks

- **A cancel's grace** (tasks.md, 7): "A cancel may give a grace, within
  the longest the tasks child allows (section 11), so that a task
  finishes what it can before it is cut off ... The task hears that it is
  asked to finish, as it hears an amendment (8.1) ... It is cancelled, as
  above and with the cancel's reason, when the grace ends, unless it
  ended first. The grace's end is kept with the task, on a timer of the
  tasks child's, so a restart keeps it. A task that cannot finish on its
  own, waiting or held, is cancelled at once. A cancel made during a
  grace ends the task no later than that grace would: one without a
  grace cancels it at once." The limit is "a grace's span: the longest
  grace a cancel may give" (tasks.md, 11).
- **Tracked and priority amended** (tasks.md, 7; people.md, 5.1): an
  amendment may change "whether it is tracked, within goals per project
  (section 11), and its priority"; `Amend` needs "maintainer, for tracked
  or priority"; "A priority moves no revision, and an amendment of the
  priority alone names none". Goals per project: "the live tracked tasks
  in one project, refused at the entrance like live tasks per project,
  whether a batch, an amendment or an accepted proposal tracks them"
  (tasks.md, 11).
- **`Move`** needs "they may amend the task; a holder for its
  escalations" (people.md, 5.1; tasks.md, 7).
- **The compound request** (people.md, 5.1; tasks.md, 7; contract.md,
  3.4): `Cancel` (with or without a grace), `Move`, `Amend` (priority
  included), `Release`, `Message`, `Stop`, on one project's tasks;
  "admitted in order, each against the state those before it leave and
  needing what it would alone, with one pool counted across them";
  "all made, or refused, naming the first operation that failed and why,
  and nothing of it made"; bounded by operations per request, a people
  limit; "its decision is priced as the limit times its costliest
  operation" (engine.md, 13).

## Choices this plan makes

- **The grace's home.** chat.rs's grace half (the `Grace` record, the
  alarms, `schedule`, `fire`) moves to a new tasks module, `grace.rs`,
  and serves any cancel; the chat's close (`Event::CloseChat`, shaped,
  removed by jig 16) asks for its grace through it and keeps its texts
  ("Finish this chat.", "Chat finish grace expired.").
- **The cancel carries its finish message's number.** Message numbers
  are root-issued, so `Control::Cancel` is `{ reason, grace, message }`,
  `message` given with `grace` and only with it, as `Release`'s
  `message` is given with `again: Some(false)`. A run's tool (jig 06.5)
  decodes `grace` and the core fills `message`.
- **The people side waits for people v3.** `Ask::Cancel`'s grace,
  `people::Amendment`'s tracked, the new `LimitKind`s and the compound's
  ask and outcomes change people records, edited in the people family's
  v3 after its owner's landing, jig 02.3 (README, Rules): they are 04.5
  to 04.7. 04.1 to 04.4 change only the tasks family, whose v3 is jig
  02.1's, and the configuration.
- **Staged state, not a copy.** A compound is checked against what the
  operations before it leave through a small `Staged` record (tasks an
  earlier cancel closes, tasks an earlier move made the party's, the
  pool's reservation so far, revisions an earlier operation moved), and
  applied only after the last check passed; nothing is cloned or rolled
  back.
- **The compound's answer stays `Copy`, numbered by place** (the
  designer's answer to ruling 181's question, 2026-10-16; jig's
  contract.md, 3.4: "Its operations take consecutive message numbers,
  one each, in the request's order, and the answer names the first: the
  operation at place *i* took the first plus *i*"; "one that adds
  nothing leaves its number unused"; "Nothing more of an operation is
  answered"). `people::Outcome` and `Reply` stay `Copy`; the compound's
  answer is `Outcome::Compound { project, operations, first_message }`
  and its refusal `Outcome::CompoundRefused { at, why }`. Every
  operation takes one number in the core's `issue`
  (`jig-domain-core/src/compound.rs`): the operation that adds a message
  to its task's inbox uses it, the others leave it unused, so
  `first_message` is never 0 for a made compound. 04.6 landed numbering
  only the operations that add a message (jig 983862753); a step after
  04.6, the supervisor's, makes it one per operation.
- **Maintainer** means the `Owner` or `Maintainer` role (people.md, 4:
  "owner: everything a maintainer may"); a policy role (numbered above 3)
  does not amend tracked or priority, and the shaped `Ask::Prioritise`
  takes the same check.
- **Untracking's projections are jig 07.5's.** 04.2 makes tracked
  amendable in the tasks child; tracking by amendment opens a goal's
  projection as a tracked `Make` does (`projections::save`); the
  untrack transition in projections and the goals' topics are jig 07.5's
  ("jig 04.2's tracked made amendable").
- **The configuration's check at start** (operations per request at
  least the larger of delegates per task plus one and goals per
  project) is jig 09.4's. The core's own worst case refuses a people
  `operations` above the tasks child's `tasks` (04.6), so that
  `route_room_checked`'s per-decision room, which prices `tasks` times
  (`tree_tasks` + `connectors` + 4), holds a compound.
- **Profiles' values** (configuration.md, 6: numbers live in the
  profiles): tasks `grace` 600 s and `project_goals` 8 in the small
  profile, `project_goals` 16 in the standard one; people `operations`
  8 small, 16 standard.

## Rules that bind it

- **A tracked conversation stays one** (the designer's answer to
  ruling 120's question, 2026-10-16; jig's tasks.md, 7: "Neither touches
  its flags, so a conversation stays one: tracked, its turns are still
  entries for its requester, with their read position and unread count
  ... No tracking is refused for a task's being a conversation"). 04.2
  and 04.5 admit tracking any task, a conversation included, and add no
  refusal, branch or test for its being one. Neither changes
  `Core::chat_requester` (`jig-domain-core/src/inbox.rs:107-121`): jig
  05.5 makes the core know a conversation by its flag
  (`conversation_requester`), and its
  `a_tracked_conversation_keeps_its_read_position` is the test of it. No
  test of this session asserts what a tracked conversation's inbox
  lines, offers or read position show. Until jig 05.5 lands, a chat
  tracked on `main` leaves its party's inbox; no client can track one
  before jig 14 carries the tracked amendment on the wire.
- **A task is asked to finish once** (the designer's answer to ruling
  123's question, 2026-10-16; jig's tasks.md, 7): a cancel with a grace
  made during a grace adds no word, and moves the grace's end, with its
  `by` and reason, only to a sooner end (04.1's Shape). It binds 04.1,
  and 04.5 and 04.6 where they send a cancel with a grace.
- **Format versions** (README, Rules): tasks v3 is jig 02.1's, and
  people v3 its owner's; an increment that changes a family's records
  needs its owner's landing and edits that version in place. Regenerate
  with `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
  jig-store-codecs-world --test drift`, once per increment that edits a
  schema.
- **Files over 800 lines at the base get at most 40 lines each**
  (code-organization.md, 7): here `jig-domain-tasks/src/{boundary.rs
  (1804), domain.rs (1278), stored.rs (1072), proposals.rs (856)}`,
  `jig-domain-people/src/{domain.rs (1884), tests.rs (1596)}`,
  `jig-domain-core/src/{routing/tasks.rs (1459), tests.rs (2993)}`
  (on `main` after jig 02.1). New
  code goes in the new modules this file names.
- **No shape of work grows** (code-organization.md, 6): `Ask::Close`,
  `Ask::Prioritise`, `Event::CloseChat` and `Settings.close_grace` stay
  as they are but for the edits this file names; jig 09 replaces
  `close_grace` with the chat template's grace and jig 16 removes the
  rest.

## Increments

### 04.1 Cancel with a grace in the tasks child

Scope: tasks.md, 7 ("Cancelling with a grace") and 11 ("a grace's
span"): `Control::Cancel` takes a grace and its finish message; the
grace is kept with the task (`TaskRecord.grace`, the former
`chat_close`), with the cancel's maker and reason, on the tasks child's
timer; a waiting or held task is cancelled at once; a cancel during a
grace ends the task no later than that grace; a grace longer than the
new tasks limit `grace` is refused. The chat's close asks for its grace
the same way. The configuration refuses a chat `close_grace_ms` longer
than the tasks child's `grace`.
Needs: jig 02.1's tasks family at version 3.
Files:
- `crates/jig-domain-tasks/src/`: `grace.rs` (new: chat.rs:19-22,
  191-242 moved and generalized), `chat.rs` (its grace half out; `close`
  calls `grace::ask_to_finish`), `control.rs` (`Control::Cancel`),
  `boundary.rs` (`TaskRecord.grace`; `Refusal::Grace`), `domain.rs`
  (`chat_alarm` → `grace_alarm`, `chat_alarms` → `grace_alarms`,
  `chat::fire`/`schedule` → `grace::`), `stored.rs`
  (`valid_chat_close` → `valid_grace`), `closing.rs`, `limits.rs`
  (`grace`), `lib.rs`.
- `crates/jig-protocol-store/`: `schema/tasks-v3.schema` (the record's
  grace: by as a party, until, reason; `Refusal::Grace` in carried
  refusals), `src/codec/tasks/{record.rs, carried.rs, phase.rs}`
  (phase.rs:406-414: `grace`/`read_grace` take `{ by, until }`, `by` a
  u64), the generated family, `src/tests/{codec/record.rs,
  codec/phase.rs (:229-232), fixtures/record.rs}`, the goldens the drift
  test writes.
- `crates/jig-protocol-config/src/`: `profile/small.rs` (`grace:
  Duration::from_secs(600)`, and its module doc line), `overrides.rs`
  (`set_tasks` names `b"grace_ms"`, in milliseconds, as its other
  durations), `charters.rs` (`chats` takes the tasks child's `grace` and
  refuses `close_grace_ms` past it, `What::Bound`), `lib.rs` (:96-97,
  `chats`' only caller, passes `limits.core.tasks.grace`),
  `tests/domain.rs`, `tests/overrides.rs` (the `SMALL` table, :131, a
  copy of the small profile's values: `(b"tasks.grace_ms", 600_000)`).
- `crates/jig-domain-core/src/acts/objects/task.rs:164`
  (`row.grace.is_some()`);
  `crates/jig-domain-core/src/routing/refusals.rs` (`task_answer`, :23,
  matches every `tasks::Refusal`: an arm for `Refusal::Grace`, mapped to
  `people::Refusal::Limit { which: people::LimitKind::Spec }` until 04.5
  points it at `LimitKind::Grace`; people's kinds are people v3's, which
  04.1 does not take; no person's ask carries a grace before 04.5).
- `crates/jig-protocol-tools/src/render.rs` (`task_problem`, :222,
  matches every `tasks::Refusal`: a `Grace` arm).
- Users of `Control::Cancel` (21 in 17 files; edited to `{ reason,
  grace: None, message: None }`): `jig-domain-core/src/routing/
  party_controls.rs`, `jig-domain-core/src/tests/{chip_phase.rs,
  document_offers.rs, last_batch.rs, project_tasks.rs}`,
  `jig-domain-tasks/src/tests/{changed.rs, last_batch.rs, revisions.rs}`,
  `jig-protocol-tools/src/{decode.rs, tests.rs}`,
  `testing/jig-test-service/src/tests/engine.rs`,
  `tests/core/tests/{effect_fences.rs, effects.rs}`,
  `tests/tasks/tests/{lifecycle.rs, lifecycle/goals.rs, memory.rs}`.
- Users of `chat_close` and the chat's `Grace` (40 in 13 files):
  `tests/core/src/stories/closing_chat.rs` (`.grace`; `.grace.clone()`
  at :173 and `.grace.as_ref()` at :217, which move it out of
  `&TaskRecord`; `by: tasks::Party::Person(person)`),
  `tests/store/src/root.rs`,
  `jig-domain-tasks/src/tests/{deadlines.rs, revisions/chat.rs}`, and
  the files above.
- Users of the tasks `Limits` literal (9 files), each gaining `grace:
  Duration::from_secs(600)`, the small profile's value:
  `jig-domain-tasks/src/tests/revisions.rs:21`,
  `tests/tasks/src/world.rs:17`, `tests/core/src/world.rs:19`,
  `tests/store/src/root.rs:23`, `jig-protocol-store/src/tests/fixtures.rs:54`,
  `jig-domain-core/src/tests.rs:31`, and the profile, `overrides.rs` and
  `limits.rs` themselves.
- The values derived from the task record's worst case, which
  `Grace { by: Party, reason }` grows, as jig 02.1's fields did (rulings
  110 and 112): `STORE_KV`'s `value`, `chunk` and `page.bytes` in
  `crates/jig-protocol-config/src/profile/small.rs:305, 313-314` and
  `profile/standard.rs:144, 150-151`; `What::Memory { over }` in
  `crates/jig-protocol-config/src/tests/domain.rs:169`; their copies in
  `testing/jig-test-service/src/tests.rs:24-25`. 04.2, 04.5 and 04.6
  refresh the same values where their records grow (04.6's `operations`
  priced into `ask_bytes` moves the memory sum).
- Downstream: the repository downstream of jig has 1 `Control::Cancel`,
  1 `chat_close`, 1 tasks `Limits` literal, 2 `close_grace` users and a
  match over every `tasks::Refusal`
  (`crates/temper-engine-smith/src/answers.rs:351`, a `Grace` arm),
  absorbed at its repin (README, Rules).
Shape:
- `jig_domain_tasks::Control::Cancel { reason: Box<[u8]>, grace:
  Option<Duration>, message: Option<u64> }`; `message` is `Some` when and only
  when `grace` is, else `Refusal::Read`; `grace` past `Limits::grace`
  is `Refusal::Grace`, nothing changed.
- `jig_domain_tasks::Grace { by: Party, until: Wall, reason: Box<[u8]>,
  message: u64 }` (no longer `Copy`), exported from `grace.rs`;
  `TaskRecord.grace: Option<Grace>` in `chat_close`'s place. `message`
  is the ask to finish's number, the cancel's `message` (ruling 124):
  `inbox::prepare`'s merge (its one key for `MessageKind::Amendment`,
  `inbox.rs:31`) passes over the word of that number, which keeps its
  slot, and `stored.rs`'s count of amendment words (`:395`) and the
  restore admit it beside one ordinary amendment.
- `grace::ask_to_finish(domain, env, number, by, until, message, reason,
  out) -> Result<(), Refusal>`: a task waiting or held is cancelled at
  once (`closing::cancel_tree` with the history `control::apply`
  writes); an active one hears an amendment-kind word (`MessageKind::
  Amendment`, its words the cancel's reason), as `control::amend` builds
  one, and keeps `Grace { by, until, reason, message }`; a person
  task's holder sees it as they see an amendment; `grace::schedule`
  arms the alarm from `until`; `grace::fire` cancels with `grace.by` and
  `grace.reason`.
- A cancel with a grace during a grace (tasks.md, 7: "One with a grace
  asks nothing more, since a task is asked to finish once, by the
  first. If its grace ends sooner, the task's grace now ends with it,
  and the task is then cancelled with this later cancel's maker and
  reason ... If its grace ends no sooner, nothing of the task
  changes"): `ask_to_finish` on an active task whose `grace` is `Some`
  adds no word and takes no room. When its `until` is earlier than the
  kept one, the task's `Grace` takes this cancel's `by`, `until` and
  `reason`, keeps its `message` (the first's ask, the one word there
  is), and `grace::schedule` re-arms the alarm from it; when it is equal
  or later, nothing of the task changes. Either way the cancel is
  answered as the first was, and its `message`, required as for any
  cancel with a grace, is left unused.
- The ask is admitted `inbox::Admission::RoomKept` (tasks.md, 8.2: "a
  cancel, its grace's ask to finish taking the same room, one ask in a
  task's life (section 7), so none of them can be refused"): one slot
  per live task and the reason's bytes (within `result_bytes`) are kept
  for it, counted in `worst_case`, so a full inbox never refuses it, and
  it is a word of its own, never merged into an amendment still
  waiting. One slot holds, since a task in a grace hears no second ask.
- `chat::close` builds its grace with `by: Party::Person(person)`,
  `reason: EXPIRED`, its finish words `FINISH`, through
  `ask_to_finish`; its cohort check is unchanged.
- `Limits::grace: Duration`, checked nonzero by `worst_case`.
- `jig_protocol_config::charters::chats(document, at, path, charters,
  grace: Duration, problems)`.
Test shape:
- `tests/tasks/tests/lifecycle/grace.rs` (new, `#[path =
  "lifecycle/grace.rs"] mod grace;` in `lifecycle.rs`, as `goals` is at
  :774), the tasks world: `World::new` (src/world.rs:191),
  `task` (:72), `World::make` (:824), `claim` (:861), `elapse` (:964),
  `restart` (:994), `settle` (:942), `record` (:798). Limits: the
  world's `LIMITS` (src/world.rs:17) gains `grace:
  Duration::from_secs(600)`, the small profile's value. Reference test:

  ```rust
  use jig_domain_tasks::{Control, Ending, Event, Key, Party, Phase, Stored};
  use jig_tasks_world::{LIMITS, World, task};
  use skein_lib::Duration;

  fn cancel(world: &mut World, number: u64, grace: Option<Duration>) {
      let reply_to = world.to();
      let message = world.record(number).last_message + 1;
      world.send(Event::Control {
          reply_to,
          by: Party::Person(1),
          task: number,
          control: Control::Cancel {
              reason: Box::from(&b"wrap up"[..]),
              grace,
              message: grace.map(|_| message),
          },
      });
  }

  #[test]
  fn a_grace_kept_with_its_task_ends_on_time_after_a_restart() {
      let mut world = World::new(190, LIMITS);
      world.make(Party::Person(1), vec![task(1, &[])]);
      cancel(&mut world, 1, Some(Duration::from_secs(5)));
      world.restart();
      world.elapse(Duration::from_secs(4));
      assert!(matches!(world.record(1).phase, Phase::Active(_)));
      world.elapse(Duration::from_secs(1));
      world.settle(1);
      let Stored::Ended(row) = &world.records[&Key::Ended(1)] else { panic!("ended row") };
      let Phase::Ended(Ending::Cancelled { by, reason, .. }) = &row.phase else { panic!("cancelled") };
      assert_eq!(*by, Party::Person(1));
      assert_eq!(reason.as_ref(), b"wrap up");
  }
  ```
- Its other tests: `a_cancel_with_a_grace_asks_the_task_to_finish_first`
  (the word's kind is an amendment, its words `b"wrap up"`, the phase
  still active), `a_grace_longer_than_the_limit_is_refused_and_changes_nothing`
  (601 s; the saved rows equal before),
  `a_waiting_task_cancelled_with_a_grace_ends_at_once`,
  `a_cancel_without_a_grace_during_one_cancels_at_once`,
  `a_second_grace_never_ends_the_task_later_than_the_first` (a 5 s
  grace, then a 10 s one: one ask in the inbox, the task's `grace` the
  first's, and the task cancelled at 5 s with `b"wrap up"`),
  `a_sooner_second_grace_ends_the_task_with_its_own_reason` (a 5 s
  grace, then a 2 s one whose reason is `b"now"`: one ask in the inbox,
  its words `b"wrap up"`, and the task cancelled at 2 s with `b"now"`),
  `a_graces_ask_to_finish_is_admitted_into_a_full_inbox` (the inbox at
  `inbox_messages` words first; a second cancel with a grace after it
  is admitted too, and adds no word), `a_graces_ask_never_merges_into_a_waiting_amendment`
  (both words in the inbox after), `a_person_task_cancelled_with_a_grace_shows_its_holder_the_ask`
  (a person task taken by party 3: the ask is in its inbox as an
  amendment's word, and the task ends cancelled when the grace ends).
- `crates/jig-protocol-config/src/tests/domain.rs`:
  `a_chat_close_grace_past_the_tasks_limit_refuses_the_start`.
- `tests/core/src/stories/closing_chat.rs` keeps its assertions, on
  `grace` in `chat_close`'s place.
- Ceilings: world tests under 0.5 s, unit tests under 20 ms.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(=peers_scenarios)'` once (it passes at the base in 0.07 s): its
`closing_chat::disposals_and_finish_grace` closes a chat whose grace
now lives in the tasks child, restarts the engine inside the grace, and
sees the chat cancelled at or after the grace's end with the kept
delegate still live; under 1 s. Nothing else is run for it.
Don't: route a party's or a run's cancel with a grace from the core (the
core sends `grace: None` until 04.5 and jig 06.5); read
`Settings.close_grace` anywhere new.
Size: at most 255 lines (about 110 of them moved out of chat.rs; 20
for a grace during a grace and its test, ruling 123's answer).
Removes: chat.rs's grace half, `TaskRecord.chat_close`, the chat alarms.

### 04.2 Tracked made amendable, and goals per project, in the tasks child

Scope: tasks.md, 7 and 11: an amendment tracks a task at a priority,
changes a tracked task's priority, or untracks it; a tracked task never
moves the spec revision by its tracking or priority; goals per project
(the live tracked tasks of one project) is a tasks limit, refused at the
entrance by a batch, an amendment or an accepted proposal that would
pass it. The history keeps tracked before and after.
Needs: jig 02.1's tasks family at version 3.
Files:
- `crates/jig-domain-tasks/src/`: `control.rs` (`Tracking`,
  `Amendment.tracked`, `Settings.tracked`, the amend path),
  `goals.rs` (new: the per-project count and its check), `batch.rs`
  (the `Make` check), `proposals.rs` (an accepted batch or amendment,
  at most 10 lines), `boundary.rs` (`Refusal::Goals`), `limits.rs`
  (`project_goals`), `lib.rs`.
- `crates/jig-protocol-store/`: `schema/tasks-v3.schema` (amendment and
  history settings' tracked; `Refusal::Goals`),
  `src/codec/tasks/{proposals.rs, activity.rs, carried.rs}`, the
  generated family, the goldens.
- `crates/jig-protocol-config/src/`: `profile/small.rs`
  (`project_goals: 8`), `profile/standard.rs` (`project_goals: 16`),
  their module doc lines, `overrides.rs`, `tests/overrides.rs` (`SMALL`:
  `(b"tasks.project_goals", 8)`).
- `crates/jig-domain-core/src/routing/refusals.rs` (`task_answer`'s arm
  for `Refusal::Goals`, mapped to `people::Refusal::Limit { which:
  people::LimitKind::ProjectLiveTasks }`, the project's bound on its
  tasks, until 04.5 points it at `LimitKind::Goals`; a test of 04.2 that
  reads a person's refused goal expects `ProjectLiveTasks`, and 04.5
  changes it to `Goals`);
  `crates/jig-protocol-tools/src/render.rs` (`task_problem`'s `Goals`
  arm).
- Users of the tasks `Amendment` literal (`tracked: None` added): in
  `jig-domain-tasks/src/{chat.rs, control.rs, tests/address.rs,
  tests/instructions.rs, tests/revisions.rs}`, `jig-domain-core/src/
  {acts/bounds.rs, amendments.rs, routing/party_decisions.rs,
  routing/runs.rs, tests.rs, tests/acts.rs, tests/acts/kind_order.rs}`,
  `jig-protocol-tools/src/nested.rs`,
  `jig-protocol-store/src/{codec/tasks/proposals.rs, tests/fixtures.rs,
  tests/codec/phase.rs}`, `tests/core/tests/stories.rs`,
  `tests/conformance/src/actions.rs:102` (jig 06.1's `RacingDecisions`
  run amendment); the tasks
  `Limits` literals of 4.1's list, each gaining `project_goals`: the
  small profile's 8, but 2 in `jig-domain-tasks/src/tests/revisions.rs:21`,
  `tests/tasks/src/world.rs:17` and `tests/core/src/world.rs:19`, where
  two goals are what the tests count to (and the core world's `tasks` is
  2).
- Downstream: its 2 tasks `Amendment` literals
  (`crates/temper-engine-smith/src/nested.rs:72`,
  `tests/engine/domain/tests/root_routes.rs:1584`), its tasks `Limits`
  literal and a `Goals` arm in `answers.rs:351`, at its repin.
Shape:
- `jig_domain_tasks::Tracking { Tracked { priority: u32 }, Untracked }`;
  `Amendment.tracked: Option<Tracking>`; `Settings.tracked: Option<u32>`.
- An amendment whose only change is `Tracked { priority }` on a tracked
  task moves no revision but `revision` (the history row's); tracking or
  untracking moves no `revisions.spec`.
- `Limits::project_goals: u32`; `goals::room(domain, project, adding:
  u32, limits) -> Result<(), Refusal>` (`Refusal::Goals`), called by
  `Make`, by an amendment that tracks an untracked task, and by an
  accepted proposal whose batch or amendment tracks.
Test shape:
- `crates/jig-domain-tasks/src/tests/revisions/tracking.rs` (new, `mod
  tracking;` in tests/revisions.rs), with `Harness` (tests/revisions.rs:
  96), `task` (:76), `amendment` (:208), `to` (:102), and `LIMITS`
  (:21) with `project_goals: 2`. Tests:
  `an_amendment_tracks_a_task_at_its_priority`,
  `an_amendment_untracks_a_goal_and_moves_no_spec_revision`,
  `a_priority_change_alone_moves_no_spec_revision`,
  `tracking_past_goals_per_project_is_refused_and_changes_nothing`,
  `a_batch_past_goals_per_project_is_refused_naming_the_limit`,
  `the_history_keeps_tracked_before_and_after`. Under 20 ms each.
- `tests/tasks/tests/lifecycle/goals.rs`: 
  `goals_per_project_count_only_live_tracked_tasks_after_a_restart`
  (two goals, one ends, a third is tracked by amendment after
  `restart`), with the world's `LIMITS` at `project_goals: 2`. Under
  0.5 s.
Proof: world, `cargo nextest run -p jig-tasks-world -E
'test(goals_per_project_count_only_live_tracked_tasks_after_a_restart)'`
once: a project at its two goals refuses a third tracked by amendment
until one ends, and admits it after a restart; under 0.5 s. Nothing
above the tasks child sends a tracked amendment until 04.5, which
connects it at the joint layer.
Don't: change projections' states (jig 07.5); let the core send a
tracked amendment (04.5).
Size: at most 150 lines.
Removes: nothing.

### 04.3 `Move`'s escalation holder; each control split into a check and an apply

Scope: people.md, 5.1 (`Move`: "a holder for its escalations"); the
split the compound needs (findings/engine-parties-and-tasks.md, 1):
`control::amend`, `control::release`, `Control::Cancel` (with 4.1's
grace), `inbox::message` (words, with a stop's release) and
`run::hold` for a party's stop each become a check over `&Domain` that
returns a plan, and an apply that takes the plan and refuses nothing;
the existing entry points call the two in turn, with no change of
behaviour.
Needs: jig 02's escalation holder (jig 02.3: `jig_domain_authority::
Domain::escalations_held(project, role)` and
`people::Refusal::NoEscalationHolder`, which jig 02 leaves `Move` to
use).
Files:
- `crates/jig-domain-tasks/src/{control.rs, inbox.rs, run.rs,
  grace.rs}`.
- `crates/jig-domain-core/src/roles.rs` (`move_admit`), and its test in
  jig 02.3's `crates/jig-domain-core/src/tests/escalation_holder.rs`.
Shape:
- `control::check_amend(domain: &Domain, env, by, number, message,
  stop_run, amendment) -> Result<AmendPlan, Refusal>` and
  `control::apply_amend(domain: &mut Domain, env, by, plan, out)`;
  `check_release`/`apply_release` (`ReleasePlan`), `check_cancel`/
  `apply_cancel` (`CancelPlan`), `inbox::check_message`/
  `apply_message`, `run::check_stop`/`apply_stop`. The plans are
  `pub(crate)` and own what the apply writes (the prepared inbox, the
  next phase, the selected subtree). `apply_message` keeps jig 02.1's
  `last_activity` write (`inbox.rs:326`), as the split of
  `closing::end_task` keeps its own (`closing.rs:389`).
- `Core::move_admit` refuses `people::Refusal::NoEscalationHolder`,
  after its role check and before its authority check, when
  `self.authority.escalations_held(project, role)` is false.
Test shape:
- The tasks child's present tests of these entry points stay as they are
  and pass unchanged (tests/revisions.rs, tests/revisions/chat.rs,
  tests/address.rs, tests/instructions.rs), with the tasks world's
  `tests/tasks/tests/activity.rs:139, 159`
  (`a_tasks_last_activity_moves_when_it_hears_words`,
  `a_requesters_last_activity_moves_when_a_delegate_ends`): the split's
  check.
- `crates/jig-domain-core/src/tests/escalation_holder.rs` (jig 02.3's),
  with its fixture (`core()`, tests.rs:308, and
  `funding_fixtures.rs:2`'s `set_role_authority`):
  `a_move_is_refused_while_nobody_holds_its_escalations`. Under 20 ms.
Proof: joint, once: `cargo nextest run -p jig-core-world -E
'test(=peers_scenarios) | test(a_plan_of_reports_and_a_procedure_is_revised_midway_and_done) | test(an_exhausted_delegate_escalates_to_a_person_who_releases_it_to_finish)'`
(3 tests at the base): a chat closed with a delegate moved and one
cancelled, a person's stop released by words, a plan amended midway and
a held delegate released by a person, each through the split paths,
pass unchanged (0.09 s together at the base); under 1 s. The move's
refusal is shown at the unit layer only: a joint story needs a party
who requested a tree and then lost their escalations' holder, and jig
02's `no_holder_fixture` (tests/core/src/stories/offers.rs) starts no
tree for its member.
Don't: change any refusal, revision or history the entry points give
today; give a plan a lifetime that borrows the domain.
Size: at most 260 lines (the split rewrites about 200 of control.rs's).
Removes: nothing.

### 04.4 The compound request in the tasks child

Scope: tasks.md, 7 ("Several operations, one decision"):
`Event::Compound` checks its operations in order, each by 4.3's check
against the domain and the `Staged` record of what the operations before
it leave, counts one pool across its moves (as chat.rs's close counts
its cohort, chat.rs:76-146), and applies them in order only after the
last check passed; otherwise it answers `Request::CompoundRefused` naming the
first failure's place and changes nothing. A compound never becomes a
proposal.
Needs: nothing beyond 4.3 (jig 02.3). jig 03 is not needed: the tasks
child's compound names no batch.
Files:
- `crates/jig-domain-tasks/src/`: `compound.rs` (new: `Operation`,
  `Staged`, `compound`), `boundary.rs` (`Event::Compound`,
  `Request::CompoundRefused`; at most 40 lines), `domain.rs` (the
  dispatch, `output_bound`: `tasks` × 8 for a compound's operations),
  `control.rs`, `inbox.rs`, `run.rs`, `moving.rs` (the checks read
  `Staged`), `lib.rs`.
- Exhaustive matches over `tasks::Event` and `tasks::Request` (an arm
  each, README, Rules): `crates/jig-domain-core/src/routing/tasks.rs`,
  `tests/tasks/src/world.rs`, and those the compiler names.
Shape:
- `jig_domain_tasks::Operation { Cancel { task, reason, grace, message },
  Move { task, reason }, Amend { task, message, stop_run, amendment:
  Box<Amendment> }, Release { task, again, message }, Message { task,
  word, release }, Stop { task } }`.
- `Event::Compound { reply_to, person: u64, project: u32, operations:
  List<Operation>, period: u64, pool_budget: u64, period_budget: u64 }`:
  each task in `project`; at most `Limits::tasks` operations, else
  `Refusal::Batch` at place 0; answered `Request::Done { reply_to }`.
- `Request::CompoundRefused { reply_to, at: u32, problem: Problem }`.
- `Staged { closing: List<u64>, moved: List<u64>, reserved: u64,
  revisions: List<(u64, Revisions)> }`: a check of an operation on a
  task in `closing` is `Refusal::State`; a release or an amendment names
  the revision `revisions` holds for its task, when it holds one; a move
  adds to `reserved`, refused `Refusal::Funding` past the pool. 4.3's
  checks take `staged: &Staged`; a single operation's entry point passes
  an empty one.
Test shape:
- `crates/jig-domain-tasks/src/tests/revisions/compound.rs` (new, `mod
  compound;`), with `Harness`, `task`, `amendment`, `to`, and `cohort`
  (tests/revisions/chat.rs:6, made `pub(super)`). Reference test:

  ```rust
  use super::*;
  use crate::Operation;
  use skein_lib::List;

  #[test]
  fn a_refused_operation_leaves_the_operations_before_it_unmade() {
      let mut test = chat::cohort();
      let before = test.stored.clone();
      let mut operations = List::with_capacity(2);
      operations
          .push(Operation::Cancel { task: 2, reason: Box::from(&b"done"[..]), grace: None, message: None })
          .unwrap();
      operations.push(Operation::Release { task: 3, again: None, message: None }).unwrap();
      let out = test.send(Event::Compound {
          reply_to: to(),
          person: 9,
          project: 1,
          operations,
          period: 0,
          pool_budget: 1_000,
          period_budget: 10_000,
      });
      assert!(out.iter().any(|request| match request {
          Request::CompoundRefused { at: 1, problem, .. } => problem.why == crate::Refusal::State,
          _ => false,
      }));
      assert_eq!(test.stored, before);
  }
  ```
- Its other tests: `a_compound_makes_its_operations_in_order_in_one_step`,
  `an_operation_on_a_task_an_earlier_cancel_closed_is_refused`,
  `moves_count_one_pool_across_the_request`,
  `an_amendment_then_a_release_of_one_held_task_are_made_together`,
  `words_then_a_stop_release_in_one_request_wake_the_task`,
  `an_operation_on_another_projects_task_is_refused_at_its_place`
  (`CompoundRefused { at: 1, .. }` for a second operation on a task of
  project 2, `Refusal::Project`). Under 20 ms each.
- `tests/tasks/tests/lifecycle/compound.rs` (new; `#[path =
  "lifecycle/compound.rs"] mod compound;` in `lifecycle.rs`, as :774 does):
  `a_compound_closing_a_task_and_moving_its_delegate_survives_a_restart`.
  Under 0.5 s.
Proof: world, `cargo nextest run -p jig-tasks-world -E
'test(a_compound_closing_a_task_and_moving_its_delegate_survives_a_restart)'`
once: the parent's cancel with a grace and its delegate's move, made in
one step, are both in the restored state; under 0.5 s. The core sends no
compound until 04.6, which connects it at the joint layer.
Don't: clone the domain or a task record to check; send any output
before the last check passed; reuse `Event::CloseChat`'s path (jig 16
removes it).
Size: at most 500 lines (ruling 159: the staged view its checks read; 886 counted at 14141fecd).
Removes: nothing.

### 04.5 The party's cancel with a grace, and tracked and priority amended by a maintainer

Scope: people.md, 5.1: `Ask::Cancel` takes a grace; `people::Amendment`
takes tracked and priority, amended by a maintainer only, with no
revision for a priority alone; the core routes both to 4.1's and 4.2's
tasks paths, issuing the finish message, and refuses a tracking
amendment that would pass the projections' room as a tracked `Make` is
refused (routing/tasks.rs:1361); the shaped `Ask::Prioritise` takes the
maintainer check.
Needs: the people family at version 3 on `main` (jig 02.3's landing,
README, Rules).
Files:
- `crates/jig-domain-people/src/`: `boundary.rs` (`Ask::Cancel`),
  `amendment.rs` (`Tracking`, `Amendment.tracked`, `amendment_bytes`),
  `refusals.rs` (`LimitKind::Grace`, `LimitKind::Goals`), `words.rs`,
  `domain.rs` (at most 40 lines), `tests.rs` (at most 40 lines).
- `crates/jig-domain-core/src/`: `routing/party_controls.rs`
  (`cancel_request`: the grace, `fresh(Family::Message)` for its
  message), `roles.rs` (`control_admit`: the grace's bound;
  `priorities_allowed`), `acts/standing.rs` (`Act::Prioritise` and a
  tracked or priority `Amend` for `Owner` and `Maintainer` only),
  `amendments.rs` (translate tracked), `routing/refusals.rs` (4.1's and
  4.2's arms → `people::LimitKind::{Grace, Goals}`),
  `routing/tasks.rs` (the projections' room counts an `Amend` that
  tracks; at most 15 lines).
- `crates/jig-protocol-store/`: `schema/people-v3.schema`, `src/codec/
  people/{asks.rs, pieces.rs, outcomes.rs}`, the generated family, the
  goldens.
- `crates/jig-protocol-engine-clients/src/translate/asks.rs` (the wire's
  cancel and amendment give `grace: None`, `tracked: None`: jig 14
  carries them).
- `testing/jig-fake-parties/src/lib.rs`, `tests/core/src/peers.rs`: a
  `Cancel { task, grace }` fake ask and its translation.
- Users of `people::Ask::Cancel` (12 literals and the store codec's
  pattern at `codec/people/asks.rs:402`, in 10 files; `grace: None`
  added): `jig-domain-core/src/{acts/objects/task.rs, tests/acts.rs,
  tests/document_offers.rs}`, `jig-protocol-engine-clients/src/
  {tests/fixtures.rs, translate/asks.rs}`, `jig-protocol-store/src/
  {codec/people/asks.rs, tests/fixtures.rs}`,
  `tests/client/src/joint/wire/asks.rs`, `tests/core/tests/{reads.rs,
  stories.rs}`. Users of the people `Amendment` literal (`tracked: None`
  added), among them `jig-domain-core/src/tests/document_offers.rs:79`,
  `jig-protocol-store/src/tests/codec/pieces.rs:22`,
  `tests/core/src/stories/instructions.rs:24` and
  `tests/people/tests/simulation.rs:448, 574`,
  `tests/conformance/src/scenarios.rs:227` (`RacingDecisions`' party
  amendment), and the exhaustive matches
  over `LimitKind`, as the compiler names them.
- Downstream: its 2 `Ask::Cancel`, 3 people `Amendment` literals, and 2
  matches over `people::LimitKind`
  (`crates/temper-protocol-store/src/records/application/refusal.rs:212`,
  `crates/temper-service/src/family/refusals.rs:99`) with the decode of
  its own wire (`refusal.rs:251`), at its repin.
Shape:
- `people::Ask::Cancel { project: u32, task: u64, reason: Box<[u8]>,
  grace: Option<Duration> }`.
- `people::Tracking { Tracked { priority: u32 }, Untracked }`, mirroring
  `tasks::Tracking`; `people::Amendment.tracked: Option<Tracking>`.
- An `Amend` whose only change is a priority on a tracked task names
  revision 0 and is not checked against the spec's revision.
- A grace past `limits.tasks.grace` is `Refusal::Limit { which:
  LimitKind::Grace }`; a tracking past goals per project is
  `Refusal::Limit { which: LimitKind::Goals }`.
- A party's cancel with a grace during a grace takes 4.1's path, which
  asks the task nothing more and keeps the sooner end with its cancel's
  `by` and reason; it is answered `Outcome::Cancelled { task }`, as the
  first, and the message number the core took for it stays unused.
Test shape:
- `crates/jig-domain-core/src/tests/acts.rs`:
  `a_member_amending_a_priority_is_refused_as_not_a_maintainer`,
  `a_policy_role_amending_tracked_is_refused`,
  `a_priority_alone_names_no_revision`. Under 20 ms each.
- `tests/core/tests/operations.rs` (new), the joint world through
  `stories::reads::{world, request}` (tests/core/src/stories/reads.rs:13,
  44), answers read from `world.people_answers` (as
  tests/core/tests/refusals.rs:37 reads them), `World::advance`
  (src/effects.rs:187), `World::restart` (src/effects/store.rs:138).
  Limits: `world::limits()` (tests/core/src/world.rs:17), the world's
  own, with 4.1's `grace` and 4.2's `project_goals`. Reference test:

  ```rust
  use jig_core_world::stories::reads::{request, world};
  use jig_domain_core as core;
  use jig_domain_people as people;
  use jig_domain_tasks as tasks;
  use jig_test_domain as root;
  use skein_lib::Duration;

  #[test]
  fn a_partys_cancel_with_a_grace_ends_the_chat_when_the_grace_ends() {
      let mut world = world(1102);
      let task = world.assigned.as_ref().expect("chat").task;
      let cancel = people::Ask::Cancel {
          project: 1,
          task,
          reason: Box::from(&b"wrap up"[..]),
          grace: Some(Duration::from_secs(5)),
      };
      request(&mut world, 1, cancel);
      assert_eq!(
          world.people_answers.last(),
          Some(&people::Reply::Outcome(people::Outcome::Cancelled { task }))
      );
      world.advance(Duration::from_secs(5));
      world.end(tasks::End::Parked, 0);
      let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) =
          world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
      else {
          panic!("the chat ended");
      };
      let tasks::Phase::Ended(tasks::Ending::Cancelled { by, reason, .. }) = &row.phase else {
          panic!("cancelled");
      };
      assert_eq!(*by, tasks::Party::Person(1));
      assert_eq!(reason.as_ref(), b"wrap up");
  }
  ```

  Its other tests: `a_maintainer_tracks_a_task_by_amendment`,
  `a_grace_past_the_limit_is_refused_naming_it`. Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_partys_cancel_with_a_grace_ends_the_chat_when_the_grace_ends)'`
once: the chat's run hears the request's reason as words to finish, and
the chat ends cancelled by the party with that reason when the grace
ends (a restart inside a grace is 4.1's proof); under 0.5 s.
Don't: add a priority, tracking or grace to `Ask::SetGoal`,
`Ask::StartChat` or `Ask::Close` (shaped).
Size: at most 170 lines (a codec increment: lands within 2.5 times).
Removes: nothing.

### 04.6 The compound request in the people child and the core

Scope: people.md, 5.1 ("A compound request"); contract.md, 3.4;
engine.md, 4.4 and 13: `Ask::Compound { project, operations }`, keyed
and kept as any ask, admitted by the people child on one project and at
most its `operations` limit; the core admits each operation in order as
it would alone (`control_admit`, `amend_admit` with `spec_revision`
and `hold_revision`, `move_admit`, `person_message`, 4.5's maintainer
check), treating a task an earlier operation moved as in the party's
tree, issues each message in the request's order, and sends one
`tasks::Event::Compound`; the answer is `Outcome::Compound`, or
`Outcome::CompoundRefused` naming the first operation that failed, by
its place, whether the core or the tasks child refused it.
Needs: the people family at version 3 on `main` (jig 02.3's landing);
4.4's `Event::Compound` and 4.5's operations, earlier in this session.
No jig 03: a compound holds no `Create`, `Propose`, `Withdraw` or
`Decide` (Don't, below).
Files:
- `crates/jig-domain-people/src/`: `compound.rs` (new: `Operation`, its
  validation and bytes), `boundary.rs` (`Ask::Compound`,
  `Outcome::{Compound, CompoundRefused}`), `limits.rs` (`operations`,
  priced in `ask_bytes`), `refusals.rs` (`LimitKind::Operations`),
  `domain.rs` and `capacity.rs`, `words.rs` (an arm each, at most 40
  lines in domain.rs), `lib.rs`.
- `crates/jig-domain-tasks/src/`: `compound.rs` and
  `compound/staged.rs` (`CompoundAdmission`, ruling 174), `lib.rs` (its
  re-export).
- `crates/jig-domain-core/src/`: `compound.rs` (new: the route),
  `run.rs` (`PersonTaskRoute::Compound`, :382), `routing/people.rs` (the
  dispatch arm),
  `routing/tasks.rs` (`CompoundRefused` to the party; at most 20
  lines), `memory.rs` (`worst_case` is `None` when
  `people.operations > tasks.tasks`), `lib.rs`.
- `crates/jig-protocol-config/src/`: `profile/small.rs`
  (`operations: 8`), `profile/standard.rs` (`operations: 16`), their doc
  lines, `overrides.rs` (`set_people`), `tests/overrides.rs` (`SMALL`:
  `(b"people.operations", 8)`).
- `crates/jig-protocol-store/`: `schema/people-v3.schema`,
  `src/codec/people/{asks.rs, outcomes.rs}`, the generated family, the
  goldens.
- The people `Limits` literals (7 files), each gaining `operations`: the
  small profile's 8 in `jig-domain-core/src/tests.rs`,
  `jig-protocol-store/src/tests/fixtures.rs`, `tests/store/src/root.rs`;
  2 in `jig-domain-people/src/tests.rs:10`, `tests/people/src/world.rs:17`
  and `tests/core/src/world.rs`, where two operations are what the tests
  send (and the core world's `tasks` is 2); and the
  exhaustive matches over `people::Ask` and `people::Outcome` the
  compiler names (in jig-domain-people, jig-domain-core,
  jig-protocol-store, jig-protocol-engine-clients, jig-test-domain,
  jig-fake-parties, the worlds), an arm each.
- Downstream: its people `Limits` literal and its matches over
  `people::Ask` (`route/asks.rs`, `store.rs`) and `Outcome`, at its
  repin.
Shape:
- `people::Operation { Cancel { task, reason, grace }, Move { task,
  reason }, Amend { task, revision, amendment: Box<Amendment>, release },
  Release { task, revision, again }, Message { task, words, release },
  Stop { task } }`, each as the `Ask` of its name less its `project`.
- `people::Ask::Compound { project: u32, operations: List<Operation> }`;
  empty or past `Limits::operations` is `Refusal::Limit { which:
  LimitKind::Operations }` at admission.
- `people::Outcome::Compound { project: u32, operations: u32,
  first_message: u64 }` (`first_message` 0 when no operation took a
  message, as landed; the step after 04.6 numbers every operation, and
  `first_message` is never 0: Choices); `people::Outcome::CompoundRefused { at: u32, why: Refusal }`.
  `valid_answer_shape` admits them for a compound only.
- `Limits::operations: u32`.
- `tasks::CompoundAdmission` (ruling 174): read-only, it holds the
  private `Staged` and borrows the `Domain`; for one operation it runs
  the checks `compound()` runs (compound.rs:190, `check`) and stages it;
  borrowed accessors give the core the projected fields its admits read
  (revisions, tracking, requester ancestry, funding, phase, address).
  `compound()` runs its checks through the same code. The core admits
  each operation in order as it would alone, its own admit and then the
  admission's, against the projection, and sends one `Event::Compound`
  once all pass; the first failure, either's, is `CompoundRefused { at }`.
Test shape:
- `crates/jig-domain-people/src/tests/compound.rs` (new; `mod compound;`
  in tests.rs), with the people `LIMITS` (tests.rs:10):
  `a_compound_past_operations_per_request_is_refused`,
  `an_empty_compound_is_refused`,
  `a_compound_sent_twice_under_one_key_is_routed_once`. Under 20 ms
  each.
- `tests/people/tests/compound.rs` (new), the people world
  (`tests/people/src/world.rs`, its `LIMITS` at `operations: 2`):
  `a_compound_answered_before_a_restart_is_answered_once_after_it` and
  `a_compound_refused_keeps_its_refusal_under_its_key_across_a_restart`
  (testing.md, 3: "operations alone and in compound requests, made whole
  or not at all"). Under 0.5 s each.
- `tests/core/tests/operations.rs`:
  `a_compound_closes_a_conversation_moving_one_delegate_and_cancelling_another`
  (the conformance's close: the delegate cancels and moves, then the
  conversation's cancel with a grace),
  `a_compound_refused_at_its_second_operation_makes_nothing`,
  `a_compound_naming_another_projects_task_is_refused_at_its_place`
  (`Outcome::CompoundRefused { at: 1, why: Refusal::Unknown }`, as a lone
  operation on that task is refused, contract.md, 3.4: "one project,
  every operation's task in it"),
  `a_compound_reorders_two_goals_at_once`,
  `a_compound_sent_again_after_a_crash_is_made_once_and_answered_once`
  (the crash and the resend as `a_read_waits_for_the_commit_in_flight_before_it`
  makes them, tests/core/tests/reads.rs:11-34). Under 0.5 s each.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_compound_sent_again_after_a_crash_is_made_once_and_answered_once)'`
once: a compound closing a conversation, committed and crashed before
its answer, sent again under its key after the restart, is in the store
once and answered once; under 0.5 s.
Don't: let a compound hold `Create`, `Propose`, `Withdraw`, `Decide`,
`Answer`, `Take`, `HandBack`, `Seen` or a project's own request; chain
two keyed requests in the core; check operations per request against
delegates or goals at start (jig 09.4); clone a `Domain` or a task
record for the admission, mutate the `Domain` through it, or write a
second copy of the tasks child's checks in the core (ruling 174).
Size: at most 420 lines (a codec increment: lands within 2.5 times).
Removes: nothing.

### 04.7 A move's conversation read position; the compound and the grace as conformance scenarios

Scope: people.md, 6 ("a move to another party erases the old
requester's in the same decision, and starts the new one's with nothing
read"); testing.md, 5 (the scenarios "a compound request, the engine
crashing before and after its commit and the request sent again: made
whole once, or not at all, and answered once" and "a cancel with a grace
across restarts: the task finishing within it, or cancelled once when it
ends, from the grace kept with it"), 6 (the referee: "every batch, and
every compound request, is in the store whole or not at all") and 9
(the plumbing's stories "join the scenarios of the worlds they belong
to, a few cuts each in the focused suite and swept ... in the fuzzy
one"). A `Move`, alone or in a compound, of a task the core takes as a
conversation (`chat_requester`, or `conversation_requester` once jig
05.5 has landed) moves its read position in the move's decision; the
conformance harness gains the two scenarios and the referee's rule for a
compound, and this session's joint stories join them.
Needs: 4.5's party cancel with a grace and 4.6's compound, earlier in
this session.
Files:
- `crates/jig-domain-people/src/`: `boundary.rs` (`Event::MoveChat`),
  `inbox.rs` (the move of the position), `domain.rs` (its dispatch arm,
  at most 5 lines).
- `crates/jig-domain-core/src/routing/tasks.rs` (on a committed move of
  a conversation, `people::Event::MoveChat`; at most 15 lines), and the
  core's touch of both parties' `me` and inbox (`touch_party`).
- `testing/jig-conformance/src/scenarios.rs` (`Scenario::{Compound,
  Grace}` in `ALL`, their `Action`s and `program` arms),
  `testing/jig-conformance/src/referee.rs` (`Observed::{CompoundAsked,
  Compound}`, their rule under `Promise::Once`; ruling 183).
- `tests/conformance/src/scenarios.rs` (both scenarios' configuration,
  actions and checks for the testing application),
  `tests/conformance/src/lib.rs` (the worker script for each scenario,
  :215, matches every `Scenario`: an arm each, the Grace scenario's a
  script of its own), `tests/conformance/tests/scenarios.rs` (their
  chosen cuts).
- Downstream: its conformance configuration and checks match every
  `Scenario` (`tests/conformance/src/config.rs:132` and
  `tests/conformance/src/scenarios.rs:512` there), and absorb the two
  variants at its repin.
Shape:
- `people::Event::MoveChat { task: u64, project: u32, from: u64, to:
  u64 }`: the people child erases `Key::ReadChat { party: from, task }`
  and saves `ReadChat { party: to, task }` at 0, in one decision.
- `Scenario::Compound`: a party's chat with two live delegates; one
  compound moves the first to the party, cancels the second, and cancels
  the chat with a grace (`Action::Compound`), then the request is sent
  again under its key (`Action::CompoundAgain`); its check: the moved
  delegate's requester is the party, the others are cancelled, and the
  party holds one answer. `Scenario::Grace`: a party cancels a running
  task with a 2 s grace (`Action::CancelWithGrace`), the program crashes
  the engine twice inside the grace, and the run finishes within it
  (`Action::Finish`) in one seed's half and not in the other; its check:
  done once, or cancelled once with the cancel's reason at or after the
  grace's end. Each program has at most 8 commits after its
  `Action::Ready`; the harness's start before it, shared by every
  program, is not counted (ruling 186).
- `Observed::CompoundAsked { tasks: Vec<u64> }`, declared by the
  scenario's action before each send of the compound (`Action::Compound`
  and `Action::CompoundAgain`): the referee keeps, for the first ask of
  a `tasks` list, each named task's row (`referee::Task`) from its last
  durable snapshot; a later ask of the same list keeps that baseline.
  On each `Observed::Durable`, the first commit after the baseline that
  changes any named row is recorded with the rows it changed.
  (Ruling 183.)
- `Observed::Compound { tasks: Vec<u64>, made: bool }`, declared by the
  scenario's actions when its answer arrives, with the `tasks` of its
  ask: made, the recorded commit changed every named row; refused, no
  named row changed since the baseline. An answer with no ask of its
  `tasks` breaks the rule too. A task whose operation changes no field
  of `referee::Task` (a graced cancel keeps its phase, `grace.rs:170`)
  is left out of both `tasks` lists.
Test shape:
- `crates/jig-domain-people/src/tests.rs` (at most 40 lines):
  `a_moved_chats_old_requester_loses_its_read_position`,
  `a_moved_chats_new_requester_starts_with_nothing_read`. Under 20 ms
  each.
- `tests/core/tests/operations.rs`:
  `a_moved_conversation_starts_its_new_requester_with_nothing_read`.
  Under 0.5 s.
- `tests/conformance/tests/scenarios.rs`:
  `compound_chosen_answer_cut_ends_the_story` and
  `grace_chosen_restart_cut_ends_the_story` (as `chosen_cut`, :19-23,
  makes them); `every_shared_adversarial_story_runs_on_the_testing_application`
  (:10) runs both by `ALL`, and the fuzzy
  `every_shared_story_survives_every_commit_and_seeded_random_cuts`
  (tests/conformance/tests/fuzzy_scenarios.rs) sweeps both. Aim for at
  most 1 s added to the fuzzy suite by the two; past 2 s, the session
  reports it, with the sweep's time by scenario from the gate's log.
- `testing/jig-conformance/src/referee.rs`'s tests:
  `a_compound_made_across_two_commits_breaks_once`,
  `a_refused_compound_with_a_changed_task_breaks_once`. Under 20 ms
  each.
Proof: joint, `cargo nextest run -p jig-conformance-world -E
'test(compound_chosen_answer_cut_ends_the_story) |
test(grace_chosen_restart_cut_ends_the_story)'` once: the testing
application, crashed after the compound's commit and sent the request
again, holds the compound once and one answer, and a graced task
restarted twice inside its grace ends once; the referee holds at each
iteration; under 2 s.
Don't: give the two scenarios their own sweep; mark the world's
`Testing` application not applicable to either (its kinds have both).
Size: at most 150 lines (about 110 of them in `testing/jig-conformance`;
ruling 183).
Removes: nothing.

## Interface sketches

Compiled at the base in scratch modules (`src/sketch_w3.rs` of
`jig-domain-tasks` and `jig-domain-people`, bodies `unimplemented!()`
and parameters prefixed `_`, under `#![expect(clippy::unimplemented,
dead_code, clippy::too_many_arguments, reason = "interface sketch")]`,
with `use alloc::boxed::Box;` in these `no_std` crates) with `cargo clippy -p jig-domain-tasks -p jig-domain-people -p jig-domain-core
--all-targets -- -D warnings`; the variants shown inside existing enums
were compiled as enums of their own (`AskSketch`, `OutcomeSketch`).

```rust
// crates/jig-domain-tasks/src/control.rs (4.1, 4.2)

/// One tool change to a live task, authenticated by the root.
pub enum Control {
    /// Close a task's subtree with a common bounded reason: at once, or once a
    /// grace of at most `Limits::grace` ends, unless the task ended first.
    Cancel {
        reason: Box<[u8]>,
        grace: Option<Duration>,
        /// The root-issued number of the word asking the task to finish; given
        /// with `grace` and only with it, refused as `Refusal::Read` otherwise.
        message: Option<u64>,
    },
    /// Lift a hold and reset the failure counts it exhausted.
    Release { again: Option<bool>, message: Option<u64> },
}

/// Whether an amendment tracks a task, with its priority, or untracks it.
pub enum Tracking {
    /// A goal from this commit, at this priority; within `Limits::project_goals`.
    Tracked { priority: u32 },
    /// No goal from this commit.
    Untracked,
}
```

```rust
// crates/jig-domain-tasks/src/grace.rs (4.1)

/// A grace kept with its task: who asked, when it ends, the cancel's reason.
pub struct Grace {
    /// The cancel's maker: a party, or a task above.
    pub by: Party,
    /// The wall time at which the task is cancelled unless it ended first.
    pub until: Wall,
    /// The reason the cancel at `until` carries.
    pub reason: Box<[u8]>,
}

/// Ask a live task to finish by `until`, or cancel it at once when it cannot
/// finish on its own (waiting or held), or when a grace it has ends sooner.
pub(crate) fn ask_to_finish(domain: &mut Domain, env: &Env<Limits>, number: u64, by: Party,
    until: Wall, message: u64, reason: &[u8], out: &mut Queue<Request>) -> Result<(), Refusal>;
/// Arm, keep or drop the task's grace alarm from its saved `Grace`, restart included.
pub(crate) fn schedule(domain: &mut Domain, env: &Env<Limits>, number: u64);
/// Cancel the task whose grace ended, with the grace's `by` and `reason`.
pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
```

```rust
// crates/jig-domain-tasks/src/compound.rs (4.4); the checks are 4.3's

/// One operation of a party's compound request, as the tasks child admits it.
pub enum Operation {
    /// `Control::Cancel` of one task and its tree.
    Cancel { task: u64, reason: Box<[u8]>, grace: Option<Duration>, message: Option<u64> },
    /// `Event::Move` of one task to the request's person, from the event's pool.
    Move { task: u64, reason: Box<[u8]> },
    /// `Event::Amend` of one task.
    Amend { task: u64, message: u64, stop_run: bool, amendment: Box<Amendment> },
    /// `Control::Release` of one held task.
    Release { task: u64, again: Option<bool>, message: Option<u64> },
    /// `Event::AuthorizedMessage`'s words, releasing a party's stop when asked.
    Message { task: u64, word: Word, release: bool },
    /// `Event::Hold` for the person's stop of one task's run.
    Stop { task: u64 },
}

// Event::Compound, sent by the core (4.6); answered `Request::Done`.
Compound {
    reply_to: ReplyTo,
    person: u64,
    project: u32,
    /// At most `Limits::tasks`; the core sends at most the people child's `operations`.
    operations: List<Operation>,
    /// The person's pool, counted once across its `Move`s, as `Event::Move` names it.
    period: u64,
    pool_budget: u64,
    period_budget: u64,
},
// Request::CompoundRefused: the first operation that failed, by its place, and why.
CompoundRefused { reply_to: ReplyTo, at: u32, problem: Problem },

/// What the operations checked so far leave, read by each later check; dropped
/// with the decision; bounded by the request's operations.
pub(crate) struct Staged {
    /// Tasks an earlier cancel without a grace closes, its subtree included.
    pub(crate) closing: List<u64>,
    /// Tasks an earlier move made the person's.
    pub(crate) moved: List<u64>,
    /// The pool's reservation the moves so far take.
    pub(crate) reserved: u64,
    /// Revisions an earlier amendment or release moves.
    pub(crate) revisions: List<(u64, Revisions)>,
}

pub(crate) fn check_amend(domain: &Domain, staged: &Staged, env: &Env<Limits>, by: Party,
    number: u64, message: u64, stop_run: bool, amendment: Amendment) -> Result<AmendPlan, Refusal>;
pub(crate) fn apply_amend(domain: &mut Domain, env: &Env<Limits>, by: Party, plan: AmendPlan,
    out: &mut Queue<Request>);
```

The compound's algorithm (the order is what matters):

```rust
let mut staged = Staged::new(&event.operations);
let mut plans = List::with_capacity(event.operations.len());
for (at, operation) in event.operations.iter().enumerate() {
    match check(domain, &staged, env, &event, operation) {
        Ok(plan) => { staged.after(&plan); plans.push(plan) }
        Err(problem) => return out.push(Request::CompoundRefused { reply_to, at, problem }),
    }
}
if staged.reserved > pool_left(domain, &event) { return refuse(at_first_move, Refusal::Funding) }
for plan in plans { apply(domain, env, plan, out) }   // refuses nothing
out.push(Request::Done { reply_to });
```

```rust
// crates/jig-domain-people/src/compound.rs and boundary.rs (4.5, 4.6)

/// Whether a maintainer's amendment tracks a task, with its priority, or
/// untracks it; mirrors `jig_domain_tasks::Tracking` variant for variant.
pub enum Tracking { Tracked { priority: u32 }, Untracked }

/// One operation of a compound request; each mirrors the `Ask` of its name,
/// less its `project`, which the request names once.
pub enum Operation {
    /// As `Ask::Cancel`, with or without a grace.
    Cancel { task: u64, reason: Box<[u8]>, grace: Option<Duration> },
    /// As `Ask::Move`.
    Move { task: u64, reason: Box<[u8]> },
    /// As `Ask::Amend`, priority included; a priority-only amendment names revision 0.
    Amend { task: u64, revision: u64, amendment: Box<Amendment>, release: Option<u64> },
    /// As `Ask::Release`.
    Release { task: u64, revision: u64, again: Option<bool> },
    /// As `Ask::Say`: words to a task in the party's tree, releasing their stop when asked.
    Message { task: u64, words: Box<[u8]>, release: bool },
    /// As `Ask::Stop`.
    Stop { task: u64 },
}

// in `Ask`:
/// Cancel a task and its tree with a bounded reason, at once or after a grace.
Cancel { project: u32, task: u64, reason: Box<[u8]>, grace: Option<Duration> },
/// Several operations on one project's tasks, made in one decision, or none of them.
Compound { project: u32, operations: List<Operation> },

// in `Outcome` (which stays `Copy`):
/// The operations were made, in order; each took one message number, the one at
/// place `i` `first_message + i`, unused where it added no message.
Compound { project: u32, operations: u32, first_message: u64 },
/// Nothing was made: the operation at zero-based place `at` failed, for `why`.
CompoundRefused { at: u32, why: Refusal },

// in `LimitKind`:
/// A grace longer than the tasks child's longest.
Grace,
/// The project's live goals are at goals per project.
Goals,
/// More operations than operations per request, or none.
Operations,
```

## Questions to raise when reached

- **A compound's answer** (contract.md, 3.4: "done, with each
  operation's outcome in order"): what an operation's outcome holds
  beyond "made". Answered by the designer, 2026-10-16: made, with a
  message number by place, and nothing more (Choices, "The compound's
  answer stays `Copy`, numbered by place").
- **One task named by two operations** of one compound (two amendments
  of its spec): this plan checks the second against the revision the
  first leaves (`Staged.revisions`), so a client names the first's
  revision plus one; the design says "each checked as it would be alone
  against the state those before it leave", which this reads as that.
  Raise it if a test of 04.4 finds the reading changes a story.

## Done when

- jig's `origin/main` has `Step: jig-04.1` to `Step: jig-04.7`
  (`git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep -c
  '^jig-04\.'` prints 7).
- `git -C ~/src/rust/jig grep -n -w chat_close origin/main -- crates`
  prints nothing.
- `cargo nextest run -p jig-core-world -E
  'test(/^a_compound_|^a_partys_cancel_with_a_grace/)'`, run once in a
  worktree of jig's `origin/main`, passes 6 tests.
