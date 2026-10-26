# Session 22: the wire's documents: `me`, the task line and queries, offers and a decision's effect

The documents half of jig's client wire collapsed, after jig 14's
requests: `me` replaces `Frame` and serves each project's merged
templates and its creating bounds; the task line and task queries,
watched and read, replace the project task filter; and each object's
document offers its acts in the operations' names, with what a decision
will do as data. As in jig 14, each record lands with both ends in one
increment (the schema at v8, edited in place; the clients' component's
translation; `jig-client-domain`'s contract; the link's translation;
their users), each run as a story against the real engine. The
conversation and goal lists stay until jig 16, which moves the
reference UI onto these task queries.

## Read first

- jig's `docs/design/client/domain/contract.md`, sections 1, 3.6, 5 (5.1
  to 5.4), 6, 6.1, 7.1 and 7.2; `docs/design/protocol/client.md`,
  sections 3, 6 and 7; `docs/design/client/domain/client.md`, sections
  7 and 8.1; `docs/design/domain/engine.md`, 11.5;
  `docs/design/client/domain/objects.md`, sections 2, 4, 5 and 11.
- jig 14's session file (`jig/14-wire.md`): its Choices and its sketches
  (`TaskFlags`, the renamed operations) hold here.
- The plan's `decisions.md`, 3 and 9; `findings/wire-and-client.md`,
  sections 1 and 5f; `findings/engine-parties-and-tasks.md`, section 1
  ("`me` and task queries").
- skein's `programming-model.md`, sections 4.4 and 6.3;
  `testing-strategy.md`, sections 2.1, 2.5 and 7; skein's
  `docs/design/codec.md`, section 3.

## What the design asks

- **`me`** (contract.md, 5.1): the party, their projects with role and
  pool, needs counted, updates up to a limit, results' read position,
  "each project's merged templates (3.6); what they may create in each
  project, as the bounds a draft is compared with (5.3)", the deployment
  role, the retention.
- **The task line and task queries** (5.1, 6.1, 7.1): one record for
  each list of tasks; a query by a fixed set of keys and orders, read
  live then ended in two groups with their own cursors, watched as
  `tasks{query}`; what a task waits on "a party (who), for a question, a
  proposal or a person task in its tree; its delegates; news; or
  nothing".
- **Offers and a decision's effect** (5.3, 5.4; engine.md, 11.5): each
  object's document says, per act, made, proposable and to whom,
  refused, or bounded; and "what accepting a proposal grants and costs,
  and the pool that funds it, after; ... what lifting its hold does ...;
  what ends ...; the reservation that moves to the party's pool".

## Choices this plan makes

- **jig 14's Choices hold** (v8 edited in place; `move_task` for `Move`'s
  variant; new code in new modules: here the contract's `me.rs`,
  `queries.rs` and `effects.rs`, its `documents.rs` being 1,161 lines at
  the base).
- **`me`'s projects cross as pieces,** each holding its templates as
  pieces, bounded by the client's document bytes; `list 7` goes
  (findings/wire-and-client.md, 5f).
- **`Me.charters` and `MeProject.chat_budget` are transitional:** the
  reference UI's chat and goal forms read them until jig 16.3 moves those
  forms onto `shapes`, and jig 16.3 removes both fields.
- **The engine keeps the name `Frame`** (jig 05's choice); the wire and
  the contract call it `Me`.
- **Accepting's effect stays the proposal's `funding`**
  (`OwnerProposal.funding`); `Effect` holds releasing, cancelling (the
  legacy `Impact`, reused, not copied, less `effects_pending`,
  `effects_settling` and `takeover_budget`: the designer's answer of
  2026-10-17T11:23Z) and moving.
- **A release's effect on a hold of an uncertain effect is `Uncertain`:**
  made again or not, as the release's `again` says; the card offers both
  (objects.md, 5).

## Rules that bind it

- **No type beside an existing one:** a record that replaces another
  replaces it in place, with each of its users in the same increment
  (code-organization.md, 6).
- **The meeting point:** each increment runs its Proof against the real
  engine, in `tests/client-protocol`, never only against
  `jig-client-scripted-engine` (README, Rules).
- **jig never names the repository downstream of jig:**
  code-organization.md, 6's grep for that name prints nothing at each
  increment's end.

## Re-check at jig `24f3bcc55` (ruling 273, after jig 14 and 21 landed)

These take precedence over the Shapes, Test shapes and sketches below
where they differ.

- **22.1:** `crates/jig-client-view/src/limits/ui.rs:103-109` is gone
  (21.4 removed the Ui's frame, roles and charters worst case): drop that
  item; the `document_bytes` cite is `ui.rs:26`; `frame/me.rs:35-38` is
  now `:30-31` and `:35-40`. Files gain the users 21.4 added:
  `crates/jig-client-view/src/ui/{borrowed,client}.rs`,
  `examples/ops/client-domain/src/{me,tests,tests_support}.rs`,
  `testing/jig-client-test-domain/src/{first/me,second}.rs`, and the view
  tests `bridge.rs`, `domain/native.rs`, `domain/pending.rs`,
  `fixtures/client.rs`. The core world has one project (`projects: 1`,
  `tests/core/src/world.rs:82`): the test and Proof read "project 1's
  template 2 holds the override's budget, template 1 the configuration's"
  in place of "project 2 the default". The sketch's `contract/me.rs`
  defines `MeParty`, `FrameParty`'s four fields.
- **22.2:** the `Amend` story's "(at `b6a63f8c1` it has neither)" is
  stale: both landed. Its ask is `Ask::Amend { project, task, revision:
  None, change: Box::new(Amendment { .. }), reason }` (boxed, ruling
  261), with all seven `Amendment` fields named (no `Default`); its task
  is created tracked (`create_tasks()` creates both members with
  `tracked: None`: give the story a member with `tracked: Some(..)`).
- **22.4:** `Offered` gains `effect: Option<Effect>` in place, in
  `contract/documents.rs:343`; `contract/effects.rs` holds `ReleaseEffect`
  and `Effect` alone (a second `Offered` there is ambiguous under the glob
  re-exports, E0659). Files gain
  `testing/jig-client-scripted-engine/src/tests/compound.rs:16`
  (`ActKind::AmendRelease`, which 22.4 removes) and the scripted engine's
  `payload.rs:573` (`fields!(Offered { act, admission })` gains `effect`,
  with `Payload` for `Effect` and `ReleaseEffect`). Each contract
  `Offered { .. }` literal (in 53 files at `24f3bcc55`) gains
  `effect: None` unless its test reads an effect. Moved cites:
  `phrases/jig.rs` 636-638, 1473-1474, 2326-2327, 2802-2803;
  `english.rs` 1398-1405; `client-v8.schema:625` (`PhaseCounts`);
  `waits.rs:12`.
- **Unchanged:** ruling 262's `OwnerExecutor` needs nothing here
  (`ExecutorKind::Person` covers a role executor).

The re-check compiled the corrected sketches under `clippy --workspace
--all-targets -D warnings` with `effect: None` in each literal, and ran
the packages depending on `jig-client-domain`: 2,335 of 2,335 focused (in
groups), 95 of 95 fuzzy. Refresh any further user of a moved or renamed
item, and list it in the commit body.

## Increments

Each increment regenerates the codec once from the edited schema:
`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world
--test drift`, then runs the drift test without the variable. Schema,
generated code and goldens count no lines (code-organization.md, 7);
each increment is a codec increment, landing within 2.5 times its Size
without asking.

### 22.1 `me`
Scope: `me` replacing `Frame` (contract.md, 5.1, 3.6, 5.3, 7.1): merged
templates and creating bounds per project; `Read::me`, `Subject::me`.
Needs: the session's start: jig 14 (the wire's requests, its v8 edits);
jig 05's `me` in the core (`Frame`, its
name kept in the engine) with, per project, jig 09's merged templates
and jig 05.4's `FrameProjectRole.creating: Option<acts::Bounds>`, the
acts judge's first bound and ceiling for creating there (`None` when
the party may create nothing).
Files: `crates/jig-domain-core/src/{acts.rs (a `Creating` beside
`Bounds`), documents.rs (`FrameProjectRole.creating`), documents/small.rs
(`frame`)}`; the v8 schema; the component's
`translate/{owner.rs (frame, 128-230 at the base), watches.rs, reads.rs,
objects.rs, pages.rs}`;
`crates/jig-client-domain/src/contract/me.rs` (new; `Frame`,
`FrameParty`, `ProjectRole` leave `documents.rs`); the link's
`translate/{documents_part0.rs (frame_fields, 132-150), reads.rs,
watches.rs}`; users: `jig-client`'s own `me` watch (the party
confirmed, the retention), the reference UI's frame and its chat and
goal forms (reading the transitional fields), the ops view's frame
users, `testing/jig-client-scripted-engine/src/` (its frame document),
`testing/jig-client-test-records`, `tests/client`'s and
`tests/client-protocol`'s frame users (`Delivery::Frame`,
`ReadValue::Frame`); and, at jig `b6a63f8c1`: the `me` watch in
`crates/jig-client-domain/src/{link.rs (13, 63, 73), requests.rs (29)}`
and its contract's `reads.rs` (47, 166) and `watches.rs` (11, 38); the
link's `translate/documents.rs` (61, 67) and `documents_part2.rs` (77, 91,
114: `goal_procedure_fields`, `Bounds::Goal`); the component's
`translate/task.rs` (104-115: `Bounds::Goal`); `OwnerProjectPolicy`
(`contract/owner_project.rs`: 16, 44), built by the component's
`translate/objects/projects.rs` (96, 175) and decoded by the link's
`translate/documents_part7.rs` (147-167); the core's tests
`crates/jig-domain-core/src/tests.rs` (417-423) and
`tests/frame_facts.rs` (166, 205: `owner_creating`), and
`tests/core/tests/task_queries.rs` (100: `creating` read as
`Bounds::Goal`); the wire's tests `src/tests/fixtures.rs` (209, 605, 612:
the `frame_projects` limit that `list 7` removes); the reference UI's
`crates/jig-client-view/src/{limits/ui.rs (103-109: the worst case over
`ProjectRole` and `OfferedCharters`), frame/me.rs (35-38)}`; the
callers of `document_worst_case` (core `acts.rs:164`, `documents.rs:339,
453`, `documents/deployment.rs:450`, `tests.rs:509-514, 800`,
`tests/deployment_connectors.rs:113`, `tests/inbox_selection.rs:440`) and
the literals of `DocumentLimits` that take `roles` (core `tests.rs:127`
and `documents/offers/tests.rs:60`; `jig-protocol-config`'s
`profile/small.rs:157`, `profile/standard.rs:99` (its `roles: 8`, small's
being 4) and `overrides.rs:392`, which destructures it;
`jig-protocol-store`'s `src/tests/fixtures.rs:596`;
`tests/core/src/world.rs:122`; `tests/store/src/root.rs:107`) (ruling
249). Downstream, the repository downstream of jig follows at its
repin, in its client domain's `src/tests/fixtures.rs` (2, 76, 77, 246),
its client view's `src/tests/fixtures.rs` (171, 200, 209, 225) and
`src/forge/page.rs` (26), its terminal shell's `src/tests/fixtures.rs`
(297, 306, 608; 306 is 22.2's `Subject::Tasks`), `src/client/tests.rs`
(17) and `src/client/tests/fixtures.rs` (25), and its own
`tests/client/src/front/tests.rs` (63) and `tests/one-process/src/world.rs`
(358-359) (ruling 249).
Shape: wire: `MeParty` (was `FrameParty`); `Template { number: u32,
name: DocumentText, executor: TemplateExecutor,
contract: TaskContractDocument, shallower: u32, budget, deadline,
tracked: option u32, flags: TaskFlags, label: u32, grace: option
duration }` (its executor, budget and deadline jig 14.2's
`TemplateExecutor`, `TemplateBudget`, `TemplateDeadline`); the policy's
document carries its overrides: `OwnerProjectPolicy` gains `templates:
list 16 TemplateOverride` (jig 14.2's record; contract.md, 5.1: "a
project's, its policy (its templates' overrides among it)"), from jig
09.2's `documents::Policy.templates`; `BoundsCreate` and `CreateProcedure` (were `BoundsGoal` and
`GoalProcedure`, their fields kept: `pool_left`, `charters`,
`procedures`, `latest`, `budget`, `maximum_budget`, `maximum_deadline`;
and `BoundsCreate` gains `authority: OwnerAuthorityDocument`, as
`BoundsAmend.authority` (`client-v8.schema:496`), by the designer's
answer of 2026-10-17T17:57Z: contract.md, 5.3, "the authority they may
give it, from their role and their pool: whole", which jig 15.3's
`shapes` fills a `Create` from),
`Bounds.create` (was `goal`); `CreatingBounds { within: BoundsCreate,
beyond: Beyond }`; `MeProject { project, name, role, pool, templates:
bytes 16777216, creating: option CreatingBounds, chat_budget: u64 }`
(was `ProjectRole`); `Me { party: MeParty, projects: bytes 16777216,
needs, needs_total, updates, results_seen, charters: bytes 16777216,
cut, retention, deployment_role }` (was `Frame`); `Read.me`,
`Subject.me`, `ReadValue.me: Me`, `DeliveryValue.me: Me`. Contract: the
sketch's `Me`, `MeParty`, `MeProject`, `Template`, `CreateBounds`,
`CreateProcedure`, `Creating` (the contract's `Bounds::Goal` becomes
`Bounds::Create`), and `OwnerProjectPolicy.templates:
List<TemplateOverride>`.
Core: `FrameProjectRole.creating` becomes `Option<acts::Creating>`,
`pub struct Creating { pub within: Bounds, pub beyond: Beyond }`, both
from the one judgement (`Admission::Bounded { within, beyond }`) jig 05.4
reads; the core's `Bounds::Goal` keeps its name until jig 16.5 renames
it with the goal form's code, and gains `authority:
Box<documents::AuthorityLine>`: in `acts/bounds.rs`'s `goal`,
`core.chat_authority(project, party, held, limits.tasks.tree_tasks)`
(`funding.rs:40-57`) through `crate::task_authority` and
`documents::act_authority`, refused as
`Limit { which: Authority }` where it does not fit, as `amend` builds
its own (`acts/bounds.rs:142-156`); counted in `offers.rs`'s heap as
`Bounds::Amend`'s authority is (`documents/offers.rs:290-297`), and in
the frame's worst case. The contract's `CreateBounds.authority:
Box<OwnerAuthorityDocument>`. Test: `src/tests/frame_facts.rs`'s
`me_carries_a_members_creating_bounds_per_project` asserts the bound's
authority is `chat_authority`'s, once, and `owner_creating` (:166) gives
one. Should 22.1 land before this reaches it, a step before jig 15.3
adds it. The schema's
header (`client-v8.schema:2`) names `Me.projects` and `MeProject.templates`
bounded by `jig_client_view::Limits::document_bytes`
(`crates/jig-client-view/src/limits/ui.rs:28`), as it names every bound,
not by `jig_client_domain::Limits` (ruling 249). The frame's worst
case (`documents.rs:430-436`) counts a project's `creating` as its
charters and procedures alone: it counts no `Beyond`, whose `Proposed {
to: Recipient::Roles }` holds up to `authority.roles` role numbers
(`acts/bounds.rs:13`), a bound that `DocumentLimits` (`documents.rs:90-97`)
does not carry. So `DocumentLimits` gains `roles: u32` (the configured
`authority.roles`), and the worst case adds, for each of `lines`
projects, a `Beyond` with `List::<u32>::worst_case(limits.roles)`
(ruling 249). `document_worst_case(&DocumentLimits)` keeps its
signature: its ten callers call it as they do, and the seven literals
that build a `DocumentLimits` take the field (both in Files) (ruling
249).
Test shape: the component's tests (fixtures
`tests/engine-clients/src/fixtures.rs:61-80`, `frame_subject` and
`frame_document`, renamed in place): `a_me_document_carries_each_project_s_templates_and_bounds`;
the link's: `me_s_projects_past_the_document_bytes_are_refused_as_too_large`;
the component's `a_project_document_carries_its_policy_s_template_overrides`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`me_serves_a_template_merged_with_its_project_s_override` (the core
world's configuration; the first owner overrides template 2's budget in
project 1 by jig 14.2's `ChangePolicy`; the delivered `me` holds the
override, project 2 the default, and project 1's document lists the
override).
Limits: the core's from `jig_core_world::world::limits()`
(`tests/core/src/world.rs:17`), the configuration's templates from
jig 09's default configuration. Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(me_serves_a_template_merged_with_its_project_s_override)'` once:
the client's `me` handle holds project 1's template 2 with the override's
budget and project 2's with the configuration's. Bound: 1 s.
Don't: expand or check a template in the engine; keep `list 7`; read
the transitional `charters` or `chat_budget` in new code.
Size: at most 300 lines.
Removes: `Frame`, `FrameParty`, `ProjectRole`, `Read::frame`,
`Subject::frame` on the wire and in the contract; `BoundsGoal` and
`GoalProcedure` by name.

### 22.2 The task line, the task's fields, and `tasks{query}` watched
Scope: the task line (contract.md, 5.1), a task query's keys and orders
(6.1), the watched subject `tasks{query}` (7.1), and the task document's
fields of 3.2 (label, flags, last activity, what it waits on).
Needs: the session's start: jig 05's `documents::TaskLine`,
`TaskQuery`, `PhaseClass`, `QueryOrder`, `ConversationRead`,
`waits::WaitsOn` (whom, how many entries, and the oldest entry's task
and kind: `Party { whom, entries, task, kind }`, `waits.rs:13`)
(ruling 249) and `documents::Task.waits_on`, its `Subject::Query`
and `Document::Lines`; jig 02's label and flags.
Files: the v8 schema; the component's `translate/{watches.rs (filter,
41-79 at the base; the `Subject::Tasks` arm, 29-32, the `Document::Tasks`
arm, 167, and the `Lines` arm, 178), task.rs, chips.rs, live.rs (155-210
at `b6a63f8c1`: the old `task_line`, `tasks` and `filter`, which the
query's lines replace)}` (ruling 249); the contract's `queries.rs`
(new) and `documents.rs` (the task's fields, at most 40 lines: it is
over 800; its `Tasks`, 1142, loses `project` and `filter` and holds
`rows: List<TaskLine>` of `queries.rs`, and `TaskFilter` and the old
`TaskLine`, 1125-1141, go) (ruling 249); the link's
`translate/{watches.rs, documents*.rs}`; the core keeps `Watched::Tasks`,
`Document::Tasks` and `documents/project_tasks.rs` (`TaskFilter`,
`ProjectTaskLine`): nothing on the wire reaches them from here, and jig
16.4 removes them with the other shaped subjects (its Files; jig 05's
Choices), so the component maps the wire's subject to `Watched::Query`
and delivers `Document::Lines`, and `Document::Tasks` ends in the
`Reason::Tag` arm beside the documents not on the wire (ruling
249); tests: the component's
`crates/jig-protocol-engine-clients/src/tests/fixtures/{live.rs
(`tasks_document`, 79-88), watches.rs (the `Tasks` subject, 90-108)}` and
the client wire's fuzzy `tests/client-wire/tests/fuzzy_client_wire/documents.rs`
(42-44, 129: `TaskFilter`, `TaskLine`, `Tasks`, `SubjectTasks`) (ruling
249); users: the reference UI's project tasks section (pages.md,
6.3: phase class, executor kind, root, requester, by last change), the
scripted engine's tasks subject and task document, test records,
`tests/client`.
Shape: wire, before `Read`: `enum PhaseClass { working, waiting, held,
closing, ended }`, `enum ExecutorKind { agent, procedure, person }`,
`enum QueryOrder { number, changed, activity, priority, ended }`,
`TaskQuery { project: option u32, requester: option OwnerRequester,
root: option u64, parent: option u64, label: option u32, phase: option
PhaseClass, executor: option ExecutorKind, tracked: option bool, order:
QueryOrder }`; before `Task`: `enum Whom { person: WhomPerson { value:
u64 }, role: WhomRole { project: u32, role: u32 } }` (`people::Whom`),
`enum WaitedFor { question, proposal, escalation, person_task }`,
`WaitsOnParty { whom: Whom, entries: u32, task: u64, kind: WaitedFor }`
(the oldest entry's task and kind, which jig 05's `WaitsOn::Party`
carries; its names there win) (ruling 249), `enum WaitsOn { party:
WaitsOnParty, delegates, news, nothing }`, `ConversationRead { seen:
u64, unread: Count }`; `Task` gains `label: u32`, `flags: TaskFlags`,
`last_activity: u64`, `waits_on: WaitsOn` (its `priority` already says
tracked); in place of `TaskFilter` and the old `TaskLine`: `TaskLine {
chip: Chip, project: u32, requester: OwnerRequester, tracked: option
u32, label: u32, flags: TaskFlags, changed: u64, last_activity: u64,
ended_at: option u64, budget: OwnerBudget, waits_on: WaitsOn, phases:
OwnerPhaseCounts, held_below: u32, conversation: option ConversationRead
}`; `SubjectTasks { query: TaskQuery, window: u32 }`; `Tasks { rows:
bytes 16777216, more: bool }`. Each mirrors jig 05's core type field for
field (`BudgetLine` as `OwnerBudget`, `PhaseCounts` as
`OwnerPhaseCounts`, `tasks::Party` as `OwnerRequester`, whose
`deployment` a query's requester refuses as unknown); where a landed
type differs, the core wins.
Contract: the sketch's `TaskQuery`, `TaskLine` and their parts.
Test shape: the component's tests: `a_task_query_subject_reaches_the_core_with_its_keys`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`a_task_query_watch_delivers_the_live_lines_its_keys_select` (two tasks
of project 1, one tracked: the query `tracked: Some(true)` delivers one
line, by priority; then the story amends the other task tracked, with
priority 2, by `Amend { revision: None, change: Amendment { tracked:
Some(Tracking::Tracked { priority: 2 }), .. } }` (jig 14.2 makes `Amend.revision`
optional and gives `Amendment` `tracked: Option<Tracking>`, ruling 260; at `b6a63f8c1`
it has neither) (ruling 249), and the next delivery holds two
lines, the first one's first), `a_task_line_says_what_its_task_waits_on`;
`tests/client/tests/stories.rs` (scripted):
`a_project_s_tasks_section_watches_one_query_by_last_change` (pages.md,
6.3 and 12.5: the project's live tasks, by last change, the section's
filters as the query's keys; the scripted engine records the one
opening). Ceilings: unit under 20 ms, world and joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_task_query_watch_delivers_the_live_lines_its_keys_select)'`
once: the handle's latest document holds the tracked task's line alone,
and a second delivery follows when the other task is amended tracked.
Bound: 1 s.
Don't: filter or order lines in the client; add keys beyond contract.md,
6.1's.
Size: at most 300 lines.
Removes: `TaskFilter`, the old `TaskLine` and `Tasks` and `SubjectTasks`
fields.

### 22.3 `tasks{query}` read
Scope: a task query's page, live then ended in two groups, each with its
cursor, the ended group cut or absent (contract.md, 6.1;
protocol/client.md, 6).
Needs: the session's start: jig 05's query read in the core with jig
08's ended part through the store's indexes (one stream with one cursor
at `b6a63f8c1`: Shape, Core).
Files: the core's `crates/jig-domain-core/src/documents/task_query.rs` (`admission`, :47, ruling 279) and its tests; the v8 schema; the component's `translate/{reads.rs, pages.rs
(27, 57, 97), pages/rows.rs, cursor.rs (288: `Scope::Tasks` has no
cursor encoding)}`; the contract's `queries.rs`; the link's
`translate/{reads.rs, pages*.rs}`; `jig-client`'s `Read` of a query;
the scripted engine's read of a query (its live tasks; its ended group
none until jig 16.1); test records; the core's
`crates/jig-domain-core/src/{reads.rs (the `Tasks` arms: 140, 157, 191,
252, 297, 333), reads/{vocabulary.rs (`Read::Tasks { query, from }`,
131; `Page.next`, 314; `Answer`), rows.rs (the `Lines` arm, 260-275;
507, 943, 1502), cursors.rs, ended.rs}}` (ruling 249).
Shape: Core (ruling 249): jig 05.3 and 08.5 left one stream:
`Read::Tasks { query, from }` answers one `Page` of live rows, then
stored rows, with one `next` (the `Lines` arm chains the last live page
to a `Cursor::Store`, and answers no `next` for a query with no ended
part). engine.md, 11.2 ("its live tasks first, in its order, then its
ended ones, each group with its own cursor") and protocol/client.md, 6
(a page "carries its two groups, live then ended, each with its cursor,
and says whether the ended group was cut ... or has none") ask for two
groups: `Read::Tasks { query, live: Option<Cursor>, ended:
Option<Cursor> }`; a new `Answer::Tasks(Box<TaskPage>)` with `pub struct
TaskPage { pub live: List<TaskLine>, pub live_next: Option<Cursor>, pub
ended: EndedGroup }` and `pub enum EndedGroup { Rows { rows:
List<TaskLine>, next: Option<Cursor>, cut: bool }, None }` (`None` when
`ended::ended_part(&query)` is `EndedPart::None`); the `Lines` arm ends
at the live group (a `Cursor::Live`, or none), and the ended group reads
from `ended`, or from its start (`Cursor::Store { scope, after: None }`),
through the store's indexes as 08.5 left them. Wire: `ReadTasks { query: TaskQuery, live: option Cursor, ended:
option Cursor }` before `Read`; `Read.tasks: ReadTasks`; `EndedRows {
rows: bytes 16777216, after: option Cursor, cut: bool }`, `enum
EndedGroup { rows: EndedRows, none }`, `TaskPage { live: bytes
16777216, live_after: option Cursor, ended: EndedGroup }`;
`ReadValue.tasks: TaskPage`. Contract: the sketch's `TaskPage`,
`EndedGroup`.
Test shape: the link's: `a_task_page_decodes_its_two_groups_with_their_cursors`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`an_ended_task_is_read_through_its_requester_s_index_after_the_live_ones`
(two tasks the first owner created, one ended), and
`a_query_naming_no_index_key_reads_no_ended_group`. Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(an_ended_task_is_read_through_its_requester_s_index_after_the_live_ones)'`
once: the page's live group holds the live task, its ended group the
ended one, read from the store. Bound: 1 s.
Don't: merge the two groups or their cursors.
Size: aim 300 lines, stop at 575 (ruling 279, the admission repair included; was 380): 180 for the wire, the component and the
view, as before, and 120 for the core's two groups with their cursors
(ruling 249).
Removes: nothing.

### 22.4 Offers and a decision's effect
Scope: each object's acts offered in the operations' names, and what a
decision will do as data (contract.md, 5.3, 5.4; engine.md, 11.5).
Needs: the session's start: jig 03's offers "proposable, and to whom".
Files: `tests/client/src/joint/offered_acts.rs` (the judge's composed control, ruling 280); `crates/jig-domain-core/src/{acts.rs (Act, Offered), documents.rs,
documents/objects.rs}` and a new `crates/jig-domain-core/src/acts/effects.rs`;
the v8 schema; the component's `translate/{objects.rs, task.rs,
objects/*.rs}`; the contract's `effects.rs` (new) and `documents.rs`
(`ActKind`, `Offered`; at most 40 lines); the link's documents'
translation; users: the reference UI's confirmation (it reads the
effect, where it read `Impact` and `LiveDelegate.takeover_budget`), the
proposal card's `Withdraw` act (offered to its proposer alone,
objects.md, 4), the scripted engine's offers, test records. Shared
hotspot: `crates/jig-domain-core/src/documents.rs`. At `b6a63f8c1` the
`Act` renames and `AmendRelease`'s end reach also, in the core,
`documents/offers.rs` (the `Offered` literals at 65 and 215 take
`effect`; the act lists at 59-155), `acts/objects.rs` (20-105) and
`acts/objects/{decisions.rs (47-187), task.rs (18-273)}`,
`acts/requests.rs` (26, 62-66, 143), `acts/standing.rs` (20-43) and
`compound.rs` (112-114); and, in `tests/client/src/joint/wire/`, the
`Offered` literals and `Act` uses of `documents.rs` (491, 703-718),
`escalations.rs` (27, 30), `notes.rs` (60, 63) and `projects/tests.rs`
(151, 427), and the map of `offers.rs` (11-28) (ruling 249).
For the designer's answer of 2026-10-17T11:23Z:
`crates/jig-domain-tasks/src/{run.rs (`run_attempt`, 129: `pub`),
lib.rs (`pub use run::run_attempt;`)}`; the contract's `Impact`
(`documents.rs:1099-1106`); the link's `impact_fields`
(`translate/documents_part3.rs:71-80`) and its fixtures
`tests/fixtures/{reads_part2.rs (92-106), reads_part6.rs (74-83)}`; the
view's `confirmation.rs` (226, 416-431) and
`phrases/{jig.rs (634-636, 1459-1460, 2291-2292, 2764-2765), english.rs
(1367-1374)}`; and the other `Impact` literals, found once with `git grep
-l -E 'effects_pending|effects_settling|PendingEffects|SettlingEffects'
-- crates tests testing examples ':!*/generated/*'` (16 files at
`b6a63f8c1`, these among them).
Shape:
- Core: `acts::Act` renames `Write` to `Message`, `TakeOver` to `Move`,
  `MarkSeen` to `Seen`, gains `Withdraw`, and loses `AmendRelease` (the
  card offers amend-and-release where `Amend` and `Release` are both
  offered, sending jig 14.3's compound); `acts::Offered { act,
  admission, effect: Option<Effect> }`; `pub enum Effect {
  Release(ReleaseEffect), Cancel(Impact), Move { reservation: u64 } }`,
  made when the document is made: a release's from the hold's reason
  (judged afresh, a run again, or uncertain), a cancel's from the task's
  live tree (below), a move's from the reservation the move takes
  (`tasks.takeover_budget`, as `documents/live_delegates.rs:33` reads
  it).
- A cancel's `Impact` (the designer's answer of 2026-10-17T11:23Z;
  contract.md, 5.4: "the live tasks below the task, counted by phase;
  and, of the task and those below it, the live runs (each attempt
  claimed whose answer its task has not heard) and the pending
  proposals (each a task made that no holder has decided)"), over `core.tasks.view_tasks()` and
  `task_tree::descendant`, as `task_tree::tree` (`task_tree.rs:813`)
  walks them:
  - `tasks`: each live task below the task, never the task itself,
    `PhaseCounts::add`ed (`task_tree.rs:256`);
  - `live_runs`: the task and each live task below it whose
    `tasks::run_attempt(&row.phase)` is `Some` (`run.rs:129`: claimed
    or running, closing at `Stage::Run`, or held from one of those);
    no writer slot is read (a run with no write hold takes none, and a
    lost run's slot outlives it);
  - `proposals`: the task and each live task below it whose `proposal`
    is `ProposalState::Pending`, as `task_tree::proposal`
    (`task_tree.rs:630`) reads it;
  - no effect is counted: "Its effects are its connectors' (5.1): each
    withdraws the entries not yet sent and settles those in flight, and
    the document counts none of them".
- Core's `Impact` and `ReleaseEffect` are new, in `acts/effects.rs` (the
  sketch: the core has no `Impact`). `Impact.tasks` is the core's
  `documents::PhaseCounts` (`task_tree.rs:228`, ten fields), which
  crosses as the wire's and the contract's `PhaseCounts`
  (`client-v8.schema:578`, `documents.rs:648`, eight fields, `ended`
  among them) by: `waiting`, `closing`, `held`, `done`, `failed` and
  `cancelled` as they are; `running` the working class, `idle + starting
  + running + backing_off`; `ended` 0 (the core counts each ending by
  its outcome; `ended` is for chips whose source gives none). `Impact`
  loses `effects_pending`, `effects_settling` and `takeover_budget` in
  the wire and the contract (the designer's answer of
  2026-10-17T11:23Z): a move's reservation is `Effect::Move`'s
  (contract.md, 5.4), and a cancel's effects are each connector's
  panel's (objects.md, 11).
- Wire: `PhaseCounts` moves before `Offered` (no byte change), and
  `Impact`, changed in place to `record Impact { tasks: PhaseCounts
  live_runs: u32 proposals: u32 }`, with it; `enum ReleaseEffect {
  judged, run_again, uncertain }`, `EffectRelease { what: ReleaseEffect
  }`, `EffectMove { reservation: u64 }`, `enum Effect { release: EffectRelease, cancel: Impact,
  move_task: EffectMove }`; `Offered { act, admission, effect: option
  Effect }`; `ActKind` renames `write` to `message`, `take_over` to
  `move_task`, `mark_seen` to `seen`, gains `withdraw`, and loses
  `amend_release`.
- Contract: the sketch's `Effect`, `ReleaseEffect`, `Offered`; `Impact`
  loses the three fields (`TaskContext.impact` and `Escalation.impact`
  keep the smaller record).
- Reference UI: the confirmation's `impact_words` says one line for the
  effects in place of two: `JigPhrase::PendingEffects` (254) becomes
  `EffectsSettle`, with no hole, in English "Effects not yet begun will
  be withdrawn; begun effects will settle"; `SettlingEffects` goes, its
  number 255 left unused (the table has gaps); `TakeoverFunds` takes
  `Effect::Move`'s `reservation` (objects.md, 11: "in jig's words, that
  effects not yet made are withdrawn and those in flight settle").
Test shape: the core's unit tests (`crates/jig-domain-core/src/tests/acts.rs`):
`a_cancel_offer_counts_the_tree_it_would_end` (one test, asserting
three facts: the task's own live run and pending proposal count in
`live_runs` and `proposals`, never in `tasks`; a running delegate with
no write hold counts in `live_runs`; a decided proposal is not
counted),
`a_release_offer_on_an_uncertain_effect_says_uncertain`,
`a_move_offer_names_the_reservation_it_takes`,
`a_party_s_own_proposal_offers_withdraw_to_its_proposer_alone`;
`tests/client-protocol/tests/protocol.rs`:
`cancelling_a_task_says_what_ends_before_it_is_sent` (the reference UI's
confirmation, by the fake person, shows the task's delegate and its live
run among what ends). Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(cancelling_a_task_says_what_ends_before_it_is_sent)'` once: the
confirmation's text names the counts the engine's document gave, before
any request leaves. Bound: 1 s.
Don't: compute an effect in the client or the reference UI; copy
`Impact` into a second record.
Size: at most 300 lines.
Removes: `ActKind`'s `write`, `take_over`, `mark_seen`,
`amend_release`, and the core's `Act::AmendRelease`; the reference UI's
reading of `LiveDelegate.takeover_budget` for a move's effect;
`Impact`'s `effects_pending`, `effects_settling` and `takeover_budget`
(wire, contract) and `JigPhrase::SettlingEffects`: the Files' `git
grep` prints nothing after 22.4 (the designer's answer of
2026-10-17T11:23Z).

## Interface sketches

Checked at the plan's base with jig 14's (one edit of the v7 schema in a
scratch worktree, generated with skein-codegen, `cargo clippy -p
jig-protocol-client-wire --lib -- -D warnings` clean; the Rust types in
a scratch crate against the base's contract types, `cargo clippy
-p w7-sketch --all-targets -- -D warnings` clean). Names from jig 05 and
jig 09 are checked at the ready check. The derives, `PhaseClass`,
`ExecutorKind`, `task` of `WaitsOn::Party` and the core types of
`acts/effects.rs` were added after, read against jig `b6a63f8c1`, not
compiled (ruling 249).

`crates/jig-client-protocol-wire/schema/client-v8*.schema`, this
session's records, each declared before its first user (`PhaseCounts`
and `Impact` before `Offered`, `WaitsOn` with its parts before `Task`):

```
# Moved before their first users, unchanged:
record PhaseCounts { ended: u32 waiting: u32 running: u32 held: u32 closing: u32 done: u32 failed: u32 cancelled: u32 }
# Added, or changed in place:
record Impact { tasks: PhaseCounts live_runs: u32 proposals: u32 }
enum PhaseClass { working, waiting, held, closing, ended }
enum ExecutorKind { agent, procedure, person }
enum QueryOrder { number, changed, activity, priority, ended }
record TaskQuery { project: option u32 requester: option OwnerRequester root: option u64 parent: option u64 label: option u32 phase: option PhaseClass executor: option ExecutorKind tracked: option bool order: QueryOrder }
record ReadTasks { query: TaskQuery live: option Cursor ended: option Cursor }
record CreateProcedure { connector: u16 code: u32 }
record BoundsCreate { pool_left: u64 charters: list 32 u32 procedures: list 32 CreateProcedure latest: option u64 budget: u64 maximum_budget: u64 maximum_deadline: option u64 authority: OwnerAuthorityDocument }
enum ReleaseEffect { judged, run_again, uncertain }
record EffectRelease { what: ReleaseEffect }
record EffectMove { reservation: u64 }
enum Effect { release: EffectRelease, cancel: Impact, move_task: EffectMove }
record Offered { act: ActKind admission: Admission effect: option Effect }
record MeParty { number: u64 kind: PartyKind login: DocumentText name: DocumentText }
record Template { number: u32 name: DocumentText executor: TemplateExecutor contract: TaskContractDocument shallower: u32 budget: TemplateBudget deadline: TemplateDeadline tracked: option u32 flags: TaskFlags label: u32 grace: option duration }
record CreatingBounds { within: BoundsCreate beyond: Beyond }
record MeProject { project: u32 name: option DocumentText role: HeldRole pool: option PoolLedger templates: bytes 16777216 creating: option CreatingBounds chat_budget: u64 }
record Me { party: MeParty projects: bytes 16777216 needs: Needs needs_total: u32 updates: Count results_seen: u64 charters: bytes 16777216 cut: bool retention: duration deployment_role: option HeldRole }
record WhomPerson { value: u64 }
record WhomRole { project: u32 role: u32 }
enum Whom { person: WhomPerson, role: WhomRole }
enum WaitedFor { question, proposal, escalation, person_task }
record WaitsOnParty { whom: Whom entries: u32 task: u64 kind: WaitedFor }
enum WaitsOn { party: WaitsOnParty, delegates, news, nothing }
record ConversationRead { seen: u64 unread: Count }
record TaskLine { chip: Chip project: u32 requester: OwnerRequester tracked: option u32 label: u32 flags: TaskFlags changed: u64 last_activity: u64 ended_at: option u64 budget: OwnerBudget waits_on: WaitsOn phases: OwnerPhaseCounts held_below: u32 conversation: option ConversationRead }
record Tasks { rows: bytes 16777216 more: bool }
record EndedRows { rows: bytes 16777216 after: option Cursor cut: bool }
enum EndedGroup { rows: EndedRows, none }
record TaskPage { live: bytes 16777216 live_after: option Cursor ended: EndedGroup }
record SubjectTasks { query: TaskQuery window: u32 }
```

Besides: `Read` renames `frame` to `me` and gains `tasks: ReadTasks`;
`Subject` renames `frame` to `me`; `ReadValue` gains `me: Me` (for
`frame`) and `tasks: TaskPage`; `DeliveryValue` gains `me: Me` (for
`frame`); `ActKind` renames `write`, `take_over`, `mark_seen` to
`message`, `move_task`, `seen`, gains `withdraw` and loses
`amend_release`; `Bounds` renames `goal` to `create: BoundsCreate`;
`Task` gains `label: u32`, `flags: TaskFlags`, `last_activity: u64`,
`waits_on: WaitsOn`; `OwnerProjectPolicy` gains `templates: list 16
TemplateOverride`.

`crates/jig-client-domain/src/contract/`, the types whose wire records
are above; owned by this session, used by jig 15's helpers and jig 16's
reference UI through `jig_client`'s re-export of them:

```rust
// contract/me.rs
/// One template, merged with the project's overrides (its executor, budget and deadline jig 14.2's types).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Template {
    pub number: u32, pub name: DocumentText, pub executor: TemplateExecutor, pub contract: TaskContractDocument,
    pub shallower: u32, pub budget: TemplateBudget, pub deadline: TemplateDeadline, pub tracked: Option<u32>,
    pub flags: TaskFlags, pub label: u32, pub grace: Option<Duration>,
}
/// A procedure a party may create with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CreateProcedure { pub connector: u16, pub code: u32 }
/// Creating's first bound and the project's ceiling past it (`Bounds::Create(CreateBounds)`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CreateBounds {
    pub pool_left: u64, pub charters: List<u32>, pub procedures: List<CreateProcedure>, pub latest: Option<Wall>,
    pub budget: u64, pub maximum_budget: u64, pub maximum_deadline: Option<Wall>,
    /// What the party may give a task alone: their role's, one level shallower, its tasks within a tree's.
    pub authority: Box<OwnerAuthorityDocument>,
}
/// What the party may create in a project, and what lies between the bounds: proposable to whom, or refused.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Creating { pub within: CreateBounds, pub beyond: Beyond }
/// One of the party's projects.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeProject {
    pub project: u32, pub name: Option<DocumentText>, pub role: HeldRole, pub pool: Option<PoolLedger>,
    pub templates: List<Template>,
    /// None: the party may create nothing in the project.
    pub creating: Option<Creating>,
    /// Transitional: jig 16.3 removes it.
    pub chat_budget: u64,
}
/// The signed-in party's own document.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Me {
    pub party: MeParty, pub projects: List<MeProject>, pub needs: Needs, pub needs_total: u32, pub updates: Count,
    pub results_seen: u64,
    /// Transitional: jig 16.3 removes it.
    pub charters: List<OfferedCharters>,
    pub cut: bool, pub retention: Duration, pub deployment_role: Option<HeldRole>,
}
// `OwnerProjectPolicy` gains `templates: List<TemplateOverride>` (jig 14.2's record): the policy's document carries its overrides.

// contract/queries.rs (every type here derives `Debug`: `missing_debug_implementations` is deny)
/// A query's order; ties go by number (jig 05's directions).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QueryOrder { Number, Changed, Activity, Priority, Ended }
/// A phase class a query selects (mirrors jig 05's `PhaseClass`; `jig_client_domain` has neither it nor `ExecutorKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhaseClass { Working, Waiting, Held, Closing, Ended }
/// The kind of executor a query selects (mirrors jig 05's `ExecutorKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutorKind { Agent, Procedure, Person }
/// A task query: a conjunction of fixed keys, and an order (mirrors jig 05's).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskQuery {
    pub project: Option<u32>, pub requester: Option<OwnerRequester>, pub root: Option<u64>, pub parent: Option<u64>,
    pub label: Option<u32>, pub phase: Option<PhaseClass>, pub executor: Option<ExecutorKind>, pub tracked: Option<bool>,
    pub order: QueryOrder,
}
/// Whom an entry waits for (mirrors `people::Whom`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Whom { Person(u64), Role { project: u32, role: u32 } }
/// What kind of entry waits: the oldest's (jig 05's).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaitedFor { Question, Proposal, Escalation, PersonTask }
/// What a task waits on, computed by the engine (`task`, `kind`: the oldest entry's, as jig 05's `WaitsOn::Party`, `waits.rs:13`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaitsOn { Party { whom: Whom, entries: u32, task: u64, kind: WaitedFor }, Delegates, News, Nothing }
/// A conversation's read position over its turns, and what is unread.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConversationRead { pub seen: u64, pub unread: Count }
/// The one record of each list of tasks (mirrors jig 05's).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskLine {
    pub chip: Chip, pub project: u32, pub requester: OwnerRequester, pub tracked: Option<u32>, pub label: u32,
    pub flags: TaskFlags, pub changed: Wall, pub last_activity: Wall, pub ended_at: Option<Wall>,
    pub budget: OwnerBudget, pub waits_on: WaitsOn, pub phases: OwnerPhaseCounts, pub held_below: u32,
    pub conversation: Option<ConversationRead>,
}
/// A query's ended group: rows from one index, or none for its keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndedGroup { Rows { rows: List<TaskLine>, after: Option<Cursor>, cut: bool }, None }
/// One page of a task query: live, then ended.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskPage { pub live: List<TaskLine>, pub live_after: Option<Cursor>, pub ended: EndedGroup }

// contract/effects.rs
/// What lifting a hold does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReleaseEffect { Judged, RunAgain, Uncertain }
/// What a decision will do, from the engine's state as the document was made (`Impact` is the contract's, `documents.rs:1099`, less its three effect and take-over fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect { Release(ReleaseEffect), Cancel(Impact), Move { reservation: u64 } }
/// One act the party may make now, its admission and its effect.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Offered { pub act: ActKind, pub admission: Admission, pub effect: Option<Effect> }
```

`crates/jig-domain-core/src/acts/effects.rs` (new), the core types that the
contract's `Effect` mirrors (the core has no `Impact`; `acts::Offered` gains
`effect: Option<Effect>`) (ruling 249):

```rust
/// What lifting a hold does: the task judged afresh, a run again, or an uncertain effect (made again or not, as the release says).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReleaseEffect { Judged, RunAgain, Uncertain }
/// What a cancel ends, from the task's live tree as the document is made (the wire's `Impact`; contract.md, 5.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Impact {
    /// The live tasks below the task, by phase.
    pub tasks: documents::PhaseCounts,
    /// The task's and those below's attempts whose answer is not heard (`tasks::run_attempt`).
    pub live_runs: u32,
    /// The task's and those below's pending proposals.
    pub proposals: u32,
}
/// What a decision will do.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Effect { Release(ReleaseEffect), Cancel(Impact), Move { reservation: u64 } }
```

## Questions to raise when reached

- None the design leaves open that these increments decide.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-22\.' | sort -u | wc -l` prints 4 (a step's trailer repeats
  across the commits that make it, so the lines are made unique before
  they are counted) (ruling 249).
- The replaced records are gone: `git -C ~/src/rust/jig grep -n -E
  '^(record|enum) (Frame|FrameParty|ProjectRole|TaskFilter|BoundsGoal|GoalProcedure)\b'
  origin/main -- crates/jig-client-protocol-wire/schema` prints nothing.
- The new ones are there: `git -C ~/src/rust/jig grep -n -E
  '^(record|enum) (Me|Template|CreatingBounds|TaskQuery|TaskLine|TaskPage|WaitsOn|Effect)\b'
  origin/main -- crates/jig-client-protocol-wire/schema` prints 8 lines.
