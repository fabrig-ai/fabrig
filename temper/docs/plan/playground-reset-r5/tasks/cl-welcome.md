+++
id = "cl-welcome"
part = "client"
kind = "feature"
needs = ["cl-connect", "cl-play-view"]
files = ["temper/crates/temper-client-domain/src/pages/welcome.rs", "temper/crates/temper-client-domain/src/forge/connect.rs", "temper/crates/temper-client-domain/src/domain.rs", "temper/crates/temper-client-domain/src/tests/pages/welcome.rs", "temper/crates/temper-client-view/src/pages/welcome.rs", "temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-view/src/forge/words.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/tests/client/tests/suite/welcome.rs", "temper/tests/client/tests/qa_all/qa_welcome*.rs"]
effort = "high"
minutes = 110
+++

## Goal

The welcome's three states and its Forge step's unfinished forge (draft
`temper/docs/design/draft/playground-reset.md`, 6.3 and 6.4): empty, the
welcome offers "Try the playground" and "Set up temper"; unfinished, it
opens at the step left undone, whose Forge step shows "FORGE · not
finished" with Start over (and, for GitHub, Go on first); whole, it moves
on to the chats page.

Acceptance:

- `pages::welcome::state(root) -> Option<State>`, `State::{Empty,
  Unfinished(Step), Whole}` (None while facts are missing, as `decision`
  waits today), from what the engine reports (`notes/playground.md`,
  section 5): a *real forge* is a Forges row not `Mode::Fake`; it is
  *whole* when connected and its first repository is adopted (a
  `repositories` entry, or its `first` `Made`); a *real account* is a
  usable model of a provider other than the playground's.
  - `Empty`: no real forge and no real account.
  - `Unfinished(Step::Forge { forge: Some(Choice::Forge(n)) })` when a
    real forge `n` is kept but not whole; else `Unfinished(Step::Model
    { way: None })` when a real forge is whole but no real account is
    usable; a real account and no real forge is `Unfinished(Step::Forge
    { forge: None })`.
  - `Whole`: a whole real forge and a real account.
  A running playground moves on from `/welcome` as today. A person who
  is not a deployment owner: `Whole` once a real account is usable, as
  today's non-owner decision. `decision` is rewritten on `state` (its
  readers keep working), and `Domain::welcome` replaces `/welcome` by the
  state's step (Unfinished) or the chats (Whole), once each time it opens.
- The Forge step with an unfinished forge (`Step::Forge { forge:
  Some(Choice::Forge(n)) }` naming a real forge not whole): the row's
  name then "· not finished" (`Word::ForgeNotFinished`), its progress or
  failure as today's underway forge shows it, and Start over
  (`forge::form::Act::StartOver { forge }`): sends `cl-forges`'
  `RemoveForge { forge }` at once (no second confirmation; the step's
  words say "Start over removes this forge and what temper set up for it;
  an owned Forgejo's binaries stay in the cache"), and on `removing`
  replaces the address with `Step::Forge { forge: None }` (the choices).
  For GitHub (`Kind::GitHub`), Go on (to the App's installation:
  `Act::Install { forge, back }` as today's GitHub flow sends it) comes
  first, and Start over's words add "The GitHub App stays yours: delete
  it in its settings on GitHub." Back and Later stay as today.
- The welcome step (`/welcome`, `Empty`): the playground card while
  `playground_offered()`, and the setup card; no playground card
  otherwise. The cards' words stay.
- An account added before stays (draft 6.4): the Model step shows it
  ready, as today.
- Tests: domain (`src/tests/pages/welcome.rs`):
  `an_empty_store_is_empty_and_offers_both`,
  `a_kept_forge_not_connected_is_unfinished_at_its_forge`,
  `a_connected_forge_without_its_first_repository_is_unfinished`,
  `a_whole_forge_without_a_real_account_is_unfinished_at_the_model`,
  `a_real_account_alone_is_unfinished_at_the_forge_step`,
  `a_whole_forge_and_a_real_account_move_on`,
  `the_playgrounds_forge_and_account_count_for_nothing`,
  `start_over_sends_remove_forge_and_opens_the_choices`; view
  (`src/tests/welcome.rs`): `the_unfinished_forge_says_not_finished_with_start_over`,
  `github_offers_go_on_before_start_over_and_names_the_app`,
  `an_empty_store_offers_the_playground_and_set_up`; client world
  (`tests/suite/welcome.rs`): `an_unfinished_owned_forge_is_started_over_from_the_welcome`,
  `a_github_forge_goes_on_to_its_installation`,
  `a_store_left_unfinished_opens_at_the_step_left_undone`. Today's welcome
  cases on the demo's facts and the strip deleted or moved.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`,
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (sections 5
  and 6), `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (section 6).
- The draft, 6.3 and 6.4. `temper/docs/design/client/README.md`, 3.5
  (the welcome). The journey `temper/docs/design/client/journeys/welcome-set-up/02.html`
  (the Forge step, as the playground's reset opens it).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s5-setup-welcome.md`,
  section 2 (the welcome's code and every reader and test).

## Owns and may change

- Owns the files in the header; in `domain.rs` only `welcome`; in
  `forge/connect.rs` only the welcome's step functions (`welcome_step`,
  `personal`, `underway_forge`); in `forge/words.rs` only new words.
- May change, call sites only: readers of `decision` (`forge/listed.rs`,
  `pages.rs`, `temper-client-view/src/choice.rs`, `frame.rs`), the
  gallery's welcome samples (`temper/tools/gallery/src/samples.rs`,
  `worlds.rs`), `temper/tests/one-process/src/m15/pages.rs`.
- Not yours: the journeys (`cl-journeys`): list a journey test failing
  because the welcome changed under `## Left`.
- If `Step::Forge`'s `forge` field cannot name a kept forge in the
  address (`?at=forge&forge=N`), make it so in `address.rs` (call sites
  only) and say so.

## Steps

1. Domain: `state`, `decision`, `welcome`, Start over; domain tests
   (`heavy cargo nextest run -p temper-client-domain --lib -E
   'test(/welcome/)'`).
2. View: the Forge step's unfinished forge, the welcome step's cards;
   `heavy cargo nextest run -p temper-client-view --lib -E
   'test(/welcome/)'`.
3. Client world: `heavy cargo nextest run -p temper-client-world --test
   suite -E 'test(/welcome/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D
   warnings`; `heavy cargo check --tests -p temper-client-gallery -p
   temper-one-process-world`; the focused tests of the three and the
   gallery. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says.
