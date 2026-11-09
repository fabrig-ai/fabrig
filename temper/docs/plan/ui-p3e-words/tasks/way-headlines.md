+++
id = "way-headlines"
part = "stock"
kind = "feature"
needs = ["words-wire"]
files = [
  "temper/crates/temper-engine-domain-ways/src/ways.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/crates/temper-service/src/stock/ways.rs",
  "temper/crates/temper-service/src/family/ways.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-engine-smith/src/tests/see.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 75
+++

# Each stock way's headline, in the ways and the `ways` read

## Goal

Each stock way carries a headline of its own, the words How work runs heads the
page with while the project runs that way, and the `ways` read carries it to the
client beside the way's name and line. The client draws it in `how-words`; this
task makes it held and sent.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.10 (the table's
*Its headline*, *In this order*):

1. **The ways** (`temper/crates/temper-engine-domain-ways/src/ways.rs`):
   `StockWay` and `WayEntry` gain `pub headline: Box<[u8]>`, after `name`
   (`temper/docs/plan/ui-p3e-words/notes/contracts.md`, section 5); the root
   (`temper-engine-domain/src/route/ways.rs`) copies a stock way's into its
   entry where it copies `name` and `line`.
2. **The stock's** (`temper/crates/temper-service/src/stock/ways.rs`): each way's
   headline byte for byte as the note's section 8 has it: "You approve every
   change", "Agents review every change, and code lands it", "Draft first, then
   agents build it on its own branch".
3. **Sent:** the service's `ways_panel` (`temper-service/src/family/ways.rs`)
   sends `headline` as the wire's `StockWay.headline`, replacing `words-wire`'s
   empty stand-in; `words: None` stays (`project-words-read`'s).
4. **Rows,** no new test function:
   - `tests/stock.rs`' `the_stock_ways_hold_their_words_and_settings` (about line
     1500): each row's headline beside its name and line;
   - `tests/family.rs`' `the_worst_small_ways_document_fits_its_decision` (about
     line 3399): each entry's `headline` at its bound, 120 bytes, so the worst
     document still fits its decision; and the ways panel's encode test there, if
     one compares a way's fields, its headline sent;
   - the ways crate's `tests.rs`, where a `StockWay` or `WayEntry` is built: its
     headline carried into the entry.
5. **Bite:** send an empty headline in `ways_panel`, see the family row that reads
   it fail; restore.
6. **Status:** delete the bullet naming `way-headlines` in
   `temper/docs/design/protocol/configuration.status.md` (13.10)
   (`notes/common.md`, *Status files*). Leave `client/README.status.md`'s 4.6
   bullet: `project-words-read` deletes it.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.10.
- `temper/docs/design/client/README.md`, 4.6 (`StockWay`).
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 2, 5 and 8.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: the two fields, the stock's three headlines, their sending, the rows and
  the status bullet.
- Compile-forced, the field alone, in any file of temper's tree that builds a
  `StockWay` or a `WayEntry` (the survey found `temper-engine-smith/src/tests/see.rs`
  and the engine world's `temper/tests/engine/domain/tests/suite/ways.rs`, if it
  builds one): its fixture's own headline or the stock's; name each in the
  report.
- `see ways` (`temper/docs/design/domain/agent.md`, 8.4.1) prints a way's name,
  "now" and line, not its headline: leave its words as they are.
- Not yours: How work runs' drawing and the gallery's samples (`how-words`), the
  project's words in the document (`project-words-read`), the wire's schema
  (`words-wire`, merged). A tier's card in the same crates is `tier-words`', which
  may run beside this task: keep to your hunks.

## Steps

1. Read the inputs; `ways.rs`' `StockWay` and `WayEntry`, the root's copy, the
   stock's ways and `ways_panel`.
2. The fields, the stock's headlines and the sending; the rows; the bite.
3. Narrow tests: `heavy cargo nextest run -p temper-service --lib -E
   'test(=tests::stock::the_stock_ways_hold_their_words_and_settings) |
   test(=tests::family::the_worst_small_ways_document_fits_its_decision)'`; `-p
   temper-engine-domain-ways`.
4. The read reaches the client: run the focused tests of `-p
   temper-engine-domain-ways`, `-p temper-engine-domain`, `-p temper-engine-smith`,
   `-p temper-service`, `-p temper-engine-domain-world` and `-p
   temper-client-journeys` (the journeys' replay reads How work runs, which draws
   no headline yet: regenerate only if it fails, and say why).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-domain-ways -p temper-engine-domain -p temper-service
   --all-targets -- -D warnings`, with any other package you changed.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the worst document's encoded size before and
after against its allowance; each compile-forced file; the rows and the bite.
