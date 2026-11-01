+++
id = "pg-forge-remove"
part = "playground"
kind = "feature"
needs = ["pg-playground-pages", "pg-words"]
files = [
  "temper/crates/temper-client-domain/src/forge/remove.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/tests/forge_remove.rs",
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/tests/forge_remove.rs",
]
effort = "high"
minutes = 55
+++

## Goal

A forge's Remove on the Forges page, confirmed, its words naming what
goes and what stays on the forge (draft
`temper/docs/design/draft/playground-reset.md`, 4, "Its client" and "On
the forge itself"), and the forge's side's one removal request, which
the welcome's Start over (`pg-start-over`) sends too: `RemoveForge {
forge }` (`forge-asks`), its answer kept and shown.

Acceptance:

- each of the person's forge rows on Forges (`listed::rows`: not the
  playground's) has a button "Remove"; pressing it sends nothing and
  shows, in that row, an alert "Remove NAME?" (NAME as the row's name,
  `connect::name`), the paragraph `ForgeRemoveGoes`, the paragraph of
  what stays by the row's kind and mode (GitHub: `ForgeRemoveGitHub` and
  a link `ForgeAppSettings` to `https://github.com/settings/apps`;
  Forgejo owned: `ForgeRemoveOwned`; managed: `ForgeRemoveManaged`;
  connected: `ForgeRemoveConnected`; fake: `ForgeRemoveFake`), a button
  "Remove forge" and a button "Cancel";
- "Cancel" drops the confirmation; "Remove forge" sends one
  `w::Request::RemoveForge { forge }` (family request, deployment
  scope) and the row shows "Removing NAME…" with no buttons until the
  answer; one removal at a time across the side;
- answered `removing`: the side keeps the forge as removed until the
  `forges` watch drops its row (the subject lists no removing forge:
  contracts, 5), and draws nothing more for it; answered
  `remove_refused`: "Not removed: " with the refusal's words
  (`not_offered` and `role` as jig's `NotOffered` and `Role` refusals
  read, `crate::cards::refusal`; `no_such_forge` read as removed), the
  Remove button back; unknown or lost: the button back;
- `forge::remove::request(side, forge, limits, out)` is the one sender,
  callable by the welcome's Start over unconfirmed;
- the tests below pass, and every existing test of the packages changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1,
  D3, and 5).
- `temper/docs/design/draft/playground-reset.md`, section 4.
- `temper/docs/design/client/forge.md`, 5.4 (the Forges page's rows),
  `temper/docs/design/client/README.md`, 3.6 (confirmations).
- `temper/docs/plan/playground-reset-r4/tasks/forge-asks.md` (the
  request and its outcomes), `tasks/pg-words.md` (the words
  `ForgeRemove`, `ForgeRemoveTitle`, `ForgeRemoveGoes`,
  `ForgeRemoveOwned`, `ForgeRemoveManaged`, `ForgeRemoveConnected`,
  `ForgeRemoveGitHub`, `ForgeRemoveFake`, `ForgeRemoveConfirm`,
  `ForgeRemoving`, `ForgeRemoveRefused`, `ForgeAppSettings`; `Confirm`
  and `Cancel` exist).
- The pattern to copy: the Accounts page's inline confirmation
  (`temper-client-domain/src/pages/accounts/acts.rs`, `submit`'s
  `confirm`; `temper-client-view/src/pages/accounts/rows.rs`, the
  Remove button and its confirmation).

## Owns and may change

Owned: the `files` above (`forge/remove.rs` and the two test files are
new).

Neighbour files, edits allowed:

- `temper-client-domain/src/tests.rs`, `temper-client-view/src/tests.rs`:
  one `mod` line each.
- `temper-client-domain/src/route.rs`: none expected (forge acts reach
  the side through `Action::Forge`); if an answered key needs routing,
  the arm alone.

Pre-approved:

- the confirmation state is the side's `remove: Option<remove::State>`
  (`Confirming(u16)`, `Removing { forge, request }`, `Refused { forge,
  why }`, `Removed(u16)`); if the view needs it per row, an accessor
  `Side::removal(&self, forge: u16) -> Option<&remove::State>`;
- if `Answer` (the side's answer vocabulary) is the place outcomes are
  kept, add `Answer::Remove(RemoveAnswer)` there instead and say so.

## Steps

1. **Domain** (`forge/remove.rs`, new; `forge/form.rs`; `forge.rs`):

   ```rust
   //! A forge's removal from the client: one request at a time (draft playground-reset.md, 4).
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum RemoveRefusal { NotOffered, Role }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum State {
       Confirming(u16),
       Removing { forge: u16, request: skein_lib::Token },
       Refused { forge: u16, why: RemoveRefusal },
       Removed(u16),
   }
   pub fn request(side: &mut super::Side, forge: u16, limits: super::Limits, out: &mut skein_lib::Queue<super::Request>);
   // forge::Act:
   Remove { forge: u16 },        // asks for the confirmation
   ConfirmRemove,                // sends
   CancelRemove,
   ```

   The request's encoder and its answer's decoder live in `remove.rs`
   (`ask(forge, limits) -> Option<jig_client::Ask>`, `answer(value,
   limits) -> Option<Answer>`, as `forge/demo.rs`' `ask`/`answer` are
   written), not in `family.rs`, which another task edits meanwhile.
   `request` refuses (sends nothing) while a removal is out, without a
   signed-in party or on an out-of-date protocol (as the demo's start
   guards). A `Removed(n)` state is dropped once the rows no longer list
   `n`.
2. **View** (`forge/forges.rs`, `forge/bindings.rs`): in `personal`, the
   Remove button (`Meaning::Remove(u16)`), and when the side's removal
   names the row, the block of the acceptance (`Role::Alert` heading
   line, paragraphs, the link, `Meaning::ConfirmRemove`,
   `Meaning::CancelRemove`); `Removing` and `Refused` as said.
3. **Tests** (client domain and view unit tests):
   - `temper-client-domain/src/tests/forge_remove.rs`:
     `remove_asks_for_its_confirmation_and_sends_nothing`;
     `confirm_sends_one_remove_forge_at_the_deployments_scope` (decode
     the ask: `w::Request::RemoveForge` with forge 3; a second confirm or
     another row's Remove meanwhile sends nothing);
     `removing_is_kept_until_the_row_goes`;
     `refused_brings_the_button_back_and_no_such_forge_reads_removed`;
     `cancel_drops_the_confirmation`.
   - `temper-client-view/src/tests/forge_remove.rs`, rows from the
     existing wire-row helpers (read only, as `tests/connect_forge.rs`
     builds `w::ForgeRow`s):
     `each_kind_says_what_goes_and_what_stays` (a GitHub row: the
     goes paragraph, `ForgeRemoveGitHub`, the App settings link; an
     owned Forgejo: `ForgeRemoveOwned`; a managed one; a connected one);
     `removing_and_refused_are_said`;
     `the_playgrounds_row_has_no_remove`.
   - Narrow runs per file.
4. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; focused tests of both; `heavy cargo check --tests` for
   `temper-client-tools`, `temper-client-shell-terminal`,
   `temper-client-shell-web`, `temper-client-world`,
   `temper-client-gallery`, one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: `forge::remove` (its types and
`request`), the three acts, the view's `Meaning`s, `remove::ask` and
`remove::answer`, `Side::removal` if added.
