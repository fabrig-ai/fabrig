# Session 02: a task's label, flags and last activity; `AsAllowed`; the escalation holder; a task's goal

The first half of the engine's plumbing for parties and tasks (jig's
`docs/design/STATUS.md`, section 3, step 2). A task gains the fields the
engine keeps for each task and that the client's sugar is built from:
its label, its flags (conversation, listed, its retention class) and its
last activity, kept in the tasks family, which moves to v3 here. One
function gives the goal a task serves, the nearest tracked task at or
above it, and the task document, the task filter, watch marks and
projections read it. A party becomes a root's requester only while
someone holds the root's escalations, and the people family moves to v3
for that refusal. The authority child resolves a
budget given `AsAllowed`, and a chat's budget is resolved through it.
Nothing here adds an operation: parties' `Create`, `Propose` and
`Withdraw` are jig 03's, on what this session lands.

## Read first

- jig's design: `domain/tasks.md`, sections 3 (a task: "Its label", "Its
  flags", "Its times"), 4 ("A party's root has someone to escalate to", "A
  budget as `AsAllowed`"), 7 (label and flags never amended), 12 (the
  stories of what parties and agents ask alike); `domain/people.md`,
  sections 4 ("Who holds escalations") and 5.1 ("A party's root has
  someone to escalate to"); `domain/authority.md`, sections 6 (the
  escalation role), 7 (`AsAllowed`), 8.1 and 8.4; `domain/core.md`, 6.2
  ("The goal a task serves"); `client/domain/contract.md`, 3.2 ("A task's
  fields cross as data", "`AsAllowed`", "A party's root has someone to
  escalate to") and 4 (*no one to escalate to*); `protocol/store.md`, 5.1
  and 5.3; `domain/testing.md`, sections 3 and 4.
- The plan: `README.md` (Rules), `code-organization.md` (6, 7),
  `decisions.md` 9 and 11.
- `findings/engine-parties-and-tasks.md`: section 1 (Label and flags; Last
  activity; `AsAllowed`; A task's goal; The root's escalation holder),
  section 3, section 5.
- skein's `programming-model.md`, sections 4.4, 4.5, 6.3 and 10.3;
  `testing-strategy.md`, sections 2.1, 2.2 and 7.

## What the design asks

- **A task's label and flags** (tasks.md, 3): "an opaque 32-bit number the
  application gives ... jig gives it no meaning"; flags *conversation*,
  *listed* and a retention class, *kept* or *forgotten once read*. "The
  label and the flags are what a task is made as: no amendment changes
  them (section 7)." Documents carry them (contract.md, 3.2).
- **Last activity** (tasks.md, 3): "when it last heard words, its run last
  committed a turn, or one of its delegates last ended, and until then
  when it is made."
- **The goal a task serves** (core.md, 6.2): "the nearest tracked task at
  or above it in its tree, itself included". Three computations disagree
  at the base (findings, 1 and 3; decisions.md, 11), and projections are
  keyed by the tree's root.
- **The root's escalation holder** (people.md, 5.1): "A party becomes a
  root's requester, by `Create`, by `Move`, or by accepting a proposal
  whose tasks are to be theirs, only when its escalations have a holder:
  their role decides escalations, or the project's policy names an
  escalation role (section 4). Otherwise the request is refused, saying
  so". The base checks it only for a chat, by the wrong rule
  (`crates/jig-domain-core/src/roles.rs:101-106`), and refuses person
  tasks addressed to a policy role (`delegation.rs:170`, `stored.rs:446`).
- **`AsAllowed`** (authority.md, 7): "*for a party,* what their pool has
  left, capped by their role's spend; ... *never more* than the cap the
  batch names with it ... It is resolved after the batch's named amounts
  are counted, from what they leave." The base resolves a chat's budget
  by hand (`crates/jig-domain-core/src/funding.rs:40-54`).

## Choices this plan makes

- **The flags** are a `Copy` struct of two booleans and a retention enum,
  in a module of their own in `jig-domain-tasks`. The paths that make
  tasks at the base give them, in place: a chat is a kept conversation,
  a goal is listed and kept, the other tasks (an agent's or a
  procedure's delegates, a recurring batch, a connector's repair) are
  `Flags::PLAIN` (not a conversation, not listed, kept). Each gets label
  0; templates give labels from jig 09.
- **"Heard words"** is a message of kind `Words`, `Question` or `Answer`
  admitted to its inbox, the three kinds a party or a task sends as words
  (`crates/jig-domain-tasks/src/inbox.rs:191-192`).
- **The well-formedness checks of a batch** that name flags or
  `AsAllowed` (the conversation flag on an agent's task only; at most one
  `AsAllowed`) land with `Create` in jig 03, the first creator that names
  flags or can give two. No path of this session can.
- **`AsAllowed` is an authority query**, `jig_domain_authority::as_allowed`,
  which the core calls before the batch's check and whose result it
  writes into the task's authority. The batch check stays as it is: it
  allocates nothing (`check.rs:110-113`), and its findings, which the core
  family stores, stay unchanged (the core family's v3 is jig 06.1's).
- **The people family moves to v3 at 02.3** (README, Rules, "Format
  versions"): the escalation holder's refusal is a people record, and jig
  03, 04 and 05 then edit v3 in place.
- **The wire stays v7** (jig 14 changes its requests, jig 22 its
  documents). The clients' component maps
  `people::Refusal::NoEscalationHolder` onto v7's `NotOffered`, which is
  what a client is told for this case at the base, and jig 14 gives it a
  record of its own. The core's `Task` document carries label, flags,
  last activity and goal; the component's translation reads none of them
  until jig 22.

## Rules that bind it

- **Format versions** (README, Rules): tasks v3 changes once, at 02.1, and
  people v3 once, at 02.3: each `git mv`s its schema, its generated module
  and its goldens to v3 and regenerates, and the old version's goldens go;
  later changes edit v3 in place. Never edit a v2 schema or its generated
  module.
- **Files over 800 lines get at most 40 more lines each**
  (code-organization.md, 7), checked with `git diff --numstat
  origin/main...HEAD -- <file>`, first column. Here:
  `crates/jig-domain-tasks/src/{boundary.rs, domain.rs, stored.rs}`,
  `crates/jig-domain-authority/src/check.rs`,
  `crates/jig-domain-core/src/documents/task_tree.rs` and the test files
  `crates/jig-domain-core/src/tests.rs`,
  `crates/jig-domain-authority/src/tests_policy.rs`. New code goes in a new
  module beside them.
- **Nothing new is added to the shapes of work** (code-organization.md, 6):
  the chat's and the goal's code (`chats.rs`, `goals.rs`,
  `routing/parties.rs`, `acts/bounds.rs`) is changed in place only, and
  jig 16 sheds it. **jig's files never name the repository downstream of
  jig**: at each increment's end, `git grep -n -i -w <its name> --
  crates testing tests examples scripts` prints nothing (README, Rules).

## Increments

### 02.1 tasks v3: a task's label, flags and last activity
Scope: tasks.md, 3 (label, flags, last activity) and 7 (never amended);
store.md, 5.1 and 5.3; contract.md, 3.2 ("documents carry them"). The
tasks family moves to v3 with the new fields; the paths that make tasks
give them; the core's `Task` document carries them.
Needs: nothing.
Files:
- `crates/jig-domain-tasks/src/flags.rs` (new), `lib.rs` (its export),
  `boundary.rs` (at most 40: the fields of `New` at :760 and `TaskRecord`
  at :844), `domain.rs` (at most 40: the made record at :1159 copies
  label and flags, `last_activity: env.wall`), `inbox.rs` (`message`,
  :169), `admission.rs` (`turn`, :42), `closing.rs` (the requester's
  record as a delegate ends, :381-388), `stored.rs` (at most 40: restore
  refuses the conversation flag on a task that is not an agent's, and a
  last activity before `created_at`, as `Refusal::Restore`).
- `crates/jig-domain-core/src/routing/parties.rs` (the goal's `New` at
  :169, listed and kept; the chat's at :226, a kept conversation),
  `routing/run_delegation.rs` (:102, :294, :439, :610) and
  `routing/tasks.rs` (:1322), `Flags::PLAIN`;
  `documents/task_tree.rs` (at most 40: `Task` at :182 and
  `task_from_row` at :645).
- `crates/jig-protocol-store`: `schema/tasks-v2.schema` moved to
  `schema/tasks-v3.schema` (`family jig-store-tasks 3`; `record Flags {
  conversation: bool listed: bool retention: Retention }`, `enum Retention
  { kept, forgotten_once_read }`; `Member` and `TaskDefinition` gain
  `label: u32 flags: Flags`; `TaskHistory` gains `last_activity: u64`),
  `src/generated/tasks_v2.rs` moved to `tasks_v3.rs` and regenerated,
  `golden/tasks/v2/` moved to `golden/tasks/v3/` and regenerated, and the
  older `golden/tasks/v1/` goes too, since nothing reads it (README,
  Rules); `src/format.rs:7` (`tasks: 3`); the
  translations of the new fields, both ways, in `src/codec/tasks/record.rs`
  and `src/codec/tasks/phase.rs` (`Member`).
- The `tasks_v2` module renamed `tasks_v3` in its 19 users (search S3): `crates/jig-protocol-store/src/{codec.rs, lib.rs,
  sizes.rs, codec/core.rs, codec/tasks.rs}`, the 8 modules of
  `src/codec/tasks/`, `src/tests/codec.rs`, `src/tests/codec/{phase.rs,
  pieces.rs, record.rs}`, `tests/store-codecs/tests/{fuzzy_decode.rs,
  memory.rs}`; `tests/store-codecs/tests/drift.rs:46` (`check("tasks",
  3)`); the format literals `crates/jig-protocol-store/src/tests/{open.rs:91,
  format.rs:36, format.rs:53, commits.rs:87}`.
- Users of the changed types, edited to their new form with nothing else
  changed (README, Rules): `New` literals in 13 files (search S1) and
  `TaskRecord` literals in 15 files, its definition's included (search
  S2); among them
  `tests/tasks/src/world.rs:72` (`task`), `crates/jig-domain-core/src/tests.rs:357`
  and :548, `crates/jig-protocol-store/src/tests/fixtures.rs:194`
  (`task_creation`) and `tests/fixtures/record.rs:15` (`task_record`);
  `documents::Task`'s literal at `tests/client/src/joint/wire/task.rs:114`.
  The downstream repository's repin absorbs `New`'s and `TaskRecord`'s
  new fields (search S18: 5 files).
Shape:
- `jig_domain_tasks::Flags { conversation: bool, listed: bool, retention:
  Retention }` with `Flags::PLAIN`; `jig_domain_tasks::Retention { Kept,
  ForgottenOnceRead }` (sketch 1).
- `New` gains `label: u32` and `flags: Flags`; `TaskRecord` gains `label:
  u32`, `flags: Flags` and `last_activity: Wall`. Label and flags are
  copied from `New` when the task is made and never written after; no
  `Amendment` field reaches them.
- `last_activity` is `env.wall` when the task is made, and is set to
  `env.wall` when a message of kind `Words`, `Question` or `Answer` is
  admitted to its inbox, when its run's turn is admitted, and on the
  requester's record when one of its delegates ends.
- `jig_domain_core::documents::Task` gains `label: u32`, `flags:
  tasks::Flags` and `last_activity: Wall`, read from the row.
Test shape:
- `crates/jig-protocol-store/src/tests/codec/record.rs`: the case
  `a_task_records_label_flags_and_last_activity_read_back` in the table of
  `task_records_read_back_as_written`; fixtures `current_record()`
  (record.rs:18) and `saved_tasks` (`tests/codec.rs:106`); the fixture
  `task_record()` (`tests/fixtures/record.rs:15`) gets label 7, flags {
  conversation: true, listed: false, retention: Kept } and a last
  activity, and `task_creation()` (`tests/fixtures.rs:194`) label 7 and
  `Flags::PLAIN`, so the existing cases carry them. Reference:

  ```rust
  fn a_task_records_label_flags_and_last_activity_read_back() {
      let mut record = current_record();
      record.label = 0x0102_0304;
      record.flags = domain::Flags { conversation: false, listed: true, retention: domain::Retention::ForgottenOnceRead };
      record.last_activity = Wall::from_nanos(9_000_000_000);
      let row = live_record(record);
      assert_eq!(saved_tasks(&row), Ok(row));
  }
  ```
- `tests/store-codecs/tests/drift.rs`:
  `the_tasks_familys_code_and_goldens_have_no_drift` reads v3.
- `tests/tasks/tests/activity.rs` (the tasks world; helpers `write` :35 and
  `take` :52; `World::new` and `task` from `tests/tasks/src/world.rs:189`
  and :72; limits the world's `LIMITS`, `tests/tasks/src/world.rs:17`, as
  the existing tests take them; this increment adds no limit):
  `a_tasks_label_and_flags_are_kept_as_made`,
  `a_tasks_last_activity_is_when_it_was_made`,
  `a_tasks_last_activity_moves_when_it_hears_words`,
  `a_tasks_last_activity_moves_when_its_run_commits_a_turn`,
  `a_requesters_last_activity_moves_when_a_delegate_ends`. Reference:

  ```rust
  #[test]
  fn a_tasks_label_and_flags_are_kept_as_made() {
      let mut world = World::new(331, LIMITS);
      let mut made = task(1, &[]);
      made.label = 7;
      made.flags = Flags { conversation: true, listed: false, retention: Retention::ForgottenOnceRead };
      assert_eq!(world.make(Party::Person(9), vec![made]), Reply::Made(vec![1]));
      assert_eq!(world.record(1).label, 7);
      assert_eq!(world.record(1).flags, Flags { conversation: true, listed: false, retention: Retention::ForgottenOnceRead });
  }
  ```
- `tests/core/tests/fields.rs` (new, the joint world): the Proof's test,
  below, written out; `World::new` and `World::restart`
  (`tests/core/src/effects/constructors.rs:71`, `effects/store.rs:138`),
  `World::set_clock` (`tests/core/src/effects.rs:124`); limits the world's
  `limits()` (`tests/core/src/world.rs:17`), unchanged.
- Ceilings: each unit test under 20 ms, each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/fields.rs`,
`a_chats_flags_and_last_activity_survive_a_cold_restart`: a chat made by
`StartChat` hears its person's words at a wall time of 9 s, the engine
restarts cold from the store, and the restored record is a kept
conversation with label 0 whose last activity is 9 s. It runs the real
core and children on the real store's codecs. Under 0.5 s. Run once,
`cargo nextest run -p jig-core-world -E
'test(a_chats_flags_and_last_activity_survive_a_cold_restart)'`; it
passes, and nothing else is wanted.

```rust
#[test]
fn a_chats_flags_and_last_activity_survive_a_cold_restart() {
    let mut world = World::new(171, false);
    world.set_clock(Time::from_nanos(9_000_000_000), Wall::from_nanos(9_000_000_000));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        reply_to: ReplyTo::new(Token::new(8001)),
        sign_in: world.sign_in.expect("signed in"),
        key: [1; 16],
        ask: people::Ask::Say { project: 1, task: 1, words: b"heard at nine".as_slice().into(), release: false },
    })));
    world.restart(171, false);
    let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Live(1)))]
    else {
        panic!("the chat is live after the restart")
    };
    assert_eq!(row.label, 0);
    assert_eq!(row.flags, tasks::Flags { conversation: true, listed: false, retention: tasks::Retention::Kept });
    assert_eq!(row.last_activity, Wall::from_nanos(9_000_000_000));
}
```
Don't: move any task's label or flags after it is made; add a field to
`Amendment`; carry label, flags or last activity on the wire (jig 22);
order or filter anything by them (task queries are jig 05's).
Size: at most 150 lines (a codec increment: lands within 2.5 times
without asking).
Removes: `schema/tasks-v2.schema`, `src/generated/tasks_v2.rs` and
`golden/tasks/v2/` (moved to v3); `golden/tasks/v1/`.

### 02.2 The goal a task serves
Scope: core.md, 6.2 ("The goal a task serves is the nearest tracked task
at or above it in its tree, itself included"); decisions.md, 11. One
function; the task document, the task filter and watch marks read it;
projections are keyed by it, so a tracked task that is not a root has a
projection of its own, fed by its subtree; and a goal's notes scope is
its goal's (core.md, 6.2: "a scope for notes, which its subtree reads and
writes").
Needs: nothing.
Files: `crates/jig-domain-core/src/goal_of.rs` (new), `lib.rs` (its
module); its users, each computation replaced by it:
`documents/task_tree.rs:653-667` (`Task.goal`, at most 40),
`documents/project_tasks.rs:90-98` (`fn goal` goes),
`routing/watch_marks.rs:193-197`, `projections.rs:156-160` and :189 (a
row feeds its goal's projection) and :205-242 (a milestone feeds its
goal's projection); the notes' goal scope, the tree's root at the base:
`routing/run_preparation.rs:224` (the brief's scopes),
`routing/mod.rs:173` (a note tool's scope) and :252 (`note_readable`).
Shape: `impl Core { #[must_use] pub fn goal_of(&self, row:
&tasks::TaskRecord) -> Option<u64> }` (sketch 2, the walk written out). A
row whose goal is `None` feeds no projection. `ProjectionTaskRow.goal`
and `ProjectionMilestoneRow.goal` (the core family) keep their format and
now hold the goal's number; nothing is translated (README, Rules). A
run's brief carries `notes::Scope::Goal { project, goal }` for
`goal_of(row)`, a note tool's goal scope and `note_readable` match
`goal_of(row)`, and a task with no goal has no goal scope: its brief
carries none, and a goal-scoped note is out of its scopes. Tests that
read a goal's notes from an untracked tree (`rg -n 'Scope::Goal'
crates/jig-domain-core/src tests/core`, once) follow the rule, their
names kept.
Test shape:
- `crates/jig-domain-core/src/tests/goal_of.rs` (new, a module of
  `tests.rs`): fixtures `core()` (`tests.rs:308`) and `make_task`
  (`tests.rs:343`), and a helper in this file, `make_below(core, limits,
  parent, number, tracked)`, which sends `tasks::Event::Make` with
  `creator: tasks::Party::Task(parent)` as `make_task` sends its root.
  Tests: `a_tracked_task_is_the_goal_it_serves`,
  `an_untracked_delegate_serves_its_nearest_tracked_ancestor`,
  `a_tracked_delegate_under_a_tracked_root_serves_itself`,
  `a_tree_with_no_tracked_task_serves_no_goal`,
  `the_task_document_names_the_nearest_tracked_goal`,
  `a_delegates_note_scope_is_its_nearest_goal`,
  `a_task_with_no_goal_reads_no_goal_scoped_note`. Reference:

  ```rust
  #[test]
  fn a_tracked_delegate_under_a_tracked_root_serves_itself() {
      let (mut core, limits) = core();
      make_tracked(&mut core, &limits, 10, 1);
      make_below(&mut core, &limits, 10, 11, Some(2));
      make_below(&mut core, &limits, 11, 12, None);
      let row = core.tasks.task(12).expect("live delegate");
      assert_eq!(core.goal_of(row), Some(11));
  }
  ```

  (`make_tracked` is `make_task` with `tracked: Some(priority)`, a second
  helper in this file.)
- Ceilings: each unit test under 20 ms.
Proof: joint layer, the existing `projections_scenario_family`
(`tests/core/tests/projections.rs:332`), unchanged, whose scenario
`a_delegate_of_a_delegate_ending_feeds_the_whole_plan_once_and_closing_waits_for_its_write`
(:202) shows a goal's projection still fed by its subtree through
`goal_of`. Under 0.5 s. Run once, `cargo nextest run -p
jig-core-world -E 'test(projections_scenario_family)'`; it passes, and
nothing else is wanted. A tracked task that is not a root cannot be made
above the tasks child before jig 04's amendment tracks one ("tracked and
priority amended"), whose Proof connects the nearest-goal rule at the
joint layer; until then the unit tests show it.
Don't: compute a goal anywhere but `Core::goal_of`; change the core
family's records.
Size: at most 90 lines.
Removes: `documents/project_tasks.rs`'s `fn goal`, the goal loop of
`task_from_row`, `watch_marks.rs`'s root test, the notes' root scope.

### 02.3 people v3: the root's escalation holder; person tasks for policy roles
Scope: tasks.md, 4 ("A party's root has someone to escalate to");
people.md, 4 and 5.1; authority.md, 6 and 8.4; contract.md, 4 (*no one to
escalate to*); findings, 3 (the chat's check by role number, and policy
roles' person tasks refused). The people family moves to v3.
Needs: nothing.
Files:
- `crates/jig-domain-authority/src/holders.rs` (new), `lib.rs`.
- `crates/jig-domain-people/src/refusals.rs` (`Refusal::NoEscalationHolder`).
- `crates/jig-domain-core/src/roles.rs:101-106` (the chat), `acts/bounds.rs`
  (`goal_start`, :221, after its standing check),
  `routing/party_decisions.rs` (`person_proposal_accept`, a batch with
  `as_holder`, :209: the accepter becomes the roots' requester),
  `delegation.rs:170-174` and `stored.rs:446` (a person task for any role
  number).
- `crates/jig-protocol-store`: `schema/people-v2.schema` moved to
  `schema/people-v3.schema` (`family jig-store-people 3`; `enum Refusal`
  gains `no_escalation_holder`), `src/generated/people_v2.rs` moved to
  `people_v3.rs` and regenerated, `golden/people/v2/` moved to
  `golden/people/v3/` and regenerated, and `golden/people/v1/` goes;
  `src/format.rs:7` (`people: 3`);
  `src/codec/people/outcomes/facts.rs` (the refusal, both ways); the
  `people_v2` module renamed `people_v3` in its 8 users (search S4):
  `crates/jig-protocol-store/src/{codec.rs, codec/people.rs, lib.rs,
  sizes.rs, tests/codec.rs, tests/codec/pieces.rs}`,
  `tests/store-codecs/tests/{fuzzy_decode.rs, memory.rs}`;
  `tests/store-codecs/tests/drift.rs:51` (`check("people", 3)`); the
  format literals `crates/jig-protocol-store/src/tests/{commits.rs:88,
  format.rs:54}`.
- An arm for the new refusal in each exhaustive match over
  `people::Refusal`, 16 files besides the generated module (search S8):
  `crates/jig-domain-people/src/{capacity.rs, domain.rs, tests.rs,
  tests/closing_chat.rs}`, `crates/jig-protocol-engine-clients/src/translate/refusals.rs`
  (onto v7's `NotOffered`, Choices) and `src/tests/fixtures/answers/refusals.rs`,
  `crates/jig-protocol-store/src/codec/people/outcomes/facts.rs` and the
  fixture `refusal_values()` (`src/tests/fixtures/people_outcomes.rs:123`),
  `testing/jig-test-domain/src/application_facts.rs`, `tests/core/src/peers.rs`,
  `tests/client/src/joint/wire/{notes.rs, offers.rs, refusals.rs}`,
  `tests/people/src/{referee.rs, world.rs}`, `tests/people/tests/simulation.rs`.
  The downstream repository's repin absorbs the module's new name and the
  new refusal (search S18: 3 files name `people_v2`, 16 match
  `people::Refusal`).
Shape:
- `impl jig_domain_authority::Domain { #[must_use] pub fn
  escalations_held(&self, project: u32, role: u32) -> bool }` (sketch 3):
  true when the role `role` of `project`'s policy decides
  `ProposalKind::Escalation`, or the policy's `escalation_role` is
  `Some`; false for an unknown project or role.
- `jig_domain_people::Refusal::NoEscalationHolder` (sketch 4).
- The core refuses `NoEscalationHolder` where a party would become a
  root's requester and `escalations_held` is false: a chat (`chat_admit`,
  in place of the base's `role > 3` test, which answered `NotOffered`), a
  goal (`goal_start`), and the acceptance of a proposed batch whose tasks
  are to be the accepter's. `Move`'s check is jig 04's, by this function.
- `Core::delegation_executor` maps a person task for role `n` to
  `authority::Executor::Role(n)` for any `n`, and the authority check of
  the creator's delegation kinds decides; restore supports a person task
  for any role.
Test shape:
- `crates/jig-domain-authority/src/tests_party.rs` (new; the crate's tests
  share no fixtures module at the base: `domain()`, `policy()` and
  `role()` move from `tests_policy.rs:86-118` to a new
  `crates/jig-domain-authority/src/tests_fixtures.rs`, used by both):
  `a_role_deciding_escalations_holds_its_own_roots`,
  `a_policy_escalation_role_holds_any_roles_roots`,
  `no_escalation_role_and_no_deciding_role_holds_nothing`,
  `an_unknown_project_or_role_holds_nothing`. Reference:

  ```rust
  #[test]
  fn a_policy_escalation_role_holds_any_roles_roots() {
      let domain = domain_with_roles(Proposals(0), Some(5));
      assert!(domain.escalations_held(1, 2));
  }
  ```

  (`domain_with_roles(decides, escalation_role)`, added to
  `tests_fixtures.rs` here, gives project 1 a policy whose role 2 decides
  `decides` and whose role 5 accepts and decides `Proposals::ALL`, as the
  authority child requires of an escalation role (`domain.rs:234-243`),
  and names `escalation_role`.)
- `crates/jig-domain-core/src/tests/escalation_holder.rs` (new; fixture
  `core()`, `tests.rs:308`; `funding_fixtures.rs:2`
  `set_role_authority`): `a_chat_is_refused_while_nobody_holds_its_escalations`,
  `a_goal_is_refused_while_nobody_holds_its_escalations`,
  `a_policy_role_numbered_five_holds_a_chats_escalations`,
  `a_person_task_for_policy_role_five_is_offered_to_delegation`,
  `a_restored_person_task_for_policy_role_five_is_supported`.
- `crates/jig-protocol-store`: `people_outcomes_read_back_as_written`
  covers the new refusal through `refusal_values()`; the drift test
  `the_people_familys_code_and_goldens_have_no_drift` reads v3.
- `tests/core/tests/fields.rs`: the Proof's test. A fixture
  `no_holder_fixture(seed)` in `tests/core/src/stories/offers.rs`, beside
  `no_acceptor_fixture` (:72) and built as it is: role 2 (member) decides
  no proposals, and the policy's `escalation_role` is `None` beside the
  world's `Some(0)` (`tests/core/src/world.rs:233`), the reason being the
  story. The member is a second party, signed in as
  `tests/core/tests/client_peer.rs:21-43` signs one in and given role 2
  by `people::Event::Roles`; the owner's `ChangePolicy` builds its
  `PolicyRole` as `changed_role` does (`tests/core/tests/funding.rs:122`).
  A second test there, in `no_holder_fixture`'s world,
  `accepting_a_chats_batch_as_ones_own_is_refused_while_nobody_holds_it`,
  makes the refusal in `person_proposal_accept`: the member, whose role
  first decides batches and escalations, starts a chat; the chat's run
  proposes a batch with `as_holder: true` (`core::NamedAction::ProposeBatch`,
  sent as `tests/core/tests/stories.rs:27` sends an action); the owner's
  `ChangePolicy` leaves the member's role deciding batches only; the
  member's `DecideProposal` accepting it is answered
  `Refused(NoEscalationHolder)`, and no task of the batch is made.
- Ceilings: each unit test under 20 ms, each world test under 0.5 s.
Proof: joint layer, `tests/core/tests/fields.rs`,
`a_party_refused_a_root_is_made_once_their_role_holds_it` (tasks.md, 12:
"a party refused a root while their escalations have no holder, and made
once the policy names one"): a member whose role decides no escalations,
in a project whose policy names no escalation role, asks `StartChat` and
is answered `Refused(NoEscalationHolder)`; an owner's `ChangePolicy`
gives the member's role the escalation decision; the member's next
`StartChat` is answered `Started`. Under 0.5 s. Run once, `cargo
nextest run -p jig-core-world -E
'test(a_party_refused_a_root_is_made_once_their_role_holds_it)'`; it
passes, and nothing else is wanted.
Don't: refuse by role number anywhere (`git grep -n -E 'role (<=|>) 3'
-- crates/jig-domain-core/src` prints only `policy.rs:158`, which checks
a policy's own mapping); check the holder in `Move` (jig 04).
Size: at most 100 lines (a codec increment).
Removes: `schema/people-v2.schema`, `src/generated/people_v2.rs` and
`golden/people/v2/` (moved to v3); `golden/people/v1/`; the base's `role
> 3` and `role <= 3` tests.

### 02.4 `AsAllowed`
Scope: tasks.md, 4 ("A budget as `AsAllowed`"); authority.md, 7 and 8.4;
contract.md, 3.2. The authority child resolves a budget given
`AsAllowed` for a party; a chat's budget is resolved through it, in place
of `chat_authority`'s minimum (`funding.rs:50-51`).
Needs: nothing.
Files: `crates/jig-domain-authority/src/allowed.rs` (new), `lib.rs` (its
export); `crates/jig-domain-core/src/funding.rs:40-54` (`chat_authority`
takes its spend from `as_allowed`; its users, `roles.rs:92`,
`routing/parties.rs:225`, `acts/bounds.rs:239` and
`documents/small.rs:110`, are unchanged).
Shape: `#[must_use] pub fn as_allowed(domain: &Domain, ask: &PersonAsk,
named: u64, cap: Option<u64>) -> u64` (sketch 3): "what their pool has
left, capped by their role's spend" (the role's authority's spend, which
the authority child admits below the project's ceiling, `rules.rs:52`),
less `named`, at most `cap`; zero for an unknown project or role, and
when nothing fits. The algorithm:

```rust
let Some(role) = domain.role(ask.project, ask.role) else { return 0 };
let fits = left(ask.pool).min(role.authority.budget.spend);
let given = fits.saturating_sub(named);
match cap {
    Some(cap) => given.min(cap),
    None => given,
}
```

A chat's batch names no other amount: `chat_authority` sets
`budget.spend` to `as_allowed(&self.authority, &ask, 0, None)`, `ask` the
party's `PersonAsk` for the chat. What was given reaches a party in
`Create`'s answer (jig 03.2); a chat's `Started` answer is unchanged.
Test shape:
- `crates/jig-domain-authority/src/tests_party.rs` (02.3's file and
  fixtures): `as_allowed_gives_what_the_pool_has_left_under_the_role`,
  `as_allowed_counts_the_batchs_named_amounts_first`,
  `as_allowed_stops_at_the_cap_the_batch_names`,
  `as_allowed_for_an_unknown_role_gives_nothing`,
  `as_allowed_gives_nothing_when_named_amounts_take_the_pool`, and
  `as_allowed_over_generated_pools_roles_and_caps_is_the_least_bound`, a
  table of 64 generated cases from a seeded `skein_lib::Rng` against the
  minimum written out in the test, under 20 ms. Reference:

  ```rust
  #[test]
  fn as_allowed_counts_the_batchs_named_amounts_first() {
      let domain = domain_with_role_spend(80);
      let ask = PersonAsk { project: 1, role: 2, pool: numbers(100), tasks_left: 8, request: PersonRequest::Watch };
      assert_eq!(as_allowed(&domain, &ask, 30, None), 50);
  }
  ```

  (`domain_with_role_spend` added to `tests_fixtures.rs`: project 1's
  role 2 with authority spend 80; `numbers` moves there from
  `tests_policy.rs:33`.)
- Ceilings: each unit test under 20 ms.
Proof: joint layer, the existing
`a_chat_takes_the_most_that_fits_under_its_persons_role`
(`tests/core/tests/funding.rs:73`), unchanged: a chat made by
`StartChat` under a role of spend 80 with 10 left in its pool is given 10
through `as_allowed`, as before. Under 0.5 s. Run once, `cargo
nextest run -p jig-core-world -E
'test(a_chat_takes_the_most_that_fits_under_its_persons_role)'`; it
passes, and nothing else is wanted.
Don't: resolve `AsAllowed` for a task's batch (no agent's tool gives it;
none is planned); change `check_batch`, `Checked` or `Finding`; answer
what was given to a chat's party.
Size: at most 40 lines.
Removes: `chat_authority`'s own minimum of pool and role spend.

## Interface sketches

Compiled at jig's base `a3b53546` as scratch modules of the crates that
hold them, `cargo clippy -p jig-domain-tasks -p
jig-domain-authority -p jig-domain-people -p jig-domain-core
--all-targets -- -D warnings`: passed. Bodies were
`unreachable!("sketch: body in the increment")`, with a typed `let _: (…)
= (…);` for the arguments (the workspace denies `todo` and
`unimplemented`). The sketch of `New` shows each of its fields; the others show the
fields and variants added beside the base's.

```rust
// Sketch 1. crates/jig-domain-tasks/src/flags.rs and boundary.rs (jig 02.1)
/// What a task is made as, kept from its making to its erasure; no amendment changes it
/// (domain/tasks.md, sections 3 and 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    /// Its turns are entries for its requester while that requester is a party; an agent's task only.
    pub conversation: bool,
    /// Once ended, indexed by its project and its label.
    pub listed: bool,
    /// How long it is kept once ended.
    pub retention: Retention,
}
impl Flags {
    /// No conversation, not listed, kept: what an agent's and a procedure's batches make.
    pub const PLAIN: Flags = Flags { conversation: false, listed: false, retention: Retention::Kept };
}
/// A task's retention class (domain/tasks.md, section 3; protocol/store.md, 11.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    /// Kept to the deployment's retention horizon.
    Kept,
    /// Erased once ended, read by its requesting party, and named by no live task.
    ForgottenOnceRead,
}
/// Root-authorized member of an atomic creation batch (domain/tasks.md, sections 3 and 4).
/// Mirrors `Member` in `tasks-v3.schema`, field for field.
pub struct New {
    pub number: u64,
    pub project: u32,
    pub executor: Executor,
    pub spec: Spec,
    pub contract: Contract,
    pub authority: Authority,
    pub numbers: Numbers,
    pub funder: Funder,
    pub dependencies: Box<[u64]>,
    pub holdings: Box<[Holding]>,
    pub wake: WakePolicy,
    pub recurring: Option<Box<RecurringTemplate>>,
    pub tracked: Option<u32>,
    /// Opaque 32-bit label the application gives; no rule of the engine's keys on it.
    pub label: u32,
    /// The engine's flags, checked with the batch.
    pub flags: Flags,
}
// TaskRecord (boundary.rs:844) gains, beside its other fields; mirrors
// `TaskDefinition` (label, flags) and `TaskHistory` (last_activity) in `tasks-v3.schema`:
//     /// As made; never amended.
//     pub label: u32,
//     /// As made; never amended.
//     pub flags: Flags,
//     /// When it last heard words, its run last committed a turn, or a delegate last ended;
//     /// until then, when it was made (domain/tasks.md, section 3).
//     pub last_activity: Wall,

// Sketch 2. crates/jig-domain-core/src/goal_of.rs (jig 02.2)
impl Core {
    /// The goal a task serves: the nearest tracked task at or above it in its tree, itself
    /// included (domain/core.md, 6.2). Documents and projections read it from here.
    #[must_use]
    pub fn goal_of(&self, row: &tasks::TaskRecord) -> Option<u64> {
        let mut current = row;
        for _ in 0..=row.depth {
            if current.tracked.is_some() {
                return Some(current.number);
            }
            match current.requester {
                tasks::Party::Task(parent) => match self.tasks.task(parent) {
                    Some(above) => current = above,
                    None => return None,
                },
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return None,
            }
        }
        None
    }
}

// Sketch 3. crates/jig-domain-authority/src/holders.rs and allowed.rs (jig 02.3, 02.4)
impl Domain {
    /// Whether a party holding `role` in `project` may be a root's requester: their role decides
    /// escalations, or the project's policy names an escalation role (domain/authority.md, 6 and
    /// 8.4). False for an unknown project or role.
    #[must_use]
    pub fn escalations_held(&self, project: u32, role: u32) -> bool;
}
/// What a budget given `AsAllowed` resolves to for a party's batch: what their pool has left,
/// capped by their role's spend, less the batch's `named` amounts, and at most `cap`
/// (domain/authority.md, section 7). Zero means nothing fits, which the caller refuses as
/// lacking budget.
#[must_use]
pub fn as_allowed(domain: &Domain, ask: &PersonAsk, named: u64, cap: Option<u64>) -> u64;

// Sketch 4. crates/jig-domain-people/src/refusals.rs (jig 02.3); mirrors `Refusal` in
// `people-v3.schema`. Refusal gains, beside its other variants:
//     /// The party would become a root's requester while nobody holds its escalations: their
//     /// role decides none, and the project's policy names no escalation role
//     /// (client/domain/contract.md, 4).
//     NoEscalationHolder,
```

Who owns what: the tasks child owns `Flags` and the fields and moves
`last_activity`; the core sets the flags of the tasks it makes and owns
`goal_of`; the authority child owns `escalations_held` and `as_allowed`,
and the core calls both and answers the party.

## Questions to raise when reached

- None the design leaves open here. If an increment finds a chat or a
  goal that the base makes on a path these files do not name, it gives
  it the flags of the Choices and says so in its report.

## Done when

- The four increments have landed on jig's `origin/main`, each with its
  Proof passing in its landing.
- `git -C <a jig worktree at origin/main> grep -n -E 'tasks_v2|people_v2'
  -- crates tests testing examples` prints nothing.
- `git -C <the same worktree> grep -n -E 'role (<=|>) 3' --
  crates/jig-domain-core/src` prints one line, `policy.rs:158`.
- `git -C <the same worktree> grep -n 'fn goal(' --
  crates/jig-domain-core/src/documents/project_tasks.rs` prints nothing.

## Searches at the base

Run once, at jig's `a3b53546` from a worktree's root;
each prints a list of files, generated modules left out (`| grep -v
/generated/`). Session 03's file cites them by the same numbers. `E` is
`crates testing tests examples`; `D8` is `crates/jig-domain-people
crates/jig-domain-core crates/jig-protocol-engine-clients
crates/jig-protocol-store testing/jig-test-domain tests/core tests/people
tests/client/src/joint tests/engine-clients tests/conformance tests/ops`;
`D9` is `crates/jig-domain-people/src crates/jig-domain-core/src
crates/jig-protocol-store/src tests/core tests/people tests/conformance
tests/ops testing/jig-test-domain`; `D10` is `D9` with
`crates/jig-protocol-engine-clients/src tests/client/src/joint`.

| | Command | Printed |
|---|---|---|
| S1 | `rg -l -U --type rust '\bNew \{\s*number' $E` | 13 files |
| S2 | `rg -l --type rust 'TaskRecord \{' $E` | 15 files |
| S3 | `rg -l --type rust 'tasks_v2' $E` | 19 files |
| S4 | `rg -l --type rust 'people_v2' $E` | 8 files |
| S7 | `rg -l -U --type rust 'people::Limits \{\|\bLimits \{\s*people:' $E` | 11 files |
| S8 | `rg -l --type rust 'Refusal::KeyConflict' $D8` | 16 files |
| S9 | `rg -l --type rust 'Ask::SetPool' $D9` | 16 files |
| S10 | `rg -l --type rust 'Outcome::PoolSet' $D10` | 12 files |
| S11 | `rg -l --type rust 'Outcome::GoalProposed\|Outcome::AmendProposed' $D10` | 22 files |
| S13 | `rg -l --type rust 'PersonProposal \{\|\.goal\b.*person_proposal\|PersonProposalState::' $E` | 46 files |
| S14 | `rg -l --type rust 'proposable:' $E` | 46 files |
| S18 | in the downstream repository, `git grep -l -E <pattern> origin/main -- '*.rs'` for `tasks::New \{\|TaskRecord \{`, `people_v2`, `people::Ask::`, `people::Refusal::\|Refusal::KeyConflict` and `Outcome::GoalProposed\|Outcome::AmendProposed` | 5, 3, 28, 16 and 2 files |
