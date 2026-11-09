+++
id = "goal-stock"
part = "stock"
kind = "feature"
needs = ["goal-kind"]
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/docs/examples/reference/settings.json",
  "temper/crates/temper/src/tests/reference_settings.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 60
+++

# Template 4's work: the stock's and the reference's

## Goal

The stock writes template 4's work, Draft first's goal template: its coordinator
plans and makes no close-out, as the stock's charter 5 does. The reference's
settings file writes its own template 4's: design and close-out, as its charter
5's second edition does. So a fresh box's Draft first draws "plan", and the
reference's case draws "design" and "close-out".

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.4 (*Template 4's
work*, *Built at each start*) and 13.9 (*template 4's work*):

1. **The stock's** (`temper/crates/temper-service/src/stock.rs`, `forge`): a
   default of kind 11, scope 0, number 4, `Value::Goal(TemplateGoal { design:
   false, close_out: false })`, sized as its neighbours are, after kind 3's two;
   the list's capacity and its `expect` words say what it now holds.
2. **The reference's** (`temper/docs/examples/reference/settings.json`): an item
   `{"setting": {"kind": "template-goal", "number": 4}, "set": {"design": true,
   "close_out": true}}`, right after its template 4's item, formatted as the
   file's other items are.
3. **Rows,** no new test function:
   - the service's stock table (`temper-service/src/tests/stock.rs`, where the
     forge's defaults are listed): kind 11 number 4 `{false, false}`, and no
     other template's;
   - where `temper-service/src/tests/family.rs` holds Draft first's path over a
     project built on the stock's defaults, it holds "plan" and no "close-out"
     unchanged: name the test that proves the stock's kind 11 reached the forge
     there, or, if none builds on the stock's defaults, add the kind 11 row to
     the stock table alone and say so in the report;
   - `temper/crates/temper/src/tests/reference_settings.rs`'
     `the_reference_settings_file_decodes`: `ITEMS` one more, and the
     `template-goal` 4 item decoding to both true, as its kind 7 item is read.
4. **Bite:** give the stock's template 4 `design: true`, see the stock row fail;
   restore.
5. **Status:** delete the bullet naming `goal-stock` in
   `temper/docs/design/protocol/configuration.status.md` (13.4, 13.9)
   (`notes/common.md`, *Status files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.4: *Template 4's work*,
  *Built at each start*; 13.9: the settings file's *template 4's work*; 13.2: *A
  goal template's work, kind 11*.
- `temper/docs/design/draft/stock-ways.md`, 3.3 and section 4.
- `temper/docs/plan/ui-p3f-design/notes/contracts.md`, sections 1 and 7.
- `temper/docs/plan/ui-p3f-design/notes/common.md`.

## Owns and may change

- Owns: the stock's kind 11 default, the reference's item, the rows and the
  status bullet.
- `temper/crates/temper-service/src/stock.rs` is walked by ui-p3e-words'
  `charter-lines`, which adds kind 9's defaults to the same `forge` list: on a
  catch-up, keep both (`notes/common.md`, *Beside this plan*).
- **The forge's start:** the forge tells jig which of its stock defaults changed
  at each start (configuration.md, 13.2, *Kept by the forge*). If a world's
  start or restore now reports kind 11 where its oracle expects only kinds 2
  and 3, that oracle is yours to widen by the one row, named in the report.
- The reference's README (`temper/docs/examples/reference/README.md`) is read by
  tests; it lists no item of the file, so it doesn't change. If a test reading
  it fails for the new item, repair it and say so.

## Steps

1. Read the inputs.
2. The stock's default and its row; the bite: `heavy cargo nextest run -p
   temper-service --lib -E 'test(/stock/)'`.
3. The reference's item and its row: `heavy cargo nextest run -p temper --lib
   -E 'test(/reference_settings/)'`.
4. Runs, counts in the report: `heavy cargo nextest run -p temper-service -p
   temper`; `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(/configuring|ways/)'`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service -p temper --all-targets -- -D warnings`.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the default and the item as written, the
test that proves the stock's default reached the forge, the rows and the bite,
the runs' counts.
