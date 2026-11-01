Read-only survey; no builds, Cargo, tests or writes. Earlier-work cutoff: **2026-10-31 13:30Z**. Reader inventory covers S5’s first-repository intent, setup persistence and welcome surfaces.

**1. In-memory setup, incremental writes, GitHub, Default**

- Contract: `temper/docs/design/protocol/setup.md:869` §6.7; incremental credential writes at `:290`, `:399`; first-repository loss at `:949`, `:959`.
- `temper-engine-domain-forge` — `temper/crates/temper-engine-domain-forge/src/boundary.rs:154` `Ask::SetUp`; `:163` `FirstAsk`; `:172` `How::ForgejoAdmin`; `:323` `SetUpProject`; `:335` `FirstState`; `:360` `First`; `:387` `Row.first`; `:1076` `Event::FirstMade`; `:1080` `Event::FirstFailed`.
- `temper-engine-domain-forge` — `temper/crates/temper-engine-domain-forge/src/domain.rs:129` `Domain.first: Map<u16, First>` is the domain’s volatile intent. It contains organization, repository, project, runner label, playground flag and creation state.
- Same file — `Domain::connection_limits` `:197` creates an empty map; `Domain::new` initializes it at `:538`; `step`, `Ask::SetUp` arm `:1178`, replaces the admitted forge’s intent at `:1226`.
- Same file — `Domain::first_made` `:222`, `Domain::first_failed` `:230`, `Domain::forges` `:238` read/update/project it; `step` handles their tells at `:1101`.
- `temper-engine-domain-forge` — `temper/crates/temper-engine-domain-forge/src/connect.rs:213` `start` clears/replaces the managed-admin connection’s first intent.
- **No durable first-repository intent record exists.** A restart loses even `FirstState::Made { id }`, its project/name, creation failure and retry fields. The durable repository adoption, if already completed, survives separately.
- Explicit regression contract: `temper-engine-domain-world` — `temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:199` `first_repository_row_is_volatile_across_restart`; asserts `first.is_none()` at `:232` and no replayed `Request::SetUp` at `:234`.
- `temper-service` — `temper/crates/temper-service/src/setup/lifecycle.rs:53` `Intent` is the second volatile copy: organization, repository, label, playground flag. `Local.first` `:67` and `Run.first` `:35` hold it; project remains in the forge domain/client.
- Service admission/sequence readers: `temper/crates/temper-service/src/route.rs:32` `Request::SetUp` → `lifecycle::first`; `setup/lifecycle.rs:1330` `first`; `setup/settings.rs:278` `Local { first: None }`; `setup/public.rs:46` `admin_connect`.
- Service execution readers: `setup/lifecycle.rs:1000` `create_local`; `:1042` `create_stopped`; `:1416` `admin_create`; `:1463` `admin_done`; `:308` `done` emits `FirstMade`, then proceeds to connection/start-time access provisioning.
- Root routing readers: `temper-engine-domain` — `temper/crates/temper-engine-domain/src/route/forge_route.rs:2999` `forge::Request::SetUp`; `src/admission.rs:95` `FirstMade`/`FirstFailed`; `temper-service/src/route.rs:105` exhaustively routes those events.

What is written as setup proceeds:

- `temper-service` — `temper/crates/temper-service/src/setup.rs:450` `advance`: host `Need::Write` becomes `lifecycle::keep` at `:491`.
- `temper-service` — `temper/crates/temper-service/src/setup/lifecycle.rs:267` `keep`: encodes an individual forge credential record and sends `Secrets::Keep { owner: Application, … }`.
- `temper-service` — `temper/crates/temper-service/src/setup.rs:512` `answer`: only `StoreEvent::Kept` updates loaded rows and advances the lifecycle; `setup/lifecycle.rs:292` `Purpose` distinguishes host, origin and admin writes.
- `temper-protocol-records` — `temper/crates/temper-protocol-records/src/lib.rs:23` `Item`: `AdminToken`, `ApiToken`, `GitToken`, `User`, `WebhookSecret`, `AppId`, `AppSlug`, `AppKey`, `Accepted`, `Origin`; `number` `:53` keys them by forge and item.
- Reconcile writes bot credentials/id/webhook secret individually; local `Origin` follows successful reconcile: `temper-service/src/setup/lifecycle.rs:329` `done`.
- Managed-admin order: reconcile → durable `Origin` → optional Create → durable admin token → connect. Anchors: `setup/lifecycle.rs:1463` `admin_done`, `:1416` `admin_create`, `:1402` `keep_admin`. A failed Create does not retain the pasted admin token.
- **Project/adoption are separate client acts after connection**, not an atomic server-side setup transaction:
  - `temper-client-domain` — `temper/crates/temper-client-domain/src/forge/forges.rs:618` `maybe_adopt` waits for both `Connection::Connected` and `FirstState::Made`.
  - Same file `:671` `setup_start`, `:684` `start_in`, `:562` `project_told`, `:743` `send_adoption`.
  - `temper/crates/temper-client-domain/src/forge/start.rs:6` `StartIn`; `StartIn::ask` `:17` sends `MakeProject { name, policy: 1 }`; `answered` `:40` permits adoption only after `ProjectMade`.
  - `temper-client-tools` — `temper/crates/temper-client-tools/src/forges.rs:1086` `intent`, `:1131` `watch_first`, `:1171` `setup_changed`, `:1271` `send_adoption` implement the ctl equivalent.
  - `jig-domain-core` — `jig/crates/jig-domain-core/src/routing/projects.rs:36` `make_project_request` writes policy/name, establishes ownership and opens its period.
  - `temper-engine-domain-forge` — `temper/crates/temper-engine-domain-forge/src/adoption.rs:459` `finish_adoption` writes `Stored::Repository` at `:502`.
- Accounts persist independently of the forge setup:
  - `jig-domain-accounts` — `jig/crates/jig-domain-accounts/src/keys.rs:123` `granted`, saves account/source at `:146`, `:156`.
  - Same crate — `src/signing.rs:244` `make`, `:312` `join`; `src/programs.rs:29` `declare`, saves at `:88`, `:99`, `:102`.
  - `jig-domain-core` — `jig/crates/jig-domain-core/src/routing/children.rs:530` routes `accounts::Request::Save` into `Record::Accounts`.
- Thus a cut setup can leave persisted credentials, an already-made remote repository, an already-created project, an adoption, or accounts. There is no whole-setup rollback.

GitHub:

- Contract steps: `temper/docs/design/protocol/setup.md:3396` §17.2:
  1. Manifest/state.
  2. Temper’s manifest form.
  3. Person creates App on GitHub.
  4. Redirect converts code, writes identity, obtains installation link and connects.
  5. Installation return validates and persists acceptance.
- `skein-forge-setup` — `skein/crates/skein-forge-setup/src/github/manifest.rs:32` `Manifest`, `step` `:114`; `convert.rs:34` `Convert`, `step` `:189`; `install.rs:28` `Install`, `step` `:40`; `accept.rs:27` `Accept`, `step` `:152`; pasted alternative `paste.rs:42` `Paste`, `step` `:205`.
- `skein-forge-setup` — `skein/crates/skein-forge-setup/src/github/write.rs:20` `Write`; `advance` `:31` writes, one acknowledgment at a time, **AppId → AppSlug → AppKey → User** (`:45`). The stored key is DER; conversion PEM/webhook/OAuth extras are not kept.
- `skein-forge-setup` — `github/accept.rs:69` writes `Accepted`; `github/installations.rs:150` also rewrites it during installation reconciliation.
- `temper-service` — `temper/crates/temper-service/src/setup/public.rs:157` `manifest`, `:200` `install`, `:219` `accept`, `:271` `ended`.
- Browser route readers — `temper/crates/temper-service/src/setup/routes.rs:251` `route`, `:334` `accept`, `:402` `manifest_visit`.
- Browser states/pages/back destinations are volatile: `temper-service/src/setup.rs:264` `Layer::page`, `:287` `hold`; `setup/public.rs:384` `hold`, `:390` `take_back`. Contract: `setup.md:3441`; restart forgets states, not kept App records.
- Credential reconstruction — `temper-protocol-records/src/forge.rs:165` `access`; App credentials/slug/accepted set read at `:227`.
- Important existing tests, `temper-service/src/tests/setup_lifecycle.rs`:
  - `:77` `conversion_keeps_origin_then_installs_answers_the_browser_and_connects_from_records`.
  - `:107` `an_install_failure_answers_the_conversion_but_the_kept_app_still_connects`.
  - `:150` `manifest_and_convert_refuse_even_partial_app_records_and_preserve_the_accepted_set`.
  - `:1611` `qa_a_link_forgotten_by_restart_ends_with_the_fallback_back`.
  - `:1621` `qa_restart_before_acceptance_restores_connecting_then_install_accept_and_reachable_work`.
- **No complete interrupted-owned-setup recovery walk exists.** GitHub’s narrower restart/install tests do exist; do not describe all restart coverage as absent.

Remaining first-repository API readers, including tests/kits:

- `temper-client-protocol-wire` — schema `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema:15` `SetUpProject`, `:16` `SetUp`, `:21` `FirstAsk`, `:56` `FirstState`, `:57` `FirstRepository`, `:67` `ForgeRow.first`.
- Generated definitions/codecs — `temper/crates/temper-client-protocol-wire/src/generated/v1.rs:1058` `SetUpProject`, `:1098` `SetUp`, `:1272` `FirstAsk`, `:2221` `FirstState`, `:2257` `FirstRepository`, `:2697` `ForgeRow`; generated golden/bound cases include `FirstRepository` at `:13159`.
- `temper-service` — `temper/crates/temper-service/src/family.rs:61` request conversion; `:860` `first_repository` converts domain state to wire.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/forge/family.rs:179` `set_up`, `:361` `first_from`; `forge/forges.rs:55` `SetUpProject`, `:67` `FirstState`, `:92` `First`; `forge/admin.rs:151` admin request; `forge/connect.rs:168` `underway`, `:183` `failed`; `forge/listed.rs:51` `dot`; `forge/words.rs:987` `forge_connection`, `:1070` `first_repository`.
- `temper-client-view` — `temper/crates/temper-client-view/src/forge/connect.rs:568` `progress_in`, `:678` `setup_target`, `:692` `setting_up`, `:740` `ready_card`, `:825` `adopted_project`, `:909` `welcome_target`, `:1042` `project_name`, `:1071` `first_line`; `forge/forges.rs:84` `personal`, `:234` `repository_line`.
- `temper-client-tools` — `temper/crates/temper-client-tools/src/forges.rs:1289` `first_json`; `src/forges/connection.rs:84` `admin_how`, `:171` `send`.
- `temper-client-world` kit — `temper/tests/client/src/translate/forge.rs:231` `admin_request`, `:553` `set_up_request`; `temper/tests/client/src/front.rs:133` emits `FirstFailed`.
- `temper-client-gallery` — `temper/tools/gallery/src/worlds.rs:283` `forges`, constructs `FirstRepository` at `:417`.
- `temper-engine-domain-forge` tests — `temper/crates/temper-engine-domain-forge/src/tests.rs:1792` admission/replacement, `:1842` made/failed/retry tells; `src/tests/connect.rs:746` managed intent; `src/tests/demo.rs:388` `SetUpProject` fixture.
- `temper-engine-domain` tests — `temper/crates/temper-engine-domain/src/tests/setup.rs:111` `setup`, `:639` `admin_first`, `:648` admin routing.
- `temper-service` tests — `temper/crates/temper-service/src/tests/setup_create.rs:302` reconcile/Create/connect, `:341` absent intent, `:354` retry, `:431` playground copy; `setup_admin.rs:67` `first`; `family.rs:854` request fixture, `:1222` first-row projection; `system.rs:349` setup fixture.
- `temper-client-domain` tests — `temper/crates/temper-client-domain/src/tests/forges.rs:376` setup fields, `:403` all first states/projects/steps; `tests/connect.rs:451` `first`; `tests/admin.rs:26` first-row fixture.
- `temper-client-view` tests — `temper/crates/temper-client-view/src/tests/connect_forge.rs:525` `first_rows`; `tests/forges.rs:583` first-state rendering; `tests/welcome.rs:151` failed first-step rendering.
- `temper-client-tools` tests — `temper/crates/temper-client-tools/src/tests/forges.rs:35` `record_first`; `tests/forge_connection.rs:340` first-project assertion; `tests/adopt.rs:323` existing-project fixture.
- `temper-client-shell-cli` tests — `temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs:64` `FirstRepository` fixture.
- `temper-client-protocol-wire` tests — `temper/crates/temper-client-protocol-wire/src/tests.rs:302` setup fixture; `src/tests/forgejo_admin.rs:9` `first`.
- `temper-codecs-world` — `temper/tests/codecs/tests/suite/client_wire.rs:23` golden round trips, `:49` `project`, `:55` `first`.
- `temper-engine-domain-world` — `temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs:42` `setup`, `:134` busy intent, `:199` volatility, `:240` creation failures, `:272` replay, `:295` row transitions.
- `temper-client-world` — `temper/tests/client/tests/suite/forges.rs:458` setup fixture; `suite/connect_forge.rs:111` owned welcome; `qa_all/qa_first_repository.rs:108` `made`, `:179` wait for connection, `:195` nonasking tab, `:216` leaving flow, `:237` shared ctl/page key, `:274` project home, `:327` retries; `qa_all/qa_forges.rs:604` `FirstMade`.
- No skein, smith or jig code reads these **temper-defined intent/welcome types**. Jig supplies the project/account persistence mechanisms; skein supplies setup plans.

Default and starter assets:

- `temper-client-domain` — `temper/crates/temper-client-domain/src/forge/forges.rs:377` `AT_ONCE_ORG = work`, `:378` `AT_ONCE_REPOSITORY = hello`, `:379` `AT_ONCE_PROJECT = Default`.
- Their sole direct production consumer is `set_up_at_once` `:381`, requesting **`SetUpProject::New("Default")`**. This is distinct from init’s seeded project 1.
- Dispatch readers — `temper-client-domain/src/forge.rs:263` `Act::SetUpAtOnce`; `temper-client-view/src/forge/bindings.rs:22` `Meaning::SetUpAtOnce`, `:88` action translation; `src/forge/connect.rs:313` button.
- Tests — `temper-client-domain/src/tests/connect.rs:1293` `at_once_sends_fixed_names_without_reading_or_marking_the_form`; `temper/tests/client/tests/suite/connect_forge.rs:111` asserts Default target, creation and “Open Default”.
- `temper-forge-setup` — `temper/crates/temper-forge-setup/src/lib.rs:30` `STARTER_PATH`, `:35` `starter_workflow`, `:55` `PLAYGROUND_PATH`, `:57` `PLAYGROUND_MAIN`.
- All direct readers: `temper-service/src/setup/lifecycle.rs:1003` `create_local`, `:1423` `admin_create`; `temper-fake/src/forge.rs:24` playground seed files; `temper-forge-setup/src/tests.rs:19` starter test; `temper-service/src/tests/setup_create.rs:333`, `:443`; `temper-service/src/tests/setup_admin.rs:220`, `:398`.

**2. Client welcome, unfinished setup and tests**

- `temper-client-domain` — `temper/crates/temper-client-domain/src/address.rs:12` `Address::Welcome`; `:19` `Step` has **Welcome, Forge, Model, Ready**.
- Same file — `print` welcome arm `:88`: `/welcome`, `/welcome?at=forge[&forge=…]`, `/welcome?at=model[…]`, `/welcome?at=ready`; `welcome_step` `:337` parses them. First readable `at` wins; invalid values fall back to Welcome.
- `temper-client-shell-web` — `temper/crates/temper-client-shell-web/src/lib.rs:63` printed-link `Route.home` is `/welcome`.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/pages/welcome.rs:12` `Welcome { decided }`; `decision` `:22` waits for me, demo, model and relevant forge facts.
- Owner’s initial welcome ends when a non-demo forge is **connected** and a real model is usable, or when the demo runs. Nonowners do not require the forge. This decision does **not** require first-repository adoption.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/domain.rs:145` `welcome` runs that decision once per opening, **only on `Step::Welcome`**, replacing it with Chats when complete. Explicit later steps never automatically redirect by that function.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/forge/listed.rs:98` `setup` chooses Continue-setting-up destination: Forge if owner and no forge started; otherwise missing Model; otherwise Ready. It suppresses the strip while demo runs.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/route.rs:349` `forge_forges` advances accepted owned setup to Model; `route.rs:5` `forge` advances successful welcome adoption to Model.
- `temper-client-domain` — `temper/crates/temper-client-domain/src/departure.rs:7` `told` registers `/welcome?at=ready` as the return from Model sign-in (`:43`).
- `temper-client-view` — `temper/crates/temper-client-view/src/pages/welcome.rs:67` `steps`; `:120` `main` composes welcome cards, shared forge flow, account ways and Ready.
- Same file — `main` `:135` offers playground while demo is known and no real forge is connected; `:184` supplies Back/Later/playground controls on Forge/Model.
- Same file — `ready` `:332` requires durable adoption to enable Open-project; without a target it uses `legacy_ready` `:208`. Model readiness is shown separately.
- **Today’s unfinished view is inferred**, not a persisted setup lifecycle:
  - `temper-client-domain/src/forge/connect.rs:143` `personal` selects the first personal started/connecting/connected/failed forge.
  - `:168` `underway`: GitHub remains underway without accepted installation; a connected Forgejo remains underway while first repository is Making.
  - `:183` `failed`: creation failure or setup failure.
  - `temper-client-view/src/forge/connect.rs:166` `welcome` follows that forge rather than offering another.
  - `:909` `welcome_target`: GitHub uses durable `row.repositories`; Forgejo uses volatile `row.first`, falling back to this client’s retained setup.
- Browser/client reload loses the local `SetUp`, pending first project/adoption and form state. `temper-client-domain/src/forge/forges.rs:303` `SetUp` is session memory; `configure` `:319` also clears it on sign-out.
- **No “FORGE · not finished” / Start-over removal flow exists. No `RemoveForge`, `RemoveProject`, `EndPlayground` or `StartPlayground` implementation exists in this checkout.**

Other production welcome readers:

- `temper-client-domain` — `temper/crates/temper-client-domain/src/pages.rs:191` `Pages::welcome`, `:196` `decide_welcome`, `:333` Ready sign-in, `:570` welcome account holder, `:582` new welcome state.
- Same crate — `src/pages/connect_model.rs:30` `open`, `:57` `address`, `:72` `way`, `:99` `changing_way`; `src/forge/picker.rs:123` `progress`.
- `temper-client-view` — `temper/crates/temper-client-view/src/root.rs:119` `build`, welcome dispatch `:185`; `src/frame.rs:169` welcome frame, setup-strip link `:368`; `src/choice.rs:44` `banner`, `:86` `can_send`; `src/pages/connect_model.rs:227` welcome-specific presentation.
- Same crate — `src/pages/welcome.rs:96`, `:273` and `src/choice.rs:91` are direct `model_usable` readers; domain `forge/listed.rs:99`, `:114` reads `decision`/`model_usable`.

Welcome tests and test-kit readers:

- `temper-client-domain` — `temper/crates/temper-client-domain/src/tests/pages/welcome.rs:153` owner completion, `:161` demo exclusions, `:171` nonowner, `:178` missing facts, `:192` ended watches, `:228` once per opening, `:238` later steps, `:248` playground, `:375` strip, `:410` unfinished-step choice, `:451` model usability.
- Same crate — `src/tests/address.rs:141` step URLs, `:153` invalid queries, `:246` forge choices; `src/tests/pages/connect_model.rs:222` shared model addresses; `src/tests/connect.rs:1209` adoption advances only welcome, `:1267` owned shortcut, `:1328` accepted shortcut advances Model; `src/tests/admin.rs:295` welcome admin flow.
- `temper-client-view` — `temper/crates/temper-client-view/src/tests/welcome.rs:202` bare frame; `:225` cards; `:303` no-demo; `:312` awaiting facts; `:328` steps; `:375` controls; `:600` connected forge hides playground; `:620` underway Ready; `:686` failure persistence; `:809` durable GitHub adoption; `:823` managed Ready; `:845` GitHub Ready; `:1051` owned five-step progress.
- Same crate — `src/tests/connect_forge.rs:329` owned shortcut, `:346` underway forge, `:415` underway Ready, `:1359` MakeProject/Adopt ordering.
- Same crate — remaining welcome consumers: `src/tests/forges.rs:163`, `src/tests/connect_model.rs:538`, `src/tests/demo.rs:20`, `src/tests/demo_start.rs:340`; shared fixtures `src/tests/fixtures.rs:265`, `:1500`.
- `temper-client-world` — `temper/tests/client/src/front/account_acts.rs:268` default welcome address.
- Same crate — `tests/suite/welcome.rs:12`, `:115`, `:151`; `suite/connect_forge.rs:111`, `:193`, `:221`, `:258`; `suite/connect_model.rs:215`; `suite/demo_start.rs:145`.
- Same crate — `tests/qa_all/qa_welcome.rs:41` canonical URLs, `:54` invalid fields, `:72` repeated `at`, `:86` null demo, `:151` later-step stability, `:193` printed-link home, `:205` fact order, `:245` running-demo redirect.
- Same crate — `tests/qa_all/qa_welcome_m3.rs:156` GitHub advances Model, `:164` adoption survives reload; `qa_welcome_m3_owned.rs:101`, `:171` owned progress/local target; `qa_welcome_m4.rs:43`, `:126`, `:158` failures/retirement/reload.
- Same crate — `tests/qa_all/qa_first_run_hooks.rs:96`, `:165`, `:213`; `qa_forges.rs:283`; model/return URL readers `qa_connect_model.rs:32`, `qa_sign_in.rs:237`, `qa_sign_in_paste.rs:123`.
- `temper-client-browser-world` — `temper/tests/client-browser/tests/browser_all/browser_sign_in.rs:16`, `:43`; `browser_github.rs:104`, `:111`, `:117`; `tests/qa_all/qa_shell.rs:146` printed-link reopen.
- `temper-one-process-world` — `temper/tests/one-process/src/m15/pages.rs:9` `steps`, welcome address `:19`.
- `temper-client-gallery` — `temper/tools/gallery/src/samples.rs:74` `samples`, `:188` `welcome_sample`, `:206` `welcome_model_sample`; `src/worlds.rs:162` `world`, `:265` `first_run_ready`; `src/tests.rs:861` welcome samples.
- `temper` native/process support — `temper/crates/temper/tests/support/person.rs:98` `welcome_sign_in`, `:269` `welcome_open`; `support/playground.rs:29` welcome reopen, `:74` `retired_reload`; `support/sign_in_welcome.rs:12` `READY`, `:106` `run`.
- `temper` process tests — `temper/crates/temper/tests/process.rs:249` welcome account survives restart; `:1783` demo→owned setup; `:2100` running demo after restart; `:2224` permanent demo end; `:4066` managed first repository/restart; `:4553` playground copy/worker reads.
- `temper` QA — `temper/crates/temper/tests/qa_all/qa_first_run.rs:119` runtime demo end; `qa_first_repository.rs:43` completed adoption/task survives restart. These test completed paths, not persisted first intent.

**3. Earlier work before the cutoff:** none usable: no `playground-apart.md` in the history, no remote branch before the cutoff that builds forge records, project deletion or forge removal, and no note that does (survey S5, summarised by the lead).

**4. Playground and first-run design/status**

- `temper/docs/design/playground.md:1` describes the **browser-hosted simulated world**, not today’s native box playground.
- Its intended crates: `playground.md:80` `temper-playground`; `:81` web-shell playground feature; `:82` product `temper-fake-git-hosted`; `:83` `temper-playground-world`.
- Status: `temper/docs/design/playground.status.md:5` — **built only:** fake-forge addresses rendered as text, not links. `:6` — **not yet:** page, native walk, browser tier, delivery.
- Checkout agrees on missing composition: no `temper-playground`/`temper-playground-world` crate or web-shell playground feature; fake git remains under `temper/testing/temper-fake-git-hosted/`.
- `temper/docs/design/draft/first-run.md:126` §2.3 specifies Welcome→Forge→Model→Ready, lazy demo start, permanent demo end on real forge, separately retired model/forge halves.
- `first-run.md:171` §2.4 specifies no initial project, Playground created at demo start, first real project named after its repository; `:200` records the historical wait on jig runtime projects.
- **No `draft/first-run.status.md` exists.**
- `temper/docs/design/protocol/setup.status.md:5` marks welcome/setup/demo/whole-first-run paths built; `:6` still marks runtime-created projects and complete restart-mid-start recovery not yet.
- That runtime-project status is **stale relative to code**: `jig-domain-core/src/routing/projects.rs:36` implements `make_project_request`; `temper-engine-domain/src/tests/setup.rs:495` tests policy-copy/commit ordering; client `StartIn::ask` sends it.
- The default still **seeds project 1**: `temper-init` — `temper/crates/temper-init/template/temper.json:165` `engine.projects`. Its playground configuration remains `application.demo` at `:358`; runtime welcome Default is the separate new-project request described above.

**5. Old forms**

- No repository-root `scripts/` directory exists; the scripts/baselines are per tree.
- All four scripts count tracked Rust source, excluding `generated/`, stripping comments; they do **not** scan `temper.json` or Markdown:
  - skein: `skein/scripts/old_forms.py:147` `counts`.
  - smith: `smith/scripts/old_forms.py:93` `counts`.
  - jig: `jig/scripts/old_forms.py:96` `counts`.
  - temper: `temper/scripts/old_forms.py:96` `counts`.
- Shared `FORMS`: identifiers containing `max_out`/`MAX_OUT`; exact `JournalRoom`; `.takes(&`; identifiers containing `worst_case`.
  - Definitions: `skein/scripts/old_forms.py:31`, `smith/scripts/old_forms.py:23`, `jig/scripts/old_forms.py:21`, `temper/scripts/old_forms.py:21`.
  - All four baseline counts are **0**: each tree’s `scripts/old-forms.txt:3`.
- Additional skein forms, `skein/scripts/old_forms.py:37`:
  - `io_resume`: `(skein_)io::resume`.
  - `earliest_helper`: functions `earliest`/`earlier`.
  - `lifecycle_enum`: `enum Lifecycle`.
  - `growth_floor`: `const GROWTH_FLOOR`.
  - `pop_loop`: counted loop whose whole body pops/discards an item.
  - `hex_decimal_fn`: functions `hex`/`decimal`.
  - `host_forwarder`: `impl …Host for Process`.
- Skein baseline, `skein/scripts/old-forms.txt:8`: respectively **8, 1, 1, 1, 0, 6, 4**. Shared-item owner exclusions: `skein/scripts/old_forms.py:73`.
- Exemptions: skein reserved bounds/provider vocabulary `:79`; smith provider/capture vocabulary and frozen benchmark directory `smith/scripts/old_forms.py:29`; jig/temper qualified kernel edge constants `jig/scripts/old_forms.py:27`, `temper/scripts/old_forms.py:27`.
- **No counter tracks Demo/playground, forge document members, configured forge slots, setup intent, project deletion or removal acts.** These counters are memory/component-form ratchets, not a forge/playground migration inventory.