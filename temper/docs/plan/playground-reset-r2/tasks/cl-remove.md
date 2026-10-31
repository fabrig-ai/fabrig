+++
id = "cl-remove"
part = "client"
kind = "feature"
needs = ["cl-surface"]
files = [
  "temper/crates/temper-client-domain/src/forge/remove.rs",
  "temper/crates/temper-client-domain/src/tests/remove.rs",
  "temper/crates/temper-client-view/src/forge/remove.rs",
  "temper/crates/temper-client-view/src/forge/remove_words.rs",
  "temper/crates/temper-client-view/src/tests/remove_forge.rs",
]
effort = "high"
minutes = 60
+++

## Goal

Remove on a forge's row, confirmed, its words naming what goes and what
stays on the forge (draft 4, "Its client" and "On the forge itself"),
and the same confirmation for the Forge step's Start over (6.4): the
removal's domain state behind `cl-surface`'s acts, and its drawing
behind `cl-surface`'s `control` hook.

Acceptance:

- `temper-client-domain`, `src/tests/remove.rs` (beside surface's codec
  tests): `a_removal_is_confirmed_before_it_is_sent`,
  `a_cancelled_removal_sends_nothing`,
  `one_removal_request_per_forge_whichever_page_sends_it` (its named
  key), `a_removed_forge_leaves_no_removal_once_its_row_is_gone`,
  `start_over_leaves_the_forge_step_on_its_choices_once_the_row_is_gone`,
  `a_refused_removal_keeps_its_refusal`,
  `no_removal_is_sent_out_of_date_or_without_a_party`.
- `temper-client-view`, new `src/tests/remove_forge.rs`:
  `each_personal_row_offers_remove`,
  `the_confirmation_names_what_goes_and_what_stays` (owned Forgejo,
  managed Forgejo, connected Forgejo, GitHub: each its words),
  `githubs_confirmation_links_to_its_app_settings`,
  `start_over_confirms_in_its_own_words`,
  `a_sent_removal_shows_removing_then_its_refusal`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5
  (`RemoveForge`'s answers).
- `temper/docs/plan/playground-reset-r2/tasks/cl-surface.md`, steps 3, 8,
  9, 11 (the state, acts, bindings and hook it fills).
- `temper/docs/design/draft/playground-reset.md`, 4, 6.4.
- `temper/docs/design/client/forge.md`, the Forges page's rows (find by
  `rg -n '^#' temper/docs/design/client/forge.md`).

## Owns and may change

Owns the files in the header. May add its test module's `mod` line in
`temper/crates/temper-client-view/src/tests.rs` (in its alphabetical
place, the line only) and `pub(crate) mod remove_words;` in
`temper/crates/temper-client-view/src/forge.rs`. Forced edits per
scope.md. Calls, never changes: `forge::connect`, `forges::Row`, the
view's `look`, `confirm.rs` and `frame` helpers.

## Steps

D = `temper/crates/temper-client-domain/src`, V =
`temper/crates/temper-client-view/src`.

1. **The domain (D/forge/remove.rs)**, filling surface's stubs:
   - `act`: `RemoveForge { forge }` / `StartOver { forge }` on a kept
     row of the forges watch: `removal = Some(Removal { forge,
     start_over, phase: Confirming })` (replacing another confirming
     one); `CancelRemove`: `None` unless `Sending`; `ConfirmRemove`
     while `Confirming`, with a party and served not `OutOfDate`: send
     `ask(forge)` as `Request::Send { request, name:
     Some(remove_key(forge)), ask }` and go `Sending { request, progress:
     Parked }`.
   - `pub fn remove_key(forge: u16) -> skein_lib::RequestKey`:
     `remove-forge-N` under `crate::key::TOOLS`, as
     `first_repository_key` names its key, so the page and ctl
     (`cl-tools`) send one request.
   - `told`: the request's `Pending` progress (as `forges::told` maps
     it) and its `Answered`: `Removing` → `Removed`; `Refused(r)` →
     `Refused(r)`; `Unknown` → `Refused(Unknown)`. Returns `true` for
     the request's tells only.
   - Start over navigates nowhere: once the engine has ended the forge,
     the forges watch drops its row and `connect::step` gives the
     choices again for the same address (draft 6.4: "opens the step's
     choices again"). A `Removed` removal whose row the watch no longer
     lists is dropped at the next `act` or `told` (no edit of
     D/forge.rs: `cl-connect` holds it).
2. **The drawing (V/forge/remove.rs, V/forge/remove_words.rs)**, after
   `V/confirm.rs`'s look for a confirmation:
   - Not confirming this row: a button, "Remove" (`Meaning::RemoveForge`)
     or, with `start_over`, "Start over" (`Meaning::StartOver`).
   - Confirming: a group named "Remove FORGE?" ("Start over with
     FORGE?"), FORGE the row's name as `V/forge/connect.rs`'s `name`
     gives it; then what goes: "temper stops using FORGE now: work on
     its repositories is cancelled, its repositories leave their
     projects, and the tokens and keys temper holds for it are erased.";
     then what stays, by kind and mode:
     owned Forgejo "Forgejo on this machine goes with its data; its
     downloaded program stays for a next set up."; managed Forgejo "The
     bot account and webhooks temper made on FORGE stay there."; connected
     Forgejo "Nothing changes on FORGE itself."; GitHub "Your GitHub App
     stays yours: delete it in [its settings on GitHub]." linking to the
     row's `web` with `/settings/apps` (no link, the plain words, when
     `web` is none). Buttons "Remove" / "Start over"
     (`Meaning::ConfirmRemove`) and "Keep it" (`Meaning::CancelRemove`).
   - `Sending`: "Removing FORGE…"; `Refused(NotOffered)`: "The
     playground's forge leaves with the playground."; `Refused(Unknown)`:
     "FORGE is gone already."
   - Words in `remove_words.rs`, its own `Word` and `say` after
     `V/accounts/words.rs`'s pattern, phrase numbers under an owner no
     other table uses (check with `rg`).
3. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; one `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-journeys`. The journeys draw no removal: a reprint
   difference there is a bug of yours.
4. Old forms from `temper/`; commit and push.

## Report

- The phrase owner and numbers chosen.
- What the code taught for `client/forge.md` (its Forges page names no
  Remove today).
