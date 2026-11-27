# Model speeds, chosen: contracts

The shapes that the changes building a choice's speed share, while they are built: goal F2 of `temper/docs/design/draft/model-speeds.execution.md`, from `temper/docs/design/draft/model-speeds.md`, section 4. They bind as any design section does (`temper/docs/design/draft/design-economy.md`, 4.3). The rules are the lasting documents': `../tasks/README.md`, section 3, `executors.md`, 1, `settings-catalogue.md`, 1 and `settings.md`, 4 for the core; `../../client/domain/contract/documents.md`, 1 and `../../protocol/client/wire.md` for the client. Once the changes have landed the code holds these shapes, and the goal's last change deletes this document.

## 1. jig's choice

```rust
// jig_domain_contract::ops, re-exported by jig_domain_tasks as the choice's types are

/// The name a choice gives the provider's standard speed (domain/tasks/README.md, 3).
pub const STANDARD_SPEED: &[u8] = b"standard";

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelEffort {
    pub model: u8,
    pub effort: Option<u8>,
    /// The choice's speed by name: one its model declares, by its own name, or
    /// `STANDARD_SPEED`; `None` takes the speed the charter's resolution gives the model.
    pub speed: Option<skein_lib::Name<32>>,
}

// jig_domain_tasks::LlmFault, a variant after `Repeated`, and the same in
// jig_domain_people's `LlmWhy`, its copy:
    /// The speed is not one the model declares, nor `standard`.
    Speed,

// jig_domain_people::ModelEffort, the people child's copy of the choice:
// the same `speed` field, of the same type and meaning.
```

`Name<32>` keeps `ModelEffort` `Copy`; a name is checked against its model's speeds where the choice is checked, never by its grammar alone.

`jig_domain_core_vocabulary::Model`'s `speed`, `Option<Box<[u8]>>`, keeps its type and means, by where the record is:

- on a configured model, its default speed, `None` for standard (F1, as built);
- on a charter's value as set, the charter's speed for that model: a name it declares, `STANDARD_SPEED`, or `None` for the model's default (`settings-catalogue.md`, 1);
- on a charter's resolution and on a run's charter, the speed the run takes, `None` for standard (`executors.md`, 1).

## 2. jig's client choice and offer

```rust
// jig_client_domain::contract::llm

pub struct ModelEffort {
    pub model: Text,
    pub effort: Option<Text>,
    /// By name, `standard`, or `None` (domain/tasks/README.md, 3).
    pub speed: Option<Text>,
}

pub struct MeModel {
    // ...every field it has today, then:
    /// The faster speeds it is served at, in the model's order (settings-catalogue.md, 1).
    pub speeds: List<super::SettingSpeed>,
    /// Its default speed by name; `None` for the provider's standard.
    pub speed: Option<Text>,
}

pub enum LlmWhy {
    // ...every variant it has today, then:
    /// The speed is not one the model declares, nor `standard`.
    Speed,
}
```

The client wire's `ModelEffort`, `MeModel` and `LlmWhy` gain the same, `speed` as `option bytes 32 = none` and `speeds` as a list of 4 of the existing `SettingSpeed` record, `= []`.
