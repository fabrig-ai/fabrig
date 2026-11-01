Read-only survey; nothing built or changed. Paths and line numbers below refer to this checkout. In dependency tables, join the directory column to each listed file suffix; these are exact repository-relative paths. Lists cover direct constructors, readers, translators and test-world dispatchers, rather than transitive Cargo dependencies.

## 1. Demo lifecycle, records, configuration, grants and forge retirement

**Important differences from the draft’s description of “today”:**

- `Demo` stores **only `forge` and `state`**, not a project number.
- `application.demo.project` is **a numeric configured project**, currently `1`.
- `StartDemo` adds the account and adoption through **the requesting person’s admitted requests**. It does not currently use deployment `AddKey`/`Adopt` acts or make a project.
- The service returns an **ended `DemoAnswer` retaining identity**, rather than an absent record. The client consumes that answer and retains `RetiredDemo`.
- `EndDemo`, `StartPlayground`, `EndPlayground`, `RemoveForge` and `RemoveProject`: **not found** in Rust code or schemas.

### Types and entry points

| Item | Exact location and shape |
|---|---|
| Engine request | `temper/crates/temper-engine-domain-forge/src/boundary.rs:155`: `Ask::StartDemo` |
| Engine answer | Same file, `:187`: `Answered::StartDemo(StartAnswer)` |
| Root answer | `temper/crates/temper-engine-domain/src/boundary.rs:396`: `ApplicationAnswer::StartDemo(forge::StartAnswer)` |
| Start result | Forge boundary `:249`: `StartAnswer::{Running, NotOffered, Failed(StartFailure)}` |
| Start failure | Forge boundary `:255`: `StartFailure::{Group { why: Box<[u8]> }, Account, Adoption, NotConnected}` |
| Configuration | Forge boundary `:263`: `DemoConfig { provider: u32, adoption: Adoption, deadline: skein_lib::Duration }` |
| Durable record | Forge boundary `:1613`: `Demo { forge: u16, state: DemoState }` |
| Durable state | Forge boundary `:1619`: `DemoState::{Running, Ended}`; absence means not started |
| Kept act | Forge boundary `:1634`: `DemoAct { key: [u8; 16], act: DemoActKind }` |
| Kept act variants | Forge boundary `:1639`: `DemoActKind::{Close { project: u32, task: u64 }, RemoveAccount { account: u32 }}` |
| Store wrappers | Forge boundary `:854`, `:861`: `Stored::Demo(Demo)`, `Stored::DemoAct(DemoAct)` |
| Store keys | Forge boundary `:907`, `:910`: `Key::Demo(u16)`, `Key::DemoAct([u8; 16])` |
| Capacity | `temper/crates/temper-engine-domain-forge/src/limits.rs:107`: `pub const fn demo_act_bound(l: &Limits) -> Option<u32>` returns `l.tasks.checked_add(1)` |

The stored shapes are:

```rust
pub struct Demo {
    pub forge: u16,
    pub state: DemoState,
}
pub enum DemoState {
    Running,
    Ended,
}
pub struct DemoAct {
    pub key: [u8; 16],
    pub act: DemoActKind,
}
pub enum DemoActKind {
    Close { project: u32, task: u64 },
    RemoveAccount { account: u32 },
}
```

### Start path

1. Client encoding: `temper/crates/temper-client-domain/src/forge/demo.rs:461`:

   ```rust
   pub fn ask(limits: &Limits) -> Option<jig_client::Ask>
   ```

   It encodes `w::Request::StartDemo` in a family request with `Scope::Deployment`.

2. Client action: same file `:525`:

   ```rust
   pub(crate) fn start(side: &mut Side, limits: Limits, out: &mut Queue<Request>)
   ```

   Guards: signed-in party, current protocol, no pending start, no retired demo. A running demo without a failure sends nothing.

3. Service translation: `temper/crates/temper-service/src/family.rs:166` maps the wire request to `root::Application::Forge(forge::Ask::StartDemo)`.

4. Service carries the configured key before admission: `temper/crates/temper-service/src/clients.rs:490`. It parks the request and queues a `HandoverRead` of `demo.key_file`, bound `4097`; implementation tested by `src/clients/demo_tests.rs:23`, `start_demo_reads_each_time_and_carries_the_key_before_admission()`.

5. Root admission/context: `temper/crates/temper-engine-domain/src/route/applications.rs:36`, `:67`; `Context { sign_in, key, carried }` is defined in `src/route/demo.rs:11`.

6. Connector decision: `temper/crates/temper-engine-domain-forge/src/demo.rs:82`:

   ```rust
   pub(crate) fn start(
       d: &mut Domain, request: Token, token: Token, out: &mut Queue<Request>
   )
   ```

   No configured demo or `Ended` → `NotOffered`. First start saves `Running`. Under the box, if connection is needed, emits `SetUpGroup`; then emits `DemoStart`.

7. Root orchestration: `temper/crates/temper-engine-domain/src/route/demo.rs`:

   - `:43` `start(domain, env, decision, request)`.
   - `:53` `begin(...)`: finds an existing deployment account of the provider, otherwise sends the person’s `core::Event::Key` carrying:

     ```rust
     people::Ask::AccountChange {
         holder: people::AccountHolder::Deployment,
         change: people::AccountChange::AddKey {
             provider, join: None, digest: carried.digest,
         },
     }
     ```

   - `:93` `progress(...)`: waits for connection; skips existing adoption; otherwise invokes `applications::start` with the person’s sign-in and `ApplicationScope::Project(adoption.project)`.
   - `:34` `key(original: [u8; 16], act: &[u8]) -> [u8; 16]`: SHA-256 prefix over `"temper demo start\0"`, original key, and purpose (`account`/`adoption`).
   - `:129` `finish(...)`: sends `Event::DemoAnswered`.

**Restart limitation:** connector `demo.rs:126`, `restart(d: &Domain, out: &mut Queue<Request>)`, asks for the running demo’s group; it makes no account/adoption act. This checkout does not implement the design’s automatic repair of an adoption interrupted during start. The corresponding test is `temper/tests/engine/domain/tests/suite/demo.rs:223`, `demo_restart_asks_the_group_makes_no_act_and_forgets_the_start_answer()`.

### End decision and kept-act replay

`temper/crates/temper-engine-domain-forge/src/domain.rs:1111` handles `Event::Connection`. For `Connected`, it calls:

```rust
// temper/crates/temper-engine-domain-forge/src/demo.rs:137
pub(crate) fn connected(d: &mut Domain, forge: u16, out: &mut Queue<Request>)
```

The decisive guard and writes are at `demo.rs:138`:

```rust
if forge == demo_forge
    || !d.configured.contains_key(&forge)
    || d.demo_state == Some(DemoState::Ended)
{
    return;
}
let ran = d.demo_state == Some(DemoState::Running);
d.demo_state = Some(DemoState::Ended);
out.push(Request::Save {
    read_afresh: None,
    release: None,
    record: Stored::Demo(Demo { forge: demo_forge, state: DemoState::Ended }),
});
```

Then:

- `ran` → `Request::ListDemoActs`;
- `ran && d.under_box` → `Request::StopGroup { forge: demo_forge }`;
- pending starts → `NotOffered`.
- An unstarted demo is also permanently ended by another configured forge connecting.

Root dispatch is `temper/crates/temper-engine-domain/src/route/forge_route.rs:2990`:

```rust
Request::ListDemoActs => demo::list_end_acts(domain),
Request::SendDemoAct(row) => demo::send_end_act(domain, row),
```

Kept-act machinery, `temper/crates/temper-engine-domain-forge/src/demo.rs`:

```rust
// :159
pub(crate) fn acts(d: &mut Domain, acts: Box<[crate::DemoAct]>, out: &mut Queue<Request>)
// :167
pub(crate) fn send_acts(d: &Domain, out: &mut Queue<Request>)
// :173
pub(crate) fn act_answered(
    d: &mut Domain, key: [u8; 16], keep: bool, out: &mut Queue<Request>
)
```

`acts` inserts and saves every row before emitting sends. `act_answered` erases `Key::DemoAct(key)` only when `keep == false`.

- Held map: `temper/crates/temper-engine-domain-forge/src/domain.rs:125`, `demo_acts: Map<[u8; 16], crate::DemoActKind>`.
- Restoration: same file `:4865`; `Stored::DemoAct` repopulates the map; `Stored::Demo` restores state at `:4868`.
- Resend at core restart `Open`: `temper/crates/temper-engine-domain/src/restart.rs:386`, `:408`, queues `forge::Event::SendDemoActs`.
- Root reply map: `temper/crates/temper-engine-domain/src/domain.rs:65`, `demo_act_replies: Map<Token, forge::DemoAct>`.
- Result handling: `src/route/demo.rs:313`, `end_acted(domain, env, decision, to, outcome)`.
- Keeps: `Again`, `Refused(NotOffered)`, `Refused(Role { held: None })`.
- Erases: success (`Compound`, `AccountRemoved`), `Unknown`, `Ended`, and other permanent refusals. Diagnostics use `Delivery::DemoNote`.

### Forge retirement today

Design: `temper/docs/design/domain/forge.md:378`–`:403`.

The public predicates are:

```rust
// temper/crates/temper-engine-domain-forge/src/demo.rs:42
pub fn forge_ended(&self, forge: u16) -> bool

// temper/crates/temper-engine-domain/src/domain.rs:164
pub fn forge_ended(&self, forge: u16) -> bool

// temper/crates/temper-service/src/forge.rs:12
pub(crate) fn ended(engine: &crate::engine::Engine, forge: u16) -> bool
```

Only the configured demo forge can satisfy the domain predicate.

| Effect | Code |
|---|---|
| Absent from forge rows | `temper/crates/temper-engine-domain-forge/src/domain.rs:241` |
| No setup offer | Same file `:308` |
| Absent from adopted IDs | Same file `:752` |
| Absent from project repositories, context and home | Same file `:890`, `:910`, `:923`; `src/items.rs:54` |
| Adoption refused / ended home ignored | `src/adoption.rs:106`, `:275` |
| Topics refuse ended repository | `src/topics.rs:112` |
| Drop pending reads and rebuild watches without ended forge | `src/domain.rs:4996`, `retire_ended(d, env, out)` |
| Reconciliation tolerates historical rows on ended forge | `src/reconcile.rs:307`, `:314`, `:321`, `:358`, `:365` |
| Service refuses calls as `Error::NotConnected` | `temper/crates/temper-service/src/route.rs:65` |
| Service drops hints | `temper/crates/temper-service/src/engine.rs:699` |
| Service refuses connection | `temper/crates/temper-service/src/forge.rs:479` |
| Setup host skips ended forge | `temper/crates/temper-service/src/setup/lifecycle.rs:141`, scans at `:143`, `:153`, `:161`, `:177`; further admission guards throughout that file |
| Setup entry points refuse ended forge | `temper/crates/temper-service/src/setup.rs:313`, `:587` |

Rows and writers remain; retirement is filtering/gating, not erasure.

### Persistence and wire

- `temper/crates/temper-protocol-store/schema/store-v10.schema:207`: `DemoState`, `Demo`.
- Same schema `:212`: `DemoClose`, `DemoRemoveAccount`, `DemoActKind`, `DemoAct`.
- `temper/crates/temper-protocol-store/src/keys.rs:170`: family `19`, 16-byte act key; `:173`: family `17`, forge number encoded as `u64`.
- `src/records.rs:48`, `:65`, `:109`: typed record conversion.
- Generated store types: `src/generated/v10.rs:9761`–`:9978`.
- Client schema: `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema:28`, `Request::start_demo`; `:37`, `DemoState { not_started, running, ended }`; `:72`, `Read::demo`; `:74`, `DemoAnswer`.
- Generated client types: `src/generated/v1.rs:1554`, `:1832`, `:2899`.
- Codec API: `src/lib.rs:189`, `demo_answer(record: &v1::DemoAnswer) -> Box<[u8]>`; `:196`, `decode_demo_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::DemoAnswer>`.

### Reverse dependencies: `StartDemo`

Definitions above plus:

| Directory | Files and first relevant line |
|---|---|
| `temper/crates/temper-engine-domain/src/` | `boundary.rs:396`, `route/applications.rs:27`, `route/demo.rs:183` |
| `temper/crates/temper-engine-domain-forge/src/` | `boundary.rs:155`, `domain.rs:348`, `connect.rs:60`, `tests/demo.rs:178` |
| `temper/crates/temper-service/src/` | `family.rs:152`, `clients.rs:494`, `clients/demo_tests.rs:8`, `tests/family.rs:1268`, `setup/settings.rs:295`, `setup/lifecycle.rs:84` |
| `temper/crates/temper-client-domain/src/` | `forge.rs:261`, `forge/demo.rs:463`, `forge/form.rs:49`, `route.rs:58`, `tests/demo.rs:356`, `tests/form.rs:91` |
| `temper/crates/temper-client-protocol-wire/` | `schema/client-wire-v1.schema:28`, `src/generated/v1.rs:1560`, `src/lib.rs:275`, `src/tests.rs:292` |
| `temper/crates/temper-client-tools/src/` | `forges.rs:519`, `forges/connection.rs:277`, `tests/forges.rs:709`, `tests/fixtures/adopt.rs:85` |
| `temper/crates/temper-client-view/src/` | `tests/fixtures.rs:1566` |
| `temper/crates/temper-client-shell-cli/src/` | `tests/fixtures/setup.rs:46` |
| `temper/crates/temper-client-shell-terminal/src/` | `tests/family.rs:27` |
| `temper/tests/client/` | `src/front.rs:888`, `src/translate/forge.rs:155`, `tests/suite/demo_start.rs:41` |
| `temper/tests/codecs/` | `tests/suite/client_wire.rs:331`, `tests/suite/fuzzy/client_wire.rs:39` |
| `temper/tests/engine/domain/` | `tests/suite/demo.rs:107` |

No direct `StartDemo` references in skein, smith or jig.

### Reverse dependencies: durable `Demo` / `DemoConfig`

| Directory | Files |
|---|---|
| `temper/crates/temper-engine-domain-forge/src/` | `boundary.rs:263`, `demo.rs:3`, `domain.rs:122`, `lib.rs:249`, `restore.rs:280`, `reconcile.rs:26`, `limits.rs:64`, `tests/demo.rs:84`, `tests/restore_fit.rs:117`, `tests/reconcile.rs:256` |
| `temper/crates/temper-engine-domain/src/` | `translate.rs:36`, `domain.rs:168`, `route/demo.rs:94`, `route/forge_route/tests.rs:275` |
| `temper/crates/temper-protocol-store/` | `schema/store-v10.schema:207`, `src/generated/v10.rs:9761`, `src/records.rs:65`, `src/keys.rs:173`, `src/tests/records.rs:795`, `src/tests/fixtures.rs:174`, `src/tests/sizes.rs:60`, `src/tests/keys.rs:132` |
| `temper/crates/temper-fake/src/` | `check.rs:142` |
| `temper/crates/temper-service/src/` | `settings.rs:16`, `engine.rs:338`, `forge.rs:14`, `route.rs:11`, `tests/setup.rs:1721`, `tests/store.rs:124` |
| `temper/tests/engine/` | `domain/tests/suite/demo.rs:57`, `forge/src/world.rs:918`, `protocol-forge/src/protocol.rs:672` |
| `temper/tests/codecs/` | `tests/suite/fuzzy/store.rs:69` |

Client-side `Demo`/`DemoState` are separate types; their dependencies are in question 4.

### Reverse dependencies: `DemoAct` and replay protocol

| Directory | Files |
|---|---|
| `temper/crates/temper-engine-domain-forge/src/` | `boundary.rs:861`, `demo.rs:151`, `domain.rs:125`, `lib.rs:251`, `restore.rs:240`, `reconcile.rs:28`, `limits.rs:107`, `tests.rs:357`, `tests/demo.rs:498`, `tests/restore_fit.rs:84`, `tests/reconcile.rs:212` |
| `temper/crates/temper-engine-domain/src/` | `route/demo.rs:262`, `route/forge_route.rs:2990`, `route/held.rs:189`, `domain.rs:65`, `decision.rs:67`, `admission.rs:84`, `restart.rs:408`, `boundary.rs:814` |
| `temper/crates/temper-protocol-store/` | `schema/store-v10.schema:212`, `src/generated/v10.rs:9860`, `src/records.rs:48`, `src/keys.rs:9`, `src/tests/records.rs:872`, `src/tests/fixtures.rs:176`, `src/tests/keys.rs:170` |
| `temper/crates/temper-fake/src/` | `check.rs:143` |
| `temper/crates/temper-service/src/` | `route.rs:124`, `clients/translation.rs:131`, `tests/box_link.rs:355` |
| `temper/tests/engine/domain/` | `tests/suite/demo.rs:479`, `src/walking.rs:900`, `src/escalation.rs:796` |
| `temper/tests/engine/forge/` | `src/world.rs:778`, `tests/suite/simulation.rs:818` |
| `temper/tests/engine/protocol-forge/` | `src/protocol.rs:504`, `tests/suite/protocol.rs:507` |
| `temper/tests/client/` | `src/translate.rs:321`, `src/translate/forge.rs:448` |
| `temper/tests/system/` | `src/composed.rs:884` |
| `temper/tests/codecs/` | `tests/suite/fuzzy/store.rs:42` |

Store goldens affected by record changes are under `temper/crates/temper-protocol-store/golden/`, each with `_smallest.bin` and `_full.bin`: `record_demo`, `enum_demo_state_running`, `enum_demo_state_ended`, `enum_value_demo`, `record_demo_close`, `record_demo_remove_account`, `enum_demo_act_kind_close`, `enum_demo_act_kind_remove_account`, `record_demo_act`, `enum_value_demo_act`.

### `engine.acts` and its `close` grant

`temper/crates/temper-init/template/temper.json:3`:

```json
"acts": [{"connector": 1, "may": ["close", "remove_account"]}]
```

Types: `jig/crates/jig-domain-people/src/boundary.rs:673`, `:688`:

```rust
pub enum ActKind {
    Close, AddKey, RemoveAccount, Adopt, MakeProject,
}
pub struct ActGrant {
    pub connector: u16,
    pub may: Box<[ActKind]>,
}
```

- Parsing: `jig/crates/jig-protocol-config/src/acts.rs:8`, `parse(...) -> Box<[ActGrant]>`; string mapping `:59`.
- Configuration field: `jig/crates/jig-protocol-config/src/lib.rs:58`, `acts: Box<[jig_domain_people::ActGrant]>`.
- Translation to core grants: `src/core.rs:72`.
- Core field: `jig/crates/jig-domain-core/src/lib.rs:161`, `grants`.
- Admission: `jig/crates/jig-domain-people/src/domain/acts.rs:106` checks matching connector and act kind; no grant → `Refusal::NotOffered`.

**Reverse dependencies of grant configuration/type:**

| Directory | Files |
|---|---|
| `jig/crates/jig-domain-people/src/` | `boundary.rs:673`, `lib.rs:104`, `domain.rs:145`, `domain/acts.rs:55`, `tests/act_charges.rs:49` |
| `jig/crates/jig-protocol-config/src/` | `lib.rs:58`, `core.rs:72`, `acts.rs:4`, `tests/acts.rs:5` |
| `jig/crates/jig-domain-core/src/` | `lib.rs:161` |
| `jig/tests/people/` | `src/world.rs:149`, `tests/suite/acts.rs:1`, `tests/suite/memory.rs:21` |
| `jig/tests/core/` | `tests/suite/deployment_acts.rs:23` |
| `jig/tests/ops/` | `src/conformance/config.rs:115` |
| `temper/crates/temper-init/` | `template/temper.json:3`, `src/tests/documents.rs:92` |
| `temper/crates/temper-service/src/` | `tests/setup.rs:1693` |
| `temper/crates/temper-engine-domain/src/` | `route/demo.rs:326` |
| `temper/tests/engine/domain/` | `tests/suite/demo.rs:21` |

No corresponding grant consumers in smith or skein.

## 2. “Ended with the playground” and another forge connecting

### Selection and closure

`temper/crates/temper-engine-domain/src/route/demo.rs:250`:

```rust
pub(crate) fn list_end_acts(domain: &mut Domain)
```

It snapshots:

- tasks in `demo.adoption.project`;
- phase `Waiting`, `Active(_)` or `Held { .. }`;
- `domain.core.conversation_requester(row).is_some()`;
- one deployment-held account of `demo.provider`.

It excludes tasks already `Closing` or `Ended`. Selection happens once; replay uses saved rows.

Act key function, same file `:241`:

```rust
fn end_key(number: u64, purpose: &[u8]) -> [u8; 16]
```

Hash input: `"temper demo end\0"`, big-endian number, `close` or `remove`; first 16 digest bytes.

Sending, same file `:285`:

```rust
pub(crate) fn send_end_act(domain: &mut Domain, row: forge::DemoAct)
```

For a close, it reads the first offered conversation template’s current grace:

```rust
people::Act::Close {
    project,
    task,
    grace,
    reason: b"Ended with the playground".as_slice().into(),
}
```

It sends `core::Event::Act` with `connector: domain.config.forge_connector`, the saved key and `carried: None`.

Actual close decision: `jig/crates/jig-domain-core/src/routing/deployment_acts.rs:95`:

```rust
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

It requires a live person-requested conversation in that project. At `:123` it constructs:

1. `tasks::Operation::Cancel` for every delegate, `grace: None`;
2. `Cancel` for the conversation with its selected grace;
3. `tasks::Event::Compound`, `by: tasks::Party::Deployment { project }`.

The client formats the retained cancellation:

- `temper/crates/temper-client-view/src/pages/chat.rs:354`, `:390`: `Ending::Cancelled` / `OwnerChipEnding::Cancelled`.
- Same file `:416`: timed cancellation.
- `temper/crates/temper-client-view/src/words.rs:2494`: `TimeCancelled` → `"Cancelled HH:MM · REASON"`.

**Literal reason reverse dependencies:** `temper/crates/temper-engine-domain/src/route/demo.rs:300`; `temper/tests/engine/domain/tests/suite/demo.rs:566`; `temper/crates/temper/tests/process.rs:2376`; `temper/crates/temper/tests/qa_all/qa_first_run.rs:178`. The generic formatting files above consume its retained value.

### Another forge’s connect

The exact decision is question 1’s `demo::connected`, invoked by connector `Event::Connection::Connected`. It does not require the other forge to be real: it requires a **different configured forge**. That matches `temper/docs/design/protocol/setup.md:2504`.

Tests:

| File | Tests |
|---|---|
| `temper/tests/engine/domain/tests/suite/demo.rs:324` | `another_connected_forge_ends_the_demo_persists_and_stops_once_and_restart_asks_no_group()` |
| Same file `:384` | `another_forge_connecting_answers_a_held_demo_start_not_offered()` |
| Same file `:548` | `demo_end_closes_chats_with_delegates_and_removes_its_account_erasing_answered_rows()` |
| Same file `:599` | `demo_end_selects_only_persons_live_conversations_and_never_chats_opened_afterward()` |
| Same file `:673` | `demo_end_committed_before_its_answer_is_resent_with_the_same_key_and_closed_once()` |
| Same file `:694` | `demo_end_unknown_and_ended_answers_erase_the_kept_act()` |
| Same file `:730` | `demo_end_without_the_grant_serves_keeps_names_and_resends_until_a_granted_restart()` |
| Same file `:750`, `:775`, `:805` | Already-closing chat, missing owner, changed conversation grace |
| `temper/crates/temper-engine-domain-forge/src/tests/demo.rs:496` | `demo_acts_are_saved_before_send_restored_kept_and_erased_on_answer()` |

### Box stop path

- `temper/crates/temper-service/src/route.rs:15`: `Request::StopGroup` → box link.
- `temper/crates/temper-service/src/route/box_link.rs:90`:

  ```rust
  pub(crate) fn stop(&mut self, forge: u16, out: &mut Queue<Out>)
  ```

  Holds the stop behind an outstanding start; emits `stop N\n` at `:163`.

- `temper/crates/temper-box/src/pipe.rs:46`: `Command::{Start(u16), Stop(u16), Unknown}`; `command(line: &[u8]) -> Command` at `:52`.
- `temper/crates/temper-box/src/host.rs:961`: `Command::Stop` → `Input::Stop`.
- `temper/crates/temper-box/src/machine.rs:622`: `stop_group`; `:652`: `stop_group_next`; closes steps, returns them to `Dormant`, emits `Said::Stopped`.
- `temper/crates/temper-box/src/pipe.rs:90`: `stopped(forge: u16) -> Box<[u8]>`.

There is **no `remove` pipe command**. Stop does not remove a forge directory or group record.

**Additional reverse dependencies for `StopGroup` / box stop:**

| Directory | Files |
|---|---|
| `temper/crates/temper-engine-domain/src/` | `boundary.rs:540`, `domain.rs:749`, `decision.rs:616`, `route/forge_route.rs:2993`, `tests/resume.rs:483`, `tests/setup.rs:84` |
| `temper/crates/temper-engine-domain-forge/src/` | `boundary.rs:1376`, `demo.rs:154`, `tests.rs:346`, `tests/demo.rs:318` |
| `temper/crates/temper-service/src/` | `route.rs:15`, `route/box_link.rs:90`, `tests/engine.rs:480`, `tests/fixtures.rs:745`, `tests/setup_github.rs:97`, `tests/box_link.rs:355`, `tests/qa_box_link.rs:1` |
| `temper/crates/temper-box/src/` | `pipe.rs:46`, `host.rs:628`, `machine.rs:266`, `terminal.rs:18`, `tests/pipe.rs:31`, `tests/machine.rs:994`, `tests/terminal.rs:84` |
| `temper/tests/client/src/` | `translate.rs:250`, `translate/forge.rs:425` |
| `temper/tests/system/src/` | `forge.rs:429`, `composed.rs:748` |
| `temper/tests/engine/domain/src/` | `walking.rs:786`, `direct.rs:207`, `protocol.rs:294`, `roles.rs:532`, `forge_routes.rs:1132`, `escalation.rs:673` |
| `temper/tests/engine/forge/` | `src/world.rs:781`, `tests/suite/simulation.rs:821` |
| `temper/tests/engine/protocol-forge/` | `src/protocol.rs:507`, `tests/suite/protocol.rs:510` |

The remaining retirement readers are question 1’s `forge_ended` paths and tests; no smith/skein/jig consumer knows temper’s retirement state.

## 3. Playground project, provider, fake LLM and account creation

### Where each is made today

| Item | Location and current behavior |
|---|---|
| Seed project | `temper/crates/temper-init/template/temper.json:165`: `engine.projects = [{ "number": 1, "policy": 1 }]` |
| Demo project reference | Same file `:623`: `"project": 1` |
| Project configuration → core | `jig/crates/jig-protocol-config/src/core.rs:13`: constructs `SeedProject` values |
| Project creation at first start | `jig/crates/jig-domain-core/src/routing/first_start.rs:74`, `project_rows`; `:87`, `start_seed`; `:126`, `people_rows` saves policy and role rows |
| Scripted provider | Template `:60`: provider name `demo`, number `3`, model `demo`, key source |
| Endpoint | Template `:84`: endpoint number `3`, dialect `3`, provider `demo`, plaintext, socket `$llm` |
| Model | Template `:111`: name `demo`, endpoint `demo` |
| Fake LLM | Template `:358`: `application.demo.llm`, listener `$llm`, provider `3`, key path, scripts |
| Fake forge | Template `:305`: forge `1`, owner `playground`, repository `widget`, landing `main` |
| Account creation | `temper/crates/temper-engine-domain/src/route/demo.rs:53`–`:87`: skips an existing deployment account of provider `3`, otherwise requests `AccountChange::AddKey` |
| Actual account row | `jig/crates/jig-domain-accounts/src/keys.rs:123`, `granted(...)`; `:135` constructs `Account { holder, provider, uses: None, plan: false, ... }` |
| Adoption | Root demo `progress` at `:93`–`:126`; adoption defaults come from `temper/crates/temper-service/src/settings.rs:33` |

**A project named `Playground` made by start: not found.** The current project is the seeded project `1`. This is also asserted by `temper/crates/temper-init/src/tests/documents.rs:474`, `the_template_keeps_project_one_without_settings_members()`.

The welcome’s new `Default` project is a different path:

- `temper/crates/temper-client-domain/src/forge/forges.rs:377`:

  ```rust
  pub const AT_ONCE_ORG: &[u8] = b"work";
  pub const AT_ONCE_REPOSITORY: &[u8] = b"hello";
  pub const AT_ONCE_PROJECT: &[u8] = b"Default";
  ```

- `set_up_at_once(...)`, same file `:381`, uses `SetUpProject::New`.
- Common client project-then-adopt state: `temper/crates/temper-client-domain/src/forge/start.rs:6`, `StartIn`; `StartIn::ask(&self, limits: &super::Limits) -> Option<Ask>` at `:17` sends `c::Ask::MakeProject { name, policy: 1 }`.

### Configuration readers and fake process group

Two readers exist:

1. `temper/crates/temper-fake/src/demo.rs:47`:

   ```rust
   pub struct Demo {
       llm: llm::Document,
       provider: u32,
       forge: u16,
       project: u32,
   }
   pub fn read(
       document: &Json, at: u32, facts: &Facts<'_>
   ) -> Result<Option<Demo>, Refusal>
   ```

   `Facts` at `:74` contains LLM configuration, configured project numbers, setup entries, listeners and directories. It validates numeric project membership at `:141`.

2. `temper/crates/temper-service/src/startup/application/demo.rs:26`, `parse(...) -> Option<settings::Demo>`.

   Public service shape, `temper/crates/temper-service/src/settings.rs:7`:

   ```rust
   pub struct Demo {
       pub project: u32,
       pub forge: u16,
       pub provider: u32,
       pub owner: Box<[u8]>,
       pub name: Box<[u8]>,
       pub cues: Box<[Box<[u8]>]>,
       pub landing: Box<[u8]>,
       pub key_file: Box<[u8]>,
       pub state: Option<temper_engine_domain_forge::DemoState>,
   }
   ```

`temper/crates/temper-fake/src/steps.rs:21`:

```rust
pub fn steps(
    setup: &Setup,
    demo: Option<&Demo>,
    flags: &[Box<[u8]>],
    listeners: Box<[core::net::SocketAddr]>,
    forgejo_listens: bool,
    first: Box<[Step]>,
) -> Steps
```

At `:73` it creates the asked `Playground` group: fake LLM, forge preparation, fake forge. `recorded: false`; start is requested again from engine state. Other fake forge groups run unasked.

### Account uses naming projects

Design: `jig/docs/design/domain/accounts.md:419`, section 4.2.

`jig/crates/jig-domain-accounts/src/records.rs:19`:

```rust
pub enum Use {
    Party(u64),
    Project(u32),
}
pub enum Holder {
    Deployment,
    Party(u64),
}
```

- `jig/crates/jig-domain-accounts/src/domain.rs:20`: `uses: Option<Box<[Use]>>`.
- `:38`: `pub fn uses(&self) -> Option<&[Use]>`.
- `jig/crates/jig-domain-accounts/src/changes.rs:110`: `Change::SetUses { account, uses }` validates and saves the account.
- `jig/crates/jig-domain-core/src/coverage.rs:16`:

  ```rust
  pub(crate) fn serves(
      people: &people::Domain,
      account: &accounts::Account,
      payer: accounts::Use,
  ) -> bool
  ```

  Explicit uses replace defaults. `uses: None` on a non-plan deployment account serves every known payer (`:43`–`:48`).

**The current scripted account has `uses: None`; restricting it to the playground project is not implemented.** Also, `Use::Project(project)` covers a project payer; party-funded work is separately `Use::Party(party)` (`coverage.rs:7`).

### Reverse dependencies: demo configuration and provider/account setup

Question 1’s `DemoConfig`/start lists apply, plus:

| Directory | Files |
|---|---|
| `temper/crates/temper-init/` | `template/temper.json:31`, `src/template.rs:2`, `src/lib.rs:116`, `src/tests/documents.rs:67`, `src/tests/files.rs:30` |
| `temper/crates/temper-fake/src/` | `demo.rs:47`, `steps.rs:2`, `lib.rs:10`, `backends.rs:77`, `check.rs:142`, `tests.rs:597`, `tests/demo.rs:2`, `tests/steps.rs:2`, `backends/tests.rs:303` |
| `temper/crates/temper-service/src/` | `settings.rs:7`, `startup.rs:909`, `startup/application.rs:23`, `startup/application/demo.rs:26`, `startup/checks.rs:667`, `startup/problems.rs:190`, `system.rs:223`, `engine.rs:18`, `forge.rs:13`, `family.rs:30`, `route.rs:10`, `clients.rs:454`, `setup/settings.rs:255`, `clients/demo_tests.rs:28` |
| `temper/crates/temper-service/src/tests/` | `family.rs:170`, `setup.rs:1631`, `store.rs:124`, `fixtures.rs:934`, `fixtures/http.rs:112`, `setup_admin.rs:617`, `setup_github.rs:121`, `clients.rs:256`, `welcome_policy.rs:65`, `checks.rs:367`, `stock.rs:49`, `startup.rs:294` |
| `temper/crates/temper/src/` | `setup.rs:14`, `box_command.rs:63`, `fake_backends.rs:283`, `fake.rs:391`, `init.rs:192`, `tests.rs:186`, `init/tests.rs:168`, `fake_backends/import.rs:22` |
| `temper/crates/temper/tests/` | `process.rs:2116`, `suite/startup.rs:22`, `suite/fake_opening.rs:192`, `suite/fake.rs:545`, `suite/forge_land.rs:401`, `fake_backends/oauth.rs:67`, `qa_all/qa_adoption.rs:259`, `qa_all/qa_forge_setup.rs:117` |
| `temper/crates/temper/tests/support/` | `configuration.rs:162`, `first_repository.rs:221`, `sign_in.rs:56`, `mod.rs:102`, `restart.rs:153` |
| `temper/tests/client-browser/src/` | `configuration.rs:165` |
| `jig/crates/jig-protocol-config/src/` | `core.rs:13`, `projects.rs:11`, `projects.rs:145`, `domain.rs:235`, `defaults.rs:31` |
| `jig/crates/jig-domain-core/src/` | `routing/first_start.rs:64` |

JSON documents explicitly carrying `application.demo`: template above; `temper/docs/examples/owned.json:246`, `temper/crates/temper/tests/support/github/engine.json:197`, `temper/crates/temper/tests/fixtures/cluster/engine.json:234` carry `null`.

**Additional direct project-use consumers/constructors:**

| Directory | Files |
|---|---|
| `jig/crates/jig-domain-accounts/src/` | `records.rs:19`, `domain.rs:20`, `keys.rs:139`, `programs.rs:84`, `signing.rs:261`, `changes.rs:110`, `ordering.rs:12`, `restore.rs:22`, `selection.rs:275` |
| `jig/crates/jig-domain-core/src/` | `coverage.rs:7`, `routing/account_requests.rs:100`, `tests/coverage.rs:73`, `tests/run_choice.rs:546`, `tests/accounts_document.rs:373`, `tests/account_admission.rs:168` |
| `jig/crates/jig-domain-people/src/` | `boundary.rs:1002`, `domain.rs:1302`, `domain/start/fit.rs:328`, `tests/start.rs:961` |
| `jig/crates/jig-protocol-engine-clients/src/` | `translate/accounts.rs:13`, `tests/accounts.rs:48` |
| `jig/crates/jig-client-protocol-link/src/` | `translate/accounts.rs:8`, `tests/owner_accounts.rs:53` |
| `jig/crates/jig-protocol-store/src/` | `codec/accounts.rs:164`, `tests/codec.rs:901` |
| `jig/crates/jig-client-view/src/` | `owner_deployment/accounts.rs:61`, `account_catalogue.rs:67`, `tests/owner_deployment.rs:802` |
| `jig/tests/` | `client/src/joint/wire/catalogue.rs:73`, `client/src/joint/wire/accounts.rs:11`, `accounts/tests/suite/changes.rs:277`, `client-wire/tests/suite/account_keys.rs:14`, `core/tests/suite/credentials.rs:370`, `client-protocol/tests/client/named_accounts.rs:160` |
| `jig/testing/` | `jig-client-test-records/src/accounts.rs:60`, `jig-client-scripted-engine/src/payload.rs:166` |
| `temper/crates/temper-client-view/src/` | `pages/accounts/rows.rs:447`, `choice.rs:260`, `tests/choice.rs:484` |
| `temper/crates/temper-client-tools/src/` | `jig/render/document/offers.rs:86`, `jig/render/document/accounts.rs:270`, `tests/fixtures/accounts.rs:24`, `tests/jig/account_waits.rs:308` |

## 4. Client modules, welcome, halves, banners and pages

### Module map

| Feature | Domain | View / presentation |
|---|---|---|
| Addresses and welcome steps | `temper/crates/temper-client-domain/src/address.rs:5`, `:19` | Welcome renderer below |
| Welcome decision | `src/pages/welcome.rs:22`, `decision(root: &Domain) -> Option<bool>`; `:55`, `model_usable(root: &Domain) -> Option<bool>` | `temper/crates/temper-client-view/src/pages/welcome.rs:120`, `main(...)` |
| Welcome automatic navigation | Domain `src/domain.rs:145`, `welcome(root, out)` | Root chooses welcome renderer at view `src/root.rs:144` |
| Playground choice | Domain `src/route.rs:337`, `playground(root, env, out)` | Welcome `src/pages/welcome.rs:149`, `Act::Playground` |
| Demo read/start/halves/cleanup | Domain `src/forge/demo.rs` | Frame, chat/inbox/task and account renderers |
| Demo answer decoding | Domain `src/forge/family.rs:164` | — |
| Demo banner and setup strip | Domain `src/forge/listed.rs:98`, `setup(root: &Domain) -> Option<Setup>` | View `src/frame.rs:352`, `demo_banner(...)` |
| Forges watch/rows/setup | Domain `src/forge/forges.rs:132`, `:204`, `:213`, `:319` | View `src/forge/forges.rs:20`, `main(...)` |
| Connect a forge / welcome Forge body | Domain `src/forge/connect.rs`, `forges.rs`, `github.rs`, `picker.rs`, `admin.rs`, `start.rs` | View `src/forge/connect.rs:35` |
| Accounts state/operations | Domain `src/pages/accounts.rs:46`, `src/pages/accounts/acts.rs:7`; `request.rs`, `paste.rs`, `ways.rs` | View `src/pages/accounts.rs:329`; `pages/accounts/rows.rs`; `accounts.rs`; `accounts/words.rs` |
| Connect model / welcome Model body | Domain `src/pages/connect_model.rs:32` | View `src/pages/connect_model.rs:227` |
| CLI playground start | `temper/crates/temper-client-tools/src/forges.rs:171`, `DEMO_START`; `:353` routes it | `src/demo.rs:94`, `print(outcome: &Outcome, as_json: bool, kept: bool, limits: &Limits) -> jig::Printed` |
| CLI account operations | Tools `src/jig/runner/accounts.rs` | Tools `src/jig/render/document/accounts.rs`, `account_lines.rs`, `account_programs.rs` |
| Web shell | `temper/crates/temper-client-shell-web/src/host.rs:43`, `Host`; shared domain/view | `src/lib.rs:51`, browser startup; `temper.css`, `task-pages.css` |

### Welcome steps and `/welcome?at=forge`

`temper/crates/temper-client-domain/src/address.rs:19`:

```rust
pub enum Step {
    Welcome,
    Forge { forge: Option<Choice> },
    Model { way: Option<ModelWay> },
    Ready,
}
```

- `print(address: &Address) -> Box<[u8]>`, `:85`.
- `Step::Forge` prints `/welcome?at=forge`, optional `&forge=N`, `:91`.
- `parse(path: &[u8]) -> Place`, `:144`.
- `welcome_step(query: &[u8]) -> Step`, `:337`.

Current completion rule, `pages/welcome.rs:22`:

- waits for demo and model facts;
- non-owner: usable real model **or running demo**;
- owner: running demo completes welcome; otherwise requires another connected forge and usable real model;
- it does not require a first repository adopted as part of that completion predicate.

`domain.rs:145` applies the decision **only at `Step::Welcome`**, once per opening, and replaces the URL with Chats when complete.

`route.rs:337` sends start, selects `demo.project`, then opens Chats immediately. The Chats page represents pending/failing start.

Welcome view signature:

```rust
// temper/crates/temper-client-view/src/pages/welcome.rs:120
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    step: Step,
    text_per_node: u32,
    sign_in: &crate::forge_sign_in::Forges,
)
```

### Model and forge halves

Domain types:

```rust
// temper/crates/temper-client-domain/src/forge/vocabulary.rs:123
pub struct Demo {
    pub state: super::demo::DemoState,
    pub project: u32,
    pub forge: u16,
    pub provider: u32,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub cues: Box<[Box<[u8]>]>,
}

// src/forge/demo.rs:177
pub struct Halves {
    pub model: bool,
    pub forge: bool,
}
```

`forge/demo.rs:182`:

```rust
pub fn halves(
    answer: Option<&Demo>,
    accounts: Option<&c::OwnerAccounts>,
    deployment: Option<&c::OwnerDeployment>,
    forges: &super::forges::State,
) -> Halves
```

- Forge half ends when another forge row is connected.
- Model half ends when another provider has usable sources.
- Cut account documents prevent deciding the model half.
- Owner-held deployment accounts count.
- `current(...)` at `:217` also applies sticky `model_ended` / `forge_ended`.
- `RetiredDemo { forge: u16, provider: u32, project: u32 }`, `:29`.
- Ended wire answer sets both halves ended and retains identity, `:113`–`:127`.
- `Domain::demo(&self) -> Option<(&forge::Demo, forge::demo::Halves)>`, `src/domain.rs:76`, returns only while at least one half remains.

**Client cleanup that the draft removes:** `forge/demo.rs:387`, `remove(side, client, out)`, sends:

```rust
c::Ask::AccountChange {
    holder: c::AccountHolder::Deployment,
    change: c::AccountChange::Remove { account: account.number },
}
```

This occurs for the **model half’s end**, once, for an owner. If the forge ended, cleanup is left to the engine. Tests: `src/tests/demo.rs:676`, `:743`, `:788`, `:806`, `:818`.

### Banner and setup strip

`temper/crates/temper-client-view/src/frame.rs:352`:

```rust
pub(crate) fn demo_banner(
    b: &mut Builder, root: &Domain,
    words: &crate::words::Words, bindings: &mut Bindings
)
```

Current behavior:

- Absent on Welcome, Accounts, ConnectModel, ConnectForge and Forges.
- Setup strip takes priority when `listed::setup(root)` returns a value.
- Otherwise chooses `DemoBoth`, `DemoModel` or `DemoForge`.
- Links independently to ConnectModel and ConnectForge.

Words, `src/words.rs`:

- `:2563`: `"This is the playground: a scripted model and a playground forge."`
- `:2564`: forge-only wording.
- `:2581`: model-only wording.
- `:2200`–`:2202`: setup strip missing forge/model wording.

Setup shape, domain `src/forge/listed.rs:91`:

```rust
pub struct Setup {
    pub playground: bool,
    pub forge: bool,
    pub model: bool,
    pub step: Step,
}
```

`setup` at `:98` chooses Forge, Model or Ready from current facts.

### Forges, Accounts and project settings

- Forges domain row: `temper/crates/temper-client-domain/src/forge/forges.rs:132`, `Row`; includes `number`, `kind`, `mode`, `connection`, `offer`, `first`, `installations`, `ended`, `app`, `web`, `repositories`, `owned`, `admin`, `ended_stop`.
- `State::{Unknown, Rows(Box<[Row]>), Ended}`, same file `:213`.
- `listed::personal(row: &Row, demo: Option<&Demo>) -> bool`, `forge/listed.rs:19`, excludes fake/demo rows.
- `listed::rows(...) -> List<&Row>`, `:39`; `playground(root) -> Option<(&Demo, &Row)>`, `:76`.
- Forges view `src/forge/forges.rs:32` draws personal rows and Add/Connect links; `:52` draws playground separately.
- Accounts view `src/pages/accounts.rs:346`, `:371` excludes demo provider from own-account rows; `:399` draws separate Playground disclosure while `halves.model`.
- Account ways `temper/crates/temper-client-domain/src/pages/accounts/ways.rs:21`:

  ```rust
  pub fn ways(
      providers: &List<c::ProviderOffer>,
      demo: Option<u32>,
      holder: c::AccountHolder,
  ) -> List<Way>
  ```

  Excludes the demo provider.

**Project settings page: not found.** `Address` has no project/settings variant. `temper/docs/design/client/README.md:4035` lists the project page and its settings as later; `:4048` leaves adoption as a standalone page until that exists. Settings CLI operations do exist, but are not a project settings page.

### Tests, fixtures, snapshots and journey HTML

| Layer | Concrete tests / fixtures |
|---|---|
| Domain | `temper/crates/temper-client-domain/src/tests/demo.rs:58`, `:163`, `:382`, `:676`, `:743`; `tests/pages/welcome.rs`; `tests/forges.rs`; `tests/connect.rs`; `tests/admin.rs`; `tests/github_app.rs`; `tests/pages/accounts.rs`; `tests/pages/connect_model.rs`; `tests/address.rs` |
| View | `temper/crates/temper-client-view/src/tests/welcome.rs:39`; `tests/demo.rs:6`; `tests/demo_start.rs:9`; `tests/forges.rs:155`; `tests/connect_forge.rs`; `tests/connect_model.rs`; `tests/accounts.rs`; shared `tests/fixtures.rs` |
| View assertion shape | Semantic tree roles, names, states and decoded actions; e.g. `tests/demo.rs:25`–`:39`. These pages are not primarily asserted by HTML snapshots |
| Composed client world | `temper/tests/client/tests/suite/welcome.rs:12`; `demo_start.rs:98`, `:237`, `:261`, `:328`; `forges.rs:332`; `accounts_page.rs:69`; `accounts.rs`; `connect_forge.rs`; `connect_model.rs` |
| Client QA | `temper/tests/client/tests/qa_all/qa_welcome.rs`, `qa_welcome_m3.rs`, `qa_welcome_m3_owned.rs`, `qa_welcome_m4.rs`, `qa_first_run_hooks.rs`, `qa_forges.rs`, `qa_accounts.rs` |
| Browser | `temper/tests/client-browser/tests/browser_all/browser_accounts.rs`; `browser_github.rs:104`; browser QA account-moving coverage in `tests/qa_all/qa_account_moving.rs` |
| Wire | `temper/crates/temper-client-protocol-wire/src/tests.rs:150`, `:292`, `:697`; `temper/tests/codecs/tests/suite/client_wire.rs:301`, `:331`; `fuzzy/client_wire.rs:30` |
| CLI | `temper/crates/temper-client-tools/src/tests/forges.rs:709`; `tests/jig/accounts.rs`; `tests/jig/account_acts.rs`; `tests/jig/projects.rs` |
| Web host | `temper/crates/temper-client-shell-web/src/host_tests.rs:229`, `:275`; `host_tests_departure.rs`; `host_tests_authorize.rs`; `journey/tests.rs` |
| Gallery | `temper/tools/gallery/src/samples.rs:81`, `worlds.rs:165`, `tests.rs:836` |

**Journey HTML is read by tests.**

`temper/docs/design/client/journeys/welcome-set-up/README.md:3` describes:

- `01.html`: welcome;
- `02.html`: Forge choices;
- `03.html`: Model after “Run on this machine”;
- `recording`: replay input.

`temper/tools/journeys/src/tests.rs:11`, `the_committed_journeys_are_their_print()`:

1. replays/reprints committed recordings;
2. enumerates printed pages;
3. reads and byte-compares committed and reprinted HTML (`:20`–`:27`);
4. compares shared CSS and journey JS too.

Related code: `temper/tools/journeys/src/lib.rs:61`, `reprint(...)`; `:97`, `print_one(...)`; `src/again.rs`; `src/print.rs`. `tests/suite/look_refresh.rs:11` explicitly names `welcome-set-up/01.html` and `02.html`.

### Reverse dependencies: client demo and halves

Question 1’s start/wire lists apply. Additional direct readers/constructors:

| Directory | Files |
|---|---|
| `temper/crates/temper-client-domain/src/` | `domain.rs:76`, `forge.rs:117`, `forge/vocabulary.rs:123`, `forge/demo.rs:2`, `forge/family.rs:164`, `forge/listed.rs:3`, `forge/connect.rs:3`, `forge/admin.rs:126`, `pages/welcome.rs:24`, `route.rs:320`, `limits.rs:31` |
| Same directory, tests | `tests/demo.rs:19`, `tests/connect.rs:68`, `tests/pages/welcome.rs:34`, `tests/pages/connect_model.rs:326` |
| `temper/crates/temper-client-view/src/` | `root.rs:83`, `pages.rs:16`, `frame.rs:127`, `choice.rs:65`, `pages/task.rs:66`, `pages/chats.rs:197`, `pages/chat.rs:209`, `pages/inbox.rs:215`, `pages/accounts.rs:399`, `pages/accounts/rows.rs:22`, `pages/connect_model.rs:291`, `pages/welcome.rs:81`, `accounts/words.rs:666`, `forge/connect.rs:56`, `forge/forges.rs:32`, `words.rs:426` |
| Same directory, tests | `tests/demo_start.rs:253`, `tests/demo.rs:324`, `tests/connect_forge.rs:428`, `tests/forges.rs:609`, `tests/change.rs:505`, `tests/connect_model.rs:473`, `tests/welcome.rs:613`, `tests/frame.rs:858`, `tests/choice.rs:1256`, `tests/fixtures.rs:266` |
| `temper/crates/temper-client-tools/src/` | `demo.rs:99`, `forges.rs:369` |
| `temper/crates/temper-client-shell-terminal/src/` | `tests.rs:62`, `tests/fixtures/chat.rs:136` |
| `temper/tests/client/` | `src/front.rs:28`, `src/front/account_acts.rs:63`, `tests/suite/demo_start.rs:100`, `tests/suite/change_panel.rs:517`, `tests/suite/welcome.rs:157`, `tests/qa_all/qa_forges.rs:1059`, `tests/qa_all/qa_welcome_m3.rs:63`, `tests/qa_all/qa_welcome_m4.rs:113`, `tests/qa_all/qa_first_run_hooks.rs:141` |
| `temper/tests/client-browser/src/` | `lib.rs:136`, `configuration.rs:154` |
| `temper/tools/gallery/src/` | `worlds.rs:270` |
| `temper/crates/temper-service/src/` | `family.rs:230`, `tests/family.rs:138` |

### Reverse dependencies: Welcome address/steps and strip

| Directory | Files |
|---|---|
| `temper/crates/temper-client-domain/src/` | `address.rs:88`, `domain.rs:149`, `pages.rs:333`, `route.rs:9`, `departure.rs:43`, `pages/connect_model.rs:32`, `forge/listed.rs:99`, `forge/picker.rs:138`, `forge/change.rs:26`, `tests/address.rs:141`, `tests/connect.rs:54`, `tests/pages/connect_model.rs:222`, `tests/pages/welcome.rs:20`, `tests/change.rs:16`, `tests/admin.rs:295` |
| `temper/crates/temper-client-view/src/` | `root.rs:144`, `frame.rs:169`, `choice.rs:47`, `words.rs:95`, `pages/welcome.rs:65`, `pages/connect_model.rs:227`, `forge/connect.rs:35`, `tests/demo_start.rs:340`, `tests/demo.rs:20`, `tests/connect_forge.rs:140`, `tests/forges.rs:163`, `tests/change.rs:194`, `tests/connect_model.rs:538`, `tests/welcome.rs:41`, `tests/fixtures.rs:265` |
| `temper/crates/temper-client-domain/tests/` | `qa_all/qa_change.rs:379` |
| `temper/crates/temper-client-view/tests/` | `qa_all/qa_change_page.rs:996` |
| `temper/tests/client/tests/` | `suite/demo_start.rs:145`, `suite/change_panel.rs:101`, `suite/connect_forge.rs:23`, `suite/connect_model.rs:215`, `suite/welcome.rs:36`, `qa_all/qa_forges.rs:283`, `qa_all/qa_welcome_m3.rs:56`, `qa_all/qa_welcome.rs:14`, `qa_all/qa_welcome_m3_owned.rs:104`, `qa_all/qa_welcome_m4.rs:61`, `qa_all/qa_first_run_hooks.rs:103`, `qa_all/qa_change_page.rs:181` |
| `temper/tests/client/src/` | `front/account_acts.rs:268` |
| `temper/tests/client-browser/tests/` | `browser_all/browser_github.rs:104` |
| `temper/tests/one-process/src/` | `m15/pages.rs:19` |
| `temper/crates/temper/tests/support/` | `person.rs:103`, `playground.rs:29` |
| `temper/tools/gallery/src/` | `samples.rs:81`, `worlds.rs:165`, `tests.rs:861` |

Forges-row consumers beyond those tables: `temper/crates/temper-client-domain/src/forge/words.rs:987`, `tests/github_app.rs:238`; `temper/crates/temper-client-tools/src/tests/forges.rs:977`; `temper/tests/client/tests/suite/forges.rs:316`, `tests/qa_all/qa_github_connect.rs:86`.

Accounts-page operation consumers are listed in question 5. No client code in smith or skein references these temper types.

Client wire goldens affected by changing demo/start shapes are under `temper/crates/temper-client-protocol-wire/golden/`, each `_smallest.bin` and `_full.bin`: `enum_request_start_demo`, `enum_read_demo`, `record_demo_answer`, `enum_demo_state_not_started`, `enum_demo_state_running`, `enum_demo_state_ended`, `enum_start_refusal_not_offered`, `record_start_refused`, `enum_start_failure_group`, `enum_start_failure_account`, `enum_start_failure_adoption`, `enum_start_failure_not_connected`, `record_start_failed`, `enum_outcome_start_refused`, `enum_outcome_start_failed`.

## 5. jig `RemoveAccount` as a deployment-act model

### Deployment act and decision

`jig/crates/jig-domain-people/src/boundary.rs:662`:

```rust
Act::RemoveAccount { account: u32 }
```

Associated protocol:

- `ActKey { connector: u16, key: [u8; 16] }`, `:637`.
- `ActKind::RemoveAccount`, `:676`.
- `ActGrant`, `:688`.

Admission/deduplication: `jig/crates/jig-domain-people/src/domain/acts.rs:68`:

```rust
pub(super) fn admit(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    key: ActKey,
    act: Act,
    out: &mut Queue<Request>,
)
```

It checks readiness, shape, retained answer/key conflict, active flight, connector grant, first owner and capacity; then emits `Request::Act`. `decided(...)` at `:151` saves permanent results as `Stored::Acted` before answering. `restore(...)` at `:191` restores retained outcomes. Remove-account answer-shape validation is `:315`.

Core route: `jig/crates/jig-domain-core/src/routing/deployment_acts.rs:71`:

```rust
people::Act::RemoveAccount { account: number } => {
    account(core, work, request, people::AccountChange::Remove { account: number });
}
```

`account(...)` at `:85` fixes the holder to `accounts::Holder::Deployment`, admits through `key_routes`, then sends `accounts::Event::Change`.

Account decision: `jig/crates/jig-domain-accounts/src/changes.rs:233`:

```rust
fn remove(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    holder: Holder,
    number: u32,
    out: &mut Queue<Request>,
)
```

It validates removability, removes account/source rows, cancels alarms, signs out signed-in sources, forgets keys, ends program sources, saves ordering and keeps the monotonically increasing counter. Immediate success is `Answer::Removed { confirmed: true, failed_hosts: [] }`; pending source cleanup completes in `signed_out(...)` at `:294`.

### Store and client wire are different envelopes

Deployment-act persistence:

- `jig/crates/jig-protocol-store/schema/people-v13.schema:304`:

  ```text
  record ActRemoveAccount { account: u32 }
  ```

- `:307`, `enum Act`.
- `jig/crates/jig-protocol-store/src/codec/people/acts.rs:4`, `encode(...)`; `:33`, `decode(...)`.
- Generated current type: `src/generated/people_v13.rs:13261`.
- Older generated/schema versions `people_v10`, `people_v11`, `people_v12` also contain it.

**There is no client `RemoveAccount` deployment-act request.** A person’s client sends:

```rust
Ask::AccountChange {
    holder: AccountHolder,
    change: AccountChange::Remove { account: u32 },
}
```

Locations:

| Layer | Location |
|---|---|
| Person-domain change | `jig/crates/jig-domain-people/src/boundary.rs:1009`, `:1033` |
| Client-domain contract | `jig/crates/jig-client-domain/src/contract/asks.rs:265`, `:340`, `:364` |
| Current client schema | `jig/crates/jig-client-protocol-wire/schema/client-v19.schema:946`, `AccountChangeRemove`; `:954`, `AccountChange`; `:955`, `AskAccountChange` |
| Client encoding | `jig/crates/jig-client-protocol-link/src/translate/asks.rs:771` |
| Client stored-request decoding | `src/translate/store.rs:298` |
| Engine decoding | `jig/crates/jig-protocol-engine-clients/src/translate/asks.rs:604` |
| Answer | `OwnerOutcome::AccountRemoved`; engine encoding `src/translate/answers.rs:297`; schema `client-v19.schema:389` |

Temper UI operation: `temper/crates/temper-client-domain/src/pages/accounts/acts.rs:13`, `Operation::RemoveAccount`; at `:157` maps to `AccountChange::Remove`. The confirmation renderer is `temper/crates/temper-client-view/src/pages/accounts.rs:628`.

### Reverse dependencies: deployment `Act::RemoveAccount`

| Directory | Files |
|---|---|
| `jig/crates/jig-domain-people/src/` | `boundary.rs:662`, `domain/acts.rs:51`, `domain/start/fit.rs:236`, `tests/start.rs:422` |
| `jig/crates/jig-domain-core/src/` | `routing/deployment_acts.rs:24` |
| `jig/crates/jig-protocol-config/src/` | `acts.rs:62`, `tests/acts.rs:19` |
| `jig/crates/jig-protocol-store/` | `schema/people-v10.schema:266`, `people-v11.schema:266`, `people-v12.schema:272`, `people-v13.schema:304`; `src/generated/people_v10.rs`, `people_v11.rs`, `people_v12.rs`, `people_v13.rs:13261`; `src/codec/people/acts.rs:23`, `src/tests/codec.rs:1347` |
| `jig/tests/people/` | `tests/suite/acts.rs:23`, `tests/suite/restore_open.rs:160`, `tests/suite/start_restore.rs:103` |
| `jig/tests/core/` | `tests/suite/deployment_acts.rs:28` |
| `temper/crates/temper-engine-domain/src/` | `route/demo.rs:302`, `store.rs:781` |
| `temper/crates/temper-service/src/` | `tests/setup.rs:1698` |
| `temper/tests/engine/domain/` | `tests/suite/demo.rs:26` |

The `DemoActKind::RemoveAccount` wrapper’s reverse dependencies are question 1’s kept-act list.

### Reverse dependencies: person/client account removal

| Directory | Files |
|---|---|
| `jig/crates/jig-domain-people/src/` | `boundary.rs:1033`, `domain/acts.rs:318` |
| `jig/crates/jig-domain-core/src/` | `routing/deployment_acts.rs:72`, `routing/key_routes.rs:92` |
| `jig/crates/jig-domain-accounts/src/` | `boundary.rs:512`, `changes.rs:102` |
| `jig/crates/jig-client-domain/src/` | `contract/asks.rs:364` |
| `jig/crates/jig-client-protocol-link/src/` | `translate/asks.rs:771`, `translate/store.rs:298`, `tests/accounts.rs:111` |
| `jig/crates/jig-protocol-engine-clients/src/` | `request.rs:343`, `translate/asks.rs:604` |
| `jig/crates/jig-protocol-store/src/` | `codec/people/asks/account_changes.rs:56` |
| `jig/tests/` | `people/tests/suite/account_changes.rs:38`, `people/src/world.rs:772`, `client/src/joint/requests.rs:279`, `client/src/joint/wire/asks.rs:412` |
| `temper/crates/temper-client-domain/src/` | `pages/accounts/acts.rs:13`, `forge/demo.rs:406`, `tests/pages/accounts.rs:434`, `tests/demo.rs:667` |
| `temper/crates/temper-client-view/src/` | `pages/accounts.rs:633`, `pages/accounts/rows.rs:254`, `accounts/words.rs:144`, `tests/accounts.rs:825` |
| `temper/crates/temper-client-tools/src/` | `jig/runner/accounts.rs:129`, `jig/render/outcome.rs:116`, `tests/jig/account_acts.rs:56` |
| `temper/crates/temper-client-shell-cli/src/` | `tests/fixtures/moves.rs:52` |
| `temper/tests/client/` | `src/front.rs:1092`, `src/front/account_acts.rs:100`, `tests/suite/accounts.rs:175` |

Historical client schemas: `client-v12-part1.schema` through `client-v18-part1.schema`, each `:54`; current `client-v19.schema:946`. Historical people-store schemas containing `AccountChangeRemove`: `people-v7.schema` through `people-v12.schema`, and `people-v13.schema:179`; their generated codecs carry the corresponding historical records.

No matching jig deployment-act or account-change consumers in smith or skein; similarly named `Act` types there are unrelated.

## 6. Temper process-tier first-run, playground and box tests

### Location and execution

- Package: `temper/crates/temper/Cargo.toml:2`, `temper`.
- Test binary: same manifest `:96`:

  ```toml
  [[test]]
  name = "process"
  path = "tests/process.rs"
  ```

- All tests below: `temper/crates/temper/tests/process.rs`.
- Harnesses: `temper/crates/temper/tests/support/boxed.rs`, `person.rs`, `ctl.rs`, `playground.rs`, `first_repository.rs`, `forgejo.rs`, `owned.rs`.

Run the tier from the checkout root:

```sh
tools/tiers/run temper process
```

Focused selection, preserving tier setup:

```sh
tools/tiers/run temper process -- -E 'test(=a_first_run_answers_on_the_demo)'
```

Runner implementation:

- `tools/tiers/run:179`: CLI accepts `TREE TIER [-- NEXTEST_ARGS...]`.
- `:207`: reads root nextest profiles.
- `:220`: reads package/prerequisite inventory.
- `:261`: builds prerequisites with `heavy cargo build`.
- `:277`: invokes `heavy cargo nextest run ... --profile process-temper --no-fail-fast`.
- `tools/tiers/packages.toml:43`: process package set is `temper`.
- `:94`: prerequisite `temper-fake-claude/fake-claude`, exported through `TEMPER_TEST_CLAUDE`.

Profile, `.config/nextest.toml:180`:

```toml
[profile.process-temper]
default-filter = "binary(=process) & (package(/^temper(-|$)/) | package(=git-transcripts))"
global-timeout = "25m"
test-threads = 1
```

CPU-measured mode is `tools/tiers/run --cpu DIR temper process`. Its separate budget, `tools/tiers/budgets.toml:53`: suite CPU `360s`, per-test CPU `60s`, wall hard limit `1080s`. The ordinary profile’s global timeout is `25m`; these are distinct settings.

### First-run and playground walks

| Line | Exact test/function | Walk |
|---|---|---|
| `:1527` | `a_fresh_init_s_first_chat_reads_its_forge()` | Fresh init, first chat reads widget’s main branch |
| `:1600` | `a_first_run_answers_on_the_demo()` | Calls `first_run(false)` |
| `:2100` | `a_first_run_answers_on_the_demo_and_a_second_finds_it_running()` | Calls `first_run(true)` |
| `:2439` | `first_run(restart: bool)` | First box: welcome before fakes start, playground choice, scripted CSV change through CI/review/landing; second box on retained directories: dark-theme scripted answer; checks same bot and drained processes |
| `:2104` | `two_first_runs_one_after_the_other_choose_their_own_ports()` | Independent first-run boxes choose independent ports |
| `:2192` | `a_first_run_ctl_demo_start_answers_running_twice()` | Sign in, two `playground start` calls, both `running` |
| `:2224` | `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account()` | Three box runs: start/chat/account; configure another fake forge; third reload verifies chat cancellation, account absence, no demo forge row, start refused and demo group dormant |
| `:2311` | `playground_end_steps(run: &str)` | CLI steps for those three runs |
| `:2370` | `assert_playground_chat_and_account_ended(...)` | Checks retained `"Ended with the playground"` cancellation |
| `:1783` | `a_first_run_plays_the_demo_then_sets_up_a_forgejo_here_with_no_restart()` | Starts playground, runs scripted chat, sets up owned Forgejo, checks runtime demo retirement, creates project `2`, runs real git/Forgejo change, checks binaries unchanged in cache |
| `:1606` | `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart()` | Owned first-run setup, change, CI/review, landing and retained restart |
| `:4553` | `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge()` | `forge setup 3 --playground`, adopts copied widget on owned forge, worker reads source |
| `:249` | `a_welcome_sign_in_with_chatgpt_returns_to_ready_and_its_account_survives_a_restart()` | Welcome Model sign-in/Ready and retained account |

Native scripted person walks:

- `temper/crates/temper/tests/support/person.rs:159`: `demo_chat(...) -> Script`.
- `:206`: `demo_change() -> Script`.
- `:246`: `playground_opening(...)`.
- `:269`: `welcome_open() -> Script`.
- `:283`: `demo_start_answered() -> Script`.
- `temper/crates/temper/tests/support/playground.rs:74`: `retired_reload(config: &Path, client: &Path)`, verifies fresh client no longer offers playground/demo model after durable retirement.

### Box/process walks

| Line | Exact test |
|---|---|
| `:140` | `the_box_finishes_its_restart_policy_after_a_refused_restore()` |
| `:1217` | `temper_box_prints_its_link_and_stops_its_processes()` |
| `:1316` | `temper_box_restarts_a_killed_forgejo_while_the_engine_serves()` |
| `:3830` | `a_default_box_with_a_sign_in_registration_starts_without_dns()` |
| `:4073` | `a_story_box_and_its_steps_die_when_the_test_process_is_killed()` |

Harness entry points in `temper/crates/temper/tests/support/boxed.rs`:

- `:25`: `Plan`.
- `:185`: `run_command(...) -> Outcome`.
- `:372`: `run_story<T>(...) -> StoryOutcome<T>`.
- `:690`: `run_ctl<T>(...)`.
- `:711`: `run_ctl_with_exits<T>(...)`.
- `:796`: `run_ctl_with_llm<T>(...)`.
- `:522`: engine restart helper.

**Not found:** a process test implementing playground reset/deletion, a box killed during reset, or a first setup killed at each setup step and then removed/restarted. Existing tests cover completed starts/restarts, retirement, adoption interrupted by a signal (`process.rs:785`), and other engine/worker restart paths; they do not yet walk the draft’s removal/reset behavior.