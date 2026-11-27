# Model speeds: contracts

The shapes that the changes building a model's speeds share, while they are built: goal F1 of `temper/docs/design/draft/model-speeds.execution.md`, from `temper/docs/design/draft/model-speeds.md`. They bind as any design section does (`temper/docs/design/draft/design-economy.md`, 4.3). The rules are the lasting documents': `settings-catalogue.md`, 1 and `executors.md`, 1 for jig; smith's `protocol/charter.md`, section 2 and `domain/session.md`, section 6; skein's `llm/vocabulary.md`, 1.3. Once the changes have landed the code holds these shapes, and the goal's last change deletes this document.

## 1. jig's model

```rust
// jig_domain_core_vocabulary, src/runs/state.rs, exported where `Model` is
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Speed {
    pub name: Box<[u8]>,                // as the provider takes it and sends it
    pub also: Box<[Box<[u8]>]>,         // at most two names it may be reported served under
    pub input_price: u64,
    pub cached_price: u64,
    pub cache_write_price: Option<u64>, // None: cache writes at `input_price`
    pub output_price: u64,
}

pub struct Model {
    // ...every field it has today, then:
    /// The faster speeds the model declares, at most four, in the order given.
    pub speeds: Box<[Speed]>,
    /// On a configured model, its default speed, one of `speeds` by name, or `None` for
    /// the provider's standard speed. On a run's charter, the run's speed by name:
    /// its model's default (`executors.md`, 1).
    pub speed: Option<Box<[u8]>>,
}

impl Model {
    /// The speed `speed` names among `speeds`; `None` at the standard speed.
    #[must_use]
    pub fn run_speed(&self) -> Option<&Speed>;
}
```

Stored in `jig/crates/jig-protocol-store/schema/settings-v3.schema`, mapped by its `settings.maps`, each field added with its default so that an older layout reads as the standard speed with no speeds:

```
record Speed { name: bytes 32 also: list 2 bytes 32 input_price: u64 cached_price: u64 cache_write_price: option u64 output_price: u64 }
record Model { ...today's fields... speeds: list 4 Speed = [] speed: option bytes 32 = none }
```

## 2. smith's charter and run

The charter, `smith/crates/smith-charter/schema/charter-v5.schema`; `speed` added with its default, so a charter without one keeps going down to an agent layout without the field, and one with it is refused there (jig's `protocol/channel/layouts.md`, 2.2 and 5.1):

```
record Speed { name: text 32 also: list 2 text 32 prices: Prices }
record Llm { ...today's fields... speed: option Speed = none }
```

The run's domain:

```rust
// smith_domain_run, src/charter.rs, exported where `Llm` is
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Speed {
    pub name: Box<[u8]>,
    pub also: Box<[Box<[u8]>]>,
    pub prices: crate::Prices,
}

pub struct Llm {
    // ...every field it has today, then:
    /// The speed every completion of this LLM asks for; `None` for the provider's default.
    pub speed: Option<Speed>,
}
```

The session's own, as its `Prices` is its own (`skein/docs/foundation/programming-model.md`, 4.5), carried whole wherever the session keeps an LLM's prices for a conversation (its opening, and the `record::Llm` a window's change of model takes), and its name wherever it keeps that LLM's effort for a prompt:

```rust
// smith_domain_session::record
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Speed {
    pub name: Box<[u8]>,
    pub also: Box<[Box<[u8]>]>,
    pub prices: Prices,
}
// `speed: Option<Speed>` beside `prices` in `record::Opening` and `record::Llm`
```

## 3. skein's

`Prompt.speed`, `Completion.served` and `Provider::standard_speed(self) -> &'static [u8]`, as skein's `llm/vocabulary.md`, 1.3 gives them, in `skein_llm`'s `types.rs` beside `Prompt`, `Completion` and `Provider`.
