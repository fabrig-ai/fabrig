+++
id = "cl-forges"
part = "rf"
kind = "feature"
needs = ["cl-choices"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/tools/gallery/**",
  "temper/tests/client/**",
  "temper/docs/design/client/journeys/*/0*.html",
]
effort = "high"
minutes = 100
+++

## Goal

The Forges page lists every forge record the engine lists, the playground's aside, and
each row offers Remove, confirmed, whose words name what goes and what stays on the forge
(draft `temper/docs/design/draft/playground-reset.md`, sections 3 and 4, "Its client";
`notes/contracts.md` T5, "The client").

- `listed::rows` holds every row whose mode is not `Fake` and that is not the playground's,
  whatever its connection (a record nothing has begun on is listed too, "not set up" or
  "not connected" in today's words).
- Opened, each listed row ends with the action "Remove forge…". Pressed, the row shows the
  confirmation in place: its words (step 3), "Remove forge" (a button, sending
  `remove_forge { forge }`) and "Keep it" (closing the confirmation). One removal at a time.
- While the removal is in flight: "Removing forge N…", the buttons disabled. On `removing`:
  "Removing forge N: temper is ending its work and erasing its records." until the row
  leaves the document, then nothing (the row is gone). On `connect_refused {
  no_such_forge }`: "This forge is gone already."; on `playground`: "The playground's forge
  goes with the playground." ; `unknown` and an entrance refusal as other forge requests
  show theirs.
- The navigation dot and the counts read the new list.
- The journeys' test still passes (the Forges page is in neither journey).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, T8;
  `notes/cl-wire.md` 3 (the removal's outcomes).
- `temper/docs/design/client/forge.md`, 5.4 (the Forges page, its rows and actions);
  `temper/docs/design/client/README.md`, 3.6 (requests and confirmations).
- The draft, section 4.

## Owns and may change

Owned: `temper/crates/temper-client-domain/`, `temper/crates/temper-client-view/` (src and
tests), `temper/tools/gallery/`. The shells, tools and the joined client world (`temper/tests/client/`) only where the compiler requires, or, in the world, an expectation the change moves (each listed in the report).

Options, pre-approved:

- The confirmation is part of the row's disclosure, drawn as the existing account removal's
  confirmation is (`temper-client-view/src/pages/accounts.rs`, `confirmation`); if a shared
  confirmation helper exists in the view (`tests/confirm.rs` names one), use it.
- A removal's answer is kept on `Side` until the row leaves the document or the person
  signs out; leaving the page keeps it.

## Steps

1. **Domain:** `forge/listed.rs::personal` lists by mode and playground alone (drop the
   connection test). New `forge/removing.rs`:

   ```rust
   pub enum RemoveAnswer { Removing, NoSuchForge, Playground, Unknown, EntranceRefused(c::EntranceRefusal) }
   pub struct Removal { pub forge: u16, pub confirming: bool, pub request: Option<Token>,
       pub progress: Option<RequestProgress>, pub answer: Option<RemoveAnswer> }
   ```

   `ask(forge, limits)` (deployment scope, `w::Request::RemoveForge`), `answer(&Answered,
   limits)` (`w::Outcome::Removing` → `Removing`, `ConnectRefused(NoSuchForge)` →
   `NoSuchForge`, `Playground` → `Playground`, `Unknown`, others `None`). `forge::Act`
   gains `AskRemove { forge }` (confirming), `KeepForge` (drops an unsent confirmation),
   `RemoveForge { forge }` (sends; ignored while a removal is in flight or the forge is not
   listed). `told` beside `making::told`; once the document no longer lists a `Removing`
   forge, the removal is dropped. Accessor `Side::removal()`.
   Tests (`src/tests/forges.rs` or a new `src/tests/removing.rs`):
   `every_kept_row_is_listed_but_the_playgrounds`,
   `remove_asks_first_then_sends_one_deployment_request`,
   `keep_it_drops_the_confirmation_and_sends_nothing`,
   `removing_is_kept_until_the_row_leaves_the_document`,
   `no_such_forge_and_playground_refusals_are_kept_for_the_row`,
   `one_removal_at_a_time`.
2. **View** (`forge/forges.rs`, `forge/bindings.rs`, `words.rs`): after a row's actions,
   "Remove forge…" (`AskRemove`); confirming, the words of step 3 under the heading
   "Remove forge N?", then "Remove forge" (danger style, as the account removal's) and
   "Keep it"; in flight and answered, the lines of the Goal. A row whose confirmation is
   open draws open.
3. **The confirmation's words**, by the row (`kind`, `mode`), each a paragraph:
   - every row: "temper forgets this forge: it stops working on it, cancels the changes
     open on its repositories, takes them out of their projects and erases its records and
     secrets. Chats that worked on it stay readable.";
   - an owned Forgejo (`mode` owned): "Forgejo on this machine goes with it: temper stops
     it and deletes its directory, every repository on it with it. Its downloaded binaries
     stay, for a next set up.";
   - a managed Forgejo (`kind` Forgejo, `mode` managed or connected): "On the forge, its
     bot and its webhooks stay: delete them there if you no longer want them.";
   - GitHub: "On GitHub, temper's App stays yours: delete it in its settings." and the
     external link "The App's settings on GitHub ↗" to `WEB/settings/apps`, `WEB` the
     row's `web` or `https://github.com`.
4. **View tests** (`src/tests/forges.rs`):
   `a_record_nothing_begun_on_is_listed_with_its_connection`,
   `an_opened_row_offers_remove_and_confirms_in_place`,
   `each_kind_says_what_goes_and_what_stays`,
   `removing_shows_its_progress_then_the_row_goes`,
   `a_refused_removal_says_why`. Update tests that expected unlisted `NotSetUp` rows
   (`no_forge_of_yours_yet_offers_connect_a_forge` and kin) to the new list, keeping what
   they check otherwise. `heavy cargo nextest run -p temper-client-view --lib` and `-p
   temper-client-domain --lib`.
5. **Gallery:** states `forges-remove-confirm` (a connected owned Forgejo's confirmation)
   and `forges-removing`. `heavy cargo nextest run -p temper-client-gallery --lib`.
6. `heavy cargo nextest run -p temper-client-journeys --lib` (as cl-choices' step 7 says:
   stale is a fault of this task; a page's bytes alone, reprint), `-p
   temper-client-world --test suite`, `-p temper-client-tools --lib`; `cargo fmt`; clippy
   `--tests` of the changed packages and the wasm clippy of domain and view; old forms;
   commit and push.

## Report

- Under `## Left`: `temper/docs/design/client/forge.md` 5.4's "lists the person's forges"
  rule and actions table, which this changes.
