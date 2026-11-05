+++
id = "emit-structure"
part = "codegen-once"
kind = "feature"
needs = ["names-once"]
files = ["skein/testing/skein-codegen/src/emit_functions.rs", "skein/testing/skein-codegen/src/emit_bounds.rs", "skein/testing/skein-codegen/src/emit_types.rs", "skein/testing/skein-codegen/src/emit_codec.rs", "skein/testing/skein-codegen/src/lib.rs", "skein/testing/skein-codegen/src/patch/module.rs"]
effort = "high"
minutes = 90
+++

# skein: the generator splits functions as values, and edits no text it has written

## Goal

`skein/docs/development/debt/D035-codegen-diff-and-emit.md`'s second clause. After emitting, the generator reparses its own Rust to
split long functions (`split_bound_functions`, `split_record_functions`, by `split('(')` and `replace`). A function that may split
becomes a value holding its statements, and is rendered once. Every generated file stays the same to the byte.

**The clause, as this cut reads it:** after an emitter writes text, nothing parses or rewrites that text. Statement and
expression text stays `format!`, and items that never split are written straight out. No Rust syntax tree is wanted.

Acceptance:

- `rg -n 'split_bound_functions|split_record_functions' skein -g '*.rs'` finds nothing.
- These are gone: `emit_codec.rs`'s `replace` (about line 267), `emit_types.rs`'s `strip_prefix` (about lines 121-124), `patch/module.rs`'s
  `replace` (about line 122), and `emit_bounds.rs`'s `.contains("limits")` (about line 61, using `emit_codec`'s `uses_limits`).
- Every drift test below passes, with `SKEIN_CODEGEN_WRITE` unset, and `git status --porcelain` shows only your edits. A generated
  byte that moves is red: the fix is in the generator.
- `check_function_lengths` (`skein-codegen/src/tests.rs`) passes unchanged.
- The branch's net lines fall. The estimate is -60, and it's the tightest of this part: if it won't fall, stop and ask, with the
  measured size.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `skein/docs/development/debt/D035-codegen-diff-and-emit.md`
- `skein/docs/design/codec.md`, 6

## Owns and may change

- **Owns:** the files in `files`.
- **They stay:** the `emit_tests.rs` split of its `test_helpers.rs` template, which cuts a source template, not emitted text;
  `patch/module.rs`'s reads of layout and hash count; `patch/renumber.rs` and `skeleton.rs`, on the author's own file; and what reads patch
  text, paths, arguments or family names.
- **The module-wide `format_push_string` expects stay,** in `emit_tests.rs` and wherever the `push_str(&format!` remain: rewriting 96 sites
  adds length. A `#[expect]` that your work empties is an error, so remove that one.
- **`skein/testing/skein-codegen/src/emit_limits.rs` already chunks directly.** That's the pattern, and it's unchanged.

## Steps

1. In `emit_functions.rs`, a sketch of the outer layer:

   ```rust
   pub(crate) struct Statement { pub text: String, pub boundary: bool, pub decoded: Option<String> } // text: one or more lines
   pub(crate) enum Tail { Line(String), Members(Vec<Member>) }
   pub(crate) enum Member { Field(String), Bools(Vec<String>) }                                      // renders decoded_i.f by chunk
   pub(crate) struct Method { pub name: &'static str, pub head: Vec<String>, pub body: Vec<Statement>, pub tail: Tail }
   pub(crate) fn render(record: &Record, method: &Method, out: &mut String, helpers: &mut String);
   pub(crate) fn fold(signature: &str, name: &str, var: &str, statements: &[String], tail: &str, out: &mut String);
   ```

2. **What keeps the bytes:**

   - A `Statement` is one maximal balanced run of lines. The 99 and 100, 70 and 90 thresholds stay, counted in lines.
   - `measure` and `encode`: a `let field_` statement has `boundary: true`, and other lines are `false`.
   - `check`: each top-level line is its own statement, so `emit_check` takes `&mut Vec<Statement>`. A Text field's two guards stay two
     statements: grouping by field moves split points. `emit_types` gains a `Guard { condition, action }`, so an `Option` over one
     guard needs no `strip_prefix`.
   - `decode_from`: each `let decoded_f` is a statement with `decoded: Some(f)`. It splits only in `skein/tests/codec/src/lints/v1.rs`,
     so that file alone pins it.
   - `emit_bounds` calls `fold` from its family-level functions, which fire at 96 declarations or more (11 checked-in files). Its
     `Self::` branch and `check`'s static-parameter branch look dead by construction: if your rewrite drops them and every file stays
     the same, report it.
   - `emit_codec` picks `limits` or `_limits` before it writes.
   - `patch/module.rs` decides before it emits.

3. Convert a method at a time, running the narrow drift test each time:
   `heavy cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'`.

4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-codegen --all-targets -- -D warnings`; `heavy cargo nextest
   run -p skein-codegen -p skein-codec-tests -p skein-codec-patches -p skein-oauth-world -p skein-ui-example-view -p jig-worker-wire-world -p
   jig-protocol-accounts-world -p jig-store-codecs-world -p jig-client-wire-world -p smith-codecs-world -p temper-codecs-world`: every tree,
   since each holds generated code; the citations check. Commit and push.

## Report

The common note's items, each pass of the drift tests, and every branch you dropped as dead. Nothing to break by hand: the drift
tests are the rule, and you'll have seen them go red while you work; name one. Don't delete the item: `codegen-close` does.
