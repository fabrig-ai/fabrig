# Session 05: waiting on, the task line, task queries, `me`, conversations by their flag

jig's step 2, the core's documents for the client's plumbing: what a
task waits on, computed by a new core module from its live subtree and
the entries waiting in it; the task line, "one record for every list of
tasks"; task queries over the live tasks, a fixed set of keys and orders
within a window, read and watched; `me` with the bounds a draft of a new
task is compared with; and conversations known by their flag, with
`Seen` over a conversation's turns admitted on it. The ended part of task
queries is jig 08.5's, after the store's indexes; the templates `me`
serves are jig 09.3's. The shaped subjects (`Goals`, `Conversations`,
`Tasks { project, filter }`) stay until jig 16 removes them, and nothing
new is added to them. The wire is not changed: jig 22 carries these
documents to the client.

## Read first

- jig's `docs/design/domain/engine.md`: 1, 5.3 ("Ended tasks are read
  through the store's indexes", for what 05.3 leaves to jig 08.5), 6
  (read positions restored at start), 11, 11.1 to 11.4, 13 ("What reads
  and deliveries compute").
- jig's `docs/design/client/domain/contract.md`: 5.1 (`me`, a task, a
  task line), 5.3 (creating bounds in `me`), 6, 6.1, 7.1.
- jig's `docs/design/domain/tasks.md`: 3 (label, flags, times, "What a
  task waits on").
- jig's `docs/design/domain/people.md`: 6 (read positions, a
  conversation).
- jig's `docs/design/domain/core.md`: 6.2 (task queries filter on
  tracked and order by priority).
- jig's `docs/design/domain/testing.md`: 3 (views), 4 (the core's world:
  "a task query's rows against every task the scenario made, filtered
  and ordered as the query asks").
- This plan's `findings/engine-parties-and-tasks.md`, 1 ("Waiting on",
  "`me` and task queries"), 3 (a conversation known by its charter);
  `findings/engine-agents-store-config.md`, 1 ("Task queries' reads").
- skein's `programming-model.md`: 4.5, 6.3; `testing-strategy.md`: 2.1,
  2.2, 7.

## What the design asks

- **What a task waits on** (engine.md, 11.3), "the first of: *a party,*
  when an entry in the task's live subtree, itself included, waits for a
  party: a question put to one, a proposal or an escalation waiting at
  parties, a person task addressed to a party or a role. It names whom
  the oldest such entry waits for, a party or a role, and how many wait;
  *nothing,* when it is working: its run live and not waiting; *its
  delegates,* when it has live delegates; *news,* when it is idle, its
  run parked or waiting, or its procedure waiting on its facts ...;
  *nothing,* otherwise". Computed, never stored, "by a walk of the
  task's live subtree, within tasks per tree".
- **The task line** (contract.md, 5.1): "the task's chip; its project
  and its requester; tracked and priority, label and flags; when it last
  changed, its last activity, and when it ended, if it has; its
  deadline; what it waits on; its budget spent and left; its live
  subtree counted by phase class, with how many are held below; and, for
  a task with a conversation, its requester's read position over its
  turns and their unread count".
- **Task queries** (engine.md, 11.2; contract.md, 6.1): keys "project;
  requester (a party, or a task); root; parent; label; phase class
  (working, waiting, held, closing, ended); executor kind; tracked",
  orders "number, last change, last activity, priority, or ended at", a
  window by cursor; admitted "on the party's role in the project it
  names, or, naming none, for tasks whose requester the party is"; "Live
  tasks are scanned"; "Marked by a task it matches, before the decision
  or after it".
- **`me`** (contract.md, 5.1 and 5.3): the party, their projects with
  name, role and pool, what waits for them, their updates unread, their
  read position over results, the merged templates (jig 09.3), "what
  they may create in each project, as the bounds a draft is compared
  with", their role in the deployment, and the answers' retention.
- **Conversations** (engine.md, 11.4): "Known by its flag, never by its
  charter, its label, or the template that made it"; read positions
  "moved only by the party's `Seen`".

## Choices this plan makes

- **The design's line takes the name `TaskLine`.** The shaped
  `Tasks { project, filter }` document's row, today's `TaskLine`
  (documents/project_tasks.rs:52), is renamed `ProjectTaskLine` in
  place, with its users; it and `GoalLine` and `ConversationLine` are
  the shaped subjects' rows, which jig 16 removes with them (decisions.md,
  3). The line reuses `BudgetLine` (spent, left, funder, deadline) and
  `PhaseCounts` with `held_below`, as `GoalLine` counts them.
- **`Frame` keeps its name** in the engine. It is the design's `me`; the
  wire's record is renamed by jig 22, and renaming the engine's type here
  would edit about 150 lines of its users for nothing a test sees.
  `Frame.charters` and `FrameProjectRole.chat_budget` are shaped and stay
  until jig 16.
- **What waits on a party** counts the entries `Core::waiting_entries`
  makes (inbox.rs:168) of the kinds `Question`, `Proposal`, `Escalation`
  and `PersonTask`; `ChatReply` and `Reply` are updates, not waits. The
  oldest one's task and kind say what the party is waited on for, as
  `people::EntryKind` (no new type: only its four waiting kinds occur
  there).
- **A query's marks compare the keys a decision cannot change.** A
  query watch is marked by a touched task, or a task along its lineage,
  whose project, label and executor kind match the query's (a key the
  query leaves out matches); requester, root, parent, phase class and
  tracked are not compared, so a task entering, leaving or moving within
  the query by one of them marks it. This is the "bounded comparison per
  watch" with no record of the keys as they were; it marks more watches
  than the design's, never fewer.
- **Orders and their direction** (contract.md, 6.1, at jig `0ec80c047`,
  gives them; ruling 133): number ascending; last change and last activity
  latest first; priority ascending, tracked tasks before untracked;
  ended at latest first, and the live group, which has no end, by
  number ascending. Ties go by number ascending. The cursor is the last
  line's number, and a window resumes after it, as
  `project_tasks::make` resumes (documents/project_tasks.rs:136-157).
- **The creating bounds** in `me` are the `Bounds::Goal` that
  `acts::bounds::goal` (acts/bounds.rs:22) judges for that project:
  its executors (charters and procedures), pool left, budget and
  deadline within, and the project's maximums. A `Create` of any other
  executor, judged when jig 03 offers `Create`, is later.md's row.
- **No ended group yet.** A query's read answers its live group; the
  ended group, through the store's indexes, is jig 08.5's.

## Rules that bind it

- **No shape of work grows** (code-organization.md, 6): nothing new is
  added to `Subject::{Goals, Conversations, Tasks}`, `Watched::*` of
  them, `Read::{Goals, Conversations}`, `GoalLine`, `ConversationLine`
  or `ProjectTaskLine`; the new query, line and waits-on do not name a
  chat, a goal or a list.
- **No field its counterpart across a boundary lacks**
  (code-organization.md, 6): `TaskLine`, `TaskQuery` and `WaitsOn` are
  written field by field here because jig 22's wire records mirror them;
  a field added during the session is named in the landing's report,
  and the supervisor updates this file's sketch.
- **Files over 800 lines at the base get at most 40 lines each**
  (code-organization.md, 7): here `jig-domain-core/src/documents/
  task_tree.rs (1018)`, `objects.rs (936)`, `tests.rs (2987)`. New
  code goes in `waits.rs`, `documents/task_lines.rs` and
  `documents/task_query.rs`.

## Increments

### 05.1 What a task waits on

Scope: engine.md, 11.3; contract.md, 5.1 ("what it waits on" on a task's
document); pages.md, 4, which shows a conversation "waiting on the
person: a question, a proposal or a person task in its tree" as
"waiting for you, and for what": a new core module computes a live
task's `WaitsOn` by a walk of its live subtree, within tasks per tree,
naming for a party whom, how many entries, and the oldest entry's task
and kind; the task's document carries it. Marking needs nothing new: an entry for a party made or ended, and a
delegate made or ended, touch the tasks along its lineage, whose `Task`
and `Tree` watches `watch_marks::touch_task` already marks
(routing/watch_marks.rs:173-241).
Needs: nothing beyond the session's start.
Files:
- `crates/jig-domain-core/src/`: `waits.rs` (new), `lib.rs`,
  `documents/task_tree.rs` (`Task.waits_on`, set in `task`; at most 10
  lines), `documents.rs` (the export).
- Users of the `documents::Task` literal, as the compiler names them
  (`crates/jig-protocol-engine-clients/src/tests/fixtures/`, the core's
  tests); `jig-protocol-engine-clients`'s translation of the task
  document drops `waits_on` until jig 22 adds its wire field (one line
  there saying so).
Shape:
- `jig_domain_core::waits::WaitsOn { Party { whom: people::Whom,
  entries: u32, task: u64, kind: people::EntryKind }, Delegates, News,
  Nothing }`; `Core::waits_on(&self, limits: &Limits, task: u64) ->
  WaitsOn`: a task not live is `Nothing`; the walk visits the task and
  its live delegates, depth first, within `limits.tasks.tree_tasks`,
  collecting the waiting kinds of `waiting_entries`; the oldest entry by
  its `at` (ties by task number) gives `whom`, `task` and `kind`, and
  `entries` counts them; with none, a live run not waiting is
  `Nothing`, live delegates are `Delegates`, an idle task, a parked or
  waiting run, or a procedure waiting on its facts is `News`, and
  anything else `Nothing`.
- `documents::Task.waits_on: WaitsOn`.
Test shape:
- `crates/jig-domain-core/src/tests/waits.rs` (new; `mod waits;` in
  tests.rs), with `tree_core` (tests.rs:501) and `new_row` (:541);
  limits from `tree_core`'s (`limits()`, tests.rs:29, the core's unit
  limits, by hand at the base). Reference test:

  ```rust
  //! What a task waits on, from its live subtree and the entries in it.
  use super::*;

  #[test]
  fn a_task_with_live_delegates_and_no_entries_waits_on_its_delegates() {
      let (mut core, limits) = tree_core();
      let parent = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, None);
      make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([parent]));
      let child = new_row(&core, 2, tasks::Funder::Task(1), None);
      make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
      assert_eq!(core.waits_on(&limits, 1), WaitsOn::Delegates);
  }
  ```

  `make_batch` is the step `tests/project_tasks.rs`'s `fixture` takes
  inline for a `tasks::Event::Make` (tests/project_tasks.rs:23-59): it
  moves into the crate's fixtures module, `src/tests/fixtures.rs`, as a
  `pub(crate)` fixture with this increment, and that `fixture` calls it.
- Its other tests: `a_question_to_the_requester_names_that_party_and_one_entry`,
  `the_oldest_of_two_waiting_entries_names_whom`,
  `a_proposal_below_says_the_party_is_waited_on_for_that_proposal`
  (`kind: people::EntryKind::Proposal { number: 4 }`, `task` the
  delegate's number),
  `a_person_task_below_names_its_role`,
  `an_idle_task_with_nothing_below_waits_on_news`,
  `a_working_task_with_live_delegates_waits_on_nothing` (engine.md,
  11.3: working comes before delegates),
  `a_task_not_live_waits_on_nothing`. Under 20 ms each.
- `tests/core/src/reference.rs` (new, the core world's reference made
  from outside, testing.md, 4): `pub fn waits_on(rows: &BTreeMap<
  root::Key, root::Record>, task: u64) -> core::waits::WaitsOn`, from
  the store's durable task rows alone (a question in a task's inbox to
  its party requester, a pending proposal or a waiting escalation held
  by a party or a role, a live person task), never from the core's
  state; `tests/core/tests/task_queries.rs`:
  `what_each_live_task_waits_on_matches_the_reference_from_the_store`
  (after `a_stopped_chat_reads_as_waiting_on_its_party`'s story and a
  delegate's question, each live task's document read against it).
  Under 0.5 s.
- `tests/core/tests/task_queries.rs` (new), the joint world through
  `stories::reads::{world, request}` (tests/core/src/stories/reads.rs:13,
  44) and `reads::{ask, latest}` (tests/core/src/reads.rs:170, 197);
  limits from `world::limits()` (tests/core/src/world.rs:17), the
  world's own. Reference test:

  ```rust
  use jig_core_world::{
      reads::{ask, latest},
      stories::reads::{request, world},
  };
  use jig_domain_core::{documents, reads, waits::WaitsOn};
  use jig_domain_people as people;

  #[test]
  fn a_stopped_chat_reads_as_waiting_on_its_party() {
      let mut world = world(1201);
      let task = world.assigned.as_ref().expect("chat").task;
      request(&mut world, 1, people::Ask::Stop { project: 1, task });
      ask(&mut world, 2, reads::Read::Task { task });
      let reads::Answer::Document(document) = latest(&world) else { panic!("a live task") };
      let documents::Document::Task(value) = document.as_ref() else { panic!("the task's document") };
      assert!(matches!(
          value.waits_on,
          WaitsOn::Party {
              whom: people::Whom::Person(1),
              entries: 1,
              task: waiting,
              kind: people::EntryKind::Escalation { .. },
          } if waiting == task
      ));
  }
  ```
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_stopped_chat_reads_as_waiting_on_its_party)'` once: a party who
stops their chat reads it as waiting on them, for that chat's
escalation, through the real store's read; under 0.5 s.
Don't: store what a task waits on, or keep it between documents; walk
past `tree_tasks`.
Size: at most 160 lines.
Removes: nothing.

### 05.2 The task line, and a task query watched

Scope: contract.md, 5.1 (the task line), 6.1 and 7.1 (`tasks{query}`,
who may watch); engine.md, 11 ("A task query's subject is its keys and
its order, of a fixed size") and 11.2 ("Live tasks are scanned",
admission, "Marked by a task it matches"): `TaskLine` made from a task's
record; `TaskQuery` with its keys and order; `documents::lines` scans the
live tasks once and keeps a window's lines in the query's order;
`Watched::Query` opens a watch on `Subject::Query`, admitted on the
query's project role, or for a query naming no project on the party
being its requester, and ended when the party loses that role;
`touch_task` marks it by this file's rule (Choices). The task's document
already carries label, flags and last activity (jig 02.1).
Needs: jig 02's label, flags and last activity on `tasks::TaskRecord`
(jig 02.1).
Files:
- `crates/jig-domain-core/src/`: `documents/task_lines.rs` (new:
  `TaskLine`, `ConversationRead`, `task_line`),
  `documents/task_query.rs` (new: `TaskQuery`, `PhaseClass`,
  `QueryOrder`, `Lines`, `lines`, `may_touch`), `documents.rs`
  (`Subject::Query`, `Watched::Query`, `Document::Lines`, `make`'s arm,
  exports, `ProjectTaskLine`, the worst case), `documents/project_tasks.rs`
  (`TaskLine` → `ProjectTaskLine`), `routing/views.rs`
  (`named_subject`, `admission`, the follow arms),
  `routing/watch_marks.rs` (`touch_task`: the query arm over the touched
  task and its lineage; `visibility_end`).
- The `ProjectTaskLine` rename's users beyond the core's two files (the
  search for `TaskLine` finds 49 lines in 16 files of jig, of which
  these are the engine's type; the rest are the wire's and the client's
  own `TaskLine`s, unchanged):
  `crates/jig-protocol-engine-clients/src/translate/live.rs:155-165`,
  `crates/jig-protocol-engine-clients/src/tests/fixtures/live.rs:88`.
- The exhaustive matches over `documents::Subject`, `Watched` and
  `Document` (an arm each): `routing/watch_marks.rs`, `routing/views.rs`,
  `reads.rs`, `reads/rows.rs`, `jig-protocol-engine-clients/src/
  translate/watches.rs:22-34` (no wire subject maps to `Query` until jig
  22) and `:129-183` (`Document::Lines` is not on the wire until jig 22:
  the delivery's existing `Reason::Tag` arm takes it), and those the
  compiler names in the worlds.
Shape:
- `TaskLine` and `ConversationRead` field by field as the sketch shows;
  `task_line(core, limits, party, row: &tasks::TaskRecord) ->
  Result<TaskLine, Missing>`, for a live row or a loaded ended one (jig
  08.4's): an ended row's `waits_on` is `Nothing` and its `phases` zero;
  `conversation` is `Some` for a task flagged conversation whose
  requester is a party, from `core.people.chat_seen(party, task)` and
  `row.last_reply`, `unread` capped as `conversation_from_row` caps it
  (documents/inbox_deployment.rs:301-313).
- `TaskQuery`, `PhaseClass`, `QueryOrder`, `Lines` as the sketch shows;
  `lines(core, limits, party, query, window) -> Result<Lines, Missing>`:
  a query naming a project needs the party's role there
  (`Missing::NotAllowed`); one naming none needs `requester:
  Some(Party::Person(party))` (else `Missing::NotAllowed`); `requester:
  Some(Party::Deployment { .. })` is `Missing::Unknown`; `phase:
  Some(PhaseClass::Ended)` selects no live line.
- `Subject::Query { query: TaskQuery }`; `Watched::Query { query:
  TaskQuery }`; `Document::Lines(Lines)`; `task_query::may_touch(query:
  &TaskQuery, row: &tasks::TaskRecord) -> bool`.
Test shape:
- `crates/jig-domain-core/src/tests/task_query.rs` (new; `mod
  task_query;`), with `tree_core`, `watch_core` (tests.rs:1735),
  `new_row`, `window` (:389), 5.1's `make_batch`, and `own_watches.rs`'s
  way of opening a watch on the core. Reference test:

  ```rust
  //! A task query's live group, its keys, order, window and marks.
  use super::*;

  #[test]
  fn a_query_by_requester_party_selects_only_their_tasks() {
      let (core, limits) = query_fixture();
      let query = TaskQuery {
          project: Some(1),
          requester: Some(tasks::Party::Person(1)),
          root: None,
          parent: None,
          label: None,
          phase: None,
          executor: None,
          tracked: None,
          order: QueryOrder::Number,
      };
      let lines = lines(&core, &limits, 1, &query, &Window { rows: 4, after: None }).expect("admitted");
      let numbers: Vec<u64> = lines.lines.iter().map(|line| line.chip.number).collect();
      assert_eq!(numbers, [1, 2]);
      assert_eq!(lines.next, None);
  }
  ```

  `query_fixture` is `tests/project_tasks.rs`'s `fixture` (:23-59)
  moved into `src/tests/fixtures.rs` as a `pub(crate)` fixture (tasks 1
  and 2 requested by party 1, task 3 by party 2), used by both files.
- Its other tests: `a_query_naming_no_project_admits_only_the_requester_themself`,
  `a_window_resumes_after_its_cursors_task`,
  `last_activity_order_puts_the_latest_first`,
  `priority_order_puts_tracked_tasks_first_by_priority`,
  `a_task_in_another_project_does_not_mark_the_query`,
  `a_delegates_question_marks_a_query_listing_its_parent`,
  `generated_commits_mark_each_query_whose_lines_change` (testing.md,
  3: "a task query by the tasks it matches before a decision or after
  it ... marked by generated commits": 32 commits drawn from
  `skein_lib::Rng::new(5)` among make, amend, cancel and end, four
  queries open; after each, a query whose lines differ from before was
  marked). Under 20 ms each.
- `crates/jig-domain-core/src/tests/project_tasks.rs`'s
  `project_tasks_filter_owner_facts_and_page_in_last_change_order`
  passes on `ProjectTaskLine` (it passes at the base).
- `tests/core/tests/client_peer.rs`, with `client_world` (:13) and `chat`
  (:45): `a_watched_task_query_is_delivered_again_when_its_task_ends`.
  Under 0.5 s.
- `tests/core/src/reference.rs`: `pub fn lines(rows, party, query:
  &TaskQuery) -> Vec<u64>`, the live tasks' numbers the query selects in
  its order, from the store's rows; `tests/core/tests/task_queries.rs`:
  `a_task_querys_lines_match_the_reference_over_the_stores_tasks` (a
  `const` table of six queries over keys and orders, each delivered
  window's numbers against the reference's). Under 0.5 s.
- The watches' fence (testing.md, 6: "every document a party's watch
  or read was given followed a durable commit that holds it"), in the
  world of `watches::world` (tests/core/src/watches.rs:532), whose
  `Fence` (:18-69) holds each delivery to a durable commit: a focused
  `a_watched_task_querys_deliveries_each_follow_their_commit` in
  `tests/core/tests/task_queries.rs`, and the seeded parties of
  `seeded_parties_take_frame_chat_tree_and_inbox_deliveries_at_their_own_pace`
  (fuzzy_watches.rs:11) open their own tasks' query beside their other
  subjects (`Parties::new`, watches.rs:112), so the fuzzy suite sweeps
  it. Under 0.5 s; the sweep's added time an aim of 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_watched_task_query_is_delivered_again_when_its_task_ends)'`
once: a client watching their own tasks' query is delivered their chat's
line, then a delivery after the commit that cancels the chat, whose
lines no longer hold it; under 0.5 s.
Don't: add an index, a map or a sorted copy kept between documents (one
scan per document, programming-model.md, 6.3); keep a query's previous
rows or keys in the views child or the core; change the shaped `Tasks`
document's rows beyond the rename.
Size: at most 300 lines.
Removes: nothing.

### 05.3 A task query read, a page at a time

Scope: contract.md, 6 (`tasks{query}`: "a task query's page (6.1) |
live state, then the store's indexes of ended tasks") and 6.1 ("Live
first, then ended, in two groups, each with its own cursor"): a read of
a query answers its live group a page at a time, through the reads'
live-then-stored machinery as `Read::Goals` uses it
(reads/rows.rs:255-300): `PageRow::Line` rows, a `Cursor::Live` while
more live lines match, and no next once they are read, since the ended
group is jig 08.5's (its "task query scope gains its ended part").
Needs: nothing beyond 5.2.
Files:
- `crates/jig-domain-core/src/`: `reads/vocabulary.rs` (`Read::Tasks`,
  `Scope::Tasks`, `PageRow::Line`), `reads.rs` (`scope`, `cursor`,
  `subject` arms; `Scope::Tasks` has no stored range yet),
  `reads/rows.rs` (the `Document::Lines` arm, at most 40 lines),
  `reads/rows/positions.rs`, `reads/cursors.rs` (arms).
- The exhaustive matches over `reads::Read`, `Scope` and `PageRow` the
  compiler names (an arm each), among them
  `crates/jig-protocol-engine-clients/src/translate/` (`Read::Tasks` has
  no wire form until jig 22), `crates/jig-domain-core/src/tests/
  conversation_cursor.rs:49`, `tests/core/tests/reads.rs`.
Shape:
- `reads::Read::Tasks { query: TaskQuery, from: Option<Cursor> }`;
  `reads::Scope::Tasks { query: TaskQuery }`; `reads::PageRow::Line(
  documents::TaskLine)`.
Test shape:
- `tests/core/tests/task_queries.rs`:
  `a_task_query_reads_a_partys_live_tasks_in_last_activity_order`,
  `a_task_query_read_pages_its_live_group_and_then_ends`,
  `a_task_query_read_waits_for_the_commit_in_flight_and_survives_the_crash`
  (its cut as `a_read_waits_for_the_commit_in_flight_before_it` makes
  it, tests/core/tests/reads.rs:11-34). Under 0.5 s each.
- `tests/core/tests/fuzzy_reads.rs`'s
  `seeded_reads_wait_for_prior_commits_and_keep_receipts_across_restart_cuts`
  (:12) also reads the party's own tasks' query after its restart, one
  more receipt per seed.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(a_task_query_reads_a_partys_live_tasks_in_last_activity_order)'`
once: a party's query over their two live tasks, read through the real
store after words reach the older one, lists that one first; under
0.5 s.
Don't: read the conversations' or goals' indexes for it; give it an
ended group (jig 08.5).
Size: at most 90 lines.
Removes: nothing.

### 05.4 `me` with the bounds of creating

Scope: contract.md, 5.1 ("what they may create in each project, as the
bounds a draft is compared with") and 5.3 ("*creating,* in `me`, per
project"): each of `me`'s projects carries the `Bounds::Goal` that
`acts::bounds::goal` judges for that party and project, or none where it
answers `Refused`; a policy change or a pool's change marks `me` as
today (`touch_project_parties`, `touch_frames`).
Needs: nothing.
Files:
- `crates/jig-domain-core/src/`: `documents.rs` (`FrameProjectRole.
  creating`, the worst case), `documents/small.rs` (`frame`), `acts.rs`
  (`bounds::goal` reached from `documents`, `pub(crate)`, at most 10
  lines).
- `FrameProjectRole`'s users (11 in 8 files): `crates/jig-domain-core/
  src/{documents.rs, documents/small.rs, tests.rs,
  tests/frame_deployment_role.rs}`, `crates/jig-protocol-engine-clients/
  src/{translate/owner.rs, tests/fixtures/documents.rs}` (the
  translation drops `creating` until jig 22),
  `tests/client/src/joint/wire/{owner.rs, deployment/tests.rs}`.
Shape:
- `FrameProjectRole.creating: Option<acts::Bounds>`: the `within` of
  `acts::bounds::goal`'s `Admission::Bounded` (a `Bounds::Goal`) for
  that party and project, `None` when it answers `Refused`.
Test shape:
- `crates/jig-domain-core/src/tests/frame_facts.rs` (existing):
  `me_carries_a_members_creating_bounds_per_project`,
  `me_carries_no_creating_bounds_for_an_observer`. Under 20 ms each.
- `tests/core/tests/task_queries.rs`:
  `me_reads_the_pool_left_in_its_creating_bounds`. Under 0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(me_reads_the_pool_left_in_its_creating_bounds)'` once: the
signed-in owner reads `me` and finds their pool left in the project's
creating bounds; under 0.5 s.
Don't: serve templates (jig 09.3); add to `Frame.charters` or
`chat_budget` (shaped); compute a bound `acts::bounds::goal` does not;
judge a `Create`'s bounds (later.md).
Size: at most 60 lines.
Removes: nothing.

### 05.5 Conversations known by their flag, and `Seen` over their turns

Scope: engine.md, 11.4; people.md, 6: a task is a conversation by its
flag, with its requester a party; `Core::chat_requester` becomes
`conversation_requester` and reads the flag in place of
`settings.chat`, its tracked test and its recurring test; the marks
(`touch_task`'s owner of a conversation, routing/watch_marks.rs:173-241)
and `Seen::Chat`'s admission (routing/people.rs:283-300) follow it.
Needs: jig 02's conversation flag, set on the chats `Ask::StartChat`
makes; 5.2's query watch, earlier in this session.
Files:
- `crates/jig-domain-core/src/`: `inbox.rs:105-121`, and the users of
  `chat_requester` (19 in 10 files): `acts/objects/task.rs`,
  `documents/inbox_deployment.rs`, `documents/offers.rs`, `inbox.rs`,
  `inbox/census.rs`, `indexes.rs`, `reads/indexed.rs`, `reads/rows.rs`,
  `routing/people.rs`, `routing/tasks.rs`; `routing/watch_marks.rs`.
- The readers of `settings.chat` other than the chat's creation and the
  offers of a chat (`routing/parties.rs`, `roles.rs`, `charters.rs`,
  `documents/small.rs`, shaped, stay): `documents/inbox_deployment.rs`,
  `inbox.rs`, `routing/watch_marks.rs`.
Shape:
- `Core::conversation_requester(&self, row: &tasks::TaskRecord) ->
  Option<u64>`: the requester party of a task flagged conversation,
  whatever its charter, tracking or recurrence; `None` otherwise. A
  tracked conversation stays one (the designer's answer to ruling 120's
  question, 2026-10-16; jig's tasks.md, 7: "Neither touches its flags,
  so a conversation stays one"), so the two other tests of a chat that
  read `tracked` and `recurring` go with `chat_requester`'s: the shaped
  `Conversations` document's (`documents/inbox_deployment.rs:340-345`)
  and `touch_task`'s `chat` (`routing/watch_marks.rs:202-207`), each
  calling `conversation_requester`.
Test shape:
- `crates/jig-domain-core/src/tests/conversation_cursor.rs` (existing):
  `a_task_flagged_conversation_under_another_charter_is_one`,
  `a_chat_charters_task_not_flagged_is_no_conversation`,
  `a_tracked_conversation_keeps_its_read_position`. Under 20 ms each.
- `tests/core/tests/reads.rs`'s `conversations_page_both_ways_from_the_index`
  passes unchanged (it passes at the base).
- `tests/core/tests/task_queries.rs`:
  `seen_over_a_conversations_turns_clears_its_lines_unread_count`.
  Under 0.5 s.
- `tests/core/tests/client_peer.rs`:
  `a_committed_turn_of_a_conversation_marks_the_query_listing_it`
  (engine.md, 11.4: "a turn committed is an update for them, and marks
  their `me`, their inbox, and the queries the task matches"; the
  query's next delivery carries the line's unread count 1). Under
  0.5 s.
Proof: joint, `cargo nextest run -p jig-core-world -E
'test(seen_over_a_conversations_turns_clears_its_lines_unread_count) |
test(=conversations_page_both_ways_from_the_index)'` once: a party's
`Seen` over their conversation's turns brings its task line's unread
count to zero, and the conversations' pages read as before; under 1 s.
Don't: read `settings.chat`, `tracked` or `recurring` to know a
conversation; add a read position for a task not flagged conversation;
rename `people::Seen::Chat` (jig 16 does, with the shed).
Size: at most 60 lines.
Removes: `chat_requester`'s charter, tracked and recurring tests, and
the same three in `inbox_deployment.rs`'s `conversations` and
`watch_marks.rs`'s `touch_task`.

## Interface sketches

Compiled at the base in a scratch module of `jig-domain-core`
(`src/sketch_w3.rs`, bodies `unimplemented!()`, parameters prefixed `_`)
with `cargo clippy -p jig-domain-tasks -p jig-domain-people -p
jig-domain-core --all-targets -- -D warnings`. jig 02's label and flags
were stubs there (`type Label = u32`, a `Flags` struct); below they are
written as jig 02's file names them (`label: u32`, `tasks::Flags`), and
the names jig 02 lands win, checked at the ready check. The variants
inside existing enums were not compiled.

```rust
// crates/jig-domain-core/src/waits.rs (5.1)

/// What a task waits on, computed when a line or a task's document is made,
/// never stored (engine.md, 11.3; contract.md, 5.1). jig 22's wire mirrors it.
pub enum WaitsOn {
    /// An entry in its live subtree, itself included, waits for a party or a
    /// role: whom the oldest such entry waits for, how many entries wait, and
    /// the oldest one's task and kind (a question, a proposal, an escalation
    /// or a person task: pages.md, 4, "waiting for you, and for what").
    Party { whom: people::Whom, entries: u32, task: u64, kind: people::EntryKind },
    /// It has live delegates, and nothing in its subtree waits for a party.
    Delegates,
    /// It is idle, its run parked or waiting, or its procedure waiting on facts.
    News,
    /// It is working, or its phase says what it waits for, or it has ended.
    Nothing,
}

impl Core {
    /// Walk the task's live subtree, within tasks per tree, and say what it waits on.
    pub fn waits_on(&self, limits: &Limits, task: u64) -> WaitsOn;
}
```

```rust
// crates/jig-domain-core/src/documents/task_lines.rs (5.2)

/// A conversation's requester's read position over its turns, and what is unread.
pub struct ConversationRead {
    pub seen: u64,
    pub unread: Count,
}

/// "One record for every list of tasks" (contract.md, 5.1 and 6.1); jig 22's
/// wire record mirrors it field for field.
pub struct TaskLine {
    /// The task's chip: number, title, executor's kind, phase.
    pub chip: Chip,
    pub project: u32,
    pub requester: tasks::Party,
    /// Its priority, when tracked.
    pub tracked: Option<u32>,
    /// jig 02's label.
    pub label: u32,
    /// jig 02's flags: conversation, listed, retention class.
    pub flags: tasks::Flags,
    /// When it last changed.
    pub changed: Wall,
    pub last_activity: Wall,
    /// When it ended, for an ended task's row (jig 08.5's ended group).
    pub ended_at: Option<Wall>,
    /// Its budget spent and left, its funder and its deadline.
    pub budget: BudgetLine,
    pub waits_on: WaitsOn,
    /// Its live subtree, itself included, counted by phase.
    pub phases: PhaseCounts,
    /// Live tasks held below it.
    pub held_below: u32,
    /// For a task flagged conversation whose requester is a party.
    pub conversation: Option<ConversationRead>,
}

/// Make a task's line from its record, live or loaded ended (jig 08.5), for a
/// party who may see it.
pub fn task_line(core: &Core, limits: &Limits, party: u64, row: &tasks::TaskRecord)
    -> Result<TaskLine, Missing>;
```

```rust
// crates/jig-domain-core/src/documents/task_query.rs (5.2, 5.3)

/// A task's phase class, as a query selects it (contract.md, 6.1).
pub enum PhaseClass {
    /// Active: idle, due, preparing, claimed, running or backing off.
    Working,
    Waiting,
    Held,
    Closing,
    /// Only in the ended group (jig 08.5); a live query naming it selects nothing.
    Ended,
}

/// A query's order (contract.md, 6.1); ties go by number.
pub enum QueryOrder {
    /// Ascending task number.
    Number,
    /// Latest change first.
    Changed,
    /// Latest activity first.
    Activity,
    /// Ascending priority, tracked tasks first.
    Priority,
    /// Latest ended first; the live group goes by ascending number.
    Ended,
}

/// A task query: a conjunction of the fixed keys, each `None` matching any
/// task, and an order (engine.md, 11.2); the watch's subject. jig 22 mirrors it.
pub struct TaskQuery {
    pub project: Option<u32>,
    /// A party or a task; `Party::Deployment` is refused as unknown.
    pub requester: Option<tasks::Party>,
    pub root: Option<u64>,
    pub parent: Option<u64>,
    pub label: Option<u32>,
    pub phase: Option<PhaseClass>,
    pub executor: Option<ExecutorKind>,
    pub tracked: Option<bool>,
    pub order: QueryOrder,
}

/// A query's live group within a window, in its order.
pub struct Lines {
    pub query: TaskQuery,
    pub lines: List<TaskLine>,
    /// The cursor of the window's last line when more live lines match.
    pub next: Option<Cursor>,
}

/// Admit the query and scan the live tasks once, keeping the window's lines in order.
pub fn lines(core: &Core, limits: &Limits, party: u64, query: &TaskQuery, window: &Window)
    -> Result<Lines, Missing>;
/// Whether a touched task, or a task along its lineage, can enter, leave or move
/// within the query: its project, label and executor kind match the query's.
pub fn may_touch(query: &TaskQuery, row: &tasks::TaskRecord) -> bool;

// in `documents::Subject` and `Watched` (5.2):
Query { query: TaskQuery },
// in `documents::Document` (5.2):
Lines(Lines),
// in `reads::Read`, `reads::Scope` and `reads::PageRow` (5.3):
Tasks { query: TaskQuery, from: Option<reads::Cursor> },   // Read
Tasks { query: TaskQuery },                                 // Scope; jig 08.5 adds its ended range
Line(documents::TaskLine),                                  // PageRow
```

The scan, whose order is easy to get wrong:

```rust
let mut order = Map::with_capacity(limits.tasks.tasks);      // key → number, one pass
for candidate in core.tasks.view_tasks().as_ref() {
    let row = core.tasks.task(candidate.number).expect("live task view");
    if matches(core, row, query) {
        order.insert(sort_key(row, query.order), row.number).expect("bounded live order");
    }
}
// sort_key: Number → number; Changed → (Reverse(changed), number);
// Activity → (Reverse(last_activity), number); Priority → (tracked.is_none(),
// tracked.unwrap_or(0), number); Ended → number (a live task has no end).
// Then skip to the line after `window.after`'s number, take `window.rows`,
// and set `next` to the last taken line's number when one more matched.
```

## Questions to raise when reached

- None: the orders' directions are contract.md, 6.1's own (ruling 133).

## Done when

- jig's `origin/main` has `Step: jig-05.1` to `Step: jig-05.5`
  (`git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep -c
  '^jig-05\.'` prints 5).
- `git -C ~/src/rust/jig grep -n -e chat_requester -e 'settings\.chat'
  origin/main -- crates/jig-domain-core/src/inbox.rs
  crates/jig-domain-core/src/routing/watch_marks.rs` prints nothing.
- `cargo nextest run -p jig-core-world -E
  'binary(task_queries)'`, run once in a worktree of jig's
  `origin/main`, passes 9 tests.
