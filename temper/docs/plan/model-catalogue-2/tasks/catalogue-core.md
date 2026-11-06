+++
id = "catalogue-core"
part = "catalogue"
kind = "feature"
needs = ["model-surface", "fleet-bounds"]
files = ["jig/crates/jig-domain-core/src/settings/catalogue.rs", "jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/settings/change.rs", "jig/crates/jig-domain-core/src/settings/restart.rs", "jig/crates/jig-domain-core/src/settings/held.rs", "jig/crates/jig-domain-core/src/settings/fleet_needs.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/documents/catalogue.rs", "jig/crates/jig-domain-core/src/documents/small.rs", "jig/crates/jig-domain-core/src/runs/llm.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/tests/core/tests/suite/models.rs"]
effort = "high"
minutes = 300
+++

# The catalogue is the current settings, and a charter resolves against it

## Goal

`Settings.models` stops being a start's snapshot: it is the current `model` settings, by number, at every start, decision and release. A charter's models resolve against it, so a model added or changed reaches new work with no charter's change, while a task keeps the model it has. This is the plan's heart: 17.11, whole, on the core's side.

Acceptance, each a story in `jig/tests/core/tests/suite/models.rs`:

- `a_set_model_is_in_the_next_tasks_charter_with_no_charter_change`: with a charter of `rest: true`, a `Model(9)` set makes a revision whose new tasks can choose it, and the offer holds it.
- `a_task_keeps_the_model_it_has`: a live task's next run, after its model's prices and window change and after it's withdrawn, runs the record its revision's charter version holds.
- `a_model_a_person_turned_off_stays_off_across_a_release`: a `picker: false` change is over the default, and a start with different defaults keeps it; `a_release_adds_its_new_model_at_the_next_start`: a start with a new default makes a revision recorded as the release's, and it's in the catalogue and the offer.
- `a_pickers_flip_makes_no_charter_version`, `a_prices_change_makes_a_charter_version`, each counting versions.
- `a_withdrawn_model_leaves_the_offer_and_the_resolution`, with a named main dropped and the next one taking its place.
- `a_charter_left_with_no_model_is_refused_as_a_reference`.
- `a_number_no_setting_has_is_a_hole`: a `Model(64)` with stock at 1 and 2: no iteration offers rows 3 to 63, and a choice naming one is `LlmFault::Unconfigured` or `Charter`, never a run.
- `past_its_bound_a_charter_leaves_models_out`: with `settings.alternatives` at 2, the rest stops, and the document's `left_out` counts them, for a project's chat template's charter.
- `a_change_asks_the_root_to_tell_the_catalogue_before_its_commit`, and a start's restore asks once, with a person's model.
- Every focused test of `jig-domain-core`, `jig-domain-tasks`, `jig-domain-fleet`, `jig-domain-accounts`, and `jig/tests/{core,tasks,fleet}` passes, and fuzzy `jig/tests/core`.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.11 whole; 17.1, 17.2, 17.3, 17.7, 17.8.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 1 and 2.
- `jig/docs/design/domain/tasks.md`, 3; `authority.md`, 3.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. `model-surface` made `CatalogueModel`, `Value::Model`, `Ask::Catalogue`, `Charter.rest`, `Model::hole`, `valid_name`, and `Settings.endpoints`.

Readers and neighbours:

- `jig-domain-tasks/src/{domain,settings,llm}.rs`, `stored/fit.rs`: `MODELS` and `efforts`' meaning stays. A hole's count of efforts is 0, with its row off any charter, so `fits` refuses it. Comments saying "configured model" or "configuration's order" become "catalogue" or "number's position", where you touch them.
- `jig-domain-tasks/src/llm.rs`'s module doc: its first lines say configuration's order; bring them in line.
- `jig-domain-core/src/{settings.rs,reads/settings.rs,routing/settings/restore.rs,documents/settings_review.rs,runs/*.rs,tests/**}`: what they need to read `resolved` or the catalogue.
- `jig-protocol-store/src/tests/**`, `jig/testing/jig-core-kit/src/**`: fixtures alone. `jig-protocol-config/src/core.rs`: keep `Settings.models`' initial value as it is, and let `Core::new` replace it.
- `jig-client-domain/src/contract/me.rs`, the client's wire and link are `model-act`'s; `left_out` here is a field of the core's project document.

Options, each pre-approved:

- `Defaults` that name no model but give `Config.settings.models` (a test world): `Core::new` takes the settings' as defaults, numbered by position, `source: Stock`, so that those worlds run as they do. Name this in the report; `config-models` makes `jig-protocol-config` fill `Defaults.models`.
- Where a resolved charter must be held for `Core::charter` to answer a `&Charter`, keep them in a map on `Current` or beside it, rebuilt with the catalogue. Price it as one more whole set of charters in the worst case (17.8) and regenerate what that moves.
- A test that pinned a catalogue fixed for the process's life gets the rule's new form; one that can no longer say anything is deleted, and named in the report.
- If `check`'s `model_valid` still compares a charter's model record to the catalogue's whole, drop that comparison for the name alone: a charter's stored record is overridden by resolution.

## Steps

```rust
// settings/catalogue.rs
/// The catalogue the current settings leave, one row per number up to the highest (17.11).
pub(crate) fn table(current: &Current) -> Box<[Model]>;
pub(crate) fn entries(current: &Current) -> Box<[CatalogueModel]>;
// settings/charters.rs
pub fn resolved(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> Option<Charter>;
pub(crate) fn left_out(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> u32;
```

- `Core::new` and the end of a settings restore set `self.settings.models = table(..)` and rebuild the resolved charters, the tasks child's tables and the fleet's `Needs` from them (`settings/restart.rs:331,345`). `settings::fleet_needs::configured` resolves its charters likewise.
- A decision (`dispatch.rs:227`): `models = true` for a changed model item, as `charters` and `templates` are. Before its commit it computes the new table, sends `Ask::Catalogue`, and, for each charter whose resolution differs, does what a replaced charter does: its `Default` or `Changed` revision moves to the new one with its record written, and the old resolution is held (`held.rs`), counted by `full`. A model change that moves no resolution makes no version. `charters` then tells the tasks child and the fleet as it does.
- A release (`restart.rs`): a model default that differs counts as any setting's, and a resolution it changes as above.
- A model change marks every `me` (`dispatch.rs`, as a template's change does).
- `documents/catalogue.rs`'s `offer` skips holes and withdrawn; `documents/small.rs` gives `left_out`.

Each test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, and with `--profile fuzzy` on `jig/tests/core`'s package; `heavy cargo check --workspace --tests`. This changes what `Settings.models` and a charter's models mean to their readers, so `temper-engine-domain` and `temper-service`'s focused tests run too. Commit and push.

## Report

Where resolved charters live, and what they cost in the worst case. Pins regenerated. Every test deleted, with its rule.
