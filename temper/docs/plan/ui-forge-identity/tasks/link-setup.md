+++
id = "link-setup"
part = "link"
kind = "feature"
needs = ["link-record"]
files = [
  "temper/crates/temper-forge-setup/src/given.rs",
  "temper/crates/temper-forge-setup/src/tests/given.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/tests/setup_lifecycle.rs",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 70
+++

# The setup host tells the forge what it read of the person

## Goal

skein's plans already answer who the person is (`ui-forge-setup`): a Forgejo `Reconcile` given a login answers `Person { id }`, and each GitHub `GET /app` answers the App's account. temper gives no login and drops the answers. Give an owned Forgejo's `Reconcile` its person's login, and turn each answer into `domain::Event::PersonOf { forge, answer }` for the forge connector, deciding GitHub's from the forge's `AppMade` and `AppId` items, as `temper/docs/design/protocol/setup.md`, 5.2 and 17.2 and `temper/docs/design/domain/forge.md`, 4.6 say. The connector decides what the answer does (`link-decide`); this task only says it.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), because the design already says what this task builds.

Acceptance:

1. `temper-forge-setup`'s `given::values` gives `person: Some(login)` for an owned entry, the person's login as `org_owner` already reads it, and `None` for managed and fake: `an_owned_setup_maps_every_given_value` (`src/tests/given.rs`) expects it, and the managed and fake tests still expect `None`.
2. The setup host's `done` (`temper-service/src/setup/lifecycle.rs`) pushes `root::Event::Forge(Box::new(domain::Event::PersonOf { forge, answer }))` (as `BotLogin` is pushed today), once per run, for:
   - a local `Reconcile` (`Kind::LocalReconcile`) ending `Person { id: Some(id) }`: `Is(id)`; `Person { id: None }`: `Gone`; `Nothing`: no event;
   - a `Convert` ending `Made { owner }`, and a `Paste` or GitHub `Reconcile` ending `Lacking { owner, .. }`: `Is(owner.id)` when the forge's `AppMade` item equals its `AppId` item and `owner` is `Some` with `kind` `Account::User`; else `NotTheirs`;
   - an admin `Reconcile` (a managed Forgejo, given no login): no event.
   Each pushed before the arm's own work (the `Origin` keep, the installations), which stays as it is.
3. Rows of the existing lifecycle tests hold it (`temper-service/src/tests/setup_lifecycle.rs`, `setup.rs`): `kept_local_records_wait_for_restore_done_and_origin_before_connecting_as_the_recorded_bot` (or the local-reconcile test nearest it) sees `PersonOf { Is }` for an owned entry; `conversion_keeps_origin_then_installs_answers_the_browser_and_connects_from_records` sees `Is` for a `User` account and `NotTheirs` for an `Organization`; `paste_reads_the_carried_pem_keeps_identity_then_origin_connects_and_erases_after_root_decides` sees `NotTheirs` (a paste writes no `AppMade`); `a_real_reconcile_prunes_accepted_after_kept_and_never_connects_or_writes_origin` sees `Is` with a kept `AppMade` equal to `AppId`, `NotTheirs` without. Where a test is no table, add its new case as one more assertion on the events it already reads, or make it a table of two rows; a new test only if none of these reaches the arm. Each rule bites once (compare `AppMade` with nothing; see a `NotTheirs` row fail).
4. The entry *5.2, 5.3, 17.2: temper gives `Reconcile` no person's login and keeps no link: ...* is gone from `temper/docs/design/protocol/setup.status.md`.
5. Focused tests pass in `temper-forge-setup` and `temper-service`; clippy clean for both.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The link* (`PersonOf`, `PersonAnswer`): it binds.
- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 5.2 (the person's login, the given value's row) and 17.2 (*The App's account sets the forge's link to its person*); `temper/docs/design/domain/forge.md`, 4.6.
- `temper/docs/plan/ui-forge-setup/notes/contracts.md`, *The plans' answers*: the outcomes as skein gives them.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** `temper-service/src/setup.rs`'s `read_item` and `engine.forge_records.rows` are read, not changed. Another test that counts the root events a setup run pushes, and now sees one more (`rg 'root::Event::Forge' temper/crates/temper-service/src/tests`): add the `PersonOf` to what it expects, and name it in the report.

**Options, each with its default:**

- **Reading `AppMade` and `AppId`:** default, from the forge's loaded rows (`engine.forge_records.rows.forge(forge)`, `rows.get(Item::AppMade)` and `Item::AppId`), after the run's writes are kept. If at `done` a `Convert`'s writes aren't yet in those rows, compare from the items the run wrote, as the lifecycle keeps them, and say so. Compare the bytes; both decode by skein's `item::app_id` and `item::app_made`, and an unreadable one is `NotTheirs`.
- **The owner's type:** `skein_forge::github::document::Account::User`; any other is `NotTheirs` (`ui-forge-setup`'s contract).
- **A helper:** one private function in `lifecycle.rs`, `app_answer(engine, forge, owner) -> domain::PersonAnswer`, used by the three GitHub arms. Default as said.
- **A run that fails** pushes nothing: only `Done` arms send.

## Steps

1. `given::values`' owned arm; its test.
2. `lifecycle.rs`: the local-reconcile arm, then the three GitHub arms through `app_answer`.
3. The rows (acceptance 3); the bites.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-forge-setup -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-forge-setup -p temper-service`. Commit ("temper-service: tell the forge who the setup read the box's person to be"), push.

## Report

As the implementer's text says, with each bite's broken rule and failure.
