# Session 16: the shapes of work leave

The reference UI's lists of chats and goals become task queries and its
chat and goal acts go through `jig_client::shapes`; then the shapes of
work leave jig: the 41 shape-specific records go from the wire (jig 13
removed the ten of the small store), with their translations, the
contract's types and the scripted engine's models of them; and the
engine's shape-specific code goes from the core, the people and tasks
children, the store and the configuration. `ops`'s wire crate is jig
21's, not this session's. Nothing named
for a chat, a goal or a list remains in the engine or the wire
(code-organization.md, 6).

## Read first

- jig's `docs/design/client/domain/pages.md`, sections 3, 4, 6 (6.2 and
  6.3) and 12.5; `client/domain/contract.md`, sections 1, 3.4, 3.6, 5.1,
  6.1 and 7.1; `client/domain/client.md`, section 19;
  `protocol/client.md`, section 3; `domain/engine.md`, sections 10 and
  11.4.
- The plan's `decisions.md`, 3; `findings/wire-and-client.md`, sections 1
  (the 51 records by name) and 2 (what goes from the page half);
  `findings/engine-parties-and-tasks.md`, section 2 (the shape-specific
  code to shed and its users); `findings/engine-agents-store-config.md`,
  section 3 (the configuration's `chats`).
- jig 14's, jig 22's and jig 15's session files: their interface sketches are the
  types this session moves the reference UI onto.
- skein's `programming-model.md`, sections 4.4 and 6.3;
  `testing-strategy.md`, sections 2.5, 4 and 7; skein's
  `docs/design/codec.md`, sections 3 and 6.

## What the design asks

- **The reference UI's lists are task queries** (pages.md, 12.5):
  conversations watch "`tasks{query}`: the party as requester, the chat
  template's label, the section's project if any, live, by last
  activity" and read "the same query, ended, by when they ended: the
  index of ended tasks by requester"; a project's goals watch "the
  project, tracked, live, by priority" and read "the project's ended
  tasks of the goal template's label, by when they ended: the index of
  ended tasks by project".
- **Its chat and goal acts go through `shapes`** (pages.md, 4 and 6.2):
  "Starting a chat: a project and the first words. `shapes::chat` fills
  the project's chat template into a `Create`"; "Closing is
  `shapes::close_chat`'s compound request"; "Moving a goal up or down is
  ... one compound request of priority-only amendments, one for each
  goal whose priority changes ... It names no revision"; setting a goal
  is `shapes::set_goal`, the form saying before it is sent whether the
  goal will be made, proposed and to whom, refused, or cannot be set.
- **No shape on the wire** (protocol/client.md, 3): "No record names a
  shape of work"; and none in the engine: "No operation, document or
  subject names a shape of work: a chat and a goal are tasks with
  fields" (contract.md, 1).

## Choices this plan makes

- **The chat template's label and the goal template's label are read
  from `me`** (`shapes::template(me, project, shapes::CHAT)`'s and
  `GOAL`'s `label`), never written as numbers in the reference UI; a
  project with no such template shows no conversations section, and no
  goals board, saying so.
- **Each of the 51 records goes, or is renamed in place to its general
  form** (the table in 16.4 and 16.5): `NoteScopeGoal` becomes
  `NoteScopeTask { project, task }`, the scope of a tracked task, which
  the notes child keeps under its own name (engine.md, 10 names "a goal"
  among the notes' scopes); `ReplyPosition` becomes `ReadPosition {
  task, through }`; `OwnerInboxObjectReply` becomes
  `OwnerInboxObjectTurn { message }`. `Write`, `MadeWords` and
  `SeenReplies` went in jig 14.
- **The lists leave with their engine code, and the asks with theirs:**
  each removal takes a record, its translation at both ends, its
  contract type, the scripted engine's model and the engine's code
  together, so that no exhaustive match maps a removed thing to
  nothing.
- **The scripted engine models the operations the reference UI now
  sends** (`Create` from a chat or a goal template, `Propose`, and the
  compounds closing a chat and moving goals) in place of `StartChat`,
  `SetGoal`, `Close` and `Prioritise`, giving the same outcomes its
  stories assert; this ends jig 14's "answered as not offered".

## Rules that bind it

- **No shape of work in the engine or the wire** (code-organization.md,
  6): the README's "Done when" 5 grep prints nothing after 16.5, and no
  operation, document, subject, record or index named for a chat, a goal
  or a list is added.
- **Deletion-heavy increments count added lines only** (the count of
  code-organization.md, 7, which counts added lines); a removed
  mechanism's tests are moved to its replacement or removed (README,
  Rules).
- **jig never names the repository downstream of jig:** its engine
  root's route of `people::Ask`, its store's sizing of `people::Ask` and
  its records of `people::Refusal::Disposal`, its family's refusal
  translation and its worlds' stories (23 files at the base, 3 of them
  production, by this session's third "Done when" grep run there)
  follow these removals at its repin; code-organization.md, 6's grep for that name prints nothing.

## Corrections at jig `9f6f80839` (ruling 284, the ready check of 2026-10-17)

These take precedence over the Shapes, Test shapes and sketches below
where they differ. Refresh any further copy of a moved or renamed item,
and list it in the commit body.

- **Every increment:** `crates/jig-domain-client` and `crates/jig-view`
  are gone: their files are under `crates/jig-client-view/src/` (the
  conversations section is `sections/conversations.rs`; `page/`,
  `sections/`, `address*`). Read each such path below as its file there.
- **16.1:**
  - Files gain the scripted engine's `queries.rs` (`selects` refuses
    ended phases at :10, `line` sets `ended_at: None` at :71,
    `conversation_read` reads `Read::Conversations` at :121); its
    `reads.rs` (:113 `EndedGroup::None`, :217 refuses an ended cursor) is
    there already.
  - The inbox's reply entry keeps `ConversationLine` until 16.4, and the
    reference UI's reply rows read it until then: 16.1 removes the other
    uses of `ConversationLine`.
  - The joint test's `stories::` is `jig_client_world::stories` in
    `tests/client-protocol/tests/protocol.rs:2`: the test writes `use
    jig_client_protocol_world::stories as bytes;` and calls
    `bytes::ready`, `bytes::start_chat`, `bytes::end_chat`,
    `bytes::conversations` and `bytes::finish`. Files gain
    `tests/client-protocol/src/stories.rs` (the three new helpers) and
    `tests/client-protocol/src/front.rs` (`task_queries_read`).
- **16.3:** Removes says "the engine's `Bounds::Goal`", not the reference
  UI's: the reference UI reads `Bounds::Create(CreateBounds { .. })`
  already (`page/project/goals.rs:169`); `Bounds::Goal` is left in the
  engine (`jig-domain-core/src/acts/bounds.rs`, `documents/offers.rs`,
  `jig-protocol-engine-clients/src/translate/task.rs`, and their tests),
  for 16.5. `Operation::Amend` takes `project`, `task` and `reason`, and
  `Amendment` has no `Default`: a `..` in a sketch is prose.
- **16.4:**
  - The inbox's reply entry holds a `TaskLine` where it held a
    `ConversationLine`: the schema's `InboxEntry` (`client-v8.schema:600`),
    the contract's `documents.rs:654` and `owner_inbox.rs:26, :157`, the
    core's `documents/inbox_deployment.rs`, and the reference UI's reply
    rows join Files.
  - Files gain these users of the records that go: `tests/client/src/joint/**`,
    `tests/client-wire/tests/fuzzy_client_wire.rs`,
    `tests/ops/src/client/{stories,transport}.rs`,
    `tests/core/src/{referee/offers,stories/reads,watches}.rs`,
    `tests/core/tests/{activity,reads}.rs`, the component's
    `translate/owner.rs`, `crates/jig-protocol-wire/src/tests/fixtures/read_values.rs`,
    and `crates/jig-domain-core/src/{acts/objects,acts/requests,documents/offers,documents/deployment}.rs`.
- **16.5:**
  - The engine's `acts::GoalProcedure` becomes `acts::CreateProcedure`
    (jig 22.1 renamed the wire's and the contract's alone): `acts.rs:93,
    :113`, `acts/bounds.rs:59`, `documents.rs:432`,
    `documents/offers.rs:236, :294`, `tests/acts.rs:737`,
    `tests/frame_facts.rs:175`, `tests/client/src/joint/wire/offers.rs:168`.
  - `testing/jig-test-service/src/family/facts.rs` does not exist: the
    service's chat names are in `src/clients/translation.rs`,
    `src/route.rs` and `src/clients.rs`.
  - `closing_chat.rs` is `tests/core/src/stories/closing_chat.rs`, and
    `procedure_goals.rs` is `tests/core/tests/stories/procedure_goals.rs`.
- **Done when:** the third grep lists 221 files at jig `9f6f80839`, not
  207.

The check compiled the 16.1 interface sketch (as
`crates/jig-client-view/src/queries.rs`) and, with the `bytes` import,
its joint test under clippy, and ran `jig-client-protocol-world` and
`jig-client-view`: 625 of 626 focused (the new test, its helpers
stubbed), 2 of 2 fuzzy.

## Corrections at jig `738c20adc` (ruling 287, the ready check of 16.2 and the re-check of 16.1, 2026-10-17)

After jig 15.1 to 15.4. These take precedence over the corrections above
and the Shapes below where they differ. The reference UI's paths below
are under `crates/jig-client-view/src/`.

- **16.1:** the older edge's button is "Read earlier"
  (`JigPhrase::ReadEarlier`, `pages.rs:48`), not "Older": the test
  presses it. The sketch's `pub const fn`s take `#[must_use]`
  (`clippy::must_use_candidate`), and `Box` is `alloc::boxed::Box` (the
  crate is `no_std`). `page/conversations.rs` is 591 lines. The
  Conversations entry is gated at `page/navigation.rs:54-61` on
  `frame.charters[].chats`: it gates on `me`'s chat template, as "Choices
  this plan makes" says, and the file joins Files.
- **16.1 and 16.2, users of the subjects they leave:** `page/shown.rs:262-298,
  :381-388`, `page/project.rs:243-262`, `ui/borrowed/project.rs:34-60`,
  `page/replies.rs:25-30`, `page/routing.rs:889`, `page/opening.rs:26-27,
  :46-52`, `application.rs:135, :195, :226`, `ui/borrowed.rs:90, :244`
  join Files where they name the subject the increment leaves.
- **16.2, Files:** the goals code is `page/project/goals.rs` (344 lines),
  `sections/project/goals.rs` (371), `page/project/owner_goals.rs` (82)
  and `page/project.rs`. The move act is `ActKind::Prioritise`:
  `acts.rs:142, :250-253, :317, :449`, `confirm.rs:461, :548, :653`,
  `confirmation.rs:275`, `outcome.rs:37` (`OwnerOutcome::Prioritised`).
  Users of `Goals`, `GoalLine`, `GoalPage` and `Ask::Prioritise`:
  `sections/project/owner.rs:569` (`GOALS_WINDOW`) and the exhaustive
  `Ask::Prioritise` arms of `sections/project/owner.rs`,
  `page/project/owner_form.rs` and `activity/sending.rs`;
  `sizes/pages.rs:158-231`; `tests/domain/pages.rs:806-916, :1649-1661,
  :2594-2608, :2692-2714, :2948-2949`, `tests/work_pages.rs:336-344`,
  `tests/page_capacity/project.rs:33-86`, `tests/domain/drafts.rs:127`;
  the scripted engine's `reads.rs:20, :90, :134` and
  `testing/jig-client-test-records/src/page_reads.rs`. temper's
  `crates/temper-client-shell-terminal/src/tests/fixtures.rs:305, :618`
  match contract variants the contract keeps: no change there.
- **16.2, the arrows' offer:** the move arrows stay gated on the
  project's `Prioritise` offer, which the engine gives in the project's
  document as in the goals' (`Subject::Goals { project } |
  Subject::Project { project }`, `jig-domain-core/src/acts/objects.rs:48-49,
  :86`): the board reads it from the project's document once it stops
  reading `Goals`, and `acts.rs:250-253` builds the compound in place of
  `Ask::Prioritise`.
- **16.2, the compound:** each operation is the whole `Operation::Amend`
  (`contract/operations.rs:9`); `Amendment` has seven fields and no
  `Default` (`contract/asks.rs:31-40`):

  ```rust
  Operation::Amend {
      project,
      task: goal,
      revision: None,
      change: Box::new(Amendment {
          addressed: None, spec: None, wake: None, dependencies: None,
          authority: None, instructions: None,
          tracked: Some(Tracking::Tracked { priority }),
      }),
      reason, // as the reference UI's other amendments give it
  }
  ```

  The scripted engine admits a compound of priority-only amendments:
  today it refuses as `NotOffered` any compound but `[Message, Release]`
  and `[Amend, Release]` (`testing/jig-client-scripted-engine/src/requests.rs:95-98`,
  `:319-325`, `:683-697`, `compound_target`); all three change.
- **16.2, the overview:** it shares the board's watch of
  `queries::goals(project)` and shows its first rows: both open it
  (`page/opening.rs:46, :50`) and `open_once` keeps one watch per
  subject (`page/routing.rs:49-59`). "Under its own handle" goes.
- **16.2, the refusal:** the reference UI's type is `Refusal::Authority {
  lacking, proposable: Option<bool> }` (`words/refusals.rs:40`), not the
  wire's `RefusalAuthority`. "Propose instead" is a new phrase: Files gain
  `phrases/jig.rs` (the variant near `:668`, its id near `:1432`, its
  classification and its list) and `phrases/english.rs`; temper names
  `JigPhrase` variants alone, and a new one breaks nothing there.
- **16.2, what it proposes:** its stories propose a refused `Amend`
  (naming its revision: `ProposeAction::Amendment.revision` is required,
  and a priority-only amendment is never beyond the party). A refused
  `Create` cannot be proposed yet: the scripted engine refuses `Create`
  and a batch's `Propose` as `NotOffered` (`requests.rs:90-94`), and the
  goal form sends `Ask::SetGoal` until 16.3 (`page/project/goals.rs:220`).
  "Propose instead" for a refused `Create` lands in 16.3, with a story of
  its own, once the engine models `Create`.
- **16.2, Needs** gains 16.1's scripted-engine reads: the ended goals
  rest on its `queries.rs:10` and `reads.rs:217` changes.
- **16.2, Proof:** "one keyed request" is a delta: `requests_made()`
  after the move less before it is 1, as `tests/client-protocol/tests/protocol.rs:216-220`
  counts (`front/faults.rs:107` counts every People answer, and `ready`
  starts a chat).
- **16.2, Size:** the 300 lines count the board, the overview, the
  refusal and the engine's compound; the users' edits listed above count
  apart, as one count in the report.

## Corrections after jig 16.1's block of 2026-10-17T22:31:24Z (ruling 288)

These take precedence over the corrections above and the Shapes below
where they differ.

- **16.1, Files** gain the engine's two transitional handlers in
  `crates/jig-domain-core/src/routing/parties.rs`: the task `StartChat`
  makes (`label: 0` at `:229`) takes label 1, and the task `SetGoal`
  makes (`label: 0` at `:170`) takes label 2: jig's default chat and goal
  templates' labels (jig's `domain/core.md`, 7.3: "| label | 1 | 2 |"),
  as their flags already are those templates' flags. Each line gets a
  comment naming 16.5, which removes both handlers. The engine reads no
  template for it (`domain/engine.md`, 11.1: "No decision reads one").
- **16.1, Test shape** gains two focused tests in
  `tests/client-protocol/src/front/effects.rs`: the author's
  `starting_a_chat_assigns_the_chat_templates_label`, and
  `setting_a_goal_assigns_the_goal_templates_label`, each reading the
  expected label from `jig_core_world::fixtures::templates()`.
- **16.2 and 16.3:** no change. 16.2's ended goals find the goals the
  goal form's `SetGoal` makes; 16.3 moves the forms to `shapes`, whose
  tasks carry the template's own label.

## Corrections after jig 15.6's block of 2026-10-17T23:03:01Z (ruling 290)

- **16.3, Files and Test shape** gain the joint story
  `tests/client/src/joint/role_stories/proposed_story.rs`, under its
  name. Once the goal form sends what `shapes::set_goal` gives, its goal
  beyond the pool is proposed as its confirmation said: its answer is
  the `people::Outcome::Proposed { .. }` the referee's rule asks, in
  place of its durable refusal; both its verdicts are `Passed`; and the
  expected offers-kept failure 15.6 gave it goes (15-helpers.md, ruling
  290). If 16.3 lands before 15.6, 16.3 changes the story so, and 15.6
  gives it nothing.
- **16.1 and 16.2:** where a landing moves only the time of that
  expected failure (`Time::from_nanos(8_416_106)`, `proposed_story.rs:54`
  on jig `25fd50d50`), it takes the time the story's failure prints, keeps
  the fault and its words, and lists it in the commit body (the steer of
  2026-10-17T23:27Z). Any other difference is a block.

## Corrections after jig 16.1's block of 2026-10-17T23:27:43Z (ruling 294)

- **16.1, Files** gain the channel world's store observer,
  `tests/channel/src/world/store.rs`. It reads `skein-kv` commit frames
  from every file under `store/`: at a recovery read (`:72`), at a write
  (`Capture::pending`, `:89-92`) and at a sync (`:163`). After a power loss
  inside a commit's write, the restart writes a repair snapshot
  (`store/snapshot.tmp`, renamed `store/snapshot`: `skein-kv`'s
  `src/store.rs:909`, `:982`), whose 18-byte `SKVS` header and chunks are
  no commit frames; 16.1's changes move the fuzzy draw `calm(6)`, power
  loss before submission 118, onto that path, and the frame check at
  `:257` fails on the snapshot's first chunk
  (`logs/jig-16-1/gate-cause.md`). The observer reads frames only from
  commit logs (`store/log-<20 digits>`, `src/store.rs:433`), at each of
  those sites and any further one; a snapshot's writes, reads and syncs
  give it no frames, and the commits the snapshot holds stay as the
  observer recovered them from the log it replaces, through the next
  restart. The check at `:257` and every other conformance check stay.
- **16.1, Test shape** gains a focused test in `tests/channel`: a power
  loss inside a commit's write, then the restart's repair snapshot; the
  observer takes no frames from the snapshot, and the commits before the
  cut stay durable. The preserved reproduction
  (`logs/jig-16-1/gate2-power-loss-reproduction.rs`) may be that test,
  if it asserts that the snapshot's write was reached.
- **temper:** its one-process durable referee is a copy of this observer
  (`tests/one-process/src/referee/durable.rs:76, :94, :149`, the check at
  `:256`); temper 04.3 takes the same repair (`temper/04-engine.md`).

## Corrections at jig `3b58a775f` (ruling 297, the ready check of 16.3 and 16.4, 2026-10-18)

After jig 16.1, with 16.2 running. These take precedence over the
corrections above and the Shapes below where they differ. The reference
UI is `crates/jig-client-view/src/` (`acts.rs`, `intents.rs`, `page/`):
`crates/jig-domain-client` and `crates/jig-view` do not exist.

- **16.3, the shapes' arguments** (`crates/jig-client/src/shapes.rs:71,
  :77, :101`): `shapes::chat(me, project, words, now)`,
  `shapes::close_chat(me, project, chat, &delegates, reason)` and
  `shapes::set_goal(me, project, draft, reason, now)`. `now` is
  `client.wall()`, as `cards/chip.rs:145` reads it; `reason` is
  `words(fields, Purpose::Reason)`, as `acts.rs:247` gives `Ask::Cancel`
  its reason.
- **16.3, the stories' `me`:** `shapes::chat` and `shapes::set_goal`
  answer `NotOffered::Creating` while `MeProject.creating` is `None`, and
  `NotOffered::Template` without the template (`shapes.rs:63-70`,
  `:106-108`). In `tests/client/src/stories.rs`:
  - `frame()` (`:31`, `creating: None`) gets `creating: Some(Creating {
    within, beyond })`, with the `CreateBounds` and `Beyond` that
    `goal_world` gives its project's `SetGoal` offer (`:697-713`: budget
    1, maximum 100, charters `[1]`, `Beyond::Proposed` to role 1);
  - `stories/conversations.rs`'s `templates()` gains the goal template:
    `number: jig_client::shapes::GOAL`, `TemplateExecutor::Charter(1)`,
    `TaskContractDocument::Report { words: 32 }`, `TemplateBudget::Given`,
    `TemplateDeadline::Given`, label 2 (ruling 288);
  - `goal_world`'s scripted rule (`:719-741`) asks what
    `shapes::set_goal` gives for the story's draft, `Ask::Propose {
    project: 1, action: ProposeAction::Batch { .. }, reason }`, in place
    of `Ask::SetGoal`, or goes where the scripted engine models `Propose`
    itself. Its answer, `Outcome::Proposed { proposal }`, and the story's
    expected values (`tests/client/tests/stories.rs:239`) stay.
- **16.3, users of what goes:** `Me.charters`, `MeProject.chat_budget`
  and `OfferedCharters` have users the Files do not name; they join Files:
  `tests/client/src/stories.rs:44-55`; `tests/client/src/joint/wire/owner.rs:80,
  :89, :813, :869` and `joint/wire/deployment/tests.rs:648`;
  `tests/client-wire/tests/fuzzy_client_wire/documents.rs:10`;
  `testing/jig-client-test-records/src/owner.rs:14-60`; the scripted
  engine's `payload.rs:394-395`, `requests.rs:357`, `tests.rs:51-56` and
  `tests/frame_facts.rs:16-35`. At the increment's start, `git grep -n -w
  -e chat_budget -e OfferedCharters -- crates testing tests` lists them;
  refresh any further user it lists, and the `charters` field of any
  further `Me` value, and name each in the commit body.
- **16.3, the chat counters:** both count `Outcome::Started`, which a
  `Create` does not give (it gives `Created { first, last, given }`,
  `jig-domain-people/src/boundary.rs:403`): the front's
  (`tests/client-protocol/src/engine.rs:232-245`, through `front.rs:95`)
  and the native client's (`tests/client-protocol/src/native.rs:156-159`,
  asserted at `tests/client-protocol/tests/native.rs:6`). The front's
  counts the tasks made under the chat template's label, as the Files
  say; the native client's counts each `OwnerOutcome::Created` that
  answers the `Ask::Create` `shapes::chat` gave. Both assertions keep 1;
  `native.rs` joins Files.
- **16.4, users the start grep misses:** the Files' `rg` searches three
  crates' `src`. The list names have users in `tests/`, `testing/` and
  other crates: at jig `3b58a775f`, the `rg` below lists 105 files, among
  them `tests/tasks/tests/{activity,person,restart,funding}.rs`,
  `tests/tasks/tests/lifecycle/goals.rs`,
  `tests/tasks/src/{world,accounting_referee}.rs`,
  `tests/core/tests/effects.rs`,
  `tests/conformance/src/scenarios/conversations.rs`,
  `tests/ops/src/joint/reads.rs`, `tests/ops/src/joint/reads/goals.rs`,
  `tests/client/src/scripted.rs`, `crates/jig-protocol-engine-clients/src/`
  and `crates/jig-protocol-store/src/{historical,reads,reads/paged}.rs`.
  At the increment's start, after 16.2 has landed, run the Files' `rg`
  over `crates testing tests`, with `-e 'Key::Goal' -e
  'Subject::Conversations' -e 'Subject::Goals'` added: each file it lists
  joins Files, and the commit body names those the Files do not.
- **16.4, the store's files:** its tests are
  `crates/jig-protocol-store/src/tests/{reads/indexed.rs, intervals.rs}`.
  Tags 14 to 16's codecs are `codec/tasks.rs:142-172` and
  `codec/tasks/index.rs` (`goal_row` and `read_goal_row` at `:65-75`,
  `conversation` and `read_conversation` at `:135-148`), on the wire of
  `generated/tasks_v3.rs`: these join Files.
- **16.4, the owner's inbox:** `OwnerInboxEntry.reply: option
  ConversationLine` (`client-v8.schema:629`) holds a `TaskLine`, as
  ruling 284 says of `InboxEntry` (`:600`).
- **16.4, Proof:** its second test is 16.2's: the filter runs both once
  16.2 has landed, which 16.4 needs.

## Corrections at jig `4fad1fcc8` (ruling 298, the re-check of 16.3 after 16.2)

These take precedence over ruling 297's "16.3, the stories' `me`" where
they differ; its other points stand.

- **16.3, the goal template:** 16.2 added it to `templates()`
  (`tests/client/src/stories/conversations.rs:21-27`), cloned from the
  chat's. It takes core.md 7.3's goal fields that `shapes::set_goal`
  reads: `executor: TemplateExecutor::Chosen`, `budget:
  TemplateBudget::Given`, `deadline: TemplateDeadline::Given`. 16.2's
  label 2, `tracked: Some(0)` and flags stay. With the chat's `AsAllowed`
  budget, `fill` sets the task's spend to 0, so `fall` judges the story's
  budget 2 `Within` and gives a `Create`, not the proposal the story
  expects; with `Charter(1)`, the procedure story's draft is
  `NotOffered::Executor`.
- **16.3, each goal world's bounds:** `frame()` gets `creating:
  Some(..)` with `goal_world`'s bounds, as ruling 297 says. `goal_world`
  (`tests/client/src/stories.rs:694`) and `procedure_goal_world` (`:746`,
  procedures `[{ connector: 2, code: 7 }]`) each set their `me`'s
  `creating` to the `CreateBounds` and `Beyond` of their project's
  `SetGoal` offer. Each one's scripted rule asks what `shapes::set_goal`
  gives for its story's draft, an `Ask::Propose` with
  `ProposeAction::Batch` (both stories' budget 2 is beyond the bounds'
  1), in place of `Ask::SetGoal`.
- **16.3, Test shape** adds the procedure story,
  `a_procedure_only_goal_form_confirms_and_sends_the_supplied_executor_once`
  (`tests/client/tests/stories.rs:283`): it keeps its name and expected
  values, as the two stories named there do.

## Corrections at jig `4fad1fcc8` (ruling 299, the ready check of 16.5, 2026-10-18)

These take precedence over 16.5's Files and Shape, and over ruling 284's
16.5 points, where they differ. Refresh any further copy, and list it in
the commit body.

- **16.5, the users:** the third "Done when" grep names neither the
  engine's outcomes and settings that 16.5 removes nor the contract's
  forms of the records. Its users are what this grep lists at the
  increment's start, in place of the third grep (259 files at jig
  `4fad1fcc8`: the third grep's 223, and 36 more, among them
  `jig-domain-core/src/{documents/small.rs, funding.rs, lib.rs,
  routing/application_routes.rs, routing/tasks.rs, sibling_routes.rs}`,
  `jig-protocol-store/src/codec/people/outcomes.rs`, the component's
  `translate/{answers.rs, history.rs}`, the reference UI's
  `{outcome, owner_history, owner_outcome}.rs`, `testing/jig-test-domain`
  and the worlds):

  ```
  git grep -l -E '\b(StartChat|SetGoal|Prioritise|GoalExecutor|Disposal|DisposalIssue|TakeOver|ConversationLine|GoalLine|OfferedCharters|BoundsGoal|GoalProcedure|ReplyPosition|MadeChat|MadeGoal|RefusalDisposal|LimitConversationWords|ReadConversations|ReadGoals|SubjectGoals|NoteScopeGoal|OwnerHistoryChangePrioritised|close_grace|chat_authority|chat_admit)\b|\b(Owner)?Outcome(::)?(Closing|GoalStarted|GoalProposed|Prioritised)\b|\bOutcome::Started\b|\bOwnerHistoryChange::Prioritised\b|\bAct::Close\b|\bBounds::Goal\b' -- '*.rs' ':!*/generated/*'
  ```

  `Settings.chat`'s readers, which no name finds, join them:
  `jig-domain-core/src/{charters.rs:56, documents/small.rs:147,
  roles.rs:114, routing/parties.rs:233, tests.rs:1054,
  tests/frame_facts.rs:42}` and `jig-protocol-config/src/tests/domain.rs:100`.
  The tasks child's `Ask::Close` (`routing/tasks.rs:1370`) and the ops
  example's (`examples/ops/domain/src/route.rs:573`) are other children's,
  and stay.
- **16.5, `Bounds::Goal` and `GoalProcedure`:** the renames to
  `Bounds::Create` and `CreateProcedure` reach the 13 files `git grep -l
  -E '\bBounds::Goal\b|\bGoalProcedure\b' -- '*.rs' ':!*/generated/*'`
  lists, ruling 284's eight and `jig-protocol-engine-clients/src/tests/documents.rs`
  and `tests/client/src/joint/wire/me.rs` among them. The check applied
  both renames: the workspace is clean under clippy, and the default and
  fuzzy suites of the four crates they touch passed.
- **16.5, the schemas:** Files gain the schemas whose records go:
  `crates/jig-client-protocol-wire/schema/client-v8.schema`,
  `crates/jig-protocol-store/schema/people-v3.schema`,
  `testing/jig-client-test-protocol-wire/schema/test-client-v1.schema` and
  `testing/jig-test-store/schema/test-connector-v1.schema`. Each is edited
  in place, under its version, as "Choices" says and jig `73de61fc8` did;
  then `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
  jig-client-wire-world --test drift --test test_client_wire` and
  `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
  jig-store-codecs-world --test drift` write their `src/generated/` code
  and `golden/` bytes, which land with the schemas.
- **16.5, the party's proposals stay:** `goal_decide`
  (`jig-domain-core/src/routing/party_decisions.rs:16`) decides every
  `Proposer::Party` proposal (`proposal_request`, :483-485), and the
  `Propose` route makes them (`routing/propose.rs:119, :213`,
  `tasks::Event::ProposePerson`): it, the tasks child's person proposals
  and the store's `PersonProposal` codec stay, which the Shape's "the goal
  decision" and "`PersonProposal` where jig 03 left it" become. Only what
  in them serves `SetGoal` alone goes.
- **16.5, lines at jig `4fad1fcc8`:** `roles.rs`'s `chat_admit` is at
  :98, the prioritising at :288-335 (`priorities_allowed`,
  `prioritise`); `jig-fake-parties/src/lib.rs`'s `Ask::Close` at :401-406;
  `templates::chat` at `jig-protocol-config/src/templates.rs:51`, read at
  `core.rs:49`. `jig-protocol-config/src/{lib.rs, charters.rs}` name no
  chat item, and leave Files. Ruling 284's `jig-test-service` paths name
  no 16.5 name: its users are those the grep lists.
- **16.5, temper:** temper's users of these (`temper-engine-domain/src/store.rs`,
  `temper-service/src/family/refusals.rs`, the store's `Disposal`
  refusal and their tests) break at temper's repin past jig 16: temper
  04.3 removes them (`temper/04-engine.md`, 04.3). 16.5 changes nothing
  in temper.
- **Done when:** the third grep lists 223 files at jig `4fad1fcc8`.

## Corrections at jig `5b4e287f0` (ruling 300, the re-check of 16.5 after 16.3 and 16.4)

These take precedence over 16.5's Shape and rulings 284 and 299 where
they differ. Refresh any further copy or derived value, and list it in
the commit body.

- **16.5, the creating bounds stay:** `acts::bounds::goal`
  (`jig-domain-core/src/acts/bounds.rs:22`) builds the `Bounds::Create`
  that `me` carries (`documents/small.rs:118`), through `goal_charter`
  (:200), `goal_start` (:223) and `Core::chat_authority`
  (`funding.rs:40`, the most a person's next task can take, read at
  `bounds.rs:69`). They stay, and may take creation's names (for example
  `bounds::create`, `create_authority`); `Act::SetGoal`'s arm
  (`acts/objects.rs:80`), the core's goal creation
  (`routing/parties.rs:134`, `goals.rs:170`) and what is then left with
  no caller go. The Shape's "`chat_authority`" and "the goal bounds
  (`acts/bounds.rs:22-91, 187-300`)" read so.
- **16.5, users that no name finds:** they go with their asks or with
  `Settings.chat`:
  - the people child's `Limits.goals` (`jig-domain-people/src/limits.rs:33`,
    `Prioritise`'s bound, filled at `jig-protocol-config/src/domain.rs:172`)
    and `disposal_capacity` (`jig-domain-core/src/memory.rs:99`,
    `jig-domain-people/src/domain.rs:1037`), with the wire's and the
    store's limit fields derived from the removed records' lists and
    words;
  - the engine frame's `OfferedCharter` (`documents.rs:233`, the
    `Frame.charters` list at :272, its worst case at :411) and
    `FrameProjectRole.chat_budget` (:248), with their writers
    (`documents/small.rs:127, :147`) and tests (`tests.rs:423`,
    `tests/frame_facts.rs:26`, `tests/frame_deployment_role.rs:70`): no
    translation reads them since 16.3;
  - the derived memory pin `jig-protocol-config/src/tests/domain.rs:197`
    (`What::Memory { over: 18_576_833_503 }`), which moves with them.
- **16.5, the note scope:** the wire's `NoteScopeGoal` and the
  contract's `NoteScope::Goal` (`jig-client-domain/src/contract/common.rs:233`)
  become `Task { project, task }`, with their translations at both ends;
  the engine's scopes (`notes::Scope::Goal`, and the people child's and
  the authority's `NoteScope::Goal`) keep their names, as "Choices" says,
  and the component maps between them. Their users are the 30 files
  `git grep -l -E 'NoteScope::Goal\b' -- '*.rs' ':!*/generated/*'`
  lists at jig `5b4e287f0`.
- **16.5, lines at jig `5b4e287f0`:** `GoalProcedure` at `documents.rs:404`
  and `documents/offers.rs:233, :291`; `tests.rs:972` (ruling 299's
  :1054); the tasks child's `Ask::Close` at `routing/tasks.rs:1355`.
  `documents/{inbox_deployment.rs, task_tree.rs}`, `inbox.rs` and
  `reads/indexed.rs` hold none of 16.5's names since 16.4, and leave
  Files. Ruling 299's grep lists 216 files, `crates/jig-client-view/src/tests/shapes.rs`
  and `testing/jig-client-scripted-engine/src/requests/creating.rs` among
  them. The re-check applied both renames at `5b4e287f0`: clippy clean,
  1087 default and 44 fuzzy tests passed in the five suites they reach.

## Corrections after the designer's answer of 2026-10-18 (jig 16.5's question of 01:53:07Z)

jig's `client/domain/contract.md`, 5.3 and 9, `pages.md`, 6.2 and
12.5, and `objects.md`, 11 now say these (jig `ad465272b`). They take
precedence over 16.5's Shape and rulings 287, 299 and 300 where they
differ.

- **16.5, the board's offer stays the project's, renamed:**
  `ActKind::Prioritise` (`jig-client-domain/src/contract/documents.rs:215`)
  becomes `ActKind::Reorder`, the wire's `ActKind` `prioritise`
  (`client-v8.schema:496`) `reorder`, and the engine's `Act::Prioritise`
  (`jig-domain-core/src/acts.rs:56`) `Act::Reorder`; none is removed.
  The project's document offers it as now (`documents/offers.rs:133`),
  made or refused, never proposable. Its users are the 28 files
  `git grep -l -E '\b(ActKind|Act)::Prioritise\b' -- '*.rs' ':!*/generated/*'`
  lists at jig `5b4e287f0` (47 lines). The arms that match
  `people::Ask::Prioritise` or `Ask::Prioritise` (`acts/requests.rs:63`,
  the scripted engine's `requests.rs:664`,
  `tests/core/src/referee/offers.rs:220`) go with that ask. The others
  take the new name, the scripted engine's `requests/priorities.rs:29`
  among them: it judges the board's compound by the project's offer.
- **16.5, its judging:** `roles.rs`'s `priorities_allowed` (:288)
  stays, and may take the act's name (for example `reorder_allowed`).
  It keeps its role check (`acts::role`, :295) and the policy's word on
  the role's Amend; its `goals` and `task_capacity` arguments and the
  check they serve go with `Ask::Prioritise`'s bound (ruling 300's
  `Limits.goals`). `prioritise`, the route to `tasks::Event::Prioritise`,
  goes, as ruling 299 says. `acts/objects.rs:81` and
  `acts/objects/task.rs:103` call it with the project and the role
  alone. `acts/standing.rs:14`'s owner-or-maintainer arm stays under the
  new name: `acts/bounds.rs:322` judges a compound's amendment that
  names `tracked` by it.
- **16.5, the board:** `crates/jig-client-view/src/page/project/moving.rs:13, :34`
  read `ActKind::Reorder` from the project's document, which every
  project section reads (`page/opening.rs`, `Read::Project`). Ruling
  287's "16.2, the arrows' offer" reads with the new name. No goal's
  document is watched, and `TaskLine` is unchanged.
- **16.5, the checks:** in `jig-domain-core`'s
  `task_controls_owner_changes_priorities_and_note_scopes_share_their_judges`
  (`src/tests/acts.rs:469`), the owner's `Act::Reorder` offer stays
  asserted `Allowed` (:502), and the `core.prioritise` lines (:503-507)
  go. jig 16.2's proof, `moving_a_goal_up_reorders_the_board_in_one_compound_request`
  (`tests/client-protocol/tests/client/goals.rs:56`), passes unchanged
  but for the name. The third "Done when" grep then finds no
  `Prioritise`.
- **16.5, Size:** the rename's changed lines count apart from the 150,
  as one count in the report.
- **Later:** the board offering no move that would amend a closing goal
  is a row of `later.md`.

## Corrections after jig 16.5's block of 2026-10-18T02:32:27Z (ruling 304)

pages.md, 7: "**Capacity:** live tasks against their limits, for the
deployment and each project; pending proposals; what was refused for
want of room, recently." The people child's `CapacityAction` has
`Compound` and `Create` (`jig-domain-people/src/capacity.rs:12-13`, from
`Ask::Compound` at :62 and `Ask::Create | Ask::Propose` at :72), which
v8 sends; the wire's and the client's `OwnerPersonCapacityAction` have
neither: the engine's translation refuses them
(`jig-protocol-engine-clients/src/translate/deployment.rs:111-113`,
`Reason::Tag`), and the joint wire calls them unreachable
(`tests/client/src/joint/wire/deployment.rs:198`).

- **16.5, the operations' capacity actions:** `OwnerPersonCapacityAction`
  gains `create` and `compound`, content-free, as the people child's, in
  the wire (`client-v8.schema:791`, then `generated/v8.rs` regenerated)
  and the client (`jig-client-domain/src/contract/owner_deployment.rs:224`).
  They map one to one in each match: the engine's
  `translate/deployment.rs:111`; the link's
  `translate/documents_part6.rs:131`; the view's
  `owner_deployment.rs:202`, with two phrases in
  `owner_deployment_words.rs`, as its other actions' are; the scripted
  engine's `owner_deployment.rs:128`; the joint wire's
  `deployment.rs:196` and its coverage cases (`deployment/tests.rs:398-412`).
  No role rule and no bound changes. Refresh any further match that
  `git grep -l -w OwnerPersonCapacityAction -- '*.rs' ':!*/generated/*'`
  lists (11 files at jig `ad465272b`), and list it in the commit body.
- **16.5, Size:** these lines count apart from the 150, as one count in
  the report, beside the rename's.

## Increments

### 16.1 The conversations section on a task query
Scope: the reference UI's conversations section (pages.md, 4 and 12.5):
its list watched as a task query and its ended chats read below it; each
line the chat's task line; the inbox's reply entries carrying the
conversation's task line (pages.md, 3).
Needs: the session's start: jig 21's reference UI with its state on the
client alone (after jig 13's reference UI on `jig_client`);
jig 22's task line, `tasks{query}` watched and read (22.2, 22.3) and
`me`'s templates (22.1); jig 15's `facts::chat_status` (15.1), `words`
(15.2) and `shapes::CHAT` and `template` (15.3): it starts once 15.1 to 15.3
have landed, beside 15.4 to 15.6 (ruling 248).
Files: the reference UI (`crates/jig-client-view/src/`): its
`Section::Conversations { project }` (at the base
`crates/jig-domain-client/src/pages/conversations.rs`, 515 lines, and
`crates/jig-view/src/sections/`), the inbox's reply rows; the scripted
engine's tasks subject and read (`testing/jig-client-scripted-engine/src/{watches.rs,
reads.rs}`: a query by requester and label, ended rows from its truth);
`tests/client`'s conversation stories; `tests/client-protocol`.
Shape: `jig_client_view`'s conversations section opens `Watch {
subject: Subject::Tasks { query: queries::conversations(party, label,
project), window } }` and reads `Read::Tasks { query: the same with
`phase: Some(PhaseClass::Ended)` and `order: QueryOrder::Ended`,
.. }` a page at a time, each time the person presses the section's
"Older" button; its line shows the chat's
title, project, `facts::chat_status` said by `words`, its unread turns
(`conversation.unread`), what it waits on (`waits_on`, said by
`words`), its live subtree counted (`phases`) and its spend (`budget`);
the inbox's reply entry
holds a `TaskLine` where it held a `ConversationLine`. The frame's
Conversations entry shows only while some project of `me` holds a
template of the chat template's number (`shapes::CHAT`), as pages.md, 2
has the entry for an application that offers chats. The query
functions are the interface sketch's.
Test shape: `tests/client/tests/stories.rs` (scripted):
`the_conversations_section_lists_live_chats_by_last_activity`,
`the_frame_shows_no_conversations_entry_without_a_chat_template` (a
`me` whose projects hold no template 1);
`tests/client-protocol/tests/protocol.rs` (joint), reference test (a new
kind: the reference UI's list through the fake person, over the real
engine's task query):

```rust
#[test]
fn a_person_s_chats_list_live_then_ended_by_last_activity() {
    use skein_fake_person::{Find, Step};
    let (mut story, client) = stories::ready(fixtures::SEED);
    stories::start_chat(&mut story, client, b"Second chat");
    stories::end_chat(&mut story, client, b"Hello");
    story.person(
        client,
        vec![
            Step::Go { address: stories::conversations() },
            Step::See { find: Find::text(b"Second chat"), within: skein_lib::Duration::from_secs(1) },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Older") },
            Step::See { find: Find::text(b"Hello"), within: skein_lib::Duration::from_secs(1) },
        ],
    );
    stories::finish(&mut story);
    assert_eq!(story.front().task_queries_read(), 1);
}
```

(`stories::ready` and `stories::finish` are at
`tests/client-protocol/src/stories.rs:17, 24` at the base, followed to
jig 12's file; `start_chat`, `end_chat`, `conversations` and the front's
`task_queries_read` counter are added there and in the front; the
button's name is the reference UI's own.) Ceilings: world under 0.5 s,
joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_chats_list_live_then_ended_by_last_activity)'` once:
the live chat lists first, and the ended one appears below it after one
read of the ended group from the store's index. Bound: 1 s.
Don't: filter or order lines in the reference UI; write the chat label
as a number.
Size: at most 250 lines.
Removes: the reference UI's uses of `Subject::Conversations`,
`Read::Conversations`, `ConversationLine` and `ConversationPage` (the
records go in 16.4).

### 16.2 The goals board on a task query
Scope: a project's goals (pages.md, 6.2 and 12.5): the board watched as
a task query by priority, ended goals read below it; moving a goal as one
compound request of priority-only amendments; the project's overview
watching the goals' query for its first rows (pages.md, 12.5: "a
project's overview | `inbox`, filtered by the project; the goals' query,
its first rows"); and a card whose act is refused where the refusal says
a proposal could be accepted offers to propose it, with a confirmation
of its own (objects.md, 9; view.md, 18.4: "where the refusal says a
proposal could be accepted, offers to propose it, with a confirmation of
its own").
Needs: 16.1's query helpers in the reference UI.
Files: the reference UI's project goals section (at the base
`crates/jig-domain-client/src/pages/project/goals.rs`, 335 lines, and
`owner_goals.rs`, 77), its move-up and move-down acts, the project's
overview section, its refusal card (`RefusalAuthority { proposable }`,
which jig 14.5 leaves as it is) and its confirmation; the scripted
engine's compound of priority-only amendments (in place of its
`Prioritise`); `tests/client`'s goal stories; `tests/client-protocol`.
Shape: the section watches `queries::goals(project)` and reads
`queries::ended_goals(project, goal_label)`; a line shows the goal's
chip and phase in words, its progress (`phases`), its spend against its
budget, its deadline and what is held below; moving a goal sends `Ask::Compound
{ project, operations }` of `Operation::Amend { revision: None, change:
Amendment { tracked: Some(Tracking::Tracked { priority: p }), .. } }`
(ruling 260), one per goal whose priority
changes, and the board says what priority does (pages.md, 6.2). The
overview watches `queries::goals(project)` with a window of its first
rows, under its own handle. A refusal for authority whose `proposable`
is `Some(true)` offers "Propose instead", whose confirmation says to
whom from the offer (`me`'s creating bounds, or the task document's
amending bounds: `Beyond::Proposed { to }`; jig's contract.md, 4: "Who
would accept a proposal is the offer's to say") and sends
`Ask::Propose` of the refused act (a batch for a refused `Create`, an
amendment for a refused `Amend`); `Some(false)` says that nobody may
accept it, and `None` offers nothing.
Test shape: `tests/client/tests/stories.rs`:
`moving_a_goal_up_sends_one_compound_request_of_priorities`,
`the_project_overview_shows_the_board_s_first_goals`,
`a_refusal_a_proposal_could_answer_offers_to_propose_with_its_own_confirmation`,
`a_refusal_nobody_could_accept_offers_no_proposal`;
`tests/client-protocol/tests/protocol.rs`:
`moving_a_goal_up_reorders_the_board_in_one_compound_request` (two
goals the first owner set, by `shapes::set_goal`; the fake person moves
the second up; the engine admits one request; the board's next delivery
lists it first). Ceilings as 16.1.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(moving_a_goal_up_reorders_the_board_in_one_compound_request)'`
once: one keyed request is made (`requests_made()` 1), and the board's
first line becomes the moved goal. Bound: 1 s.
Don't: send one request per goal; name a revision in a priority-only
amendment.
Size: at most 300 lines.
Removes: the reference UI's uses of `Subject::Goals`, `Read::Goals`,
`GoalLine`, `GoalPage`, `Goals` and `Ask::Prioritise`.

### 16.3 The chat and goal acts through `shapes`
Scope: starting a chat, closing it and setting a goal in the reference
UI through `shapes` (pages.md, 4 and 6.2; client.md, 19); `me`'s
transitional `charters` and `chat_budget` go.
Needs: 16.2.
Files: the reference UI's start-a-chat form, close dialog and set-goal
form (at the base in `crates/jig-domain-client/src/{acts.rs, intents.rs,
pages/}` and `crates/jig-view/src/`); the v8 schema (`Me.charters`,
`MeProject.chat_budget`, `Charter`, `OfferedCharters` go); the
component's `translate/owner.rs`; the contract's `me.rs`; the link's
documents' translation; the scripted engine (`testing/jig-client-scripted-engine/src/{requests.rs,
payload.rs, truth.rs}`: `Create` from the chat and goal templates,
`Propose`, the compound closing a chat, each giving the outcome its
`StartChat`, `SetGoal` and `Close` gave); `tests/client-protocol`'s
front, whose chat counter counts tasks the engine made under the chat
label (it counted `StartChat`); `tests/client`'s stories.
Shape: the start form gives `shapes::chat(me, project, words)`; the
close dialog lists each live delegate with its fate and gives
`shapes::close_chat(me, project, chat, &delegates)`, each a
`shapes::Delegate { task, fate }`; the goal form
builds a `shapes::GoalDraft`, shows `shapes::fall` against the
project's `me` creating bounds (`MeProject.creating`) as it is written (made; proposed, and to
whom; refused, and why; cannot be set), and sends what
`shapes::set_goal` gives; a `NotOffered` sends nothing and says why.
Test shape: `tests/client/tests/stories.rs`: the existing
`signing_in_and_starting_a_chat` and
`a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said` keep
their names and expected values (the scripted engine now models
`Create` and `Propose`); add
`closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person`
(view.md, 18.10: the chat has two live delegates; the close dialog
cancels one and moves the other; one compound request);
`tests/client-protocol/tests/protocol.rs`: the existing
`a_person_signs_in_by_the_printed_link_and_starts_a_chat` keeps its
assertions (`chats_made()` 1). Ceilings as 16.1.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(=a_person_signs_in_by_the_printed_link_and_starts_a_chat)'` once:
the person's chat is made by a `Create` from the chat template and the
real engine counts one chat. Bound: 1 s.
Don't: let the reference UI fill a template itself; keep a fallback to
`StartChat`, `Close` or `SetGoal`.
Size: at most 300 lines.
Removes: the reference UI's uses of `Ask::StartChat`, `Ask::Close`,
`Ask::SetGoal`, `Disposal`, `GoalExecutor`, `Bounds::Goal`; `Me.charters`,
`MeProject.chat_budget`, `Charter`, `OfferedCharters` on the wire and in
the contract.

### 16.4 The lists leave the wire and the engine
Scope: the conversation and goal lists' records, translations, contract
types and models, and the engine's list code (protocol/client.md, 3;
contract.md, 1; findings/engine-parties-and-tasks.md, 2).
Needs: 16.1 and 16.2 (nothing on the client reads a list record).
Files: the v8 schema; the component's `translate/{reads.rs, watches.rs,
pages.rs, pages/*.rs, inbox.rs}`; the contract and the link's
translations; the scripted engine; `testing/jig-client-test-records`;
`crates/jig-domain-core/src/{documents.rs (Subject, Watched),
documents/{project_tasks.rs, task_tree.rs, inbox_deployment.rs},
reads/{vocabulary.rs, cursors.rs, indexed.rs, rows.rs,
rows/positions.rs}, inbox.rs, routing/watch_marks.rs, indexes.rs,
forget.rs}` (`forget.rs` is jig 08.3's: its erase of tags 14 and 15
goes); `crates/jig-domain-tasks/src/{index.rs, closing.rs}` (the
Conversation and Goal keys and the `Goal` row's save);
`crates/jig-protocol-store/src/{keys/tasks.rs, codec/tasks/index.rs,
reads/indexed.rs, reads/joins.rs, intervals.rs}`; their tests
(`tests/core` reads and projections,
`crates/jig-domain-core/src/tests/conversation_cursor.rs`, the store's
`tests/reads/indexed.rs` and `tests/intervals.rs`). Where step 2 already
removed a piece, `rg -l -w -e ConversationNewest -e ConversationOldest -e
'Stored::Goal' -e 'Scope::Conversations' -e 'Scope::Goals' -e
'Watched::Conversations' -e 'Watched::Goals' -e 'Read::Conversations' -e
'Read::Goals' crates/jig-domain-core/src crates/jig-domain-tasks/src
crates/jig-protocol-store/src` at the increment's start lists no file
for it, and nothing is done for it. Shared hotspots:
`crates/jig-domain-core/src/{documents.rs, routing/mod.rs}`,
`crates/jig-protocol-store/src/keys/`.
Shape: the records that go, or are renamed in place:

| Record | Fate |
|---|---|
| `ReadConversations`, `ReadGoals`, `SubjectGoals`, `Read.conversations`, `Read.goals`, `Subject.conversations`, `Subject.goals` | gone: task queries |
| `ConversationLine`, `ConversationPage`, `Conversations`, `GoalLine`, `GoalPage`, `Goals` | gone: `TaskLine`, `TaskPage`, `Tasks` |
| `ReplyPosition { chat, through }` | renamed `ReadPosition { task, through }` |
| `OwnerInboxObjectReply { message }` | renamed `OwnerInboxObjectTurn { message }` |
| `Place`, `Draft`, `JigRecord`, `Address`, `AddressInbox`, `AddressConversations`, `AddressTask`, `AddressProject`, `ProjectSection`, `TaskSection` | gone in jig 13 (the reference UI's records leave the client's store); nothing here |

and in the engine: `documents::{Subject, Watched}::{Goals,
Conversations}` and `Tasks { project, filter }` where jig 05 left it,
their documents, `reads::Read::{Conversations, Goals}` and
`reads::Scope::{Conversations, Goals}` with `reads/cursors.rs`, the
conversations known by their charter (if jig 05 left it); the store's
tasks tags 14 (`ConversationNewest`), 15 (`ConversationOldest`) and 16
(`Goal`), as jig 08 left them for this increment (its file, 8.2 and
8.4): their keys, values and codecs, their saves (the core's
`indexes.rs` conversation rows, the tasks child's `Goal` row at a
goal's end), their readers (the conversation and goal parts of
`reads/rows/positions.rs` and of both `reads/indexed.rs`, the core's and
the store's) and the forget's erase of tags 14 and 15; tasks v3 and
people v3 edited in place. Tag 13, the unread results' index by
`(party, position)`, stays: it names no shape of work.
Test shape: tests of what goes are removed with it, or moved to the task
query's tests where they check a behaviour the query keeps (README,
Rules); no new test. Ceilings unchanged.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_person_s_chats_list_live_then_ended_by_last_activity) |
test(moving_a_goal_up_reorders_the_board_in_one_compound_request)'` once:
both lists still read through task queries with the list code gone.
Bound: 2 s. And `git grep -n -E
'^(record|enum) (ConversationLine|ConversationPage|Conversations|GoalLine|GoalPage|Goals|ReadConversations|ReadGoals|SubjectGoals|ReplyPosition|OwnerInboxObjectReply)\b'
-- crates/jig-client-protocol-wire/schema`, once in the worktree,
prints nothing.
Don't: keep a list record for another user; add an index.
Size: at most 120 lines added.
Removes: the table's records and the engine's list code above, tags
14 to 16 among it.

### 16.5 The shape asks leave the wire and the engine
Scope: the shape asks' records, translations, contract types and
models, and the engine's chat and goal asks, routes and settings
(findings/engine-parties-and-tasks.md, 2; engine-agents-store-config.md,
3).
Needs: 16.3 (nothing on the client sends a shape ask).
Files: the users are what this session's third "Done when" grep lists
at the increment's start; at the base it lists 207 files, 100 of them
production, by crate: the client crates (`jig-domain-client` 25 and
`jig-view` 12, now the contract, `jig-client` and the reference UI;
the link 8; the component 6: `translate/{asks.rs, answers.rs,
refusals.rs, owner.rs}`), the scripted engine 4, the test records 2;
`crates/jig-domain-core/src/` 21 (`acts.rs`, `acts/{bounds.rs,
objects.rs, objects/decisions.rs, objects/task.rs, requests.rs,
standing.rs}`, `chats.rs`, `goals.rs`, `documents.rs`,
`documents/{inbox_deployment.rs, offers.rs, task_tree.rs}`, `inbox.rs`,
`person_task.rs`, `reads/{indexed.rs, vocabulary.rs}`, `roles.rs`,
`routing/{parties.rs, party_controls.rs, people.rs}`; 16.4 takes the
lists' part); `crates/jig-domain-people/src/` 7 (`boundary.rs`,
`capacity.rs`, `domain.rs`, `lib.rs`, `limits.rs`, `refusals.rs`,
`words.rs`); `crates/jig-domain-tasks/src/` 4 (`boundary.rs`, `chat.rs`,
`domain.rs`, `lib.rs`); `crates/jig-protocol-store/src/codec/people/{asks.rs,
outcomes/facts.rs}`; the test kits that translate people refusals or
make chats: `testing/jig-fake-parties/src/lib.rs` (its `Ask::Close` and
chat asks, at the base lib.rs:54 and 396, become `Create` from the chat
template and the compound close), `testing/jig-test-connector/src/application_core.rs`,
`testing/jig-test-domain/src/{admission.rs, application_facts.rs}`,
`testing/jig-test-service/src/family/facts.rs`,
`testing/jig-test-store/src/records/facts.rs`,
`testing/jig-client-test-view/src/{connector_values.rs,
connector_values_refusal.rs}`; and `crates/jig-domain-core/src/lib.rs`
(`Settings.chat`, `close_grace`), `crates/jig-protocol-config/src/{lib.rs,
charters.rs, core.rs}` (what jig 09 left of `chats`),
`crates/jig-protocol-store/src/codec/tasks/proposals.rs` (`PersonProposal`,
if jig 03 left it); `crates/jig-protocol-config/src/templates.rs`
(jig 09.1's transitional `templates::chat` and the check it serves: a
template 1 naming no charter or no grace refused, `What::Refers` at
`templates`; its test moves or goes). Tests: `tests/core`'s `stories/closing_chat.rs` and
`procedure_goals.rs`, `tests/people`, `tests/engine-clients/tests/closing.rs`,
`crates/jig-domain-tasks/src/tests/revisions/chat.rs`, and the 107 test
files the grep lists. Shared hotspots:
`crates/jig-domain-core/src/{lib.rs, routing/mod.rs}`,
`crates/jig-domain-people/src/boundary.rs`, `crates/jig-protocol-store/src/codec/`.
Shape: the records that go, or are renamed in place:

| Record | Fate |
|---|---|
| `StartChat`, `Close`, `Disposal`, `DisposalCancel`, `DisposalTakeOver`, `SetGoal`, `GoalExecutor`, `GoalExecutorCharter`, `GoalExecutorProcedure`, `Prioritise`, `Priority` | gone: `Create`, `Propose` and `Compound`, through `shapes` |
| `NoteScopeGoal { project, goal }` | renamed `NoteScopeTask { project, task }` |
| `OwnerOutcomeClosing`, `OwnerOutcomeGoalStarted`, `OwnerOutcomeGoalProposed`, `OwnerOutcomePrioritised`, `MadeChat`, `MadeGoal` | gone: `OwnerOutcomeCompound`, `OwnerOutcomeCreated`, `OwnerOutcomeProposed` |
| `DisposalIssue`, `RefusalDisposal`, `LimitConversationWords` | gone: a compound's refusal at its place; `LimitWords` |
| `OwnerHistoryChangePrioritised { priority }` | gone: an amendment's history line |
| `BoundsGoal`, `GoalProcedure` | renamed `BoundsCreate`, `CreateProcedure` in jig 22.1; nothing here |
| `ActKind`'s `close`, `set_goal` | gone: `cancel`, `move_task`, `amend` offered, and `create` bounded in `me` |
| `ActKind`'s `prioritise` | renamed `reorder`: the project's offer of a compound of priority-only amendments (the designer's answer of 2026-10-18, above) |
| `OwnerPersonCapacityAction`'s `close`, `chat`, `goal`, `priority` | gone with their asks; `create` and `compound` added, content-free, the people child's capacity actions of the operations (jig 03's and jig 04's), on the wire and the client (ruling 304, above) |

and in the engine: `people::Ask::{StartChat, SetGoal, Prioritise,
Close}`, `people::{Disposal, Keep, GoalExecutor}`,
`people::Outcome::{Closing, GoalStarted, GoalProposed, Prioritised,
Started}`, `people::Refusal::Disposal` and `DisposalIssue`, and each
match arm naming them; the core's chat and goal creation
(`routing/parties.rs:134-242` at the base), `chat_admit` and the
prioritising (`roles.rs:75-114, 228-272`), `chat_authority`
(`funding.rs:40-54`), the goal bounds (`acts/bounds.rs:22-91, 187-300`),
the goal decision (`routing/party_decisions.rs:16-97`), `chats.rs` and
`goals.rs` where jig 03 and jig 04 left them, `acts::Act::{Close,
SetGoal, Prioritise}`, the set-goal offer's judging (`acts::Bounds::Goal`
is renamed `Bounds::Create`, the creating bounds `me` carries, its
fields kept), the people child's capacity
actions of the removed asks, `Settings.chat` and `Settings.close_grace`
with their readers in `jig-domain-core` (9 at jig 09's count, its file,
"Transitional until jig 16") and jig 09.1's `templates::chat`, which
filled them, with its template-1 check; the tasks child's chat close and
`PersonProposal` where jig 03 left it; the store's codecs of these.
Test shape: removed tests' behaviours that the operations keep move to
them (closing a chat becomes `shapes::close_chat`'s compound in
`tests/core`'s stories, kept under their names and expected values);
others are removed. Ceilings unchanged.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(=a_person_signs_in_by_the_printed_link_and_starts_a_chat) |
test(closing_a_chat_by_shapes_moves_its_delegate_then_cancels_it_after_its_grace)'`
once: chats are made and closed through the operations with the shape
code gone. Bound: 2 s. And the README's "Done when" 5 grep, once in the
worktree, prints nothing.
Don't: keep a shape ask "for compatibility"; translate a removed record
into an operation on the engine's side.
Size: at most 150 lines added.
Removes: the table's records and the engine's shape asks above,
`Settings.chat`, `Settings.close_grace` and `templates::chat` among them.

## Interface sketches

Checked at the plan's base: the queries compiled in a scratch crate
with jig 22's sketch types (`cargo clippy -p w7-sketch
--all-targets -- -D warnings` clean); checked again at the ready check.
The reference UI reaches the contract's types through `jig_client`'s
re-export of them.

```rust
// crates/jig-client-view/src/queries.rs: the reference UI's lists (pages.md, 12.5).
/// The conversations section's live query: the party's tasks of the chat template's label, by last activity.
pub const fn conversations(party: u64, chat_label: u32, project: Option<u32>) -> TaskQuery {
    TaskQuery {
        project,
        requester: Some(OwnerRequester::Person(party)),
        root: None,
        parent: None,
        label: Some(chat_label),
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Activity,
    }
}
/// The goals section's live query: the project's tracked tasks, by priority.
pub const fn goals(project: u32) -> TaskQuery;
/// The goals section's ended read: the project's ended tasks of the goal template's label, by when they ended.
pub const fn ended_goals(project: u32, goal_label: u32) -> TaskQuery;
```


## Questions to raise when reached

- None the design leaves open: the lists' queries are pages.md, 12.5's
  table, and the forms' judgements contract.md, 5.3's.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-16\.' | sort -u | wc -l` prints 5.
- The README's "Done when" 5 grep prints nothing.
- `git -C ~/src/rust/jig grep -l -E
  '\b(StartChat|SetGoal|Prioritise|GoalExecutor|Disposal|DisposalIssue|TakeOver|ConversationLine|GoalLine|OfferedCharters|BoundsGoal|GoalProcedure|ReplyPosition|MadeChat|MadeGoal|RefusalDisposal|LimitConversationWords|ReadConversations|ReadGoals|SubjectGoals|NoteScopeGoal)\b'
  origin/main -- '*.rs' ':!*/generated/*'` prints nothing (207 files at
  the base).
