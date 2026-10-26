# Session 08: the ended indexes, ended read positions, and task queries' ended reads

The store's half of jig's plumbing (jig's `docs/design/STATUS.md`,
section 3, step 2: "the two ended indexes and the ended read
positions"). The tasks child saves two indexes in the decision that ends
a task: ended tasks by requesting party and label, and ended tasks
flagged listed by project and label (store.md 7.4). A task's retention
class, not its chat charter, decides whether it is forgotten once read,
and the forget erases its rows in the new indexes. A conversation's read
position moves to the people child's ended family when the task ends, is
never restored at start, and is joined by a point read where a read, a
mark, a forget or the restart's census needs it. Task queries then read
their ended part through the indexes. The conversation and goal indexes
(tasks tags 14 to 16) keep their readers, the shaped lists, until jig 16
removes both (this file's "Choices"). Six increments; the first two need
only jig 02.1.

## Read first

- jig's `docs/design/protocol/store.md`: 1 (the "Ended tasks are found
  through indexes" item), 4.4 (the people rows and the last row), 7.2
  (the inbox's census), 7.3 (the task query rows and the paragraphs
  after the table), 7.4, 11.1 ("A task forgotten once read"), 15 (items
  2, 3, 8).
- jig's `docs/design/domain/people.md`, 6 (a conversation's positions,
  the retention class); `domain/engine.md`, 6 (step 1), 11.2, 11.4, 13;
  `domain/tasks.md`, 3 (the flags).
- jig's `docs/design/client/domain/contract.md`, 5.1 (the task line), 6.1.
- jig's `docs/design/domain/testing.md`, 3 (the tasks and people
  worlds), 5 (the conformance scenarios), 6 (the referee's "Nothing
  lost"), 7 (the store item), 9.
- `findings/engine-agents-store-config.md`, sections 1 (the last four
  store items), 2, 4 (the last three items), 5.
- `findings/engine-parties-and-tasks.md`, section 4 (the last item).
- skein's `programming-model.md`, 6.3; `testing-strategy.md`, 2.2 and 6.

## What the design asks

- **Two ended indexes, the tasks child's,** each saved in the decision
  that ends a task (store.md 7.4): "*ended by requester:* key `(party,
  label, ended at reversed, task)`, a row for every ended task whose
  requester is a party; *ended by project:* key `(project, label, ended
  at reversed, task)`, a row only for an ended task flagged *listed*.
  The flag is the task's, set from its template when it was made, so the
  engine reads no template to index." "A task moved to a party is
  indexed by the requester it has when it ends."
- **Forgotten once read** (people.md 6): "once it has ended, its
  requester has read its result and, for a conversation, its turns, and
  no live task names it, its record and its read position are erased in
  one decision". Store.md 7.4: "its decision erases the rows its record
  names whole: its result's update, its rows in the ended indexes, and
  its ended read position. Its replies' updates stay". "A task whose
  requester is not a party is kept, whatever its class." Retention's
  timers and its passes are not planned (STATUS, section 3).
- **Ended read positions** (store.md 7.4): "*a live conversation's,* in
  a family keyed `(party, task)`, loaded at start ... *an ended one's:*
  the decision that ends the task moves its position to the family of
  ended read positions, keyed `(task, party)`, read on demand. A read of
  the conversation, a task query's row and the inbox's census join it by
  a point read".
- **Task queries' ended part** (store.md 7.3): "served from one index,
  chosen by the query's keys in the table's order: a requester party; a
  requester task or a parent; a project. A query naming none of these
  has no ended part, and its page says so. The keys beyond the index's
  prefix ... filter its rows. Its rows come in the index's order: within
  a label, newest ended first from the two ended indexes; by task from
  the ended delegates'". "A task query's row joins its task's record
  ... and, for a task flagged conversation, its requester's ended read
  position, by a point read."

## Choices this plan makes

- **Tags.** `tasks::Key::EndedByParty` is tasks tag 19 and
  `tasks::Key::EndedByProject` tag 20, appended after `Delegate` (18),
  outside the tasks restore range (`[JIG, 1, 0]` to `[JIG, 1, 6]`,
  `intervals.rs:15`). `people::Key::EndedRead { task, party }` is people
  tag 7; the people restore range becomes `[JIG, 2, 0]` to `[JIG, 2, 7]`.
  Each index row is a small `Stored` value holding its key's fields, as
  `Update` and `Edge` are, in tasks v3 and people v3, edited in place.
- **The forget is made in the decision of the mark after which the
  task is read** (people.md 6: "in the `Seen`'s own decision: the mark
  and the erase commit together, so no restart sees the task read and
  not forgotten"): a `Seen::Chat` on an ended conversation, as the base
  forgets an ended chat (`seen_chat_loaded` pushes the mark and the
  forget into one decision); and a `Seen::Results` for the forgettable
  tasks the people child's cached unread results
  (`people::Domain::cached_results`, at most `inbox_entries + 1` per
  party) name at or below the mark, from what each cached `ResultRef`
  carries, with no load (08.3's Shape). A task the cache does not name,
  or that a live task names when it is read, stays until retention's
  pass, which is not planned (people.md 6: "Retention's next pass
  forgets the others"; `later.md`'s row of what jig's `STATUS.md` leaves
  not planned). jig's own templates are both kept (configuration.md
  3.1), so a chat made by `people::Ask::StartChat` is no longer
  forgotten once this lands.
- **Each ended conversation gets an `EndedRead` row** at its end,
  `through: 0` when its party read nothing, so that the restart's census
  reads the ended read positions family and joins each row's ended
  record, in place of reading the conversation index (tag 15). The
  conversation and goal indexes (tags 14, 15, 16) are then read only by
  `reads::Scope::{Conversations, Goals}`, which jig 16 removes with the
  shaped lists; jig 16 removes the three indexes and their readers in
  the same increment. The result index (tag 13, `(party, position)`)
  stays: it is the census's index of unread results by position, and
  names no shape of work.
- **A query whose phase class excludes ended has no ended part,** as
  one naming none of the three keys (`EndedPart::None`).
- **Who joins what.** The store joins an ended row's owner record and,
  for a task flagged conversation whose requester is a party, its
  `EndedRead` row's `through`, by a point read, in the row's own field
  `reads::Row.ended_read: Option<u64>` (ruling 172); `related` keeps
  the immutable evidence it carries, an inbox reply's activity line
  among it. The core reads that position where it reads
  `people::Domain::chat_seen` for a live one.

## Rules that bind it

- **Format versions** (README, Rules): tasks v3 is jig 02.1's and people
  v3 jig 02.3's; this session edits them in place. Regenerate with
  `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
  jig-store-codecs-world --test drift`, once per increment that edits a
  schema; the drift and memory tests then pass unchanged in name.
- **Files over 800 lines at the base get at most 40 lines each**
  (code-organization.md, 7): here `jig-domain-tasks/src/boundary.rs`
  (1804), `jig-domain-people/src/domain.rs` (1884),
  `jig-domain-core/src/reads/rows.rs` (1698),
  `jig-domain-core/src/routing/tasks.rs` (1459), the tests files
  `jig-domain-core/src/tests.rs` (2993), `jig-domain-people/src/tests.rs`
  (1596), `tests/tasks/src/world.rs` (1206),
  `tests/client/src/joint/wire/cursor.rs` (1063),
  `tests/core/src/observations.rs` (1068),
  `testing/jig-test-domain/src/lib.rs` (1125),
  `examples/ops/domain/src/route.rs` (1589). `jig-domain-people/src/inbox.rs`
  and `boundary.rs` are 791 lines: new people code goes in a new module,
  `jig-domain-people/src/ended.rs`. New core code goes in
  `jig-domain-core/src/forget.rs` and `reads/ended.rs`.
- **No shape of work grows** (code-organization.md, 6): nothing new is
  added to `Scope::Conversations`, `Scope::Goals`, the conversation or
  goal indexes, or `Settings.chat`'s readers; this session edits them
  only where a Shape below names the edit.

## Increments

### 08.1 The ended index by requesting party and label

Scope: store.md 7.4's "ended by requester" index: the tasks child saves
an `EndedByParty` row in the decision that ends a task whose requester
is a party, keyed `(party, label, ended at reversed, task)`; the store
keys it at tasks tag 19 and encodes it in tasks v3 (store.md 4.4, 15
item 3). Nothing reads it before 8.5.
Needs: jig 02.1's label on `tasks::TaskRecord` and its tasks family at
version 3, at the session's start.
Files:
- `crates/jig-domain-tasks/src/`: `index.rs` (`EndedByParty`,
  `Key::EndedByParty`, `Key::live`), `boundary.rs` (`Stored::EndedByParty`,
  `Stored::key`; at most 40 lines), `closing.rs` (the save, beside
  `Stored::EndedDelegate` at :379), `domain.rs` (`output_bound`'s
  `limits.tasks.checked_mul(26)`, :451, counts an ending's saves, 26
  since `EndedDelegate`: 27 with this row; at most 40 lines), `stored.rs`
  (a restored row is refused, as `Stored::Result` at :792), `owned.rs`,
  `lib.rs` (export), `tests/index.rs`.
- The values derived from `tasks::max_out`, which `output_bound`'s
  multiplier raises (27 here, 28 in 8.2): `route_room_checked`
  (`jig-domain-core/src/routing/mod.rs:277`) and `room_max`, which size
  the journal's room, and from them, under rulings 110 and 112, the
  profiles' bounds (`crates/jig-protocol-config/src/profile/{small,
  standard}.rs`), `What::Memory { over }` at
  `crates/jig-protocol-config/src/tests/domain.rs:169` and their copies
  in `testing/jig-test-service/src/tests.rs:24-25`, where they move.
- `crates/jig-protocol-store/`: `src/keys/tasks.rs` (tag 19, both ways),
  `schema/tasks-v3.schema` (`record EndedByParty { party: u64 label: u32
  ended_at: u64 task: u64 }`, `ended_by_party` in `Value`),
  `src/codec/tasks.rs` and `src/codec/tasks/index.rs` (both ways), the
  generated family, the goldens the drift test writes,
  `src/tests/fixtures/keys.rs` (`tasks_key`, :100, draws the variant:
  its `below(19)` at :102 becomes `below(20)`),
  `src/tests/codec/record.rs`, `src/historical.rs` and
  `src/reads/{indexed.rs, joins.rs}` (an arm each).
- The exhaustive matches over `tasks::Key` and `tasks::Stored` (34 files
  in jig at the base, from `git grep -l -E
  'Stored::EndedDelegate|Key::EndedDelegate'` less the docs; among them
  `jig-domain-core/src/{inbox/census.rs, indexes.rs, proposals.rs,
  reads.rs, reads/indexed.rs, reads/rows.rs, reads/rows/positions.rs,
  reads/vocabulary.rs, restart/census.rs, routing/tasks.rs, stored.rs}`,
  `tests/client/src/joint/{truth.rs, wire/cursor.rs}` (the cursor's tag
  19), `tests/tasks/src/{world.rs, accounting_referee.rs}`,
  `tests/core/tests/effects.rs`, `tests/ops/src/joint/reads/delegates.rs`):
  an arm each.
- The tasks world: `tests/tasks/src/accounting_referee.rs` (`committed`,
  :212), `tests/tasks/tests/{referee.rs, lifecycle/delegates.rs}`.
- Downstream: the repository downstream of jig matches `tasks::Stored`
  in 3 files (its engine root's restart, its engine world's roles and
  roles referee), absorbed at its repin.
Shape:
- `jig_domain_tasks::EndedByParty { party: u64, label: u32, ended_at:
  Wall, task: u64 }` with `const fn key(self) -> Key`, and
  `Key::EndedByParty { party: u64, label: u32, ended_at: Newest, task:
  u64 }`, as sketched; `Key::live` is false for it.
- `Stored::EndedByParty(EndedByParty)`; the tasks child never restores
  it (`Refusal::Restore`).
- `closing.rs`: when the ending task's requester is `Party::Person(party)`,
  `Request::Save { record: Stored::EndedByParty(EndedByParty { party,
  label: ended.label, ended_at: env.wall, task: number }) }`, in the same
  output as the `Stored::Ended` save. A `Party::Task` or
  `Party::Deployment` requester saves no row.
- The row names the requester the task has when it ends: a delegate
  moved to a party (the tasks child's `Event::Move`) is indexed by that
  party (store.md 7.4: "A task moved to a party is indexed by the
  requester it has when it ends").
Test shape:
- `crates/jig-domain-tasks/src/tests/index.rs` (exists; its tests are
  the pattern): `a_newer_ended_task_sorts_first_within_party_and_label`,
  `one_label_sorts_before_the_next_for_one_party`.
- `crates/jig-protocol-store/src/tests/keys.rs` keeps
  `core_and_people_keys_sort_as_bytes_as_they_derive` and
  `every_generated_key_decodes_to_itself` unchanged; the generator
  (`fixtures/keys.rs:100`) draws the new variant.
- `tests/core/tests/ended_indexes.rs` (new; the core world on the real
  store). Fixtures: `jig_core_world::stories::reads::{world, request}`
  (`tests/core/src/stories/reads.rs:13, 44`), `World::end`
  (`tests/core/src/effects.rs:610`), `World::turn` (:574),
  `world.store.rows`. Limits: the read story's (`stories/reads.rs:17`),
  hand-built at the base; nothing new. Reference test:

  ```rust
  //! The ended indexes, saved by the decision that ends a task (store.md 7.4).
  use jig_core_world::stories::reads::world;
  use jig_domain_core as core;
  use jig_domain_tasks as tasks;
  use jig_test_domain as root;

  fn finish(world: &mut jig_core_world::effects::World) {
      world.end(
          tasks::End::Finished {
              result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
              cancel_delegates: false,
          },
          0,
      );
  }

  fn ended_rows(world: &jig_core_world::effects::World) -> Vec<tasks::Key> {
      let mut found = Vec::new();
      for key in world.store.rows.keys() {
          if let root::Key::Core(core::Key::Tasks(key @ tasks::Key::EndedByParty { .. })) = key {
              found.push(*key);
          }
      }
      found
  }

  #[test]
  fn an_ended_chat_is_indexed_by_its_requesting_party_and_label() {
      let mut world = world(820);
      let task = world.assigned.as_ref().expect("chat").task;
      finish(&mut world);
      let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
          &world.store.rows[&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))]
      else {
          panic!("ended chat")
      };
      let ended_at = tasks::Newest::of(row.ended_at.expect("ended").as_nanos());
      assert_eq!(ended_rows(&world), [tasks::Key::EndedByParty { party: 1, label: 0, ended_at, task }]);
  }
  ```

  (label 0: jig 02 gives the tasks `StartChat` makes label 0; the
  requesting party is the world's person, 1.)
- Also there: `an_ended_delegate_of_a_task_saves_no_party_row` (a
  delegate whose requester is a task ends: `ended_rows` holds only its
  root's row once that ends).
- `tests/tasks/tests/lifecycle/delegates.rs` (its `delegates` and
  `finish_delegate`, :4 and :14; the move as at :40):
  `a_delegate_moved_to_a_party_is_indexed_by_that_party_when_it_ends`
  (moved to person 9, finished: the world's records hold
  `Key::EndedByParty { party: 9, label: 0, task: 2, .. }` and no row for
  any other party). The tasks child's `Event::Move` is at the base, so
  8.1 needs no jig 04 step for it.
- The tasks world's referee (`accounting_referee.rs`, `committed`): a
  commit that saves `Stored::Ended` for a task whose requester is a party
  saves its `EndedByParty` row; the tasks world's focused and fuzzy
  lifecycle tests (`tests/tasks/tests/fuzzy_lifecycle.rs`) run it, which
  joins the index to a swept scenario family (domain/testing.md 3 and
  9). `tests/tasks/tests/referee.rs`:
  `an_ended_task_without_its_party_index_row_is_refused`.
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(an_ended_chat_is_indexed_by_its_requesting_party_and_label)'`
once: a person's chat ends and the real store holds its
`(party, label, ended at, task)` row, committed with the ended record;
under 0.5 s. Nothing reads the index until 8.5, which connects it.
Don't: save the row from the core (`indexes.rs`); read a template or
`Settings.chat` to index; restore the row at start.
Size: at most 150 lines of production code (a codec increment: lands
within 2.5 times).
Removes: nothing.

### 08.2 The ended index by project and label, for tasks flagged listed

Scope: store.md 7.4's "ended by project" index: an `EndedByProject` row
saved beside 8.1's when the ending task is flagged listed, keyed
`(project, label, ended at reversed, task)`, at tasks tag 20, in tasks
v3. The flag is read from the task's record.
Needs: jig 02.1's flags on `tasks::TaskRecord` (`listed`), at the
session's start.
Files: 8.1's list, with `EndedByProject` in place of `EndedByParty`
(tag 20; `record EndedByProject { project: u32 label: u32 ended_at: u64
task: u64 }`, `ended_by_project` in `Value`; `output_bound`'s
multiplier, `domain.rs:451`, becomes 28, and `tasks_key`'s draw,
`fixtures/keys.rs:102`, `below(21)`), and the same 34 files' matches and
the downstream's 3.
Shape:
- `jig_domain_tasks::EndedByProject { project: u32, label: u32,
  ended_at: Wall, task: u64 }` with `const fn key(self) -> Key`;
  `Key::EndedByProject { project: u32, label: u32, ended_at: Newest,
  task: u64 }`; `Stored::EndedByProject(EndedByProject)`; never
  restored.
- `closing.rs`: when `ended.flags.listed`, `Request::Save { record:
  Stored::EndedByProject(EndedByProject { project: ended.project, label:
  ended.label, ended_at: env.wall, task: number }) }`, whatever the
  requester.
Test shape:
- `crates/jig-domain-tasks/src/tests/index.rs`:
  `a_newer_listed_task_sorts_first_within_project_and_label`.
- `tests/core/tests/ended_indexes.rs`:
  `an_ended_goal_is_indexed_by_its_project_and_label` (the read story's
  chat ended first, as `a_restart_reads_no_ended_task_whose_result_was_read`
  ends it before its `census_goal`, `tests/core/tests/restart.rs:464`: a
  goal asked while the chat is live is never assigned, `world.assigned`
  staying the chat; then a goal made by
  `jig_core_world::stories::offers::goal(100)`,
  `tests/core/src/stories/offers.rs:92`, listed with label 0 as jig 02
  makes `SetGoal`'s, ends: the store holds `EndedByProject { project: 1,
  label: 0, .. }`), `an_ended_chat_saves_no_project_row` (not listed).
- The tasks world's referee also asks a listed task's `EndedByProject`
  row in the commit that ends it; `tests/tasks/tests/referee.rs`:
  `an_ended_listed_task_without_its_project_index_row_is_refused`.
- Ceilings as 8.1.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(an_ended_goal_is_indexed_by_its_project_and_label)'` once: a goal
ends and the real store holds its project row beside its party row;
under 0.5 s. 8.5 connects the read.
Don't: index a task by its tracked field or its charter; change
`Stored::Goal` (tag 16), which jig 16 removes.
Size: at most 120 lines of production code (a codec increment).
Removes: nothing.

### 08.3 Forgotten once read, by the task's retention class

Scope: people.md 6 and store.md 7.4, 11.1: an ended task of retention
class forgotten once read, requested by a party, is erased in the
decision of the mark after which it is read, the mark and the erase in
one commit: its record, its result's update,
its rows in the two ended indexes and its read position, and, while
they exist, its rows in the result and conversation indexes (tags 13 to
15). The class replaces the chat's charter as what decides it; a kept
task, jig's chat among them, is never erased by a read.
Needs: jig 02.1's retention class on `tasks::TaskRecord`; jig 03.1's
`people::Ask::Create { project, batch }` (a batch of agents' tasks, each
member with its label and flags), to make a conversation of class
forgotten once read in the core world.
Files:
- `crates/jig-domain-core/src/`: `forget.rs` (new: `inbox.rs:406-450`'s
  `forget_chat` moved and keyed on the class, `Owner` and `forgets` as
  sketched), `inbox.rs` (`seen_chat_loaded`'s forget push at :386 goes
  through `forget.rs`), `routing/people.rs` (`Seen::Results` at
  :271-282: the candidates come from `people.cached_results(person)` in
  place of `marked_chats`, and are forgotten in the mark's step),
  `routing/tasks.rs` (:716) and `inbox/census.rs` (:131), where a
  `ResultRef` is made, `routing/vocabulary.rs` (`Now::LoadEndedChat`
  goes, its one sender being `Seen::Results`; `Event::ForgetChat { row }`
  → `Event::Forget { owner }`),
  `routing/mod.rs`, `stopping.rs`, `routing/watch_marks.rs`,
  `tests/completion_retry.rs` (the renamed arms), `lib.rs`.
- `crates/jig-domain-people/src/`: `boundary.rs` (`Event::ForgetChat`
  → `Event::Forget`, the same fields; `Forgettable` and
  `ResultRef.forgettable`, as sketched), `domain.rs` (the step's
  `Event::ForgetChat` arm, :429; at most 40 lines), `inbox.rs`
  (`forget_chat_event`'s name; `marked_chats` goes, its one reader being the forget's
  candidates), `tests.rs`, `tests/marked_chats.rs` (removed with it).
- `crates/jig-protocol-store/src/historical.rs`, `src/tests/gets.rs`;
  `testing/jig-test-store/src/{route.rs, tests/routes.rs}`;
  `testing/jig-test-domain/src/{routes.rs, admission.rs}`;
  `examples/ops/domain/src/route.rs` (:1344); `tests/core/src/{effects.rs,
  world.rs, client_peer.rs}`, `tests/client-protocol/src/engine.rs`,
  `tests/client/src/joint/session.rs`, `tests/conformance/src/lib.rs`,
  `tests/ops/src/joint/{engine.rs, front.rs}`, `tests/people/src/world.rs`,
  `tests/people/tests/{memory.rs, simulation.rs}`,
  `testing/jig-test-service/src/{route.rs, clients/translation.rs}`: the
  removed `Now` variant and the renamed core and people `Event`
  variants, an arm or a name each (from `git grep -n -E
  'LoadEndedChat|ForgetChat|SeenChatLoaded|Now::SeenChat|ChatSeen|marked_chats|\.chat_seen\('`).
- The six test files that make a `ResultRef` (of the nine that `git
  grep -l 'ResultRef {' -- ':!docs'` lists at the base, all but
  `boundary.rs` and the two above): `forgettable: None` each.
- `tests/core/src/effects.rs` (a `World.initial_ask: people::Ask`,
  defaulting to the base's chat), `effects/constructors.rs`
  (`World::configured_with(.., initial)` beside `configured`, :85),
  `effects/deliveries.rs` (the first sign-in's hard-coded `StartChat`,
  :153-158, reads `initial_ask`): for `forgettable_world`, about 25
  lines.
- `tests/core/tests/activity.rs`, `tests/core/tests/reads.rs`,
  `tests/core/tests/restart.rs`: the forgetting tests named below.
- `testing/jig-conformance/src/scenarios.rs` (`Scenario`, `ALL`,
  `Action`, `program`); the testing application's bindings,
  `tests/conformance/src/scenarios.rs` (`action`, `check`; its `ask` at
  :28), `tests/conformance/src/lib.rs` (`peers`' match over every
  `Scenario`, :216-253: an arm for the new one),
  `tests/conformance/tests/scenarios.rs`.
- Downstream: 4 files name these variants (its engine root's archive
  and `now` routes, its service's translation, its client world's
  translation), absorbed at its repin; its conformance configuration
  matches every `Option<Scenario>` (`tests/conformance/src/config.rs:132-138`
  there), which `ConversationEndsWhileRead` breaks at the repin.
Shape:
- `jig_domain_core::forget::Owner`, as sketched: what one forget
  erases, made by `Owner::of_row` from a loaded record, reading
  `row.flags` (jig 02.1's), or by `Owner::of_result` from a cached
  `ResultRef`. `forgets(owner: &Owner, reading: Reading) ->
  Option<u64>`: the requesting party whose reading forgets the task
  now. `Reading { results, turns: Option<u64>, named }`: the
  requester's results position, its position over the task's turns
  (`None` when not flagged conversation), and whether
  `core.tasks.stub(task)` exists.
- **The mark and its forgets are one decision** (people.md 6; the
  design answer of 2026-10-16T19:02Z). The base's `Seen::Results`
  decides the mark, then loads, and forgets in later decisions: a crash
  between commits the mark and loses the forget, which no restart finds
  again (the fuzzy sweep's `ConversationEndsWhileRead`, cut 17, seed
  757).
- `people::ResultRef` gains `forgettable: Option<people::Forgettable>`,
  as sketched, `None` for a kept task, set where a ref is made from the
  ended record (`routing/tasks.rs:716`, `inbox/census.rs:131`).
- On `Seen::Results { through }`: in the mark's own step, after its
  `people::Event::Decided { outcome: MarkedSeen }`, `Event::Forget {
  owner }` for each `ResultRef` of `core.people.cached_results(person)`,
  read before the mark applies, with `position <= through`,
  `forgettable` set and a task not live, its owner
  `Owner::of_result(person, &result)`. Nothing is loaded, and a mark
  over kept results only, jig's own templates among them, is decided as
  at the base. `Now::LoadEndedChat` goes.
- On `Seen::Chat` of an ended conversation: as at the base
  (`Now::SeenChat`, then `seen_chat_loaded`), which pushes `Event::Forget
  { owner }`, from `Owner::of_row`, in the mark's decision.
- `forget.rs::forget(core, env, work, owner)` writes in the mark's
  decision, once `forgets` names the party:
  `Erase(Key::Ended(task))`, the result's `Update` key, `EndedByParty`
  and, when listed, `EndedByProject` (8.1, 8.2), the tag 13 to 15 rows
  as at the base, and `people::Event::Forget`, which erases the read
  position. Its queue, `Queue::with_capacity(6)` at the base
  (`inbox.rs:415`), becomes 8 with the two index erases.
- The core's conversation test decides whether turns are read
  (`chat_requester`, or `conversation_requester` once jig 05.5 has keyed
  it on the flag); the class comes from jig 02's
  `TaskRecord.flags.retention`.
Test shape:
- `tests/core/tests/forgetting.rs` (new). Fixtures: 8.1's; a helper
  `forgettable_world(seed: u64) -> World` in
  `tests/core/src/stories/reads.rs`: the read story's world
  (`stories/reads.rs:17`) whose initial conversation is made by jig 03.1's
  `people::Ask::Create` (one member: an agent task under the world's charter 1,
  flagged conversation, label 1, retention forgotten once read) in place
  of `StartChat`, `world.assigned` naming it. `World::configured_unassigned`
  (`tests/core/src/effects/constructors.rs:93`) takes no initial ask: the
  first sign-in sends a hard-coded `StartChat`
  (`tests/core/src/effects/deliveries.rs:153-158`), so the helper builds
  its world with `World::configured_with(.., initial)`, which sets
  `World.initial_ask`, read there (Files).
  Reference test:

  ```rust
  //! A task's retention class decides what its reading erases (people.md 6).
  use jig_core_world::stories::reads::{forgettable_world, request};
  use jig_domain_core as core;
  use jig_domain_people as people;
  use jig_domain_tasks as tasks;
  use jig_test_domain as root;

  fn ended(task: u64) -> root::Key {
      root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
  }

  fn finish(world: &mut jig_core_world::effects::World) {
      world.end(
          tasks::End::Finished {
              result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
              cancel_delegates: false,
          },
          1,
      );
  }

  #[test]
  fn a_conversation_forgotten_once_read_leaves_no_row_once_read() {
      let mut world = forgettable_world(830);
      let task = world.assigned.as_ref().expect("conversation").task;
      world.turn(1, 1, None, b"reply");
      finish(&mut world);
      let root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) = &world.store.rows[&ended(task)]
      else {
          panic!("ended conversation")
      };
      let through = row.result_position;
      request(&mut world, 61, people::Ask::MarkSeen { seen: people::Seen::Chat { task, through: 1 } });
      assert!(world.store.rows.contains_key(&ended(task)));
      request(&mut world, 62, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
      assert!(!world.store.rows.contains_key(&ended(task)));
      for key in world.store.rows.keys() {
          if let root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { task: number, .. })) = key {
              assert_ne!(*number, task);
          }
      }
  }
  ```
- The conformance world (domain/testing.md 5: "a conversation ending
  while its party reads it, the engine restarting between: no turn read
  comes back unread, and a task forgotten once read leaves nothing a
  page shows"): `jig_conformance::scenarios::Scenario::ConversationEndsWhileRead`,
  in `ALL`, its program: two conversations made by jig 03.1's `Create`,
  one kept and one forgotten once read, a turn each, their first turns
  marked, `Step::Crash`, both ended, `Step::Crash`, their turns and
  results marked, `Step::Crash`; its actions `Action::{Conversation {
  forgettable: bool }, Turn, MarkTurns, EndConversations, MarkResults}`.
  The testing application's `check`: the kept conversation's durable
  read position is the turns its party marked, and the store holds no
  row of the forgettable one (record, result update, index rows,
  position). `tests/conformance/tests/scenarios.rs`:
  `conversation_ends_while_read_chosen_cut_ends_the_story` (the
  `chosen_cut` pattern, :18); the fuzzy sweep
  `every_shared_story_survives_every_commit_and_seeded_random_cuts`
  (`fuzzy_scenarios.rs`) runs it at each commit: aim to add at most 0.5 s
  to the fuzzy suite (the eight scenarios' sweep ran in 1.4 s at the
  base; jig 06.1's `RacingDecisions` makes nine); past that, the sweep's cuts for it are made fewer first.
- Also there: `a_kept_chat_stays_after_its_party_read_it` (a
  `StartChat` chat, read through both marks: its ended record and its
  `EndedByParty` row stay),
  `a_forgettable_conversation_read_in_the_other_order_is_forgotten`
  (results first, then the chat's turns), and
  `a_results_mark_forgets_in_the_commit_that_moves_its_position` (turns
  marked, then `Seen::Results`: the one commit of `world.commits` that
  erases `tasks::Key::Ended(task)` also saves the party's
  `people::Stored::ReadResults`).
- `conversation_ends_while_read_chosen_cut_ends_the_story` cuts after
  the commit of the forgettable conversation's results mark
  (`Cut::AfterCommit` at that commit, in place of `chosen_cut`'s 7), so
  the focused gate meets what the fuzzy sweep met at cut 17, seed 757.
  The scenario's `check` stays: the store holds no row of the
  forgettable conversation.
- `crates/jig-domain-core/src/tests/forget.rs` (new; `mod forget;` in
  tests.rs): one table test, `forgets_names_the_party_only_once_its_reading_releases_the_task`,
  its rows a const beside it: kept; requested by a task; a conversation
  with a turn unread; named by a live task (`Reading.named`); the result
  unread; and the one row that forgets, each through `Owner::of_row` and
  then `forgets`. Beside it,
  `an_owner_of_a_cached_result_is_the_owner_of_its_record`: the fixture's
  `Owner::of_row` equals `Owner::of_result` of the `ResultRef` that
  `routing/tasks.rs:716` makes for it. Its record is a new fixture,
  `fixtures::ended_report(number: u64, requester: tasks::Party) ->
  tasks::TaskRecord`, in `crates/jig-domain-core/src/tests/fixtures.rs`
  (result position 7, last reply 2). Under 20 ms.
- The base's forgetting tests move to the forgettable conversation,
  their assertions kept: `marking_an_ended_chats_replies_after_its_result_forgets_it`
  and the helper `all_replies_read_then_ended_survive_restart_and_forgetting`
  (`tests/core/tests/activity.rs:181, 243`),
  `a_forgotten_chats_result_leaves_the_updates_index` and
  `a_forgotten_chat_leaves_both_conversation_families`
  (`tests/core/tests/reads.rs:197, 308`). `a_restart_reads_no_ended_task_whose_result_was_read`
  (`tests/core/tests/restart.rs:479`) keeps its `StartChat` chat, now
  kept: its expected `world.census_ended` becomes 3 (the chat, now kept,
  and the two unread goals), the one changed literal.
- Ceilings: world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(a_conversation_forgotten_once_read_leaves_no_row_once_read) |
test(a_kept_chat_stays_after_its_party_read_it)'` once: a conversation
of class forgotten once read is gone from the real store, its index rows
with it, once its party read its turns and its result, while a kept chat
read the same way stays; under 1 s.
Don't: run retention's timers or passes; erase a reply's `Update` row;
forget a task whose requester is not a party; forget on a read (a
`Read`), only on a mark; decide a forget in any decision but its
mark's; load anything for a `Seen::Results`.
Size: at most 215 lines of production code (about 25 of them in
`testing/jig-conformance`; 40 for `Owner` and the forget from a cached
result, by the design answer of 2026-10-16T19:02Z).
Removes: `Now::LoadEndedChat`, with nothing in its place; the
chat-keyed forget (`forget_chat`'s `chat_requester` test as
the forget's condition); `people::Domain::marked_chats` and its test
`marked_chats_lists_only_that_persons_positions`.

### 08.4 Ended read positions, and the census on them

Scope: store.md 3, 7.2, 7.4 and 15 items 2 and 8; people.md 6;
engine.md 4.2, 5.3, 6 and 11.4: a conversation's read position moves to
the people child's ended family, `EndedRead { task, party, through }` at
people tag 7, in the decision that ends it; the family is not restored
at start; a mark on an ended conversation, the forget of 8.3, an ended
conversation's line and the restart's census read it by a point read; a
forget erases it; a mark's loads are held, and one mark on an ended
conversation loads at a time.
Needs: jig 02.3's people family at version 3.
Files:
- `crates/jig-domain-people/src/`: `ended.rs` (new: the move at the
  end, the ended mark, `chat_mark_loading`, the census's input),
  `boundary.rs`
  (`Forgettable.turns_read`, `Key::EndedRead`,
  `Stored::EndedRead`, `Key::live`, `Event::ChatState`'s
  `seen`), `domain.rs` (restore: an `EndedRead` row is refused; at most
  40 lines), `inbox.rs` (`chat_state`, `chat_seen_decision`: their
  ended arms call `ended.rs`), `lib.rs`, `tests.rs`, `tests/ended.rs`
  (new).
- `crates/jig-domain-core/src/`: `inbox/census.rs` (`Census::admits`,
  `restore_inbox_row(&mut self, limits, record: &Record) -> bool`, one
  pending position), `restart/census.rs` (`Census::interval`, and
  `admits` on it), `restart.rs` (`restart_done`: a position pending
  refuses), `forget.rs` (8.3's `Reading.turns`: a `Seen::Chat`'s the
  position its mark decides, a `Seen::Results`'s from
  `Forgettable.turns_read`), `inbox.rs` (`seen_chat_loaded`),
  `routing/vocabulary.rs` (`Now::LoadEndedRead`,
  `Event::EndedReadLoaded`, `Held::Load`), `routing/people.rs` (the
  `Seen::Results` arm, :279-289, and the ended `Seen::Chat`, :310: held
  loads, one mark at a time), `routing/mod.rs`, `stopping.rs`,
  `reads/vocabulary.rs` (`Row.ended_read: Option<u64>`, :194, ruling
  172; every `reads::Row` literal gains it, `None` but the store's
  indexed join: `git grep -n -E '^\s*unjoined[:,]' -- '*.rs'
  ':!**/generated/**'` prints ten at jig `05e9d00e7`),
  `documents/inbox_deployment.rs` (`conversation_from_row` takes the
  joined position for an ended row), `reads/indexed.rs` (:199),
  `reads/rows.rs` (:1189; at most 40 lines), `inbox.rs:132`,
  `routing/tasks.rs` (:776, `seen: None`; at most 40 lines),
  `inbox/census.rs` (:31, :135), `lib.rs`.
- `crates/jig-protocol-store/`: `src/keys/people.rs` (tag 7),
  `src/intervals.rs` (`RestoreRange::People` ends before tag 7;
  `census_ranges` reads `[JIG, 2, 7]` in place of `[JIG, 1, 15]`),
  `src/restore.rs` (`decode` hands up an `EndedRead` row and then its
  joined ended record, the two in one page and counted as two rows),
  `src/historical.rs` (`Now::LoadEndedRead`'s key
  and answer), `src/reads/indexed.rs` (`joined`: a conversation row's
  `ended_read` is its `EndedRead`'s `through`; ruling 172), `schema/people-v3.schema` (`record
  EndedRead { task: u64 party: u64 through: u64 }`), `src/codec/people.rs`,
  the generated family, the goldens, `src/tests/{intervals.rs, restore.rs,
  codec.rs, fixtures/keys.rs}` (`people_key`'s `below(7)`, :60, becomes
  `below(8)`).
- The roots that hand a census row to the core: `testing/jig-test-domain/src/lib.rs:982`,
  `examples/ops/domain/src/restart.rs:123` (each passes the record as it
  comes to `restore_inbox_row`), and `Held::Load`'s route in
  `examples/ops/domain/src/route.rs` (held as the `Output::CoreLoad` that
  :1368-1374 sends at once at the base), `testing/jig-test-domain/src/routes.rs`
  and `testing/jig-test-service/src/route.rs` (beside `Held::ReadLoad`,
  :214);
  `tests/ops/src/conformance/mod.rs` (`census_owner`, :1071-1078, called
  at :428, maps a `ConversationOldest` or `Result` row to its owner: it
  hands up the `EndedRead` row and then its ended record; at most 40
  lines).
- `tests/core/src/effects/store.rs` (`census_ended`, :50-52, counts every
  census row at the base): it counts only the census's `Stored::Ended`
  rows, so `a_restart_reads_no_ended_task_whose_result_was_read`
  (`tests/core/tests/restart.rs:497`) keeps expecting 3.
- The users of `people::Key`/`Stored` (21 files at the base, from `git
  grep -l ReadChat -- ':!docs' ':!*/generated/*'`, the schema among them): `jig-domain-core/src/routing/{application_routes.rs,
  people.rs, watch_marks.rs}`, `jig-domain-core/src/tests/restore_admission.rs`,
  `jig-protocol-store/src/tests/{codec.rs, fixtures/keys.rs}`,
  `testing/jig-test-service/src/clients.rs`,
  `tests/client/src/joint/wire/cursor.rs` (people tag 7),
  `tests/core/src/{observations.rs, watches.rs}`,
  `tests/core/tests/activity.rs`, `tests/people/src/world.rs`,
  `tests/people/tests/{memory.rs, simulation.rs}`: an arm each.
- The exhaustive lists of `core::Now` (15 files at the base, from `git
  grep -l 'Now::RestoreRefused' -- ':!docs'`, 8.3's list among them):
  an arm each for `Now::LoadEndedRead`; the exhaustive lists of
  `core::Held` (19 files at the base, from `git grep -l
  'Held::NotesLoad' -- ':!docs'`): an arm each for `Held::Load`, which
  the worlds that send a released `Held::ReadLoad` to their store
  (`tests/core/src/effects/deliveries.rs:38`,
  `tests/conformance/src/lib.rs:527`) send there as they send its `Now`
  at the base (`tests/core/src/effects.rs:323-330`); the users of
  `people::Event::ChatState` (11 files, from `git grep -l ChatState --
  ':!docs'`): its `seen` field.
- `tests/conformance/src/scenarios.rs`: 8.3's scenario check reads
  `EndedRead` in `ReadChat`'s place.
- `tests/people/tests/simulation.rs`.
- Downstream: 3 files match `people::Stored` (its engine store, its
  engine world's roles and referee), its engine root's restart
  (`restore_inbox_row`), `now` routes (`Now`) and `held` routes
  (`Held::Load`, the `Now` it wraps routed as `route/now.rs:141` routes
  `Now::SeenChat` at the base), and its client world's
  translation (`ChatState`), absorbed at its repin; and temper's census
  walk, which jig's reference code does not show, repaired as the
  README's "jig 08.4's census" says.
Shape:
- `people::Key::EndedRead { task: u64, party: u64 }`;
  `people::Stored::EndedRead { task: u64, party: u64, through: u64 }`;
  `Key::live` false for it alone (store.md 4.4: "ended conversations'
  read positions | task, then party | on demand").
- The decision that ends a conversation whose requester is a party
  (`Event::ChatState { live: false, .. }` from `routing/tasks.rs:784`):
  the people child removes `chats_seen[(party, task)]`, erases
  `Key::ReadChat`, and saves `EndedRead { task, party, through }`,
  `through` 0 when nothing was read.
- `Event::ChatState` gains `seen: Option<u64>`: the census's joined
  position for an ended conversation; `None` on the live path.
- 8.3's `people::Event::Forget` erases `Key::EndedRead { task, party }`
  in `ReadChat`'s place.
- `Now::LoadEndedRead { request: Token, task: u64, party: u64 }`, keyed
  `people::Key::EndedRead { task, party }`; its answer
  `Event::EndedReadLoaded { request, task, party, through: u64, busy:
  bool }`, `through` 0 when absent. `Seen::Chat` on an ended
  conversation: `SeenChat` loads the record, then `LoadEndedRead`, then
  the mark is decided against that position, saves `EndedRead`, and
  forgets in that decision, its `Reading.turns` the position the mark
  decides: no load follows the mark (people.md 6; the design answer of
  2026-10-16T19:02Z).
- **The mark's two loads are held** (engine.md 4.2: "held: what must
  wait for the decision's commit to be durable: ... a load (so that it
  sees every commit before it)"; 5.3): `core::Held::Load(Box<Now>)`, "A
  point load, sent once every commit before it is durable". `Seen::Chat`
  on an ended conversation pushes `Now::SeenChat`, then
  `Now::LoadEndedRead`, each in a `Held::Load`, never as `Request::Now`,
  which the journal's door sends at once, ahead of the commits not yet
  durable (skein's `journal.rs:252`). A root holds a `Held::Load` in the
  decision and, once the journal releases it, routes the `Now` it wraps
  as the base routes that load (`examples/ops/domain/src/route.rs:1368-1374`).
  jig's other point loads stay `Request::Now` (`later.md`).
- **One mark at a time on an ended conversation** (people.md 6: "One
  such mark loads at a time"): while a `Seen::Chat` of a task waits on
  its loads, the core answers `people::Outcome::Again(people::Again::Busy)`,
  before any load, to another `Seen::Chat` of that task, and to a
  `Seen::Results` whose forget would take that task.
  `people::Domain::chat_mark_loading(&self, task: u64, except: Token) ->
  bool` (`ended.rs`) reads the pending flights, as `seen_context`
  (`inbox.rs:213`) does. 8.3's scenario's `MarkTurns` marks two
  conversations, one mark each, so neither is busy.
- `people::Forgettable` gains `turns_read: bool`, for a conversation:
  set where the ref is made (at the end, from the position that
  decision moves; at the census, from the joined `EndedRead`) and by a
  `Seen::Chat` on the ended conversation, in its decision. 8.3's
  forget from a cached result reads it in place of `chat_seen`.
- `Core::restore_inbox_row(&mut self, limits: &Limits, record: &Record)
  -> bool`: an `EndedRead` row is held as the one pending position, and
  the next row, its ended record, takes it into `ChatState { live:
  false, seen: Some(through) }`; a census whose pending position meets
  another row refuses the restart (`false`).
- `restore_inbox_row` refuses a record after a pending position that
  is not that position's ended task, with that position's party its
  requester; `restart_done(RestartStep::LoadInbox)` with a position
  pending refuses the restart. A record without a pending position
  takes no `ChatState`: an ended conversation's comes with its position.
- **Each ended task once per party, whichever part names it** (store.md
  7.2: "neither part's progress skips a row of the other"). At the
  base, the one `Progress.unread_result` mark per party
  (`inbox/census.rs:116-122`), set by an ended conversation's unread
  result in the first part, skips every lower unread result of the same
  party in the second.
- `Census::interval(&self, key: &Key) -> Option<u32>`: 0 for
  `Key::People(people::Key::EndedRead { .. })`, 1 + k for a `Result` of
  the census's k-th party past its read position, `None` for any other
  key; `admits` is `interval(key).is_some()`, and `census_ranges`
  builds its intervals in that order (store.md 7.2: "The census places
  each key it admits in the walk's order"). temper's root places its
  census cursors by it, and keeps no copy of the order.
- The census's page (`restore.rs`): an `EndedRead` row and its ended
  record count two against the asker's rows, and their bytes against
  its bytes; the page ends before a pair that does not fit, its cursor
  the index row before the pair, and a page whose first pair does not
  fit is cut (store.md 7.2: "never parts a position from its record").
- `conversation_from_row(core, limits, party, row, seen: Option<u64>)`:
  `Some` for an ended row, from the row's `ended_read` (ruling 172).
Test shape:
- `crates/jig-domain-people/src/tests/ended.rs` (new; `mod ended;` in
  tests.rs), the people `LIMITS` (tests.rs:10):
  `an_ended_conversation_moves_its_position_to_the_ended_family`,
  `an_ended_read_position_is_refused_at_restore`,
  `a_mark_on_an_ended_conversation_saves_its_ended_position`. Under 20
  ms each.
- `crates/jig-protocol-store/src/tests/intervals.rs`:
  `the_people_restore_range_stops_before_ended_read_positions`, and
  `the_census_reads_conversations_then_results_in_listed_order` renamed
  `the_census_reads_ended_read_positions_then_results_in_listed_order`,
  its expected first interval `[JIG, 2, 7]`.
- `crates/jig-protocol-store/src/tests/intervals.rs`:
  `the_census_holds_the_keys_it_admits_and_no_others` also holds each
  admitted key to the interval `Census::interval` names.
- `crates/jig-protocol-store/src/tests/restore.rs`:
  `the_census_hands_up_an_ended_read_position_then_its_task`, and
  `the_census_never_parts_an_ended_read_position_from_its_task` (two
  `EndedRead` rows: asked for 3 rows, the page holds one pair, its
  cursor the first `EndedRead`'s key; asked for 1, the page is cut).
- `crates/jig-domain-core/src/tests/restore_admission.rs`:
  `a_census_position_followed_by_another_task_is_refused`.
- `tests/core/tests/restart.rs`:
  `a_restart_keeps_an_unread_goal_older_than_an_unread_chat` (a goal
  ends, then a chat, neither read; after `world.restart`,
  `census_unread_results` lists both, newest first, and
  `census_updates` is unchanged).
- `tests/core/tests/activity.rs`:
  `an_ended_chats_read_position_is_read_back_after_a_restart` (a chat
  with two turns, the first marked, ends; after `world.restart`, the
  store holds `EndedRead { through: 1 }`, no `ReadChat`, and the
  conversation's line reads one unread turn).
  `a_chat_ended_with_its_result_unread_keeps_its_read_position` keeps its
  assertions on `EndedRead { task, party: 1 }` in `ReadChat`'s place.
- `tests/core/tests/restart.rs`: `a_restart_restores_unread_results_and_chat_totals`
  passes unchanged.
- `tests/core/tests/activity.rs`, with `world.store_delay = 3`, as
  `tests/core/tests/application_documents.rs:76` sets it: each commit
  waits three ticks before it reaches the store, while a `Request::Now`
  load reaches it at once (`testing/jig-test-store-files/src/store.rs:117-138,
  198-204`):
  `a_mark_sent_as_its_chat_ends_waits_for_the_end` (a chat with two
  turns, the first marked, ends; its party marks through 2 before the
  end's commit reaches the store: the mark is answered `MarkedSeen`, and
  the store then holds `EndedRead { through: 2 }`; what it catches: a
  load sent as `Request::Now` finds no ended record, and the mark is
  refused unknown);
  `a_second_mark_on_an_ended_chat_while_the_first_loads_is_busy` (a chat
  with three turns ends with none read; marks through 2 and through 1
  sent back to back: the second is answered `Again(Busy)`; sent again
  once the first is answered, it is answered `MarkedSeen`; the store
  holds `EndedRead { through: 2 }`, and the conversation's line reads
  one unread turn).
- `tests/core/tests/forgetting.rs`, with `world.store_delay = 3`:
  `a_results_mark_while_a_chat_mark_loads_is_busy` (the forgettable
  conversation's turns marked, then it ends with its result unread; a
  `Seen::Chat` through its last turn and a `Seen::Results`, back to
  back: the results mark is answered `Again(Busy)`; sent again once the
  first is answered, it forgets the task: no row of it is left,
  `EndedRead` included).
- `tests/core/tests/forgetting.rs`:
  `a_conversation_read_before_a_restart_is_forgotten_by_its_results_mark`
  (the forgettable conversation's turns marked, then `world.restart`,
  then `Seen::Results`: no row of it is left, from the census's
  `turns_read`).
- The people world (domain/testing.md 3: "read positions over
  conversations, live and ended, across a restart"),
  `tests/people/tests/simulation.rs`:
  `a_conversations_read_position_moves_to_the_ended_family_and_reads_back_after_a_restart`
  (the position saved as `EndedRead` at the end, none restored, and a
  mark after the restart decided against the position the world loads).
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(an_ended_chats_read_position_is_read_back_after_a_restart) |
test(a_restart_restores_unread_results_and_chat_totals)'` once: after a
restart, an ended chat reads with the position its person left, and
`me`'s unread count holds six, from the census's joined positions and
the unread results; under 1 s.
Don't: restore an `EndedRead` row at start; hold ended positions in the
people child; read an ended record in the census that no `EndedRead` row
and no unread result row names; part an `EndedRead` row from its record
across pages; load anything after a mark is decided; send a mark's load
as `Request::Now`; let two decisions on one ended conversation rest on
loads made before either was decided.
Size: at most 410 lines of production code (a codec increment; 40 of
them for the census's pairs, places and once per party, by the design
answer of 2026-10-16; 15 for `turns_read`, by the design answer of
2026-10-16T19:02Z; 40 for the held loads and one mark at a time, by
the design answer of 2026-10-16T19:32Z; 15 for `Row.ended_read`, by
ruling 172).
Removes: ended positions in `people::Domain::chats_seen` and the
census's read of tag 15.

### 08.5 Task queries' ended part through the indexes

Scope: store.md 7.3 and engine.md 11.2: a task query's ended group is
read through one index, chosen by `ended_part` (the sketch): the
ended-by-party index from the party, and the label where named; the
ended delegates (tag 17) for a requester task or a parent; the
ended-by-project index for a project, listed tasks only. The query's
other keys filter the index's rows; a page goes on from the last row
examined and may come back cut; each row joins its record and, for a
conversation, its `EndedRead`'s `through` in `reads::Row.ended_read`
(08.4, ruling 172); the core makes its task line.
Needs: jig 05's task query: `documents::TaskQuery`,
`reads::{Read, Scope}::Tasks` with its live group, `PageRow::Line`, and
`documents::task_line` for a loaded ended row.
Files:
- `crates/jig-domain-core/src/`: `reads/ended.rs` (new: `EndedPart`,
  `ended_part`, the filter, the rows into task lines), `reads/vocabulary.rs`
  (`Scope::Tasks { query }`'s stored range: its ended part),
  `documents/task_lines.rs` (`task_line` takes the joined position of an
  ended conversation), `reads/rows.rs`
  (at most 40 lines), `reads/rows/positions.rs` (`examined` takes the
  two keys).
- `crates/jig-protocol-store/src/`: `intervals.rs` (`scope`'s arm: the
  prefix of the chosen index), `reads/indexed.rs` (`ended_tasks`, on
  `family`), `reads.rs` (the dispatch at :66-67 gains `Scope::Tasks`),
  `tests/reads/indexed.rs`.
- `tests/core/tests/ended_indexes.rs`.
- `testing/jig-conformance/src/scenarios.rs`; the testing application's
  bindings, `tests/conformance/src/scenarios.rs` and its deliveries'
  routing in `tests/conformance/src/lib.rs` (and `peers`' match over
  every `Scenario`, :216-253: an arm for the new one);
  `tests/conformance/tests/scenarios.rs`.
- Downstream: its conformance configuration matches every
  `Option<Scenario>` (`tests/conformance/src/config.rs:132-138` there),
  which `QueryWatchedWhileTasksEnd` breaks at the repin.
Shape:
- `jig_domain_core::reads::ended::{EndedPart, ended_part}` as sketched.
- The ended group's positions are `Position::Owner(Key::Tasks(...))` of
  the index read (`EndedByParty`, `EndedDelegate`, `EndedByProject`), so
  jig 05's cursor carries them as it carries the other index keys.
- Filters beyond the prefix: the label (for the ended delegates), the
  project (for the party's index), the root (the row's `ancestors`' top
  or the task), the executor kind, tracked, the requester party (for the
  project's index). A filtered page is cut as store.md 7.3 says.
- The live group's last page carries `next: Some(Cursor::Store { scope:
  Scope::Tasks { query }, after: None })` when `ended_part(&query)` is not
  `None`; when it is, `next` is `None`, which is how the answer says the
  query has no ended part (jig 22's `EndedGroup::none`).
Test shape:
- `crates/jig-protocol-store/src/tests/reads/indexed.rs` (fixtures
  `owner_at`, `keep`, `page`, `load` at :9-30 and `super`):
  `ended_tasks_read_one_partys_label_newest_first`,
  `a_projects_ended_part_holds_listed_tasks_only`.
- `tests/core/tests/ended_indexes.rs`, through jig 05's task query read:
  `a_partys_ended_chats_page_newest_first_after_a_restart`,
  `a_projects_ended_goals_page_from_the_project_index`,
  `a_parents_ended_delegates_page_by_task`,
  `a_filter_past_the_prefix_comes_back_cut_and_goes_on`,
  `a_query_naming_no_index_says_it_has_no_ended_part`.
- The conformance world (domain/testing.md 5: "a task query watched
  while its tasks end, and watched again after each restart: every
  delivery after its commit, the ended ones found through their
  indexes"): `Scenario::QueryWatchedWhileTasksEnd`, in `ALL`, its
  program: the party watches the task query naming it as requester (jig
  05's `Watched::Query`), three of its tasks end, a `Step::Crash` after
  each, and after each restart the party watches the query again and
  reads its ended group; its actions `Action::{WatchQuery, EndTask,
  ReadEnded}`. The testing application's `check`: each delivered line
  names a task whose record the store held when it was delivered, and
  the ended group's rows are the ended tasks, newest first, through
  `EndedByParty`. `tests/conformance/tests/scenarios.rs`:
  `query_watched_while_tasks_end_chosen_cut_ends_the_story`; the fuzzy
  sweep runs it at each commit: aim to add at most 0.5 s to the fuzzy
  suite, as 8.3's.
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(a_partys_ended_chats_page_newest_first_after_a_restart)'` once: a
person's three ended chats come back newest first from the real store's
index after a restart, each line with its unread count; under 0.5 s.
The wire carries task queries from jig 22, whose story runs them at the
joint layer.
Don't: read the conversation or goal index for a task query; scan
ended tasks without an index; order an ended group by the query's live
order.
Size: at most 250 lines of production code (about 25 of them in
`testing/jig-conformance`).
Removes: nothing (the shaped scopes go in jig 16).

### 08.6 The referee: no turn a party has seen comes back unread

Scope: domain/testing.md 6 ("Nothing lost": "no turn a party has seen
comes back unread, across its task's end and restarts (people.md,
section 6)"): the conformance referee observes each committed read
position, live (`ReadChat`) and ended (`EndedRead`), and finds a
violation when one moves back while its task's record is in the store,
or leaves the store in a commit that keeps its task's record. It runs
in each world on the referee: the core world, the conformance world
with 8.3's scenario and its sweep, `ops`'s conformance and the
one-process world.
Needs: 8.4, earlier in this session (the `EndedRead` family).
Files:
- `testing/jig-conformance/src/referee.rs` (`Snapshot.read_positions`;
  the rule's call where a durable snapshot is observed; at most 40
  lines, the file being 1116) and `testing/jig-conformance/src/referee/positions.rs`
  (new, the pattern of `referee/conversations.rs`).
- The snapshots' builders, each filling the field from the store's
  `ReadChat` and `EndedRead` rows: `tests/core/src/observations.rs:593`
  (the core world's and the testing application's),
  `tests/ops/src/conformance/observations.rs:124`,
  `tests/one-process/src/referee.rs:144`; the tests building a snapshot
  with `..Snapshot::default()` stay as they are.
- `tests/conformance/tests/referee.rs`.
- Downstream: its snapshots' builders, `tests/conformance/src/observations.rs`
  and `tests/one-process/src/referee.rs` there, fill no `read_positions`,
  so the rule checks nothing there; nothing breaks.
Shape:
- `jig_conformance::referee::Snapshot.read_positions: BTreeMap<(u64,
  u64), u64>`: by party, then task, the position's `through`.
- `referee/positions.rs`: `pub(super) fn check(before: &BTreeMap<(u64,
  u64), u64>, after: &Snapshot) -> Result<(), Violation>`: for each
  position in `before`, while `after.tasks` holds its task, `after`
  holds the position at a `through` at least as large; a position
  missing from `after` is a violation unless its task is missing too;
  each a `Violation` of `Promise::Lost` ("Nothing lost"), through
  `require` as `referee/conversations.rs` uses it.
  The positions of the last durable snapshot are kept across
  `Observed::Restart`, so a restart that loses one is a violation.
Test shape:
- `tests/conformance/tests/referee.rs` (the pattern of
  `a_cold_store_that_loses_a_drained_party_answer_is_a_violation`, :200,
  its referee from `jig_core_world::observations::Observer`):
  `a_read_position_moving_back_across_an_end_is_a_violation`,
  `a_read_position_erased_without_its_task_is_a_violation`,
  `a_read_position_kept_across_a_restart_passes`. Under 20 ms each.
Proof: world. `cargo nextest run -p jig-conformance-world -E
'test(conversation_ends_while_read_chosen_cut_ends_the_story)'` once:
8.3's scenario, its conversation ended and the engine restarted while
its party reads it, runs under the referee with this rule and finds no
position moved back or lost; under 1 s. The fuzzy sweep runs the rule at
each commit of that scenario with nothing more to run.
Don't: read a domain's state for the rule (the referee watches the
store and the fakes only, domain/testing.md 6); compare positions of
two parties.
Size: at most 50 lines of production code (in `testing/jig-conformance`).
Removes: nothing.

## Interface sketches

Compiled at the base under `cargo clippy -p <scratch crate>
--all-targets -- -D warnings`, in a scratch crate of jig with jig 02's
and jig 05's types stubbed (`documents::TaskQuery` with its executor kind and order elided), as
their files name them: checked at the ready check. Bodies are
`unreachable!("sketch")` but for the two `key` functions and the
algorithms in comments.

```rust
// crates/jig-domain-tasks/src/index.rs (8.1, 8.2)
/// A row of the index of ended tasks by requesting party (store.md 7.4),
/// saved by the decision that ends a task whose requester is a party.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // as `Update`'s, index.rs:34
pub struct EndedByParty { pub party: u64, pub label: u32, pub ended_at: Wall, pub task: u64 }
impl EndedByParty {
    /// Its key: newest ended first within the party and label.
    #[must_use]
    pub const fn key(self) -> Key {
        Key::EndedByParty {
            party: self.party,
            label: self.label,
            ended_at: Newest::of(self.ended_at.as_nanos()),
            task: self.task,
        }
    }
}
/// A row of the index of ended listed tasks by project (store.md 7.4),
/// saved by the decision that ends a task flagged listed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EndedByProject { pub project: u32, pub label: u32, pub ended_at: Wall, pub task: u64 }
// impl EndedByProject { #[must_use] pub const fn key(self) -> Key } as above.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // the base's
pub enum Key {
    // ... the base's variants, then:
    /// An ended task of a requesting party: `(party, label, ended at reversed, task)`.
    EndedByParty { party: u64, label: u32, ended_at: Newest, task: u64 },
    /// An ended listed task of a project: `(project, label, ended at reversed, task)`.
    EndedByProject { project: u32, label: u32, ended_at: Newest, task: u64 },
}
// crates/jig-domain-tasks/src/boundary.rs: Stored::{EndedByParty(EndedByParty), EndedByProject(EndedByProject)}
```

```rust
// crates/jig-domain-people/src/boundary.rs (8.4)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // the base's
pub enum Key {
    // ... the base's variants, then:
    /// An ended conversation's read position, its requester's (store.md 7.4); not live.
    EndedRead { task: u64, party: u64 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)] // the base's
pub enum Stored {
    // ...
    /// Saved by the decision that ends a conversation; `through` 0 when nothing was read.
    EndedRead { task: u64, party: u64, through: u64 },
}
```

```rust
// crates/jig-domain-people/src/boundary.rs (8.3; the design answer of 2026-10-16T19:02Z, not compiled at the ready check)
/// What the forget of a task forgotten once read needs from its unread
/// result's entry (people.md 6); 8.4 adds `turns_read`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Forgettable {
    /// The task's label, for its row in the ended index by requester.
    pub label: u32,
    /// Whether the task is flagged listed, for its row in the ended index by project.
    pub listed: bool,
    /// Its last turn, for a conversation; `None` for any other task.
    pub last_reply: Option<u64>,
}
// `ResultRef` gains `pub forgettable: Option<Forgettable>`: `None` for a kept task.
```

```rust
// crates/jig-domain-core/src/forget.rs (8.3; `Owner` and `forgets` by the design answer of 2026-10-16T19:02Z, not compiled at the ready check)
/// What one forget erases: an ended task forgotten once read, requested by a
/// party, from its loaded record (`Seen::Chat`) or its cached unread result
/// (`Seen::Results`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Owner {
    pub task: u64,
    pub party: u64,
    pub project: u32,
    pub label: u32,
    pub ended_at: Wall,
    pub result_position: u64,
    pub listed: bool,
    /// Its last turn, for a conversation; `None` for any other task.
    pub last_reply: Option<u64>,
}
impl Owner {
    /// `None` unless `row` has ended with a result, is forgotten once read, and its requester is a party.
    #[must_use]
    pub fn of_row(row: &tasks::TaskRecord) -> Option<Owner> {
        unreachable!("sketch")
    }
    /// `None` for a kept task's result.
    #[must_use]
    pub fn of_result(party: u64, result: &people::ResultRef) -> Option<Owner> {
        unreachable!("sketch")
    }
}
/// What one ended task's forget needs to know, read in the mark's decision.
#[derive(Clone, Copy, Debug)]
pub struct Reading {
    /// The requester's read position over results.
    pub results: u64,
    /// The requester's position over the task's turns; `None` when not a conversation.
    pub turns: Option<u64>,
    /// Whether a live task names it (a stub exists).
    pub named: bool,
}
/// The requesting party whose reading forgets `owner` now, if any (people.md 6).
/// `Owner::of_row` has checked the class, the requester, the phase and the result.
#[must_use]
pub fn forgets(owner: &Owner, reading: Reading) -> Option<u64> {
    // if reading.named || reading.results < owner.result_position { return None; }
    // match (owner.last_reply, reading.turns) {
    //     (Some(last), Some(seen)) if seen >= last => Some(owner.party),
    //     (Some(_), _) => None,
    //     (None, _) => Some(owner.party),
    // }
}
```

```rust
// crates/jig-domain-core/src/reads/ended.rs (8.5)
/// The index serving a task query's ended part (store.md 7.3), chosen by
/// the query's keys in this order: a requester party; a requester task or
/// a parent; a project.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndedPart {
    /// The ended-by-party index, from the party, and the label where named.
    ByParty { party: u64, label: Option<u32> },
    /// The ended delegates of the requester task.
    Delegates { requester: u64 },
    /// The ended-by-project index, listed tasks only, and the label where named.
    ByProject { project: u32, label: Option<u32> },
    /// No index serves it: the page says it has no ended part.
    None,
}
/// Choose the index of `query`'s ended part (jig 05's `documents::TaskQuery`).
#[must_use]
pub fn ended_part(query: &TaskQuery) -> EndedPart {
    // if let Some(phase) = query.phase && phase != PhaseClass::Ended { return EndedPart::None; }
    // match (query.requester, query.parent) {
    //     (Some(tasks::Party::Person(party)), _) => EndedPart::ByParty { party, label: query.label },
    //     (Some(tasks::Party::Task(task)), _) | (None, Some(task)) => EndedPart::Delegates { requester: task },
    //     (Some(tasks::Party::Deployment { .. }), _) => EndedPart::None, // jig 05 refuses it as unknown
    //     (None, None) => match query.project {
    //         Some(project) => EndedPart::ByProject { project, label: query.label },
    //         None => EndedPart::None,
    //     },
    // }
}
```

Who owns what: the tasks child saves the two ended indexes and the
store encodes them; the people child owns `EndedRead` and moves it at a
conversation's end; the core loads it (`Now::LoadEndedRead`, in a
`Held::Load`) for a mark on an ended conversation, one mark at a time,
and the store joins it for a read and the census. A forget ends a
forgettable task's rows; retention (not planned) ends a kept one's.

## Questions to raise when reached

- **The census reads each ended conversation's position** (8.4), as the
  base reads each ended conversation: store.md 7.2 asks "never every
  ended task". Raise it when 8.4 lands, with `world.census_ended` (the
  core world's count of ended records the census read,
  `tests/core/src/effects.rs:82`) after
  `a_restart_restores_unread_results_and_chat_totals`.

## Done when

- Each of `jig-08.1` to `jig-08.6` has landed: for k in 1 to 6, `git -C
  ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log origin/main
  --all-match --grep='^Plan: client-api$' --grep="^Step: jig-08.$k$"
  --format=%H -1` prints a commit.
- In a worktree of jig's `origin/main`, once: `cargo nextest run -p
  jig-core-world -p jig-tasks-world -p jig-conformance-world -E
  'test(=an_ended_chat_is_indexed_by_its_requesting_party_and_label) |
  test(=an_ended_goal_is_indexed_by_its_project_and_label) |
  test(=a_delegate_moved_to_a_party_is_indexed_by_that_party_when_it_ends) |
  test(=a_conversation_forgotten_once_read_leaves_no_row_once_read) |
  test(=a_kept_chat_stays_after_its_party_read_it) |
  test(=an_ended_chats_read_position_is_read_back_after_a_restart) |
  test(=a_partys_ended_chats_page_newest_first_after_a_restart) |
  test(=conversation_ends_while_read_chosen_cut_ends_the_story) |
  test(=query_watched_while_tasks_end_chosen_cut_ends_the_story) |
  test(=a_read_position_moving_back_across_an_end_is_a_violation) |
  test(=a_read_position_erased_without_its_task_is_a_violation)'`
  passes these eleven tests and runs no other.
- `git -C ~/src/rust/jig grep -n -E 'RestoreRange::People =>.*\[keys::JIG, 2\]\)' origin/main
  -- crates/jig-protocol-store/src/intervals.rs` prints nothing (the
  people range no longer takes tag 7).
