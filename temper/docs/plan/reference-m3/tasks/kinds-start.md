+++
id = "kinds-start"
part = "kinds-root"
kind = "feature"
needs = ["kinds-route", "apps-start"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/stock/**", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/tests/kinds.rs", "temper/crates/temper-engine-domain/src/tests/resume_fixture.rs", "temper/tests/engine/domain/src/walking.rs", "temper/tests/engine/domain/src/oracle/config.rs"]
effort = "high"
minutes = 120
+++

# kinds-start: temper declares kinds 1 to 3, and the forge's defaults at each start

## Goal

temper's service declares its three kinds to jig's builder (C1) and gives the forge their
defaults at each start; the root hands the forge those defaults once its records are restored,
carries the forge's answer of which differ to the core (C2, **At a start**), and routes the
core's word to save them. The defaults are, until `kinds-document` makes them the binary's
(13.4), what the document configures today: its landing rules and its `charter`, with the
review charter 4, as 13.6, step 1 bridged jig's kinds. Nothing reads the held rules yet
(`kinds-switch`), so no change's behaviour moves; a start makes a release's revision when the
document's rules or charter changed between starts.

Acceptance:

- `temper-service`'s `stock` module:
  - `pub fn kinds(limits: &temper_engine_domain::Limits, connector: u16) ->
    Box<[jig_domain_core::settings::Kind]>`
    (C1): kind 1 `Deployment` and kind 2 `Project`, each `per_scope` `forge.landing_rules`,
    `charters` 0, `value_bytes` the encoding's size (`temper_engine_smith::kinds::size`) of a
    largest rule at the bounds `kinds-route` judges by (segments, segment bytes, gates,
    approvals); kind 3 `Deployment`, `per_scope` 2, `charters` 1, `value_bytes` a charter's
    size; each `connector` the forge's.
  - `pub fn forge_defaults(landing: &LandingPolicy, charter: u32) -> Box<[forge::settings::
    Stock]>`: kind 1 number `n` for the document's `n`th deployment rule, kind 2 scope `p`
    number `n` for project `p`'s, each translated to the forge's `settings::LandingRule`
    member for member (`connector` and `kind` dropped), sized by `kinds::size`; kind 3 number
    1 `charter`, number 2 charter 4. The binary and every `Given` take it from the document.
- `temper_engine_domain::Config` gains `kinds_stock: Box<[forge::settings::Stock]>`;
  `system.rs` and `startup/checks.rs` fill it with `forge_defaults` and `core.kinds` with
  `kinds`; every other `Config` literal gives `Box::default()` (and `core.kinds` stays as
  `apps-surface` left it).
- The root, where the forge's range of records has reached the forge (the restart's
  `Phase::RestoreConnector` completing, `restart.rs`), and only when `core.kinds` names the
  forge's connector: `forge.give_settings_defaults(stock)`, its answer translated (`charter`
  to `charters`) and pushed to the core as `connector::Event::SettingsDefaults { defaults }`,
  in that decision, before `Phase::Link`. `Ask::SaveSettingsDefaults { revision: _ }` for the
  forge's connector calls `forge.save_settings_defaults(out)`: the body of `route/kinds.rs`'
  `save_defaults`, which `kinds-route` routes there.
- Tests pass (Steps); `temper-service`'s and `temper-engine-domain`'s focused tests; and,
  since a start's meaning changes, `heavy cargo nextest run -p temper-engine-domain-world
  --test suite -E 'test(/settings::|restart/)'`.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.4 (**Built at each start**), 13.6
  (steps 1 and 2, as this cut wrote step 2); `temper/docs/design/domain/root.md`, 5.6 (the
  bullet's last sentence on a start), 6 (**Configuration**) and 9.1 (step 3's "Before them");
  `temper/docs/design/domain/forge.md`, section 16 (its kinds' values before its records).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C1, C2 (**At a
  start**) and C3.
- `temper/docs/plan/reference-m3/tasks/apps-start.md` (what the core does with
  `SettingsDefaults` and when it asks `SaveSettingsDefaults`).

## Owns and may change

- Owns the files in the header for the edits above. `domain.rs`: the root keeps
  `kinds_stock` beside its configuration, nothing else. Never `forge_route.rs` (the change
  family's `change-row-llm` and `change-root` may run beside this task there).
- Options: if the forge's restored rows reach it later than `RestoreConnector`'s completion
  (they are work items still queued), give the stock as a work item queued after them, and
  say where. If `apps-start` takes `SettingsDefaults` only within a named start phase, send
  it there and say which. If a world's start is refused because a stored or default state
  names a kind it no longer declares, that world gives `kinds` and `kinds_stock` as the
  binary does, in its own builder file (name it in the report).
- Not this task's: the binary's own stock (kind 1 rule 1 as 13.4 has it) and the document's
  refusals (`kinds-document`); reading the held rules and charters (`kinds-switch`).

## Steps

1. `stock::kinds`, `stock::forge_defaults`, the `Config` field and its fills.
2. The root's give and answer at the restart, and the save's route.
3. Tests:
   - temper-service `src/tests/stock.rs`: `kinds_are_declared_with_their_bounds` (the three
     kinds; kind 1's `value_bytes` equal to a largest rule's size);
     `the_documents_rules_and_charter_are_the_forges_defaults`.
   - temper-engine-domain `src/tests/kinds.rs`: `a_start_gives_the_forge_its_defaults`
     (a restart with a stored `Default` row equal to the stock and one not: the core gets
     `SettingsDefaults` with `differs` false and true, before the link);
     `save_settings_defaults_writes_the_forges_rows`; `no_kinds_declared_sends_no_defaults`.
   Narrow: `heavy cargo nextest run -p temper-service --lib -E 'test(/stock::/)'`;
   `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/kinds::/)'` (check the
   counts).
4. fmt, old forms, clippy on both crates; their focused tests; the world tests in Goal.
5. Commit ("Declare temper's kinds and give the forge their defaults at each start") and
   push.

## Report

The implementer's text's report, plus: where the stock is given in the restart; any world
that now declares the kinds.
