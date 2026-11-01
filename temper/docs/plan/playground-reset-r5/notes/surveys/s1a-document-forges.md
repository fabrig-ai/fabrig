Read-only survey; no builds or Cargo commands run. Draft §§1–7 read first.

Path abbreviations below identify both crate and repository directory:

- **S** = `temper-service`, `temper/crates/temper-service`
- **T** = `temper`, `temper/crates/temper`
- **I** = `temper-init`, `temper/crates/temper-init`
- **F** = `temper-forge-setup`, `temper/crates/temper-forge-setup`
- **D** = `temper-engine-domain-forge`, `temper/crates/temper-engine-domain-forge`
- **R** = `temper-engine-domain`, `temper/crates/temper-engine-domain`
- **K** = `temper-fake`, `temper/crates/temper-fake`
- **J** = `jig-protocol-config`, `jig/crates/jig-protocol-config`

**1. Configuration types, parsing, validation, defaults, readers**

- **Document entry points — S:** `src/startup.rs:414` `parse_with`; `:423` `parse`; `:683` `named_with`; `:719` `named`; `:778` `checked_start_with`. `named_with` returns paths/public configuration; checked startup resolves supplied file contents and runtime endpoints.
- **Parsed application — S:** `src/startup/application.rs:18` `Members`: `forges`, `identities`, `demo`, `asked_owned`, limits and forwarded overrides. `:770` `parse` requires `application.forges`, `identities`, `setup`, `demo`, `limits`.
- **Public startup result — S:** `src/startup.rs:222` `Named`; `forges: Box<[startup::Forge]>`, `identities: Box<[startup::Identity]>`, `llm`, projects, listeners, overrides.
- **Runtime result — S:** `src/settings.rs:139` `Engine`; `forges: Box<[settings::Forge]>`, `demo: Option<Demo>`, `setup: Box<[Setup]>`, `records_only`, `identities`, jig configuration and limits.

- **Forge document type — S:** `src/startup/application.rs:96` `Forge`: `records_only`, `number: u16`, `kind`, `api`, `transport`, `git`, `git_program`, `web`, `app_read_only`, transitional credential-file paths, `user`, joined identity number.
- **Forge runtime type — S:** `src/settings.rs:55` `Forge`: adds `mode`; contains resolved transport, optional credential values, numeric user and identity number.
- **Kind — S:** `src/settings.rs:80` `Kind::{Forgejo, GitHub}`. **Fake is presently a mode, not a kind.**
- **Parser — S:** `src/startup/application.rs:355` `forge`: requires number/kind/API/transport/git; accepts `"forgejo"` and `"github"` only.
- **Numbers — S:** `src/startup/application/read.rs:83` `short` checks conversion to `u16`; `application.rs:572` `references` rejects duplicate forge numbers. Numbers come from the document; there is no runtime forge-number allocator.
- **List bounds — S:** `src/startup/application.rs:788` `parse` bounds forges by `limits.forge.client.repositories`; `:812` rejects an empty list; `:815` requires identity count equal to forge count.
- **Default mode — S:** `src/startup/application.rs:1169` `resolve` constructs `settings::Forge` with `Mode::Connected`; `:1045` `git_programs` overlays Managed/Owned/Fake from checked setup entries.
- **Connector modes — D:** `src/boundary.rs:282` `Mode::{Connected, Managed, Owned, Fake}`.

- **Addresses — S:** `src/startup/application.rs:355` `forge`; `:213` `transport`; `:269` `git_base`; `:484` `forge_endpoint`.
  - GitHub `api:null` → `api.github.com:443`; `transport:null` → TLS for that name, machine roots.
  - GitHub `git:null` and `web:null` → `https://github.com`.
  - Forgejo supplies API/transport/git; web must be null, with missing web tolerated by the transitional parser.
  - API rejects in-process addresses; plaintext requires a loopback socket.
  - Git/web require HTTP(S), a host, no credentials/query/fragment/trailing slash; HTTP requires loopback or localhost.
  - Optional `git_program` is nonempty, NUL-free, at most 4096 bytes; resolved under data or absolute.
- **Path/value resolution — S:** `src/startup/application.rs:1250` `resolve_forge`; `:1021` `resolved_transport`; `:1109` `resolved_user`; `:1134` `resolve`.
- **GitHub listener coupling — S:** `src/startup/application.rs:709` `github_listener`: configured GitHub requires a fixed client-listener port.
- **Read-only App default — S:** `src/startup/application.rs:367` `forge`: absent `app_read_only` → false; its presence on Forgejo is refused.
- **Design/code discrepancy:** `temper/docs/design/protocol/configuration.md:240` documents `log_origins`; neither parsed `Forge` nor `forge()` currently accepts it. **S** `src/forge.rs:93` `settings` supplies an empty component origins list. Limit names/checks exist independently.

- **Git identity document type — S:** `src/startup/application.rs:148` `Identity`: number, optional forge join, transitional token path/presence flag, login, author.
- **Runtime identity — S:** `src/settings.rs:96` `Identity`: number, forge, token, login, `Author`; `:106` `Author`: name/email.
- **Identity parser — S:** `src/startup/application.rs:502` `identity`; `:572` `references`.
  - Exactly one identity per forge and one forge per identity; duplicate identity numbers rejected.
  - Preferred join is `identities[].forge`; transitional `forges[].identity` remains accepted.
  - Login excludes `:` and controls; author fields are nonempty and exclude NUL/newline/`<>`.
  - Null login becomes empty pending connect-time bot defaults; null author is permitted for GitHub, refused for Forgejo by `references`.
  - Old Forgejo token/API/user/webhook file members still exist; partial old forms are validated, rather than silently treated as records-only.
- **Identity limits — S:** `src/startup/checks.rs:212` `credentials`; `:443` `identities`: configured IDs must fit engine/worker credential capacities and encoded values must fit the worker wire.

- **Setup types — F:** `src/setup.rs:14` `Account` (login/email); `:28` `Owned`; `:63` `Fake`; `:71` `Mode`; `:78` `Entry`; `:88` `Setup`.
- **`Entry` fields — F:** `src/setup.rs:78`: forge number, directory, bot, orgs, mode, `asked: bool`.
- **Setup parser — F:** `src/setup.rs:93` `Setup::read`; `:417` `mode`; `:451` `entry`; `:535` `list`.
  - Modes: `"managed"` → records-only admin connect; object `managed` → legacy admin-token file; `owned`; `fake`.
  - `start` absent → `asked=false`; `"asked"` accepted only for owned mode.
  - Managed string form has no directory/orgs; owned/fake carry their listener and local setup details.
- **Setup cross-check — F:** `src/check.rs:21` `Forge`; `:34` `Members`; `:40` `Checked`; `:64` `members`; `:181` `checked`.
  - Checks forge references, unique forge/directory entries, local API/git/listener agreement and listener conflicts.
  - Checks bot/person distinction, old credential-path agreement and mode/start restrictions.
- **Runtime setup wrapper — S:** `src/settings.rs:130` `Setup` holds checked `Entry` and optional handover.
- **Who supplies cross-check inputs — T:** `src/setup.rs:17` `check_with`: joins `Named.forges` to identities and constructs `F::check::{Forge,Members}`.

- **Public demo type — S:** `src/settings.rs:7` `Demo`: project, forge, provider, owner/name, cues, landing, key-file path and optional domain state.
- **Public demo parser — S:** `src/startup/application/demo.rs:26` `parse`: accepts null; otherwise reads project/forge/LLM provider/key/asked cues; forge must be configured and name fake setup; owner/first repository/landing come from that setup.
- **Domain demo conversion — S:** `src/settings.rs:20` `Demo::domain`: derives adoption host from configured forge git URL and supplies setup deadline.
- **Full scripted demo — K:** `src/demo.rs:46` `Demo`; `:73` `Facts`; `:82` `read`; `:155` `unique_listen`; `:171` `models`.
  - Checks project existence, fake setup, listener uniqueness, scripts and key path.
  - Provider must offer keys alone, have models, and each model’s endpoint must match fake LLM listener and plaintext requirements.
- **Full demo parser caller — T:** `src/setup.rs:17` `check_with`, specifically `:84`–`:99`, builds `Facts` from `Named.llm`, projects, checked setup and listeners.

- **LLM configuration — J:** `src/lib.rs:53` `Engine`, field `llm: Llm`; `:75` `engine` requires `llm`.
- **LLM types/parser — J:** `src/llm.rs:11` `Llm` (endpoints/models/descriptions/providers); `:22` `Endpoint`; `:40` `Transport`; `:44` `parse`; `:130` `endpoint`; `:233` `model`.
- **Provider type/parser — J:** `src/llm/providers.rs:11` `Provider`; `:29` `Registration`; `:57` `parse`; `:105` `provider`. Endpoint names/numbers and model names are unique; model endpoint/provider joins are checked; at most 64 models. There is no implicit scripted provider.
- **LLM readers relevant here:** **J** `src/core.rs:11` `core`, `src/files.rs:13` `files`; **S** `src/startup.rs:187` `redirects`, `:384` `engine_with`, `:683` `named_with`; `src/startup/checks.rs:28` `stock`, `:761` `agent`, `:939` `socket_count`; `src/values.rs:10` `credential_bytes`; **T** `src/accounts_runtime.rs:6` `resolve`, `src/setup.rs:17` `check_with`; **K** `src/demo.rs:171` `models`. The draft retains these generic LLM members.

- **Deployment act grants — J:** `src/lib.rs:58` `Engine::acts`; `:144` `engine` calls `src/acts.rs:8` `parse`.
- **Grant parser — J:** `src/acts.rs:43` `grant`: optional acts defaults to empty; checks connector references, unique connectors/kinds; accepts `close`, `add_key`, `remove_account`, `adopt`, `make_project`.
- **Grant types — `jig-domain-people`:** `jig/crates/jig-domain-people/src/boundary.rs:673` `ActKind`; `:688` `ActGrant { connector, may }`.
- **Grant reader — J:** `src/core.rs:72` `core` copies `engine.acts` into core `Config::grants`.
- **Absent:** no `RemoveForge`, `RemoveProject`, `StartPlayground` or `EndPlayground` Rust variants exist in `temper/crates`/`jig/crates`.

- **Limits — S:** `src/limits.rs:45` `Limits`; `:78` `Forges` (installations/log_origins); `:95` `CLIENT`; `:145` `FORGE`; `:331` `profile`; `:358` `apply`.
  - Small defaults: `forge.repositories=2`, `forge.client.repositories=2`; routing installations=8, log origins=4.
  - **No `forge.forges` limit exists.**
  - **D** `src/limits.rs:17` `Limits` contains repositories/tasks/etc., not a separate forge count.
- **Limit names — S:** `src/limits/names.rs:70` `apply` handles `forge.repositories`; `:119` client repositories; `:190` installations; `:210` component forges; `:236` component repositories.
- **Component defaults — S:** `src/forge.rs:165` `small`: component forges=4; repositories copied from domain `forge.repositories`; connections from client calls; deadline from client lifetime.
- **Setup derivation — S:** `src/startup/application.rs:161` `configured_limits`, especially `:209`, forwards resolved `forge.client.repositories` even without an override.
- **Setup recipient — F:** `src/limits.rs:103` `small`; `:118` `apply`: forwarded client repositories becomes setup `Limits.repositories`. **S** `src/setup/settings.rs:253` `local_starts` also copies it into local plan limits.
- **No current sum over configured forge entries derives `forge.repositories` or its kin.** They are profile/override values. Configuration count still controls:
  - **S** `src/startup/application.rs:788` `parse`: forge-list bound and identity-count equality.
  - **D** `src/domain.rs:492` `Domain::new`: configured/writer maps use client repositories; identity map uses domain repositories.
  - **S** `src/forge.rs:454` `config`: identity list sized from forge count.
  - **S** `src/forge/tokens.rs:36` `Tokens::new`; `src/forge/records.rs:17` `Rows::new`: configured-number tables.
  - **S** `src/startup/checks.rs:1017` `record_writes`: one Origin inventory entry per forge plus five scalar record shapes.
  - **S** `src/system.rs:1097` `note_bound`: configured forge count plus demo-act diagnostic bound.
- **Other limit readers/checks — S:** `src/startup/checks.rs:252` `limits`, `:325` `longest_remote`, `:387` `payload`, `:655` `root`, `:1061` `records`; `src/worker_settings.rs:84` `payload_limits`; `src/translate.rs:10` `limits`.
- **Underlying component check — `skein-forge-connection`:** `skein/crates/skein-forge-connection/src/limits.rs:7` `Limits`; `:371` `check_settings` checks configured count against component forges.

- **Memory — J:** `src/lib.rs:66` `Engine::memory: u64`; `:146` `engine` reads required `memory_bytes`.
- **Memory admission — S:** `src/startup.rs:285` `engine_limits` copies it into root limits; `:305` `check_memory_bytes` rejects zero; called by `named_with` too.
- **Memory projection — S:** `src/limits.rs:372` `engine_candidate` initially uses `u64::MAX`, replaced by configured budget during startup.
- **Runtime reader — R:** `src/limits.rs:35` `Limits::memory_bytes`; `src/domain.rs:612` `within_budget` compares held bytes to it. **No forge-count memory formula remains here.**

- **Remaining production readers of forge/identity/demo configuration — S:**
  - `src/startup.rs:21` `setup_files`, `:41` `handover_reads_with`, `:81` `setup_values`, `:778` `checked_start_with`, `:1063` `prepare_forges`.
  - `src/startup/application.rs:908` `files`, `:912` `asked_files`, `:915` `listed_files`, `:1080` `missing_asked_files`, `:1134` `resolve`, `:1291` `configured`.
  - `src/startup/application/landing.rs:175` `parse`, `:409` `project_landing`; `src/startup/application/sign_in/forges.rs:69` `check`.
  - `src/system.rs:165` `Service::prepared`, `:960` `forget_handover`; `src/engine.rs:116` `Engine::new`, `:310` `iterate`, `:426` `clients_pass`; `src/route.rs:9` `request`.
  - `src/forge.rs:12` `ended`, `:42` `settings`, `:114` `file_access`, `:218` `Layer::new`, `:409` `indexed`, `:436` `prepare`, `:454` `config`.
  - `src/forge/records.rs:67` `Loader::new`, `:130` `advance`; `src/forge/tokens.rs:36` `Tokens::new` and its token/accepted/connected/account/announce operations.
  - `src/setup/settings.rs:8` `fixed`, `:104` `forgejo`, `:163` `prepare`, `:253` `local_starts`, `:329` `source_forge`.
  - `src/setup/lifecycle.rs:544` `configured`, `:589` `connected`, `:707` `advance`, `:824` `restored`, `:1104` `connect_local`, `:1208` `take_read`, `:1266` `take_removal`.
  - `src/setup/routes.rs:251` `route`; `src/setup.rs:82` `Layer::webhooks_listening`, `:97` `Layer::new`.
  - `src/workers.rs:34` `Workers::forges`, `:147` `encoded_assignment`, `:474` `limits`, `:554` `configured`; `src/assignment.rs:74` `down`.
  - `src/translate.rs:25` `items`, `:41` `repository_item`, `:167` `identity`, `:181` `github_identity`; `src/values.rs:132` `Values::fill`.
  - `src/family.rs:27` `to_root`, `:305` `panel`, `:443` `address`, `:446` `pull_address`, `:951` `document`, `:1029` `landing_member`.
  - `src/clients.rs:450` `advance`, `:540` `iterate`, `:674` `endings_step`; `src/clients/bundle.rs:339` `forge_metas`, `:385` `fill_forge_sign_in`.
- **Remaining production readers — T:** `src/lib.rs:342` `forges`, `:378` `prepare_with_limits`; `src/forge.rs:153` `start`, `:312` `reconcile_settings`; `src/forge_sign_in.rs:13` `run`; `src/fake.rs:335` `forge_document`, `:371` `llm_configuration`; `src/fake/check.rs:79` `configuration`; `src/box_command.rs:47` `run_configuration`, `:189` `owned_notices`; `src/fake_backends.rs:261` `checked_peers`, `:331` `validate`, `:472` `prepare`.
- **Setup/fake readers — F:** `src/given.rs:103` `values`; `src/step.rs:144` `steps`. **K:** `src/steps.rs:21` `steps`; `src/backends.rs:350` `rewrite`, `:448` `managed`, `:496` `forge`.
- **Client instructions requiring migration — `temper-client-view`:** `temper/crates/temper-client-view/src/words.rs:2253` `Word::ForgeConfigCode`, `:2304` `Word::ForgeConfig` explicitly tell the user to edit `application.forges`.
- **Across lower trees:** no skein/smith/jig Rust or Cargo reader depends on temper’s configuration types or `Configured`; lower-level jig/forge types are inputs to temper, not reverse dependencies.

**2. `temper init`: template, files, ports**

- **Template embedding — I:** `src/template.rs:12` `TEMPER`, `:13` `AGENT`; `:15` `documents` substitutes binary/program paths, login/email and five listener addresses; `:53` `listener` formats `127.0.0.1:PORT`.
- **Template forge slots — I:** `template/temper.json:211` `application.forges`: 1=fake Forgejo, 2=GitHub with null default addresses, 3=owned Forgejo.
- **Identity slots — I:** `template/temper.json:241` `application.identities`: identity 1→forge 2 with null bot defaults; identity 2→forge 1; identity 3→forge 3.
- **Setup — I:** `template/temper.json:303`: forge 1, `demo/forge`, fake git `demo/bin/git`, owner `playground`, repository `widget`, landing `main`; `:328`: forge 3, `forge-3`, fetched Forgejo/runner, person login/email, `"start":"asked"`.
- **Demo — I:** `template/temper.json:358` `application.demo`: fake LLM listener/provider 3/key path/scripts; `:622` forge 1/project 1.
- **LLM — I:** `template/temper.json:60` demo provider 3, keys-only/model `demo`; `:84` demo endpoint 3 at `$llm`; `:111` model `demo`.
- **Grants/budget/limits — I:** `template/temper.json:3` connector 1 may close/remove_account; `:164` memory=1,073,741,824; `:629` client repositories=3; `:631` domain repositories=3.
- **Produced files — I:** `src/lib.rs:119` `files`: data `worker.secret`, `worker.digest`, `demo/llm.key` (private); data `demo/bin/git` symlink to binary; config `agent.json`, `temper.json`. `:19` `Facts`; `:33` `Ports`.
- **Shell writer — T:** `src/init.rs:180` `write_with_program` supplies discovered git/Claude paths, user login, random input and held listeners to `I::files`.
- **Port selection — T:** `src/init.rs:272` `core` defaults in order: clients 8367, owned Forgejo 8368, fake forge 8369, fake LLM 8370, webhooks 8371.
- **Reservation — T:** `src/init.rs:78` `Io::port` tries preferred loopback port, falls back to port 0; `:98` `listened` listens to reserve it exclusively and retries kernel allocation on a preferred-port listen race; `:108` `ports` holds all five sockets together.
- **Mapping/lifetime — T:** `src/init.rs:209` `write_with_program` maps socket indices to `Ports`; `:163` `Written` returns listeners. `:276` `run` closes them after standalone init; box startup can retain/hand them onward.

**3. Fixtures and migration size**

Counts are lexical `rg -c` estimates, not compiler-resolved call graphs.

| Crate | Main fixture definitions | Approximate uses |
|---|---|---:|
| S | `src/tests/fixtures/startup.rs:9` `APPLICATION`; `:61` `document` | 55 `fixtures::document` calls |
| S | same file `:17` `DEMO`, `:23` `DEMO_SETUP`, `:27` `demo_application`, `:35` `demo_document`, `:48` `demo_service` | 10 demo-helper calls |
| S | same file `:264` `webhook_document`, `:274` `engine_alone_document`, `:281` `worker_alone_document` | all document variants together: 91 calls/10 files |
| S | `src/tests/fixtures.rs:62` `engine`, `:125` `forges` | 130 `fixtures::engine` calls/28 files |
| F | `src/tests/fixtures.rs:12` `TWO`, `:17` `three`, `:22` `fake_setup`, `:32` `setup`, `:45` `members`, `:292` `hooked_members` | 49 qualified fixture calls |
| T | `tests/support/configuration.rs:6` `documents`, `:117` `old_form_check_document`, `:132` `check_document`, `:146` `demo_document` | 17 qualified calls; 20 including `fake_entry` |
| `temper-one-process-world` | `temper/tests/one-process/src/fixtures.rs:105` `document`, `:110` `world_document` | 5 qualified calls/2 files |
| `temper-real-loop-world` | `temper/tests/real-loop/src/fixtures.rs:16` `documents`, `:81` `write`, `:138` `bind` | 4 qualified calls/2 files |
| `temper-client-browser-world` | `temper/tests/client-browser/src/configuration.rs:25` `documents`, `:126` `write`, `:154` `demo` | 2 qualified callers |
| D | `src/tests.rs:1223` `configured`, `:1251` `setup_domain` | 54 lexical helper occurrences, including definitions |

- **Additional document fixtures — T:** `tests/fixtures/cluster/engine.json:187` `application.forges`; `tests/support/github/engine.json:176` same member.
- **T setup builders/editors:** `tests/support/owned.rs:21` `document`, `:43` `old_form`, `:823` `bounded_template`, `:876` `deployment_profile`; `tests/support/github_land.rs:801` `configure_engine`; `tests/support/live_github.rs:152` `configure_engine`; `tests/support/first_repository.rs:705` `generated`.
- **Other T fixture readers/editors:** `tests/support/account_file_qa.rs:22` `configure`; `account_move_qa.rs:44` `configure`; `account_move.rs:31` `configure`; `case.rs:202` `setup`; `forgejo_elsewhere.rs:137` `configure`; `landing_by_turn.rs:29` `new`; `webhooks.rs:26` `configuration`; `forge_sign_in.rs:11` `configure`; `forge_land.rs:747` `live_deployment`; `forge_land/rung.rs:821` `setup_tools`.
- **World-specific document editors:** `temper/tests/one-process/src/stories.rs:264` `configure`; `lower/github.rs:36` `configure`; `temper/tests/real-loop/src/github.rs:148` `write`; `src/forge.rs:45` `write`; `src/fixtures.rs:147` `bind_documents`; `temper/tests/client-browser/src/lib.rs:197` `github`.
- **Template readers outside init:** S `src/tests/stock.rs:10` `generated`, `welcome_policy.rs:17` `generated`; K `src/backends/tests.rs:15` `fixture`, `src/tests/demo.rs:413` `template`, `src/oauth/tests.rs:102` `defaults_and_registration_follow_the_shipped_chatgpt_template`; `temper-engine-domain-world` `temper/tests/engine/domain/tests/fixtures/stock.rs:2` `box_model`, `src/forge_routes.rs:1918` `box_profile`; T `tests/support/first_repository.rs:114` `run_spend`, `:211` `csv_scripts`; `tests/suite/fake_opening.rs:190` `demo_chat_fixtures_select_their_opening_requests`.
- **I template assertions:** `src/tests/documents.rs:58` `the_default_is_the_owned_example_so_edited`; `:394` `no_placeholder_is_left`; `:474` `the_template_keeps_project_one_without_settings_members`; T `src/init/tests.rs:94` `init_finds_ports_and_programs_and_writes_exact_modes_in_order`.

- **S shared-fixture reader files** (named anchors; these modules contain the individual assertions):
  - `src/tests/startup.rs:12` `transcript_document`; `checks.rs:28` `standalone_shape_is_required_of_jigs_parts`; `root_checks.rs:38` `engine_with`; `limits.rs:108` `the_small_profile_passes_each_domains_validity`.
  - `setup.rs:51` `github_config`; `setup_adoption.rs:7` `old_form_configured`; `setup_reachable.rs:240` `picker_config`; `setup_scopes.rs:72` `preparations_four_scopes_grant_adopt_and_without_repository_it_is_refused_at_repository`.
  - `family.rs:13` `configured`; `forge.rs:377` `temper_word_keeps_the_marker_bytes_and_length`; `git_tokens.rs:89` `configured_login_and_author_override_the_app_defaults`; `translate.rs:8` `a_workspace_reaches_items_with_each_remote_made_from_its_base`.
  - `system.rs:156` `standalone_announces_clients_without_announcing_in_process_workers`; `engine.rs:395` `startup_admission_does_not_promise_a_connected_forge`; `worker.rs:455` `grant_sizes_take_credentials_source_names_and_generated_identities`; `worker_parts.rs:164` `an_engine_s_items_reach_the_worker_s_workspace_field_for_field`.
  - `workers.rs:123` `an_assignment_carries_its_items_encoded_charter_and_identity_value`; `clients.rs:159` `a_sign_in_ending_waits_for_its_erase_to_commit`; `server.rs:236` `a_call_past_the_mounts_room_is_answered_busy`; `clients_unwatch.rs:12` `EXPIRY`.
  - `sign_in_configuration.rs:11` `jig`; `sign_in.rs:44` `world_settings`; `accounts.rs:93` `configured_accounts_are_refused_at_their_path`; `accounts_host.rs:785` `registered`.
  - `store_route.rs:590` `loaded_tail`; `stop/pre_settings.rs:197` `refuse`; `box_link.rs:343` `demo_end_diagnostics_reach_serves_named_notes`.
  - Outside `src/tests`: `src/startup/calls_tests.rs:8` `document`; `src/clients/demo_tests.rs:23` `start_demo_reads_each_time_and_carries_the_key_before_admission`; `src/clients/google/tests.rs:56` claims-reader test; `src/forge/records.rs:211` unconfigured-number test; `src/workers.rs:545` channel-limit test; `src/server.rs:375` listener-limit test.
- **F direct cross-check tests:** `src/tests/managed.rs:57` `managed_string_refuses_api_token_files`; `src/tests/members.rs:16` `a_forge_whose_api_token_names_another_file_is_refused_naming_its_path`, `:174` `records_only_local_members_skip_files_but_keep_local_checks`.
- **T direct document/slot tests:** `tests/qa_all/qa_adoption.rs:9` init configuration test; `qa_forge_setup.rs:41` owned-slot test, `:104` five-port test; `qa_forge_handover.rs:35` `configured`; `qa_first_run.rs:119` playground-end test; `qa_first_repository.rs:43` restart test; `tests/suite/startup.rs:333` `managed`, `:1733` checked-profile test; `tests/suite/forge.rs:321` TLS-refusal test; `tests/suite/fake.rs:446` `forge_setup`; `tests/process.rs:2416` `playground_end_second_forge`.
- **T remaining fixture readers:** `tests/fake_backends/walk.rs:115` `forge_number`; `qa.rs:237` split-directory test; `src/forge/tests.rs:37` records-only refusal test; `src/tests.rs:379` unresolved-host test.

**4. Forge connector construction input and reverse dependencies**

- **Input — D:** `src/boundary.rs:298` `Configured`:
  - `admin_connect: bool`, `bot: Box<[u8]>`, `owned: Option<Owned>`, `git`, `web`, `read_only`;
  - `number: u16`, `kind: Kind`, `mode: Mode`, `asked: bool`.
- **Owned public details — D:** `src/boundary.rs:291` `Owned`: Forgejo version, optional runner version, password-file **path**.
- **Builder — S:** `src/startup/application.rs:1291` `configured`:
  - Enumerates `settings.forges`; starts at Connected, no bot/owned details/admin-connect.
  - Joins setup by forge number; copies bot login; derives Managed/Owned/Fake.
  - Managed without admin-token file enables `admin_connect`.
  - Owned details use pinned versions and `Place::PersonPassword`.
  - `asked = demo forge OR matching setup.entry.asked`.
  - Copies kind/number/web/read-only; copies git only if within `PULL_ADDRESS_BYTES`, otherwise empty.
- **Host construction — S:** `src/system.rs:165` `Service::prepared`, especially `:221`–`:238`, builds `R::Config`, sets `under_box` from box-pipe presence.
- **Validation construction — S:** `src/startup/checks.rs:655` `root` builds the same public input with `under_box=false`.
- **Carrier — R:** `src/translate.rs:16` `Config`, fields `forges` at `:34`, `under_box`, identities/client config/demo.
- **Constructor — D:** `src/domain.rs:492` `Domain::new(limits, seed, client_config, identities, forges, under_box)`; validates pull-address size and copies rows into `configured`, keyed by number.
- **Constructor callers — R:** `src/domain.rs:275` `Domain::new`; `:1220` `refused` constructs the child to check configuration.
- **Routing capacity — R:** `src/domain.rs:147` `connection_limits` forwards component forge/installation limits to **D** `src/domain.rs:197` `connection_limits`.

- **Production readers of `Configured` — D:**
  - `src/domain.rs:208` `configured`; `:238` `forges`; `:306` `offer`; `:1021` `step` (`BoxClosed`, `AppRecorded`).
  - `src/connect.rs:35` `start`; `:152` `admin`; `:349` `reachable`.
  - `src/demo.rs:135` connection-triggered demo ending; `src/reconcile.rs:291` `configured_forge` / `:294` `reference_resolves`.
  - `src/brief.rs:140` `pull_identity`; `src/landing.rs:113` `project_landing`.
- **Production readers — R:** `src/route/forge_route.rs:135` `landing_words`, `:164` `landing_pull`, configuration lookup at `:2647`; `src/domain.rs:159` `forges`; `src/route/panels.rs:20` forge-document generation.
- **Offer rule — D:** `src/domain.rs:306` `offer`: owned + asked + not ended, while NotSetUp/SetUpFailed/SettingUp; Offered under box, otherwise NotUnderTheBox. Other modes/states → NotOffered.

- **Direct `Configured` test construction/readers:** 51 lexical `Configured {` matches across 18 files, including its declaration; approximately 50 construction/update sites.
  - **D:** `src/tests.rs:1223` `configured`, `:1251` `setup_domain`; `src/tests/connect.rs:10` `github`, `:318` installation-update test; `src/tests/demo.rs:139` `configured`; `src/tests/brief.rs:272` configured-row clone.
  - **S:** `src/tests/startup.rs:1866` configured-input assertions; `src/tests/setup_adoption.rs:260`, `:435` `Configured` expectations; `src/tests/setup_lifecycle.rs:700` `Configured`.
  - **R:** `src/tests/setup.rs:17` `signed_in_on`, `:623` `admin_root`; `src/route/forge_route/tests.rs:728` `landing_forge`.
  - **`temper-client-world`:** `temper/tests/client/src/front.rs:161` first-run configuration; `src/front/account_acts.rs:125` `fixture_root`; `tests/suite/forges.rs:32` configured rows.
  - Same crate: `tests/qa_all/qa_forges.rs:55` `configured`, `:87` `configured_with`, `:90` `configured_accounts`; `qa_first_repository.rs:24` configured row; `qa_welcome_m3_owned.rs:14` `owned_fixture`.
  - **`temper-engine-domain-world`:** `temper/tests/engine/domain/tests/suite/demo.rs:31` configured rows; `tests/qa_all/qa_first_repository.rs:16`, `:212` configured rows.
- **Other typed forge test-kit readers:** `temper-client-world`, `temper/tests/client/src/fixtures.rs:21` `forge_settings`; `temper-system-world`, `temper/tests/system/src/composed/link.rs:29` `assignment` constructs `S::settings::Forge`.

- **Does not exist yet:** a stored forge-configuration row, runtime forge creation/number allocation, a separate `forge.forges` admission limit, or the draft’s removal/reset acts. Existing secret `Origin`/credential records do not replace `application.forges`; startup still needs that document list.