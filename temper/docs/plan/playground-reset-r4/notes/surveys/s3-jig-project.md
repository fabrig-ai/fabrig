Surveyed checkout: `d372bfcf6b`. No builds, tests, writes, or service calls.

**The draft assumes several mechanisms that do not exist at this revision:** horizon retention, complete task erasure, project adoption release, application settings by project scope, and a durable jig project-removal workflow. `RemoveProject` is found only in `temper/docs/design/draft/playground-reset.md:145`.

Paths below are relative to the repository root. In compact reference lists, a directory followed by `file:line` means their concatenation. Reverse-reference lists identify direct source references to the named variants; they are not a compiler-derived, exhaustive dependency graph for every enclosing carrier.

**1. Projects: creation, records, uniqueness, numbering, ownership**

The design contract is:

- `jig/docs/design/domain/people.md:359`: `MakeProject`, requested in deployment scope by a deployment owner.
- `jig/docs/design/domain/people.md:504`: project records and policy changes.
- `jig/docs/design/domain/people.md:522`: creation commits the number, copied policy, sole-owner roster, first period, and answer together.
- `jig/docs/design/domain/people.md:544`: connector-issued deployment acts.

The party request and deployment act are distinct:

```rust
// jig/crates/jig-domain-people/src/boundary.rs:296
MakeProject { name: Box<str>, policy: u32 }

// :645
Act::MakeProject {
    maker: u64,
    name: Box<str>,
    policy: u32,
}
```

The implementation is in **jig-domain-core**, using people’s request, membership, and storage vocabulary:

```rust
// jig/crates/jig-domain-core/src/projects.rs:33
pub(crate) fn make_project(
    &mut self,
    limits: &Limits,
    maker: u64,
    name: Box<str>,
    number: u32,
) -> Result<Made, people::Refusal>
```

Here `number` is the configured **policy number**, not the allocated project number.

`make_project` checks, in order:

1. Maker’s deployment role is `Some(people::Role::Owner)`.
2. Name shape.
3. Configured policy exists.
4. Project, roster, and number capacity.
5. Name uniqueness.
6. Allocates `Family::Project`, installs authority policy and core project state.

The surrounding decision is:

- `jig/crates/jig-domain-core/src/routing/projects.rs:36`, `make_project_request(...)`.
- `:94`, `make_project_act(...)`.
- `:52`: saves `people::Stored::Policy`.
- `:57`: installs `people::Event::Roles` with the maker alone as owner.
- `:61`: opens period `1`.
- `:68`: sets its next boundary to `None`.
- `:70`: emits `Request::ProjectMade { project }`.
- `:71`: returns `Outcome::ProjectMade { project }`.

There is **no separate `ProjectRecord`**. The project’s durable records are:

```rust
// jig/crates/jig-domain-people/src/boundary.rs:770
Stored::Roles {
    project: u32,
    holdings: Box<[Holding]>,
}

// :775
Stored::Policy {
    project: u32,
    name: Box<str>,
    value: crate::PolicyValue,
}
```

Their keys are `people::Key::Roles(u32)` and `people::Key::Policy(u32)`, at `boundary.rs:708` and `:710`.

Core holds these separate projections:

```rust
// jig/crates/jig-domain-core/src/lib.rs:362
pub projects: List<u32>;
// :369
pub(crate) names: Map<u32, Box<str>>;
// :371
pub permission_roles: Map<u32, Box<[people::PermissionRole]>>;
// :373
pub template_overrides: Map<u32, Box<[people::TemplateOverride]>>;
```

Authority holds the policy itself. A reusable in-memory policy removal already exists:

```rust
// jig/crates/jig-domain-authority/src/domain.rs:45
Event::Dropped { project: u32 }
```

`step`, at `domain.rs:199`, handles this at `:270` by removing the policy and emitting `PolicyFact::Dropped { project, existed }`. It does not erase people’s durable policy row.

**Names.** `jig/crates/jig-domain-people/src/names.rs`:

- `:16`: `NAME_BYTES: usize = 64`.
- `:20`: `pub fn name_rule(name: &str) -> Option<NameRule>`.
- `:53`: `pub fn same_name(a: &str, b: &str) -> bool`.

Names must be nonempty, at most 64 UTF-8 bytes, contain no control characters, have no leading/trailing whitespace, and not consist entirely of ASCII digits. Equality uses Unicode simple case folding, without normalization. Creation and rename both compare against `Core.names`; rename excludes its own project (`projects.rs:24`).

**Numbers.** `jig/crates/jig-domain-core/src/numbers.rs`:

- `:39`: `Deployment.projects: u64`, the largest seeded or allocated project number.
- `:48`: `Family::Project`.
- `:199`: `pub fn fresh(counters: &mut Counters, family: Family) -> Option<u64>`.

Creation bounds the allocated value to `u32`. Allocation advances the durable high-water mark; deleting a policy/name would not make its number reusable.

**Seeding.** `jig/crates/jig-domain-core/src/routing/first_start.rs:75`, `project_rows`, consumes `seed_projects` and installs the same core maps. Stored seeded and runtime projects share the people policy/roles families.

**Store placement.** `jig/crates/jig-protocol-store/src/keys.rs:51` encodes:

- Jig owner prefix `01`.
- Core `01 00`.
- Tasks `01 01`.
- People `01 02`.
- Notes `01 03`.
- Accounts `01 04`.

`keys/people.rs:28` and `:32` encode project roles and policy beneath people tags `4` and `5`, with the project written as a `u64`.

**Reverse references—project policy/roster rows.** Direct `Stored::{Policy,Roles}` / `Key::{Policy,Roles}` readers and constructors:

```text
jig/crates/jig-domain-people/src/
  boundary.rs:799 domain.rs:630 tests.rs:441
jig/crates/jig-domain-people/src/domain/
  start.rs:82
jig/crates/jig-domain-people/src/tests/
  start.rs:536

jig/crates/jig-domain-core/src/
  tests.rs:536
jig/crates/jig-domain-core/src/routing/
  first_start.rs:29 people.rs:162 parties.rs:421 projects.rs:27
  application_routes.rs:272 watch_marks.rs:326
jig/crates/jig-domain-core/src/restart/
  census.rs:70
jig/crates/jig-domain-core/src/tests/
  coverage.rs:56 run_choice.rs:617 deployment_connectors.rs:34
  restore_admission.rs:176 acts.rs:270 accounts_document.rs:73
  inbox_selection.rs:39 restart_checks.rs:242 frame_deployment_role.rs:23

jig/crates/jig-protocol-config/src/tests/
  deployment_owners.rs:138
jig/crates/jig-protocol-store/src/
  codec/people.rs:177 keys/people.rs:28
jig/crates/jig-protocol-store/src/tests/
  codec.rs:589 codec/holds.rs:416 fixtures/keys.rs:66

jig/testing/jig-test-domain/src/
  lib.rs:1191
jig/testing/jig-test-service/src/
  clients.rs:538
jig/testing/jig-test-store/src/tests/
  fixtures.rs:299

jig/examples/ops/domain/src/
  tests.rs:546 restart.rs:190

jig/tests/store/tests/suite/
  joins.rs:265
jig/tests/store-codecs/tests/suite/
  profiles.rs:108
jig/tests/channel/src/
  fixtures.rs:620
jig/tests/client/src/joint/
  deployment_story.rs:115 wire/cursor.rs:731
jig/tests/people/src/
  world.rs:503
jig/tests/people/tests/suite/
  restore_open.rs:178 start_restore.rs:183 fuzzy/simulation.rs:50
jig/tests/ops/src/conformance/
  mod.rs:260
jig/tests/core/src/
  fixtures.rs:371 observations.rs:874 watches.rs:261
  effects/bootstrap.rs:30 effects/store.rs:118
jig/tests/core/tests/
  settings/support.rs:212
jig/tests/core/tests/suite/
  first_start.rs:64 settings.rs:27 route_orders.rs:35 deployment_acts.rs:339
  funding.rs:275 start.rs:203 switch_step.rs:277 store_adapter.rs:35

temper/crates/temper-engine-domain/src/
  store.rs:646 restart.rs:505
temper/crates/temper-engine-domain/src/tests/
  resume.rs:273 setup.rs:47 resume_fixture.rs:528
temper/crates/temper-service/src/tests/
  setup_reachable.rs:197 setup_github.rs:35 welcome_policy.rs:147

temper/tests/client/src/front/
  accounts.rs:26
temper/tests/conformance/src/
  scenarios.rs:545
temper/tests/engine/domain/src/
  roles.rs:190 escalation_referee.rs:816 roles_referee.rs:113
  forge_routes.rs:1525 walking/creating.rs:58
temper/tests/engine/domain/tests/suite/
  applications.rs:57 root_routes.rs:3136 settings.rs:324 roles.rs:120
  forge_routes/paths.rs:51 periods.rs:198 escalation.rs:526
```

No references to these jig types were found in skein or smith.

**2. Store families, task erasure, and retention**

**The horizon erasure path is not found.** The design describes it, but `jig/docs/design/protocol/store.status.md:6` explicitly lists sections `7.3`, `11.1`, and `15` retention as **“Not yet.”** `jig/docs/design/domain/people.status.md:15` likewise lists retention as not yet built.

The design also describes **shrinking**, not complete deletion:

- `jig/docs/design/protocol/store.md:2067`: kept tasks stay whole without a horizon.
- `:2069`: past a horizon, full records become a summary; ended-index rows remain.
- `:2074`: transcripts go past a longer horizon.
- `:2076`: answered keys expire independently.
- `:2077`: forgotten-once-read owners/indexes go in one decision.
- `:2080`: their remaining history/activity/transcript goes in a later retention pass.

No task-summary storage variant or horizon sweeper was found.

The inventory is:

| Family / content | Exact record/key and definition | Erasure implemented today |
|---|---|---|
| Deployment number mark | `core::CoreRecord::Deployment(Deployment)`, `CoreKey::Deployment`; `jig/crates/jig-domain-core/src/stored.rs:249`, `:290`; `numbers.rs:8` | Preserved; no project deletion path. |
| Live and ended task | `tasks::Stored::{Live,Ended}(Box<TaskRecord>)`, `Key::{Live,Ended}(u64)`; `jig/crates/jig-domain-tasks/src/boundary.rs:1006`, `:1179`; `index.rs:63` | Ending removes `Live` and saves `Ended`; reading can erase a forgettable `Ended`. No horizon erasure. |
| Retained references | `Stored::Stub(Stub)`, `Key::Stub(u64)`; `boundary.rs:583`, `:1208` | `closing.rs:43`, `prune_stubs`, removes stubs no live task names. Not complete task erasure. |
| Conversation/chat | Same task record: `TaskRecord.flags`, `project`, `requester`, `last_reply`, `replies`; `boundary.rs:1012`, `:1076`, `:1092` | No separate chat family. Forgetting uses the task path below. |
| Agent conversation window | `CoreRecord::RunProof(RunProof)`, `CoreKey::RunProof(u64)`; `stored.rs:259`, `:796`; fields `conversation`, `window` | Live claim bookkeeping; no horizon task purge. |
| Transcript file | Store `Write::DropTranscript { task: u64 }`; `jig/crates/jig-protocol-store/src/boundary.rs:57` | Explicit lower-level file-drop operation exists; no core retention producer. |
| Transcript places | `keys::Own::Transcript { task }`, `Own::Attempt { task, attempt }`; `keys/own.rs:6` | Drop erases transcript place; orphan attempt keys are swept after its commit. |
| Messages | `TaskRecord.inbox: Box<[Word]>`, `boundary.rs:1164`; `Word`, `:154` | Embedded in task rows, not a separately keyed messages family. Immutable activity/turns also carry message evidence. |
| Task history | `Stored::History(History)`, `Key::History { task, revision }`; `boundary.rs:1198`, `index.rs:85`; `control.rs:108` | No horizon deletion. Not removed by current read-triggered forget. |
| Milestones | `Stored::Milestone(Milestone)`, `Key::Milestone { task, position }`; `boundary.rs:1189`, `index.rs:90`; `control.rs:161` | No horizon deletion. |
| Activity | `Stored::Activity(ActivityLine)`, `Key::Activity { task, position }`; `boundary.rs:1187`, `index.rs:95`; `activity.rs:10` | No horizon deletion. |
| Ended task indexes | `Key::EndedByParty { party, label, ended_at, task }`, `Key::EndedByProject { project, label, ended_at, task }`; `index.rs:125`, `:133` | Current forget removes party index and, if listed, project index. |
| Task edges | `Stored::{Delegate,EndedDelegate}(Edge)`, keys `{ requester, task }`; `boundary.rs:1185`, `index.rs:115`; `index.rs:197` | Lifecycle/move code maintains these. No whole-task historical sweep. |
| Results census | `Stored::Result(ResultRow)`, `Key::Result { party, position }`; `boundary.rs:1181`, `index.rs:164` | Current forget erases its result row. |
| Party updates | `Stored::Update(Update)`, `Key::Update { party, at, task, object }`; `index.rs:41`, `:100` | Current forget erases the result update; old reply updates remain, with readers skipping absent owners. |
| Read positions | `people::Stored::{ReadResults,ReadConversation,EndedRead}` and matching keys; `people/src/boundary.rs:749`, `:700` | Forget removes the task’s ended position. Global `ReadResults { party }` is retained. |
| Policy/name copy | `people::Stored::Policy { project, name, value }`, `Key::Policy(u32)` | No project erasure. |
| Project roles | `people::Stored::Roles { project, holdings }`, `Key::Roles(u32)` | Replaced as a whole; no project removal operation. |
| Parties/sign-ins | `Stored::Person { number, identity }`, `Key::Person(u64)`; `Stored::SignIn`, `Key::SignIn(u64)`; `people/src/boundary.rs:742` | Global party identities, not project-owned records. Service credential removal is itself marked not built. |
| Project-period funds and party pools | `Stored::{Ledger,ClosedLedger}(FundingRecord)`, `Key::{Ledger,ClosedLedger}(Funder)`; `tasks/src/funders.rs:330`; `value.rs:190` | Superseded settled ledgers move to `ClosedLedger`; they are not erased by task forgetting. |
| Resource writer/pool slots | `Stored::Writer(WriterSlot)`, `Stored::Pool(PoolSlots)`; `boundary.rs:870`, `:880` | Lifecycle/connector mechanisms release these. `PoolSlots` is resource capacity, distinct from a party’s financial pool. |
| Pending/decided proposals | `Stored::{PersonProposal,DecidedProposal}`; `boundary.rs:1191` | Lifecycle decisions archive proposals. No project purge. |
| Core turns/terminals | `CoreKey::Turn { task, attempt, turn }`, `Terminal { task, attempt }`; `core/src/stored.rs:300` | Turns are encoded into transcript files; terminals have core keys. No horizon purge. |
| Calls and retained decisions | `CoreKey::Call(CallKey)`, `ProposalDecision { proposer, proposal }`, `EscalationDecision { task, revision }`; `stored.rs:296` | Named-call lifecycle pruning exists; no project-wide erasure. |
| Goal projections | `CoreKey::Projection(ProjectionKey)`; `projections.rs:78`: `Goal`, `Task`, `Lifecycle`, `Revision` | Projection settlement has its own release path, including `Ask::ForgetProjection`; not project deletion. |
| Adoptions | No jig-owned adoption record family | Connector-owned; question 4. |
| Settings | `CoreKey::Settings(settings::Key)`; `settings.rs:116` | Current implementation has deployment settings only; no project-scope erase interface. |
| Scoped notes | `notes::Record::{Entry,Line}`, `Key::Entry { name }`, `Key::Line { scope, name }`; `notes/src/boundary.rs:80` | Individual `Change::Delete` erases entry and line (`notes/src/domain.rs:256`); no project sweep. |
| Account project uses | `accounts::Stored::Account { uses: Option<Box<[Use]>>, … }`, `Use::Project(u32)`; `accounts/src/records.rs:19`, `:46` | No project-removal cleanup. |
| Retained party answers | `people::Stored::Answer { key, ask, models, outcome, at }`, `Key::Answer(RequestKey)`; `boundary.rs:778` | Timed expiry exists. No check that a referenced project has since disappeared. |
| Retained deployment answers | `Stored::Acted { key, act, outcome, at }`, `Key::Acted(ActKey)`; `boundary.rs:740` | Timed expiry exists; replay currently returns saved outcomes unchanged. |

The financial sources are exactly:

```rust
// jig/crates/jig-domain-tasks/src/value.rs:190
pub enum Funder {
    Task(u64),
    Pool { project: u32, person: u64, period: u64 },
    Recurring { project: u32, task: u64, period: u64 },
    Period { project: u32, period: u64 },
}
```

A deployment-wide or party-wide period-total family independent of the project **was not found**. Existing party pools include the project in their identity. The draft’s retained-spend requirement therefore needs more than simply preserving an existing global total.

Task ending posts spend once to its original funding source:

- `jig/crates/jig-domain-tasks/src/funders.rs:246`, `end`.
- `:267`: increments original ledger `numbers.spent_below`.
- `jig/crates/jig-domain-tasks/src/retire.rs:8`, `retire`.
- `:59`, `close`: erases `Key::Ledger(funder)` and saves `Stored::ClosedLedger(row)`.

**Existing partial “erase now.”**

```rust
// jig/crates/jig-domain-core/src/forget.rs:87
pub(crate) fn forget(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<crate::Event>,
    owner: &Owner,
) -> crate::Requests
```

`Owner::of_row(&tasks::TaskRecord) -> Option<Self>`, at `:31`, accepts only an ended `Retention::ForgottenOnceRead` task requested by a person, with a result position.

`forgets(owner: &Owner, reading: Reading) -> Option<u64>`, at `:73`, requires no live reference, read result, and read conversation turns where applicable.

`forget`, at `:108`, erases:

```rust
tasks::Key::Ended(owner.task)
update.key()                         // result update
index.key()                          // EndedByParty
tasks::Key::Result { party, position: owner.result_position }
// plus EndedByProject when owner.listed
```

It also sends `people::Event::Forget { project, task, party, last_reply }`.

`jig/crates/jig-domain-people/src/inbox.rs:861`, `forget_event(...)`, erases `Key::EndedRead { task, party }` and calls `forget_chat` (`:600`) to remove volatile conversation contributions.

This is **not** an unconditional complete erase: it neither removes history/activity/milestones/transcript nor accepts arbitrary kept tasks.

**Transcript erasure mechanism.**

- `jig/crates/jig-protocol-store/src/commits.rs:57`: accepts `Write::DropTranscript`.
- `:154`: erases `Own::Transcript { task }`.
- `transcripts/drop.rs:66`: `queue(store: &mut Store, task: u64, through: u64) -> Option<()>`.
- `:86`: `erases(...)` removes orphan `Own::Attempt` keys after the drop commit.
- `:149`: opens the transcript’s shard directory.
- `:192`: removes the task file.
- `:206`: syncs its directory before continuing.
- `transcripts.rs:161`, `name`, and `:175`, `path`: filename is sixteen hexadecimal digits, beneath the sixteen-digit hexadecimal name of `task >> 12`.

There is no equivalent variant in core’s write vocabulary:

```rust
// jig/crates/jig-domain-core/src/routing/vocabulary.rs:130
pub enum Write {
    CopySettingsRevision(u32),
    Save(Record),
    Erase(Key),
}
```

Consequently a complete domain-requested task purge needs a bridge to the store’s existing transcript-drop mechanism.

**Reverse references—transcript places/drop:**

```text
jig/crates/jig-protocol-store/src/
  boundary.rs:57 commits.rs:57 transcripts.rs:100 keys/own.rs:22
  transcripts/drop.rs:51 transcripts/read.rs:181 transcripts/cut.rs:62
jig/crates/jig-protocol-store/src/tests/
  transcripts.rs:49 fixtures/keys.rs:28 frame_patches.rs:118
jig/testing/jig-test-store-files/src/store/
  readback.rs:147
jig/tests/store/src/
  model.rs:56
jig/tests/store/tests/
  qa_all/qa_frames.rs:73 support/frame_reads.rs:83
  suite/pending_switch.rs:93 suite/crashes.rs:141
jig/tests/channel/src/world/
  store.rs:103
temper/tests/one-process/src/referee/
  durable.rs:253
```

**Reverse references—read-position variants:**

```text
jig/crates/jig-domain-people/src/
  boundary.rs:795 domain.rs:2019 ended.rs:48 inbox.rs:585
  domain/start.rs:138 inbox/moving.rs:18 tests.rs:353
jig/crates/jig-domain-people/src/tests/
  moving_chat.rs:38 ended.rs:39 start.rs:448
jig/crates/jig-domain-core/src/
  inbox/census.rs:106 restart/census.rs:25 start/links.rs:115
jig/crates/jig-domain-core/src/routing/
  people.rs:158 application_routes.rs:268 watch_marks.rs:321
jig/crates/jig-domain-core/src/tests/
  inbox_selection.rs:218 restore_admission.rs:173 restart_checks.rs:246
jig/crates/jig-protocol-store/src/
  codec/people.rs:159 patching/check.rs:156 historical.rs:12
  restore/reader.rs:136 restore.rs:101 keys/people.rs:10 reads/indexed.rs:255
jig/crates/jig-protocol-store/src/tests/
  readers.rs:57 restore.rs:393 phase.rs:1544 fixtures/keys.rs:63
  reads/indexed.rs:395 format.rs:144 keys.rs:138 codec.rs:656 intervals.rs:372
jig/testing/jig-test-service/src/
  clients.rs:535
jig/testing/jig-test-store/src/tests/
  fixtures.rs:302
jig/tests/store/tests/
  qa_all/qa_check.rs:365 suite/readers.rs:220 suite/patching.rs:129
jig/tests/one-process/src/
  referee.rs:339
jig/tests/client/src/joint/wire/
  cursor.rs:713
jig/tests/ops/src/conformance/
  mod.rs:1178 observations.rs:235
jig/tests/core/src/
  watches.rs:257 observations.rs:222
jig/tests/core/tests/suite/
  forgetting.rs:86 activity.rs:211 operations.rs:356 switch_step.rs:300
jig/tests/conformance/src/scenarios/
  conversations.rs:131
jig/tests/people/src/
  world.rs:500 ended.rs:18
jig/tests/people/tests/suite/
  memory.rs:200 restore_open.rs:179 start_restore.rs:184
  simulation.rs:771 fuzzy/simulation.rs:51
temper/crates/temper-engine-domain/src/
  archive.rs:202 store.rs:392
temper/crates/temper-service/src/store/
  ranges.rs:94
temper/tests/conformance/src/scenarios/
  conversations.rs:139
temper/tests/engine/domain/src/
  roles.rs:221 roles_referee.rs:121
temper/tests/engine/domain/tests/suite/
  loads.rs:327
```

**Reverse references—core write variant addition**, relevant if transcript dropping is exposed through `core::Write`:

```text
jig/crates/jig-domain-core/src/
  routing/vocabulary.rs:132 settings/held.rs:290 settings/restart.rs:555
  settings/tests.rs:701 tests/own_watches.rs:28 tests/activity.rs:43
jig/crates/jig-protocol-store/src/
  boundary.rs:49 commits.rs:97 tests/commits.rs:319
jig/examples/ops/domain/src/
  store.rs:56 route.rs:241 tests.rs:268 tests/host_tools.rs:46
jig/testing/jig-test-domain/src/
  routes.rs:29 lib.rs:56
jig/testing/jig-test-store/src/
  route.rs:69
jig/testing/jig-test-store-files/src/
  store.rs:458
jig/testing/jig-test-service/src/
  clients.rs:486
jig/tests/store/src/
  model.rs:35
jig/tests/one-process/src/
  referee.rs:502
jig/tests/ops/src/conformance/
  mod.rs:328
jig/tests/core/src/
  application_observations.rs:60 watches.rs:234 observations.rs:216
jig/tests/core/tests/suite/
  settings.rs:1042 stories.rs:334 activity.rs:168 charters.rs:280 route_orders.rs:55
jig/tests/channel/src/world/
  store.rs:215
temper/crates/temper-engine-domain/src/
  store.rs:556 decision.rs:297 route.rs:275 tests/setup.rs:440
temper/crates/temper-service/src/
  store/route.rs:104 clients.rs:652
temper/tests/engine/domain/src/
  escalation_referee.rs:497 roles_referee.rs:102 store.rs:433
temper/tests/engine/domain/tests/suite/
  root_routes.rs:3707 settings.rs:516
temper/tests/one-process/src/referee/
  durable.rs:213 durable/tests.rs:105 restart.rs:52
```

**3. Compound cancellation and grace**

There are three relevant layers.

Party operations:

```rust
// jig/crates/jig-domain-people/src/compound.rs:8
pub enum Operation {
    Cancel { task: u64, reason: Box<[u8]>, grace: Option<Duration> },
    // Move, Amend, Release, Message, Stop
}
```

Task operations additionally carry the numbered finish message:

```rust
// jig/crates/jig-domain-tasks/src/compound.rs:10
Operation::Cancel {
    task: u64,
    reason: Box<[u8]>,
    grace: Option<Duration>,
    message: Option<u64>,
}
```

The task child accepts:

```rust
// jig/crates/jig-domain-tasks/src/boundary.rs:1421
Event::Compound {
    reply_to: ReplyTo,
    by: Party,
    project: u32,
    operations: List<crate::Operation>,
    period: u64,
    pool_budget: u64,
    period_budget: u64,
}
```

`jig/crates/jig-domain-tasks/src/compound.rs:154`:

```rust
pub(crate) fn compound(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Compound,
    out: &mut Queue<Request>,
)
```

It stages/checks **every** operation before applying any; failure returns `Request::CompoundRefused { reply_to, at, problem }`. Success applies the staged plans and returns `Request::Done`.

The deployment restriction is `maker_allowed`, `compound.rs:293`: `Party::Deployment { project }` may only submit `Operation::Cancel` for tasks of that project. Other operation kinds return false.

The connector deployment act is specifically `people::Act::Close`; there is no general `Act::Compound`:

```rust
// jig/crates/jig-domain-core/src/routing/deployment_acts.rs:99
fn close(
    core: &mut Core,
    work: &mut Queue<Event>,
    request: Token,
    project: u32,
    task: u64,
    grace: Option<skein_lib::Duration>,
    reason: Box<[u8]>,
) -> Result<(), people::Refusal>
```

It requires a live conversation requested by a person. At `:128` it appends:

1. Each direct live delegate: cancel with `grace: None`.
2. The conversation: cancel with the supplied grace.

It then sends the compound as `tasks::Party::Deployment { project }` (`:152`). Cancelling delegates closes their subtrees.

Grace is durable task state:

```rust
// jig/crates/jig-domain-tasks/src/grace.rs:9
pub struct Grace {
    pub by: Party,
    pub until: Wall,
    pub reason: Box<[u8]>,
    pub message: u64,
}
```

- `control.rs:1061`, `check_cancel(...) -> Result<CancelPlan, Box<Problem>>`.
- `control.rs:1114`, `apply_cancel(...)`.
- `grace.rs:31`, `check_finish(...) -> Result<FinishPlan, Refusal>`.
- `grace.rs:90`, `apply_finish(...)`.
- `grace.rs:162`, `fire(...)`.
- `closing.rs:133`, `cancel_tree(...)`.
- `closing.rs:228`, `progress(...)`.

Waiting/held tasks can cancel immediately despite a supplied grace; a task is asked to finish once, and a later cancel can shorten its deadline.

**A successful compound answer means cancellation was accepted, not that every task has ended.** The proposed deletion must separately wait for closing and connector settlement before erasing records.

**Reverse references—task compound API:**

```text
jig/crates/jig-domain-tasks/src/
  compound.rs:162 domain.rs:592 lib.rs:73
  tests/revisions/compound.rs:21 tests/payer.rs:117
jig/crates/jig-domain-core/src/
  compound.rs:34 revisions.rs:66 roles.rs:26 amendments.rs:130
  acts/bounds.rs:108 acts/objects/task.rs:122
  routing/vocabulary.rs:901 routing/deployment_acts.rs:152
jig/tests/tasks/src/
  world.rs:502
jig/tests/tasks/tests/
  open/resume.rs:279 lifecycle/compound.rs:28
  suite/added.rs:913 suite/fuzzy/amendments.rs:357
  suite/placed.rs:649 suite/kept.rs:118 suite/circles.rs:290
```

**4. Connector notification of adoption release or settings erasure**

**Project adoption release notification: not found.**

Current adoption request:

```rust
// jig/crates/jig-domain-core/src/routing/vocabulary.rs:95
Ask::Adopt {
    request: Token,
    project: u32,
    adoption: people::Adoption,
}
```

Current answer:

```rust
// jig/crates/jig-domain-core/src/connector.rs:173
Event::Adopted {
    owner: Token,
    project: u32,
    role: people::ResourceRole,
}
```

The opaque payload is:

```rust
// jig/crates/jig-domain-people/src/boundary.rs:184
pub struct Adoption {
    pub resource: ResourceName,
    pub role: ResourceRole,
    pub options: Box<[Box<[u8]>]>,
}
```

Core’s resource-role map is volatile (`core/src/lib.rs:351`), rebuilt from connector reports. There is no jig adoption storage family.

Do not reuse this similarly named operation as though it released an adoption:

```rust
// jig/crates/jig-domain-core/src/routing/vocabulary.rs:99
Ask::Release {
    task: u64,
    root: u64,
    ending: tasks::Ending,
    entry: u64,
}
```

That releases a **closing task’s connector holds/effects**.

Temper stores the adoption in `temper_engine_domain_forge::Repository`:

- `temper/crates/temper-engine-domain-forge/src/boundary.rs:98`: `Repository { project: u32, home: bool, provider, host, owner, name, prefix, role, … }`.
- `:811`: `Stored::Repository(Repository)`.
- `:886`: `Key::Repository(skein_forge::api::Repository)`.

**Application settings erasure notification: not found.** The current jig implementation does not yet offer the application-kind interface described in engine section 17.5:

```rust
// jig/crates/jig-domain-people/src/settings.rs:3
pub enum Setting {
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
}
```

`jig/crates/jig-domain-core/src/settings.rs:116` has `Key::{Default,Changed,Held,Charter,Proposal,Extent,History}`. `jig/crates/jig-protocol-store/src/keys/settings.rs:58` writes a literal scope `0`; `:99` rejects any decoded scope other than `0`.

Temper’s configuration design at `temper/docs/design/protocol/configuration.md:1472` declares:

- Kind 2: project landing rules.
- Kind 4: project landing, explicitly designed/not built.
- Kind 5: repository gates, explicitly designed/not built.

Moreover:

```text
temper/crates/temper-protocol-store/schema/store-v10.schema:217
# Key family 20 is reserved for Setting, still design-only.
```

Thus there are no existing kind-2/4/5 setting-row erase callbacks to extend at this revision.

**Reverse references—core connector-ask dispatch surfaces** that a new project-release/settings-erasure notification would reach:

```text
jig/crates/jig-domain-core/src/
  routing/tasks.rs:756 routing/parties.rs:464 routing/application_routes.rs:63
  routing/run_delegation.rs:172 stopping.rs:69 projections.rs:339
  tests/route_orders.rs:51 tests/fill.rs:98
jig/examples/ops/domain/src/
  route.rs:498
jig/testing/jig-test-domain/src/
  routes.rs:172 tests/link.rs:413
temper/crates/temper-engine-domain/src/
  route/asks.rs:73 tests/projection_settled.rs:127
```

**5. Deployment grants and RemoveAccount**

The grant names a typed act kind:

```rust
// jig/crates/jig-domain-people/src/boundary.rs:673
pub enum ActKind {
    Close,
    AddKey,
    RemoveAccount,
    Adopt,
    MakeProject,
}

// :688
pub struct ActGrant {
    pub connector: u16,
    pub may: Box<[ActKind]>,
}
```

`ActKind::ALL`, at `:683`, bounds configured kind entries.

Configuration parsing is `jig/crates/jig-protocol-config/src/acts.rs:8`, `parse(...) -> Box<[ActGrant]>`. `grant`, at `:43`, accepts these strings at `:60`:

```rust
b"close"          => ActKind::Close,
b"add_key"        => ActKind::AddKey,
b"remove_account"=> ActKind::RemoveAccount,
b"adopt"          => ActKind::Adopt,
b"make_project"   => ActKind::MakeProject,
```

Unknown kinds, duplicate kinds, duplicate connector entries, and unconfigured connectors are refused.

Temper already grants removal:

```json
// temper/crates/temper-init/template/temper.json:3
"acts": [{"connector": 1, "may": ["close", "remove_account"]}]
```

Runtime admission:

- `jig/crates/jig-domain-people/src/domain/acts.rs:54`, `kind(&Act) -> ActKind`.
- `:68`, `admit(...)`.
- `:88`: retained answer replay precedes checking current grants.
- `:111`: requires matching connector and `grant.may.contains(&kind(&act))`.
- `:120`: requires at least one configured first owner.
- `:155`: emits `Request::Act { request, connector, act }`.

`RemoveAccount` itself is `Act::RemoveAccount { account: u32 }`, `people/src/boundary.rs:662`.

`core/src/routing/deployment_acts.rs:71` maps it to `people::AccountChange::Remove { account }`; helper `account`, at `:86`, fixes the holder to `accounts::Holder::Deployment`. The deployment act cannot remove a party’s account.

Retained acts use:

```rust
// jig/crates/jig-domain-people/src/boundary.rs:637
pub struct ActKey {
    pub connector: u16,
    pub key: [u8; 16],
}

// :740
Stored::Acted { key: ActKey, act: Act, outcome: Outcome, at: Wall }
```

`domain/acts.rs:175`, `save`, persists permanent answers. `:190`, `restore`, erases expired answers or retains them. Timer expiry is `people/src/domain.rs:654`; ordinary answer expiry follows at `:657`.

**The draft’s “retained answer naming a removed project answers unknown” behavior is not present.** Saved act replay currently clones the saved outcome, independent of current grants or project existence (`domain/acts.rs:88`).

**Reverse references—deployment grants/retained-act API:**

```text
jig/crates/jig-domain-people/src/
  boundary.rs:688 domain.rs:145 lib.rs:104 tests.rs:611
  domain/acts.rs:51 domain/start.rs:211 domain/start/fit.rs:236
  tests/act_charges.rs:49 tests/start.rs:215
jig/crates/jig-domain-core/src/
  lib.rs:161 routing/deployment_acts.rs:24 routing/people.rs:156
  routing/application_routes.rs:273 routing/watch_marks.rs:331
  restart/cursor.rs:43 restart/census.rs:73
jig/crates/jig-protocol-config/src/
  acts.rs:4 lib.rs:58 tests/acts.rs:5
jig/crates/jig-protocol-store/src/
  codec/people.rs:190 codec/people/acts.rs:23 keys/people.rs:36
  tests/keys.rs:134 tests/codec.rs:1347 tests/intervals.rs:46
jig/testing/jig-test-service/src/
  clients.rs:540
jig/tests/client/src/joint/wire/
  cursor.rs:708
jig/tests/ops/
  tests/suite/acts.rs:107 src/conformance/config.rs:115
jig/tests/core/
  tests/suite/deployment_acts.rs:23 tests/credentials/move_retention.rs:87
  src/observations/projects.rs:82
jig/tests/people/src/
  world.rs:149
jig/tests/people/tests/suite/
  memory.rs:21 acts.rs:1 restore_open.rs:122 start_restore.rs:101

temper/crates/temper-engine-domain/src/
  route/demo.rs:276 store.rs:778
temper/crates/temper-engine-domain-forge/src/tests/
  demo.rs:499 reconcile.rs:213
temper/crates/temper-protocol-store/src/
  records.rs:57 tests/records.rs:873
temper/crates/temper-service/src/tests/
  setup.rs:1693
temper/tests/codecs/tests/suite/fuzzy/
  store.rs:47
temper/tests/engine/domain/
  tests/suite/demo.rs:21 src/roles.rs:225 src/roles_referee.rs:125
```

**6. Rows resumed at startup; kept multi-step acts**

**A jig-owned durable multi-step project act: not found.**

`PendingAct`, `jig/crates/jig-domain-people/src/domain/acts.rs:8`, contains the act, its key, and reply waiters **in memory**. `Stored::Acted` stores the completed outcome, not a pending continuation.

Jig does have durable task closing:

- `jig/crates/jig-domain-tasks/src/boundary.rs:730`: `Stage::{Run,Delegates,Effects,Releases,Settled}`.
- `:751`: `Closing { stage: Stage, ending: Ending }`.
- `TaskRecord.phase` retains that continuation.
- `closing.rs:228`, `progress`, advances it.
- `closing.rs:330` onwards settles subscriptions, funding, references, holds and then archives the ending.

Jig also owns startup sequencing:

```rust
// jig/crates/jig-domain-core/src/start/script.rs:7
pub fn start_done(
    &mut self,
    env: &Env<Limits>,
    done: Phase,
    out: &mut Queue<Request>,
) -> Action
```

The sequence includes loading core/inbox, restoring connectors, linking, sweeping, switching wires, adopting runs, reading connectors afresh, settling outboxes, then `Phase::Open` (`start.rs:10`; `start/script.rs:14` onwards). No project-deletion continuation is present.

**Temper’s `DemoAct` is the concrete persisted retry example.**

```rust
// temper/crates/temper-engine-domain-forge/src/boundary.rs:1634
pub struct DemoAct {
    pub key: [u8; 16],
    pub act: DemoActKind,
}
pub enum DemoActKind {
    Close { project: u32, task: u64 },
    RemoveAccount { account: u32 },
}
```

Lifecycle:

1. `temper/crates/temper-engine-domain/src/route/demo.rs:251`, `list_end_acts`, selects work before making any act.
2. `temper-engine-domain-forge/src/demo.rs:159`, `acts`, saves each `Stored::DemoAct`.
3. `:167`, `send_acts`, emits `Request::SendDemoAct`.
4. `temper-engine-domain/src/route/demo.rs:285`, `send_end_act`, translates to jig `Event::Act`.
5. `temper-engine-domain/src/restart.rs:408` sends `forge::Event::SendDemoActs` after restart.
6. `route/demo.rs:313`, `end_acted`, retains retryable answers.
7. `forge/src/demo.rs:173`, `act_answered`, erases a completed `Key::DemoAct`.

`Again`, missing grant, and missing first owner keep the row. `Unknown`, `Ended`, successful compound close, and account removal count as done (`route/demo.rs:321`).

These rows are **independent acts**, not an ordered project-delete → forge-remove → account-remove workflow. Their success conditions also do not wait for cancellation’s full closing.

**Reverse references—`DemoAct` lifecycle:**

```text
temper/crates/temper-engine-domain-forge/src/
  boundary.rs:861 demo.rs:151 domain.rs:125 lib.rs:251 reconcile.rs:28
  restore.rs:240 tests.rs:357
temper/crates/temper-engine-domain-forge/src/tests/
  demo.rs:498 restore_fit.rs:84 reconcile.rs:212
temper/crates/temper-engine-domain/src/
  boundary.rs:814 domain.rs:65 decision.rs:67 admission.rs:84 restart.rs:408
  route/demo.rs:262 route/forge_route.rs:2990 route/held.rs:189
temper/crates/temper-protocol-store/src/
  records.rs:48 keys.rs:9 tests/records.rs:872 tests/fixtures.rs:176 tests/keys.rs:170
temper/crates/temper-service/src/
  route.rs:294 clients/translation.rs:131
temper/crates/temper-fake/src/
  check.rs:143
temper/tests/client/src/
  translate.rs:321 translate/forge.rs:448
temper/tests/codecs/tests/suite/fuzzy/
  store.rs:42
temper/tests/system/src/
  composed.rs:884
temper/tests/engine/domain/
  tests/suite/demo.rs:479 src/walking.rs:900 src/escalation.rs:796
temper/tests/engine/forge/
  tests/suite/simulation.rs:818 src/world.rs:778
temper/tests/engine/protocol-forge/
  tests/suite/protocol.rs:507 src/protocol.rs:504
```

**7. Conformance world, referee, cases and end checks**

Both locations exist and have different responsibilities:

| Location | Responsibility |
|---|---|
| `jig/testing/jig-conformance/` | Reusable application-independent harness, observations, referee and shared scenarios. |
| `jig/tests/conformance/` | Concrete testing application implementing those interfaces. |
| `jig/tests/core/` | Lower complete engine world, including detailed forgetting/project cases. |
| `jig/testing/jig-test-domain/` | Concrete test application root composed around jig. |

Interfaces:

- `jig/testing/jig-conformance/src/harness.rs:11`: `pub trait Durable`.
- `:95`: `pub trait Application`.
- `:228`: `Harness::new(config, seed)`.
- `:278`: `on_commit(&mut self, hook: impl FnMut(&Self) + 'static)`.
- `:303`: `crash(&mut self) -> Result<(), Violation>`.
- `:325`: `send(&mut self, event: A::Event)`.
- `:334`: `iterate(&mut self) -> Result<bool, Violation>`.
- `:458`: `drain(&mut self) -> Result<Outcome, Violation>`.
- `jig/testing/jig-conformance/src/scenarios.rs:128`: `ScenarioApplication::configuration`.
- `:133`: `ScenarioApplication::action`.
- `:138`: `ScenarioApplication::check`.
- `:249`: `run<A: ScenarioApplication>(scenario, cut, seed) -> Result<Option<Outcome>, Violation>`.

`run` executes the program, drains/crashes as directed, and calls `A::check` at the end.

Concrete bindings:

- `jig/tests/conformance/src/lib.rs:147`: `Testing`.
- `:242`: `impl Application for Testing`.
- `:696`: durable/fake evidence translated by `observed`.
- `jig/tests/conformance/src/scenarios.rs:104`: `impl ScenarioApplication for Testing`.
- `:132`: final scenario checks.

**Existing example matching this work:**

- `jig/tests/conformance/src/scenarios/conversations.rs:20`: creates one kept and one forgettable conversation.
- `:56`: marks turns.
- `:65`: marks results.
- `:127`: `check_ending(world: &World) -> Result<(), Violation>`.

Its final check explicitly scans keys:

```rust
// jig/tests/conformance/src/scenarios/conversations.rs:141
world.store.rows.keys().all(|key| match key {
    root::Key::Core(
        core::Key::Tasks(
            tasks::Key::Ended(task)
            | tasks::Key::EndedByParty { task, .. }
            | tasks::Key::EndedByProject { task, .. },
        )
        | core::Key::People(
            people::Key::ReadConversation { task, .. }
            | people::Key::EndedRead { task, .. }
        ),
    ) => *task != forgotten,
    // Result updates are checked separately.
    _ => true,
})
```

It separately scans `Stored::Result` values (`:158`). The test exercising a crash at the result-mark commit is `jig/tests/conformance/tests/suite/scenarios.rs:102`, `conversation_ends_while_read_chosen_cut_ends_the_story`.

A simpler direct engine case is:

```rust
// jig/tests/core/tests/suite/forgetting.rs:39
fn a_conversation_forgotten_once_read_leaves_no_row_once_read()
```

It finishes a task, marks turns/results, and asserts the ended owner and ended indexes disappeared. Despite its name, it does not assert complete history/activity/transcript removal.

**Referee end checks.**

- `jig/testing/jig-conformance/src/referee.rs:243`: `Snapshot`.
- `:294`: `Observed`.
- `:499`: durable snapshot checks.
- `:922`: `Referee::observe`.
- `:1121`: `check_at(&self, now: u64) -> Result<(), Violation>` checks delivery and uncertainty deadlines.
- `referee/positions.rs:4`: retained tasks cannot lose or regress their read positions.

`Snapshot` has tasks, positions, writers, pools, decisions, balances, receipts, etc. It does **not** contain an inventory of every store key, project policy, scoped note, account-use entry, or application setting. Therefore **a referee-wide “no row names this removed project” check is not found**. The existing scenario uses explicit per-family key/value scans in its ending check.

Changes to shared conformance interfaces also reach:

```text
jig/tests/ops/src/conformance/mod.rs:169
jig/tests/ops/src/conformance/observations.rs:177
temper/tests/conformance/src/lib.rs:159
temper/tests/conformance/src/scenarios.rs:138
temper/tests/conformance/src/observations.rs:145
jig/tests/one-process/src/referee.rs:309
temper/tests/one-process/src/referee.rs:194
jig/tests/core/tests/suite/referee.rs:446
jig/tests/core/tests/referee/fixtures.rs:95
jig/tests/conformance/tests/suite/referee.rs:220
jig/testing/jig-conformance/src/referee/compound/tests.rs:52
```

**8. Client settings, wire route, and end-to-end act additions**

**Project settings.**

The client’s owner document is `jig/crates/jig-client-domain/src/contract/owner_project.rs:12`:

```rust
pub struct OwnerProject {
    pub acts: List<Offered>,
    pub number: u32,
    pub name: DocumentText,
    pub resources: List<OwnerProjectResource>,
    pub policy: OwnerProjectPolicy,
    pub members: List<OwnerProjectMember>,
    pub current_period: Option<PoolLedger>,
    pub pools: List<PoolLedger>,
    // configured funding, spend, recurring and cut facts
}
```

Relevant view code:

- `jig/crates/jig-client-view/src/page/project.rs:34`: `ProjectPage`, including `owner`.
- `:192`: `parts(doc: &Project, section: ProjectSection, limit: u32) -> List<Part>`.
- `:198`: `ProjectSection::Settings`.
- `jig/crates/jig-client-view/src/sections/project.rs:144`: settings rendering.
- `jig/crates/jig-client-view/src/sections/project/owner.rs:542`: owner settings.
- `jig/crates/jig-client-view/src/page/project/owner_form.rs:19`: `supported(doc: &OwnerProject, ask: &Ask) -> bool`.
- `:144`: `baseline(doc: &OwnerProject, ask: &Ask) -> Option<Ask>`.

Delete action, deletion confirmation state, and exact-name confirmation for a project: **not found** at this revision.

**Account removal over the client wire.**

The client does not send deployment `Act::RemoveAccount`. It sends:

```rust
// jig/crates/jig-client-domain/src/contract/asks.rs:265
Ask::AccountChange {
    holder: AccountHolder,
    change: AccountChange,
}

// :365
AccountChange::Remove { account: u32 }
```

The route is:

1. `jig-client-domain/src/requests.rs:46`, `Domain::send`, enters the request table.
2. `:192` / `:207`: table flush emits `Down::Send`.
3. `jig/crates/jig-client/src/client.rs:616`: maps that to `link::Event::Send`.
4. `jig-client-protocol-link/src/translate/asks.rs:686`: translates `Ask::AccountChange`.
5. `:771`: builds wire `AccountChange::Remove`.
6. `jig-client-protocol-link/src/component.rs:418`:

   ```rust
   fn keyed(
       key: RequestKey,
       ask: Ask,
       value: Option<Secret>,
       limits: &w::Limits,
   ) -> Result<w::Keyed, w::Problem>
   ```

7. `component.rs:173` encodes the envelope; `:176` emits `Message::Request`.
8. `jig/crates/jig-client-protocol-http/src/call.rs:19`:

   ```rust
   pub fn exchange(
       mapping: &Mapping,
       credential: Option<&Secret>,
       message: Message,
   ) -> Call
   ```

   `Message::Request(body)` maps to `POST /v1/request` (`:22`, `:31`), under the application’s configured prefix.

9. `jig/crates/jig-protocol-engine-clients/src/request.rs:5`, `Component::request`, decodes `w::Keyed`; `:91`, `jig_request`, decodes/translates its jig request.
10. `translate/asks.rs:604`: wire `AccountChange::Remove` becomes people’s `AccountChange::Remove`.

Current client wire version is `19` (`jig/crates/jig-client-protocol-wire/src/lib.rs:12`).

**New RemoveProject act: concrete end-to-end surfaces.**

It needs:

- Party request, owner authorization/offer, outcome and keyed replay behavior.
- Deployment `Act`, `ActKind`, `ActKind::ALL`, config string and grant tests.
- Core routing and project-closed admission checks.
- Durable removal continuation plus restore/loading/counting/join handling.
- Full erasure orchestration and transcript-drop bridge.
- Connector release/settings notifications and their application translations.
- Client `Ask`, offered act/outcome, request retention/store translation.
- Wire records/variants, codecs, version plumbing, generated code and goldens.
- Project settings action and name confirmation.
- Test roots, scripted engine, worlds, referees, and application exhaustive matches.

The current store codec uses people v13, tasks v9 and core v6 (`jig/crates/jig-protocol-store/src/codec.rs:17`). Current schemas containing the analogue are:

```text
jig/crates/jig-protocol-store/schema/people-v13.schema:163   AskMakeProject
jig/crates/jig-protocol-store/schema/people-v13.schema:190   Ask
jig/crates/jig-protocol-store/schema/people-v13.schema:282   OutcomeProjectMade
jig/crates/jig-protocol-store/schema/people-v13.schema:307   Act
jig/crates/jig-client-protocol-wire/schema/client-v19.schema:884   MakeProject
jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema:83   Ask
```

**Recent addition from history.** Two ancestor commits are directly applicable:

- `5690ec233e`: “Rename and MakeProject in the engine … (people v12, client wire v16)”.
- `0d2aa85e88`: “Rename and MakeProject at the client: client wire v16”.

They predate moving jig beneath `jig/`; their historical `crates/...`, `testing/...`, `tests/...`, and `examples/...` paths now have that prefix. The engine change touched authorization/offers, core routing/numbers/policies, people request/outcome/act handling, engine-client translation, store schema/codecs/version metadata, test roots, and independent observations. The client change touched contract/request retention, wire schema/generated code, link translations, view forms/outcomes/confirmation, scripted-engine behavior, HTTP version plumbing, and client worlds.

The following current direct-reference register gives the concrete request/outcome addition surface, including tests and application roots. It includes constructors and exhaustive handlers of `MakeProject` / `ProjectMade`; adding removal will also require its new lifecycle-specific files.

```text
jig/crates/jig-domain-people/src/
  boundary.rs:296 batch.rs:228 capacity.rs:73 domain.rs:717
  models.rs:123 words.rs:49
  domain/acts.rs:46 domain/start.rs:105 domain/start/fit.rs:192
  tests/projects.rs:11 tests/act_charges.rs:33

jig/crates/jig-domain-core/src/
  projects.rs:33 roles.rs:139 person_task.rs:29 projections.rs:357
  acts/requests.rs:31 settings/held.rs:307 tests.rs:2344
jig/crates/jig-domain-core/src/routing/
  projects.rs:36 people.rs:209 children.rs:25 tasks.rs:1166
  vocabulary.rs:29 application_routes.rs:251 watch_marks.rs:544
  deployment_acts.rs:22 party_controls.rs:212 mod.rs:368 views.rs:644
jig/crates/jig-domain-core/src/tests/
  projects.rs:39 own_watches.rs:31 completion_retry.rs:66
  account_signing.rs:75 activity.rs:46 acts.rs:335
  project_period_watches.rs:68 application_documents.rs:174
  start.rs:775 engine_timers.rs:65

jig/crates/jig-protocol-config/src/
  acts.rs:64 domain.rs:642 tests/acts.rs:14 tests/seed_policy.rs:1
jig/crates/jig-protocol-engine-clients/src/
  request.rs:177
jig/crates/jig-protocol-engine-clients/src/translate/
  asks.rs:135 llm.rs:220 answers.rs:218
jig/crates/jig-protocol-engine-clients/src/tests/
  asks.rs:292 answers.rs:388

jig/crates/jig-protocol-store/src/
  codec/people.rs:95 codec/people/asks.rs:472
  codec/people/acts.rs:6 codec/people/outcomes.rs:154 tests/codec.rs:1348

jig/crates/jig-client-domain/src/
  contract/asks.rs:249 contract/owner_outcome.rs:146 requests.rs:275
jig/crates/jig-client-protocol-link/src/translate/
  asks.rs:702 store.rs:184 compound.rs:65 answers.rs:92
jig/crates/jig-client-protocol-link/src/tests/
  projects.rs:10 fixtures/link_store.rs:47
jig/crates/jig-client-view/src/
  page/project/owner_form.rs:49 sections/project/owner.rs:89
  activity/sending.rs:52 outcome.rs:52 owner_outcome.rs:65
  owner_outcome_words.rs:62 tests/owner_outcome.rs:483
jig/crates/jig-client/src/thread/
  model.rs:286

jig/testing/jig-client-test-kit/src/
  check.rs:241
jig/testing/jig-client-scripted-engine/src/
  requests.rs:349 requests/projects.rs:15 truth.rs:14 payload.rs:235
  payload/owner_outcome.rs:65 payload/owner_outcome/tests.rs:38 tests/projects.rs:11
jig/testing/jig-test-domain/src/
  lib.rs:69 link.rs:304 client_documents.rs:110 routes.rs:109
jig/testing/jig-test-service/src/
  route.rs:35 clients/translation.rs:114

jig/examples/ops/domain/src/
  boundary.rs:148 client_documents.rs:165 route.rs:251
  tests.rs:287 tests/host_tools.rs:77

jig/tests/client/src/
  observe.rs:111
jig/tests/client/src/harness/
  referee.rs:553 referee/offers.rs:168
jig/tests/client/src/joint/
  session.rs:606 session/tests/project_names.rs:157
  wire/asks.rs:147 wire/outcomes.rs:70 wire/llm.rs:201 wire/asks/compound.rs:64
jig/tests/client/tests/stories/
  projects.rs:42
jig/tests/client-protocol/src/
  engine.rs:343 front/observe.rs:129
jig/tests/client-wire/tests/suite/fuzzy/
  client_wire.rs:342

jig/tests/core/src/
  observations/projects.rs:31 effects/deliveries.rs:28 client_peer.rs:127
  peers.rs:231 referee/offers.rs:166 watches.rs:372 observations.rs:1163
  world/deliveries.rs:210
jig/tests/core/tests/suite/
  settings.rs:25 effects.rs:336 fuzzy/settings.rs:196
  route_orders.rs:15 projects.rs:99 deployment_acts.rs:231 effect_fences.rs:596
jig/tests/people/
  tests/suite/acts.rs:21 src/world.rs:640
jig/tests/conformance/src/
  lib.rs:645 scenarios/queries.rs:166
jig/tests/ops/src/
  direct.rs:214 joint/bytes.rs:648 joint/engine.rs:294
  conformance/turn_backoff.rs:183 conformance/mod.rs:355

temper/crates/temper-engine-domain/src/
  boundary.rs:813 domain.rs:361 decision.rs:58 route.rs:290 store.rs:720
  route/demo.rs:342 tests/setup.rs:484
temper/crates/temper-client-domain/src/
  forge/forges.rs:295 forge/start.rs:18 tests/connect.rs:664 tests/admin.rs:237
temper/crates/temper-client-view/src/
  cards.rs:152 tests/connect_forge.rs:770
temper/crates/temper-client-tools/src/
  jig/render/outcome.rs:99 jig/render/document/act_answers.rs:252
  tests/forges.rs:1033 tests/forge_connection.rs:499
temper/crates/temper-service/src/
  route.rs:308 clients/translation.rs:141 tests/welcome_policy.rs:227
temper/crates/temper/tests/support/
  case/project.rs:1 case.rs:416 ctl.rs:865 ctl/member.rs:104

temper/tests/client/
  tests/suite/connect_forge.rs:174 src/front.rs:94
  src/translate.rs:331 src/translate/forge.rs:454
temper/tests/system/src/
  composed.rs:894
temper/tests/engine/domain/
  tests/suite/settings.rs:156 src/walking.rs:923 src/escalation.rs:815
temper/tools/gallery/src/
  worlds.rs:2609
```

Historical schema/generated readers also contain these variants:

```text
jig/crates/jig-protocol-store/
  schema/people-v12.schema:144
  src/generated/people_v12.rs:984
  src/generated/people_v13.rs:1078
jig/crates/jig-client-protocol-wire/
  schema/client-v16.schema:368 schema/client-v16-part1.schema:60
  schema/client-v17.schema:368 schema/client-v17-part1.schema:63
  schema/client-v18.schema:368 schema/client-v18-part1.schema:63
  schema/client-v19.schema:391 schema/client-v19-part1.schema:83
  src/generated/v16.rs:2642 src/generated/v17.rs:2644
  src/generated/v18.rs:2648 src/generated/v19.rs:2776
```

The reverse-reference inventory above is concrete but **does not fulfill an exhaustive every-reader/constructor inventory for the broad `TaskRecord`, funding, notes, account-use, settings, and enclosing store enums**. Those carriers have additional consumers beyond the listed removal/API surfaces.