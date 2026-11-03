+++
id = "welcome-ready"
part = "wizard"
kind = "feature"
needs = ["demo-project", "no-seeded-project"]
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/forge/admin.rs",
  "temper/crates/temper-client-domain/src/forge/picker.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/connect.rs",
  "temper/crates/temper-client-domain/src/tests/forges.rs",
  "temper/crates/temper-client-domain/src/tests/admin.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/tests/suite/welcome.rs",
  "temper/tests/client/tests/suite/first_repository.rs",
  "temper/tests/client/tests/suite/connect_forge.rs",
  "temper/tests/client/tests/suite/welcome_m3.rs",
  "temper/tests/client/tests/suite/welcome_m3_owned.rs",
  "temper/tests/client/tests/suite/welcome_edges.rs",
  "temper/tests/client/tests/suite/first_run_hooks.rs",
  "temper/tests/client/tests/suite/forges.rs",
  "temper/tests/client/tests/suite/forges_edges.rs",
  "temper/tests/client/tests/suite/github_connect.rs",
  "temper/tests/client/tests/suite/task.rs",
  "temper/crates/temper/tests/support/managed_client.rs",
  "temper/docs/design/client/journeys/welcome-set-up/01.html",
  "temper/docs/design/client/journeys/welcome-set-up/02.html",
  "temper/docs/design/client/journeys/welcome-set-up/03.html",
  "temper/docs/design/client/journeys/welcome-set-up/04.html",
  "temper/docs/design/client/journeys/welcome-set-up/README.md",
  "temper/docs/design/client/journeys/welcome-set-up/recording",
  "temper/tools/journeys/src/tests.rs",
  "temper/tools/journeys/tests/suite/look_refresh.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/forge.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 120
+++

# The wizard adopts nothing, and Ready makes "Default"

## Goal

The welcome is one wizard on every forge, Welcome, Forge, Model, Ready: its Forge step connects a
forge and nothing more, asking no project's name and adopting no repository, and its Ready makes
the project "Default" and opens it. No client flow adopts a set up's first repository any more:
`ConnectForge`'s Ready offers "Adopt a repository" instead. The wire is unchanged in this task:
`SetUp` and `ForgejoAdmin.first` still carry a project, which the client fills with the
repository's name and nothing reads (`setup-project-goes` removes it).

Acceptance:

- `temper-client-domain`'s `pages/welcome.rs` `decision` is K2: `Whole` once a real forge is
  connected (its row `Connected`, and on GitHub `forges::accepted`), a model is usable and `me`
  lists a project the person holds; `first_adopted` is gone; the first step not done otherwise,
  Ready among them.
- Ready's one act sends `MakeProject { name: "Default", policy: 1 }` once per press, as K1 says,
  opens the made project, opens the person's own "Default" on `NameTaken`, and offers itself
  again after any other refusal or no answer.
- No `SetupField::Project` or admin project field, no "Into a new project, named" anywhere in the
  client; nothing in the web client sends `MakeProject` or `Adopt` for a set up's first
  repository (`maybe_adopt` and its chain gone); GitHub's Ready has no picker of its own.
- The welcome's Forge step moves on to Model on a connected forge its address names, as 3.5 says.
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`; `notes/contracts.md`, K1 and K2.
- `temper/docs/design/client/README.md`, 3.5, The welcome: *The store's state*, *Forge* (its
  last paragraph: the move to Model), *Model* (the forge's line), *Ready*; 5.6, the welcome's
  departures from its mockup (the words); 10, the welcome's tests.
- `temper/docs/design/client/forge.md`, 5.4: Details (Forgejo on this machine, a managed
  Forgejo: no project field), Setting up (its line and fifth step), Ready on that Forgejo, Ready,
  Ready on GitHub.
- `temper/docs/design/protocol/setup.md`, 6.7: "Adopted by ctl's call alone".
- jig's `jig/docs/design/domain/people.md`, 5.1 (`MakeProject`; `jig_client::contract::Refusal::NameTaken`).

## Owns and may change

`temper-client-domain`:

- `pages/welcome.rs`: `decision` as K2 (the person "has a project" when `me.projects` lists one;
  the GitHub clause through `forges::accepted`); `first_adopted` goes. The welcome's Ready act's
  state lives here (default) or in `Pages`: the attempt's number (from 1), the pending key, at
  most one, and the last outcome, kept while the client runs. Sketch:

  ```rust
  pub struct Welcome { pub decided: bool, pub making: Option<Making>, pub attempt: u64, pub ended: Option<Ended> }
  pub enum Ended { Refused(jig_client::contract::Refusal), Unknown }
  ```

  Names are yours; the shapes are K1's.
- `pages.rs`: a new `Act` (default `Act::MakeDefault`), sending `jig_client::Ask::Jig(MakeProject {
  name: "Default", policy: 1 })` under `Key { asker: Owner::Pages.asker(), number: serial }` and
  the name `crate::key::named(crate::key::TOOLS, "welcome-default-N")`, N the attempt; refused
  while one is out or while `decision` would not have the forge and the model done. On
  `ProjectMade { project }`: open it as `Act::OpenProject` does, choosing it once `me` lists it
  (keep the number until then). On `Refusal::NameTaken`: the same with the `me` project named
  `Default` the person holds; with none, keep the refusal. Anything else: keep it, nothing sent.
- `route.rs`: `forge()`'s `adopted_in_welcome` goes; in its place, while the address is
  `Welcome { Forge { forge: Some(Choice::Forge(n)) } }` and row `n` is connected as K2 says,
  `Show { Welcome { Model { way: None } }, replace: true }`. `forge_forges`' move on an accepted
  `SetUp` stays.
- `forge/forges.rs`: `SetupField::Project` and the draft's third field go; `send_setup` sends
  `SetUpProject::New(repository)` (unread; the wire keeps it until `setup-project-goes`). The set
  up's adoption goes: `maybe_adopt`, `setup_start`, `retry_start`, the set up's `send_adoption`,
  `SetUp.adoption` and `SetUp.starting`, `FirstAdoption`, `FirstProject`, `project_told`, and
  what only they read. `SetUp.project` stays. `start_in` goes once no caller remains.
  `start.rs`'s `StartIn`, `start_project_key` and `first_repository_key` stay: ctl reads them.
- `forge/admin.rs`: its `Field::Project` goes; `FirstAsk.project` is `New(repository)`; its
  first adoption after made goes.
- `forge/picker.rs`: GitHub Ready's choice goes. Pre-approved: delete the module and
  `Side.pickers`, `Act::Picker` and their routing in `forge.rs` (default), or keep only what a
  remaining reader reads, saying which.
- `forge/form.rs`: `Act::StartIn` and `Act::Picker` go with their readers. `SetupWrite` stays.

`temper-client-view`:

- `pages/welcome.rs`: `steps` counts Ready done once the forge is connected and a model usable;
  `ready_heading` is "Almost ready" alone; `ready` and `legacy_ready` become one Ready: the
  checklist as 3.5 orders it, its Project line "Project: Default", the act "Open Default"
  (`Word::SetupOpen` with the name), disabled with the reason while forge or model is not done,
  disabled with "Making Default…" while out, then the refusal's or unknown's line; "Open Forgejo
  ↗" after the act on a connected owned Forgejo offering it (`forges::open_link`, as now).
  `model_line` is unchanged.
- `forge/connect.rs`: `welcome` shows a GitHub forge connected with an accepted installation as
  any connected forge (status line and "View"); `setup_fields` and `admin_form` lose the project
  field; `ready_card` adopts nothing (forge.md 5.4's Ready: connected title, connected as, "Made
  ORG/NAME." when the row has a made first repository, `adopt`, the Forges line; managed:
  `admin_ready` without "Open PROJECT", with "Adopt a repository"); `github_picker`,
  `github_adopted`, `picker_control`, `adopted_project`, `welcome_target`'s project and
  `project_name` (the `SetUpProject` one) go or lose the project; `first_line`, `progress_in`'s
  fifth step and `welcome_line` take 3.5's and 5.4's words with no PROJECT; `setup_ending` keeps
  "Set up again" and loses the start's and adoption's refusals.
- `forge/bindings.rs`: the meanings of what went (`StartIn`, `PickerName`, picker acts).
- `words.rs`: words that lose PROJECT take their new text (`SetupInto` → "ORG/NAME",
  `WelcomeForgeLine`, `WelcomeForgeDone`, `ForgeMakingAndAdopting` → "Making ORG/NAME",
  `ForgeMakingFirst` → "Making the first repository"); words nothing draws go (`SetupNew`,
  `SetupAdopt`, `SetupIn`, `SetupReady` if unread, `WelcomeNewProject`, `WelcomeProject`,
  `WelcomeOpensOnce`, `WelcomeForgeNoRepository`, GitHub's picker words); new: "Project: NAME",
  "It opens once your forge and model are ready.", "Making Default…", "Default was not made:
  WHY" (jig's refusal words through `w.refusal`), "It is not known whether Default was made.",
  "Made ORG/NAME.". A new word takes the next free number in `words.rs`'s numbering.

Tests, each kept to its purpose or, where its purpose was the adoption that goes, rewritten to the
new behaviour or removed; list each in the report:

- Domain: `tests/pages/welcome.rs` (`a_kept_forge_without_its_first_repository_is_unfinished_at_forge`,
  `a_whole_store_moves_on_to_chats`, `an_adopted_first_repository_without_a_model_opens_model`:
  connected, no project → Ready; with a project → Whole; plus K1's press, made, name taken, other
  refusal, unknown, one at a time); `tests/connect.rs` (490-1410: the set up's and picker's
  adoption tests go; `a_set_up_asks_for_a_new_project_regardless_of_the_current_project` becomes
  the set up sending the repository's name; `github_adoption_advances_only_the_welcome` becomes
  the move on a connected forge); `tests/forges.rs`
  (`setup_carries_each_field_and_new_project_names_without_semantic_form_checks`: two fields);
  `tests/admin.rs` (`connecting_then_ready_starts_the_project_only_once`,
  `an_admin_first_adoption_advances_only_the_welcome`).
- View: `tests/welcome.rs` and `tests/connect_forge.rs`, every test reading the project field,
  "Open PROJECT", "ORG/NAME is ready", "In PROJECT", the picker or the adoption's outcome (the
  survey's ~70 lines; `rg 'SetupNew|SetupOpen|SetupReady|WelcomeProject|picker|adopt' src/tests`).
- Client world (`temper/tests/client`, package `temper-client-world`): the files above;
  `first_repository.rs`'s adoption stories go or become "made, unadopted, adopted from the form
  by hand", `welcome.rs`'s `whole_welcome_moves_to_chats_after_the_first_repository_is_adopted`
  becomes Ready's "Open Default" then chats, `welcome_m3.rs`'s picker stories go, and
  `fixtures.rs`' helpers follow. `github_connect.rs` only if it reads the picker.
- `crates/temper/tests/support/managed_client.rs` (a live sitting, never run here): its "adopt"
  steps follow the new flow, ConnectForge's Ready then "Adopt a repository" and the form;
  `heavy cargo check -p temper --tests` only, and say so.
- **The `welcome-set-up` journey goes stale** (its recording types into "Into a new project,
  named" and its `SetUp` carries that name), and `temper-client-journeys`'
  `the_committed_journeys_are_their_print` runs in the gate. Default, unless the lead's release
  note says otherwise: delete `temper/docs/design/client/journeys/welcome-set-up/` whole (it is
  walked anew later, `temper/docs/design/client/journeys.md`, 8), and move
  `tools/journeys/src/tests.rs`' `a_reprint_keeps_a_journeys_readme` and
  `tests/suite/look_refresh.rs`' `PAGE` and `OTHER_PAGE` to `welcome-playground`'s pages. Never
  edit a recording or a print by hand.
- Any other test that fails because the welcome adopts nothing: its literal or step alone, listed.

Status, as `notes/common.md` says:

- `temper/docs/design/client/README.status.md`, **Not yet**: remove the sentence from `3.5, The
  welcome: whole once a real forge's first repository is` through `` `ready` waits for that
  adoption. `` (the one ending before `3.5, A project with no repository`).
- `temper/docs/design/client/forge.status.md`, **Not yet**: remove the sentence from `5.4: the
  set up names a project` through `` `github_adopted`). ``
- `temper/docs/design/protocol/setup.status.md`, the second **Not yet**: remove the sentence `6.7,
  15: the client adopts the first repository: … (\`forge/picker.rs\`, \`Act::Adopt\`).`, leaving
  the `6.7: \`SetUp\` and …` sentence.
- The lead's release note on the journey: take the default (delete `welcome-set-up/` whole). When
  you delete it, `temper/docs/design/client/journeys.status.md` replaces `No gap.` with
  `- **Not yet:** 9: \`welcome-set-up\`: no recording and no pages, its walk having changed (the
  Forge step names no project); it is walked anew in a browser (8).`

## Steps

1. Domain: `decision`, Ready's act, the route's move, the adoption chain gone; `heavy cargo
   nextest run -p temper-client-domain --lib -E 'test(welcome) | test(connect) | test(forges) |
   test(admin)'`.
2. View: Ready, the Forge step, `ConnectForge`'s Ready and Setting up, the words; `heavy cargo
   nextest run -p temper-client-view --lib -E 'test(welcome) | test(connect_forge)'`.
3. The client world: `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(welcome) | test(first_repository) | test(connect_forge) | test(forges) |
   test(first_run_hooks) | test(github_connect) | test(task)'`.
4. The journey (as above); `heavy cargo nextest run -p temper-client-journeys`.
5. `heavy cargo check -p temper --tests`; `heavy cargo check -p temper-client-tools --tests`
   (ctl reads `StartIn` and the keys); the help pages changed with this design: `heavy cargo
   nextest run -p temper-docs`.
6. The status edits; the citations check.
7. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world -p temper-client-journeys --all-targets -- -D
   warnings`; the focused tests of `temper-client-domain`, `temper-client-view` and their direct
   dependents (`temper-client-tools`, `temper-client-shell-web`, `temper-client-shell-terminal`,
   `temper-client-shell-cli`, `temper-client-world`). Commit the code and tests, the journey,
   then the status text; push.

## Report

As the implementer's text says, with each test removed or rewritten and why, what `picker.rs`
became, and whether the journey was deleted.
