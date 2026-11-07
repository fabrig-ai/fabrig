# ui-model-line: the contract between its tasks

The binding names. `turn-llm` makes them and `model-line` reads them. The design is `jig/docs/design/client/domain/client.md`, section 10, and `temper/docs/design/client/README.md`, 3.5, *The work's account and model*.

```rust
// jig-client-domain/src/contract/turn.rs
pub struct Turn {
    pub messages: List<Message>,
    pub used: Usage,
    pub spent: u64,
    pub read_through: Option<u64>,
    pub cut: bool,
    /// The model its completion was made with and the effort it asked for
    /// (client/domain/client.md, 10); none where the turn names no model.
    pub llm: Option<super::ModelEffort>,
}

// jig-client/src/thread.rs
pub struct Thread {
    // ...
    /// The `llm` of the newest decoded turn, taken when the window reaches the live end
    /// (client/domain/client.md, 10).
    pub latest_llm: Option<ModelEffort>,
}
impl Thread {
    /// The thread has read as far as the task's document announces.
    #[must_use]
    pub fn caught_up(&self) -> bool;
}
```

- **`ModelEffort`** is the contract's own (`contract/llm.rs`): `model: Text`, `effort: Option<Text>`. An `effort` of `None` here means the turn asked for none.
- **No stored format and no wire form changes.** The turn's bytes already cross the wire whole (`w::TurnBytes::Carried`) and smith's `Turn` already holds `model: text 128` and `effort: option text 32` (`smith/crates/smith-transcript/schema/`, `v6`).
