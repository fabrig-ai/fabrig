+++
id = "cl-welcome"
part = "pg"
kind = "feature"
needs = ["cl-forges"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/tools/gallery/**",
  "temper/tests/client/**",
  "temper/docs/design/client/journeys/*/0*.html",
]
effort = "high"
minutes = 120
+++

## Goal

The welcome reads the deployment as empty, unfinished or whole (draft
`temper/docs/design/draft/playground-reset.md`, 6.3) and a set up cut short starts over
from the welcome's Forge step (6.4): "FORGE · not finished" with Start over, and, on
GitHub, Go on before it.

- `pages::welcome::state(root) -> Option<State>`, `State::{Empty, Unfinished(Step), Whole}`,
  None while its facts are missing (as `decision` waits today). For the deployment's owner:
  - **Whole:** the playground running (as today), or a real forge whole and a real account
    usable. A forge is *whole* when it is a real row (not fake, not the playground's)
    connected with a repository adopted (`row.repositories` not empty, or its `first` made)
    and, on GitHub, an accepted installation (`forges::accepted`).
  - **Empty:** no real row begun (cl-choices' "nothing has begun on" is not begun) and no
    account of a provider other than the playground's.
  - **Unfinished(step):** otherwise; `step` is `Forge { forge: None }` while no real forge
    is whole, else `Model { way: None }`.
  For anyone else, as today: Whole when a model is usable or the playground runs, else
  Empty.
- Opening `/welcome` (the plain Welcome step): Whole replaces it with the chats page, as
  today; Unfinished(step) replaces it with that step; Empty stays. Later steps never move
  on by themselves, as today.
- The welcome's Forge step, for a begun real row that is not whole and not underway
  (`connect::underway` false), or a GitHub row connected without an accepted installation,
  draws the card "FORGE · not finished" (FORGE the forge's name as Forges names it): its
  status lines as today's welcome draws them, then on GitHub "Go on" (sending `Install {
  forge, back }`, `back` the welcome's Forge step) before "Start over"; elsewhere "Start
  over" alone, then the "View" link as today. A row underway keeps today's drawing.
- "Start over" asks in place: "Start over removes this forge and its records; the choices
  open again." with, by kind, cl-forges' "what stays" paragraph (owned: "Forgejo on this
  machine is set up afresh; its downloaded binaries are kept."), then "Start over" (sending
  `remove_forge` through cl-forges' removal) and "Keep it". Once the row leaves the
  document the step draws its choices.
- The Model step shows an account added before as ready (today's usable account lines).
- "Try the playground" is offered while no real row is begun, whatever the accounts (6.1,
  6.3); the footer's playground link likewise.
- The committed journeys replay unchanged (both start empty).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, T6
  ("Start: refused NotOffered while any Kept record that is not the playground's exists").
- `temper/docs/design/client/README.md`, 3.5 (the welcome, its steps and decision) and
  5.6; `temper/docs/design/client/forge.md`, 5.4 (rows' names and status words).
- The draft, 6.3 and 6.4; the tasks `cl-choices` and `cl-forges` (`tasks/cl-choices.md`,
  `tasks/cl-forges.md`) for the predicates and the removal this builds on.

## Owns and may change

Owned: `temper/crates/temper-client-domain/`, `temper/crates/temper-client-view/` (src and
tests), `temper/tools/gallery/`. The shells, tools and the joined client world (`temper/tests/client/`) only where the compiler requires, or, in the world, an expectation the change moves (each listed in the report).

Options, pre-approved:

- Keep `welcome::decision` as `state(root).map(|s| s == State::Whole)` if its readers
  (the setup strip, `listed::setup`) read better so; the strip goes in a later task.
- The engine refuses a playground start while a never-begun record is kept (contracts T6);
  the welcome still offers it then, and the start's `not offered` shows as today.
- If no account document distinguishes "an account of another provider" from "a usable
  one", Empty reads "no usable account of another provider" and the report says so.

## Steps

1. **Domain** (`pages/welcome.rs`, `domain.rs::welcome`, `forge/connect.rs`): `State`,
   `state`, `pub fn whole(row: &Row) -> bool` and `pub fn begun(row: &Row) -> bool` in
   `forge/connect.rs` (with cl-choices' predicate as `!begun`); `pub fn unfinished<'a>(rows,
   demo) -> Option<&'a Row>` (the first begun real row not whole); `domain.rs::welcome`
   pushes `Request::Show { address: Welcome { step }, replace: true }` for Unfinished.
   `forge::Act` gains `StartOver { forge }` (opens the removal's confirmation marked as a
   start over) — reuse `removing::Removal` with a `start_over: bool`.
   Tests (`src/tests/pages/welcome.rs`):
   `an_empty_deployment_stays_on_the_welcome`,
   `a_real_account_alone_is_unfinished_at_forge`,
   `a_forge_set_up_without_its_first_repository_is_unfinished_at_forge`,
   `a_whole_forge_without_a_model_is_unfinished_at_model`,
   `a_whole_forge_and_a_model_move_on_to_the_chats`,
   `a_github_forge_without_an_accepted_installation_is_not_whole`,
   `a_running_playground_moves_on_as_before`,
   `start_over_confirms_then_removes_and_the_choices_return`; update
   `the_welcome_moves_on_once_an_account_and_a_forge_are_the_persons` and
   `the_welcome_waits_for_its_facts` to the new rule, keeping what they check.
2. **View** (`pages/welcome.rs`, `forge/connect.rs::welcome`, `words.rs`): the card, its
   heading "FORGE · not finished", "Go on" (primary) and "Start over" (secondary, then the
   confirmation), as the Goal says; the playground card's and footer's condition from
   `!any begun real row`. Tests (`src/tests/welcome.rs`):
   `an_unfinished_owned_forgejo_offers_start_over_and_its_view`,
   `an_unfinished_github_offers_go_on_before_start_over`,
   `start_over_confirms_in_place_with_what_stays`,
   `a_real_account_does_not_hide_the_playground`,
   `the_model_step_shows_an_account_added_before_ready`; update
   `a_connected_real_forge_hides_the_playground_before_its_end_is_read` to "a begun real
   forge hides the playground".
3. **Gallery:** `welcome-forge-not-finished` (owned, set up failed) and
   `welcome-forge-go-on` (GitHub without an accepted installation).
4. `heavy cargo nextest run -p temper-client-domain --lib`, `-p temper-client-view --lib`,
   `-p temper-client-gallery --lib`, `-p temper-client-journeys --lib` (stale is this
   task's fault; a page's bytes alone, reprint with `heavy cargo run -p
   temper-client-journeys` and commit the pages, never a recording), `-p
   temper-client-world --test suite` (its welcome cases: update their expectations only
   where the new rule moves them, and list each), `-p temper-client-tools --lib`. `cargo
   fmt`; clippy `--tests` and the wasm clippy of domain and view; old forms; commit and
   push.

## Report

- Each client-world test whose expectation moved, and why.
- Under `## Left`: `temper/docs/design/client/README.md` 3.5's decision text this replaces.
