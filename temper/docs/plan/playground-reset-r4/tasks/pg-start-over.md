+++
id = "pg-start-over"
part = "playground"
kind = "feature"
needs = ["pg-forge-remove", "pg-welcome-states"]
files = [
  "temper/crates/temper-client-domain/src/forge/connect.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/tests/start_over.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/tests/start_over.rs",
]
effort = "high"
minutes = 50
+++

## Goal

A set up cut short, found later, starts over (draft
`temper/docs/design/draft/playground-reset.md`, 6.4): the welcome's
Forge step shows the unfinished forge, "FORGE · not finished", with
Start over, which removes it (`RemoveForge`, through
`forge::remove::request`, `pg-forge-remove`) and opens the step's
choices again; for GitHub, whose App's records are kept as they come,
the step offers Go on, to the App's installation, before Start over,
which names the App on GitHub as the person's to delete.

Acceptance:

- `forge::connect::unfinished<'a>(rows: &'a State) -> Option<&'a Row>`:
  the first row that is not `playground`, not `finished`, and has no
  step live (connection not `SettingUp`/`Connecting`, and no first
  repository `Making`); `connect::live(row) -> bool` names that test;
- on `/welcome?at=forge` with no forge chosen, when `unfinished` finds a
  row, the step draws, instead of its choices: a card named "NAME · not
  finished" (`ForgeNotFinished`, NAME as `connect::name` gives it), the
  row's status line (`status`), and:
  - for a Forgejo row: `ForgeNotFinishedDescription` and a primary
    button "Start over";
  - for a GitHub row: `ForgeGitHubNotFinished`, a primary link "Go on"
    to `/welcome?at=forge&forge=N` (the row's own step, as Connect a
    forge shows it: the App's installation when it is made), then
    `ForgeGitHubStartOver`, a link `ForgeAppSettings` to
    `https://github.com/settings/apps`, and a secondary button "Start
    over";
- "Start over" sends the removal at once, unconfirmed (the step's words
  say what it does); while it is out the card says "Starting over…"
  and offers nothing; once removed, the row is gone from the rows and
  the step draws its choices again; refused, the card shows "Not
  removed: " with the refusal's words (as Forges does) and Start over
  again;
- a row whose step is live keeps today's welcome (its status, its
  progress, its link); the Forges page and Connect a forge are
  unchanged by this task;
- an account added before stays (6.4): nothing here touches accounts;
- the tests below pass, and every existing test of the packages changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- `temper/docs/design/draft/playground-reset.md`, 6.4 and section 4 ("On
  the forge itself").
- `temper/docs/design/client/forge.md`, 5.4 (the set up's steps) and
  `temper/docs/design/client/README.md`, 3.5 (`Welcome`, its Forge step).
- `temper/docs/plan/playground-reset-r4/tasks/pg-forge-remove.md` (the
  sender and its states), `tasks/pg-welcome-states.md` (`Row.finished`,
  `Row.playground`, where the welcome opens), `tasks/pg-words.md` (the
  words `ForgeNotFinished`, `ForgeNotFinishedDescription`,
  `ForgeGitHubNotFinished`, `ForgeGitHubStartOver`, `ForgeStartOver`,
  `ForgeGoOn`, `ForgeStartingOver`, `ForgeAppSettings`,
  `ForgeRemoveRefused`).

## Owns and may change

Owned: the `files` above (the two test files are new).

Neighbour files, edits allowed:

- `temper-client-domain/src/forge.rs`: the new act's dispatch arm
  calling `remove::request`; `temper-client-domain/src/tests.rs`,
  `temper-client-view/src/tests.rs`: one `mod` line each.
- `temper-client-view/src/pages/welcome.rs`: none expected (its Forge
  step calls `forge::connect::welcome`); if the card must sit above the
  step's lead, the call's place alone.

Pre-approved:

- the act is `forge::Act::StartOver { forge: u16 }` and its view
  meaning `Meaning::StartOver(u16)`; if `pg-forge-remove`'s
  `Act::Remove` plus `Act::ConfirmRemove` can be bound as one press
  without a confirmation state, use the new act anyway (Start over is
  unconfirmed by design);
- if `connect::personal` would still pick an unfinished row for the
  Model or Ready steps, leave those steps as they are and say so.

## Steps

1. **Domain** (`forge/connect.rs`, `forge/form.rs`, `forge.rs`):
   `live`, `unfinished` as the acceptance; `Act::StartOver { forge }`
   → `remove::request(side, forge, limits, out)`.
2. **View** (`forge/connect.rs`, `forge/bindings.rs`): in `welcome`,
   before `personal`'s branch, `unfinished`'s card as the acceptance;
   the Go on link through the existing `link` helper with
   `Address::Welcome { step: Step::Forge { forge:
   Some(Choice::Forge(n)) } }`; the removal's state read from the side
   (`Side::removal(n)` or the state `pg-forge-remove` gave).
3. **Tests** (client domain and view unit tests):
   - `temper-client-domain/src/tests/start_over.rs`:
     `an_unfinished_forge_is_one_with_no_live_step` (rows: a Forgejo
     `SetUpFailed`: unfinished; `SettingUp`: not; connected with a
     first repository `Making`: not; `finished`: not; `playground`:
     not; a GitHub `Connected`, App not installed: unfinished);
     `start_over_sends_the_removal_unconfirmed` (one
     `w::Request::RemoveForge { forge }` at once; again: nothing).
   - `temper-client-view/src/tests/start_over.rs`:
     `the_forge_step_shows_a_forgejo_not_finished_with_start_over`
     (card "Forgejo on this machine · not finished" or as `name` reads
     the row, the description, the button; no choices);
     `github_not_finished_offers_go_on_before_start_over` (the link "Go
     on" decoding to the row's step, the App settings link, the button
     after it);
     `starting_over_then_removed_draws_the_choices_again` (removal out:
     "Starting over…", no button; rows delivered without the row: the
     choices);
     `a_live_set_up_keeps_its_progress` (a `SettingUp` row: no "not
     finished").
   - Narrow runs per file.
4. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; focused tests of both; `heavy cargo check --tests` for
   `temper-client-tools`, `temper-client-shell-terminal`,
   `temper-client-shell-web`, `temper-client-world`,
   `temper-client-gallery`, one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: `connect::live`, `connect::unfinished`,
`Act::StartOver`, `Meaning::StartOver`.
