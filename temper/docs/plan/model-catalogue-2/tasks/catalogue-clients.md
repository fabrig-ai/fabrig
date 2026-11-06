+++
id = "catalogue-clients"
part = "catalogue"
kind = "feature"
needs = ["model-surface", "catalogue-store"]
files = ["jig/crates/jig-protocol-engine-clients/src/boundary.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/translate/llm.rs", "jig/crates/jig-protocol-engine-clients/src/tests/catalogue.rs", "temper/crates/temper-engine-domain/src/route/**", "jig/testing/jig-test-domain/src/**"]
effort = "high"
minutes = 100
+++

# The clients' component takes each catalogue, and a root routes it

## Goal

The clients' component keeps a startup copy of the models and turns names to indices by it. It takes each catalogue as it comes, so a request naming a model added since the start finds it, and `me` shows it. Each root routes `Ask::Catalogue`: the store's first, then the clients'.

Acceptance:

- `a_catalogue_replaces_the_models_a_request_names`: a `Create` naming a new model resolves to its index after `Event::Catalogue`; before, it's refused as today.
- `a_hole_is_no_model`: an authority's named set, a `MeModel`, and a refusal's naming skip a hole by position, with later rows keeping theirs.
- The root's test in `temper/tests/engine/domain/`: `an_ask_for_the_catalogue_tells_the_store_then_the_clients`; jig's reference root likewise in `jig/testing/jig-test-domain`.
- `component.rs`'s `fits` runs on each catalogue: one past the wire's bounds is a panic, as the start's, since the core's check holds those bounds.
- Focused tests of `jig-protocol-engine-clients`, `jig/tests/engine-clients`, `temper-engine-domain`, `jig-test-domain` pass.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.11, *Who hears*.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 2.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. `model-surface` gave each root a drop arm for `Ask::Catalogue`, which this task fills. Neighbours, call sites only: `jig-protocol-engine-clients/src/{request,reads,watches}.rs`, `public/me.rs`; `jig-protocol-config/src/clients.rs` (`models` still gives the start's); the `Settings.models` literals in fixtures.

Options, each pre-approved:

- `boundary.rs`'s `Model` is the clients' form, and `Event::Catalogue` carries it: a root translates `CatalogueModel` to it with the function `jig-protocol-config/src/clients.rs:41` holds, moved to `jig-protocol-engine-clients` so that a root and config both call it. A hole translates to a row of empty name.
- The component's existing watchers re-read on the core's `me` mark. Send no document from here.

## Steps

```rust
// boundary.rs
pub enum Event { .., Catalogue { models: Box<[Model]> } }
// translate
pub fn catalogue(models: &[jig_domain_core::settings::CatalogueModel]) -> Box<[Model]>;
```

`self.settings.models` stays where every reader finds it, replaced by the event. Each test breaks its rule once by hand. `cargo fmt`; each tree's `old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents; `heavy cargo check --workspace --tests`. Commit and push.

## Report

Which other readers of a startup copy you found, if any.
