+++
id = "goal-jig-client"
part = "goal-jig"
kind = "feature"
needs = ["goal-jig-surface", "goal-jig-config", "goal-jig-store"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-domain/src/contract/me.rs", "jig/crates/jig-client-domain/src/contract/operations.rs", "jig/crates/jig-client-protocol-link/src/translate/**", "jig/crates/jig-protocol-engine-clients/src/translate/**", "jig/crates/jig-protocol-engine-clients/src/tests/**", "jig/crates/jig-client/src/shapes.rs", "jig/crates/jig-client/src/tests/**", "jig/tests/client-wire/**", "jig/docs/design/protocol/configuration.status.md", "jig/docs/design/domain/authority.status.md"]
effort = "high"
minutes = 90
+++

# goal-jig-client: `me` serves a template's resources, and the client's fill sends them

## Goal

A template's symbols travel from the engine to the client and back as a batch member's
(jig's `protocol/configuration.md`, 3.3; contract C2): `me` serves each effective template's
`resources`; the client's fill copies them into the `NewTask` it makes, its `symbols`; the
engine's client translation puts them into `jig_domain_people::Member.symbols`, where
`goal-jig-surface` left `Box::new([])`.

Acceptance:

- **Client wire** (`client-v19.schema`, changed in place before the first release, with its
  generated code and goldens): a `record Symbol { connector: u16 symbol: u16 }`; the served
  `Template` record gains `resources: list 32 Symbol`; `NewTask` gains `symbols: list 32
  Symbol`.
- **Client contract** (`jig-client-domain`): `contract::Symbol { connector: u16, symbol: u16 }`;
  `me.rs`'s `Template` gains `resources: List<Symbol>` (or the container its other lists
  use); `operations.rs`'s `NewTask` gains `symbols` of the same type, with an empty default
  wherever `NewTask` has a constructor or builder.
- **Translations:** the engine's (`jig-protocol-engine-clients`) from
  `jig_domain_core::Template.resources` to the wire, and from the wire's `NewTask.symbols` to
  `jig_domain_people::Member.symbols` (bounded by the wire's list, refused past
  `limits.tasks.holdings` as the translation refuses its other lists, under a new
  `w::Path::NewTaskSymbols` if paths are named per field); the client link's
  (`jig-client-protocol-link`) both ways.
- **The fill** (`jig-client/src/shapes.rs`, `fill`): the `NewTask` it makes from a template
  carries the template's `resources` as its `symbols`, in order.
- Tests: in `jig-client`'s tests, `a_filled_template_carries_its_resources_as_symbols`; in
  `jig-protocol-engine-clients`'s tests, `a_new_tasks_symbols_reach_the_people_member` and
  `a_served_template_carries_its_resources`; the client wire's golden drift check passes.
- Status: in `jig/docs/design/protocol/configuration.status.md`, the three lines beginning
  "3.3:" go (the configuration's reading and the store's keeping are `goal-jig-config`'s and
  `goal-jig-store`'s, merged before this task; if either is not on your base, keep its line
  and say so); in
  `jig/docs/design/domain/authority.status.md`, the clause "4: a batch member's connector
  symbols (`jig-domain-people`'s `Member` names none)." goes. No other status text.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 3.3 ("Filled as any default", "Served in
  `me`"); `jig/docs/design/domain/authority.md`, section 4 (a connector's symbol).
- `jig/docs/design/protocol/client.md` and `jig/docs/design/client/domain/contract.md`, 5.1
  (what `me` carries): read only the template's lines.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2.

## Owns and may change

- Owns the files in the header.
- **Reader files, literals only:** every other construction of the client's `NewTask` or
  `Template` in jig and temper (`jig/crates/jig-client/src/tests/fixtures.rs`,
  `jig/crates/jig-protocol-engine-clients/src/tests/fixtures/objects.rs`,
  `jig/testing/jig-client-test-domain/**`, `jig/testing/jig-client-scripted-engine/**`,
  `jig/tests/client/**`, temper's client crates and tests), each given an empty list.
- **Neighbours:** plan reference-m7a changes `jig-client-protocol-wire` (its `fleet-jobs`
  task): keep the schema hunks to `Symbol`, `Template` and `NewTask`, and regenerate, never
  edit generated code by hand.
- **Options:** if the served `Template` and the one a policy's override edits are one wire
  record, add `resources` to it once and leave overrides without it (configuration.md, 3.3:
  never overridden). If temper's client fills templates through its own code rather than
  `jig-client`'s `fill`, copy the resources there too, and name the file.

## Steps

1. Edit the schema; regenerate with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`, then run it without
   the variable.
2. Contract, translations, fill; the three tests by name with `heavy cargo nextest run -p
   <pkg> --lib -E 'test(=<path>)'` (check each selects one).
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` (and temper's if touched);
   `heavy cargo clippy -p jig-client-protocol-wire -p jig-client-domain -p
   jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client --all-targets -- -D
   warnings`; their focused tests and `jig-client-wire-world`; `heavy cargo check --tests`
   on each temper package you touched.
4. Commit ("jig client: a template's resources served and filled as symbols"), push.

## Report

The wire records changed, each translation, the tests and counts, and any option taken.
