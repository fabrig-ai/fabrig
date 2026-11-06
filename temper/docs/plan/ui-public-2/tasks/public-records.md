+++
id = "public-records"
part = "public-records"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-protocol-wire/schema/public-v1.schema",
  "jig/crates/jig-client-protocol-wire/schema/public-v1.fields",
  "jig/crates/jig-client-protocol-wire/src/generated/public_v1.rs",
  "jig/crates/jig-client-protocol-wire/golden/public/**",
  "jig/crates/jig-client-protocol-wire/src/tests/public/fixtures.rs",
  "jig/crates/jig-domain-core/src/public.rs",
  "jig/crates/jig-domain-core/src/public/line.rs",
  "jig/crates/jig-domain-core/src/public/index/relations.rs",
  "jig/crates/jig-domain-core/src/tests/public/**",
  "jig/crates/jig-protocol-engine-clients/src/public/records.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/public.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/public/fixtures.rs",
  "jig/tests/client-protocol/tests/suite/public.rs",
  "jig/tests/client-wire/tests/suite/fuzzy/public_wire.rs",
  "jig/docs/design/domain/people.status.md",
  "jig/docs/design/client/domain/contract.status.md",
]
effort = "medium"
minutes = 40
+++

# A public line carries a goal's spend alone, no budget, and how a task ended

## Goal

jig's public records stop carrying a task's spend and any budget, and an ended task's phase says how it ended. temper's public view keeps compiling on the new record, and nothing more changes there: its look is the next task's.

Acceptance:

- jig's `public-v1.schema` is as `temper/docs/plan/ui-public-2/notes/contracts.md` has it. Its generated code, goldens and `public-v1.fields` agree, and the drift test and `the_fields_golden_is_the_reachable_wire_shape` pass.
- A goal's line has `spent: Some(spent + spent_below)`. A task's has `None`. No line has a budget.
- A task that ended done, failed or cancelled has the phase `Done`, `Failed` or `Cancelled`, on a live board's tree line and on an ended goal's frozen line alike. Nothing makes a phase `Ended`: the variant is gone.
- `PublicCounts.ended` still counts all three.
- temper's public view shows "12 spent" on a goal's line and no spend on a task's, with its tests passing.
- The **Not yet** lines naming 13.3 in `jig/docs/design/domain/people.status.md` and 12 in `jig/docs/design/client/domain/contract.status.md` are gone.

## Inputs

- `jig/docs/design/client/domain/contract.md`, section 12: `PublicLine` and the source table.
- `jig/docs/design/domain/people.md`, 13.3, *A public line*.
- `temper/docs/plan/ui-public-2/notes/contracts.md`, *jig's `public-v1.schema`*, binding.
- `temper/docs/plan/ui-public-2/notes/common.md`.

## Owns and may change

Owns the files in the header.

Readers and neighbours, for these edits only:

- **`temper/crates/temper-client-view/src/public/lines.rs`,** the required field's lines alone. The spend becomes:
  ```rust
  if let Some(spent) = line.spent() {
      self.words.text(b, Word::RowSpent, &[Hole::Number(*spent)]);
  }
  ```
  and the `PublicPhase::Ended` arm goes. `Word::RowSpent` exists and says "12 spent". `Word::PublicEnded` stays in the table: the next task uses it.
- **`temper/crates/temper-client-view/src/tests/public/records.rs`:** `spent: if goal { Some(12) } else { None }`, and no `budget`.
- **`temper/crates/temper-client-view/src/tests/public.rs`:** `b"12 spent \xc2\xb7 of 50"` becomes `b"12 spent"` in both allowed lists.
- **Any other reader of `PublicLine`, its parts or `PublicPhase`,** in any tree: the field's name, type path or value alone. A survey found `temper/tests/one-process/src/public/*` reading only `task()` and the tree, and no reader of `spent()` or `budget()` beyond the files above. If a package's check shows another, edit it the same way and name it in your report.

Options, each decided:

- **Regenerating:** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'` writes the generated code and goldens (`jig/tests/codec-families.rs`). If it leaves `golden/public/enum_public_phase_ended_*.bin` behind, delete them.
- **`public-v1.fields`** has no writer: edit it to what `the_fields_golden_is_the_reachable_wire_shape` lists. That the diff shows in review is its point.
- **A fixture helper with a `budget` parameter** (`jig/crates/jig-domain-core/src/tests/public/reads.rs`, `line`) loses it, and its callers follow.
- **A test that used `PublicPhase::Ended`** (`jig/crates/jig-domain-core/src/tests/public/reads.rs`, 131) uses `Done`.
- **An encoded length pinned by a test** follows the new shape. A bound nothing pins needs nothing.

Not this task: temper's words, look or forge records, `Word::ChatbarSpend` (private pages use it), and any design text.

## Steps

1. Change the schema and regenerate. Then `jig-domain-core`'s `PublicLine` and `line.rs`: `spent` is written as `priority` is, from `row.tracked`, and `phase` matches on the `Ending` inside `tasks::Phase::Ended`, listing every variant. Then `relations.rs`'s arm and `jig-protocol-engine-clients/src/public/records.rs`.
2. Add focused tests in `jig/crates/jig-domain-core/src/tests/public/index/projection.rs`, on the index's own fixtures:
   ```rust
   #[test]
   fn a_goals_line_carries_its_spend_and_a_tasks_carries_none() { /* a goal with spent and spent_below, and a delegate: Some(sum), None */ }
   #[test]
   fn an_ended_line_says_how_it_ended() { /* Done, Failed, Cancelled (with and without a result): each its phase */ }
   ```
3. Repair the fixtures and expected records of the readers above, then temper's three lines.
4. Remove the two **Not yet** lines.
5. **This changes what a shared item means,** so run the tests of every package that reads it, through heavy:
   - `heavy cargo nextest run -p jig-domain-core -p jig-protocol-engine-clients -p jig-protocol-engine-clients-http -p jig-client-protocol-wire -p jig-client-protocol-http -p jig-client-wire-world -p jig-client-protocol-world -p temper-client-view -p temper-client-domain`
   - `heavy cargo check -p temper-one-process-world -p temper-client-browser-world -p temper-client-shell-web --tests`
6. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, and clippy on every package you changed. Commit and push.

## Report

As the implementer's text says, plus: each test's rule broken by hand and the failure seen, and any reader edited beyond the list above.
