+++
id = "config-models"
part = "catalogue"
kind = "feature"
needs = ["model-surface"]
files = ["jig/crates/jig-protocol-config/src/llm.rs", "jig/crates/jig-protocol-config/src/llm/providers.rs", "jig/crates/jig-protocol-config/src/defaults.rs", "jig/crates/jig-protocol-config/src/core.rs", "jig/crates/jig-protocol-config/src/clients.rs", "jig/crates/jig-protocol-config/src/charters.rs", "jig/crates/jig-protocol-config/src/members.rs", "jig/crates/jig-protocol-config/src/profile/**", "jig/crates/jig-protocol-config/src/tests/**", "jig/crates/jig-client-protocol-wire/schema/**", "jig/crates/jig-client-protocol-wire/src/generated/**"]
effort = "high"
minutes = 120
+++

# jig's configuration takes passed models, and a provider names its endpoint

## Goal

An application that passes defaults passes its models, as it passes its charters; a provider names the endpoint its added models take. Under passed defaults the document's models still parse, so that temper can move in its own task, which then makes them refused. The profiles' charter bound rises to the catalogue's.

Acceptance:

- `pass` takes `Defaults.models`: `Llm.models` is what it passes, and `Engine`'s readers of the catalogue read them.
- A provider's `endpoint`, naming one of `endpoints`, parses, and a name no endpoint has is `What::Refers` at its path; `jig_protocol_config::Provider.endpoint: Option<Box<[u8]>>`.
- `core.settings.alternatives` is 63 in `small` and `standard`, the client's wire takes 63, and `charter_alternatives_fit_the_wire` holds. A resolved charter of 64 models fits `run_bytes` at `standard`, tested.
- `Defaults::Document`: the `llm` models are `Defaults.models` numbered by position, `source: Stock`, `picker` as given, a `shown` and `description` as given. A charter with `"rest": true`, and one with no `models`, read as `rest`.
- The `Shape` rows in `members.rs` say so, and `temper/docs/help/reference/*` regenerates by its test's writer.
- Focused tests of `jig-protocol-config`, `jig-client-protocol-wire`, and `temper`'s reference generators pass.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 3: `llm`, `charters`, and the paragraph on a model's provider.
- `jig/docs/design/domain/engine.md`, 17.1, 17.7 and 17.11.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 6, and 4 for the wire's ceiling.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. Readers, call sites and literals: `jig-domain-core/src/lib.rs` (`Settings.endpoints`' values), `jig-protocol-config/src/{templates,projects,domain}.rs`, temper's fixtures that write a provider, if a test parses them strictly. Regenerate `temper/docs/help/reference/*` by `temper/crates/temper/src/tests/configuration.rs`'s writer.

Options, each pre-approved:

- Keep `Llm.models`' parallel arrays only if `core.rs` and `clients.rs` read them more simply through `Defaults.models`; otherwise they go.
- `endpoint` stays optional in this task. `stock-models` requires it under `Defaults::Passed`, where it also refuses a document's `models`.
- A pin that moves with the alternatives' bound (the memory's worst case, a counted baseline) is regenerated and named.

## Steps

`Defaults.models: Box<[CatalogueModel]>` reaches `Config` through `core.rs`, replacing the literal `Settings.models`' input; `core.rs` builds them under `Document`. A test each, breaking its rule once by hand: `a_providers_endpoint_must_name_one_of_its_endpoints`, `passed_models_are_the_catalogue`, `a_documents_models_are_numbered_by_position`, `a_charter_with_no_models_takes_the_rest`, `a_resolved_charter_of_64_models_fits_run_bytes`.

`cargo fmt`; each tree's `old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents. Commit and push.

## Report

Pins that moved, old and new. Whether a resolved charter of 64 fits `small`'s `run_bytes`, and what it does when it doesn't.
