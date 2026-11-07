+++
id = "stock-models"
part = "stock"
kind = "migration"
needs = ["catalogue-core", "catalogue-store", "catalogue-clients", "config-models", "smith-limits"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/stock*.rs", "temper/crates/temper-service/src/tests/first_model.rs", "temper/crates/temper-service/src/tests/fixtures*", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/template/bare.json", "temper/crates/temper-init/src/tests/**", "temper/crates/temper-fake/src/**", "temper/crates/temper-playground/**", "temper/docs/examples/owned.json", "jig/crates/jig-protocol-config/src/llm.rs", "jig/crates/jig-protocol-config/src/llm/providers.rs"]
effort = "high"
minutes = 300
+++

# The models leave `temper.json`: the defaults are the binary's

## Goal

temper's stock gives the seven models and the scripted one as defaults of the `model` setting, numbered for good, and its charters name their first choices and the rest. `temper init` writes no models, and a document that names them is refused at its path. What a person sees doesn't change: this release's offer, orders, budgets and largest turn are the same.

Acceptance:

- `rg -n '"models"' temper/crates/temper-init/template/temper.json temper/crates/temper-init/template/bare.json temper/docs/examples/owned.json` finds nothing; each provider has its `endpoint`.
- `a_documents_models_are_refused_at_their_paths`: `engine.llm.models` and `engine.llm.providers[0].models`, under `Defaults::Passed`; a provider with no `endpoint` likewise.
- `stock_numbers_each_model_for_good`: `contracts.md`, 7's table; `a_row_whose_provider_isnt_configured_isnt_a_default`: a bare document has no 8, and the scripted model is there with `demo`.
- `every_stock_charter_names_its_first_choices_and_the_rest`: `rest: true`, and its resolution equals what the charter offered before, in the same order, on `temper init`'s box and on the bare document.
- The budgets are as `temper/crates/temper-service/src/tests/stock.rs:894` pins them (`C=10,896`, `K=43,584`), and the largest turn and every stock figure in `stock_brief.rs`, `limits.rs` and `first_model.rs` are unchanged, or moved only by `smith-limits`' or `config-models`' bound and regenerated there. Any other move is a question.
- A `temper init`'d box's `me` offer, resolved defaults and `model list` are as today: `temper/crates/temper-service/src/tests/first_model.rs` passes with no change of expected values.
- The fake boxes: `temper-fake`'s `rewrite` leaves no `models` in either document, and a fake Claude program's chat runs on its endpoint with a stock name; `temper-fake`'s tests pass, with the fake LLM's own `llm.json` lists unchanged in meaning.
- `temper/docs/help/reference/*` regenerated, and focused and fuzzy tests of every temper package this touches pass; `heavy cargo check --workspace --tests` compiles.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 2, 13.1, 13.4, 13.8; `setup.md`, 8.3.
- `jig/docs/design/protocol/configuration.md`, 3; `domain/engine.md`, 17.1, 17.7, 17.11.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 6 and 7.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files.

Readers and neighbours, keeping what each test checks: the survey's list of direct target members, in the focused tier:

- `temper/crates/temper-service/src/tests/checks.rs`, `temper/crates/temper/src/tests/reference_settings.rs`, `temper/crates/temper-playground/src/{documents.rs,documents/channel.rs,documents/settings.rs,tests/stories.rs}`, `temper/crates/temper-playground/stories/a/deployment.json`, `temper/tests/engine/domain/tests/fixtures/stock.rs`, `temper/tests/one-process/src/fixtures.rs`, `temper/tests/real-loop/src/{fixtures,case,landing}.rs`, `temper/crates/temper/tests/suite/forge_land.rs`, `temper/crates/temper/src/tests/{settings_fields,configuration,limits}.rs`.
- `jig/testing/jig-channel-kit/src/fixtures.rs`: only if jig's own document form can't serve temper's world, which then takes `Given::Own`.
- The process and browser tiers' fixtures are `process-fixtures`', not this task's: they aren't run here.

Options, each pre-approved:

- **A test's own models:** a world or test wanting another catalogue passes `Given::Own { jig: Defaults { models, .. }, .. }`. `stock::checked_with(models, result_bytes)` is the seam for a test that wants the stock's charters over its own table; `temper-service/src/tests/stock.rs`' singleton, overflow and empty cases go through it.
- **A stock row's provider** missing: filtered. **No row left:** `engine_with` refuses at `engine.llm.providers`, `What::Bound`, in place of `startup.rs:291`'s check. Say so in the report: a deployment with no stock provider has no default model (`draft/model-catalogue.status.md`, Open).
- **The fake Claude:** `program_route` replaces the endpoint and the provider's `endpoint`, and stops touching models, so the fake serves the stock's three Anthropic names. If the fake's program model is pinned by a test's literal, it takes `claude-opus-5-5`, the first the charter takes.
- **`temper/crates/temper-fake/src/demo.rs:129`,** which reads `llm.models`: it takes the scripted model's quantities from `stock::models`, or from `temper-fake`'s own constant if `temper-service` isn't its dependency, the two held equal by a test.
- `STOCK_VALUES` in `settings_fields.rs` counts the models: regenerate `reference/settings-fields.md`.
- A help page's stale words past their generated parts are `models-help`'s; if a help test fails here on a sentence you can't avoid, change that sentence alone.

## Steps

```rust
// temper-service/src/stock.rs
const MODELS: &[StockModel] = &[ /* contracts.md, 7, whole: number, provider, name, shown, effort, efforts, prices */ ];
pub(crate) fn models(llm: &jig_protocol_config::Llm) -> Box<[CatalogueModel]>;
pub(crate) fn checked_with(models: Box<[CatalogueModel]>, result_bytes: u32) -> Option<Defaults>;
pub(crate) fn checked(llm: &jig_protocol_config::Llm, result_bytes: u32) -> Option<Defaults>;   // models(llm), then checked_with
```

Efforts and prices are the templates' as they stand, with `source: Stock`, `picker: true`, `withdrawn: false`, `estimated: false`. `charter()` names `first_choices` that are among the rows, main the first, with `rest: true`, and with no first choice among them, the lowest number's as main. Budgets read the rows, as `stock.rs:186` read `llm.models`. `startup/checks.rs:846-862`: resolved charters' models against the agent's bounds, through `settings::charters::resolved`.

`jig-protocol-config`: under `Defaults::Passed`, `llm.rs` and `llm/providers.rs` refuse `models` at each path and require `endpoint`.

Each new test breaks its rule once by hand. `cargo fmt`; each tree's `old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, and with `--profile fuzzy` on `temper/tests/real-loop`'s package; `heavy cargo check --workspace --tests`. Commit and push.

## Report

Every pin that moved, old and new, and why it was this change. Which focused fixtures took `Given::Own`. What the fake boxes' programs now run. Anything a process story will read differently, for `process-fixtures`.

## Rebase (2026-11-07, step merge, foreman model-catalogue-2/stock-models/foreman-1)

combine main’s fixture optimizations with stock-models’ model-free documents and typed catalogue fixtures
The merge conflicts in `temper/crates/temper-service/src/tests/fixtures.rs` and `temper/crates/temper-service/src/tests/stock.rs`.
The overlapping main change is `43877e1ec47bd2a95f4323ba9ffcf4940a34065f` (“Reuse completed service fixtures and checked settings”).

In `fixtures.rs`, retain both `Json` from main and `start` from this branch in the `startup` re-exports.
Keep main’s `OnceLock` engine cache, completed-store reuse, supplied-store helpers and fixture checks.
Keep this branch’s model-free `ENGINE`, `small_model()` and `parse_engine()` path through `checked_with` for the fake fixture.

In `stock.rs`, resolve `single_model()` using main’s `fixtures::Json` helper.
Read the first provider and endpoint from one parsed document, wrap each in an array, and replace providers, endpoints and `application.demo` together.
Remove main’s reads and edits of provider `models[0]` and `engine.llm.models[3]`: those members no longer exist.
Retain this branch’s `single_engine()`, which selects `gpt-6.1-sol` through `checked_with` and `config::pass`.
Keep main’s batched `without_projects()` edits and this branch’s catalogue tests and typed-model callers.

Preserve the automatically merged `fixtures/startup.rs` changes from both sides, including `Json`, `start()` and the branch’s agent bounds.
This follows `contracts.md`, sections 6–7: documents carry provider endpoints; singleton catalogues enter as typed defaults.
Run the focused service fixture checks and stock tests through heavy, then the required focused/fuzzy checks; preserve the pinned budgets and first-model expectations.
