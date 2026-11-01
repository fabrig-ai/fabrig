Read-only survey; no cargo, builds, tests, or writes.

References use these directory prefixes; each prefix also identifies the crate:

- `F` = `temper/crates/temper-engine-domain-forge`
- `E` = `temper/crates/temper-engine-domain`
- `S` = `temper/crates/temper-service`
- `P` = `temper/crates/temper-protocol-store`
- `R` = `temper/crates/temper-protocol-records`
- `C` = `temper/crates/temper-client-domain`
- `W` = `temper/crates/temper-client-protocol-wire`
- `V` = `temper/crates/temper-client-view`
- `T` = `temper/crates/temper-client-tools`

`F/src/domain.rs:120`, for example, expands to `temper/crates/temper-engine-domain-forge/src/domain.rs:120`.

**1. Runtime state, durable families, limits, and adding a family**

- **Runtime ownership:** `F/src/domain.rs:119` — `Domain`; `configured: Map<u16, Configured>` at `:120`. Construction copies supplied configuration in `Domain::new`, `:492`, with map capacity `l.client.repositories`, `:507`.
- `F/src/boundary.rs:298` — `Configured`: `admin_connect`, bot login, owned details, git/web addresses, read-only flag, number, kind, mode, asked-start flag.
- `F/src/boundary.rs:276` — `Kind::{Forgejo, GitHub}`; `:282` — `Mode::{Connected, Managed, Owned, Fake}`.
- `F/src/domain.rs:115` — `ConnectionState = (Connection, Option<FailedStep>)`; `Domain.connections` at `:128`, `first` at `:129`, `ended` at `:130`, `apps` at `:131`, installations at `:135`.
- `F/src/boundary.rs:930` — `Connection::{NotConnected, NotSetUp, SettingUp { step }, SetUpFailed { why }, Connecting, Connected { id, login }, Stopped { reason }}`.
- `F/src/boundary.rs:955` — `SetUpStep::{Fetch, Prepare, Run}`; `:963` — `FailedStep::{Fetch, Prepare, Run, Connect}`.
- These are volatile. `Domain::connection`, `F/src/domain.rs:212`, defaults to `NotConnected` and masks an ended demo forge. There is no durable connection-state family.
- `F/src/boundary.rs:362` — `First`: playground flag, organization, repository, project, creation state, label. `Domain.first` is volatile; `first_made`/`first_failed`, `F/src/domain.rs:223`, `:231`, update it.
- **No durable forge record, forge-number allocator/high-water record, general `RemoveForge`, `Playground`, `StartPlayground`, or `EndPlayground` exists.** Current numbers arrive through `Configured`.

- **Complete `Stored` variant set:** `F/src/boundary.rs:802` — `Call`, `ProposedEffect`, `Repository`, `Hold`, `Names`, `Subscription`, `BranchHead`, `PullState`, `Ci`, `Landed`, `Entry`, `Client`, `Change`, `Issue`, `Release`, `Application`, `Writer`, `Demo`, `Binding`, `DemoAct`, `Flight`, `Landing`, `Receipt`.
- `F/src/boundary.rs:883` — corresponding `Key` enum; `F/src/lib.rs:222` — `stored_key`; `:258` — `stored_bytes`. Both are integration points for another family.
- **Limits:** `F/src/limits.rs:17` — `Limits`; existing counts include repositories, tasks, holds, subscriptions, entries, named calls, changes, issues, adoptions, applications, bindings, landing rows, and child limits. **There is no `Limits.forges`.**
- `F/src/limits.rs:69` — `valid`; `:101` — `connection_valid`; `:107` — `demo_act_bound = tasks + 1`.
- `F/src/domain.rs:197` — `Domain::connection_limits(forges, installations)` separately sizes volatile connection/first/ended/App tables and their installation product.
- `S/src/engine.rs:137` supplies those counts from `forge_component.forges`/`installations`.
- `S/src/limits.rs:78` — the separate `Forges` limit type contains only installations and log origins; it is not a count of retained forge records.

- **Demo family:** `F/src/boundary.rs:854` — `Stored::Demo`; `:908` — `Key::Demo(u16)`; `:1613` — `Demo { forge, state }`; `:1619` — `DemoState::{Running, Ended}`.
- `F/src/demo.rs:82` — `start`, saves `Running` at `:106`; `:137` — `connected`, saves `Ended` at `:148` when another configured forge connects. It does not erase the demo row.
- `F/src/demo.rs:26` — `configure_demo`; `:33` — `demo`; `:37` — `demo_state`; `:42` — `forge_ended`; `:126` — `restart`.
- `P/src/keys.rs:173`, `:212` — Demo key encoder/decoder, family 17.
- `P/src/records.rs:65`, `:123` — Demo encoder/decoder.
- `P/schema/store-v10.schema:207` — `DemoState`; `:208` — `Demo`; generated types at `P/src/generated/v10.rs:9761`, `:9787`.
- `F/src/domain.rs:4868` — `restore` accepts a Demo state only when its forge matches configured `DemoConfig`.

- **DemoAct family:** `F/src/boundary.rs:861` — `Stored::DemoAct`; `:910` — `Key::DemoAct([u8; 16])`; `:1634` — `DemoAct { key, act }`; `:1639` — `DemoActKind::{Close { project, task }, RemoveAccount { account }}`.
- `F/src/domain.rs:125` — `demo_acts: Map<[u8;16], DemoActKind>`; capacity assigned at `:534`.
- `F/src/demo.rs:159` — `acts`, saves each selected row before sending; `:167` — `send_acts`; `:173` — `act_answered`, erases after a terminal answer when `keep == false`.
- `P/src/keys.rs:10` — `DEMO_ACT = 19`; encode/decode at `:170`, `:211`.
- `P/src/records.rs:48`, `:109` — encoder/decoder.
- `P/schema/store-v10.schema:212` — `DemoClose`; `:213` — `DemoRemoveAccount`; `:214` — `DemoActKind`; `:215` — `DemoAct`; `:244` — `Value::DemoAct`.
- Generated counterparts: `P/src/generated/v10.rs:9869`, `:9905`, `:9929`, `:9955`; `Value` at `:10997`.
- `F/src/domain.rs:4865` — `restore` reloads `demo_acts`; `E/src/restart.rs:408` — restart’s `Open` step queues `Event::SendDemoActs`.

- **Writer family:** `F/src/boundary.rs:852` — `Stored::Writer(skein_forge_client::Writer)`; `:906` — `Key::Writer(u16)`.
- `skein-forge-client`, `skein/crates/skein-forge-client/src/identity.rs:37` — durable writer payload `Writer { forge: u16, author: u64 }`.
- Do not confuse it with `F/src/boundary.rs:523` — resource-slot `Writer::{Run, Lost, Entry}`.
- `F/src/domain.rs:165` — `writers: Map<u16,u64>`; `:1487` — public `writer`; `:1523` — `drain` promotes child `Stored::Writer` into the connector family.
- `skein-forge-client`, `src/domain.rs:153` — `writer`, emits the child save; `src/keep/restore.rs:3` — `restore_writer`.
- `P/src/keys.rs:176`, `:209` — Writer key encoder/decoder, family 16.
- `P/src/records.rs:78`, `:133` — Writer encoder/decoder.
- `P/schema/store-v10.schema:204` — `ForgeWriter`; generated type `P/src/generated/v10.rs:9732`.
- `F/src/domain.rs:4875` — reloads both child writer state and connector writer table.

- **Shared restart/store readers:** `E/src/restart.rs:360` — `restart_request`, loads `Range::Forge`; `:462` — `restore_page_row`, forwards forge rows at `:515`.
- `F/src/domain.rs:4785` — `restore`; `:4915` — `restored`, checks fit and restores child state before reporting `RestartStage::Restored`.
- `F/src/restore.rs:138` — `row`, exhaustive family admission; DemoAct at `:240`, Writer at `:247`, Demo at `:280`.
- `F/src/reconcile.rs:136` — `Reconciliation::restore`, exhaustive reference census; Writer/Demo/DemoAct at `:239`, `:240`, `:244`.
- `F/src/reconcile.rs:67` — `capacity`; `:291` — `configured_forge`; `:294` — `reference_resolves`.
- `F/src/domain.rs:5294` — reconciliation row classification; `:5382` — Writer missing-link reporting; `:5386` — Demo missing-link reporting; `:5401` — DemoAct has no links.
- `E/src/store.rs:533` — `Record::Forge`; `:543` — record key; `:640` — retained-byte accounting.
- `S/src/store/route.rs:108` — `translate` encodes forge keys/values; `S/src/store/ranges.rs:56`, `:81` — forge key/range translation.
- `P/src/records.rs:159` — `row`, verifies decoded payload agrees with its key.
- `P/src/lib.rs:15` — `STORE_VERSION = 10`; `:19` — `application`; `:56` — `bounds`; `:113` — `interval`.
- `temper-fake`, `temper/crates/temper-fake/src/check.rs:81` — `evidence`, reads Writer at `:94`, explicitly ignores Demo/DemoAct at `:142`.

- **Adding a family, using DemoAct’s current integration as the example:** add payload/`Stored`/`Key` in `F/src/boundary.rs:861`; exports/key/byte accounting in `F/src/lib.rs:56`, `:222`, `:258`; bound in `F/src/limits.rs:107`; runtime table/restore/reconciliation matches in `F/src/domain.rs:125`, `:534`, `:4865`, `:5298`, `:5401`; fit in `F/src/restore.rs:240`; record-kind/reference classification in `F/src/reconcile.rs:28`, `:244`; behavior in `F/src/demo.rs:159`.
- Codec files: `P/schema/store-v10.schema:212`; generated `P/src/generated/v10.rs:9955`; `P/src/keys.rs:10`, `:170`, `:211`; `P/src/records.rs:48`, `:109`; version/module/bounds in `P/src/lib.rs:9`, `:15`, `:56`; generated `P/golden/record_demo_act_smallest.bin` and DemoAct-kind/Value goldens.
- Keys have explicitly assigned tags: existing Flight is 21, Landing 23, Receipt 24 (`P/src/keys.rs:12`, `:85`, `:88`). Do not infer a new tag from enum position.
- Generation/check owner: `temper-codecs-world`, `temper/tests/codecs/tests/suite/drift.rs:5` — `schema_matches`; `:42` — `store_schema_matches_generated_code_and_goldens`.
- Behavior-specific root files: `E/src/domain.rs:65` — `demo_act_replies`; `E/src/route/demo.rs:250` — `list_end_acts`; `:285` — `send_end_act`; `:313` — `end_acted`; `:415` — `end_note`; `E/src/restart.rs:408` — replay; `E/src/route/forge_route.rs:2990` — dispatch.
- Additional exhaustive delivery/admission readers: `E/src/boundary.rs:814`, `E/src/decision.rs:67`, `:471`, `E/src/admission.rs:84`, `E/src/route/held.rs:189`, `E/src/domain.rs:1030`; `S/src/route.rs:294`; `S/src/clients/translation.rs:53` — application-only events rejected from core translation.
- Design contract: `temper/docs/design/protocol/store.md:89`, `:114`, `:169`, `:189` — family, addition/version rule, DemoAct durability.

- **Family-specific test readers:** `F/src/tests.rs:1203` — Writer saves; `F/src/tests/demo.rs:84` — Demo/SetUp behavior; `F/src/tests/reconcile.rs:212` — DemoAct reference checks; `F/src/tests/restore_fit.rs:84` — DemoAct restore admission.
- `P/src/tests/records.rs:769` — Writer round trip; `:792` — Demo round trip/key agreement; `:869` — DemoAct round trip; `:1193` — `writer_rules`.
- `P/src/tests/keys.rs:115`, `:132`, `:166` — Writer, Demo, DemoAct keys; `P/src/tests/fixtures.rs:173`, `:174`, `:176` — generated keys; `P/src/tests/sizes.rs:62` — Demo sizes.
- `S/src/tests/store.rs:123`, `:203` — Demo and Writer through real store; `S/src/tests/setup.rs:1721` — restored ended Demo.
- `temper-engine-domain-world`, `temper/tests/engine/domain/tests/suite/demo.rs:479`, `:494`, `:540`, `:767` — saved/kept/erased/restarted DemoAct readers; `tests/suite/reconcile.rs:79` — orphan Writer.
- `temper-engine-forge-world`, `temper/tests/engine/forge/src/world.rs:754`, `:918`, `:920`, `:924` — save/key mapping; `tests/suite/writer.rs:15` — Writer persistence.
- `temper-engine-protocol-forge-world`, `temper/tests/engine/protocol-forge/src/protocol.rs:672`, `:674`, `:678` — Demo/DemoAct/Writer key mapping.
- `temper-codecs-world`, `temper/tests/codecs/tests/suite/fuzzy/store.rs:40`, `:69`, `:75` — DemoAct/Demo/Writer fuzz inputs.
- Writer evidence fixtures: `temper-fake`, `temper/crates/temper-fake/src/tests/check.rs:197`; `temper`, `temper/crates/temper/src/fake/check/tests.rs:280`.
- DemoAct delivery/exhaustive-request test-kit readers: `temper-client-world`, `temper/tests/client/src/translate.rs:321`, `src/translate/forge.rs:448`; `temper-system-world`, `temper/tests/system/src/composed.rs:884`; `temper-engine-domain-world`, `temper/tests/engine/domain/src/walking.rs:900`, `src/escalation.rs:796`; `temper-engine-forge-world`, `tests/suite/simulation.rs:818`, `src/world.rs:778`; `temper-engine-protocol-forge-world`, `tests/suite/protocol.rs:507`, `src/protocol.rs:504`.
- Limits construction/read boundaries outside F: `E/src/limits.rs:68`, `E/src/translate.rs:107`, `E/src/tests/resume_fixture.rs:450`; `S/src/limits.rs:145`, `:393`, `S/src/forge.rs:165`, `S/src/store.rs:84`, `S/src/startup.rs:250`, `S/src/startup/application/landing.rs:320`.
- Limits fixtures: `F/src/tests.rs:57`, `F/src/tests/limits.rs:7`; `P/src/tests/fixtures.rs:31`; `temper-codecs-world`, `temper/tests/codecs/tests/support/store.rs:3`; `temper-engine-forge-world`, `temper/tests/engine/forge/src/world.rs:33`, `tests/suite/memory.rs:18`; `temper-engine-protocol-forge-world`, `temper/tests/engine/protocol-forge/src/protocol.rs:68`, `:82`, `:130`, `tests/suite/protocol.rs:393`.
- No skein, smith, or jig crate/test kit reads temper’s `Configured`, `Stored`, Demo, or DemoAct types. Jig supplies generic store, secrets, application-admission, and act machinery.

**2. Credentials and secrets keyed by forge number**

- Contract: `temper/docs/design/protocol/credentials.md:101` — section 2.1, Application-owned secret rows; number is `family << 48 | forge << 16 | item`; forge remains `u16`.
- `R/src/lib.rs:18` — `FAMILY_FORGE = 1`; `:20` — `CARRIED = 1 << 63`; `:53` — `number`; `:57` — `parse`.
- `R/src/lib.rs:23` — `Item`: `AdminToken=1`, `ApiToken=2`, `GitToken=3`, `User=4`, `WebhookSecret=5`, `AppId=6`, `AppSlug=7`, `AppKey=8`, `Accepted=9`, `Origin=10`; `Item::code` at `:37`.
- `R/src/codec.rs:14`, `:24` — `encode`/`decode`; `:136`, `:157` — pasted-bundle codecs. Pasted App PEM item 11 is not a forge-record `Item`.
- `jig-protocol-store`, `jig/crates/jig-protocol-store/src/boundary.rs:108` — `SecretOwner`; `:129` — `SecretValue`; `:157` — `Secrets::{Keep, Erase, Get, Load}`; `:218` — `SecretRow`.
- `jig-protocol-store`, `src/secrets.rs:239` — `secrets`; Keep dispatch `:252`, Erase dispatch `:280`; `:478` — `keep`.

- **Writer:** `S/src/setup/lifecycle.rs:267` — `keep(engine, forge, item, bytes, purpose)`, encodes, derives `records::number`, submits `Secrets::Keep { owner: Application, ... }` at `:283`.
- `S/src/setup.rs:450` — `advance`: setup-host `Need::Write` reaches `lifecycle::keep` at `:491`; `:534` — `item_record` maps host items.
- `S/src/setup.rs:512` — `answer`: only `Kept` updates the in-memory record table and advances the waiting setup operation.
- `S/src/setup/lifecycle.rs:308` — `done`, writes Origin for local Forgejo at `:331`, pasted App at `:361`, manifest App at `:367`.
- `S/src/setup/lifecycle.rs:707` — `advance`, rewrites Accepted after installation deletion at `:777`.
- `S/src/setup/lifecycle.rs:1402` — `keep_admin`, writes AdminToken at `:1414`; `:1442` — `writes_failed`, includes admin Origin completion at `:1467`.
- Admin input/reconcile: `S/src/setup/public.rs:46` — `admin_connect`; App input: `:117` — `connect`; both read carried values through `Secrets::Get`.
- Owned handover lifecycle: `S/src/setup/lifecycle.rs:1208` — `take_read`; `:1232` — `read`; `:1266` — `take_removal`; `:1285` — `removed`. This removes the handover file, not the forge’s stored credential family.

- **Erasure currently present:** `S/src/setup/public.rs:370` — `erase`; it asserts `number >= CARRIED`, then submits `Secrets::Erase { owner: Application, number }` at `:380`.
- **No erase-all-forge-records function, AdminToken Forget implementation, or forge Forget act exists.** The “Forget before making a new App” text in `S/src/setup.rs:335` is diagnostic text, not an implemented act.

- **Load/readers:** `R/src/forge.rs:53` — `Rows`; `:57` — `ForgeRows`; `Rows::push` at `:69`; `forge` at `:103`; `ForgeRows::get` at `:109`; `has` at `:117`.
- `R/src/forge.rs:10` — `Origin`; `:29` — `origin`; `:165` — `access`, checks origin/kind and constructs connection access.
- `S/src/forge/records.rs:67` — `Loader::new`; `:105` — `loaded`; `:130` — `advance`, paged Application-secret load at `:139`, configured-forge access reconstruction at `:168`, unconfigured-number reporting afterwards.
- `S/src/setup.rs:605` — `read_item`; `S/src/setup/lifecycle.rs:552` — `connect`; `:566` — `accepted`; `:899` — `reconcile_local`; `:1104` — `connect_local`.
- `S/src/forge/tokens.rs:124` — `announce`, reads AppSlug at `:151`; `S/src/startup/checks.rs:1021` — `record_writes`, includes encoded Origin size at `:1049`.
- Test-kit writer/reader: `temper-github-app`, `temper/testing/temper-github-app/src/lib.rs:21` — `records`, numbers at `:43`; `src/store.rs:128`, `:136`, `:189` — seed/load Application secrets.
- Credential test readers: `S/src/tests/setup.rs:125`, `tests/setup_admin.rs:168`, `tests/setup_adoption.rs:118`, `tests/setup_github.rs:452`, `tests/setup_lifecycle.rs:14`, `tests/forge.rs:641`, `tests/limits.rs:408`; `temper-codecs-world`, `temper/tests/codecs/tests/suite/records.rs:13`, `suite/fuzzy/records.rs:23`.

**3. SetUp, Connect, admin/App/manifest acts, senders, and retained answers**

- **Wire definitions:** `W/schema/client-wire-v1.schema:16` — `SetUp { forge:u16, org:bytes, repository:bytes, project:SetUpProject, playground:bool }`. The actual request already carries more than `{ forge }`.
- `W/schema/client-wire-v1.schema:15` — `SetUpProject::{Existing { project:u32 }, New { name }}`.
- Generated `W/src/generated/v1.rs:1098` — `SetUp`; `:1083` — `SetUpParts`.
- `W/schema/client-wire-v1.schema:24` — `Connect { forge:u16, how:How }`; generated `Connect` at `W/src/generated/v1.rs:1394`.
- `W/schema/client-wire-v1.schema:19` — `GitHubApp { id:u64 }`; `:20` — `GitHubManifest { back:Option<bytes> }`.
- `W/schema/client-wire-v1.schema:21` — `FirstAsk { org, repository, project, label }`; `:22` — `ForgejoAdmin { first:Option<FirstAsk> }`; `:23` — `How::{GithubApp, GithubManifest, ForgejoAdmin}`.
- Generated `How` at `W/src/generated/v1.rs:1357`; App/manifest/admin types at `:1199`, `:1230`, `:1326`.
- Other wire setup-related requests: `W/schema/client-wire-v1.schema:25` — `Install { forge, back }`; `:26` — `Reachable { forge, installation:Option<u64>, page:u32 }`; `:27` — `Accept { forge, installation:u64 }`; `:28` — `Request`, including `StartDemo`.
- `W/src/lib.rs:45` — `CLIENT_VERSION = 11`; `:256` — `decode_requesting` validates Connect inputs, including nonzero App/installation IDs and admin first-repository names.
- **No plain client Connect variant for an arbitrary connected Forgejo exists.** The three `How` choices above are exhaustive.

- **Client Acts:** `C/src/forge/form.rs:37` — `Act`; `SetUp { forge }` at `:44`, `SetUpAtOnce { forge }` at `:45`, `StartIn { forge }` at `:47`, `StartDemo` at `:49`, `ConnectAdmin { forge }` at `:51`, `ConnectApp { forge }` at `:53`, `ConnectManifest { forge, back }` at `:54`, `Install { forge, back }` at `:55`.
- `C/src/forge.rs:256` — `step` dispatches admin/App/manifest/setup acts.
- `C/src/forge/forges.rs:351` — `set_up`; `:381` — `set_up_at_once`; `:414` — `send_setup`; `:179` — `ask`; `C/src/forge/family.rs:198` builds wire `SetUp`.
- Welcome defaults: `C/src/forge/forges.rs:377` — `AT_ONCE_ORG = work`; `:378` — repository `hello`; `:379` — project `Default`.
- `C/src/forge/admin.rs:121` — `send`, builds optional first-repository intent and carried admin-token request; wire Connect at `:171`, carry at `:200`.
- `C/src/forge/github.rs:96` — `send`, sends Install or App Connect; `:147` — `manifest`; `:159` — `dispatch`, handles keyed request/carried App PEM.
- `C/src/forge/forges.rs:618` — `maybe_adopt`; `:671` — `setup_start`; `:743` — `send_adoption`. Project creation/adoption after the first repository are client follow-up work.

- **UI senders:** `V/src/forge/bindings.rs:87` — `Meaning` → setup/admin/App/manifest/install Acts; `:100` — StartIn.
- `V/src/forge/connect.rs:313` — welcome’s SetUpAtOnce button; `:398` — manifest/App choice; `:875` — StartIn; `:1550` — admin-connect control.
- CLI/tool senders: `T/src/forges.rs:39` — `SET_UP`; `:61` — `CONNECT`; `:119` — `INSTALL`; `:129` — `ACCEPT`; `:170` — `DEMO_START`; `:329` — `send`; setup request built at `:361`.
- `T/src/forges/connection.rs:33` — `request`, constructs admin/App/manifest Connect, Install, or Accept; `:259` — `told`; `:329` — answer decoding.
- `T/src/forges.rs:490` — `restore`, reads retained wire SetUp at `:502`.

- **Domain definitions:** `F/src/boundary.rs:152` — `Ask`; SetUp at `:154`, Connect at `:156`, Install/Accept/Reachable following; `:162` — `FirstAsk`; `:172` — `How`.
- `S/src/family.rs:27` — `to_root`: SetUp translation `:46`; Connect/Install/Accept `:84`; Reachable `:179`.
- `E/src/boundary.rs:387` — `Application::Forge`; `:393` — `ApplicationAnswer`.
- `E/src/route/applications.rs:19` — `start`, forces setup/connect/install/accept/reachable to Deployment scope; `:53` — `admitted`, passes role/carried/retention to connector.
- `F/src/domain.rs:1170` — `step` dispatches Connect-related asks; `:1178` — SetUp handler.
- SetUp validates names/project/playground values, checks `offer`, checks setup busy, keeps volatile `First`, marks Forges, emits `Request::SetUp` at `F/src/domain.rs:1240`. It uses an existing configured forge.
- `F/src/domain.rs:306` — `offer`: owned + asked + suitable connection state + live box.
- `F/src/connect.rs:35` — `start`: GitHub admission checks owner role, configured membership, kind/mode/read-only, busy, carried key, and connection state.
- `F/src/connect.rs:152` — `admin`: requires Owner, managed Forgejo with `admin_connect`, available state, carried token, and organization differing from bot login; keeps optional first intent.
- `F/src/connect.rs:113` — `send`: emits Connect for App/admin; Manifest for manifest conversion; Install or Accept for their paths.
- `F/src/connect.rs:234`, `:256`, `:276`, `:296` — `connected`, `installed`, `accepted`, `ended`; completion restores previous state or records diagnostic/App provenance as appropriate.
- `F/src/connect.rs:349` — `reachable`, admits a repository-picker read.

- **Root/protocol handling:** `E/src/route/forge_route.rs:2969`, `:2981`, `:2984`, `:2999` — forwards Connect/Accept/Install/SetUp.
- `E/src/boundary.rs:554`, `:561`, `:567`, `:578`, `:583` — root output variants SetUp/Connect/Manifest/Accept/Install.
- `S/src/route.rs:32` — SetUp saves service first intent, asks box; `:38` — admin/App Connect; `:47` — Manifest; `:53`, `:56` — Accept/Install.
- `S/src/setup/public.rs:46` — `admin_connect`, reads pasted token and runs Reconcile; `:117` — `connect`, pasted GitHub App; `:157` — `manifest`, starts Personal manifest plan; `:200` — `install`; `:219` — `accept`; `:406` — `reachable`.
- `S/src/setup/routes.rs:251` — `route`; `:334` — callback `accept`; `:402` — `manifest_visit`. Browser visits continue setup-host plans; they are not additional wire Act variants.
- `S/src/setup/lifecycle.rs:1330` — `first`; `:1000` — `create_local`; `:1088` — `adopt_local`; `:1104` — `connect_local`.
- `S/src/route/box_link.rs:167` — start-line encoding; `:302` — `drain`, translates box replies into root events.

- **Answer retention — implementation differs from the broader Application design wording:** `F/src/boundary.rs:184` — `Answered` says only adoption outcomes are durable.
- `F/src/boundary.rs:846` — `Stored::Application { token, until, outcome:ApplicationOutcome }`; `ApplicationOutcome`, `:411`, is only Adopted/CoreRefused/ForgeRefused.
- `F/src/adoption.rs:21` — `State.kept`; `:195` — `finish`, saves `Stored::Application` at `:199`; `:59` — `application_outcome`.
- `P/src/records/application.rs:8`, `:40` — Application encoder/decoder; `P/src/keys.rs:158` — Application key family 15.
- SetUp answers: `F/src/domain.rs:132` — `setup_answers`; `:327` — `setup_answer`, stores `(until, answer)` only in memory.
- Connect/Install/Accept/Reachable answers: `F/src/connect.rs:9` — `State`; `:26` — `answer`, bounded in-memory table, evicts first token when full.
- `F/src/domain.rs:343` — `application_answer`, searches connect/demo/setup memory before durable adoption outcomes; missing token returns None.
- `E/src/route/applications.rs:133` — `answer`; `E/src/route/held.rs:173` — retained application answer lookup; `S/src/family.rs:505` — wire answer conversion.
- Consequently **a restart does not reload SetUp/Connect answers from Application rows today**.

- **Act/wire test readers:** `W/src/tests.rs:294`, `:706`, `:860`; `W/src/tests/forgejo_admin.rs:25`.
- `C/src/tests/forges.rs:35`, `tests/connect.rs:200`, `tests/admin.rs:75`, `tests/github_app.rs:23`, `tests/form.rs:90`; `V/src/tests/forges.rs:93`, `tests/connect_forge.rs:199`, `tests/welcome.rs:91`.
- `T/src/tests/forges.rs:293`, `tests/forge_connection.rs:66`, `tests/adopt.rs:315`, `tests/fixtures/adopt.rs:84`.
- `temper-client-shell-cli`, `temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs:26`; `temper-client-shell-terminal`, `temper/crates/temper-client-shell-terminal/src/tests/family.rs:22`.
- `F/src/tests.rs:348`, `F/src/tests/connect.rs:33`, `F/src/tests/demo.rs:383`; `E/src/tests/setup.rs:86`, `tests/resume.rs:485`; `E/src/domain.rs:742`, `E/src/decision.rs:617` — exhaustive request/output handling.
- `S/src/tests/family.rs:846`, `tests/setup_admin.rs:91`, `tests/setup_github.rs:100`, `tests/system.rs:341`, `tests/engine.rs:483`, `tests/fixtures.rs:747`; setup lifecycle/create/adoption tests exercise service continuations.
- `temper-client-world`, `temper/tests/client/src/translate/forge.rs:121` — `request`; `:231` — `admin_request`; `:553` — `set_up_request`; `:519` — `answered`; `tests/suite/forges.rs:290`.
- `temper-engine-domain-world`, `temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:51`; request-routing readers in `src/direct.rs:208`, `src/protocol.rs:295`, `src/roles.rs:533`, `src/forge_routes.rs:1133`, `src/walking.rs:787`, `src/escalation.rs:674`.
- Other exhaustive kit readers: `temper-system-world`, `temper/tests/system/src/forge.rs:422`, `src/composed.rs:741`; `temper-engine-forge-world`, `temper/tests/engine/forge/src/world.rs:782`, `tests/suite/simulation.rs:822`; `temper-engine-protocol-forge-world`, `temper/tests/engine/protocol-forge/src/protocol.rs:508`, `tests/suite/protocol.rs:511`; `temper-codecs-world`, `temper/tests/codecs/tests/suite/client_wire.rs:49`.

**4. Forges rows and configured-forge readers**

- **Row source:** `F/src/domain.rs:238` — `Domain::forges`, iterates `configured` in number order, excludes ended demo forge, joins connection/failure, offer, first intent, installations, App provenance, adopted repositories, owned details/admin flag.
- `F/src/boundary.rs:379` — domain `Row`; `:399` — `Forges { rows }`.
- `E/src/domain.rs:159` — root `forges` accessor; `E/src/route/panels.rs:16` — builds Forges watch; `E/src/decision.rs:88` — `Delivery::Forges`; `:441` — delivery-byte accounting.
- `F/src/boundary.rs:1428` — `Request::MarkedForges`; `E/src/route/forge_route.rs:3062` — marks the single Forges subject.
- `S/src/route.rs:128` — delivery; `S/src/family.rs:711` — `forges`, converts domain rows to wire records.
- `W/schema/client-wire-v1.schema:67` — `ForgeRow`; `:68` — `Forges`; generated `ForgeRow` at `W/src/generated/v1.rs:2697`, `Forges` at `:2821`.
- `W/src/lib.rs:48` — `FORGES = 16`; `:282` — `forges` encoder; `:290` — `decode_forges`.
- `C/src/forge/family.rs:239` — `forges` decoder; `C/src/forge/forges.rs:132` — client `Row`; `:204` — `rows`; `:477` — `told`; `C/src/domain.rs:94` — public Forges state.
- `V/src/root.rs:247` — routes to Forges view; `V/src/forge/forges.rs:15` — view consumes client rows; `T/src/forges.rs:574` — `rows`, `:579` — `changed`, `:714` — `encode_rows`.

- **All configured-map readers in `temper-engine-domain-forge*`:**
- `F/src/domain.rs:208` — `configured`; `:238` — `forges`; `:306` — `offer`; `:492` — construction; `:1058` — `Event::BoxClosed` loops configured forges; `:1072` — `Event::AppRecorded` membership.
- `F/src/connect.rs:35` — `start`, membership/kind/mode/read-only at `:73`; `:152` — `admin`, config at `:163`; `:349` — `reachable`, config at `:363`.
- `F/src/demo.rs:137` — `connected`, other-configured-forge check at `:140`.
- `F/src/brief.rs:140` — `pull_identity`, reads configured addresses at `:156`.
- `F/src/landing.rs:113` — `project_landing`, rejects Turn policy on GitHub at `:120`.
- `F/src/reconcile.rs:291` — `configured_forge`; `:294` — `reference_resolves`.
- No configured-forge inventory reader exists in sibling crates `temper-engine-domain-forge-change`, `-issues`, or `-landing`.

- **Root-domain readers:** `E/src/translate.rs:34` — `Config.forges`; `E/src/domain.rs:275` — constructor; `:1140` — `refused`, constructs a validation connector using config at `:1225`; `:147` — connection-limit forwarding.
- `E/src/route/forge_route.rs:135` — `landing_words`; `:163` — `landing_pull`; `:2647` — obtains configured forge for landing text.
- Root row/watch/accounting readers are the `domain.rs`, `route/panels.rs`, `decision.rs`, and `route/forge_route.rs` sites above.

- **Service configuration/startup readers, grouped by file:**
- `S/src/settings.rs:20` — `Demo::domain`; `:145` — `Engine.forges`; `:216` — sign-in forge mapping.
- `S/src/startup/application.rs:355` — `forge` parser; `:572` — `references`; `:709` — `github_listener`; `:770` — `parse`; `:915` — `listed_files`; `:1045` — `git_programs`; `:1080` — `missing_asked_files`; `:1134` — `resolve`; `:1250` — `resolve_forge`; `:1291` — `configured`.
- `S/src/startup/application/demo.rs:26` — `parse`, checks configured demo forge.
- `S/src/startup/application/landing.rs:175` — `parse`; `:409` — `project_landing`, resolves forge kind at `:432`.
- `S/src/startup/application/sign_in/forges.rs:16` — `parse`; `:69` — `check`, validates configured forge mapping at `:90`.
- `S/src/startup/application/sign_in.rs:282` — `parse`; `:383` — `resolve`, preserves forge mappings at `:407`.
- `S/src/startup/checks.rs:230` — `webhooks`; `:325` — `longest_remote`; `:387` — `channel`; `:655` — `root`; `:939` — `socket_count`; `:1021` — `record_writes`.
- `S/src/startup.rs:230` — startup forge inventory; `:880` — asked-owned initial states; `:908` — resolved Engine inventory; `:1063` — `prepare_forges`.
- `S/src/system.rs:154` — `Service::new`; `:165` — `prepared`, wires forge/setup/demo/root configuration at `:217`, `:224`, `:235`; `:960` — `forget_handover`; `:1097` — `note_bound`.
- `S/src/engine.rs:116` — `Engine::new`, constructs credential loader; `:310` — `iterate`; `:426` — `clients_pass`.
- `S/src/workers.rs:34` — retained forge inventory; `:554` — `configured`, moves settings inventory at `:575`.
- `S/src/forge.rs:42` — `settings`; `:114` — `file_access`; `:218` — `Layer::new`; `:409` — `indexed`; `:436` — `prepare`; `:454` — `config`; `:478` — `connect`.
- `S/src/forge/records.rs:17` — grouped-row configured-number set; `:67` — `Loader::new`; `:130` — `advance`.
- `S/src/forge/tokens.rs:36` — `Tokens::new`; `:67` — `token`; `:86` — `removed`; `:92` — `accepted`; `:107` — `connected`; `:115` — `account`; `:124` — `announce`.
- `S/src/setup/settings.rs:8` — `fixed`; `:163` — `prepare`; `:253` — `local_starts`; `:329` — `source_forge`; `:419` — configuration diagnostics.
- `S/src/setup.rs:82` — `webhooks_listening` walks settings inventory; `:97` — `Layer::new`; `:605` — `read_item`.
- `S/src/setup/lifecycle.rs:544` — `configured`; `:707` — `advance`; `:824` — `restored`; `:1208` — `take_read`; `:1266` — `take_removal`; public setup handling uses this configured lookup.
- `S/src/family.rs:27` — `to_root`, resolves adoption host; `:305` — `panel`; `:443` — `address`; `:711` — Forges conversion.
- `S/src/route.rs:9` — `request`, supplies inventory to panel and worker translation at `:143`, `:159`.
- `S/src/assignment.rs:74` — `down`; `S/src/translate.rs:25` — `items`, repository/identity translation.
- `S/src/clients.rs:453` — `advance`, supplies inventory to `family::to_root` at `:491`.
- `S/src/clients/bundle.rs:339` — `forge_metas`; `:388` — `fill_forge_sign_in`.
- `S/src/clients/google.rs:239` — sign-in settings forge-map copy; this is sign-in metadata, not the runtime Forges document.
- `S/src/clients/github.rs:16` — `claims`, recognizes `/forges/` callback routes; `S/src/setup/routes.rs:34`, `:251`, `:402` — route parsing/handling.
- `S/src/limits/names.rs:70`, `:190`, `:210`; `S/src/limits.rs:338`; `S/src/worker_settings.rs:84`; `S/src/translate.rs:10` — count/bound configuration consumers, distinct from configured-record membership.

- **Configured literal/constructor test readers:** `F/src/tests.rs:1225`, `:1251`, `:1431`; `F/src/tests/connect.rs:16`; `F/src/tests/demo.rs:146`.
- `E/src/tests/setup.rs:19`, `:625`; `E/src/route/forge_route/tests.rs:728`, `:838`; `E/src/tests/resume_fixture.rs:351`.
- `S/src/tests/startup.rs:1872`; `tests/setup_lifecycle.rs:700`; `tests/setup_adoption.rs:260`.
- `temper-engine-domain-world`, `temper/tests/engine/domain/src/walking.rs:360`; `tests/suite/demo.rs:31`; `tests/qa_all/qa_first_repository.rs:16`, `:212`.
- `temper-client-world`, `temper/tests/client/src/front.rs:161`; `src/front/account_acts.rs:190`; `tests/suite/forges.rs:32`; `tests/qa_all/qa_forges.rs:60`, `:87`, `:137`, `:398`, `:482`, `:1117`; `tests/qa_all/qa_first_repository.rs:24`; `tests/qa_all/qa_welcome_m3_owned.rs:26`.

- **Additional client row readers:** `C/src/forge/demo.rs:186`, `forge/listed.rs:4`, `forge/connect.rs:4`, `forge/admin.rs:121`, `forge/github.rs:96`, `forge/picker.rs:24`, `forge/words.rs:886`, `pages/welcome.rs:22`, `route.rs:286`.
- `V/src/forge/connect.rs:17`, `forge/bindings.rs:87`, `pages/welcome.rs:302`, `limits.rs:132`, `words.rs:1909` — row presentation/actions and capacities.
- **Direct wire-row test/fixture readers:** `W/src/tests.rs:333`; `C/src/tests/forges.rs:212`, `tests/connect.rs:496`, `tests/admin.rs:27`, `tests/demo.rs:624`, `tests/pages/welcome.rs:100`.
- `V/src/tests/fixtures.rs:288`, `tests/forges.rs:332`, `tests/connect_forge.rs:16`, `tests/welcome.rs:17`; `T/src/tests/forges.rs:40`, `tests/forge_connection.rs:435`.
- `temper-client-shell-cli`, `temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs:84`; `temper-client-shell-terminal`, `temper/crates/temper-client-shell-terminal/src/tests/fixtures/chat.rs:324`; `temper-client-shell-web`, `temper/crates/temper-client-shell-web/src/host_tests.rs:428`.
- `S/src/tests/family.rs:1042`, `tests/startup.rs:2125`; `temper-client-world`, `temper/tests/client/src/front.rs:1121`, `src/translate/forge.rs:595`, `:625`.
- `temper-codecs-world`, `temper/tests/codecs/tests/suite/client_wire.rs:44`, `suite/fuzzy/client_wire.rs:87`.
- Gallery fixture readers: `temper/tools/gallery/src/worlds.rs:292`, `temper/tools/gallery/src/worlds/github.rs:65`.