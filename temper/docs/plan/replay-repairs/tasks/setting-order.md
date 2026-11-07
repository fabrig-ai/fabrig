+++
id = "setting-order"
part = "walk-order"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-people/src/settings.rs",
  "jig/crates/jig-protocol-store/src/tests/intervals.rs",
  "jig/crates/jig-protocol-store/src/tests/keys.rs",
  "jig/crates/jig-protocol-store/src/tests/fixtures/keys.rs",
  "jig/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 60
+++

# A setting's derived order is the store's key order

## Goal

A box restarted after a goal's tree was cancelled panicked at `temper/crates/temper-engine-domain/src/loads.rs:383`, *startup continuation advances in walk order*. The cause the cut found: `jig_domain_people::Setting` derives `Model` before `Application` and `ProjectPolicy`, while the store's settings keys tag `Model` 6, after their 4 and 5 (`jig/crates/jig-protocol-store/src/keys/settings.rs`, `setting_write`). The store walks settings in byte order, so a startup page ending on an application's setting or a project's policy continues at a model's, which the domain's typed position reads as moving back. `jig/docs/design/protocol/store.md`, 4.1 requires bytes to sort as the derived order does, and its property test draws no settings key. Move `Model` last, as `notes/contracts.md` gives it; no stored byte changes.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4).

Acceptance:

1. Failing first: `domain_and_encoded_cursors_order_the_same_restart_walk` (`jig/crates/jig-protocol-store/src/tests/intervals.rs`) takes settings keys among its boundary keys in `cursor_keys()`: `Default` and `Changed` of `Charter(1)`, `Rules`, `Application { kind: 1, scope: 0, number: 1 }`, `ProjectPolicy { project: 1 }` and `Model(1)`, each as `Key::Core(CoreKey::Settings(..))`. It fails before the fix and passes after.
2. `settings_subfamilies_and_history_follow_their_domain_order` (`tests/keys.rs`) gains the rows `Default(Application { .. })`, `Default(ProjectPolicy { .. })`, `Default(Model(1))` and `Changed(Model(1))` in byte order, and asserts beside its byte order that each key's derived order follows its predecessor's, as its name says; it fails before the fix on the model rows.
3. `Setting` is ordered as `notes/contracts.md` says, with no field or doc line changed but where the move needs it.
4. The entry *4.1: a settings key's bytes sort as its derived order does but for `model`...* is gone from `jig/docs/design/protocol/store.status.md`.
5. This changes what `Setting`'s order means, so its readers' tests run: focused tests pass in `jig-domain-people`, `jig-domain-core` (`--lib`), `jig-protocol-store`, `jig-protocol-engine-clients`, `jig-protocol-tools`, `jig-core-world` and `jig-conformance-world`, and `heavy cargo nextest run --profile fuzzy -p jig-core-world -E 'test(/^fuzzy::settings/)'`; `heavy cargo check --tests -p temper-engine-domain -p temper-service -p temper-engine-domain-world` compiles.

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `temper/docs/plan/replay-repairs/notes/contracts.md`, *`Setting`'s order*: it binds.
- `jig/docs/design/protocol/store.md`, 4.1 (*Bytes sort as the domain's keys do*) and 7.2 (the walk's `(interval, key)` order).
- `jig/docs/design/domain/engine.md`, 17.1 (what a setting is).

## Owns and may change

**Owned:** the `files` above.

**Readers** (assertions only: an expected list or order of settings that puts a model's setting before an application's or a project's policy follows the new order; no other edit): jig's tests under `jig/crates/*/src/tests/`, `jig/crates/jig-domain-core/src/settings/tests.rs`, `jig/tests/core/tests/**` and `jig/tests/conformance/tests/**`, and `jig/testing/*` referees whose expected orders are built by hand. The survey found the enum read by name throughout, never by position, so expect few.

**Not yours:** temper's tests whose expected order moves. `restart-story` owns them and runs after this task; list any you see red under `## Left`, with the test and the order it expected.

**Options, each with its default:**

- `tests/fixtures/keys.rs`'s generator draws no settings key, which is how 4.1's property test missed this. Default: if it draws from a list of key shapes, add `CoreKey::Settings` keys of each subfamily there too, keeping its sample count; if that would reshape the generator, leave it and say so.
- A jig test that compares a rendered settings inventory or a golden whose order moves: regenerate it by its own tool, and say so.
- If any production code depends on `Model` preceding `Application` or `ProjectPolicy` (a range bounded by them, a `<` between settings): stop with `Question:`, the site and what it does.

## Steps

1. Acceptances 1 and 2, then run them and see them fail on the model keys: `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/domain_and_encoded_cursors_order_the_same_restart_walk|settings_subfamilies_and_history_follow_their_domain_order/)'`.
2. Move `Model`; both pass.
3. The readers' assertions (acceptance 5's packages), then the status entry.
4. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, `heavy cargo clippy -p jig-domain-people -p jig-protocol-store --all-targets -- -D warnings`, acceptance 5, commit and push.

## Report

As the implementer's text says; also each reader's assertion moved, and whether the generator now draws settings keys.
