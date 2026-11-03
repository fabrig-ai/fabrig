+++
id = "restore-connector"
part = "restore"
kind = "feature"
needs = ["restore-surface", "connector-replaced"]
files = ["jig/testing/jig-test-connector/src/**", "jig/testing/jig-test-domain/src/routes.rs", "jig/testing/jig-test-domain/src/lib.rs", "jig/testing/jig-test-domain/src/client_documents.rs", "jig/examples/ops/domain/src/route.rs", "jig/testing/jig-test-store/**"]
effort = "high"
minutes = 120
+++

# restore-connector: jig's test connector answers a restore's items and forgets what is erased

## Goal

`jig-test-connector` answers the two asks `restore-surface` declared (contract B3): a
restore's items, kept as the proposal's from the values it kept as replaced and answered with
their sizes and charters, and `ForgetReplaced`, erasing the replaced values of revisions the
history no longer keeps. jig's test domain routes both, as it routes `JudgeSettings` and
`KeepSettings` (jig's `domain/connectors.md`, 17).

Acceptance:

- `Ask::RestoreSettings { owner, proposal, items }` reaches the test connector through
  `jig-test-domain`'s routes as `connector::Event::RestoreSettings { token, proposal, items }`;
  the connector keeps each item as the proposal's (as `keep_proposal` does): a `Set {
  revision }` with the value kept as replaced by that revision, `Remove` and `Reset` as they
  are; and answers `connector::Request::SettingsRestored { token, judged }`, one
  `SettingJudged` per item in order (`Accept { size, charters }`; `Refuse { why }` where the
  value is past its kind's bounds or no replaced value is kept for it), which the route turns
  into `core::effects::connector::Event::SettingsRestored { owner, judged }`.
- `Ask::ForgetReplaced { before }` reaches it as `connector::Event::ForgetReplaced { before }`;
  it erases every replaced value and record of a revision below `before`.
- `jig/examples/ops/domain`'s route gives both asks the arm its other settings asks have.
- The unit tests below pass with `jig-test-connector`'s, `jig-test-domain`'s and
  `jig-test-store`'s focused tests.

## Inputs

- `jig/docs/design/domain/connectors.md`, 17 ("What its items replaced, for a restore");
  `jig/docs/design/domain/engine.md`, 17.10.
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B3; `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/connector-replaced.md`, which this builds on
  (`State.replaced`, `Record::SettingReplaced`).

## Owns and may change

- Owned: `jig-test-connector/src/**` (`boundary.rs`' two events and the request,
  `settings.rs`' `restore` and `forget_replaced`, `domain.rs`' dispatch, `src/tests/settings.rs`);
  `jig-test-domain/src/routes.rs` (the two asks and the answer, replacing
  `unreachable!("a restore: restore-connector")`), `src/lib.rs` and `src/client_documents.rs`
  (the new request's arms where `SettingsJudged`'s are); `jig/examples/ops/domain/src/route.rs`
  (its arm); `jig-test-store/**` only if the connector's records need a new key (a proposal's
  item from a replaced value is a `SettingProposal` record as today: none expected).
- Neighbour files, the new event's and request's arms only, wherever the compiler names a
  match on the test connector's `Event` or `Request` (`jig/tests/test-connector/src/world.rs`,
  `jig/tests/core/src/peers.rs` among them).
- Options, pre-approved:
  - If `jig-test-domain` keeps the judge's owner by a token map (`settings_judges`), reuse it
    for the restore's owner; if a second map is cleaner, add it beside and say so.
  - If no replaced value is kept for an asked revision (an inconsistency of the core's), answer
    `Refuse { why: b"no value kept" }` and keep nothing for that item.

## Steps

1. `boundary.rs`: `Event::RestoreSettings { token: Token, proposal: u64, items:
   Box<[RestoredItem]> }` with `RestoredItem { kind, scope, number, edit: RestoredEdit }`
   (`Set { revision: u64 }`, `Remove`, `Reset`); `Event::ForgetReplaced { before: u64 }`;
   `Request::SettingsRestored { token: Token, judged: Box<[SettingJudged]> }`.
2. `settings.rs`: `pub(crate) fn restore(domain, token, proposal, items, out)` and
   `pub(crate) fn forget_replaced(domain, before, out)`; `domain.rs` dispatches them.
3. `jig-test-domain/src/routes.rs`: the two asks translated (the owner kept by token), and
   `Request::SettingsRestored` translated back to the core's event, as `SettingsJudged` is.
4. Tests, `jig-test-connector/src/tests/settings.rs`:
   - `a_restores_items_are_kept_from_the_values_replaced_and_answered_with_sizes`
   - `a_restore_item_past_its_kinds_bounds_is_refused`
   - `forgetting_erases_the_values_replaced_by_older_revisions_only`
   and one in `jig-test-domain`'s tests beside its settings routing tests (find them with
   `rg JudgeSettings jig/testing/jig-test-domain/src/tests`):
   `a_restore_ask_reaches_the_connector_and_its_answer_reaches_the_core`.
5. Narrow tests: `heavy cargo nextest run -p jig-test-connector --lib -E 'test(/settings/)'`
   (check the count); `heavy cargo nextest run -p jig-test-domain --lib`. Then `cargo fmt`;
   `heavy cargo clippy -p jig-test-connector -p jig-test-domain --all-targets -- -D
   warnings` (and the ops example's package); the focused tests of those packages and of
   `jig/tests/test-connector`'s. Commit and push.

## Report

The routing of the owner; the records a restore writes; the tests and their counts.
