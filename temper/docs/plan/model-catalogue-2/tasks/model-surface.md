+++
id = "model-surface"
part = "setting"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-people/src/settings.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/catalogue.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/settings/change.rs", "jig/crates/jig-domain-core/src/settings/models.rs", "jig/crates/jig-domain-core/src/runs/state.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-protocol-store/schema/**", "jig/crates/jig-protocol-store/src/generated/**", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/keys/settings.rs", "jig/crates/jig-protocol-store/golden/**", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-client-protocol-wire/schema/**", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/golden/**"]
effort = "high"
minutes = 200
+++

# `model` is a kind of setting, and a charter can say "the rest"

## Goal

The surface every other task of this part builds on: the kind, its value, `Charter.rest`, `Model.provider` and `Defaults.models`, in the core, the store's schema and the client's wire, with each match in the workspace taking its arm. It is inert: a `model` item is checked, stored and read like any setting's, but `Settings.models` still comes from the start and no charter is resolved. Those are `catalogue-core`'s.

Acceptance:

- `contracts.md`, 1's types and fields exist with those names, and 4's `Setting::Model`, `SettingValue::Model`, `CatalogueEntry`, `SettingModel.provider` and `Charter.rest`, with its `ModelSource`. `Ask::Catalogue` exists and nothing sends it; each root drops it.
- An owner's `ChangeSettings` setting `Model(9)` to a whole value makes a revision, survives a restart and reads back whole; it shows in the history and a proposal's items, in every form they're encoded in.
- Refused: a `Model(0)` or `Model(65)`; a name out of `valid_name`; a set naming another name than its number's default or stored change; a provider or endpoint that doesn't exist; a window or output of 0. Each is `Fault::Shape` or `Fault::Reference`, by 17.1.
- `Edit::Removed` of a model stores its current value with `withdrawn: true`; `Edit::Reset` of one with no default is `Fault::Shape`.
- `Value::Charter` round-trips `rest` in store and wire; `Model.provider` is filled from the provider's `models` at `jig-protocol-config`'s parse, and `Core::provider` reads it, with `accounts::Providers::serving` and `Provider.models` gone.
- The store's stored value holds the provider, and its schema and generated code change in place with one set of goldens, as `jig/docs/design/protocol/store.md`, 5 says.
- `heavy cargo check --workspace --tests` compiles, and focused tests of every package touched pass.

## Inputs

- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 1, 2 (`Ask::Catalogue` alone) and 4.
- `jig/docs/design/domain/engine.md`, 17.1, 17.11, and 17.2 for the decision.
- `jig/docs/design/protocol/store.md`, 4.4 and section 5.
- `jig/docs/design/client/domain/contract.md`, 3.2, for `ChangeSettings`, which this task doesn't change.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files.

The survey's list of what a kind touches is your checklist; add the arm each needs, with a real body where this task's acceptance reads it (store, wire, check, read, history, proposals) and a plain one elsewhere:

- **jig's core:** `jig-domain-tasks/src/proposals.rs:199`, `person_proposals.rs:181`; `jig-domain-core/src/settings/{application,restart,charters,fleet_needs,held}.rs`, `routing/settings/dispatch.rs:227`, `dispatch/project_policy.rs:28,103`, `routing/settings/restore.rs:170,205`, `reads/settings.rs:149`, `documents/settings.rs:168`, `documents/settings_review.rs:36`, `limits.rs:90,107,238` (a whole set prices 64 models; a revision two sets; `writes_bound` its capacity plus 64).
- **jig's config:** `jig-protocol-config/src/core.rs:12` (`Defaults.models` from `llm.models`, numbered by position, `source: Stock`; `Model.provider` from the provider that names it), `domain.rs:338,354`, `llm.rs`, `llm/providers.rs`, `charters.rs` (`rest` read as the member, default false, and true with no `models`).
- **jig's client and protocol:** `jig-client-protocol-link/src/translate/settings.rs:15,335`, `jig-protocol-engine-clients/src/translate/settings.rs:37,335`, `translate/settings/change.rs:59,69`, `translate/llm.rs:412`, `translate/refusals.rs:421`, `jig-client-view/src/words/words/refusals.rs:236`, `jig-protocol-tools/src/decode.rs:255`.
- **jig's store:** `schema/{settings-v1,people-v14,tasks-v9}.schema`, `keys/settings.rs:53,97` (a new permanent tag, existing ones kept), `codec/settings.rs:79,399,1940`, `codec/people/outcomes/facts.rs:836`, `codec/tasks/proposals.rs:789`, `src/tests/**`.
- **jig's kits:** `jig-client-scripted-engine/src/codec/values/settings.rs:39,141`, `jig-client-test-records/src/{settings,settings_proposals}.rs`, `jig-test-domain/src/lib.rs:1304`, `jig-core-kit/src/peers.rs:548`, `jig-core-kit/src/conformance/naming.rs:108`.
- **Every `Model { .. }` literal** gains `provider`, and `Charter { .. }` `rest: false`: about 105 files in jig and temper, literal only, by the required-field exception. A fixture helper may absorb many.
- **temper, by the required-field exception,** an arm only, with the setting's name "Model N" where one is printed; `temper-settings-form` adds the real form, `temper-client-tools/src/jig/settings_adapter.rs:612,622,851,860`, `jig/settings_file.rs:16`, `jig/render/outcome.rs:456`, `jig/render/document/{settings,proposal}.rs`, `temper-settings-form/src/{file,render,wire}.rs`, `temper-client-view/src/cards/proposal.rs:719`, `temper-engine-smith/src/settings.rs:72`, `temper-engine-domain` and `temper-service`'s roots dropping `Ask::Catalogue`, `jig`'s reference root likewise.

Options, each pre-approved:

- `Defaults` is built in many fixtures: give it `models: Box::default()` where they don't name one, and let `jig-protocol-config` fill it.
- Should a generated module's size push a crate past a lint or an old-form count, take the scoped `#[expect]`; a raised count stays yours.
- Stored `Default` and `Changed` records keep their `models` vocabulary: `settings::remap`'s arm for `Value::Model` does nothing, since it holds no index.
- `remap`, `moves` and `ModelName` stay. Don't simplify them here.

## Steps

```rust
// settings/catalogue.rs
//! The model setting's value and shape (jig/docs/design/domain/engine.md, 17.1 and 17.11).
pub enum ModelSource { Stock, List, Person }
pub struct CatalogueModel { .. }           // contracts.md, 1
#[must_use] pub fn valid_name(name: &[u8]) -> bool;
pub(crate) fn shape(value: &CatalogueModel, number: u32, settings: &Settings) -> Result<(), Fault>;
impl Model { pub fn hole() -> Self; pub fn is_hole(&self) -> bool; }   // in runs/state.rs
```

`Core::provider` keeps its signature and returns `Some(model.provider)` where it is configured. `Settings` gains the document's endpoints' facts, `endpoints: Box<[EndpointFacts]>` (number, dialect, program), which `shape` checks a model against, filled in `jig-protocol-config/src/core.rs`.

Tests, each breaking its rule once by hand:

- In `jig-domain-core/src/settings/tests.rs`: `a_model_setting_is_checked_whole`, `a_number_keeps_its_name`, `removing_a_model_withdraws_it_and_keeps_its_number`, `resetting_a_model_with_no_default_is_out_of_shape`, `valid_name_takes_the_shapes_a_provider_names_a_model_by`.
- In `jig/tests/core/tests/suite/settings.rs`: `a_model_setting_survives_a_restart_and_reads_back_whole`.
- In `jig-protocol-store/src/tests/settings.rs`: a round trip of `Value::Model` and `Charter.rest`; in `jig-protocol-engine-clients/src/tests/settings.rs`: of `SettingValue::Model`.

Regenerate each schema's code and goldens with its own generator, never by hand. `cargo fmt`; each tree's `old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents; `heavy cargo check --workspace --tests`. This changes what `Providers::serving`'s readers read, so run `jig-domain-accounts`' and `jig-domain-core`'s whole focused suites. Commit and push.

## Report

Any arm you gave a plain body that a later task should read again, by file. The goldens regenerated.
