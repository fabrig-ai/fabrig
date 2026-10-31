+++
id = "cl-welcome"
part = "client"
kind = "feature"
needs = ["cl-connect-view", "cl-play-domain"]
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/{welcome,welcome_words}.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
]
effort = "high"
minutes = 75
+++

## Goal

The welcome's three states (draft 6.3): the store is empty, unfinished
or whole, judged from the kept forges (`Row.finished`) and the accounts.
Empty: the welcome offers "Try the playground" and "Set up temper".
Unfinished: it opens at the step left undone, the Forge step (its
unfinished forge, `cl-connect-view`'s) or the Model step (an account
added before shows ready, 6.4). Whole: it moves on to the chats page.
The playground is offered while no real forge is kept, and never once
one is; its start no longer needs a held read.

Acceptance:

- `temper-client-domain`, `src/tests/pages/welcome.rs`:
  `an_empty_store_stays_on_the_welcome`;
  `a_kept_forge_not_finished_opens_the_forge_step`;
  `a_real_account_without_a_forge_opens_the_forge_step`;
  `a_finished_forge_without_a_usable_account_opens_the_model_step`;
  `a_whole_store_moves_on_to_the_chats`;
  `a_running_playground_moves_on_to_the_chats`;
  `leaving_decides_nothing`;
  `the_playground_starts_without_a_held_read_and_selects_its_project_once_running`;
  `a_non_owner_moves_on_as_before` (today's non-owner rule).
  Each opening is a `Request::Show { replace: true }`, once per opening,
  as today's decision.
- `temper-client-view`, `src/tests/welcome.rs`:
  `an_empty_store_offers_the_playground_and_set_up`;
  `a_kept_real_forge_offers_no_playground` (finished or not; the footer
  offer too); `the_model_step_shows_an_account_added_before_ready`.
- The existing tests of both files pass, rewritten where they relied on
  the old decision (each listed in the report).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/tasks/cl-surface.md` (step 5:
  `decide` moved here), `cl-connect.md` (`connect::unfinished`,
  `Shown::Unfinished`), `cl-play-domain.md` (`playground()`, `Leaving`).
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.3, 6.4.
- `temper/docs/design/client/README.md`, 3.5 (the welcome and its
  decision: what changes).
- `temper/docs/design/client/journeys/welcome-set-up/README.md` and
  `welcome-playground/README.md`.

## Owns and may change

Owns the files in the header. In `D/route.rs` (held by `cl-play-domain`
before you): the `playground` fn alone. May add `mod welcome_words;` in
`temper/crates/temper-client-view/src/pages.rs` (the line only). Stop
reading the domain's shims (`forge::demo::{other_model_usable,
owner_deployment, ..}`, `listed::setup`) in your files where the new
rule needs them no longer; do not delete them (`cl-tidy`).

## Steps

D = `temper/crates/temper-client-domain/src`, V =
`temper/crates/temper-client-view/src`.

1. **The judgment (D/pages/welcome.rs).**
   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Store { Empty, Unfinished(Undone), Whole }
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Undone { Forge, Model }
   /// None while facts are missing (the forges watch, the accounts, the read).
   pub fn store(root: &Domain) -> Option<Store>;
   /// Whether "Try the playground" is offered (draft 6.3).
   pub fn playground_offered(root: &Domain) -> Option<bool>;
   ```
   A real forge is a row whose mode is not `Fake` and which is not the
   running playground's. A usable real account: `model_usable` as today
   (the scripted provider excluded). `Whole`: a real forge `finished`
   and a usable real account. `Unfinished(Forge)`: no real forge
   `finished` (kept not finished, or none kept but a usable real
   account). `Unfinished(Model)`: a real forge `finished`, no usable
   real account. `Empty`: no real forge kept, no usable real account.
   `playground_offered`: the playground's read has answered and no real
   forge is kept.
2. **The decision (`decide`, moved here by `cl-surface`):** at
   `Step::Welcome`, once per opening, with facts: a running playground
   or `Whole` → the chats (as today); `Unfinished(Forge)` →
   `Welcome { step: Forge { forge: None } }`; `Unfinished(Model)` →
   `Welcome { step: Model { way: None } }`; `Empty` stays; `Leaving`
   decides nothing. A non-owner keeps today's rule. `decision` goes if
   nothing else reads it (else it returns whether `store` is `Whole`).
3. **The start (D/route.rs `playground`):** send the start whether or
   not a read is held; open the chats at once; select the playground's
   project when the running read arrives (in `demo_progress`'s
   neighbourhood: a `pending_select: bool` the route holds, or the pages'
   own selection on the next read; pick the one that needs no file
   outside yours and say which).
4. **The view (V/pages/welcome.rs):** the playground card and the footer
   offer only when `playground_offered` is `Some(true)`;
   `real_forge_connected` goes for `store`; the Forge step draws
   `crate::forge::connect::welcome` (the unfinished forge is its); the
   Model step as today. New words, if any, in `welcome_words.rs` after
   `V/accounts/words.rs`'s pattern.
5. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; one `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world -p temper-client-journeys`.
6. **Journeys:** both walks start on an empty store; reprint, and if the
   journeys test reports one stale (the start's exchange order moved),
   `heavy cargo run -p temper-client-journeys -- again NAME`; if `again`
   refuses, stop and ask (`Question:`, its refusal).
7. Old forms from `temper/`; commit and push.

## Report

- The rule chosen for step 3, and each test rewritten.
- What the code taught for `client/README.md` 3.5.
