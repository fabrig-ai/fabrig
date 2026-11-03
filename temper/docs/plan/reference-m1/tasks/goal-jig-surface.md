+++
id = "goal-jig-surface"
part = "goal-jig"
kind = "interface"
needs = []
files = ["jig/docs/design/domain/tasks.status.md", "jig/docs/design/domain/connectors.status.md", "jig/crates/jig-domain-people/src/batch.rs", "jig/crates/jig-domain-people/src/lib.rs", "jig/crates/jig-domain-core/src/settings/templates.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/routing/task/terminals.rs", "jig/crates/jig-domain-core/src/routing/party/holdings.rs", "jig/crates/jig-domain-core/src/routing/run/delegation.rs", "jig/crates/jig-domain-core/src/tests/**"]
effort = "high"
minutes = 75
+++

# goal-jig-surface: a batch member's symbols, a template's resources, a taking's entry

## Goal

The jig surface the rest of the goal family builds on, carried with today's behaviour:

- **Symbols** (contract C2): `jig_domain_people::Symbol { connector: u16, symbol: u16 }`; a
  batch member's `jig_domain_people::Member.symbols: Box<[Symbol]>`; a template's
  `jig_domain_core::Template.resources: Box<[Symbol]>`.
- **The holdings boundary:** the core's asks for a task's holdings say whether the task is
  tracked and which of the connector's symbols it names; a connector's answer may refuse,
  naming a symbol it does not offer that task, or the task holding a resource the member
  would write under a hold that is not its own.
- **A taking's entry:** `Ask::Hold`, which tells a connector a task took a write hold, carries
  a fresh outbox entry for the taking's own effects (jig's `connectors.md`, 4.7), as
  `Ask::Release` carries one for a release's.

This task fills the new fields with empty lists, `false` or the value at hand, and maps a
refusal as the holdings answer's `None` is mapped today. The core's bodies (passing a
member's symbols, naming refusals) are `goal-jig-holdings`'s; the configuration's, the
store's and the client's are `goal-jig-config`'s, `goal-jig-store`'s and `goal-jig-client`'s;
the forge's are temper's tasks.

Acceptance:

- In `jig-domain-people` (`src/batch.rs`, exported from `lib.rs` beside `Member`):

  ```rust
  /// A connector's symbol: a resource it names after the task once the task has its
  /// number (domain/authority.md, section 4; protocol/configuration.md, 3.3).
  #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
  pub struct Symbol {
      pub connector: u16,
      pub symbol: u16,
  }
  ```

  and `Member` gains, after `named`, `pub symbols: Box<[Symbol]>` (doc: "Connectors' symbols
  this member holds, each resolved by its connector once the task has its number
  (domain/tasks.md, 6.2).").
- `jig_domain_core::Template` (`src/settings/templates.rs`) gains, after `llm`,
  `pub resources: Box<[jig_domain_people::Symbol]>` (doc citing configuration.md, 3.3);
  `merged` and `current_merged` carry it unchanged (a policy's override never touches it).
- In `jig-domain-core/src/vocabulary.rs`:
  - `Ask::TaskHoldings` gains `tracked: bool` and `symbols: Box<[u16]>` (this connector's
    symbols of the member, in the member's order); `HoldingsNeed` gains the same two fields.
  - `Ask::Hold { task, resource }` gains `entry: u64` (doc: "The taking's own outbox entry,
    for the effects its connector's kind has at a taking (domain/connectors.md, 4.7).").
  - A new answer refusal, exported from the crate's root:

    ```rust
    /// Why a connector refuses a member's holdings (domain/tasks.md, 6.1 and 6.2).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum HoldingsRefusal {
        /// The member names a symbol this connector does not offer it.
        Symbol { symbol: u16 },
        /// The member would write a resource this task holds, outside its subtree.
        Held { holder: u64 },
    }
    ```

    and `Event::Holdings`, `Event::DelegateHoldings` and `Event::ProcedureHoldings` each
    gain `refused: Option<HoldingsRefusal>`, set only with `holdings: None`.
- The core fills them: `creation_holdings` (`routing/party/holdings.rs`) sends
  `tracked: new.tracked.is_some()` and `symbols: Box::new([])`; the delegate and procedure
  paths (`routing/run/delegation.rs`, `holdings_need`, and wherever else a `HoldingsNeed` is
  built) send the member's `tracked.is_some()` and no symbols. An answer with `refused:
  Some(_)` is handled exactly as `holdings: None` is today.
- `routing/task/terminals.rs`, `taken`: each write holding's `Ask::Hold` carries a fresh entry
  from `crate::fresh(&mut core.counters, crate::Family::ConnectorRow)`, as
  `routing/task/lifecycle.rs`, `release` takes one; `taken` takes `core` for it. If no entry
  is left, the task is held as `release` holds it (`tasks::Hold::Budget`), and no `Ask::Hold`
  of that taking is sent.
- A core test (`jig-domain-core/src/tests/`, beside `fill.rs`'s holdings tests):
  `a_taking_tells_each_write_holds_connector_with_a_fresh_entry` (two write holdings, two
  `Ask::Hold`, distinct entries, neither equal to a release's entry of the same task later).
- Everything else compiles and passes as before.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 3.3; `jig/docs/design/domain/authority.md`,
  section 4 (the paragraph on a connector's symbol); `jig/docs/design/domain/connectors.md`,
  3.1 ("Symbols") and 4.7; `jig/docs/design/domain/tasks.md`, 6.1 and 6.2.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2 and its line
  under "Names the families write here", and C3.

## Owns and may change

- Owns the files in the header.
- **Reader files, call sites and literals only** (a new field's empty value, `..` in a
  pattern, the new `refused: None`): every construction of `people::Member` (the store's
  `jig/crates/jig-protocol-store/src/codec/people/batch.rs` decode gives `symbols:
  Box::new([])` and its encode ignores them; `jig/crates/jig-protocol-engine-clients/src/
  translate/operations.rs` gives `Box::new([])`; `jig/crates/jig-client/src/shapes.rs` if it
  builds one; fixtures in `jig/crates/jig-protocol-store/src/tests/fixtures*.rs`,
  `jig/tests/core/src/fixtures/`, `jig/tests/channel/src/fixtures.rs`,
  `jig/tests/people/src/fixtures.rs`, `jig/testing/jig-test-profile/src/fixtures.rs`,
  `jig/testing/jig-test-service/src/tests/fixtures.rs`,
  `temper/crates/temper-service/src/tests/fixtures.rs`); every construction of
  `jig_domain_core::Template` (`jig/crates/jig-protocol-config/src/templates.rs` gives
  `resources: Box::new([])`; `jig/crates/jig-protocol-store/src/codec/settings.rs`,
  `decode_template`, the same; tests and conformance fixtures that build one); every match
  or construction of `Ask::TaskHoldings`, `Ask::Hold`, `HoldingsNeed`, `Event::Holdings`,
  `Event::DelegateHoldings` and `Event::ProcedureHoldings` (`jig/testing/jig-test-domain/
  src/routes.rs`, `jig/testing/jig-test-service/src/clients/translation.rs`,
  `jig/examples/ops/domain/src/route.rs`, `jig/tests/client-protocol/src/engine.rs`,
  `temper/crates/temper-engine-domain/src/route/asks.rs`,
  `temper/crates/temper-engine-domain/src/route/forge_route.rs`,
  `temper/crates/temper-service/src/clients/translation.rs`). In temper, `asks.rs` answers
  `refused: None` and ignores `entry`, `tracked` and `symbols`: their bodies are the forge's
  tasks.
- `jig/crates/jig-domain-core/src/routing/run/delegation.rs` and `routing/task/
  terminals.rs` are changed on plan reference-m7a's trunk too: keep the hunks to these
  fields.
- **Options:** if `HoldingsNeed` is built where the member's `tracked` is not at hand, send
  `false` there and name the site in the report. If a test elsewhere asserts the exact
  `Ask::Hold` value, update it to the entry it now carries. If `fresh` cannot be reached from
  `taken`'s caller without changing more than `routing/task.rs`'s one call, pass the
  counters instead and say so.
- **Design:** remove the lines this closes: in `jig/docs/design/domain/tasks.status.md`,
  "6.2: a taking's effects: `Ask::Hold` carries no outbox entry."; in
  `jig/docs/design/domain/connectors.status.md`, "4.7: a taking's effects: `Ask::Hold`
  carries no outbox entry.". Every other entry is another task's.

## Steps

1. Add the types and fields; build with `heavy cargo check -p jig-domain-core --tests` and
   then each reader package the compiler names (`heavy cargo check -p <pkg> --tests`, among
   them `jig-protocol-store`, `jig-protocol-config`, `jig-protocol-engine-clients`,
   `jig-test-domain`, `jig-test-service`, `temper-engine-domain`, `temper-service`).
2. The entry at a taking, and its test: `heavy cargo nextest run -p jig-domain-core --lib -E
   'test(a_taking_tells_each_write_holds_connector_with_a_fresh_entry)'`.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and the same for temper;
   `heavy cargo clippy -p jig-domain-people -p jig-domain-core -p jig-protocol-store
   -p jig-protocol-config -p jig-protocol-engine-clients -p temper-engine-domain --all-targets
   -- -D warnings` (with every other package you changed); `heavy cargo nextest run -p
   jig-domain-people -p jig-domain-core`, then the focused tests of the reader packages
   above, `jig-core-world` (`jig/tests/core`) and `temper-engine-domain`.
4. Commit ("jig: symbols, a template's resources and a taking's entry, as surface"), push.

## Report

The fields and types as written, each reader file touched and how, the tests run with their
counts, and any option taken.
