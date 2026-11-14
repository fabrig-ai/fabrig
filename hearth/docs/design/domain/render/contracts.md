# Render's contracts

2026-11-14. The shapes that the changes of the goal rewriting render under `temper/docs/design/draft/design-economy.md`, 7 share, and that no other area reads (`design-economy.md`, 4.3). They bind as any design section does. The goal's last change deletes this document and its status; no lasting document cites it.

## 1. What stays

- **Production outside the crate keeps its sources.** `hearth-protocol-builder`, `hearth-domain-builder`, `hearth-service-builder` and `hearth-client-protocol-records` compile unchanged, and every item of `hearth-domain-render` they name keeps its path, its fields, its variants and its meaning. A name they use may become an alias of a generic type (section 2) where their sources compile and behave as before.
- **Behaviour:** every digest and canonical record; every error, its kind, fields, place and order; the requests a call makes, and their order; every logical byte a limit charges (`README.md`, 2.3 and 5.4); the outcomes.
- **Tests:** their cases, expectations, independent oracles and bounds, the memory ceilings and the exact charges among them; the expected files under `hearth/tests/render/` byte for byte, with nothing blessed (`HEARTH_BLESS` is never set). A fixture that builds or inspects a record a change reshapes is adapted to what stays and checks the same.
- **Items only tests name outside the crate,** in `hearth-render-world` and `hearth-test-render`: `compose::{Layer, Provenance, Composed, Composite, Pair, Merge}`, `entries::{Fault, Expanded, Group, Concrete, Made, Fielded}`, `errors::Collector`, `evaluate::{Evaluator, Budget, Fault, Context, Binding, Bindings, Written, Operand}`, `revision::{Class, Documents}`, `schema::{Fault, TaskDefaults, Defaults, Check, Against, Validator}`, and the functions `entries::expand`, `evaluate::{expression, output}`, `evaluate::template::run`, `revision::{class, decoded_size}`, `schema::declarations` and the constant `evaluate::ELEMENT`. One goes or changes only with each test that names it adapted to check the same behaviour through what stays (`design-economy.md`, 7.2).

## 2. Shared shapes

### 2.1 A keyed change

`diff::Keyed<T>`, an enum: `Added { key, value: T }`, `Removed { key, value: T }` and `Changed { key, before: T, after: T }`, each `key` a `Box<[u8]>`. It is the one definition behind `parts::config::{GroupKeyed, UserKeyed, PresetKeyed, LineKeyed}`, `parts::data::DataKeyed`, `parts::live::ConsumersKeyed`, `parts::software::DownloadKeyed` and `parts::packages::{SeedUserKeyed, SeedGroupKeyed}`, which stay as aliases of it. One function makes a part's keyed changes from its two maps, in key order.

### 2.2 A changed value

`diff::Changed<T> { before: T, after: T }`. It is the one definition behind `parts::meta::{NamePair, MemberPair, PlacementPair}`, `parts::packages::{SourcesPair, VariantPair, PreseedsPair, InitramfsPair, AptPair}` and `parts::boot::Change`, which stay as aliases of it.

### 2.3 A package's identity

`resolution::Inst { name, architecture, version }`, ordered by name, then architecture, then version: the closure's key. `diff::Listed` is an alias of it, and `resolution::Triple` goes.

## 3. What is the crate's own

- **An item that no code outside the crate names is the crate's own:** private, or `pub(crate)` where another module or the crate's tests reach it. The modules `additions`, `collisions`, `ids`, `kinds` and `reserved` are private. An item that nothing names once it is the crate's own goes.
- **One record per subject at each stage of `README.md`, 3:** a record whose members repeat another's, with no rule of its own, merges into it (`design-economy.md`, 5.2); one that adds positions, removal, provenance or ownership that a later rule reads stays.
