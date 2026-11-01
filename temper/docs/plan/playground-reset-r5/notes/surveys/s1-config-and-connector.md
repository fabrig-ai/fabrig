Read-only survey; no Cargo, builds, tests, or Git commands run. Paths are repository-relative. Within a crate group, paths beginning `src/` or `tests/` are relative to the stated crate directory.

**1. Document types, parsing, validation, defaults**

- **`temper-service`**, `temper/crates/temper-service`:
  - `src/startup/application.rs:18` — `Members`: parsed application document; `forges`, `identities`, `demo`, `asked_owned`, landing policy, listeners, limits, workers.
  - `src/startup/application.rs:96` — `Forge`: `records_only`, `number: u16`, `kind`, API address, transport, git base/program, web base, App ownership flag, transitional credential-file paths, user, identity number.
  - `src/settings.rs:55` — `Forge`: resolved runtime equivalent; credential values are `Secret`, addresses remain typed.
  - `src/settings.rs:80` — `Kind::{Forgejo, GitHub}`. Fake/owned/managed are modes, not additional kinds.
  - `src/startup/application.rs:143` and `src/settings.rs:87` — parsed/resolved `Transport::{Plaintext, Tls { server_name, trust }}`.
  - `src/startup/application.rs:148` — `Identity`: number, optional forge join, transitional token path, login, author.
  - `src/settings.rs:98` — `Identity`; `src/settings.rs:109` — `Author { name, email }`.
  - `src/settings.rs:130` — `Setup { entry, handover }`; `src/settings.rs:139` — `Engine`, containing resolved forge/setup/identity arrays.
  - `src/startup.rs:221` — `Named`: shell-facing parsed inputs, including forges, identities, files, LLM settings and listeners.

- **Parser and validation — `temper-service`:**
  - `src/startup.rs:463` — `load_document`: parses jig’s engine and temper’s application, checks cross-component references and bounds.
  - `src/startup/application.rs:770` — `parse`: requires `forges`, `identities`, `setup`, `demo`, limits and other application members.
  - `src/startup/application.rs:788` — forge list bounded by `forge.client.repositories`.
  - `src/startup/application.rs:812` — empty forge list refused; identity count must equal forge count.
  - `src/startup/application.rs:355` — `forge`: strict field parser; kinds `"forgejo"`/`"github"` only.
  - `src/startup/application/read.rs:83` — `short`: checks representability as `u16`; this helper itself does **not** require a positive number.
  - `src/startup/application.rs:572` — `references`: unique forge/identity numbers, exactly one identity per forge, reverse join, charter reference.
  - `src/startup/application.rs:213` — `transport`; `:269` — `git_base`; `:484` — `forge_endpoint`: URL/address/transport checks, including loopback restriction for plaintext.
  - `src/startup/application.rs:502` — `identity`: validates login and git author fields; supports nullable login/author and transitional token input.
  - `src/startup/application.rs:908` — `files`; `:912` — `asked_files`; `:915` — `listed_files`: enumerate named credential/trust files.
  - `src/startup/application.rs:1134` — `resolve`: constructs runtime `settings::Forge` and identities from parsed members and supplied file contents.
  - `src/startup/application.rs:1250` — `resolve_forge`: resolves paths; `:1045` — `git_programs`: joins setup’s fake git program and checks conflicting declarations.
  - `src/startup/application.rs:1291` — `configured`: joins forge, setup and demo into domain `Configured` rows.

- **Defaults — `temper-service`:**
  - `src/startup/application.rs:355`, `forge`: GitHub null API → `api.github.com:443`; null transport → TLS for that name; null git/web → `https://github.com`; `app_read_only` defaults false.
  - Same function: Forgejo needs its API/transport/git; its web field must be null/absent. Absence of old file credential members selects records-only handling.
  - `src/startup/application.rs:1291`, `configured`: default mode `Connected`; setup changes it to `Managed`, `Owned`, or `Fake`; `asked` comes from setup or demo.
  - `src/translate.rs:181` — `github_identity`: fills missing bot author information from App slug/user; explicit configured author values override defaults.
  - `src/setup/settings.rs:253` — `local_starts`: missing local identity login takes setup bot login.

- **Setup document — `temper-forge-setup`**, `temper/crates/temper-forge-setup`:
  - `src/setup.rs:14` — `Account`; `:28` — `Owned`; `:66` — `Fake`; `:74` — `Mode::{Managed, Owned, Fake}`; `:82` — `Entry`; `:94` — `Setup`.
  - `Entry` holds forge number, directory, bot login/email, orgs, mode and `asked`.
  - `src/setup.rs:99` — `Setup::read`: strict parsing and directory resolution; `Setup::entry` finds by forge number.
  - `src/check.rs:21` — `Forge`; `:34` — `Members`: configuration facts supplied for setup’s cross-checks.
  - `src/check.rs:64` — `members`: forge references, entry/directory/listener uniqueness, local endpoint/git agreement and transitional file-path agreement.
  - `src/check.rs:181` — `checked`: parses setup and overrides, validates the whole setup, identifies records-only entries.
  - `src/setup.rs`, `entry`: `start: "asked"` accepted only for owned mode; absence means automatic start. Owned `"fetched"` binaries resolve into versioned cache paths.

- **Demo — `temper-service`:**
  - `src/settings.rs:7` — `Demo`: project, forge, provider, first repository, cues, landing, key-file path, optional runtime state.
  - `src/startup/application/demo.rs:25` — `parse`: nullable `application.demo`; validates configured forge reference, requires its fake setup entry, takes that entry’s first repository, resolves the demo key path.
  - `src/settings.rs:20` — `Demo::domain`: derives the repository host from the configured forge git base.
  - **`temper-fake`**, `temper/crates/temper-fake/src/demo.rs:82` — `read`: separately validates the executable fake LLM/demo configuration against setup/project/provider facts.

- **LLM, deployment acts and memory — `jig-protocol-config`**, `jig/crates/jig-protocol-config`:
  - `src/lib.rs:53` — `Engine`: `acts`, `memory`, `llm` and other jig members.
  - `src/lib.rs:75` — `engine`; `:144` — parses acts; `:146` — required `memory_bytes`.
  - `src/llm.rs:11` — `Llm`; `:22` — `Endpoint`; `:45` — `parse`: endpoints, models, providers, uniqueness and provider/model bindings.
  - `src/llm/providers.rs` — `Provider`, `Registration`, `parse`: provider numbers, supported models and account-source declarations.
  - `src/acts.rs:8` — `parse`: absent acts → empty grants; checks connector references and uniqueness.
  - `src/acts.rs:43` — `grant`: accepted words are `close`, `add_key`, `remove_account`, `adopt`, `make_project`; duplicate/unknown words refused.
  - **`jig-domain-people`**, `jig/crates/jig-domain-people/src/boundary.rs:673` — `ActKind`; `:688` — `ActGrant`.
  - `jig/crates/jig-protocol-config/src/core.rs:72` — core configuration receives `engine.acts`.

- **Limits and memory — `temper-service`:**
  - `src/limits.rs:45` — `Limits`; `:78` — `Forges { installations, log_origins }`; `:148` — small `FORGE` profile; `:372` — `profile`.
  - `src/startup/application.rs:161` — `configured_limits`: small profile plus dotted overrides, including setup/box handoff.
  - `src/limits/names.rs:70` — `forge.repositories`; `:119` — `forge.client.repositories`; `:190` — `forges.installations`; `:210` — `forge_component.forges`.
  - `src/forge.rs:127` — `shared`: passes **one deployment-wide** `forge.repositories` value to the connection component.
  - `src/forge.rs:165` — `small`: default `forge_component.forges = 4`; component repository capacity derives from `forge.repositories`.
  - `src/startup/checks.rs:1017` — `record_writes`; `:1061` — `records`: record-size and component-bound checks, some inspected per configured forge.
  - `src/startup/checks.rs:325` — `longest_remote`: maximum git-base length over forges, **not a sum**.
  - `src/startup.rs:285` — `engine_limits`; `:305` — `check_memory_bytes`: requires nonzero memory budget and passes it to the domain.
  - **`temper-engine-domain`**, `temper/crates/temper-engine-domain/src/limits.rs:35` — `Limits::memory_bytes`; `src/domain.rs:613` — admission compares held bytes with that budget.
  - **There is no per-forge repository-limit member or summation of repository limits over configured forges today.** `forge.repositories` is already connector-wide. There is no `forge.forges` field; present forge-count bounds are `forge.client.repositories` and `forge_component.forges`.

**2. Init template, ports and box**

- **`temper-init`**, `temper/crates/temper-init`:
  - `template/temper.json:211` — `application.forges`: 1 fake Forgejo, 2 GitHub, 3 owned Forgejo.
  - `template/temper.json:241` — identities: identity 1 → forge 2; identity 2 → forge 1; identity 3 → forge 3.
  - `template/temper.json:303` — setup: fake forge 1 at `demo/forge`; owned forge 3 at `forge-3`, fetched Forgejo/runner, `start: "asked"`.
  - `template/temper.json:358` — demo: forge 1, project 1, provider 3; fake LLM listener/key/scripts.
  - `template/temper.json:3` — `engine.acts`: connector 1 may `close` and `remove_account`.
  - `template/temper.json:31` — `engine.llm`: OpenAI plus demo provider; demo endpoint uses `$llm`.
  - `template/temper.json:164` — `memory_bytes = 1073741824`; `:625` — application limits admit three configured forges.
  - `src/template.rs:12` — `TEMPER`; `:15` — `documents`: substitutes binary, agent, git, five listeners, git bases, person login/email and program directory.
  - `src/lib.rs:19` — `Facts`; `:33` — `Ports { clients, owned, forge, llm, webhooks }`.
  - `src/lib.rs:119` — `files`: writes data `worker.secret`, `worker.digest`, `demo/llm.key`, `demo/bin/git` symlink; config `agent.json`, then `temper.json`. Declares private/public file modes.

- **`temper`**, `temper/crates/temper`:
  - `src/init.rs:78` — `Io::port`: binds preferred loopback port; bind failure falls back to port 0.
  - `src/init.rs:98` — `Io::listened`: listens to reserve exclusively; preferred-port listen failure also retries port 0.
  - `src/init.rs:108` — `Io::ports`: holds all five reservations together.
  - `src/init.rs:180` — `write_with_program`: discovers programs/person, supplies selected ports to `temper-init`, writes files, returns held listeners.
  - `src/init.rs:272` — `core`: preferred ports `[8367, 8368, 8369, 8370, 8371]` = clients, owned Forgejo, fake forge, fake LLM, webhooks.
  - `src/box_command.rs:225` — `configuration`: initializes a missing deployment and keeps init’s listener reservations.
  - `src/box_command.rs:47` — `run_configuration`: reads checked setup/demo, builds steps, supplies program/environment/listeners/data root to the box.
  - `src/box_command.rs:125` — `open_listeners`: holds listeners for **always and dormant groups**; reuses inherited/init descriptors; checks duplicates.
  - `src/box_command.rs:187` — `owned_notices`: supplies public startup notices.

- **Step construction / box ownership:**
  - **`temper-forge-setup`**, `src/step.rs:27` — `Group`; `:38` — `Steps`; `:144` — `steps`: owned fetch/prepare/Forgejo/runner groups, managed reconciles, final `internal serve --box`.
  - **`temper-fake`**, `src/steps.rs:21` — `steps`: prepends ordinary fake processes and adds dormant playground group containing fake LLM, prepare and fake forge; playground group is not recorded.
  - **`temper-box`**, `temper/crates/temper-box/src/host.rs:32` — `Settings`; `:139` — `Service`; `:175` — `Service::new`: receives already-built opaque steps and held descriptors.
  - `temper/crates/temper-box/src/records.rs:28` — `Records`; `:38` — `new`; `:191` — `path`: durable owned-group markers are `box/started/N`.
  - `temper/crates/temper-box/src/pipe.rs:46` — `Command::{Start, Stop, Unknown}`; `:52` — `command`.
  - **No `remove N`, persisted `box/groups/N` settings, or construction of a new group from a runtime forge record exists.**

**3. Readers and reverse dependencies**

- **Across trees:** no skein, smith or jig crate reads temper’s document/`settings::Forge`/domain `Configured`. They receive translated generic settings, opaque application records or worker payloads. In particular, `skein-forge-connection::Settings::forges` is a separate type, not a reader of `temper.json`.

- **`temper` production readers**, base `temper/crates/temper`:
  - `src/lib.rs:342` — `forges`: resolves each configured API and prepares transport; `:378` — `prepare_with_limits`: reads startup/settings/setup files, creates runtime endpoints and calls service startup.
  - `src/setup.rs:17` — `check_with`: converts `Named.forges`/identities/listeners into setup `Members`, validates setup and demo.
  - `src/forge.rs:153` — `start`; `:312` — `reconcile_settings`: select forge/setup by number, construct setup-shell inputs.
  - `src/forge_sign_in.rs:13` — `run`: joins sign-in forge number to configured Forgejo/setup.
  - `src/fake.rs:335` — `forge_document`: deployment setup → fake forge document; `:371` — `llm_configuration`: deployment demo → fake LLM.
  - `src/box_command.rs:47` — `run_configuration`, described above.
  - `src/fake_backends.rs` — fake-backend command wiring consumes the rewritten deployment through ordinary setup/startup.

- **`temper-service` production readers**, base `temper/crates/temper-service`:
  - `src/startup.rs:683` — `named_with`; `:778` — `checked_start_with`: consume parsed forges, files, identities, setup, limits and endpoint runtimes.
  - `src/startup/application.rs:709` — `github_listener`; `:1080` — `missing_asked_files`; other parser/resolution readers are listed in question 1.
  - `src/startup/application/landing.rs:175` — `parse`; `:409` — `project_landing`: resolve repository-check git hosts from forges.
  - `src/startup/application/sign_in/forges.rs:16` — `parse`; `:77` — `check`: sign-in forge/client joins and callback URL checks.
  - `src/startup/checks.rs:230` — `webhooks`; `:325` — `longest_remote`; `:387` — `channel`; `:655` — `root`; `:1017` — `record_writes`; `:1061` — `records`.
  - `src/system.rs:165` — `Service::prepared`: builds forge component/setup host, root `Config`, demo and workers; `:960` — `forget_handover`; `:1097` — `note_bound`.
  - `src/forge.rs:42` — `settings`: joins configuration/runtime endpoints; `:114` — `file_access`; `:218` — `Layer::new`; `:409` — `indexed`; `:436` — `prepare`; `:454` — `config`.
  - `src/engine.rs:116` — `Engine::new`: creates credential loader and connection limits; `:310` — `iterate`; `:426` — `clients_pass`: passes retained forges to client translation.
  - `src/workers.rs:554` — `Workers::configured`: takes ownership of settings’ forge array; `:147` — `encoded_assignment`.
  - `src/assignment.rs:74` — `down`; `src/translate.rs:25` — `items`; `:41` — `repository_item`: derive remotes, git program and GitHub installation account.
  - `src/translate.rs:181` — `github_identity`; `src/forge/tokens.rs:36` — `Tokens::new`; `:124` — `announce`: forge-number → identity/installation token routing.
  - `src/forge/records.rs:17` — `Rows::new`; `:67` — `Loader::new`; `:130` — `advance`: load credentials, retain configured forge values, construct access and connect each forge.
  - `src/setup/settings.rs:8` — `fixed`; `:104` — `forgejo`; `:163` — `prepare`; `:253` — `local_starts`; `:329` — `source_forge`: setup host configuration and local reconcile state.
  - `src/setup.rs:82` — `webhooks_listening`; `:97` — `Layer::new`: host forge tables.
  - `src/setup/lifecycle.rs:544` — `configured`; `:707` — `advance`; `:824` — `restored`; `:1208` — `take_read`; `:1266` — `take_removal`: runtime lookups in retained `workers.forges`.
  - `src/setup/routes.rs:251` — `route`; `:402` — `manifest_visit`: validate/route forge-specific HTTP setup requests.
  - `src/clients.rs:450` — `advance`; `:540` — `iterate`; `:674` — `endings_step`: pass forge settings to application request translation.
  - `src/family.rs:27` — `to_root`: configured forge lookup for adoption; `:305` — `panel`; `:443` — `address`; `:446` — `pull_address`; `:951` — `document`; `:1029` — `landing_member`: forge URLs.
  - `src/route.rs:9` — `request`: routes setup, forge calls and client deliveries.
  - `src/clients/bundle.rs:339` — `forge_metas`; `:385` — `fill_forge_sign_in`: browser Forgejo sign-in metadata.
  - `src/startup/application/sign_in.rs:383` — `resolve`; `src/clients/google.rs:222` — `new`: retain sign-in joins.

- **Other production/domain readers:**
  - **`temper-fake`**, `temper/crates/temper-fake/src/backends.rs:350` — `rewrite`; `:448` — `managed`; `:496` — `forge`: rewrite configured forge endpoints and append the managed fake-backend forge.
  - **`temper-engine-domain`**, `temper/crates/temper-engine-domain/src/translate.rs:34` — `Config::forges`; `src/domain.rs:197` — `Domain::new`; `:1140` — `refused`: construct/check connector from configuration.
  - Same crate, `src/route/forge_route.rs:135` — `landing_words`; `:164` — `landing_pull`; lookup near `:2647`: landing result URLs use `Configured`.
  - **`temper-engine-domain-forge`**, `src/domain.rs:208` — `configured`; `:238` — `forges`; `:306` — `offer`; `:492` — `Domain::new`; `:1058`/`:1072` — connection/App event handling; `:1178` — SetUp admission.
  - Same crate, `src/connect.rs:35` — `start`; `:151` — `admin`; `:363` — reachable admission: configured kind/mode/ownership checks.
  - Same crate, `src/demo.rs:137` — `connected`: another configured forge ends demo; `src/landing.rs:113` — `project_landing`: configured kind checks; `src/brief.rs:156` — forge metadata lookup; `src/reconcile.rs:292` — configured forge/identity link resolution.
  - **`temper-engine-domain-forge-change`, `-issues`, `-landing`** do not read the document or `Configured`; the connector top supplies their domain inputs.

- **Credentials keyed by forge number:**
  - Design: `temper/docs/design/protocol/credentials.md:101`, §2.1 — `family << 48 | forge << 16 | item`; configured forge number is the stable key.
  - **`temper-protocol-records`**, `temper/crates/temper-protocol-records/src/lib.rs:18` — `FAMILY_FORGE = 1`; `:20` — `CARRIED = 1 << 63`; `:23` — `Item`; `:53` — `number`; `:57` — `parse`.
  - Items 1–10: `AdminToken`, `ApiToken`, `GitToken`, `User`, `WebhookSecret`, `AppId`, `AppSlug`, `AppKey`, `Accepted`, `Origin`.
  - Same crate, `src/forge.rs:53` — `Rows`; `:57` — `ForgeRows`; `:69` — `Rows::push`; `:176` — `access`; `:29` — `origin`: groups by forge and validates kind/API origin.
  - **`temper-service`**, `src/setup/lifecycle.rs:267` — `keep`: computes `records::number(forge, item)`; `src/setup/public.rs:370` — `erase`; `src/forge/records.rs:24` — `Rows::push`: configured/unconfigured credential handling.
  - Same crate, `src/setup/lifecycle.rs:552` — `connect`; `:589` — `connected`; `src/forge/tokens.rs:67` — `token`, `:86` — `removed`, `:92` — `accepted`, `:107` — `connected`: access and worker-identity updates.
  - No main-store forge-configuration record is hidden in these credential rows; `Origin` is only kind/API binding, not full forge metadata.

- **Forges page/data path:**
  - **`temper-client-protocol-wire`**, `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema:84` — `Watch::forges`.
  - **`temper-engine-domain`**, `src/route/panels.rs:7` — `document`: deployment application document with subject 0 calls `domain.forge.forges()`.
  - **`temper-engine-domain-forge`**, `src/domain.rs:238` — `forges`: iterates `configured`, skips ended forge, joins connection, first intent, installations and adopted repositories.
  - **`temper-service`**, `src/route.rs:128` — Forges delivery; `src/family.rs:711` — `forges`: domain rows → wire.
  - **`temper-client-protocol-wire`**, schema `:69` — `ForgeRow`; `:70` — `Forges`; `src/lib.rs:45` — `CLIENT_VERSION = 11`; `:48` — `FORGES = 16`; `:282` — `forges`; `:290` — `decode_forges`, enforcing strictly increasing numbers.
  - **`temper-client-domain`**, `src/forge/forges.rs:169` — `watch`; `:204` — `rows`; `:319` — `configure`; `:477` — `told`; `src/forge/family.rs:239` — `forges`: decode into client rows.
  - Same crate, `src/domain.rs:94` — `Domain::forges`; `src/forge.rs:142` — `Side::forges`; `src/forge/listed.rs:39` — `rows`; `src/forge/connect.rs:121` — `choices`; `:143` — `personal`; `src/pages/welcome.rs:22` — `decision`.
  - Same crate, `src/forge/demo.rs:182` — `halves`; `src/forge/picker.rs:123` — `progress`; `src/forge/admin.rs:121` — `send`: consume derived rows.
  - **`temper-client-view`**, `src/forge/forges.rs:20` — `main`; `src/forge/connect.rs:71` — `main`; `src/pages/welcome.rs:120` — `main`; `src/frame.rs:437` — `setup_underway`, `:452` — `forges_entry`.
  - **`temper-client-tools`**, `src/forges.rs` — `Forges`, `print_rows`; `src/root.rs:25` — retained forges tool state; `src/forges/connection.rs:171` — `send`; repository picker code consumes rows.
  - **`temper-client-shell-web`**, `src/journey.rs:25` — `Facts::forges`; `src/journey/record.rs:17` — `Record::new`: recorded/replayed forge documents.
  - **`temper-client-gallery`**, `temper/tools/gallery/src/samples.rs:308` — `mockup`: forge-page journey links.
  - **`temper-protocol-records` has no Forges-page `ForgeRow`/`Forges` wire type.** Its forge types above are credential records/access helpers.

- **Tests/worlds/fixtures — counts and main builders:** direct-reference/template inventory contains **125 Rust files and two JSON fixture documents**, counting files rather than individual tests; this includes typed configs and derived client-row tests. Main JSON builders are service `fixtures::startup::{document,demo_document}`, CLI `support::configuration::{documents,check_document,demo_document}`, owned/example helpers, real-loop `documents`, one-process `world_document`, browser `documents`, and first-repository/template helpers. Typed domain worlds additionally build `Config.forges` directly.
- **`temper-service` test inventory**, base `temper/crates/temper-service/src`:
  - `tests/fixtures.rs:62` — `engine`; `:125` — `forges`; `tests/fixtures/startup.rs:9` — `APPLICATION`; `:27` — `demo_application`; `:35` — `demo_document`; `:61` — `document`; `:196` — `with`; `:295` — `owned_worker_alone_document`.
  - `tests/fixtures/worker.rs:71` — `assigned`; `tests/fixtures/forge.rs:232` — `hook`; `tests/fixtures/http.rs:95` — `pump_until`.
  - `tests/startup.rs:445` — `the_standalone_example_reads_into_its_members`; `:503` — `with_forge_git_program`; `:1850` — `forge_configuration_retains_only_git_bases_within_the_pull_address_bound`; `:1865` — `forge_configuration_keeps_kind_mode_and_asked_start`.
  - `tests/checks.rs:77` — `default_engine_and_worker_grants_fit_the_channel_and_link`; `tests/root_checks.rs:217` — `forges_past_the_clients_op_bytes_refuse_the_start`.
  - `tests/limits.rs:247` — `records_and_installations_override_and_check_both_bounds`; `tests/family.rs:13` — `configured`; `:1029` — `forges_encode_the_domains_rows_without_settings`.
  - `tests/forge.rs:417` — `a_refusal_naming_a_forge_is_indexed_to_the_configured_forge`; `tests/git_tokens.rs:89` — `configured_login_and_author_override_the_app_defaults`.
  - `tests/setup.rs:51` — `github_config`; `:438` — `tls_forges`; `tests/setup_adoption.rs:7` — `old_form_configured`; `:38` — `configured`.
  - `tests/setup_create.rs:302` — `reconcile_then_create_then_connect_makes_the_first_repository_and_bot_workflow`; `tests/setup_admin.rs:180` — `admin_paste_reconciles_creates_with_its_label_then_keeps_and_restarts_from_records`.
  - `tests/setup_lifecycle.rs:31` — `app`; `tests/setup_github.rs:60` — `pump`; `tests/setup_github_render.rs:104` — `status`.
  - `tests/setup_reachable.rs:246` — `sent`; `tests/setup_scopes.rs:72` — `preparations_four_scopes_grant_adopt_and_without_repository_it_is_refused_at_repository`.
  - `tests/sign_in_configuration.rs:159` — `with_forges`; `tests/sign_in.rs:18` — `configured`.
  - `tests/translate.rs:8` — `a_workspace_reaches_items_with_each_remote_made_from_its_base`; `tests/worker_parts.rs:164` — `an_engine_s_items_reach_the_worker_s_workspace_field_for_field`.
  - `tests/workers.rs:123` — `an_assignment_carries_its_items_encoded_charter_and_identity_value`; `tests/clients.rs:248` — `a_request_before_decisions_open_is_answered_not_ready`.
  - `tests/system.rs:433` — `a_new_webhook_after_idle_gets_its_whole_delivery_period`; `tests/welcome_policy.rs:17` — `generated`; `clients/demo_tests.rs:23` — `start_demo_reads_each_time_and_carries_the_key_before_admission`.
- **`temper` test inventory**, base `temper/crates/temper`:
  - `src/tests.rs:308` — `forge_commands_refuse_managed_and_fake_entries_by_name`; `src/init/tests.rs:94` — `init_finds_ports_and_programs_and_writes_exact_modes_in_order`.
  - `src/forge/tests.rs:37` — `reconcile_refuses_records_only_owned_and_fake_without_writing_files`; `src/fake_backends/tests.rs:748` — `four_forges_pass_ordinary_admission_with_a_directoryless_managed_entry`.
  - `tests/support/configuration.rs:6` — `documents`; `:117` — `old_form_check_document`; `:132` — `check_document`; `:146` — `demo_document`.
  - `tests/support/owned.rs:21` — `document`; `:43` — `old_form`; `:560` — `EXAMPLE`; `:638` — `example_document`; `:784` — `bounded`; `:801` — `bounded_records`; `:823` — `bounded_template`; `:876` — `deployment_profile`; `:1044` — `checked_setup`.
  - `tests/support/first_repository.rs:454` — `playground_read_evidence`; `:542` — `live_for`; `:705` — `generated`.
  - `tests/support/github_land.rs:801` — `configure_engine`; `tests/support/live_github.rs:152` — `configure_engine`; `tests/support/github/existing.rs:143` — `Existing::act`.
  - `tests/support/case.rs:202` — `setup`; `tests/support/forgejo_elsewhere.rs:137` — `configure`; `tests/support/webhooks.rs:26` — `configuration`; `tests/support/forge_sign_in.rs:11` — `configure`.
  - `tests/support/account_file_qa.rs:22`, `account_move_qa.rs:44`, `account_move.rs:31` — each `configure`.
  - `tests/support/csv_repair.rs:103` — `at`; `tests/support/landing_by_turn.rs:29` — `new`; `tests/support/github.rs:226` — `row`.
  - `tests/support/forge_land.rs:646` — `assert_forgejo_setup_after_sign_in`; `:747` — `live_deployment`; `:947` — `forge_identity`; `tests/support/forge_land/rung.rs:821` — `setup_tools`.
  - `tests/fixtures/cluster/engine.json:187`, `tests/support/github/engine.json:176` — static `application.forges`.
  - `tests/process.rs:785` — `an_adopt_cut_by_a_signal_is_made_once`; `:2399` — `assert_playground_end_forges`; `:3830` — `a_default_box_with_a_sign_in_registration_starts_without_dns`; `:4553` — `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge`.
  - `tests/suite/startup.rs:333` — `managed`; `:359` — `a_setup_naming_another_api_token_file_refuses_temper_serve_naming_its_path`; `:1735` — shipped-document matrix.
  - `tests/suite/forge.rs:321` — `reconcile_refuses_a_tls_forge_naming_it`; `tests/suite/fake.rs:446` — `forge_setup`; `tests/suite/configuration.rs:137` — `the_clusters_documents_check_as_their_shapes`.
  - `tests/suite/fake_opening.rs:190` — `demo_chat_fixtures_select_their_opening_requests`; `tests/fake_backends/qa.rs:237` — split-directory configuration case; `tests/fake_backends/walk.rs:115` — `forge_number`.
  - `tests/qa_all/qa_adoption.rs:9` — init document case; `qa_first_run.rs:18` — `forge_address`; `qa_first_repository.rs:43` — restart case; `qa_forge_handover.rs:35` — `configured`; `qa_forge_setup.rs:41` — init forge-three case.
- **Other crate-local tests**, paths rooted at `temper/crates/CRATE`:
  - **`temper-init`**: `src/tests/fixtures.rs:5` — `OWNED`; `src/tests/documents.rs:58` — `the_default_is_the_owned_example_so_edited`.
  - **`temper-forge-setup`**: `src/tests/fixtures.rs:45` — `members`; `src/tests/managed.rs:11` — `members`; `src/tests/members.rs:16` — API-file disagreement case.
  - **`temper-fake`**: `src/backends/tests.rs:119` — rewrite case; `src/tests/demo.rs:12` — `fixture`; `src/tests/steps.rs:23` — whole ordered steps case; `src/oauth/tests.rs:102` — shipped-template registration case.
  - **`temper-engine-domain`**: `src/tests/resume_fixture.rs:259` — `config_with_project_room`; `src/tests/setup.rs:17` — `signed_in_on`, `:623` — `admin_root`; `src/route/forge_route/tests.rs:728` — `landing_forge`.
  - **`temper-engine-domain-forge`**: `src/tests.rs:1223` — `configured`, `:1251` — `setup_domain`; `src/tests/connect.rs:10` — `github`; `src/tests/demo.rs:139` — `configured`.
  - **`temper-client-domain`**: `src/tests/forges.rs:21` — setup/watch case; `src/tests/connect.rs:38` — flow case; `src/tests/admin.rs:50` — `deliver`; `src/tests/github_app.rs:238` — `github_row`; `src/tests/demo.rs:163` — model-half case; `src/tests/pages/welcome.rs:311` — invalid forges document case.
  - **`temper-client-view`**: `src/tests/fixtures.rs:296` — `root_at_with_forges`; `src/tests/connect_forge.rs:229` — next-number case; `src/tests/forges.rs:308` — 64-installation case.
  - **`temper-client-tools`**: `src/tests/forges.rs:69` — `deliver`; `src/tests/forge_connection.rs:417` — `first_page`; `src/tests/printed.rs:14` — `every_printed_call_parses`.
  - **`temper-client-protocol-wire`**: `src/tests.rs:333` — `forges_rows`.
  - **`temper-client-shell-web`**: `src/host_tests.rs:14` — `fixture_forges`; `src/journey/tests.rs:133` — `facts`; `tests/suite/host/walk.rs:26` — `new`.
- **World/test-kit crates**, each base is `temper/tests/PATH`:
  - **`temper-one-process-world`**, `one-process`: `src/fixtures.rs:110` — `world_document`; `src/stories.rs:264` — `configure`; `src/lower/github.rs:36` — `configure`; `src/restart_control.rs:37` — `checked_setup`.
  - **`temper-real-loop-world`**, `real-loop`: `src/fixtures.rs:16` — `documents`; `src/forge.rs:45` — `write`; `src/github.rs:148` — `write`; `tests/suite/github.rs:23` — seeded-App restart case.
  - **`temper-client-browser-world`**, `client-browser`: `src/configuration.rs:25` — `documents`, `:154` — `demo`; `src/lib.rs:197` — `github`.
  - **`temper-system-world`**, `system`: `src/composed/link.rs:29` — `assignment`.
  - **`temper-engine-protocol-forge-world`**, `engine/protocol-forge`: `src/fixtures.rs:125` — `writer_settings`, `:172` — `github_settings`.
  - **`temper-engine-domain-world`**, `engine/domain`: `src/walking.rs:269` — `config`; `src/forge_routes.rs:1918` — `box_profile`; `tests/fixtures/stock.rs:2` — `box_model`; `tests/suite/demo.rs:12` — `config`; `tests/qa_all/qa_first_repository.rs:10` — `driver_with_name_bytes`, `:199` — **`first_repository_row_is_volatile_across_restart`**.
  - **`temper-client-world`**, `client`: `src/fixtures.rs:21` — `forge_settings`; `src/front.rs:153` — `github_fixture_with`; `src/front/account_acts.rs:125` — `fixture_root`; `src/translate/forge.rs:625` — `delivered_forges`.
  - Same crate: `tests/suite/forges.rs:27` — `config`; `tests/suite/connect_forge.rs:111` — owned setup case; `tests/suite/welcome.rs:151` — unstarted playground retirement case.
  - Same crate: `tests/qa_all/qa_forges.rs:55` — `configured`; `qa_first_repository.rs:17` — `configured`; `qa_welcome_m3_owned.rs:14` — `owned_fixture`; `qa_github_connect.rs:76` — abandoned-install case.
  - **`temper-codecs-world`**, `codecs`: `tests/suite/client_wire.rs:70` — setup/first-repository round trip. Credential-number readers additionally live in `tests/suite/records.rs:36` — `record_numbers_and_origin_addresses_keep_their_meaning`, and `tests/suite/fuzzy/records.rs:20` — generated record-number input.
  - No additional document-forge builder was found under `temper/testing/`; hosted git and GitHub-App kits receive their own typed inputs.

**4. Runtime state, persistence and adding a family**

- **`temper-engine-domain-forge`**, base `temper/crates/temper-engine-domain-forge`:
  - `src/boundary.rs:298` — `Configured`: public startup metadata, no credentials.
  - `src/domain.rs:119` — `Domain`: `configured: Map<u16, Configured>`; adjacent maps include connections, first intents, ended diagnostics, Apps, installations, repositories, writers and git identities.
  - `src/domain.rs:492` — `Domain::new`: copies supplied configured rows into the map; capacities derive from client repository bounds.
  - `src/domain.rs:197` — `connection_limits`: resizes volatile connection/first/App/installation tables from component limits.
  - `src/domain.rs:208` — `configured` explicitly describes startup configuration “never restored from records.”
  - `src/boundary.rs:101` — `Repository`; `src/domain.rs:137` — repository map; `src/adoption.rs:496` — adopted repository insertion/persistence.
  - **A forge record would naturally be another connector-owned `Stored`/`Key` family beside `Repository`.** That is an implementation fit, not an existing type.
  - **No durable `Configured`, first-repository intent, next-forge counter, or runtime forge-number allocator exists.**

- **Actual main-store families:** `temper-protocol-store`, `temper/crates/temper-protocol-store/src/keys.rs:80` — `encode`; `:190` — `decode`; domain `Stored` at `temper-engine-domain-forge/src/boundary.rs:802`, `Key` at `:875`.
  - Tags **0–7**: `Call`, `ProposedEffect`, `Repository`, `Hold`, `Names`, `Subscription`, `BranchHead`, `PullState`.
  - Tags **8–15**: `Ci`, `Landed`, `Entry`, `Client`, `Change`, `Issue`, `Release`, `Application`.
  - Tags **16–19**: `Writer`, `Demo`, `Binding`, `DemoAct`.
  - Tag **21**: `Flight`; tag **23**: `Landing`; tag **24**: `Receipt`.
  - Thus **23 implemented connector families**. Tags **20 and 22 are not decoded/encoded here**; no `Setting` or top-level `Delivery` variant exists in this codec. Do not confuse nested client delivery values with a family.
  - `temper/docs/design/protocol/store.md:60`, §2, is stale: it lists Setting/Delivery and calls Landing/Receipt unbuilt; `temper/docs/design/protocol/store.status.md:6` records built Landing/Receipt.
  - Secrets are separate: `temper-protocol-records::FAMILY_FORGE = 1`, items 1–10 above; main-store application tag is jig’s tag **2**, not credential family 1.

- **Adding a family — concrete extension points:**
  - Domain kind: `temper-engine-domain-forge/src/boundary.rs:802` — `Stored`; `:875` — `Key`; `src/reconcile.rs:8` — `RecordKind`.
  - Key/byte accounting: same crate, `src/lib.rs:222` — `stored_key`; `:258` — `stored_bytes`.
  - Codec: `temper-protocol-store/schema/store-v10.schema:244` — `Value`; `:245` — versioned `Row`; regenerate types; `src/keys.rs:80`/`:190` — key encode/decode.
  - Value translation: `temper-protocol-store/src/records.rs:22` — `encode`; `:93` — `decode`; `:160` — `row`, checking key/value agreement; `:169` — bounds.
  - Version/sizing: `temper-protocol-store/src/lib.rs:15` — `STORE_VERSION = 10`; `:19` — `application`; `:64` — `bounds`.
  - Engine application/restore: `temper-engine-domain-forge/src/domain.rs:4785` — `restore`; `src/restore.rs:138` — `row` fit checks; `src/reconcile.rs:136` — restoration references; domain restored-link dispatch near `src/domain.rs:5303`.
  - Root/store wiring: `temper-engine-domain/src/restart.rs:514` — `Record::Forge` → forge Restore event; `temper-service/src/store/route.rs:95` — `down`, `:398` — `decode`; `src/store/ranges.rs:81` — whole application interval restoration.
  - Add key ordering, codec/key-agreement, fit and restart tests; maintain design/status.

- **Recent example — `Flight`:**
  - `temper-protocol-store/src/keys.rs:12` — `FLIGHT = 21`; schema `store-v10.schema:218` — `Flight`.
  - `temper-protocol-store/src/records.rs:45`/`:105` — encode/decode; `temper-engine-domain-forge/src/domain.rs:2274` — saves before sent attempt; `:4861` — restores; `:4148` — `erase_flight`.
  - `temper-protocol-store/src/tests/keys.rs:183` — `flight_keys_name_task_then_entry_under_family_twenty_one`; `src/tests/records.rs:907` — `flight_rows_keep_both_key_fields_and_refuse_a_disagreeing_key`.

**5. SetUp and Connect today**

- **Wire — `temper-client-protocol-wire`:**
  - `schema/client-wire-v1.schema:16` — `SetUp { forge: u16, org, repository, project: ExistingProject|NewProject, playground: bool }`.
  - `schema/client-wire-v1.schema:21` — `FirstAsk { org, repository, project, label }`.
  - `schema/client-wire-v1.schema:23` — `How::{github_app { id }, github_manifest { back? }, forgejo_admin { first? }}`.
  - `schema/client-wire-v1.schema:24` — `Connect { forge: u16, how }`; `:28` — request variants.
  - Both select an **already configured forge number**. Neither carries kind, addresses, bot git identity or setup entry.

- **Domain — `temper-engine-domain-forge`:**
  - `src/boundary.rs:152` — `Ask::{SetUp, Connect}`; `:163` — `FirstAsk`; `:171` — `How`; `:323` — `SetUpProject`.
  - `src/domain.rs:1178` — SetUp validates names/project/playground shape, checks `offer`, limits concurrent setup, stores `First` **in memory**, emits `Request::SetUp`.
  - `src/domain.rs:306` — `offer`: only configured **owned + asked** forge; requires box for an actionable offer.
  - `src/connect.rs:35` — `start`: owner, configured forge, kind/mode, busy/connection and carried-key checks.
  - `src/connect.rs:151` — `admin`: configured managed Forgejo with `admin_connect`; takes carried admin token, optionally retains volatile first-repository intent.
  - `src/connect.rs:120` — `send`: emits App-connect, manifest, install or accept requests; it does not create a forge.

- **Root/service execution:**
  - **`temper-engine-domain`**, `src/route/applications.rs:19` — `start`: forces these requests to deployment scope; `:53` — `admitted`: forwards core admission to connector.
  - Same crate, `src/route/forge_route.rs:2969` — Connect forwarding; `:2999` — SetUp forwarding.
  - **`temper-service`**, `src/family.rs:27` — `to_root`: wire → domain, preserving carried credential handle.
  - Same crate, `src/route.rs:32` — SetUp → setup lifecycle; `:38` — Connect → admin/App/manifest host path.
  - Same crate, `src/setup/lifecycle.rs:858` — `start_local`; `:899` — `reconcile_local`; `:1000` — `create_local`; `:1088` — `adopt_local`; `:1104` — `connect_local`: box start, handover/reconcile, first repository, adoption, connection.
  - Same crate, `src/setup/public.rs:46` — `admin_connect`; `:117` — App `connect`; `:157` — `manifest`: host plans write credential records as they progress.

- **Senders:**
  - **`temper-client-domain`**, `src/forge/family.rs:182` — `set_up`; `src/forge/forges.rs:179` — `ask`, `:351` — `set_up`, `:381` — `set_up_at_once`, `:414` — `send_setup`: Forges/welcome setup flows.
  - Same crate, `src/forge/admin.rs:121` — `send`: Forgejo-admin Connect; `src/forge/github.rs:96` — `send`: pasted App Connect; `:147` — `manifest`: manifest Connect.
  - **`temper-client-tools`**, `src/forges/connection.rs:25` — connect request construction; `:171` — `send`: `temper ctl` Connect paths.
  - Tests/worlds listed above also send both acts directly.
  - **No `RemoveForge`, `StartPlayground`, `EndPlayground`, or jig `RemoveProject` act exists in current Rust/schema code.**