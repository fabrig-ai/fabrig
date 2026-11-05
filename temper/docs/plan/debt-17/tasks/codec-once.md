+++
id = "codec-once"
part = "codec-once"
kind = "feature"
needs = []
files = ["skein/tests/codec/build.rs", "skein/tests/codec/Cargo.toml", "skein/tests/codec/tests/suite/names.rs", "skein/tests/codec/tests/suite/main.rs", "skein/crates/skein-codec/src/lib.rs", "skein/docs/development/debt/D034-codec-double-generation.md", "skein/docs/development/debt/D028-worlds-as-kits.md"]
effort = "medium"
minutes = 15
+++

# skein: tests/codec generates its fixtures once, and skein-codec compiles none of them

## Goal

Pay `skein/docs/development/debt/D034-codec-double-generation.md`. `suite/drift.rs` already requires each of the six
committed modules to equal the generator's output, so `names.rs` compares a copy with itself.

Acceptance:

- `skein/tests/codec/build.rs` and `skein/tests/codec/tests/suite/names.rs` are gone, and `skein/tests/codec/Cargo.toml` has
  no `[build-dependencies]`.
- `rg -n '#\[path|extern crate self' skein/crates/skein-codec/src/lib.rs` finds nothing.
- `cargo tree -p skein-codec-tests -e build` shows no `skein-codegen`, and `Cargo.lock` doesn't move: `skein-codegen`
  stays a dev dependency for `drift.rs`.
- Every test left in `skein-codec` and `skein-codec-tests` passes as it is.
- `skein/docs/development/debt/D034-codec-double-generation.md` is deleted.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D034-codec-double-generation.md`
- `skein/docs/design/codec.md`, 6: it does not name the second generation, so it needs no change.

## Owns and may change

- **Owns:** the files in this spec's header.
- **The one approved test deletion:** what `names.rs` compiled (`bound_tests::bounds_of_type_*`, `family_bounds_match` and
  the generated modules' second run of `golden_tests`), and the `golden_tests` that `generated_tests` ran inside
  `skein-codec`. They stay covered:
  - `suite/drift.rs` holds all six families' text to the generator's;
  - `skein-codec-tests`' library still compiles the same six modules and runs their `golden_tests`;
  - `build.rs`'s check for the old names `worst_case_bytes` and `worst_case_heap` is
    `every_bound_name_is_reserved` in `skein/testing/skein-codegen/src/tests.rs`.

  Check those three before you delete. If one doesn't hold, stop and ask.
- If a unit test in `skein-codec/src/` turns out to read `generated_tests` or the `skein_codec` self name (at this plan's
  base none does), stop and ask.
- **`skein/docs/development/debt/D028-worlds-as-kits.md`:** its sentence about the two `#[path]`s only, which keeps its other
  `#[path]`:

  ```markdown
  - One `#[path]` crosses a crate's root:
    `skein/tests/fake-peers/tests/suite/github_memory.rs` compiles a file
    of `skein/tests/fake-forge`. The review counted 10; most are gone.
  ```

## Steps

1. Delete `build.rs`, `tests/suite/names.rs`, its `pub mod names;` in `tests/suite/main.rs` and the `[build-dependencies]`
   table.
2. In `skein-codec/src/lib.rs`, delete the `extern crate self as skein_codec` and `generated_tests` items with their
   attributes. `#![cfg_attr(not(test), no_std)]` stays.
3. `heavy cargo nextest run -p skein-codec -p skein-codec-tests`. The count falls by the deleted tests alone; say by how
   many.
4. Edit `D028`, delete `D034`.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-codec -p skein-codec-tests
   --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, and the three checks above. A removal: nothing to break by hand.
