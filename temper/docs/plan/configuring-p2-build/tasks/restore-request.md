+++
id = "restore-request"
part = "restore"
kind = "feature"
needs = ["policy-change", "settings-store", "restore-connector", "restore-forge"]
files = ["jig/crates/jig-domain-core/src/routing/settings/restore.rs", "jig/crates/jig-domain-core/src/routing/settings/**", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/**", "jig/crates/jig-domain-core/src/reads/settings.rs", "jig/crates/jig-domain-core/src/effects.rs", "jig/crates/jig-domain-core/src/routing/party/**", "jig/crates/jig-domain-people/src/domain/applications.rs", "jig/crates/jig-domain-core/src/tests/settings/restore.rs", "jig/crates/jig-domain-core/src/tests/settings.rs"]
effort = "high"
minutes = 150
+++

# restore-request: `Restore { revision, reason }` proposes the settings as a revision left them

## Goal

The core admits a party's `Restore` (contract B4): it loads the history's records after R a
page at a time, folds what each item replaced into one proposal of settings against the
newest, and proposes it through the usual admission, an application's items taken from their
connectors' kept values (`Ask::RestoreSettings`), a deleted project's settings and policy left
out and named (jig's `domain/engine.md`, 17.10). It also erases what forgotten revisions'
items replaced, at their connectors (`Ask::ForgetReplaced`).

Acceptance:

- `Restore { revision: R, reason }` from a party who may propose settings answers
  `Outcome::Restored { proposal, to, left_out }`; the proposal's items are, for each setting a
  revision after R changed, the earliest such revision's `replaced`: `Value(v)` as *set* v,
  `Default` and `NotOffered` as *reset*, `Removed` as *removed*; for each project whose policy
  a later revision set, `ProjectPolicy` *set* to the policy it replaced; its reason
  `restores revision R: ` and the party's reason. An application's items reach their connector
  as `RestoreSettings` (a *set* with the revision whose record replaced it), and the proposal is
  kept with the sizes and charters the connector answered.
- Refused: forgotten (R below the history's oldest kept: `Refusal::Forgotten { oldest }`);
  empty (R the newest, or nothing to restore: `Malformed::SettingsUnchanged`); stale (a
  revision made while it loads, and R 0 or past the newest: `Refusal::Stale { newest }`); too
  large (items past a whole set or more than one project's policy: `Malformed::Settings {
  setting, fault: SettingFault::Count }`); a connector's refusal of an item (`Refusal::Judged`,
  as a change's is), after which the core sends `DropProposalSettings`. One restore loads at a
  time; a second while one loads is refused busy (the refusal a settings request gets while
  the core is busy with another, or, if none exists, `Refusal::Stale { newest }`: say which).
- A revision's record erased from the history sends `ForgetReplaced { before: <the new
  oldest> }` to each connector owning a kind, in the same decision.
- The unit tests below pass with `jig-domain-core`'s and `jig-domain-people`'s focused tests.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.6, 17.7 (the history), 17.8 (bounds), 17.10 (whole).
- `jig/docs/design/domain/connectors.md`, 17 ("What its items replaced, for a restore").
- `jig/docs/design/domain/people.md`, 5.1 and 5.2.
- `jig/docs/design/client/domain/contract.md`, 3.2 (`Restore`) and 6 (`revisions`' paging).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B2, B3, B4; `notes/common.md`.
- Old code, jig's `configuring-revisions/restore` (unreachable from your slot, quoted):
  `jig-domain-core/src/routing/restore.rs` (`start`, `load`, `loaded`, `page`, `propose`,
  `reason`), `settings.rs`' `Restoring` and `restore_count`; `Restore { revision, reason }` on
  the people boundary, `OutcomeRestored { proposal, to, left_out }`, `RefusalForgotten {
  oldest }`. It paged the history newest to R+1, folded the records newest first into a map so
  the earliest after R won (`Value` to `Set`, `Default` and `NotOffered` to `Reset`, `Removed`
  to `Removed`), and proposed through the usual admission, the reason "restores revision R:
  ...". Refusals: busy (one restore in flight), unknown (0 or past the newest), forgotten,
  empty, stale (a revision made while it loads), too large as `Malformed::Settings { fault:
  Count }`. It departed from 17.10: no application's items, no project's policy, no deleted
  project's items, too large counted on `Set` items alone; this task does all four.

## Owns and may change

- Owned: `jig-domain-core/src/routing/settings/restore.rs` (new: the request's start, the
  pages' loading and folding, the proposal), the rest of `src/routing/settings/**`
  (`dispatch.rs`' admission and proposal paths it calls), `src/settings.rs` and
  `src/settings/**` (`Restoring`, the history erasure in `restart.rs`' revision writes sending
  `ForgetReplaced`), `src/reads/settings.rs` (the page loads it shares with `revisions`),
  `src/effects.rs` (`Event::SettingsRestored`'s handling), `src/routing/party/**` (routing
  `people::Ask::Restore` to it); `jig-domain-people/src/domain/applications.rs` (the
  admission of `Restore` as a settings `Propose` is); the unit test file
  `jig-domain-core/src/tests/settings/restore.rs` and its `mod` line.
- Every `unreachable!("a restore: restore-request")` arm gets its body; the others stay.
- Status: remove from `jig/docs/design/domain/engine.status.md`'s **Not yet** "17.10:
  restoring a version: no `Restore` request, and a revision's record keeps no item's replaced
  value (settings-v1's `Revision` and `Item`)." (both are built once you and `settings-store`
  have); leave "17.5, 17.10: a connector keeps no value its items replaced" to
  `restore-story`.
- Options, pre-approved:
  - **A deleted project:** its settings (an application's items of its scope) and its
    `ProjectPolicy` are left out and named in `left_out`, never proposed; a project is
    deleted when it is not live now.
  - **Too large first:** the bound (a whole set, one project's policy) is counted over every
    item the records give, a deleted project's included, before any is left out (contract
    B4), so `left_out` never passes the store's bound.
  - **Who may ask:** whoever may `Propose` settings (the people child's admission of
    `Propose { action: Settings }`); the proposal goes to the owners of its items as any
    proposal of settings (and, with a project's policy, to a party who owns both, as
    `policy-change` built).
  - **Busy:** if no refusal fits a second restore while one loads, use `Refusal::Stale {
    newest }` and say so.
  - If the records' pages are read through the store (an effect and its answer), reuse
    `reads/settings.rs`' `load` and its answer path as `revisions` does; if they are in the
    core's memory, fold them directly; say which.
  - The roots already route `RestoreSettings` and `ForgetReplaced` to their connectors
    (`restore-connector` in jig's test domain, `restore-forge` in temper): the worlds' tests
    see the new asks answered. Unit tests answer the ask by hand.

## Steps

1. `routing/settings/restore.rs`: `pub(crate) fn start(core, env, party, revision, reason) ->
   ...`, the load of every page after R (at most the history's limit), `loaded(...)` per page,
   the fold earliest-wins, `propose(...)` building the items and the reason, then the usual
   proposal admission (`propose_admitted`), with the application's items through
   `RestoreSettings` and its answer `Event::SettingsRestored`.
2. `restart.rs`' history erasure: `connector::Ask::ForgetReplaced { before }` to each kind's
   connector beside the erasure writes.
3. Tests, `jig-domain-core/src/tests/settings/restore.rs`, in the style of `application.rs`
   and `application_proposals.rs`:
   - `restoring_a_revision_proposes_what_later_revisions_replaced`
   - `a_restore_reads_every_page_of_the_history_after_r`
   - `a_restore_of_a_forgotten_revision_names_the_oldest_kept`
   - `a_restore_of_the_newest_is_empty`
   - `a_revision_made_while_a_restore_loads_refuses_it_stale`
   - `a_restore_past_the_whole_set_is_refused_too_large`
   - `a_restore_proposes_a_project_policy_it_replaced`
   - `a_restore_leaves_out_and_names_a_deleted_projects_items`
   - `a_restore_asks_the_connector_for_an_applications_replaced_values` (the ask's items, and
     the proposal kept with the answered sizes)
   - `erasing_a_revision_from_the_history_tells_connectors_to_forget_what_it_replaced`
4. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E
   'test(/settings::restore/)'` (check the count), then `-E 'test(/settings/)'`; `heavy cargo
   nextest run -p jig-domain-people --lib`. The history erasure sends a new ask, a change of
   what the core emits: also the focused tests of `jig-domain-core`, `jig-test-domain` and
   the core world (`jig/tests/core`). Then `cargo fmt`; `heavy cargo clippy -p
   jig-domain-core -p jig-domain-people --all-targets -- -D warnings`. Commit and push.

## Report

How the pages load (the option taken); the refusal used for busy; what `left_out` names; the
tests and their counts.

## Answers

### 2026-11-03: May restore-request replace its named placeholder arms outside the allowed file list? Options: (A) authorize those arms and necessary associated handling in the files below (recommended); (B) assign those files to another task and provide that prerequisite before this task continues. (configuring-p2-build/restore-request-1, foreman configuring-p2-build/restore-request/foreman-1)

restore-request owns replacing every `unreachable!("a restore: restore-request")` arm, including those outside the explicit files list.
The spec’s “Owns and may change” explicitly says: “Every `unreachable!("a restore: restore-request")` arm gets its body; the others stay.”
Its cited `temper/docs/plan/configuring-p2-build/notes/common.md` confirms: “the named later task replaces it with a body; every other task leaves it as it is.”
All eleven reported paths contain such named arms in this checkout.
The next run implements those bodies and the listed owned work, then runs the specified checks, commits and pushes.
This settles the named placeholder replacements; it grants no blanket authorization for unrelated edits or skeleton changes.
