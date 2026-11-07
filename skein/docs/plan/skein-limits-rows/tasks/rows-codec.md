+++
id = "rows-codec"
part = "rows"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-rows/**",
  "skein/crates/skein-codec/Cargo.toml",
  "skein/crates/skein-codec/src/tests/**",
  "skein/crates/skein-codec/src/value/tests.rs",
  "skein/crates/skein-codec/src/patch/tests.rs",
  "skein/crates/skein-codec/src/patch/run/tests.rs",
  "skein/crates/skein-codec/src/patch/plan/tests.rs",
  "skein/crates/skein-codec/src/patch/text/tests.rs",
  "skein/crates/skein-codec/src/schema/tests.rs",
  "skein/crates/skein-codec/src/schema/fingerprint/tests.rs",
  "skein/docs/design/testing.status.md",
  "skein/docs/plan/skein-limits-rows/plan.md",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "medium"
minutes = 80
+++

# The rows kit, and `skein-codec`'s folds

## Goal

The two other fold tasks build on `skein-rows`, a small kit: `check` runs every row of a table test and names each that fails. This task makes it, and folds `skein-codec`, which has the alike cases and no harness to learn, so that the kit meets real tables before the others take it.

Acceptance:

- `skein/testing/skein-rows` exists as `skein/docs/design/testing.md`, 6 has it, with the signature in `skein/docs/plan/skein-limits-rows/notes/rows.md`, and its own tests below pass and bite.
- `skein-codec`'s honest groups are folded by `notes/rows.md`'s rules. The survey found 45 tests that fold into 21 tables, saving 24 processes: what you reach is what the rules let you, more or less, with the rest under *Left apart*.
- Every removed test name is a row's name, every assertion stays, and one table per file bites.
- `plan.md`, *Before and after*, *rows-codec*, is filled.
- `python3 -I -B tools/lints/lints.py` reports nothing, and the focused tests of `skein-rows` and `skein-codec` pass.

## Inputs

- `skein/docs/plan/skein-limits-rows/inputs/request.md`, item 2.
- `skein/docs/plan/skein-limits-rows/notes/common.md` and `skein/docs/plan/skein-limits-rows/notes/rows.md`: both bind.
- `skein/docs/design/testing.md`, 6: *A table test's rows*, and the layout's line.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process*, and 10.
- `tools/lints/design.md`, 3 to 5: what a new kit under `testing/` takes.

## Owns and may change

Owns the files in the header. In `Cargo.toml` and `Cargo.lock`, only the lines a new crate needs: a `members` entry and a `[workspace.dependencies]` entry, as `skein-fork` has them. In `skein/docs/design/testing.status.md`, remove the line `6: skein-rows.`, and nothing else there.

Not `skein-codec`'s product code. Not `skein/testing/skein-codegen`: its `table` helper takes function rows, which `skein/docs/foundation/testing-strategy.md`, 10 doesn't allow by hand, and generated code keeps it.

Options, each decided:

- **Row data can't be `'static` or a tuple gets long:** a small `struct` beside the table, or build the array in the test.
- **`check`'s `Fn(R)` won't take a body that needs `&mut`:** make it local to the body. A shared `&mut` across rows breaks *A row's setup stays a row's*, and `FnMut` isn't the answer.
- **Clippy objects to `panic!` in the kit:** a scoped `#[expect(clippy::panic, reason = "...")]`. A failure is this kit's whole job.

## Steps

1. **The kit.** `skein/testing/skein-rows/`, with `Cargo.toml` (copy `skein-scratch`'s shape: `description = "One table test's rows: every row runs, and a failure names each"`, no dependencies, `[lib] doctest = false`). Then `python3 -I -B tools/lints/lints.py --set harness skein/testing/skein-rows/Cargo.toml`, a copy of `tools/lints/clippy/ordinary.toml` as its `clippy.toml`, and the two root `Cargo.toml` lines. Its `src/lib.rs`:

   ```rust
   //! One table test's rows (skein/docs/design/testing.md, 6): cases of one
   //! behaviour share a process (skein/docs/foundation/testing-strategy.md,
   //! 8), and every row still runs and names itself when it fails.
   //!
   //! Ordinary Rust (programming-model.md, 10.2): it runs in tests only.
   use std::panic::{AssertUnwindSafe, catch_unwind};

   /// Runs `body` on every row, then panics naming each row that panicked,
   /// with its message. Fewer than two rows, or two rows of one name, panic
   /// before any row runs.
   #[track_caller]
   pub fn check<R>(rows: impl IntoIterator<Item = (&'static str, R)>, body: impl Fn(R)) { ... }
   ```

   A row's message is its panic payload, a `String` or `&str`. Any other payload reads `<no message>`. The failure says how many of how many failed, then one line per row: `name: message`.

2. **Its tests,** in `src/tests.rs`, each catching `check`'s own panic and reading its message:
   - `every_row_runs_after_an_earlier_row_fails`: a `Cell` counts the bodies. Bite by dropping `catch_unwind`, and see it fail for that.
   - `a_failure_names_each_failing_row_with_its_message`: two of three rows fail. Both names and messages are in the message, and the passing row's name isn't.
   - `rows_that_all_pass_return`.
   - `a_table_of_fewer_than_two_rows_fails_before_any_row_runs`: zero and one, with the count of bodies 0.
   - `two_rows_of_one_name_fail_before_any_row_runs`.

3. **Fold `skein-codec`.** `skein-rows.workspace = true` under a new `[dev-dependencies]`. By module, largest first, from `skein/crates/skein-codec/src/`; line ranges are at commit `92d04ee863`:

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `patch/run/tests.rs`:229–241, 836–840 | Derived conversion emits canonical bytes for the new schema | 3 | schemas, patch, root, input, output |
   | `patch/run/tests.rs`:607–623, 782–788 | Whole-declaration carry refuses values that require explicit rebuilding | 3 | schemas, patch, bytes, error path |
   | `patch/run/tests.rs`:367–374 | Identity conversion stops at held values and preserves bytes | 2 | bytes or secret schema |
   | `patch/run/tests.rs`:377–394 | Copying held content is misbuilt and permits restarting | 2 | bytes or secret schema |
   | `patch/run/tests.rs`:455–461, 913–919 | Wrong scalar puts halt naming the expected schema part | 2 | schema, root, bytes, error path |
   | `patch/plan/tests.rs`:638–655 | Identity plans propagate held stops through nested record declarations | 2 | bytes or secret schema |
   | `patch/plan/tests.rs`:745–768 | Held stops require matching foreign clauses in both schemas | 2 | old and new field types, flags |
   | `patch/plan/tests.rs`:704–713 | Identity pieces refuse items that reach held fields | 2 | bytes or secret schema |
   | `patch/plan/tests.rs`:225–245, 337–349 | Patch lines must match the schema changes they describe | 2 | schemas, patch lines |
   | `patch/plan/tests.rs`:212–222, 812–830 | Missing changes report the first required operation in order | 2 | schemas, lines, missing operation |
   | `patch/plan/tests.rs`:205–209, 519–539 | Explicit layout-change lines admit their matching schema changes | 2 | old and new schemas, lines |
   | `patch/text/tests.rs`:54–91, 248–263 | Grammar refusals identify the expected token at its span | 2 | text, line, span, expected |
   | `patch/tests.rs`:130–142 | Patch text admission enforces both sides of its capacity | 2 | capacity, expected result |
   | `patch/tests.rs`:59–78 | Schema retention enforces its text and version capacities | 2 | capacity, prior schemas, result |
   | `tests/piece.rs`:27–46 | An item exceeding writer room leaves output unchanged | 3 | room, prior items, item, output |
   | `tests/piece.rs`:55–63 | Splitting complete pieces returns every item in order | 2 | bytes, expected items |
   | `tests/piece.rs`:66–77 | Splitting refuses incomplete item lengths and incomplete item bodies | 2 | malformed bytes |
   | `value/tests.rs`:261–274 | Debug never renders stored text or secret content bytes | 2 | schema, bytes, forbidden strings |
   | `schema/tests.rs`:158–189 | Held byte and secret clauses retain their external references | 2 | schema, expected types and spans |
   | `schema/tests.rs`:192–213 | Clause words followed by colons remain ordinary field names | 2 | schema, expected field types |
   | `schema/fingerprint/tests.rs`:62–87 | Held secrets hash their literal canonical clause production bytes | 2 | schema, canonical encoding |

   Read each one first. The `Wrong scalar puts` pair is an example to check: one asserts `restart`, the other `next`'s `Err(Halt::Order)`, so it stays apart unless both can hold both truly. `tests/piece.rs`'s first group reads `out.finish()` in one test and `out.written()` in the others, which a body reading `finish()` can hold for all three.

   The survey left these apart, for your check of *Left apart*: `patch/run/tests.rs` 52, `patch/plan/tests.rs` 27, `patch/text/tests.rs` 13, `patch/tests.rs` 8, `tests/piece.rs` 12, `tests.rs` 5, `value/tests.rs` 14, `schema/tests.rs` 16, `schema/fingerprint/tests.rs` 12, for different operations, lifecycles, parser phases and contracts, or because each is already a table.

4. **Fill `plan.md`,** by `notes/rows.md`'s form.

5. **The end of a run.** `cargo fmt`, old forms, citations, `python3 -I -B tools/lints/lints.py`, and `heavy cargo clippy -p skein-rows -p skein-codec --all-targets -- -D warnings`. Then `heavy cargo nextest run -p skein-rows -p skein-codec`, and read the tables' `PASS` times against the 40 ms guide. Commit and push.

## Report

Tests before and after, tables and rows; the names-removed count against the names-found count; which tables you broke and how they failed; each listed group you left apart and each unlisted one you folded; anything about the kit's interface the other fold tasks should know, since they start from it. Your implementation and check minutes apart.
