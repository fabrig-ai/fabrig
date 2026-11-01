Read-only survey; no builds or Cargo commands run. References are repository-relative. Where a bullet gives a crate directory, subsequent `src/...` and `tests/...` references are relative to that directory. “Readers” includes production consumers, test fixtures, and exhaustive dispatchers.

**1. `temper box`: startup, groups, pipe, persistence, restart**

- **`temper` — `temper/crates/temper/src/lib.rs:50`**, `Command::Box`; `configure_with`:93 selects it at :154; `run` dispatches to `box_command::run` at :195.
- **`temper` — `temper/crates/temper/src/box_command.rs:41`**, `run` calls `configuration`:225, then `run_configuration`:47.
- `configuration`:225 runs `init::core` only when the configuration directory is absent or empty; otherwise reads its `temper.json`. A nonempty directory without the document refuses.
- `run_configuration`:47 checks the document through `setup::check`; obtains `Checked.setup`, demo configuration, limits and listeners; calls `temper_fake::steps` at :78; supplies owned-forge notices through `owned_notices`:189; opens listeners through `open_listeners`:125; constructs `temper_box::host::Service` at :111; drives it on one skein kernel.
- **`temper` — `temper/crates/temper/src/setup.rs:10`**, `Deployment`; `check_with`:17 joins configured forges with identities, builds setup `Members`, reads `application.setup`, checks demo configuration, and applies box limits. `check`:107 supplies stock configuration.

- **`temper-forge-setup` — `temper/crates/temper-forge-setup/src/step.rs`**:
  - `Stage`:10 = `Fetch`, `Prepare`, `Run`; `Stage::word`:17 gives pipe spellings.
  - `Group`:27 contains `forge: u16`, opaque `label`, `recorded`, insertion position `before`, and `(Stage, Step)` entries.
  - `Steps`:38 contains `always` and `asked`.
  - `Kind`:44 = `Once`, `Long`, `Link`.
  - `Step`:51 holds command name/arguments, lifetime, readiness endpoint, inherited listeners, and public started notice.
  - `forge`:118 builds `temper forge ACTION DIRS N [forgejo|runner]`.
  - `steps`:144 walks document setup entries in order.
- `steps`:144 builds each owned forge as optional **fetch**, **prepare**, **forgejo**, optional **runner**. Forgejo has a version readiness probe; Linux also passes its held listener.
- An owned entry with `asked=true` becomes a `Group { recorded: true }` at `step.rs:176`; without `asked`, its commands enter `always`.
- Managed entries with a legacy admin-token file get an always-running-chain `forge reconcile` once step at `step.rs:192`; client-managed entries do not.
- `steps` appends `temper internal serve DIRS --box` at `step.rs:202`; no generated step uses `Kind::Link`.

- **`temper-fake` — `temper/crates/temper-fake/src/steps.rs:21`**, `steps` wraps setup’s steps:
  - Non-demo fake forges: always **forge prepare N**, then **fake forge N**, at :49.
  - Demo group: **fake llm**, **forge prepare DEMO_FORGE**, **fake forge DEMO_FORGE**, at :73.
  - Demo group has label `Playground`, `recorded=false`, and is always asked.
  - Caller-supplied `first` steps precede everything; owned-group insertion positions are shifted accordingly at :102.
  - `command`:109 builds fake command arguments.

- **`temper-box` — `temper/crates/temper-box/src/layout.rs:18`**, `flatten` merges always steps and asked groups into `Placed`:12; `Asked`:6 preserves forge number, record policy and stage. `label`:61 finds the group’s opaque display label.
- **`temper-box` — `temper/crates/temper-box/src/host.rs:175`**, `Service::new` flattens steps and constructs `Machine`, `Records`, `Logs`, child state and held listeners.
- **`temper-box` — `temper/crates/temper-box/src/machine.rs:112`**, `Machine`; `GroupState`:67; `Status`:58; `State`:93.
- `Machine::start`:191 incorporates recorded groups into the startup chain; remaining groups stay dormant.
- `ask`:598 starts a dormant group, acknowledges an already-started group, and serializes group startup.
- `advance_group`:669 writes a marker only after the whole recorded group stands; `group_started`:684 emits the terminal fact and `started N`; `written`:690 handles marker completion.
- `stop_group`:622 and `stop_group_next`:652 stop in reverse order, disarm restarts, wait for closure, then return the group to dormant.
- `restart`:768 applies bounded exponential backoff; `give_up`:792 stops the box after the restart allowance; `stop`:813 starts ordered whole-box shutdown.
- **Important:** `stop N` preserves the started marker and all forge files. It is not removal.

- **Pipe transport:** `temper-box/src/host.rs:1119`, `spawn_pipes`, passes step listeners at child fds `3..`; serve’s request pipe follows them, then its answer pipe.
- **`temper` — `temper/crates/temper/src/lib.rs:575`**, `serve` calculates those fds from inherited-listener count, checks that both are pipes, and calls `Service::adopt_box` at :604.
- **`temper-service` — `temper/crates/temper-service/src/system.rs:495`**, `Service::adopt_box`; `box_event`:997 receives replies; `box_io`:1039 writes commands and requests bounded LF-delimited reads.

- **Box parser — `temper-box/src/pipe.rs:46`**, `Command::{Start, Stop, Unknown}`; `command`:52 accepts only `start N\n` and `stop N\n`, canonical decimal `u16` without leading zeroes except `0`.
- `Reader`:15 / `Reader::byte`:24 bound complete lines including LF, discarding overflow through LF.
- Reply constructors: `step`:82 → `step N fetch|prepare|run\n`; `started`:86; `stopped`:90; `failed`:94 truncates the reason to `box.line_bytes`.
- **`temper-box/src/host.rs:948`**, `take_control_line`, maps parsed commands to `Input::Ask` / `Input::Stop`; unknown/oversized input is logged and ignored.
- `host.rs:607`, `order`, formats `Said::{Step,Started,Stopped,Failed}` through those constructors.
- Refusal policy is `machine.rs:571`, `refused`: `busy`, `not asked`, `stopping`; `terminal.rs:144`, `why`, formats reasons.

- **Serve parser — `temper-service/src/route/box_link.rs`**:
  - `Link`:43; `Ask::{Start,Stop}`:38; `Out`:20; `Reply`:171.
  - `ask`:70 emits `start`:167; holds one other group’s start while an ask is active.
  - `stop`:90 emits `stop`:163, or queues it; pending stop precedes held start.
  - `read`:108 splits/bounds lines; `line`:192 checks that a reply matches the active ask; `decoded`:239 parses the four reply verbs; `number`:276 parses bounded decimal.
  - Unlike the box command parser, reply `number` does not reject leading zeroes.
  - `stopped` clears the active stop without a domain completion event; failed stop is only logged.
  - `closed`:147 drops active/held work and emits `Out::Closed`.
  - `drain`:302 translates replies into connection/setup events and calls setup `started`.

- **Persisted box files — `temper-box/src/records.rs`**:
  - `Records`:28; `new`:38 collects only configured `recorded` groups.
  - `tick`:84 probes markers before policy startup; `complete`:114 records successful probes.
  - `write`:77 creates an empty private marker; `path`:191 computes **`<data>/box/started/N`**.
  - `box` and `box/started` are made mode `0700`; marker mode is `0600`.
  - It probes the known configured numbers; it does not discover groups by directory enumeration.
- **Logs exist today:** `temper-box/src/logs.rs:52`, `Logs`; `new`:72 derives names from step names, spaces becoming hyphens. Files are `<data>/box/logs/NAME.log`, rotation `.log.1`; previous launch’s directory is `box/logs.previous` (`tick` operations at :203–257).
- **Does not exist:** `<data>/box/groups/N`, serialized group settings, runtime group discovery, `remove N`, `Command::Remove`, directory deletion on group stop, or started-marker deletion.

- **Restart decision:** `host.rs:1166`, `Host::iterate`, waits for log/record preparation, then calls `Machine::start` at :1204. The document rebuilds every step; markers activate recorded owned groups. The demo has no marker and waits for serve to request it.
- **Demo restart request:** `temper-engine-domain-forge/src/demo.rs:126`, `restart`, emits `SetUpGroup` only for restored `DemoState::Running` needing connection. An ended demo does not restart.

Readers of these group/step/persistence APIs:

- **`temper` — `src/box_command.rs:47,125,189`**, `run_configuration`, `open_listeners`, `owned_notices`; `src/fake_backends.rs:67,261`, `run`, `checked_peers`; `src/fake_backends/tests.rs:230,645`, step-composition assertions.
- **`temper-fake` — `src/steps.rs:21`**, `steps`; `src/backends.rs:38`, `steps`; tests `src/tests/steps.rs:23,177,194`, `documents_give_the_box_its_whole_ordered_steps`, `checked_steps`, `first_steps_precede_fakes_and_both_groups_without_entering_the_playground`; `src/backends/tests.rs:70`, `welcome_steps_name_only_the_four_held_peers_and_share_the_seed`.
- **`temper-forge-setup` — `src/tests/steps.rs:8`**, `names`, and tests at :17,52,66,78,83,88,113,162; `src/tests/fetch.rs:510`, `groups_fetch_before_preparation`; `src/tests/managed.rs:99`, `managed_string_starts_serve_without_legacy_reconcile`.
- **`temper-box` production:** `src/host.rs:175,607,917,948,1049,1119,1166`, `Service::new`, `order`, `record_work`, `take_control_line`, `reply_entries`, `spawn_pipes`, `iterate`; `src/layout.rs:18,61`, `flatten`, `label`; `src/machine.rs:123`, `Machine::new`; `src/pipe.rs:82`, `step`; `src/records.rs:38`, `Records::new`; `src/logs.rs:72`, `Logs::new`; `src/terminal.rs:75,144,159`, `Formatter::fact`, `why`, `max_bytes`.
- **`temper-box` test kit — `src/tests/fixtures.rs`**, `step`:15, `owned_steps`:18, `Rig::new`:35, `Harness::{new,grouped,buffered,configured,configured_echo}`:181/187/190/198/207, `open_step_listeners`:743, `grouped_steps`:814.
- **`temper-box` direct tests:** `src/tests/pipe.rs:7,28,61`, bounded reader/canonical commands/reply spellings; `src/tests/host.rs:154,200,250,287,310,543,579,615,644,697,721,957,1190,1248,1290`, pipe, marker, dormant, stop, listener and loaded-command cases.
- **`temper-box` machine group tests — `src/tests/machine.rs`**: `asked_groups_follow_their_chains_records_failures_and_stop`:214; final-once gating:730; owned readiness/marker gating:788; stale unrecorded marker:892; reverse stop/restart:989; stop refusals:1011; disarmed backoff:1048; termination during stop:1085; group lifecycle facts:1147/1190; late preparation withdrawal:1619/1640/1659.
- **Other `temper-box` readers:** `src/tests/restart.rs:48,171,176,181,186`, `probing_serve` and abort-order tests; `src/tests/terminal.rs:36,61,74,130`, layout/labels/owned notices; `src/tests/logs.rs:11,56,246`, `harness`, launch rotation and group failure tails; `tests/real_loop.rs:434`, `a_split_sign_in_link_stays_whole_on_the_shared_terminal`.
- **Boundary tests constrain the change:** `temper-box/src/tests/boundary.rs:4`, `the_box_depends_on_no_temper_crate_but_the_setup`; `allowed`:19 permits only `Step`, `steps`, `Steps`, `Group`, `Stage`, `Kind`; `the_box_names_only_steps_and_step_of_the_setup`:56 scans source.

**2. Directories, fake files, binary cache**

- **No universal `forge-N` path constructor today.** `temper-forge-setup/src/setup.rs:78`, `Entry.directory`, comes from `application.setup[].directory`; `Setup::read`:93 resolves it relative to **data**. Absolute paths remain absolute.
- **Default owned directory:** `temper-init/template/temper.json:328`, forge 3’s `directory: "forge-3"`.
- **Default fake directory:** same template :305, forge 1’s `directory: "demo/forge"`; fake git executable :316, `"demo/bin/git"`; demo model key :364, `"demo/llm.key"`.
- **`temper-forge-setup/src/setup.rs:126`**, `Setup::read`, resolves fake `git` relative to data; owned Forgejo/runner binaries at :133 resolve relative to **cache**.
- `setup.rs:277`, `owned`, expands `"fetched"` into `forgejo/VERSION/forgejo` at :298 and `forgejo-runner/RUNNER_VERSION/forgejo-runner` at :303.
- **`temper-forge-setup/src/place.rs:6`**, `Place`; `path`:47 computes `forgejo/app.ini`, `state`, `admin_token`, person password/header files, legacy bot files, `runner/secret`, `runner/config.yaml`, locks and partial dump; `work_path`:74 adds `/forgejo`; `backup`:69 adds `/backups/NAME`.
- **`temper-forge-setup-shell/src/files.rs:52`**, `layout`, makes `home`, `forgejo`, `forgejo/repositories`, `runner`, `backups`; creates `home/gitconfig`. `clear`:39 removes only an interrupted preparation’s selected files, preserving backups/state.
- **`temper/src/forge.rs:153`**, `start`, takes `Entry.directory`, creates it privately, and makes `Place` at :239. Fake preparation uses `directory/admin_token` through `prepare_fake`:283.
- **`temper/src/init.rs:180`**, `write_with_program`, computes demo output paths and makes `demo`, `demo/bin` at :234. **`temper-init/src/lib.rs: files`** supplies initial files; `template.rs:18`, `document` substitutes the chosen endpoints.
- **Fetch:** `temper-forge-setup/src/fetch.rs:82`, `pins`, uses the already-resolved cache filenames; `Pin`:12; `Fetch`:173; `part`:190 adds `.part`; `step`:219 hashes an existing file first and skips download on a matching pin.
- **Fetch host:** `temper-forge-setup-shell/src/fetch.rs:59`, `hash`; `open`:75; `finish`:89; `run`:98. `temper/src/forge.rs:207` selects this path.
- **Cache test readers:** `temper/tests/support/forge_land.rs:577`, `cache_forgejo_binaries`; `cache_binary`:612; `assert_cached_binary_unchanged`:722; `temper/tests/support/first_repository.rs:741`, `stage_binaries`; `temper/tests/process.rs:1783`, demo-to-owned setup; `tests/qa_all/qa_first_run.rs:212`, bad cached-release case.
- **Path/API readers beyond commands:** `temper-service/src/startup.rs:77`, `handover_path`; `setup/settings.rs:253`, `local_starts`; `startup/application.rs:1045`, `git_programs`, and `configured`:1291; `clients/bundle.rs:339`, `forge_metas`; `temper/src/forge_sign_in.rs:13`, `run`; `temper/src/fake.rs:335`, `forge_document`.
- **Fake-backend sandbox:** `temper/src/fake_backends.rs:223`, `directories`, computes `<source-data>/box/fake-backends/{config,data}` and shares source cache. Peer documents live in effective config: `github.json`, `oauth.json`, `llm.json`, `forgejo.json` (`REQUIRED`:23).
- **Schema-sensitive path reader:** `temper/src/fake_backends/import.rs:16`, `RESOURCES`, explicitly names setup indices 0/1, fake git path, owned directory and demo key; `Source::read`:29 / `rebase`:116 depend on this init shape.
- No separate persistent fake-forge database directory exists: its forge/git state is memory. The filesystem paths above are handover, executable, configuration and key outputs.

**3. Serve-side path from acts to `start N`; reverse dispatchers**

- **`temper-engine-domain-forge` — `src/boundary.rs:154`**, `Ask::SetUp`; `Ask::StartDemo`:155; request variants `StopGroup`:1376, `SetUpGroup`:1380, `SetUp`:1388.
- **`temper-engine-domain` — `src/route/applications.rs:19`**, `start`, assigns deployment scope to SetUp/Connect/StartDemo; `admitted`:53 passes the admitted application to its owner.
- **SetUp:** `temper-engine-domain-forge/src/domain.rs:1169`, `step`, validates SetUp, records volatile `First` at :1226, emits `Request::SetUp` at :1240.
- **StartDemo:** `temper-engine-domain-forge/src/demo.rs:82`, `start`, saves `Stored::Demo(Running)` and emits `SetUpGroup` at :110; `restart`:126 reissues the group request after restoration.
- **Translation:** `temper-engine-domain/src/route/forge_route.rs:2993`, `outputs`, forwards `StopGroup`, `SetUpGroup`, `SetUp` to root requests.
- **Root variants:** `temper-engine-domain/src/boundary.rs:540/544/554`, `Request::{StopGroup,SetUpGroup,SetUp}`.
- **Service:** `temper-service/src/route.rs:9`, `request`; SetUp at :32 first calls `setup::lifecycle::first`, then `Link::ask`; SetUpGroup at :20 directly calls `Link::ask`; StopGroup at :15 calls `Link::stop`.
- **Wire:** `temper-service/src/route/box_link.rs:70`, `Link::ask` → `start`:167 → `Out::Write`; `drain`:302 stores pending output; `system.rs:1039`, `box_io`, sends it.
- **Connect does not ask the box to start a group.** `route.rs:38` routes GitHub App to `setup/public.rs:117`, `connect`, or managed Forgejo to `admin_connect` in that file. These drive setup plans against an already-running forge.
- A successful other-forge connection can instead **stop the playground**: `temper-engine-domain-forge/src/demo.rs:137`, `connected`, saves Ended and emits `StopGroup` at :154.

Additional production readers of root request variants:

- `temper-engine-domain/src/domain.rs:749`, `Domain::stage_now`; `src/decision.rs:591`, `resume` matches corresponding `Output` variants at :616.
- `temper-service/src/engine.rs:19`, `Engine.box_link`, initialized by `Engine::new`:149; `src/system.rs:251/506/997/1039`, `prepared`, `adopt_box`, `box_event`, `box_io`.
- Serve-link consumers/tests: `temper-service/src/tests/box_link.rs:42–343` cover ask/hold/refusal/parser/stop/closure; `tests/qa_box_link.rs:9,63,117`, `qa_unasked_and_invalid_lines_are_named_without_state_changes`, `qa_split_lines_and_coalesced_steps_keep_the_last_step`, `qa_line_byte_bound_includes_lf_and_recovers_after_overflow`.
- `temper-service/src/tests/setup.rs:1249`, `box_started`; `tests/setup_create.rs:263`, `ask_copy`; `tests/system.rs:172`, `the_box_pipes_carry_no_service_data_and_close_on_drain_or_abort`; its other box-system cases at :238 onward.
- Exhaustive test dispatchers: `temper-service/src/tests/fixtures.rs:745`, `settle_root`; `tests/engine.rs:480`, `startup_locate`; `tests/setup_github.rs:97`, `pump`.
- Root tests: `temper-engine-domain/src/tests/setup.rs:84`, `settle`; keyed replay test :137 and role-refusal test :239; `tests/resume.rs:483`, `stopping_continues_projection_deferred_finishing_without_new_commits`.
- Forge-domain tests: `temper-engine-domain-forge/src/tests.rs:346/418/487`, exhaustive request matches; setup assertions at :1343/1577/1694/1775/1833; `tests/demo.rs:277`, `group_asks`, end assertions :318/334/450.

World/test-kit readers of those variants:

- **`temper-client-world` — `temper/tests/client/src/front.rs:1379`**, `immediate`; `src/translate.rs:243`, `incoming`; `src/translate/forge.rs:404`, `incoming`; `tests/suite/forges.rs:290`, `settle`.
- **`temper-system-world` — `temper/tests/system/src/forge.rs:422`**, `collect`; `src/composed.rs:741`, `root_request`, and :787, `play`.
- **`temper-engine-domain-world` — `temper/tests/engine/domain/src/direct.rs:207`**, `collect`; `protocol.rs:294`, `play`; `walking.rs:786`, `observe`; `roles.rs:532`, `observe`; `forge_routes.rs:1132`, `collect`; `escalation.rs:673`, `observe`.
- Same crate, `tests/suite/demo.rs:199/233/300/359/379`: demo failure, restart, busy link and end cases.
- Same crate, `tests/qa_all/qa_first_repository.rs:105/129/146/234/265/279/289`: invalid/bounded names, busy intent, volatile restart, creation failure and replay assertions.
- **`temper-engine-forge-world` — `temper/tests/engine/forge/src/world.rs:781`**, `take_top`; `tests/suite/simulation.rs:821`, `a_change_produced_opened_checked_queued_and_landed`.
- **`temper-engine-protocol-forge-world` — `temper/tests/engine/protocol-forge/src/protocol.rs:507`**, `take_top`; `tests/suite/protocol.rs:510`, `land_change`.
- Searches across **skein, smith and jig found no readers of temper’s box/group/setup types or request variants**. Their libraries supply mechanisms below these interfaces.

**4. Setup crates, setup host, state versus store, step driving**

- **`temper-forge-setup` — `src/setup.rs`**: `Owned`:28, `Fetch`:38, `Runner`:20, `Fake`:63, `Mode`:71, `Entry`:78, `Setup`:88.
- `Setup::read`:93 reads document entries; `entry`:161 finds a configured number; parsers `owned`:277, `fake`:374, `mode`:417, `entry`:451, `list`:535.
- **`src/check.rs:40`**, `Checked`; `Forge`:21; `Members`:34; `members`:64 cross-checks setup against document forge/identity members and listener uniqueness; `checked`:181 reads and validates the list.
- **`src/given.rs:103`**, `values`, turns an entry into skein’s setup `Given`: bot identity, orgs, owner, token names, listener and limits.

- **Offline owned preparation:** `temper-forge-setup/src/prepare.rs:148`, `Prepare`; `Found`:30; `Event`:46; `Request`:62; `step`:575.
- `Prepare` keeps phase, resolved `Owned`/`Place`, found state, drawn values and version/upgrade decisions in memory. It emits one request per answer.
- `temper-forge-setup-shell/src/prepare.rs:45`, `run`, retains locks, calls `prepare::step`, executes versions/commands, creates layout, writes files, and feeds the next event.
- `prepare.rs:23`, `found`, reads the preparation record and discovers existing files; `locked`:17 takes Forgejo/runner locks.
- **Persistent preparation record:** `temper-forge-setup/src/state.rs:18`, `State`, stores Forgejo/runner versions, `prepared`, and Forgejo secrets; `encode`:56 / `decode`:75. This is the forge directory’s `state`, separate from engine store records.
- **Process specs:** `temper-forge-setup/src/spec.rs:38`, `Spec`; `command`:100; `version`:209; `listening`:218. `run.rs:33`, `launch`, derives Forgejo/runner launches.
- **Shell hosts:** `temper-forge-setup-shell/src/command.rs:24`, `run_one`; `process.rs:21/42/60`, `lock`, `command`, `exec`; `run.rs:18/44`, `checked`, `run`; Linux `parent.rs:20`, `run`, retains the listener and supervises Forgejo’s child.

- **The online setup host is skein’s**, not a second implementation inside either temper setup crate:
  - `skein-forge-setup/src/host.rs:24`, `Host`; `new`:38, `down`:54, `up`:166, `fire`:211.
  - `host/settings.rs:7/12/19`, `Settings`, `Forge`, `Kind`; `host/boundary.rs:17/111/128/133/138`, `Plan`, `Request`, `Answer`, `Need`, `Event`.
- **Temper host adapter:** `temper-service/src/setup.rs:18`, `Layer`, holds the host, staged plans, active runs, caller correlations, file operations and pending writes.
- `setup/settings.rs:8`, `fixed`, builds host settings from configured endpoints; `forgejo`:104 maps setup entries through `given::values`; `prepare`:163 builds `Layer`; `local_starts`:253 builds local startup state.
- `setup.rs:432`, `begin`, waits for record loading and webhook bind before constructing `Host`; `advance`:450 drives host events/timers; `run_plan`:585 sends `host::Request::Run`.
- `Need::Read` is answered through `read_item`:605; `Need::Write` calls `setup/lifecycle.rs:267`, `keep`, issuing `Secrets::Keep { owner: Application, number: records::number(forge,item) }`.
- `setup.rs:512`, `answer`, updates the record cache only after `Kept`, then advances the corresponding purpose; failures stop availability.

- **Local sequence:** `setup/lifecycle.rs:1185`, `started` → handover read → `Plan::TakeOver` → durable admin record → handover unlink → `reconcile_local`:899 → durable `Origin` → optional `create_local`:1000 → fake adoptions through `adopt_local`:1088 → `connect_local`:1104.
- Handover correlation: `FileStep`:88; `take_read`:1208, `read`:1232, `take_removal`:1266, `removed`:1285, `file_answer`:1303.
- **Shell handover readers:** `temper-service/src/lib.rs:129/136`, `HandoverRead`, `HandoverRemoval`; `system.rs:315/322/336/339`, public take/answer methods; `temper/src/host.rs:109/126`, `read_handover`, `remove_handover`.
- **Restart:** `setup/lifecycle.rs:824`, `restored`; `start_local`:858 uses staged handover if present, otherwise kept records. `Trigger::StartDemo` deliberately waits for the demo’s box ask.

- **First-repository intent is not durable:**
  - `temper-engine-domain-forge/src/boundary.rs:360`, `First`; `FirstState`:335; `domain.rs:129`, `Domain.first`.
  - Writers/readers: `domain.rs:222/230/238/1226`, `first_made`, `first_failed`, `forges`, SetUp admission; `connect.rs:214`, managed-connect intent.
  - Service copy: `temper-service/src/setup/lifecycle.rs:53`, `Intent`; stored in `Local.first` (:65) or `Run.first` (:34).
  - Writers/readers: `first`:1330; `create_local`:1000 consumes local intent; `create_stopped`:1042 uses the run copy; `admin_create`:1416 reads managed intent; `setup/public.rs:89`, `admin_connect`, creates that copy.
  - `setup/settings.rs:279` initializes `first=None` each start. Neither intent is in `Stored` (`temper-engine-domain-forge/src/boundary.rs:802`) or the credential item set.
  - Direct outward reader: `temper-service/src/family.rs:711`, `forges`, and `first_repository`:860 encode it for clients.
  - Existing test explicitly contracts volatility: `temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:234`, `first_repository_row_is_volatile_across_restart`.

Other readers of setup entries/check results that moving forges out of the document reaches:

- **`temper`**: `src/setup.rs:17`, `check_with`; `src/forge.rs:153/312`, `start`, `reconcile_settings`; `src/forge_sign_in.rs:13`, `run`; `src/fake.rs:335/371`, `forge_document`, `llm_configuration`; `src/fake/check.rs:79`, `configuration`; `src/lib.rs:378`, `prepare_with_limits`; box/fake-backend readers listed above.
- **`temper-service`**: `src/settings.rs:130`, `Setup { entry, handover }`; `startup.rs:21/41/64/77/81/778`, `setup_files`, `handover_reads_with`, `handover_reads`, `handover_path`, `setup_values`, `checked_start_with`; `startup/application.rs:1045/1291`, `git_programs`, `configured`; `startup/application/sign_in/forges.rs:53`, `public`; `clients/bundle.rs:339/385`, `forge_metas`, `fill_forge_sign_in`; `system.rs:960`, `forget_handover`.
- **`temper-fake`**: `src/demo.rs:74/82/155`, `Facts`, `read`, `unique_listen`; `src/steps.rs:21`, `steps`; `src/forge.rs:53`, `Document::fake`.
- **`temper-forge-setup` internal consumers**: `check.rs:64/181`, `members`, `checked`; `given.rs:103`, `values`; `step.rs:144`, `steps`; `prepare.rs:172`, `Prepare::new`; `fetch.rs:82`, `pins`; `run.rs:33/82`, `launch`, `admit_listener`; `ini.rs:59`, `text`; `runner.rs:21`, `configuration`; `spec.rs:100`, `command`; `sign_in.rs:47`, `instructions` parameters include `Place`.
- **`temper-forge-setup-shell`** consumes preparation/path/spec/fetch APIs in `src/{prepare,files,command,process,run,parent,fetch,reconcile}.rs`, functions listed above; legacy `reconcile.rs:17/201/486`, `Settings`, `Running::new`, `reconcile`, writes file outputs, not engine-store records.
- **`temper-one-process-world`**: `temper/tests/one-process/src/restart_control.rs:37`, `checked_setup`; read by `src/world.rs:660`, `service`; fixture setup document `src/stories.rs:264`, `configure`.
- **`temper` test kit**: `tests/support/owned.rs:388/876/1044`, `file`, `deployment_profile`, `checked_setup`; `tests/support/first_repository.rs:705`, `generated`.
- **Setup type fixtures/tests:** `temper-forge-setup/src/tests/fixtures.rs:22/32/35/45/98/104`, `fake_setup`, `setup`, `read_setup`, `members`, `owned`, `place`; `tests/read.rs:10/25/56/271/345/367/397`, entry/path/start cases; `tests/members.rs:28/56/86/98/174`, cross-check cases; `tests/managed.rs:26–114`, managed-string/legacy cases; `tests/given.rs:67/75/83`, all mode mappings.
- **Shell test kit:** `temper-forge-setup-shell/tests/support/mod.rs:27/56`, `Scratch::place`, `settings`; `tests/suite/prepare.rs:13/33`, `owned`, `run`; `tests/suite/files.rs:21`, clear/layout case; `tests/suite/fetch.rs:80`, `local_tls_fetch`; `tests/suite/reconcile.rs:26`, `done`.
- **Service fixtures/tests:** `temper-service/src/tests/fixtures.rs:1030`, `owned_setup`; `tests/setup_create.rs:230/263/302/341/354/431`, configuration, intent, create/failure/playground cases; `tests/setup_adoption.rs:7/38/141/176/210/247/341/424/479/497`, old/new setup and adoption paths; `tests/setup_admin.rs:16`, `config`; `tests/setup_scopes.rs:72`, preparation scopes; `tests/startup.rs:464/467/573/1865/2134`, checked setup/configuration readers; `tests/sign_in_configuration.rs:272`, `forge_metas_escape_attributes_publish_modes_and_change_the_etag`.
- **Binary tests:** `temper/src/tests.rs:161/308`, listener clashes and mode refusals; `src/init/tests.rs:317`, generated configuration admission; `src/forge/tests.rs:27/62/91`, setup checking; `src/fake_backends/tests.rs:225/644/751`, retained setup/step checks; `tests/suite/startup.rs:872`, `every_command_checks_setup_before_reading_files`.

**5. Fake forge and fake LLM; ports and box integration**

- **`temper-fake` is a library.** The executables are subcommands of `temper`: `temper/src/fake.rs:143`, `configure`; `Arguments`:129; `Source` selects standalone document, deployment setup forge, or deployment demo.
- `fake.rs:335`, `forge_document`, reopens `temper.json`, finds the setup entry, reads its handover and calls `temper_fake::forge::Document::fake`.
- `fake.rs:371`, `llm_configuration`, reopens the deployment, reads demo configuration/key and calls `llm::configuration`.
- `fake.rs:402`, `start`, creates the appropriate peer, adopts inherited listeners at :453 and installs termination handling. Forge state is held by the peer, not restored from an observation file.
- **`temper-fake/src/forge.rs:43`**, `Document`; `fake`:53 maps setup listen/owner/repositories; `seed`:200 creates fresh domain/git state; `setup`:284 seeds setup accounts; `peer`:374 builds the hosted forge peer.
- **`temper-fake/src/llm.rs:20`**, `Document`; `Configuration`:130; `configuration`:143; `peer`:383 / `recording_peer`:388. Scripts and model declarations come from the demo or standalone document.
- The ordinary playground’s three-step group and its readers are listed in question 1; neither fake has a fixed hardcoded listening port.
- **Default preferences:** `temper/src/init.rs:272`, `core`, requests `[8367,8368,8369,8370,8371]` for clients, owned Forgejo, fake forge, fake LLM, webhooks respectively. `Io::port`:78 / `Io::ports`:108 select another kernel port if occupied; `write_with_program`:209 maps them into `temper_init::Ports`.
- **`temper-init/src/lib.rs:33`**, `Ports`; `template.rs:53`, `listener`, renders `127.0.0.1:PORT`. The selected nonzero values are written into the document and held by the box.
- **`--fake-backends` is implemented.** `temper/src/fake_backends.rs:39/67/472`, `configure`, `run`, `prepare`.
- **`temper-fake/src/backends.rs:38`**, `steps`, returns four always-long prefix steps: `fake github`, `fake issuer`, `fake model`, `fake forgejo`, all `--document ... --seed 7`; `SEED`:31; `Peers`:24 holds their four addresses.
- `fake_backends.rs:235`, `reserve`, chooses nonzero loopback ports excluding deployment ports and callback `1455`; `checked_peers`:261 validates retained ports. No fixed peer-port quartet exists.
- **`temper-fake/src/backends.rs:350`**, `rewrite`, changes both deployment and agent model endpoints, GitHub roots, issuer roots, bundle and worker paths; `managed`:449 adds forge/identity/setup number 4; `forge`:496 targets existing GitHub number 2.
- `fake_backends.rs:223`, `directories`, retains effective config/data and shares cache; `prepare`:472 retains compatible documents across restart. Peer memory is fresh each launch.
- **Direct readers:** `temper/src/fake_backends.rs:67/261/472`, `run`, `checked_peers`, `prepare`; `temper-fake/src/backends/tests.rs:70/119/163/183/291/398`, supervision, rewriting, generated-peer and managed-admin cases; `temper/src/fake_backends/tests.rs` exercises preparation/retention/import.
- The draft’s “flag/OAuth/supervision missing” statements in `temper/docs/design/draft/fake-backends.md` are stale; adopted description is `temper/docs/design/protocol/setup.md:1414`, §7.5.
- There is **no implemented box group-removal/reset path** for these fakes.

**6. Tests, binaries, worlds and kits**

- **`temper` manifest — `temper/crates/temper/Cargo.toml`**: binary `process`:97 → `tests/process.rs`; binary `suite`:131 → `tests/suite/main.rs`; binary `qa_all`:135 → `tests/qa_all/main.rs`.
- **Profile distinction:** `.config/nextest.toml:180`, `profile.process-temper`, selects binary `process`; subprocess tests in `suite` and `qa_all` are separate binaries despite also spawning the shipped box.
- **Shared process kit — `temper/tests/support/boxed.rs`**: `Plan`:25, `Outcome`:34; `run`:170 / `run_echo_steps`:158 spawn box; `run_command`:185 uses `skein_world::end_to_end::Binary::start_with_tree` and `real::World<Judge>`; `run_ctl`:690 and variants converge on `run_ctl_peers`:856 with `CtlJudge`; `run_story`:372 uses `StoryJudge`; `dying_with_test`:971 installs parent-death cleanup.
- Other kits: `tests/support/ctl.rs` supplies scripted client calls; `support/owned.rs:564`, `Owned`, and :684, `Bounded`; `support/first_repository.rs:705/741/753`, `generated`, `stage_binaries`, `assert_box`; `support/forge_land.rs:577/592/627`, cached binaries, setup steps and forge oracle.
- **Simulated counterpart:** `temper-one-process-world`, `temper/tests/one-process/src/world.rs:660`, `service`, prepares the service directly; `box_listener.rs:12`, `Listener`, models the box-held webhook listener. It does **not** run `temper-box`.
- **Host real-loop test:** `temper-box/tests/real_loop.rs:434`, `a_split_sign_in_link_stays_whole_on_the_shared_terminal`, tests `temper_box::host::Service` with synthetic children, not the shipped `temper box`.

Relevant `process` tests, all in `temper/crates/temper/tests/process.rs`:

- :140 `the_box_finishes_its_restart_policy_after_a_refused_restore`.
- :1217 `temper_box_prints_its_link_and_stops_its_processes`; :1316 `temper_box_restarts_a_killed_forgejo_while_the_engine_serves`.
- :1527 `a_fresh_init_s_first_chat_reads_its_forge`; :1600 `a_first_run_answers_on_the_demo`.
- :1606 `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`; :1705 `a_csv_change_fails_the_persons_gate_is_repaired_and_lands_on_the_owned_forge`.
- :1783 `a_first_run_plays_the_demo_then_sets_up_a_forgejo_here_with_no_restart`.
- :2100 `a_first_run_answers_on_the_demo_and_a_second_finds_it_running`; :2104 `two_first_runs_one_after_the_other_choose_their_own_ports`; :2192 `a_first_run_ctl_demo_start_answers_running_twice`.
- :2224 `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`; shared `first_run`:2439 and quiet-log oracle `assert_quiet_logs`:2499.
- :3830 `a_default_box_with_a_sign_in_registration_starts_without_dns`; :4073 `a_story_box_and_its_steps_die_when_the_test_process_is_killed`.
- :4553 `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge`.
- Other box-backed work/restart consumers: :1437 review exhaustion; :1929 plan landing/restart; :2039 coder failure; :4468 settings revision with running work; :4618 green-light goal; :4779/4785/4790/4795/4800/4805 restart cases, using `held_review_restart`:4847.
- Related owned-command tests: :1077 `forge_prepare_makes_an_owned_forge_then_remakes_nothing`; :1112 `a_prepared_forge_runs_forgejo_and_its_runner_once_each`; :1138/1165 upgrade cases; :1183 reconcile convergence.

`process::fake_backends` tests, under `temper/crates/temper/tests/fake_backends/`:

- `mod.rs:114`, `the_serve_child_reads_retained_effective_documents_across_restart`.
- `walk.rs:128`, `forgejo_elsewhere_and_a_key_reach_a_first_chat_on_the_fake_box`.
- `walk.rs:145`, `chatgpt_sign_in_and_github_one_click_reach_a_first_chat_on_the_fake_box`.
- `walk.rs:150`, `github_one_click_and_a_key_reach_a_first_chat_on_the_fake_box`; :155, `githubs_own_app_and_a_key_reach_a_first_chat_on_the_fake_box`.
- `walk/here_key.rs:39`, `forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`.
- `walk/program_here.rs:44`, `a_program_and_forgejo_here_reach_ready_on_the_fake_box`; :91, `a_program_and_imported_account_survive_a_box_restart`.
- `walk/adoption.rs:28`, `context_preserves_the_writer_and_a_refused_setup_preserves_the_first_key`.
- `closed.rs:7`, `a_closed_chat_keeps_its_turns_and_replays_its_first_close`; :89, `a_cancel_replays_after_the_task_ends_and_a_new_key_is_refused`.
- `qa.rs:237/328/359/380/400/413`, `qa_split_directories_isolate_source_records_and_rewrite_every_public_root`, `qa_restart_preserves_effective_edits_data_and_secrets`, changed-source/different-source/partial-preparation refusal, concurrent-box refusal.
- `qa.rs:449/456/464/471/479/508`, extra worker argument, duplicate provider, missing GitHub target, duplicate endpoint, unprovisioned bundle and invalid bundle refusal tests.
- `oauth.rs:232`, `standalone_oauth_signs_in_and_answers_the_first_model_chat`, uses standalone OAuth process wiring rather than the four-peer box prefix.
- Walk helpers: `walk.rs:11`, `Walk`; `walk/github.rs:19/33`, `one_click`, `own_app`; `walk/chatgpt.rs:3`, `run`; `walk/elsewhere.rs:45`, `run`.

Other shipped-box test readers:

- **`suite` — `tests/suite/box_listeners.rs:40/91`**, `a_fresh_box_holds_serves_clients_port`, `a_later_box_refuses_a_taken_port_naming_the_port_and_document`.
- **`suite` — `tests/suite/startup.rs:409/434/459`**, fresh init/dormant forge, inherited listener adoption, unmatched listener refusal; :525/541/550, missing document/members/unknown forge; :559–626, box limits; :1364, init-directory policy; :1414/1452, box fd validation/placement; :1482/1515/1563, old-form versus records-only handover; :1832/1859/1938/1944/2054, Forgejo listener/parent lifecycle.
- **`qa_all` — `tests/qa_all/qa_first_run.rs:88/119/212/217`**, dormant owned port, demo end on owned connect, bad cache, group-label failure; :373/379/385, parent termination/interrupt/death.
- **`qa_all` — `tests/qa_all/qa_forge_setup.rs:41/104/136/174/210/250/268`**, default asked forge, distinct ports, dormant directory/cache absence, missing-file subsets, unasked missing-file refusal, missing box pipes, drain diagnostics.
- **`qa_all` — `tests/qa_all/qa_forge_handover.rs:420/509/581/608`**, `fake_box_restart_replaces_admin_and_restores_adopted_access`, `asked_owned_setup_connects_and_box_record_restarts_it`, `removal_failure_blocks_reconcile_then_next_start_recovers`, `asked_prepare_failure_is_visible_and_can_be_asked_again`.
- **`qa_all` — `tests/qa_all/qa_adoption.rs:193/282`**, `a_running_box_adopts_every_seeded_repository_before_connecting`, `the_default_box_prepares_adopts_and_hands_over_the_link`.
- **`qa_all` — `tests/qa_all/qa_first_repository.rs:43`**, `first_landed_task_and_adoption_survive_a_box_restart_without_setup_replay`; `qa_landing_by_turn.rs:17`, `a_change_lands_by_turn_on_the_owned_forge`.

- **Missing tests/implementation:** no `remove N` process story; no EndPlayground project/forge/account erasure story; no box killed mid-reset and resumed removal; no setup killed at every step then removed/recreated from cached binaries. Existing handover/preparation recovery and successful-restart tests do not establish those draft requirements.