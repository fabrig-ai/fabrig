# debt-20: the shapes that cross tasks

Binding where a spec cites a section. Inside them the choices are yours.

## 1. The store's copy write (jig D025)

```rust
// jig-protocol-store/src/boundary.rs
/// Copy the bytes the map holds under `from` to `to` in this commit (store.md 6).
Copy { from: core::Key, to: core::Key },

// jig-domain-core/src/settings/revisions.rs
/// A held revision's key and the history row it enters as.
#[must_use]
pub fn history_copy(number: u32) -> (crate::Key, crate::Key);
```

`Write::CopySettingsRevision` leaves `jig_protocol_store::Write` alone. The core's `core::Write::CopySettingsRevision`, and the same word in an application root's write set (temper's engine, `ops`, `jig-test-domain`), stay: they are a domain's own vocabulary. Whoever routes a root's writes to the store (`temper-service/src/store/route.rs`, `jig-test-store/src/route.rs`) calls `settings::history_copy` and writes `store::Write::Copy { from, to }`.

## 2. The catalogue's remaps (jig D025)

```rust
// jig-domain-tasks/src/llm.rs
impl ModelEffort {
    /// The index of a kept choice whose model, or the effort it named, left the catalogue: no configured model has it.
    pub const GONE: u8 = u8::MAX;
}
// jig-domain-tasks/src/limits.rs
impl Limits { pub fn word_bytes(&self) -> u32; }                 // the codec's `word_bytes`, whole
// jig-domain-tasks/src/stored/models.rs (new; `mod models;` in stored.rs)
impl Stored { pub fn remap_models(&mut self, moved: &[u64; 64]); } // `codec/tasks/models.rs`'s `remap` and what it calls
// jig-domain-tasks/src/value.rs and jig-domain-authority/src/value.rs
impl Models { #[must_use] pub fn remap(self, moved: &[u64; 64]) -> Self; }  // as `jig_domain_people::Models::remap`

// jig-domain-core/src/settings/models.rs (new; `mod models; pub use models::{model_bit, remap};` in settings.rs)
/// The bit of today's model of this name, or 0.
#[must_use]
pub fn model_bit(name: &[u8], names: &[ModelName]) -> u64;
/// `value`, read under the catalogue `kept`, under today's `names`.
pub fn remap(value: &mut Value, kept: &[ModelName], names: &[ModelName]);
```

The codec turns its `List<wire::ModelName>` into `settings::ModelName`s for `kept`. Stored bytes, the value a row decodes to, and `u8::MAX`'s being the index all stay as they are.
