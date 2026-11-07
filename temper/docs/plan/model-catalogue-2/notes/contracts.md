# model-catalogue-2: the contracts between parts

The run's binding calls. The design sections they implement are jig's `domain/engine.md`, 17.1 and 17.11; smith's `protocol/agent.md`, 4; temper's `protocol/configuration.md`, 13.4. Each side builds to this note, and neither needs the other's task unless its spec's `needs` says so. Names are binding; bodies are the task's.

## 1. The setting (jig)

```rust
// jig-domain-people/src/settings.rs
pub enum Setting { Charter(u32), Template(u32), Policy(u32), Rules, Model(u32), Application { .. }, ProjectPolicy { .. } }

// jig-domain-core/src/settings/catalogue.rs, new; re-exported from settings
pub enum ModelSource { Stock, List, Person }

/// A `model` setting's value whole (domain/engine.md, 17.1).
pub struct CatalogueModel {
    pub number: u32,                     // 1 to MODELS; never given to another name
    pub model: Model,                    // what a run reads
    pub shown: Option<Box<[u8]>>,
    pub description: Option<Box<[u8]>>,
    pub picker: bool,                    // whether a picker shows it
    pub withdrawn: bool,                 // out of the offer and of a charter's resolution; its row stays for the tasks that hold it
    pub source: ModelSource,
    pub estimated: bool,                 // its prices are its provider's defaults
}

// jig-domain-core/src/settings.rs
pub enum Value { Charter(..), Template(..), Policy(..), Rules(..), Model(Box<CatalogueModel>), Application(..) }
pub struct Defaults { charters, templates, policies, rules, pub models: Box<[CatalogueModel]> }

// jig-domain-core/src/runs/state.rs
pub struct Model { .., pub provider: u32, .. }     // the numbered LLM provider that serves it

// jig-domain-core/src/settings/charters.rs
pub struct Charter { .., pub rest: bool }          // after the models it names, every other model of the catalogue
```

- **Position:** model `i`, in `Settings.models`, a `ModelEffort`, a bit of `Models`, is the setting numbered `i + 1`. `Settings.models` has one row up to the highest number; a number no setting has is a hole, `Model::hole()`, whose `name` is empty. No real name is empty, so a lookup by name never finds one. An iteration that offers or resolves skips holes with `Model::is_hole`. A withdrawn model keeps its row.
- **Numbers:** the binary's defaults take low numbers, from 1, and keep them for good. A person's or a list's take the highest free number, downward from `MODELS`, so a release's new default can't meet one.
- **Name:** a set naming another name than its number's default or stored change is `Fault::Shape`.
- **Removed and reset:** `Edit::Removed` of a model stores a set of its current value with `withdrawn: true`, so its number and name stay. `Edit::Reset` of a model with no default is `Fault::Shape`.
- **A charter's resolution:** `settings::charters::resolved(charter: &Charter, models: &[CatalogueModel], limits: &Limits) -> Option<Charter>`. Each model the charter names is replaced by the catalogue's record of that name; withdrawn ones are dropped, and the main is the first left. With `rest`, every other row that is no hole and not withdrawn follows, in number order, while the charter stays within `limits.settings.alternatives` and `limits.run_bytes`. `None` when no model is left. Runs, needs and tasks' charter tables read the resolved charter; a setting's value, a read and a proposal keep the charter as set.
- **Charter versions:** a decision or a release that changes a charter's resolution counts as a change of that charter: its `Default` or `Changed` revision moves to the new revision and the old resolved charter is held, as for any charter replaced (`held.rs`). Its stored value doesn't change.
- **Left out:** `documents` gives a project's chat template's charter's count of rows `resolved` passed over for its bound.

## 2. Telling the parts (jig)

```rust
// jig-domain-core/src/vocabulary.rs
pub enum Ask { .., Catalogue { models: Box<[CatalogueModel]> } }
// sent at the end of a start's settings restore, and in a decision that changes the catalogue, before its commit

// jig-protocol-engine-clients/src/boundary.rs
pub enum Event { .., Catalogue { models: Box<[Model]> } }     // replaces `Settings.models`

// jig-protocol-store
impl Store { pub fn catalogue(&mut self, names: Box<[jig_domain_core::settings::ModelName]>); }
```

A root routes `Ask::Catalogue` to both, the store's first. The tasks child hears `Event::Charters { efforts, .. }`, as it does; the accounts child reads `Model.provider`, and `Provider.models` goes; the fleet's `Needs` are replaced as for a charter's change.

**At open** the store reads the settings family's `model` changes by key range, before any family's restore, and makes `names` the given names overlaid with them, so that a stored vocabulary's person's model is not taken for gone.

## 3. An owner's act (jig)

```rust
// jig-client-domain/src/contract/settings.rs
pub enum ModelAct { Picker { number: u32, on: bool }, Add { provider: u32, name: Text } }
pub struct ChangeModel { pub act: ModelAct }
```

A new keyed request beside `ChangeSettings`, by an owner in the deployment. The core makes the one `Item` at the newest revision, `Running::Stays`, and decides it as a `ChangeSettings`; its answers and refusals are `ChangeSettings`' (`not theirs`, `out of shape`, `empty`, `full`), and it is never stale. `Add`:

- **The name's shape:** `settings::catalogue::valid_name`: 1 to 128 bytes, smith's charter's bound, of ASCII letters, digits, `-`, `.`, `_` and `:`. Otherwise `Fault::Shape`.
- **A name the catalogue holds:** withdrawn, it clears `withdrawn`; otherwise `Fault::Shape`.
- **The number:** the highest free. None free is `Fault::Count`.
- **The rest:** `settings::catalogue::new_model(provider, name, source, models, endpoints) -> Option<CatalogueModel>`: the provider's dearest stock row by `most_costly`, or with none the catalogue's dearest stock row on the provider's `endpoint`, gives `endpoint`, `dialect`, `program`, `window`, `output` and prices; no efforts, no effort; `picker: true`, `estimated: true`, `source`. `None` is `Fault::Reference`. `model-catalogue-3` calls it with `ModelSource::List`.

## 4. `me` and the wire (jig)

```rust
// jig-client-domain/src/contract/llm.rs
pub struct MeModel { .., pub number: u32, pub source: ModelSource, pub estimated: bool }
pub enum ModelSource { Stock, List, Person }
// contract/me.rs
pub struct MeProject { .., pub left_out: u32 }
// contract/settings.rs
pub enum Setting { .., Model(SettingNumber) }
pub enum SettingValue { .., Model(Box<CatalogueEntry>) }
pub struct CatalogueEntry { number, model: SettingModel, shown, description, picker, withdrawn, source, estimated }
pub struct SettingModel { .., provider: u32 }
pub struct Charter { .., rest: bool }
```

The client's wire and the settings family's schema change in place, each with its generated code and goldens. The wire's charter alternatives ceiling and the profiles' `core.settings.alternatives` are 63. A withdrawn model is out of `me`'s offer.

## 5. The agent's limits (smith, fleet)

An endpoint of `agent.json` has no `models`. It has:

```json
"limits": {"window": 1048576, "output": 8192, "reasoning_item": 65536, "head_ms": 60000, "idle_ms": 120000, "oversized_reasoning": "fail"}
```

`oversized_reasoning` is optional, `fail` by default, and the deadlines take the names and units the model's have today. `profile.declared` gains `charter_models`, the most models one charter's LLMs name; `temper init` gives 64.

```rust
// jig-domain-link/src/lib.rs
pub struct EndpointBound { pub endpoint: u32, pub window: u32, pub output: u32 }
pub struct Bounds { .., pub endpoints: Box<[EndpointBound]>, .. }   // replaces `models` and `ModelBound`
// jig-domain-fleet/src/needs.rs
pub struct ModelNeed { pub endpoint: u32, pub name: Box<[u8]>, pub window: u32, pub output: u32 }
```

`AgentShortfall::Model` is a need whose endpoint no bound holds; `Window` and `Output` a need past its endpoint's bound. The fleet's `catalogue` goes.

## 6. The document (jig's configuration, temper)

- **`engine.llm.providers[].endpoint`,** the name of the endpoint its added models take: required under `Defaults::Passed`, and optional under `Defaults::Document`.
- **Under `Defaults::Passed`,** `engine.llm.models` and a provider's `models` are refused at their paths; a seed's named models find none; `pass` takes `Defaults.models`.
- **Under `Defaults::Document`,** `engine.llm.models` stays, for jig's own reference and worlds: each is a default numbered by its position, from 1, `source` `Stock`. A charter with no `models` member is `rest: true`.

## 7. temper's stock

```rust
// temper-service/src/stock.rs
pub(crate) fn models(llm: &jig_protocol_config::Llm) -> Box<[CatalogueModel]>;
```

Numbers, for good: 1 `claude-opus-5-5`, 2 `claude-sonnet-5-5`, 3 `claude-fable-5-1`, 4 `gpt-6.1-sol`, 5 `gpt-6-astra`, 6 `gpt-6.1-terra`, 7 `gpt-6-luna`, 8 the scripted `demo`. The stock names each row's provider by number: anthropic 2, openai 1, demo 3. A row whose provider the document doesn't configure isn't among the defaults, and it reads the provider's `endpoint` for `endpoint`, `dialect` and `program`. Every stock charter takes `rest: true`, naming `FIRST_CHOICES` or `CONFIGURING_FIRST_CHOICES` where they're among the defaults.
