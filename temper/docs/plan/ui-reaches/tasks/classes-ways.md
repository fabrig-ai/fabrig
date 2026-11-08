+++
id = "classes-ways"
part = "classes"
kind = "feature"
needs = ["classes-kind"]
files = [
  "temper/crates/temper-engine-domain-ways/src/ways.rs",
  "temper/crates/temper-engine-domain-ways/src/lib.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain-ways/Cargo.toml",
  "temper/crates/temper-service/src/stock/ways.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/tests/engine/domain/tests/suite/ways.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "Cargo.lock",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "medium"
minutes = 60
+++

# The stock ways lay kind 8

## Goal

Choosing a way sets what reaches the project's people: each stock way's items carry kind 8 set to that way's table, You approve's, Agents review's or Draft first's (`temper_protocol_reaches::Classes::stock`), so that "Use this" and "Keep it" move the classes with the rules.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.10 (the table's kind 8 entries, *A way's items*) and `temper/docs/design/domain/forge.md`, 5.10 (*The stock tables*):

1. **`StockWay`** (`temper-engine-domain-ways/src/ways.rs`) gains `classes: Option<temper_protocol_reaches::Classes>`, documented as the table the way sets whole, none for a world's way that sets none.
2. **`items`** pushes, after kind 7's where the way lays it, `Item { kind: 8, scope: project, number: 1, edit: Edit::Set(Value::Classes(table)) }` where `classes` is some.
3. **The stock data** (`temper-service/src/stock/ways.rs`): You approve `Some(Classes::stock(Way::YouApprove))`, Agents review `Some(Classes::stock(Way::AgentsReview))`, Draft first `Some(Classes::stock(Way::DraftFirst))`.
4. **Nothing else moves:** a way's path (`forge.md`, 8.13) and which way a project runs are derived as they are; kind 8 takes no part in them.
5. **Rows,** no new test function: the ways crate's items table (`src/tests.rs`, about line 74), each stock way's items ending in its kind 8 item, and a way with `classes` none adding none; the service's stock way contents (`tests/stock.rs`, about line 1445), each way's table; the service's ways document items (`tests/family.rs`, about line 2994) and its worst small document's size (about line 3398), re-measured, the bound it checks unchanged (if the document no longer fits its bound, stop and ask with the figures); the engine world's expected stock items (`temper/tests/engine/domain/tests/suite/ways.rs`, about line 100); the gallery's stock-way proposal fixture (`temper/tools/gallery/src/worlds/ways.rs`, about line 373), its items as the stock now has them.
6. **Bite:** drop the kind 8 item from `items`, see the items row fail; restore.
7. Runs, counts in the report: `heavy cargo nextest run -p temper-engine-domain-ways -p temper-service -p temper-engine-domain -p temper-engine-domain-world -p temper-client-gallery`; `heavy cargo nextest run -p temper-engine-domain-ways --profile fuzzy`; clippy for every package changed.
8. Status: delete the **Not yet** entry naming `ui-reaches/classes-ways` in `temper/docs/design/protocol/configuration.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`; `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/protocol/configuration.md`, 13.10.
- `temper/docs/design/domain/forge.md`, 5.10, *The stock tables*.
- The code: `temper-engine-domain-ways/src/ways.rs` (`StockWay`, `Laid`, `items`, `laid_ways`), `temper-service/src/stock/ways.rs`.

## Owns and may change

Owns the field, the item, the stock data and the rows. May add `temper-protocol-reaches` to the ways crate's `Cargo.toml` and `Cargo.lock`, and the field's literal in any other constructor of `StockWay` (a world's or a test's own ways: `classes: None`), each named. Not the ways' paths (`look-round-24/ci-workflows` is changing `path.rs`, `path/derive.rs` and their tests beside this task), the client's proposal cards (the client's tasks draw kind 8's preview), or `route/ways.rs` beyond a literal.

Options, each decided:

- **A client-world or browser story** that compares a way's items whole: give it the kind 8 item, compile-checked where it is a slow tier's (`heavy cargo check -p PKG --tests`), and name it.
- **The engine world's `ways.rs`** is also `look-round-24/rule-view`'s: change only the expected items' rows, nothing else of it.

## Steps

1. Read the Inputs; the field, the item and the data.
2. The rows and the bite.
3. The runs of acceptance 7; the status entry; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: the field and item as written, every constructor given the field, the rows, the document's size before and after, the bite and the runs' counts.
