+++
id = "names-once"
part = "codegen-once"
kind = "feature"
needs = []
files = ["skein/testing/skein-codegen/src/emit_limits.rs", "skein/testing/skein-codegen/src/emit_bounds.rs", "skein/testing/skein-codegen/src/lib.rs", "skein/testing/skein-codegen/src/patch/module.rs", "skein/tests/codec/tests/support/generic.rs"]
effort = "medium"
minutes = 20
+++

# skein: `snake`, `pascal` and `rust_type` are defined once

## Goal

The last clause of `skein/docs/development/debt/D035-codegen-diff-and-emit.md`'s **Paid when**. Generated code doesn't change by a byte.

Acceptance:

- `rg -c 'fn (snake|pascal)\b' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'` reads 2, and
  `rg -c 'fn rust_type\b' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'` reads 1.
- The drift tests below pass, with `SKEIN_CODEGEN_WRITE` unset: no generated file changes. `git status --porcelain` shows only your
  edits.
- The branch's net lines fall by at least 35.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `skein/docs/development/debt/D035-codegen-diff-and-emit.md`

## Owns and may change

- **Owns:** the files in `files`.
- **`snake` keeps `patch/module.rs`'s form,** which also maps `.` and `-` to `_`. `emit_limits`'s form only ever sees declaration
  names, and `check.rs` limits those to ASCII alphanumerics, so every checked schema spells the same. `module.rs`'s callers need the
  arm, so it's the one to keep.
- **Home:** `snake` and `pascal` stay in `emit_limits.rs`, `pub use emit_limits::{pascal, snake};` in `lib.rs`, an addition; `emit_bounds.rs`
  uses `crate::emit_types::rust_type`. `skein/tests/codec` has `skein-codegen` as a dev-dependency, and `generic.rs` is compiled only
  from `tests/suite/main.rs`, so it imports them.

## Steps

1. Move each, keeping every caller.

2. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-codegen -p skein-codec-tests --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-codegen -p skein-codec-tests -p skein-codec-patches`; the citations check. Commit and
   push. **This departs from the implementer's text:** no generated output can change here without those failing, so the other
   trees' drift tests are `emit-structure`'s to run.

## Report

The common note's items. A rewrite: nothing to break by hand. Don't delete the item: `codegen-close` does.
