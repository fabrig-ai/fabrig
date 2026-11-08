+++
id = "stock-ways"
part = "stock"
kind = "feature"
needs = ["ways-surface", "ask-words"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/stock/ways.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/Cargo.toml", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 70
+++

## Goal

Hold the three stock ways as data in the `stock` module and give them to
the engine at each start, beside the forge's defaults, so that the root's
`ways` read has the stock's words and items
(`temper/docs/design/protocol/configuration.md`, 13.10).

Acceptance:

- `temper_service::stock::ways() -> Box<[temper_engine_domain_ways::StockWay]>`
  gives, in this order (13.10's table, its words byte for byte):
  - "You approve", "An agent makes each change; nothing reaches main until
    you approve it.": rule 1 `branch`, open `""`, enforced, CI, up to
    date, no gate, one approval `{ role: 0, people: 1, freshness: Clean }`;
    no `ways`; no delegation;
  - "Agents review", "An agent makes each change, another reviews it, and
    code lands it.": rule 1 equal to the forge stock's kind 2 rule 1
    (`stock::forge`'s value, built from one shared function, not a copy);
    no `ways`; no delegation;
  - "Draft first", "You and an agent write a design draft; agents build it
    on its own branch, and it lands whole.": rule 1 `branch`, exact
    `main`, enforced, CI, up to date, no gate, no approval; rule 2
    `branch`, open `temper`, enforced, no CI, not up to date, no gate, no
    approval; `ways` `Laid { drafts: true, goals: true, way: 4 }`;
    `delegation: true`.
- `stock::Given::Own` gains `ways: Box<[StockWay]>`; the startup sets
  `Config::ways_stock` to `stock::ways()` for `Given::Stock` and to the
  world's own for `Given::Own`; every construction of `Given::Own` in any
  package gives `crate::stock::ways()` (or `temper_service::stock::ways()`)
  unless it wants none.
- A row table in `src/tests/stock.rs` holds each way's name, line, rule
  numbers and values, `ways` and `delegation`, and Agents review's rule 1
  equal to the forge's stock rule 1; a startup test (beside the one that
  checks `kinds_stock`) shows the started root's `ways_stock` is the
  stock's three.
- `temper/docs/design/protocol/configuration.status.md`: the words "the
  stock module holds no ways, the engine is given none at a start, and"
  leave its 13.10 **Not yet**, which keeps what is left for the reads.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 1 (`StockWay`, `Laid`) and 4.
- `temper/docs/design/protocol/configuration.md`, 13.10 (the table), 13.4
  (kind 2's rule 1 and *A project's rule 1 is given to the forge*), 13.2
  (a pattern beginning at `branch`).
- `temper/crates/temper-service/src/stock.rs`, `forge` and `Given`;
  `temper/crates/temper-service/src/startup.rs`, where `kinds_stock` is set.

## Owns and may change

- **Owns:** `stock.rs`'s `ways` and `Given`; a new `stock/ways.rs` if you
  keep the data there; `startup.rs`'s setting of `ways_stock`; the tests
  above; the `temper-engine-domain-ways` dependency line in
  `temper-service/Cargo.toml`.
- **Reader files, the new field only:** each construction of
  `Given::Own` (the service's fixtures and tests, `temper/crates/temper`'s
  tests, `temper-playground`, `temper/tests/one-process`,
  `temper/tests/real-loop`); the slow tiers' packages are compiled, not
  run.
- **Options, pre-approved:** if the service's intermediate engine struct
  carries `kinds_stock` on its way to the root's `Config`, give it
  `ways_stock` beside it the same way.

## Steps

1. Factor the forge's stock rule 1 into a function both `forge` and
   `ways` call; write `ways`.
2. `Given::Own`'s field and the startup; the forced constructions.
3. The tests; break a way's line once to see its row fail, and restore.
4. The status words.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-playground
   -p temper --all-targets -- -D warnings` (and each package whose
   `Given::Own` you touched); `heavy cargo nextest run -p temper-service
   --lib`; `heavy cargo check --tests` for each other touched package.
   Commit and push.

## Report

As the implementer's text asks; list each `Given::Own` site you changed.
