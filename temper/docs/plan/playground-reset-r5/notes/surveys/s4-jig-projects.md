The draft builds on working cancellation and store primitives, but **three prerequisites do not exist yet**: a whole-task eraser, durable project-deletion work, and connector notifications for adoption release or scope erasure. Retention at a horizon is itself unfinished.

References are repository-relative. In the compact reader inventories, a stated directory prefixes every following filename; the directory also identifies the crate. Line lists identify separate matches in that file. No builds, Cargo commands, branch reads, or writes were performed.

**1. Projects, creation, and store families**

- `jig-protocol-config` — `jig/crates/jig-protocol-config/src/projects.rs:11`, `Project`: configured seed, not a durable runtime project record; `:21`, `Policy`; `:135`, `projects`.
- `jig-domain-core` — `jig/crates/jig-domain-core/src/lib.rs:130`, `ConfiguredPolicy`; `:138`, `SeedProject`; `:362`, `Core::projects`; `:369`, `Core::names`. Runtime identity, name, policy, permissions and template overrides are held separately.
- `jig-domain-people` — `jig/crates/jig-domain-people/src/boundary.rs:771`, `Stored::Policy { project, name, value }`, keyed by `Key::Policy` at `:708`: **this is the durable project/name/policy copy**. There is no separate durable `Project` row or project lifecycle/status field.
- `jig-domain-people` — `jig/crates/jig-domain-people/src/policy_value.rs:28`, `PolicyValue`: copied mutable project policy; `PolicyChange` at `:13`.
- `jig-domain-core` — `jig/crates/jig-domain-core/src/projects.rs:33`, `Core::make_project`: checks deployment ownership, name uniqueness and room; copies a configured policy; allocates identity; installs policy and maps. `:15`, `Core::rename`.
- `jig-domain-core` — `jig/crates/jig-domain-core/src/routing/projects.rs:36`, `make_project_request`: saves policy, installs maker’s Owner holding, opens period 1, emits `Request::ProjectMade`, answers. `:78`, `make_project_act`: additionally verifies maker is deployment Owner.
- Creation entrances: `jig-domain-people`, `jig/crates/jig-domain-people/src/boundary.rs:296`, `Ask::MakeProject`; `:645`, `Act::MakeProject`; `:534`, `Outcome::ProjectMade`.
- First-start creation: `jig-domain-core`, `jig/crates/jig-domain-core/src/routing/first_start.rs:11`, `seed`; `:64`, `seed_projects`; `:74`, `project_rows`; `:87`, `start_seed`. Seeds are consumed; existing stores restore projects from rows.
- Restoration: `jig-domain-core`, `jig/crates/jig-domain-core/src/policy.rs:452`, `Core::restore_policy`; dispatch in `jig/crates/jig-domain-core/src/routing/people.rs:162`, `Stored::Policy`.
- Never-reused identity: `jig-domain-core`, `jig/crates/jig-domain-core/src/numbers.rs:7`, `Deployment`; `:35`, `Deployment::projects`; `:44`, `Family::Project`; allocation in `projects.rs:63`.
- In-memory policy removal already exists: `jig-domain-authority`, `jig/crates/jig-domain-authority/src/domain.rs:45`, `Event::Dropped`; `:81`, `PolicyFact::Dropped`. This removes authority’s policy only; it is not project deletion.

Store ownership is specified at `jig/docs/design/protocol/store.md:334`, §4.4. Concrete families and project references:

- **People / policy:** `jig-domain-people`, `boundary.rs:771`, `Stored::Policy`; `:764`, `Stored::Roles`; `:166`, `Holding`. Roles are keyed by project; identities are deployment-wide.
- **People / parties:** same crate, `boundary.rs:742`, `Stored::Person`; `:755`, `Stored::SignIn`. Neither row has a project field. A party can belong to several projects; deleting a project cannot simply erase every identity in its roster.
- **People / retained requests:** same crate, `boundary.rs:773`, `Stored::Answer`; `:740`, `Stored::Acted`. Original asks, acts and outcomes can retain project/task numbers.
- **People / read positions:** same crate, `boundary.rs:749`, `Stored::ReadResults`; `:751`, `ReadConversation`; `:753`, `EndedRead`. Conversation keys name task, not project.
- **People / inbox:** same crate, `boundary.rs:41`, `Entry { task, project, whom, kind, at }`; `:16`, `Whom::Role { project, role }`; `results.rs:7`, `ResultRef`. Inbox/census projections are volatile, derived from durable task/result/update rows.
- **Tasks / task records:** `jig-domain-tasks`, `jig/crates/jig-domain-tasks/src/boundary.rs:1006`, `TaskRecord`; `:1179`, `Stored`, including `Live`, `Ended`, `Stub`, histories, milestones, activity, proposals, ledgers and indexes.
- **Tasks / creators:** same crate, `boundary.rs:904`, `New`; `:265`, `Party`, including `Deployment { project }`.
- **Tasks / proposals:** same crate, `proposals.rs:141`, `Proposal`; `:153`, `PersonProposal { project, ... }`; `ProposalAction` at `:86` can contain proposed `New` records and task references.
- **Tasks / placed holds:** same crate, `placed.rs:31`, `PlacedHold { project, ... }`; `:25`, `Scope::Project` exists as a type, but project-scope placed holds remain unbuilt (`jig/docs/design/domain/tasks.status.md:7`).
- **Tasks / financial identities:** same crate, `value.rs:190`, `Funder`: `Pool { project, person, period }`, `Recurring { project, task, period }`, `Period { project, period }`, and `Task`.
- **Tasks / spend and pools:** same crate, `funders.rs:330`, `FundingRecord`; `value.rs:174`, `Numbers`; `boundary.rs:1199`, `Stored::ClosedLedger`; `:1215`, `Stored::Ledger`. `FundingRecord::parent` can itself name a project-period funder.
- **Tasks / resource pools:** same crate, `boundary.rs:880`, `PoolSlots`; `:1219`, `Stored::Pool`. These are resource-capacity slots, distinct from financial party pools.
- **Tasks / ended project index:** same crate, `index.rs:225`, `EndedByProject`; `:124`, `Key::EndedByProject`. Other task-related indexes: `Update` at `:42`, `ResultRow` at `:164`, `Edge` at `:197`, `EndedByParty` at `:204`.
- **Tasks / indirect remains:** same crate, `control.rs:108`, `History`; `:161`, `Milestone`; `activity.rs:10`, `ActivityLine`: task-keyed rows can retain task/party/funder references without a top-level project field.
- **Core / deployment:** `jig-domain-core`, `stored.rs:249`, `CoreRecord`; `CoreRecord::Deployment` keeps the high-water mark, which must survive deletion.
- **Core / task-owned remains:** same crate, `stored.rs:240`, `CallRecord`; `:738`, `TurnRecord`; `:763`, `TerminalRecord`; `:796`, `RunProof`; `:845`, `EscalationDecisionRecord`; `:867`, `ProposalDecisionRecord`.
- **Core / goal projections:** same crate, `projections.rs:21`, `ProjectionGoal { project, ... }`; `:91`, `ProjectionRecord`. Cleanup is `projections.rs:390`, `forget`, emitting `Ask::ForgetProjection`.
- **Core / settings:** same crate, `settings.rs:116`, `settings::Key`; `:296`, `settings::Record`; `:19`, `Value`; `:49`, `Item`. Current code supports only `Setting::{Charter,Template,Policy,Rules}` (`jig-domain-people`, `settings.rs:3`).
- **Project-scoped application settings:** designed in `store.md:349` and `domain/connectors.md:846`, **not implemented in jig’s current settings vocabulary**. `jig-protocol-store`, `keys/settings.rs:59`, `setting_write`, always writes scope zero; `:102`, `setting_read`, rejects nonzero scope.
- **Adoptions:** no jig-owned durable adoption row/family. Jig carries `jig-domain-people::Adoption` (`boundary.rs:184`) and receives connector resource facts (`jig-domain-core`, `connector.rs:118`, `Event::Resource`). The connector owns durable adoption records.
- **Notes:** `jig-domain-notes`, `jig/crates/jig-domain-notes/src/boundary.rs:24`, `Scope::{Project,Goal,Resources}` all name projects; `:46`, `Entry`; `:71`, `Line`; `:89`, `Record::{Entry,Line}`. These are additional project-bearing rows omitted from draft §5’s explicit cleanup list.
- **Accounts:** `jig-domain-accounts`, `jig/crates/jig-domain-accounts/src/records.rs:19`, `Use::Project`; `:52`, `Stored::Account { uses, ... }`. Project deletion needs a rule for explicit account uses; current code has no project-removal cleanup.
- **Component / transcripts:** `jig-protocol-store`, `jig/crates/jig-protocol-store/src/keys/own.rs:5`, `Own::{Transcript,Attempt}` name task; transcript bodies can retain further references.

Physical keys/codecs:

- `jig-protocol-store`, `jig/crates/jig-protocol-store/src/keys.rs:52`, `encode`: core/tasks/people/notes/accounts owner tags.
- Same crate, `keys/people.rs:4`, `write`; `:52`, `read`: Roles tag 4, Policy tag 5, Answer tag 6, EndedRead tag 7, Acted tag 8.
- Same crate, `keys/tasks.rs:4`, `write`; `:107`, `read`; `:162`, `funder`: live/history/ended/index/funding addressing.
- Same crate, `codec.rs:16`, `Limits`: current codecs are core v6, tasks v9, people v13, notes v1, settings v1; `:47`, `encode`; `:63`, `decode`.
- Same crate, `codec/people.rs:153`, `encode`; `:218`, `decode`; policy/roles translations at `:177`, `:181`, `:243`, `:258`.
- Same crate, `schema/people-v13.schema:307`, `Act`; generated `src/generated/people_v13.rs:9443`, `StoredPolicy`; `:9360`, `StoredRoles`; `:13432`, `StoredActed`.

Current spend behavior differs from the draft’s proposed preservation:

- `jig-domain-core`, `funding.rs:97`, `Core::period`, derives current period from **project-keyed** ledgers; `:116`, `pool_budget`, uses project membership/policy.
- `jig-domain-tasks`, `retire.rs:10`, `retire`, closes old pools/periods into `Stored::ClosedLedger`; `:65`, `close`.
- There is **no independent durable deployment/party-period aggregate replacing those project-keyed ledger references**. Preserving spend while requiring “no row names the project” needs an accounting representation/exception specified explicitly.

Direct policy/roster readers, beyond definitions and creation above (`Stored::{Policy,Roles}`, `Key::{Policy,Roles}`):

- `jig-domain-core`, prefix `jig/crates/jig-domain-core/src/`: `routing/people.rs:162,163`; `routing/parties.rs:421`; `routing/application_routes.rs:272,274`; `routing/watch_marks.rs:326,332`; `restart/census.rs:70,71`.
- `jig-domain-people`, prefix `jig/crates/jig-domain-people/src/`: `domain.rs:630,850,908,995,1119,2035,2038,2076,2080,2118`; `domain/start.rs:82,212,215,245,261,319,357`.
- `jig-test-domain`: `jig/testing/jig-test-domain/src/lib.rs:1191`; `jig-test-service`: `jig/testing/jig-test-service/src/clients.rs:538,539`.
- `jig-test-store`: `jig/testing/jig-test-store/src/tests/fixtures.rs:299`.
- `jig-ops-domain`: `jig/examples/ops/domain/src/restart.rs:190`; `src/tests.rs:546`.
- `jig-core-world`, prefix `jig/tests/core/`: `src/effects/bootstrap.rs:30,32`; `src/effects/store.rs:118`; `src/watches.rs:261,271`; `src/fixtures.rs:371,373,522,523,671,692`; `src/observations.rs:874`.
- Same crate/tests: `tests/settings/support.rs:212,213`; `tests/suite/first_start.rs:64,65,70,112,169,170,175,177,476`; `settings.rs:27,28,279,280,292,332,335`; `route_orders.rs:35,40,213`; `deployment_acts.rs:339,438,439`; `funding.rs:275,276,344,375`; `start.rs:203,204`; `switch_step.rs:277`; `store_adapter.rs:35`.
- `jig-domain-core` unit readers, prefix `jig/crates/jig-domain-core/src/`: `tests.rs:536,1664,1769,2076,2080,3065`; `tests/coverage.rs:56`; `run_choice.rs:617`; `deployment_connectors.rs:34`; `restore_admission.rs:176,177,355`; `acts.rs:270`; `accounts_document.rs:73,542`; `inbox_selection.rs:39`; `restart_checks.rs:242`; `frame_deployment_role.rs:23,27,33,135,149,161`.
- `jig-domain-people` unit readers, prefix `jig/crates/jig-domain-people/src/`: `tests.rs:441,610,612,707,845,947,961,1079,1227,1229,1246,1254`; `tests/start.rs:536,568,646,829`.
- `jig-people-world`, prefix `jig/tests/people/`: `src/world.rs:503,506`; `tests/suite/fuzzy/simulation.rs:50`; `restore_open.rs:178`; `start_restore.rs:183`.
- `jig-client-world`: `jig/tests/client/src/joint/deployment_story.rs:115`; `src/joint/wire/cursor.rs:731,735,753,754`.
- `jig-channel-world`: `jig/tests/channel/src/fixtures.rs:620,621`; `jig-store-world`: `jig/tests/store/tests/suite/joins.rs:265`; `jig-store-codecs-world`: `jig/tests/store-codecs/tests/suite/profiles.rs:108`.
- `jig-ops-world`: `jig/tests/ops/src/conformance/mod.rs:260,1133,1134`.
- `jig-protocol-config` tests: `jig/crates/jig-protocol-config/src/tests/deployment_owners.rs:138,199,212`.
- `jig-protocol-store` tests, prefix `jig/crates/jig-protocol-store/src/`: `tests/codec.rs:589,605,614,622,634,670,1054,1060,1316`; `tests/codec/holds.rs:416`; `tests/fixtures/keys.rs:66,67`.
- Temper readers are collected under question 7. No readers of these jig project records were found in skein or smith.

**2. Task erasure and retention**

- Contract: `jig/docs/design/domain/tasks.md:198`, `Kept` versus `ForgottenOnceRead`; `jig/docs/design/protocol/store.md:2060`, §11.1.
- **Horizon shrinking, summaries and the later remains sweep are not implemented.** Explicit status: `jig/docs/design/protocol/store.status.md:6`, retention “Not yet”; configuration horizons likewise `jig/docs/design/protocol/configuration.status.md:6`.
- Consequently, **there is no function that erases a task whole, including all rows and transcript**, and no existing “erase this one task at once” domain entry point.
- Retention types: `jig-domain-tasks`, `jig/crates/jig-domain-tasks/src/flags.rs:21`, `Retention`; `jig-domain-people`, `jig/crates/jig-domain-people/src/batch.rs:117`, `Retention`.
- Existing partial eraser: `jig-domain-core`, `jig/crates/jig-domain-core/src/forget.rs:16`, `Owner`; `:31`, `Owner::of_row`; `:76`, `forgets`; `:88`, `forget`.
- `forget` erases `Ended`, result update, party index, result census row, optional project index; asks people to forget the read position. It does **not** erase history, activity, milestones, terminals, calls, delegate edges or transcript.
- It requires the forgotten-once-read class, a person requester, an ended result and completed reading; live references bar forgetting. It is not a general deletion primitive.
- People’s part: `jig-domain-people`, `jig/crates/jig-domain-people/src/inbox.rs:861`, `forget_event`; `:600`, `forget_chat`; entrance `boundary.rs:828`, `Event::Forget`.
- Core routes/readers of that mechanism: `jig-domain-core`, prefix `jig/crates/jig-domain-core/src/`: `routing/people.rs:37,428,435,437,445`; `routing/mod.rs:588`; `routing/watch_marks.rs:385`; `inbox.rs:429`; `ended_marks.rs:12,41,43,51`; `stopping.rs:125,155`; `tests/forget.rs:2`.
- A **single-task transcript drop does exist**: `jig-protocol-store`, `jig/crates/jig-protocol-store/src/boundary.rs:57`, `Write::DropTranscript { task }`.
- Implementation: same crate, `commits.rs:33`, `operations`, handles drop at `:57`; `:141`, `operation`, erases transcript place at `:154`; `transcripts/drop.rs:145`, `begin`, processes one selected task.
- File removal: same crate, `transcripts/drop.rs:179`, `arrived`: removes file, syncs directory, closes directory; missing file is accepted. Attempt-place cleanup is deferred via `:71`, `queue`; `:94`, `erases`; restart recovery `:120`, `restore`.
- **It can be requested immediately for one task at the store boundary**, but that does not erase the task and is not exposed in `jig-domain-core::Write`.
- Every direct `DropTranscript` reader: `jig-protocol-store`, `commits.rs:57,62,154,184`; `src/tests/transcripts.rs:1090`; `jig-store-world`, `jig/tests/store/src/model.rs:56`; `tests/suite/crashes.rs:141`; `tests/suite/pending_switch.rs:93`.
- Existing tests: `jig-protocol-store`, `src/tests/transcripts.rs:1118`, `a_dropped_transcript_is_removed_before_its_commit`; `:1128`, missing-file case; `:1145`, orphan-place case; `:1176`, failure case. `jig-store-world`, `tests/suite/crashes.rs:138`, crash persistence.

**3. Cancellation, conversation close, deployment compound**

- Design: `jig/docs/design/domain/tasks.md:1082`, tree cancellation; `:1089`, grace; `:1163`, compound.
- Party compound: `jig-domain-people`, `jig/crates/jig-domain-people/src/boundary.rs:234`, `Ask::Compound`; `compound.rs:8`, `Operation`; cancellation carries `task`, `reason`, optional grace.
- Deployment close: same crate, `boundary.rs:650`, `Act::Close { project, task, grace, reason }`.
- `jig-domain-core`, `jig/crates/jig-domain-core/src/routing/deployment_acts.rs:49`, `route`; `:104`, `close`: only accepts a live person-requested conversation in the named project.
- `close` constructs immediate cancels for direct delegates, then a cancel with the supplied grace for the conversation; numbers finish messages; sends `tasks::Event::Compound` as `Party::Deployment`.
- **No public general deployment `Compound`/`Cancel` act exists.** The deployed implementation is the private compound construction inside `close`; forge-wide cancellation needs this generalized or a new entrance.
- Task compound types: `jig-domain-tasks`, `jig/crates/jig-domain-tasks/src/compound.rs:10`, `Operation`; `:144`, `Compound`; `:154`, `compound`; `:194`, `CompoundContext`; `boundary.rs:1421`, `Event::Compound`.
- Deployment authority is deliberately narrow: same crate, `compound.rs`’s `CompoundAdmission` checks deployment operations as cancels; ordinary `control.rs:749`, `allowed`, otherwise rejects deployment control.
- Cancel preflight/application: same crate, `control.rs:1031`, `Cancel`; `:1038`, `CancelPlan`; `:1061`, `check_cancel`; `:1114`, `apply_cancel`.
- Grace: same crate, `grace.rs:9`, durable `Grace`; `:32`, `check_finish`; `:97`, `apply_finish`; `:152`, `schedule`; `:178`, `fire`; `boundary.rs:83`, `MessageKind::FinishRequested`.
- Tree closure: same crate, `closing.rs:133`, `cancel_tree`; `:190`, `effects_settled`; `:210`, `settled`; `:228`, `progress`; `:306`, `end_task`. It waits for runs, delegates and connector cleanup before ending.
- Existing conformance examples: `jig-conformance-world`, `jig/tests/conformance/src/scenarios/lifecycle.rs:73`, `compound`; `:155`, `check_ending`; selected-cut tests in `jig/tests/conformance/tests/suite/scenarios.rs:1`.
- Project deletion must cancel **all project roots**, including non-conversations, wait for their endings, and then erase them. Calling existing `Act::Close` for every task will not do that.

**4. Act definition end to end; `RemoveAccount`; files for `RemoveProject`**

There are two vocabularies: client asks and connector-made deployment acts. Deployment acts have no client-network `Act` enum; their wire representation is the retained store codec.

`RemoveAccount`’s current route:

- `jig-domain-people`, `jig/crates/jig-domain-people/src/boundary.rs:662`, `Act::RemoveAccount`; `:676`, `ActKind::RemoveAccount`; `:637`, connector-scoped `ActKey`; `:688`, `ActGrant`.
- `jig-protocol-config`, `jig/crates/jig-protocol-config/src/acts.rs:8`, `parse`; `:43`, `grant`; `:62`, `"remove_account"` → `ActKind::RemoveAccount`. `engine.acts` is `Engine::acts` in `src/lib.rs:58`.
- `jig-domain-core`, `jig/crates/jig-domain-core/src/routing/vocabulary.rs:427`, `Event::Act`; `routing/deployment_acts.rs:10`, `start`, validates carried-value shape, then enters people.
- `jig-domain-people`, `jig/crates/jig-domain-people/src/domain/acts.rs:67`, `admit`: readiness, retained replay/key conflict, grant, configured first owner, answer room and subject checks. `:55`, `kind`, selects required grant.
- **Deployment acts use grants and subject checks, not the asking party’s role.** See `jig/docs/design/domain/people.md:544`, §5.3.
- People emits `Request::Act`; core dispatches `RemoveAccount` in `routing/deployment_acts.rs:71`; `:88`, `account`, fixes holder to Deployment.
- `jig-domain-core`, `routing/key_routes.rs:62`, `admitted`; `routing/account_requests.rs:88`, `change`, maps removal at `:127`.
- `jig-domain-accounts`, `jig/crates/jig-domain-accounts/src/changes.rs:38`, `change`; `:233`, `remove`: validates ownership, erases account/source rows and initiates source cleanup.
- Answer translation: `jig-domain-core`, `routing/account_requests.rs:18`, `outcome`; people retains permanent answer in `domain/acts.rs:170`, `save`; `:155`, `decided`; retryable `Again` is not retained.
- Store wire: `jig-protocol-store`, `schema/people-v13.schema:304`, `ActRemoveAccount`; `:307`, `Act`; `:308`, `StoredActed`; generated `src/generated/people_v13.rs:13268`, `ActRemoveAccount`; `:13386`, `Act`.
- Store translation: same crate, `src/codec/people/acts.rs:4`, `encode`, removal at `:23`; `:33`, `decode`, removal at `:47`.
- Client equivalent is `Ask::AccountChange` with `AccountChange::Remove`, not deployment `Act::RemoveAccount`: `jig-client-protocol-wire`, `schema/client-v19.schema:946`, `AccountChangeRemove`; `:955`, `AskAccountChange`; `jig-protocol-engine-clients`, `src/translate/asks.rs:1`, `ask`.

A new **owner-facing and deployment-facing** `RemoveProject` touches these layers, in order:

1. Contract/docs: `jig/docs/design/domain/people.md:333` and `:544`; `domain/tasks.md:1163`; `protocol/store.md:334,2060`; relevant status files. Specify deletion state, completion answer and retained-answer behavior.
2. Client wire: `jig-client-protocol-wire`, `schema/client-v19-part1.schema:83`, `Ask`; `schema/client-v19.schema:546`, `ActKind`; `:395`, `OwnerOutcome`; regenerate current wire.
3. Client contract: `jig-client-domain`, `src/contract/asks.rs:117`, `Ask`; `src/contract/documents.rs:202`, `ActKind`; `src/contract/owner_outcome.rs:146`, project outcomes.
4. Wire translators: `jig-client-protocol-link`, `src/translate/asks.rs:702`, `store.rs:184`, `answers.rs:92`, `documents_part2.rs:69`, `compound.rs:65,119`.
5. Engine wire: `jig-protocol-engine-clients`, `src/translate/asks.rs:135`, `answers.rs:218`, `task.rs:77`, `llm.rs:220`; `src/request.rs:177,325,376,449`.
6. Domain vocabulary: `jig-domain-people`, `src/boundary.rs:228`, `Ask`; `:396`, `Outcome`; `:644`, `Act`; `:673`, `ActKind`; `:683`, `ActKind::ALL`; add durable deletion record/key, with ownership decided in jig.
7. Admission/authority: `jig-domain-people`, `src/domain.rs:1162`, `project`; `:1289`, `valid_ask`; `:1474`, `valid_answer_shape`; `:1657`, `admit_ask`; `:2353`, `admission`; `src/domain/acts.rs:32,55,67,249`, validation/kind/admission/outcome; `src/domain/start/fit.rs:211`, act restore-fit.
8. Decision: `jig-domain-core`, `src/routing/people.rs:330`, owner dispatcher; `src/routing/deployment_acts.rs:49`, deployment dispatcher; `src/projects.rs:33`, project maps; `src/acts.rs:24`, `Act`; `src/acts/standing.rs:6`, owner check; `src/documents/offers.rs:134`, project offers. Add the deletion state machine and startup resumption here.
9. Cleanup: task/people/core/notes/accounts owners and store/transcript bridge listed in §§1–3; connector asks/events and roots in §§5,7.
10. Persistence: `jig-protocol-store`, `schema/people-v13.schema:190,307,310`, `Ask`, `Act`, `Value`; `src/codec/people/{asks,acts,outcomes}.rs`; keys, family schema and translations for whichever owner keeps deletion progress.
11. Grants: `jig-protocol-config`, `src/acts.rs:59`, kind parser; tests `src/tests/acts.rs:19,66`; temper’s generated initial configuration/reset grants.
12. Client presentation, test kits, conformance adapters and exhaustive readers below.

Additional exhaustive **domain `Ask` readers**, beyond dispatch/serialization above:

- `jig-domain-people`, prefix `jig/crates/jig-domain-people/src/`: `domain.rs:771,824,952,1174,1335,1431,1555,1878,2154,2270,2311,2396`; `capacity.rs:73,126`; `models.rs:123,209,279`; `batch.rs:228`; `words.rs:49`; `domain/start.rs:280`; `domain/start/fit.rs:192`.
- `jig-domain-core`, prefix `jig/crates/jig-domain-core/src/`: `acts/requests.rs:31,170`; `routing/party_controls.rs:212,287`; `roles.rs:139`; `person_task.rs:29,178,218`.
- `jig-domain-core::acts::Act` readers: same prefix, `acts/requests.rs:120`; `acts/objects.rs:45,89`; `acts/objects/task.rs:59,113,251`; `acts/objects/decisions.rs:79,118,183,245`; `acts/standing.rs:14`; `documents/offers.rs:134,426`.
- `jig-people-world`: `jig/tests/people/src/world.rs:640,1041,1068`.
- `jig-client-world`: `jig/tests/client/src/joint/wire/asks.rs:147`; `asks/compound.rs:64`; `llm.rs:201`; `src/observe.rs:111`; `src/harness/referee.rs:553`; `src/harness/referee/offers.rs:168`.
- `jig-client-protocol-world`: `jig/tests/client-protocol/src/front/observe.rs:129`.
- `jig-client-test-kit`: `jig/testing/jig-client-test-kit/src/check.rs:194`, `named`.
- `jig-client-scripted-engine`: `jig/testing/jig-client-scripted-engine/src/requests.rs:349,409,504,704,760`; `src/requests/projects.rs:15,92`.
- `jig-client-domain`: `jig/crates/jig-client-domain/src/requests.rs:275`.
- `jig-client`: `jig/crates/jig-client/src/facts.rs:174`; `src/thread/model.rs:286`.
- `jig-client-view`, prefix `jig/crates/jig-client-view/src/`: `page/project/owner_form.rs:49,237,288,345`; `activity/sending.rs:52`; `sections/project/owner.rs:89,660,748,781,814,1266`; `acts.rs:143,281,314,443`; `confirm.rs:474,587`; `confirmation.rs:322`; `cards/common.rs:40`.
- Outcome readers in same crate: `owner_outcome.rs:65,170`; `owner_outcome_words.rs:62,159,233,330,395`; `outcome.rs:52`.
- Test kit/wire checks: `jig-client-protocol-link`, `src/tests/projects.rs:10,31,47,81`, `tests/fixtures/link_store.rs:47`; `jig-client-view`, `src/tests/domain/drafts.rs:132`, `tests/domain/confirmation/owner_escalation.rs:185`, `tests/owner_outcome.rs:483`.

All direct **deployment `Act` / grant** readers:

- `jig-domain-people`: `src/domain/acts.rs:32,55,249`; `src/domain/start/fit.rs:211`; `src/domain.rs:199,225`; `src/lib.rs:104`.
- `jig-domain-core`: `src/routing/deployment_acts.rs:21,56`; `src/routing/vocabulary.rs:427`; `src/lib.rs:161`.
- `jig-protocol-store`: `src/codec/people/acts.rs:4,33`; `src/tests/codec.rs:1347,1348`.
- `jig-protocol-config`: `src/acts.rs:43`; `src/lib.rs:58`; `src/tests/acts.rs:19,66`.
- `jig-core-world`: `jig/tests/core/src/observations/projects.rs:42`; `tests/suite/deployment_acts.rs:23,42,45,54,127,171,174,190,296,300,369,407,427`.
- `jig-people-world`: `jig/tests/people/src/world.rs:149,150,198,240,470,480`; `tests/suite/acts.rs:14,29,66,67,68,69,134,145,223,225,245,264`; `memory.rs:21,28,30`; `restore_open.rs:117,160`; `start_restore.rs:103`.
- `jig-domain-people` tests: `src/tests/act_charges.rs:33,49,59`; `src/tests/start.rs:422,467,501,838`.
- `jig-ops-domain`: `jig/examples/ops/domain/src/route.rs:1123`; `jig-ops-world`: `jig/tests/ops/src/conformance/config.rs:115`.
- Temper readers: §7. No skein/smith reader of jig’s deployment `Act` or `ActKind`.

**5. Kept multi-step work, adoption release, settings scope erasure**

- **No generic durable multi-step-work row or project-deletion continuation exists in jig.** `Stored::Acted` is a completed answer, not pending workflow state (`jig-domain-people`, `boundary.rs:740`).
- `PendingAct` is volatile: `jig-domain-people`, `domain/acts.rs:8`; startup retries are the connector’s responsibility by design (`jig/docs/design/domain/people.md:598`).
- Existing startup machinery can host a new phase, but does not implement deletion: `jig-domain-core`, `jig/crates/jig-domain-core/src/start.rs:10`, `Phase`; `start/script.rs:7`, `start_done`; `:111`, `start_resume`; `restart.rs:282`, `restart_done`.
- Temper’s concrete pattern: `temper-engine-domain-forge`, `jig`-independent `temper/crates/temper-engine-domain-forge/src/boundary.rs:1637`, `DemoAct`; `:1640`, `DemoActKind`; `domain.rs:125`, `demo_acts`; `:4866`, restoration.
- Same crate, `demo.rs:159`, `acts`, saves before send; `:167`, `send_acts`; `:173`, `act_answered`, erases terminal rows; restart hook `:126`, `restart`.
- **No adoption-release notification exists.** Existing adoption entrance is `jig-domain-core::Ask::Adopt` (`routing/vocabulary.rs:95`), answer `connector::Event::Adopted` (`connector.rs:190`). `connector::Ask` at `connector.rs:249` has no unadopt/release-adoption variant.
- `Ask::Release` (`routing/vocabulary.rs:99`) and `connector::Ask::Release` (`connector.rs:311`) concern a **closing task’s resources**, not release from a project’s adopted-resource set.
- **No settings-scope-erased notification or handler exists.** Application-setting judgment/application is designed at `jig/docs/design/domain/connectors.md:846`; current `Setting` and settings records contain no application kinds/scope.
- Temper kinds 2/4/5 are specified at `temper/docs/design/protocol/configuration.md:1466`, §13.2; kinds 4/5 explicitly say designed, not built. Current stock/configured landing policy is not that durable `Setting` family.
- Existing temper adoption/task-release routing: `temper-engine-domain`, `temper/crates/temper-engine-domain/src/route/asks.rs:16`, `ask`; adoption `:113`; task release `:143`; exhaustive `connector::Ask` translation `route/forge_route.rs:3599`, `effect_ask`.
- Existing durable repository adoption: `temper-engine-domain-forge`, `temper/crates/temper-engine-domain-forge/src/adoption.rs:459`, `finish_adoption`, saves `Stored::Repository` at `:502`. There is no corresponding project-adoption-release handler.

**6. Conformance world, referee, cases, binaries**

- `jig-conformance` — `jig/testing/jig-conformance/src/harness.rs:97`, `Application`; `:157`, `Application::observed`; `:201`, `Harness`; ordinary test-world code, generic over the application.
- Fake durable store: same file `:10`, `Durable`; `:27`, implementation for `jig_fake_store::Store`.
- Driving/faults: same file `:278`, `on_commit`; `:283`, `cut`; `:303`, `crash`; `:325`, `send`; `:334`, `iterate`; `:458`, `drain`; `:538`, `cold_snapshot`.
- `jig-conformance` — `jig/testing/jig-conformance/src/scenarios.rs:138`, `ScenarioApplication::check`; `:249`, `run`, invokes the application’s end check after the program.
- `jig-conformance-world` — `jig/tests/conformance/src/lib.rs:147`, `Testing`; `:242`, `Application for Testing`; `:696`, `observed`, forwards durable/system/worker evidence.
- Adapter/referee inventory: `jig-core-world`, `jig/tests/core/src/observations.rs:259`, `Observer`; `:667`, `submitted`; `:722`, `durable`; row projection at `:769` settings, `:774` holds, `:786` proposals, `:791` tasks, `:827` activity, `:831` history, `:851` writers, `:858` pools, `:861` ledgers, `:874` roles, `:891` read positions.
- Independent project-policy expectations: same crate, `src/observations/projects.rs:11`, `Makes`; `:26`, `inbound`; `:73`, `durable`. It learns made projects from retained answers, not live core state.
- `jig-conformance` — `jig/testing/jig-conformance/src/referee.rs:243`, `Snapshot`; `:294`, `Observed`; `:385`, `Referee`; `:922`, `observe`; `:1121`, `check_at`.
- **Snapshot is not a complete all-family project-reference inventory.** It has tasks/read positions/holds/balances/etc., but not every policy, note, account-use, retained act or connector row. No “project removed; no row names it” referee check exists.
- **No per-family RemoveProject conformance cases exist.** Shared stories are in `jig/testing/jig-conformance/src/scenarios.rs:152`, `program`; application bindings/end checks in `jig/tests/conformance/src/scenarios.rs:108`, `ScenarioApplication for Testing`.
- Closest erasure end-check example: `jig-conformance-world`, `jig/tests/conformance/src/scenarios/conversations.rs:127`, `check_ending`: scans durable keys for owner/index/read-position remains, then values for result rows. This is a concrete model for a deletion-family case, but covers only forgetting.
- Closest cancellation case: same crate, `src/scenarios/lifecycle.rs:73`, `compound`; `:155`, `check_ending`.
- Referee negative tests: same crate, `tests/suite/referee.rs:12`, early/replayed application answers; `:280`, `a_read_position_erased_without_its_task_is_a_violation`.
- Existing **codec family cases**, distinct from conformance stories: `jig-protocol-store`, `src/tests/codec/cases.rs:5`, `people_rows_read_back_as_written`; `:25`, call rows; `:38`, component rows; `:56`, accounts; `:70`, notes; `:103`, task values.
- Example: same crate, `src/tests/codec.rs:588`, `a_projects_policy_reads_back`, registered in `src/tests/codec/cases.rs:8`; funding example `src/tests/codec.rs:818`, `task_ledgers_read_back`.
- Store design text is compiled into tests: same crate, `src/tests/words.rs:11`, `DESIGN = include_str!(...store.md)`.
- Test binaries: `jig-conformance-world` → `suite` (`jig/tests/conformance/tests/suite/main.rs:1`) and standalone `qa_settings_proposals_wire` (`tests/qa_settings_proposals_wire.rs:1`).
- Related binaries: `jig-core-world` → `suite`, `qa_all`; `jig-store-world` → `suite`, `qa_all`; `jig-people-world` → `suite`; `jig-tasks-world` → `suite`; `jig-store-codecs-world` → `suite`. Entrypoints are each package’s `tests/<binary>/main.rs:1`.
- Codec/forget unit tests run in library test binaries `jig_protocol_store` and `jig_domain_core`; no new deletion binary exists.

**7. Temper readers and variants that force changes**

Deployment acts and new client acts/outcomes:

- `temper-engine-domain`, `temper/crates/temper-engine-domain/src/store.rs:780`, exhaustive `people::Act` byte accounting; `:720`, `people::Ask`; `:957`, `people::Outcome`. A new variant requires arms here.
- Same crate, `src/route/demo.rs:285`, `send_end_act`, constructs `people::Act`; `:313`, `end_acted`, exhaustively handles outcomes; `:250`, `list_end_acts`, selects today’s close/remove-account work.
- `temper-service`, `temper/crates/temper-service/src/tests/setup.rs:1693`, deployment grants.
- `temper-engine-domain-world`, `temper/tests/engine/domain/tests/suite/demo.rs:21`, grants; `:509`, `acted`; `:566,569`, close/remove-account outcome checks.
- `temper-client-tools`, prefix `temper/crates/temper-client-tools/src/`: `jig/render/acts.rs:13`, `name`; `:94`, `has_tool`; `:122`, `call`; `jig/render/document/admissions.rs:28`, `ActKind`; `jig/render/outcome.rs:99,100,839,840`, `Ask`; `jig/render/document/act_answers.rs:252`, `OwnerOutcome`.
- `temper-client-view`, `temper/crates/temper-client-view/src/cards.rs:128`, `owner_outcome`, exhaustive project outcome arm at `:152`.
- `temper-client-domain`, `temper/crates/temper-client-domain/src/forge/start.rs:17`, `Start::ask`; `:40`, `Start::answered`, consumes `ProjectMade` before adoption.
- `temper-gallery`, `temper/tools/gallery/src/worlds.rs:2143`, `ActKind`; `:2609`, `OwnerOutcome`.
- Client tests consuming creation/outcomes: `temper-client-domain`, `src/tests/connect.rs:664,692,778,825,944,1226`; `src/tests/admin.rs:237,309,322`; `temper-client-view`, `src/tests/connect_forge.rs:770,1750`; `temper-client-tools`, `src/tests/forges.rs:1034,1044`; `src/tests/forge_connection.rs:500,508,567`.
- Process helpers: crate `temper`, `temper/crates/temper/tests/support/case/project.rs:75,120`, project answer/request; `tests/support/ctl/member.rs:104,161`; `tests/support/case.rs:518`.

Core project events and exhaustive routing, if adding a sibling such as `ProjectRemoved`:

- `temper-engine-domain`, `src/route.rs:290`, core request → `Delivery::ProjectMade`.
- Same crate, `src/decision.rs:58`, `Delivery::ProjectMade`; `:484`, delivery accounting.
- Same crate, `src/boundary.rs:811`, `delivery_output`, exhaustive delivery match.
- Same crate, `src/domain.rs:361`, startup output handling; `:680`, `only_decided`; `:1005`, `released_into`, project event at `:1013`.
- `released_into` starts the project’s calendar after commit and explicitly notes that landing policy, forge judges and gate policy currently contain optional seeded overrides; deletion must clear/update those consumers.
- `temper-service`, `src/route.rs:308`, `root::Delivery`; `src/clients/translation.rs:141`, `root::Delivery`.
- `temper-engine-domain-world`, prefix `temper/tests/engine/domain/`: `src/walking.rs:923`; `src/escalation.rs:815`; `tests/suite/settings.rs:163,322`; `tests/suite/demo.rs:566`.
- `temper-client-world`, `temper/tests/client/src/translate.rs:331`; `src/translate/forge.rs:454`.
- `temper-system-world`, `temper/tests/system/src/composed.rs:894`.
- Unit/integration construction checks: `temper-engine-domain`, `src/tests/setup.rs:489,532,537`; `temper-service`, `src/tests/welcome_policy.rs:227,234`.

Policy/roster readers in temper (`Stored::{Policy,Roles}`, `Key::{Policy,Roles}`), additional to `store.rs`:

- `temper-engine-domain`, `temper/crates/temper-engine-domain/src/restart.rs:505`; `src/tests/resume.rs:273`; `src/tests/setup.rs:47,509`; `src/tests/resume_fixture.rs:528`.
- `temper-engine-domain-world`, prefix `temper/tests/engine/domain/`: `src/roles.rs:190,192,224,226`; `src/escalation_referee.rs:816,817`; `src/roles_referee.rs:113,126,174,176`; `src/forge_routes.rs:1525`; `src/walking/creating.rs:58,59`.
- Same crate/tests: `tests/suite/applications.rs:57,96`; `root_routes.rs:3136,3449,3450,3486,3535,3536,3688,3693`; `settings.rs:324,325`; `roles.rs:120,121,133,166,222,248,274`; `forge_routes/paths.rs:51,52`; `periods.rs:198,200`; `escalation.rs:526`.
- `temper-conformance-world`, `temper/tests/conformance/src/scenarios.rs:545`.
- `temper-client-world`, `temper/tests/client/src/front/accounts.rs:26`.
- `temper-service`, `temper/crates/temper-service/src/tests/setup_reachable.rs:197`; `tests/setup_github.rs:35`; `tests/welcome_policy.rs:147`.

Adoption/connector consumers:

- `temper-engine-domain`, `src/route/asks.rs:16`, `ask`, handles `core::Ask::Adopt` at `:113`, task `Release` at `:143`, `ForgetProjection` at `:238`.
- Same crate, `src/route/forge_route.rs:61`, `adopt_repository_ask`; `:3599`, exhaustive `effect_ask`; `src/route/applications.rs:33`, adoption application route.
- `temper-engine-domain-forge`, `src/adoption.rs:67`, `start`; `:137`, `adopted`; `:167`, `answered`; `:459`, `finish_adoption`; `src/boundary.rs:114`, `Repository` retains project ownership/adoption information.
- No current reader handles **adoption released from project** or **project settings scope erased**, because those variants do not exist. New notifications require implementation in these roots/connectors, not merely additional ignored arms.

Other jig roots/test kits forced by new core request/connector variants:

- `jig-test-domain`, prefix `jig/testing/jig-test-domain/src/`: `routes.rs:19`, `route_core`; `:116`, `route_ask`; `:785`, `route_effect_ask`; `:987`, `application_answer`; `lib.rs:69,1008`, project delivery; `link.rs:304`; `client_documents.rs:110`.
- `jig-test-service`: `jig/testing/jig-test-service/src/route.rs:35`; `src/clients/translation.rs:114`.
- `jig-ops-domain`, prefix `jig/examples/ops/domain/src/`: `boundary.rs:148`; `route.rs:251,323,733,734,902`; `client_documents.rs:165`; `tests.rs:287,335,389,471,508,646`; `tests/host_tools.rs:77`.
- `jig-domain-core` internal request matches, prefix `jig/crates/jig-domain-core/src/`: `routing/mod.rs:368,409`; `routing/children.rs:25,43,456,505,810`; `routing/tasks.rs:1166`; `routing/views.rs:644`; `routing/watch_marks.rs:544`; `settings/held.rs:307`; `projections.rs:357`.
- `jig-core-world`, prefix `jig/tests/core/src/`: `effects/deliveries.rs:28`; `client_peer.rs:127`; `peers.rs:231`; `referee/offers.rs:166`; `watches.rs:372`; `observations.rs:1163`; `world/deliveries.rs:210`.
- Same crate/tests: `jig/tests/core/tests/suite/effects.rs:336`; `deployment_acts.rs:231,388`; `effect_fences.rs:596`.
- `jig-domain-core` unit request matches: `src/tests.rs:2344,2698,2716`; `tests/own_watches.rs:31,309,327`; `completion_retry.rs:66,76,89,142,253`; `account_signing.rs:75`; `activity.rs:46`; `acts.rs:335`; `project_period_watches.rs:68,127,203`; `application_documents.rs:174`; `start.rs:775`; `engine_timers.rs:65`.
- `jig-conformance-world`: `jig/tests/conformance/src/lib.rs:645`; `src/scenarios/queries.rs:166`.
- `jig-client-world`: `jig/tests/client/src/joint/session.rs:606`; `src/joint/wire/outcomes.rs:70`; `src/joint/wire/offers.rs:30`.
- `jig-client-protocol-world`: `jig/tests/client-protocol/src/engine.rs:343`.
- `jig-ops-world`, prefix `jig/tests/ops/src/`: `direct.rs:214`; `joint/bytes.rs:648`; `joint/engine.rs:294,402`; `conformance/turn_backoff.rs:183`; `conformance/mod.rs:355,746`.

The existing public project document types—`jig-domain-core::documents::Project` (`documents/project_note.rs:106`) and `jig-client-domain::contract::Project` (`contract/documents.rs:689`)—are read models, not deletion-progress records. A deleted-project state machine, owner Delete control, terminal deletion answer, startup completion fence, all-family deletion cases, and complete referee erasure inventory remain to be built.