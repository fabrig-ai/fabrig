# Session 14: the wire collapsed: the requests

jig's client wire, `jig-client-protocol-wire` at v8, takes the requests
in general form: the plumbing operations of contract.md, 3.2 replace the
shape asks in place (`Message`, `Move`, `Seen`, `Decide`, `Answer`,
`Cancel` with a grace), `Create`, `Propose`, `Withdraw` and `Compound`
are added, an answer in a version the engine does not serve is typed
per owner, and an application's reads and subjects name their scope and
window at jig's level. The documents (`me`, the task line and task
queries, offers and a decision's effect) are jig 22's, which needs this
session. Each record lands with both ends in one increment: the schema, the
clients' component's translation, `jig-client-domain`'s contract, the
link's translation and their users, run against the real engine. The
shape asks that have no replacement on the client yet (`StartChat`,
`Close`, `SetGoal`, `Prioritise` and the conversation and goal lists)
stay until jig 16.

## Read first

- jig's `docs/design/client/domain/contract.md`, sections 1 to 4, 7.5,
  8 and 9; `docs/design/protocol/client.md`, sections 3, 4.1, 5, 6, 7, 9
  and 11; `docs/design/client/domain/client.md`, sections 4, 6, 7, 8.1
  and 8.2.
- The plan's `decisions.md`, 3 and 9; `findings/wire-and-client.md`,
  sections 1, 5 and 6; `findings/engine-parties-and-tasks.md`, section 2.
- skein's `programming-model.md`, sections 4.4, 6.3 and 8;
  `testing-strategy.md`, sections 2.1, 2.5, 4 and 7.
- skein's `docs/design/codec.md`, section 3 ("A variant holds one
  record", "A sum crosses as one piece", "A type uses only types declared
  before it").

## What the design asks

- **Operations, one record each, by their names** (protocol/client.md,
  3): "`Create`, `Propose`, `Withdraw`, `Message`, `Amend`, `Cancel`,
  `Release`, `Stop`, `Move`, `Decide`, `Answer`, `Take`, `HandBack`,
  `Seen`, and the project's own; and `Compound { project, operations }`".
  What each names is contract.md, 3.2's table; a compound's bounds and
  answer are 3.4; refusals' facts are section 4.
- **In general form:** "No record names a shape of work: a chat is a
  `Create` filled from a template; closing it, a compound request; a list
  of conversations or of goals, a task query whose rows are task lines;
  the charters offered for chats, templates in `me`" (protocol/client.md,
  3).
- **Versions per owner** (section 8; protocol/client.md, 9): "A version
  not served is answered so, typed, before the record is read: *version
  not served*, naming the owner and the ranges the engine serves. A keyed
  request, a read and a watch's opening are each answered so; nothing is
  done." The client keeps that owner's requests unsent; outside jig's
  range it is out of date (client.md, 8.2).
- **An application's scopes and windows** (7.5; protocol/client.md, 4.1
  and 11; client.md, 7): "A family's read names its owner, its scope (a
  project, the deployment, or the party's own) and its record's bytes";
  watches and reads carry a window.

## Choices this plan makes

- **The wire stays at v8,** jig 11's bump, edited in place
  (decisions.md, 9); no increment here moves `VERSION`.
- **The keyed request's answer envelope `Answer` is renamed
  `KeyedAnswer`** (as `ReadAnswer`, `OpenAnswer`, `SignInAnswer` name
  theirs), so that the operation's record can be `Answer`. Its bytes do
  not change.
- **`move` is not a variant name the codec takes** (skein-codegen
  refuses it: "invalid generated variant name move"), so `Move`'s
  variant in `Ask` and `Operation` is `move_task` (and in jig 22's
  `ActKind` and `Effect`), as `OwnerTaskCapacityAction` already names
  it; the record is `Move`.
- **`Decide` and `Answer` hold their target as a record:** a variant
  holds one record (codec.md, 3), so `Decide { project, on: DecideOn }`
  and `Answer { project, on: AnswerOn }`, the targets keeping their
  records less `project`.
- **A compound's operations reuse the operations' records,** each naming
  its project; one naming another project than its compound's is refused
  not well formed at its place, by the translation, before the core.
- **A batch and a compound cross as pieces** of at most 1,048,576 bytes
  each (`Create.tasks`, `ProposeBatch.tasks`, `Compound.operations`), in a
  keyed request of at most 1,114,240 bytes (14.1) (ruling 222): with `list`
  their worst case passes `u32` (skein-codegen: "encoded worst case for
  Create exceeds u32").
- **The scripted engine answers the new operations as not offered**
  (`Refusal::NotOffered`) until jig 16.3 models `Create`, `Propose` and
  `Compound` for the reference UI's chats and goals; no client world
  scenario sends them before then. The renamed operations it models as
  it modelled their old forms.
- **New code goes in new modules** where its file is near or past 800
  lines: the component's `served.rs`, the contract's `operations.rs`,
  the link's `translate/operations.rs` (component.rs is 784 lines at the
  base, the link's `translate/asks.rs` 755).

## Rules that bind it

- **No type beside an existing one:** a record that replaces another
  replaces it in place, with each of its users in the same increment
  (code-organization.md, 6); the edits README's Rules allow outside the
  Files cover those users.
- **The meeting point:** each increment runs its Proof against the real engine,
  in `tests/client-protocol` (the joint world over bytes) or `tests/client`,
  never only against `jig-client-scripted-engine` (README, Rules).
- **jig never names the repository downstream of jig:** code-organization.md, 6's grep for that name prints
  nothing at each increment's end. Its family's refusal translation and
  its engine follow these records at its repin.

## Increments

Each increment regenerates the codec once from the edited schema:
`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world
--test drift`, then runs the drift test without the variable. Schema,
generated code and goldens count no lines (code-organization.md, 7);
the drift test also runs ops' wire (`v1`), so 14.1's edit of ops' schema
rewrites `examples/ops/client-protocol-wire/src/generated/v1.rs` and
`golden/v1`, which are 14.1's (ruling 247);
each increment is a codec increment, landing within 2.5 times its Size
without asking.

### 14.1 The envelope: version not served, and an application's scope and window
Scope: the typed refusal per owner (contract.md, section 8;
protocol/client.md, 9; client.md, 8.2), and the scope and window of an
application's read and subject, and the scope of its keyed request, at
jig's level (contract.md, 3.3 and 7.5; protocol/client.md, 4.1 and 11;
client.md, 7 and 8.1).
Needs: the session's start: jig 03 to 09 and jig 21.2's landing; 14.1 is
built beside jig 21.3 and 21.4 and lands only after jig 21's last landing
(the fence, decisions.md 12; ruling 247), so `jig_client::Client`
with the link's states and per-owner versions from the hello (jig 12),
its family reads and watches as bytes under an owner (jig 12), and
`ops`'s root on it with its families' records in
`jig-ops-client-protocol-wire` (jig 21.1, 21.2); jig 07's party's own scope
(`ApplicationScope`'s third variant) and windows on a connector's
document and read in the core, and jig 07.6's `people::Ask::Application`
carrying an `ApplicationScope` in place of its project.
Files: `crates/jig-client-protocol-wire/schema/client-v8.schema` and
`client-v8-part1.schema` (generated `src/generated/v8.rs`, goldens);
`crates/jig-protocol-engine-clients/src/{component.rs (keyed request,
695-698 at jig `1350b6564`) (ruling 222), reads.rs (58-64), watches.rs (180-190), boundary.rs
(ToApplication, 70-93), served.rs (new), translate/answers.rs,
routes.rs}`; `crates/jig-client-protocol-link/src/` (the answer, read and
opening translations; the link's version state);
`crates/jig-client-domain/src/contract/` (`NotServed`, `Served`, `Owner`,
`Scope`, `Window`); the family record `jig_client_domain::Family {
family, version, bytes }`, which `Ask::Family`, `Readable::Family` and
`Watchable::Family` hold (the family `Send`, `Read` and `Watch` of Shape),
and its users: `crates/jig-client-domain/src/{boundary.rs (:10), domain.rs,
requests.rs, reads.rs}`, `crates/jig-client-protocol-link/src/component.rs`,
`crates/jig-client-protocol-link/src/translate/{reads.rs, watches.rs,
store.rs}`, `crates/jig-client-view/src/bridge.rs`,
`testing/jig-client-test-domain/src/family/{client.rs, wire.rs}`,
`tests/client-protocol/src/family.rs`,
`tests/client/src/{test.rs, scripted.rs, observe.rs}` (ruling 222);
`testing/jig-test-service/src/family.rs` (to_root, 6-69: its `scope()`,
70-82, goes) (ruling 222); `ops`'s client root, which jig 21.2 put on
`jig_client` (`examples/ops/client-domain`), and its wire crate,
`jig-ops-client-protocol-wire` (jig 21.1: `schema/ops-client-v1.schema`
and `src/codec.rs`, v1 edited in place): the scope of its families'
`Subject { scope, kind }` moves out of the subject's record and bytes
into the family `Watch` and `Read`, in the module of
`examples/ops/client-domain/src/` that jig 21.2 adds to map its values to
`Family` (`git grep -n -w Family -- examples/ops` at the increment's start
finds it; no line at jig `1350b6564`) (ruling 222): at jig `b6a63f8c1` it is
`examples/ops/client-domain/src/family.rs` (`owned()` 9-11, `send` 20-37),
with the subject encoders at `wire.rs:46` and `:118` and `wire::ask` at
`:186` (ruling 247); the wire's
users of `Answer`, 7 production files at the base (the link's
`component.rs` and `translate/answers.rs`; the component's
`component.rs`, `limits.rs`, `routes.rs`, `sign_in.rs`,
`translate/answers.rs`); tests: the wire crate's `src/tests.rs`,
`tests/engine-clients/tests/{requests,closing}.rs`, and the users of
`Answer` and of the family record that these missed:
`tests/client-protocol/src/native/wire.rs` (117-122 at jig `b6a63f8c1`, an
exhaustive match on `Answer` at :116; the other `w::Answer::decode` at :149),
`crates/jig-client/src/tests/{fixtures.rs (293), sign_in.rs
(52 at `b6a63f8c1`), store.rs (191)}`, `tests/ops/src/joint/bytes.rs` (58-59
`Owned`, 64 `Keyed`, 96-99 `w::Answer::FinalAnswer`, 104-107
`ReadAnswer::Value` and `.record()`, 113-114 a delivery's
`.document().record()`), `tests/ops/src/client/app.rs` (314-320: `Keyed`,
`keyed.request()` to `ask_value`, which unwraps `FamilyRequest.record`)
(ruling 247), `crates/jig-protocol-engine-clients/src/tests/{asks.rs,
watches.rs, reads.rs}` with their `fixtures/` (`asks.rs:95` and `:155` send
raw bytes, `b"abc"` and `[255]`, as the keyed request, which becomes a
`FamilyRequest` holding them; the digest constant stays) (ruling 222). Shared hotspot:
`crates/jig-protocol-engine-clients/src/translate/`. Refresh any further
copy, and list it in the commit body. (ruling 222)
Shape:
- Wire: `Served { owner, first, last }` moves before the answers;
  `NotServed { owner: Owner, served: list 16 Served }`; `enum KeyedAnswer
  { not_served: NotServed, final_answer: Owned, busy, not_ready,
  signed_out }` (was `Answer`); `ReadAnswer`, `OpenAnswer` gain
  `not_served: NotServed`, `WatchEnd` gains `not_served`, each as its
  first variant, tag 0, so that "the answer *version not served*" is
  "encoded the same in every version of jig's wire" (protocol/client.md,
  9): no later version adds a variant before it, and the goldens the
  drift test writes for `not_served` (`golden/v8/enum_keyed_answer_not_served_*.bin`
  and the three others') pin its bytes; `enum Scope { project: ScopeProject, deployment, own }`,
  `Window { rows: u32, after: option Cursor }`, `FamilyRecord { scope:
  Scope, window: Window, record: bytes 1048576 }`, after `Cursor`. A
  family's read and a family subject's `Owned.record` is a `FamilyRecord`
  whose `record` is the family's bytes in `Owned.version`; and
  `FamilyRequest { scope: Scope, record: bytes 1048576 }`: a family's
  keyed request's `Keyed.request` is a `FamilyRequest` whose `record` is
  the family's bytes, the digest taken over those bytes as they came.
  The keyed request's ceiling rises with them: `Keyed.request`
  (`client-v8.schema`) and `TableEntry.request` (`client-v8-part1.schema`)
  become `bytes 1114240`, since `Create.tasks`, `Compound.operations`,
  `ProposeBatch.tasks` and `FamilyRequest.record` at 1,048,576 encode to
  1,048,584 bytes or more, which `Keyed::new` refuses at 1,048,576, and a
  full `Propose` with its reason is 1,114,126 (ruling 222).
- Engine: the three serve checks answer the typed record (`served.rs`:
  `pub(crate) fn not_served(settings: &Settings, owner: w::Owner, limits:
  &w::Limits) -> w::NotServed`) instead of `respond(400, ..)`;
  `ToApplication::{Read, Watch}` gain `scope:
  people::ApplicationScope` (`core::documents::ApplicationScope` is private,
  `crates/jig-domain-core/src/documents.rs:23`) (ruling 222) and `window:
  core::documents::Window`, decoded from `FamilyRecord` after the serve
  check, and `ToApplication::Request` gains `scope` in place of the
  project its application decoded, from `FamilyRequest`; the test
  service's `to_root` passes them to its root's `ReadApplication`,
  `WatchApplication` and `Application`, whose `people::Ask::Application`
  names the scope (jig 07.4).
- Client: the link translates `KeyedAnswer::NotServed` into the request
  table's held-for-its-owner progress, `ReadAnswer::NotServed` into the
  read's `Page` version not served (jig 12.4's `Readout::NotServed`,
  its `served` the answer's range of that owner), `OpenAnswer::NotServed` and
  `WatchEnd::NotServed` into `Ended { why: version not served }`, and
  each into the link's owners not served (jig's own: out of date), as
  jig 12.4's judgement of a declared owner, until the next hello; the
  family `Watch`, `Read` and `Send` name `scope: Scope` beside their
  bytes (the window is the `Watch`'s).
Test shape:
- `crates/jig-client-protocol-wire/src/tests.rs`: add
  `version_not_served_is_tag_zero_of_each_answer` (the first byte of each
  of the four answers' `not_served` encodings is `0`) and
  `a_family_record_round_trips_with_its_scope_and_window`, each as a row of
  the `wire_values_round_trip_unchanged` table (`tests.rs:321`), the only
  way a function without `#[test]` runs there (ruling 222); and extend
  `each_answer_kind_round_trips_unchanged` (its loop gains
  `KeyedAnswer::NotServed(fixtures::not_served())`, a fixture added to
  `src/tests/fixtures.rs`).
- `crates/jig-protocol-engine-clients/src/tests/answers.rs`, fixtures in
  `src/tests/fixtures.rs` and `fixtures/answers.rs`:
  `a_keyed_request_outside_its_owners_range_is_answered_not_served`,
  `a_read_outside_its_owners_range_names_the_served_ranges`,
  `an_opening_outside_its_owners_range_is_answered_not_served`.
- The link's tests (`src/tests/`, fixtures `src/tests/fixtures.rs`):
  `a_not_served_answer_holds_its_request_for_its_owner`,
  `a_watch_ended_not_served_ends_its_handle_naming_why`.
- `tests/engine-clients/tests/reads.rs`, the world's `Peer`
  (`tests/engine-clients/src/peer.rs`), reference test (a new kind:
  scope and window handed to the application):

  ```rust
  #[test]
  fn a_family_read_hands_its_scope_and_window_to_the_application() {
      let mut settings = fixtures::settings();
      settings.served = List::with_capacity(2);
      settings.served.push(Served { family: None, first: 8, last: 8 }).expect("jig");
      settings.served.push(Served { family: Some(1), first: 1, last: 1 }).expect("family 1");
      let mut peer = Peer::started(&settings);
      let bearer = signed_in(&mut peer);
      let record = fixtures::family_read(w::Scope::Own, fixtures::window(3));
      peer.post(b"/jig/v1/read", record, Some(&bearer));
      let Some(ToApplication::Read { scope, window, family, .. }) = peer.take_application() else {
          panic!("the read reaches the application");
      };
      assert_eq!(family, 1);
      assert_eq!(scope, people::ApplicationScope::Own);
      assert_eq!(window, core::documents::Window { rows: 3, after: None });
  }
  ```

  (`fixtures::family_read`, `fixtures::window` and
  `Peer::take_application` are added to the world's
  `src/fixtures.rs` and `src/peer.rs`; `signed_in` is
  `tests/requests.rs:9`'s, moved to the fixtures; the test imports
  `jig_domain_people as people`, `jig_protocol_engine_clients::{Served,
  ToApplication}` and `skein_lib::List`; `fixtures::settings()`
  (`tests/engine-clients/src/fixtures.rs:18-32`) serves only jig `8..=8`
  in a list of capacity 1, so the test sets capacity 2 and adds family 1's
  `1..=1`; `core::documents::ApplicationScope` is private, and the own
  scope is `people::ApplicationScope::Own`.) (ruling 222)
- `tests/client-protocol/tests/client.rs` (jig 12's story file):
  `a_request_pending_across_a_restart_that_stops_serving_its_family_stays_unsent`
  (jig 12's `Story`, `tests/client-protocol/src/client.rs`, gains
  `restart_serving(served)`: the engine restarts, as the world's
  `FrontFault::Restart` restarts it, with `Settings.served` for family 1
  set to `2..=2`, beside the fixture's `1..=1` at
  `tests/client-protocol/src/fixtures.rs:61-64` (ruling 222), the reason being the
  test; the family's request is sent with `src/family.rs:15`'s `request`
  while its answer is cut, `Story::cut_answer`), and
  `a_family_request_in_the_deployment_scope_reaches_the_people_child_with_it`
  (the test application's request sent in `Scope::Deployment`; the
  story's engine records the people child's admitted ask, and it is
  `people::Ask::Application { scope: ApplicationScope::Deployment, .. }`).
- Limits: the engine's from the world's fixtures
  (`tests/client-protocol/src/fixtures.rs:71-94` (ruling 222), hand-set at the base;
  this plan does not move them). Ceilings: unit under 20 ms, world under
  0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_request_pending_across_a_restart_that_stops_serving_its_family_stays_unsent)
| test(a_family_request_in_the_deployment_scope_reaches_the_people_child_with_it)
| test(=a_person_s_part_request_is_answered_from_the_connector_s_outcome)'`
once: the restarted engine answers the pending family request version
not served, the client keeps it unsent and its `pending(key)` says held
for family 1's version; the deployment-scoped family request reaches the
people child naming that scope; and the existing part-request story
passes unchanged, nothing else of it checked. Bound: 2 s.
Don't: change the hello or `Settings.served`'s shape; let the client
send a held request again before a hello serves its version; derive a
family subject's or a family request's scope in the application's
protocol layer.
Size: at most 300 lines.
Removes: the HTTP 400 answers to an unserved version (three sites); the
test service's `scope()` (family.rs:70-82) (ruling 222).

### 14.2 The operations renamed and merged in place
Scope: `Move`, `Seen` over a conversation's turns, `Decide`, `Answer`,
`Cancel` with a grace, `Amend` naming tracked and priority and, for a
priority alone, no revision (contract.md, 3.2, 3.4 and 5.2); and the
project's own `ChangePolicy` naming the policy's template overrides
(contract.md, 3.2: "the policy, its templates' overrides among it";
3.6).
Needs: the session's start: jig 04's `people::Ask::Cancel` with a
grace and the grace's span limit, `Move`'s check, tracked and priority
in an amendment with an optional revision; jig 08's read positions over
conversations; jig 09.2's `people::TemplateOverride` and
`PolicyChange::Templates { overrides }`.
Files: the v8 schema; `crates/jig-protocol-engine-clients/src/translate/{asks.rs,
answers.rs, refusals.rs}`; `crates/jig-client-domain/src/contract/asks.rs`
(`Ask`, `Amendment`, `Seen`) and `policy_edit.rs` (`PolicyEdit`); `crates/jig-client-protocol-link/src/translate/{asks.rs,
answers.rs}`; users: the reference UI's acts and cards
(`crates/jig-client-view/src/`: take over, mark seen, the proposal,
escalation, question and person task cards, the cancel confirmation),
`testing/jig-client-scripted-engine/src/{requests.rs, payload.rs}`,
`testing/jig-client-test-records/src/lib.rs`, the conversation's read
position (`Seen::Replies` users), the link's `restore_ask`, an exhaustive
match of 23 arms (`crates/jig-client-protocol-link/src/translate/store.rs:78`),
the asks of `crates/jig-client-protocol-link/src/tests/fixtures/link_store.rs`,
and `Ask::DecideProposal` at `tests/ops/src/client/stories.rs:300,457`
(ruling 222); tests `tests/client`,
`tests/client-protocol`, the crates' own. The users are what `rg -l -e
'Ask::(TakeOver|MarkSeen|DecideProposal|DecideEscalation|AnswerQuestion|AnswerPerson)\b'
-e 'Seen::Replies\b' -e 'w::(TakeOver|MarkSeen|SeenReplies)\b' crates
testing tests examples` lists at the increment's start, the people
child's own asks aside (at the base they are in `jig-domain-client`,
`jig-view`, the link, the component and the scripted engine).
Shape (wire, mirrored field for field by the contract):
- `Move { project: u32, task: u64, reason: text 65536 }` (was
  `TakeOver`); `Ask.move_task: Move`.
- `SeenConversation { task: u64, through: u64 }` (was `SeenReplies {
  chat, through }`); `enum SeenThrough { results: SeenResults,
  conversation: SeenConversation }` (was `enum Seen`); `Seen { through:
  SeenThrough }` (was `MarkSeen`); `Ask.seen: Seen`;
  `OwnerOutcomeMarkedSeen` and `OwnerPersonCapacityActionMarkSeen` hold
  `SeenThrough`.
- `DecideProposal { proposal: ProposalKey, decision: Decision }`,
  `DecideEscalation { task: u64, revision: Revision, decision: Decision
  }` (each less `project`); `enum DecideOn { proposal: DecideProposal,
  escalation: DecideEscalation }`; `Decide { project: u32, on: DecideOn
  }`; `Ask.decide: Decide` replaces `decide_proposal` and
  `decide_escalation`.
- `AnswerQuestion { task, message, words }`, `AnswerPerson { task,
  revision, answer }` (each less `project`); `enum AnswerOn { question:
  AnswerQuestion, person: AnswerPerson }`; `Answer { project: u32, on:
  AnswerOn }`; `Ask.answer: Answer` replaces `answer_question` and
  `answer_person`.
- `Cancel { project, task, reason, grace: option duration }`;
  `Amendment` gains `tracked: option Tracking`, with `record
  TrackingTracked { priority: u32 }` and `enum Tracking { tracked:
  TrackingTracked, untracked }` declared before it, mirroring
  `people::Tracking` variant for variant (ruling 260: tasks.md, 3,
  "Whether it is tracked, and its priority if so"); the contract's
  `Amendment.tracked: Option<Tracking>`, `enum Tracking { Tracked {
  priority: u32 }, Untracked }`; a change of priority alone is an
  amendment whose only change is `tracked: Some(Tracked { priority })`;
  `Amend.revision: option Revision`; `AdmissionLimit` gains `grace`.
- The policy's template overrides, declared before `PolicyEdit` and
  mirroring `people::TemplateOverride` field for field:
  `TemplateCharter { value: u32 }`, `enum TemplateExecutor { charter:
  TemplateCharter, chosen }`, `TemplateAmount { value: u64 }`,
  `TemplateAsAllowed { cap: option u64 }`, `enum TemplateBudget {
  amount, as_allowed, given }`, `TemplateSpan { value: duration }`, `enum
  TemplateDeadline { none, span, given }`, `TemplateOverride { template:
  u32, executor: option TemplateExecutor, budget: option TemplateBudget,
  deadline: option TemplateDeadline, priority: option u32 }`,
  `PolicyEditTemplates { overrides: list 16 TemplateOverride }`, and
  `PolicyEdit.templates: PolicyEditTemplates` (jig 22.1's `Template`
  and the policy's document reuse these).
- Contract (`jig_client_domain::contract::Ask`): `Move`, `Seen {
  through: SeenThrough }`, `Decide { project, on: DecideOn }`, `Answer {
  project, on: AnswerOn }`, `Cancel { .., grace: Option<Duration> }`,
  `Amend { .., revision: Option<Revision> }`; `PolicyEdit::Templates {
  overrides: List<TemplateOverride> }` (the interface sketch).
- The component translates each into the people ask jig 04 and jig 08
  left (at the base `people::Ask::{Move, MarkSeen, DecideProposal,
  DecideEscalation, AnswerQuestion, AnswerPerson, Cancel, Amend}`); a
  `Decision` that does not fit its target stays untranslatable, as at
  the base.
Test shape: round trips in the wire crate's tests for `Decide` and
`Answer` (one table test, a row per target); the component's
`src/tests/` (asks): `a_decide_on_an_escalation_reaches_the_people_child_with_its_revision`,
`a_cancel_with_a_grace_reaches_the_people_child_with_its_span`,
`a_priority_only_amend_names_no_revision`,
`a_policy_change_of_template_overrides_reaches_the_people_child`; the
link's: `a_seen_over_a_conversation_names_its_task`;
`tests/client-protocol/tests/client.rs`:
`an_owner_s_template_override_is_answered_policy_changed` (the first
owner overrides template 2's budget in project 1); `tests/client-protocol/tests/client.rs` (jig 12's story file),
reference test (a new kind: an operation sent through `jig_client::Client`
and answered by the real engine, on jig 12's `Story`):

```rust
#[test]
fn a_cancel_with_a_grace_ends_the_task_once_its_grace_has_passed() {
    let mut story = Story::live(fixtures::SEED);
    let task = story.running_task();
    let grace = Duration::from_millis(200);
    let reason = contract::Text::new(Box::from(b"enough".as_slice()), 64).expect("fits");
    let cancel = contract::Ask::Cancel { project: 1, task, reason, grace: Some(grace) };
    story.send(CANCEL, Ask::Jig(Box::new(cancel)));
    assert!(story.run_until(|s| !s.answered(CANCEL).is_empty(), Duration::from_secs(1)));
    assert!(matches!(
        story.answered(CANCEL).as_slice(),
        [Outcome::Answered(Answered::Jig(contract::Answer::Done(contract::Outcome::Owner(
            contract::OwnerOutcome::Cancelled { task: t }
        ))))] if *t == task
    ));
    assert!(story.heard_asked_to_finish(task));
    assert!(story.phase(task) != Some(contract::Phase::Cancelled), "the grace has not passed");
    assert!(story.run_until(|s| s.phase(task) == Some(contract::Phase::Cancelled), Duration::from_secs(1)));
    assert!(story.elapsed_since_answer(CANCEL) >= grace);
}
```

(`Story::live`, `send`, `run_until` and `answered` are jig 12's
(`tests/client-protocol/src/client.rs`); `running_task`,
`heard_asked_to_finish`, `phase` and `elapsed_since_answer` are added to
`Story` by this increment, and `CANCEL` is a `Key` constant beside
jig 12's `STOP`; `contract::Text` has no `From<&[u8]>`, so the test makes
its reason with `Text::new` and a bound of 64 (ruling 222).) Limits
from the world's fixtures; the grace's span limit from
`jig_protocol_config::profile::profile(Profile::Small)`
(`crates/jig-protocol-config/src/profile.rs:45`), the test's 200 ms
within it. Ceilings: unit under 20 ms, joint under 0.5 s.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_cancel_with_a_grace_ends_the_task_once_its_grace_has_passed) |
test(an_owner_s_template_override_is_answered_policy_changed)'` once:
the task hears it is asked to finish, and ends cancelled after its
grace, not before; and the owner's override is answered
`PolicyChanged`. Bound: 2 s.
Don't: rename `ActKind`'s variants (jig 22.4 does); merge `Decision` into
typed decisions; touch `Write` or `Amend.release` (14.3).
Size: at most 300 lines.
Removes: `TakeOver`, `MarkSeen`, `SeenReplies`, the four `Ask` variants
the two merges replace.

### 14.3 The compound request, and `Message`
Scope: `Compound { project, operations }` and its answer (contract.md,
3.4 and section 4); `Message` replacing `Write`, a stopped task's words
and an amendment's release becoming a compound's `Release` (pages.md, 4;
objects.md, 5).
Needs: the session's start: jig 04's people compound request, admitted
in order and committed together or refused naming its first failing
operation; jig 09's operations per request limit.
Files: the v8 schema; `crates/jig-protocol-engine-clients/src/translate/{asks.rs,
answers.rs, refusals.rs}` and a new `translate/operations.rs`;
`crates/jig-client-domain/src/contract/{asks.rs, operations.rs (new),
refusals.rs}` (the contract's `Answer::Refused` gains its place);
`crates/jig-client-protocol-link/src/translate/{operations.rs (new),
answers.rs}`; users: the reference UI's words to a stopped chat and its
amend-and-release act, its conversation's sending (`Write` users),
`testing/jig-client-scripted-engine/src/requests.rs` (it models a
compound of `Message` and `Release`, and of `Amend` and `Release`, as it
modelled `Write { release }` and `Amend { release }`), test records, the
link's `restore_ask` (`crates/jig-client-protocol-link/src/translate/store.rs:78`:
arms for `Message` and `Compound`, none for `Write`, `Amend` without its
release) and the asks of
`crates/jig-client-protocol-link/src/tests/fixtures/link_store.rs`
(ruling 222); `Made::Words`' users (the reference UI's outcome words);
`OwnerOutcomeSaid.released`'s users (at jig `5a9a97485`):
`crates/jig-client-domain/src/contract/owner_outcome.rs:67`,
`crates/jig-client-protocol-link/src/translate/answers.rs:73`,
`crates/jig-client-view/src/owner_outcome.rs:143`,
`testing/jig-client-scripted-engine/src/payload/owner_outcome.rs:49`,
`tests/client/src/joint/wire/outcomes.rs:40-41`,
`tests/client/src/observe.rs:558-559` (at jig `1350b6564`) (ruling 222), and their tests.
Re-check at jig `447ddfdf4` (after 14.1, ruling 261): the Files gain
`crates/jig-client-protocol-link/src/limits.rs:105` (`wire.take_over_reason`,
named after the record: `move_reason`); the full `Amendment` literals
`tracked` reaches: the link's `translate/pages_part5.rs:169-203`
(`amendment_fields`), `crates/jig-client-view/src/confirmation/amendment.rs`
(:309, :314, :369, :417), `tests/engine-clients/tests/requests.rs:109`,
`crates/jig-protocol-engine-clients/src/tests/fixtures.rs:88`
(`w::AmendmentParts`), and the `fields!(Amendment { .. })` list of
`crates/jig-client-scripted-engine/src/payload.rs:127`; the link's
`translate/documents_part6.rs:168` (`answers::seen`, now `SeenThrough`);
for `AdmissionLimit`'s `grace`, the exhaustive match of
`crates/jig-client-view/src/words/refusals.rs:92-128`, a
`JigPhrase::EngineLimitGrace` in `phrases/jig.rs` (beside the other
`EngineLimit` phrases, :1042, :1674, :1842, :2986) and
`phrases/english.rs:355`, and `crates/jig-client-domain/src/contract/admission_limits.rs`;
`tests/client-wire/tests/fuzzy_client_wire.rs` (`check!` of
`SeenReplies` :277, `TakeOver` :314, `MarkSeen` :317).
`testing/jig-client-test-records/src/lib.rs` uses none of the changed
records: no edit. The contract boxes `Amendment` in `Ask::Amend` (and
14.3's `Operation::Amend`), as the sketch does; the
`#[expect(clippy::large_enum_variant)]` of `contract/asks.rs:116-117`
then goes, unfulfilled. `a_priority_only_amend_names_no_revision`
asserts revision 0 reaches the people child (`people::Ask::Amend`'s
`revision: u64`, `crates/jig-domain-people/src/boundary.rs:278`). The
Story world's grace limit is `tests/core/src/world.rs:38` (600 s, as
`small.rs:50`). Moved cites: `tests/client/src/observe.rs:563-564`,
the engine's `translate/answers.rs:180`.
Re-check at jig `447ddfdf4` (ruling 261): the schema's :303-304 are
now :305-306. The Files gain `crates/jig-client-protocol-link/src/limits.rs:100`
(`wire.write_words`, named after the record: `message_words`); for
`AdmissionLimit`'s `operations`, the same view words, phrases and
contract file as 14.2's `grace` (`JigPhrase::EngineLimitOperations`);
`tests/client-wire/tests/fuzzy_client_wire.rs` (`check!` of `Write`
:302, `MadeWords` :357); `Made::Words`'s users
`tests/ops/src/client/app.rs:414`, `tests/client/tests/referee.rs`,
and the view's `JigPhrase::MadeWords` (`phrases/jig.rs` :226, :1261,
:2324, :2574; `words.rs:219`).
Shape:
- Wire: `Message { project: u32, task: u64, words: text 65536 }` (was
  `Write`, less `release`); `Amend` loses `release`; `enum Operation {
  message: Message, amend: boxed Amend, cancel: Cancel, release: Release,
  stop: Stop, move_task: Move }`; `Compound { project: u32, operations:
  bytes 1048576 }` (pieces, each an `Operation`); `Ask.message: Message`,
  `Ask.compound: Compound`; `OwnerOutcomeCompound { project: u32,
  operations: u32, first_message: u64 }`, field for field
  `people::Outcome::Compound` (the designer's answer of 2026-10-16;
  jig's contract.md, 3.4: "the operation at place *i* took the first
  plus *i*"; "Nothing more of an operation is answered"), and
  `OwnerOutcome.compound`; no per-operation list or `OperationOutcome`;
  `Refused { refusal: Refusal, at: option u32 }`; `AdmissionLimit`
  gains `operations`; `MadeWords` and `Made.words` go (`Message`'s
  outcome is `OwnerOutcomeSaid`, as jig 04 left the people outcome);
  `OwnerOutcomeSaid` loses `released`, words releasing nothing (jig's
  `domain/people.md`, 5.1, at `5a9a97485`: "Words release nothing: a
  `Message` adds words to the inbox").
- Engine: a compound's pieces decoded each as an `Operation` and
  translated as alone; one naming another project is answered
  `Refused { refusal: Refusal::Malformed { at: place, why:
  Malformed::ProjectMismatch }, at: Some(place) }` before the core, and
  nothing is kept. `enum Malformed { project_mismatch }` and
  `RefusalMalformed { at: u32, why: Malformed }`, `Refusal.malformed`,
  start here; 14.4 and 14.5 add their variants. A `Message` translates
  to `people::Ask::Say { .., release: false }` (`people::Operation::Message`
  in a compound), and people's `Outcome::Said` to `OwnerOutcomeSaid`
  without its `released`: those people fields stay until the step after
  14.3 (the supervisor's) removes them from the engine.
- Contract: `Ask::Message`, `Ask::Compound { project, operations:
  List<Operation> }` (the interface sketch); `Answer::Refused { refusal,
  at: Option<u32> }`. The reference UI's words to a stopped chat are
  `Compound { [Message, Release { revision, again: None }] }`, the
  revision the hold's as the chat's document gave it (jig's
  `client/domain/pages.md`, 4, at `5a9a97485`: "the release, which names
  the hold's revision the page showed").
Test shape: the wire crate: `a_compound_s_operations_round_trip_as_pieces`;
the component's tests: `a_compound_reaches_the_people_child_in_order`,
`a_compound_operation_naming_another_project_is_refused_at_its_place`,
`a_compound_refusal_names_its_first_failing_place`; the link's:
`a_compound_answer_names_its_first_message_and_operations` (the
engine's `Outcome::Compound` out of `translate/answers.rs`'s error arm,
:176 at jig `1350b6564` (ruling 222), and through the link to the contract's answer,
its three fields unchanged);
`tests/client-protocol/tests/protocol.rs`:
`words_to_a_stopped_task_release_it_in_one_compound_request` (the
reference UI's words to a stopped chat, by the fake person: one keyed
request made, the run resumed). Reuse `stories::submit_stop` and
`stories::send_words` (`tests/client-protocol/src/stories.rs:30, 41` at
the base, followed to jig 12's file). Limits: operations per request
from `profile(Profile::Small)`; the world's own from its fixtures.
Ceilings as 14.2.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(words_to_a_stopped_task_release_it_in_one_compound_request) |
test(=a_person_s_words_reach_a_running_run_through_bytes)'` once: the
stopped task resumes after one request made once (`requests_made()`
counts 1), and words still reach a running run as `Message`. Bound:
2 s.
Don't: let the client chain a `Message` and a `Release` as two
requests; give `Operation` records of its own without `project`; change
`people::Ask::Say`, `people::Operation::Message`, `people::Outcome::Said`
or the tasks child's message (the step after 14.3).
Size: at most 310 lines (280, and 30 for `OwnerOutcomeSaid.released`,
whose 23 lines on the client's side and the engine's translation at jig
`5a9a97485` are counted by `git grep -n -E 'Said|said' 5a9a97485 --
crates/jig-client-domain crates/jig-client-protocol-link
crates/jig-client-view crates/jig-protocol-engine-clients/src
testing/jig-client-scripted-engine tests/client tests/client-protocol
crates/jig-client-protocol-wire/schema | grep -c released`).
Removes: `Write`, `Write.release`, `Amend.release`, `MadeWords`,
`Made::Words`, `OwnerOutcomeSaid.released`; and, with `Amend.release`,
`OwnerOutcomeAmended.released` and `OwnerOutcomeAmendProposed.released`
(`client-v8.schema:303-304`), which nothing sets once it goes:
`Amend`'s answer is "amended" (jig `client/domain/contract.md:164`),
and a `Release` releases (`:285-286`). `people::Outcome::Amended`'s
`released` stays until the step after 14.3, as `Said`'s does; the two
count within this Size (ruling 222).

### 14.4 `Create`
Scope: a party's `Create` of a batch (contract.md, 3.2; tasks' fields
crossing as data; `AsAllowed`; where a party may reach; a party's root
with someone to escalate to), its answer and its refusals' new facts
(section 4: not well formed, no one to escalate to, the goals limit).
Needs: the session's start: jig 03's `people::Ask::Create` of a batch
with person executors; jig 02's label, flags, retention class,
`AsAllowed` and its resolution, the escalation holder's check; jig 04's
goals per project; their refusals in the people vocabulary.
Files: the v8 schema; `crates/jig-protocol-engine-clients/src/translate/{operations.rs,
answers.rs, refusals.rs}`; `crates/jig-client-domain/src/contract/{operations.rs,
refusals.rs}`; `crates/jig-client-protocol-link/src/translate/{operations.rs,
answers.rs}`; users: each exhaustive match over the contract's `Refusal`
and `Limit` (the reference UI's refusal words, the scripted engine's
refusals), the scripted engine's request handling (`Create` answered
not offered, jig 16.3 models it), the link's `restore_ask` (an arm for
`Create`: `crates/jig-client-protocol-link/src/translate/store.rs:78`) and
the asks of `crates/jig-client-protocol-link/src/tests/fixtures/link_store.rs`
(ruling 222); downstream, the repository downstream
of jig's family refusal translation (one production file) follows at
its repin.
Re-check at jig `447ddfdf4` (ruling 261): `Ask::TaskHoldings` is at
`crates/jig-domain-core/src/routing/parties.rs:612`. For
`AdmissionLimit`'s `goals`, the same view words, phrases and contract
file as 14.2's `grace` (`JigPhrase::EngineLimitGoals`). A role executor
names no project: `NewTask.executor` is `OwnerExecutor` (the designer's
answer of 2026-10-17T14:05Z, jig ad3b26d04, below).
Shape: wire, in this order before `Ask`: `FieldRule` and
`VerdictChoice` move up (no byte change); `TaskRef { member: u32 |
task: u64 }`; `NewContract { report: ContractReport { words } |
verdict: ContractVerdict { choices: list 32 VerdictChoice } | change:
ContractChange { connector, kind, words } }`; `NewBudget { named |
as_allowed: BudgetAsAllowed { cap: option u64 } }`; `enum Retention {
kept, forgotten_once_read }`; `TaskFlags { conversation: bool, listed:
bool, retention: Retention }`; `NewTask { executor: OwnerExecutor, spec:
Specification, contract: NewContract, authority: OwnerAuthority,
budget: NewBudget, wake: OwnerWakePolicy, dependencies: list 32
TaskRef, named: bytes 16777216, tracked: option u32, label: u32,
flags: TaskFlags }`, field for field `people::Member`
(`crates/jig-domain-people/src/batch.rs:28-42`; the designer's answer
of 2026-10-17, jig b58a49c39, contract.md, 3.2: "authority, whose
spend and deadline are the task's budget and deadline ..., that budget
as named there or `AsAllowed`"): a `NewBudget::Named` budget is
`authority.spend` (`routing/create.rs:188`), the deadline is
`authority.deadline`;
`executor` is `OwnerExecutor` (`client-v8.schema:91-94`), field for
field `people::TaskExecutor` (`batch.rs:44-49`), its role a
`PersonAddress::Role(u32)` of the `Create`'s project (contract.md, 3.2:
"a person, or a role, which is the project's: roles are per project"),
as `Amend`'s `AddressChange.address` names one (`client-v8.schema:213`);
the component maps it to `people::TaskExecutor`, its person arm as
`translate/asks.rs`'s `address` maps a `PersonAddress` (`:326-329` at
`447ddfdf4`), and no refusal is added for it; the chips' `Executor`
keeps its `ExecutorRole.project`;
`named` is the grants named after the task, pieces of `OwnerGrant` as
`OwnerAuthority.grants` are (`Member.named`, resolved at
`routing/create.rs:179`); no approval contract ("an approval is a
procedure's") and no holdings list (each connector names a party's
task's holds from its executor and spec before the batch commits,
`Ask::TaskHoldings` at `routing/parties.rs:608`; tasks.md, 4); `Create
{ project: u32, tasks: bytes 1048576 }`; `Ask.create`; `CreatedGiven {
task: u64, amount: u64 }` and `OwnerOutcomeCreated { first: u64, last:
u64, given: option CreatedGiven }`, field for field
`people::Outcome::Created` (`boundary.rs:403`; contract.md, 4: "the
first and the last of the tasks' numbers, consecutive in the batch's
order"); `Malformed` gains `two_as_allowed`, `conversation_not_agent`
and `cycle` (`people::Malformed::{AsAllowedTwice, Conversation,
Cycle}`, `refusals.rs:55-61`; contract.md, 4: "a cycle among a batch's
dependencies"); `Refusal.no_escalation_holder`;
`AdmissionLimit.goals`. Contract: the interface sketch's `NewTask`,
`TaskRef`, `NewContract`, `NewBudget`, `TaskFlags`, `Retention`,
`Malformed`, `Ask::Create`.
Test shape: the component's tests:
`a_create_reaches_the_people_child_with_each_task_s_fields` (one task
an agent's, one a role's: `OwnerExecutor::Person(PersonAddress::Role(2))`
reaches it as `TaskExecutor::Person(PersonAddress::Role(2))`),
`an_as_allowed_answer_names_what_was_given`; the link's:
`a_create_s_tasks_cross_as_pieces_within_the_request_s_bytes`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`a_party_creates_a_batch_whose_as_allowed_task_hears_what_it_was_given`,
`a_root_task_with_no_one_to_escalate_to_is_refused_saying_so`
(on jig 12's `Story`; fixtures: the world's first owner, `fixtures::FIRST_OWNER`, project 1,
the core world's configuration, `jig_core_world::world::config`). Limits
as 14.3. Ceilings as 14.2.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_party_creates_a_batch_whose_as_allowed_task_hears_what_it_was_given)'`
once: the engine makes both tasks at once and answers their numbers, the
`AsAllowed` one with the amount it gave. Bound: 1 s.
Don't: compute `AsAllowed` or check a draft's authority in the client;
let the scripted engine make tasks from `Create` before jig 16.3; name a
project in `NewTask`'s executor, or add a refusal for a role's project.
Size: at most 300 lines.
Removes: nothing.

### 14.5 `Propose` and `Withdraw`
Scope: a party's `Propose` of a batch or an amendment (a widening of a
task's authority among them), and `Withdraw` of their own pending
proposal (contract.md, 3.2 and 3.5). The authority refusal's
`proposable` stays as it is (section 4).
Needs: the session's start: jig 03's `people::Ask::{Propose, Withdraw}`
and the party as a proposal's proposer.
Files: the v8 schema; the component's `translate/{operations.rs,
answers.rs, refusals.rs}`; the contract's `operations.rs`,
`refusals.rs`; the link's `translate/{operations.rs, answers.rs}`;
the link's `restore_ask` (arms for `Propose` and `Withdraw`:
`crates/jig-client-protocol-link/src/translate/store.rs:78`) and the asks of
`crates/jig-client-protocol-link/src/tests/fixtures/link_store.rs`
(ruling 222); the
reference UI's amend confirmation (objects.md, 11: "beyond those is
proposed, and to whom (a `Propose` of the amendment)"), which at the
base sends `Ask::Amend` for an amendment its offer says is proposable
(`crates/jig-client-view/src/tests/domain/confirmation.rs:311` tests it) (ruling 222) and
now sends `Propose` of the amendment, saying to whom; the scripted
engine's model of a proposed amendment; downstream, the same family
refusal translation as 14.4.
Shape: wire: `ProposeBatch { tasks: bytes 1048576 }` (pieces of
`NewTask`), `ProposeAmendment { task: u64, revision: Revision, change:
Amendment }` (its revision required, as in `people::Action::Amend {
task, revision: u64, amendment }`, `batch.rs:8-12`), `enum ProposeAction {
batch, amendment: boxed ProposeAmendment }` (no widening: "A party
proposes a batch, or an amendment, a widening of a task's authority
among them", people.md, 5.1; `people::Action` is `Batch | Amend`),
`Propose { project: u32, action: ProposeAction, reason: text 65536 }`,
`Withdraw { project: u32, proposal: u64 }`, `Ask.{propose, withdraw}`;
`OwnerOutcomeProposed { proposal: u64 }` (contract.md, 3.2: "its
number answered"; whom it waits for is its document's `waiting`, and
the offer's `to` before it is sent: the designer's answer of
2026-10-17, contract.md, 4), `OwnerOutcomeWithdrawn { proposal: u64
}`; `RefusalAuthority { lacking: Lacking, proposable: option bool }`
unchanged, as main's three states map onto it
(`crates/jig-protocol-engine-clients/src/translate/refusals.rs:26-30`):
none, the refusal makes no claim (`None`, contract.md, 4: "A refusal
that another check decided ... makes no claim either way"); false, no
proposal could be accepted (`Accepter::Nobody`); true, one could
(`Accepter::Role(_)`); `Malformed` gains `proposed_as_allowed`. Contract:
the sketch's `ProposeAction`, `Ask::{Propose, Withdraw}`.
`OwnerOutcomeGoalProposed` stays until jig 16.
Test shape: the component's tests:
`a_propose_of_an_amendment_reaches_the_people_child_with_its_reason`;
`tests/client-protocol/tests/client.rs` (jig 12's story file):
`a_party_proposes_a_batch_and_withdraws_it_before_it_is_decided`,
`a_create_beyond_authority_is_refused_saying_a_proposal_could_be_accepted`
(its `proposable` is `Some(true)`); the
reference UI's (`crates/jig-client-view/src/tests/domain/confirmation.rs`) (ruling 222):
`a_changed_revision_refuses_in_the_confirmation_while_spend_changes_only_its_words`
(`:295`, its proposable amendment at `:311`) keeps its name and its
other assertions, and expects `Ask::Propose` of the amendment where it
expected `Ask::Amend`; and
`an_amendment_beyond_the_offer_says_to_whom_it_will_be_proposed`. Limits
and ceilings as 14.4.
Proof: joint, `cargo nextest run -p jig-client-protocol-world -E
'test(a_party_proposes_a_batch_and_withdraws_it_before_it_is_decided)'`
once: the proposal's number is answered, it waits at the policy's role,
and the withdraw leaves it withdrawn, the party its proposer. Bound: 1 s.
Don't: send a `Propose` on the party's behalf when a `Create` is refused;
offer `Withdraw` on a card (jig 22.4 does).
Size: at most 270 lines.
Removes: the reference UI's `Amend` of an amendment beyond the offer's
first bound.

## Interface sketches

Checked at the plan's base: the schema as one edit of the v7 schema,
this session's records and jig 22's together
(`client-v7*.schema` in a scratch worktree, applied by a script; the
same edits apply to jig 11's v8), generated with skein-codegen, and
`cargo clippy -p jig-protocol-client-wire --lib -- -D warnings`
clean (its `--all-targets` fails only in its own test fixtures that
construct the changed records, which the increments edit). The Rust
types compiled in a scratch crate against the base's contract types,
`cargo clippy -p w7-sketch --all-targets -- -D warnings` clean.
Names from jig 05, 07, 09, 11 and 12 are checked at the ready check.

`crates/jig-client-protocol-wire/schema/client-v8*.schema`, the records
this session adds or changes (each increment's Shape says which are
its). Each is declared before its first user, as the generator requires
("A type uses only types declared before it", codec.md, 3): `Served`
before the answers, `FieldRule` and `VerdictChoice` before `NewTask`:

```
# Moved before their first users, unchanged:
record Served { owner: Owner first: u32 last: u32 }
record FieldRule { name: bytes 65536 max: u32 }
record VerdictChoice { code: u32 words: u32 followups: u32 fields: list 32 FieldRule }
# Added, or changed in place:
record NotServed { owner: Owner served: list 16 Served }
enum KeyedAnswer { not_served: NotServed, final_answer: Owned, busy, not_ready, signed_out }
record TrackingTracked { priority: u32 }
enum Tracking { tracked: TrackingTracked, untracked }
record Amendment { addressed: option AddressChange spec: option Specification wake: option OwnerWakePolicy dependencies: option list 32 u64 authority: option OwnerAuthority instructions: option text 65536 tracked: option Tracking }
record SeenConversation { task: u64 through: u64 }
enum SeenThrough { results: SeenResults, conversation: SeenConversation }
record TemplateCharter { value: u32 }
enum TemplateExecutor { charter: TemplateCharter, chosen }
record TemplateAmount { value: u64 }
record TemplateAsAllowed { cap: option u64 }
enum TemplateBudget { amount: TemplateAmount, as_allowed: TemplateAsAllowed, given }
record TemplateSpan { value: duration }
enum TemplateDeadline { none, span: TemplateSpan, given }
record TemplateOverride { template: u32 executor: option TemplateExecutor budget: option TemplateBudget deadline: option TemplateDeadline priority: option u32 }
record PolicyEditTemplates { overrides: list 16 TemplateOverride }
record Message { project: u32 task: u64 words: text 65536 }
record AnswerQuestion { task: u64 message: u64 words: text 65536 }
record AnswerPerson { task: u64 revision: Revision answer: PersonAnswer }
enum AnswerOn { question: AnswerQuestion, person: AnswerPerson }
record Answer { project: u32 on: AnswerOn }
record DecideProposal { proposal: ProposalKey decision: Decision }
record DecideEscalation { task: u64 revision: Revision decision: Decision }
enum DecideOn { proposal: DecideProposal, escalation: DecideEscalation }
record Decide { project: u32 on: DecideOn }
record Amend { project: u32 task: u64 revision: option Revision change: Amendment reason: text 65536 }
record Cancel { project: u32 task: u64 reason: text 65536 grace: option duration }
record Move { project: u32 task: u64 reason: text 65536 }
record Seen { through: SeenThrough }
enum Operation { message: Message, amend: boxed Amend, cancel: Cancel, release: Release, stop: Stop, move_task: Move }
record Compound { project: u32 operations: bytes 1048576 }
record TaskRefMember { value: u32 }
record TaskRefTask { value: u64 }
enum TaskRef { member: TaskRefMember, task: TaskRefTask }
record ContractReport { words: u32 }
record ContractVerdict { choices: list 32 VerdictChoice }
record ContractChange { connector: u16 kind: u16 words: u32 }
enum NewContract { report: ContractReport, verdict: ContractVerdict, change: ContractChange }
record BudgetAsAllowed { cap: option u64 }
enum NewBudget { named, as_allowed: BudgetAsAllowed }
enum Retention { kept, forgotten_once_read }
record TaskFlags { conversation: bool listed: bool retention: Retention }
record NewTask { executor: OwnerExecutor spec: Specification contract: NewContract authority: OwnerAuthority budget: NewBudget wake: OwnerWakePolicy dependencies: list 32 TaskRef named: bytes 16777216 tracked: option u32 label: u32 flags: TaskFlags }
record Create { project: u32 tasks: bytes 1048576 }
record ProposeBatch { tasks: bytes 1048576 }
record ProposeAmendment { task: u64 revision: Revision change: Amendment }
enum ProposeAction { batch: ProposeBatch, amendment: boxed ProposeAmendment }
record Propose { project: u32 action: ProposeAction reason: text 65536 }
record Withdraw { project: u32 proposal: u64 }
enum Ask { start_chat: StartChat, message: Message, answer: Answer, take: Take, hand_back: HandBack, decide: Decide, release: Release, amend: boxed Amend, stop: Stop, cancel: Cancel, close: Close, move_task: Move, set_goal: SetGoal, prioritise: Prioritise, seen: Seen, edit_note: EditNote, delete_note: DeleteNote, change_policy: ChangePolicy, set_roles: SetRoles, make_service: MakeService, set_pool: SetPool, compound: Compound, create: Create, propose: Propose, withdraw: Withdraw }
record OwnerOutcomeCompound { project: u32 operations: u32 first_message: u64 }
record CreatedGiven { task: u64 amount: u64 }
record OwnerOutcomeCreated { first: u64 last: u64 given: option CreatedGiven }
record OwnerOutcomeProposed { proposal: u64 }
record OwnerOutcomeWithdrawn { proposal: u64 }
enum Malformed { two_as_allowed, conversation_not_agent, cycle, proposed_as_allowed, project_mismatch }
record RefusalMalformed { at: u32 why: Malformed }
record Refused { refusal: Refusal at: option u32 }
record ScopeProject { value: u32 }
enum Scope { project: ScopeProject, deployment, own }
record Window { rows: u32 after: option Cursor }
record FamilyRecord { scope: Scope window: Window record: bytes 1048576 }
record FamilyRequest { scope: Scope record: bytes 1048576 }
```

Besides: `ReadAnswer`, `OpenAnswer` gain `not_served: NotServed`, and
`WatchEnd` gains `not_served`, each as its first variant (tag 0);
`PolicyEdit` gains `templates: PolicyEditTemplates`; `AdmissionLimit` gains `grace`,
`operations`, `goals`; `Refusal` gains `malformed: RefusalMalformed`,
`no_escalation_holder`; `OwnerOutcome` gains `compound`, `created`,
`proposed`, `withdrawn`; `Made` loses `words`; `OwnerOutcomeMarkedSeen`
and `OwnerPersonCapacityActionMarkSeen` hold `SeenThrough`. `StartChat`,
`Close`, `SetGoal`, `Prioritise` and their records stay until jig 16;
`me`, the task line, task queries and `Offered` are jig 22's.
`Keyed.request` and `TableEntry.request` become `bytes 1114240` (14.1)
(ruling 222).

`crates/jig-client-domain/src/contract/` (jig 11's module), the types
whose wire records are above; owned by this session, used by jig 15's
helpers and jig 16's reference UI through `jig_client`'s re-export of
them:

```rust
// contract/operations.rs
/// What a `Seen` moves.
pub enum SeenThrough { Results { through: u64 }, Conversation { task: u64, through: u64 } }
/// What a `Decide` decides.
pub enum DecideOn { Proposal { proposal: ProposalKey, decision: Decision }, Escalation { task: u64, revision: Revision, decision: Decision } }
/// What an `Answer` answers.
pub enum AnswerOn { Question { task: u64, message: u64, words: Text }, Person { task: u64, revision: Revision, answer: PersonAnswer } }
/// One operation of a compound request; each names the compound's project.
pub enum Operation {
    Message { project: u32, task: u64, words: Text },
    Amend { project: u32, task: u64, revision: Option<Revision>, change: Box<Amendment>, reason: Text },
    Cancel { project: u32, task: u64, reason: Text, grace: Option<Duration> },
    Release { project: u32, task: u64, revision: Revision, again: Option<bool> },
    Stop { project: u32, task: u64 },
    Move { project: u32, task: u64, reason: Text },
}
/// A new task's dependency: a member of its batch, by place, or a task.
pub enum TaskRef { Member(u32), Task(u64) }
/// The result contract a new task owes.
pub enum NewContract { Report { words: u32 }, Verdict { choices: List<VerdictChoice> }, Change { connector: u16, kind: u16, words: u32 } }
/// A new task's budget: its authority's spend, or `AsAllowed`, whose answer says what was given.
pub enum NewBudget { Named, AsAllowed { cap: Option<u64> } }
/// What becomes of a task once ended.
pub enum Retention { Kept, ForgottenOnceRead }
/// A task's flags, each with engine semantics.
pub struct TaskFlags { pub conversation: bool, pub listed: bool, pub retention: Retention }
/// One task of a `Create` or a proposed batch, field for field `people::Member`; `named` holds the grants named after it.
pub struct NewTask {
    pub executor: OwnerExecutor, pub spec: Specification, pub contract: NewContract, pub authority: OwnerAuthority,
    pub budget: NewBudget, pub wake: OwnerWakePolicy, pub dependencies: List<TaskRef>,
    pub named: List<OwnerGrant>, pub tracked: Option<u32>, pub label: u32, pub flags: TaskFlags,
}
/// What a party proposes.
pub enum ProposeAction { Batch { tasks: List<NewTask> }, Amendment { task: u64, revision: Revision, change: Box<Amendment> } }
/// Why a request is not well formed; `Refused`'s place names where.
pub enum Malformed { TwoAsAllowed, ConversationNotAgent, Cycle, ProposedAsAllowed, ProjectMismatch }
/// jig's requests as this session leaves them (the project's own, and the four shape asks jig 16 removes, elided).
pub enum Ask {
    Create { project: u32, tasks: List<NewTask> },
    Propose { project: u32, action: ProposeAction, reason: Text },
    Withdraw { project: u32, proposal: u64 },
    Message { project: u32, task: u64, words: Text },
    Amend { project: u32, task: u64, revision: Option<Revision>, change: Box<Amendment>, reason: Text },
    Cancel { project: u32, task: u64, reason: Text, grace: Option<Duration> },
    Release { project: u32, task: u64, revision: Revision, again: Option<bool> },
    Stop { project: u32, task: u64 },
    Move { project: u32, task: u64, reason: Text },
    Decide { project: u32, on: DecideOn },
    Answer { project: u32, on: AnswerOn },
    Take { project: u32, task: u64 },
    HandBack { project: u32, task: u64 },
    Seen { through: SeenThrough },
    Compound { project: u32, operations: List<Operation> },
}

// contract/policy_edit.rs: a policy's template overrides (14.2); jig 22.1's `Template` reuses the three enums.
/// A template's executor: a charter, or the creator's choice among what the role may create with.
pub enum TemplateExecutor { Charter(u32), Chosen }
/// A template's budget.
pub enum TemplateBudget { Amount(u64), AsAllowed { cap: Option<u64> }, Given }
/// A template's deadline.
pub enum TemplateDeadline { None, Span(Duration), Given }
/// A project's override of one configured template (mirrors `people::TemplateOverride`).
pub struct TemplateOverride {
    pub template: u32, pub executor: Option<TemplateExecutor>, pub budget: Option<TemplateBudget>,
    pub deadline: Option<TemplateDeadline>, pub priority: Option<u32>,
}
// `PolicyEdit` gains `Templates { overrides: List<TemplateOverride> }`: the project's overrides replaced as one list.

// The envelope (contract/versions.rs).
/// An owner of records.
pub enum Owner { Jig, Family(u16) }
/// The typed answer to a message in a version the engine does not serve.
pub struct NotServed { pub owner: Owner, pub served: List<Served> }
/// Where an application's read or subject is admitted; `Own` names no party.
pub enum Scope { Project(u32), Deployment, Own }
```

The component owns translation in both directions for each record; the
link owns the client's; neither keeps state for them.

## Questions to raise when reached

- None the design leaves open that these increments decide.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-14\.[0-9]' | sort -u | wc -l` prints 5 (ruling 222).
- The replaced records are gone: `git -C ~/src/rust/jig grep -n -E
  '^(record|enum) (Write|TakeOver|MarkSeen|SeenReplies|MadeWords)\b'
  origin/main -- crates/jig-client-protocol-wire/schema` prints nothing.
- The new ones are there: `git -C ~/src/rust/jig grep -n -E
  '^(record|enum) (Create|Propose|Withdraw|Compound|NotServed|FamilyRecord)\b'
  origin/main -- crates/jig-client-protocol-wire/schema` prints 6 lines,
  and `git -C ~/src/rust/jig grep -n not_served origin/main --
  crates/jig-client-protocol-wire/schema` prints 4.
