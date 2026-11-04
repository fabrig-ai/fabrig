+++
id = "patch-check-scratch"
part = "codegen"
kind = "feature"
needs = []
files = ["skein/testing/skein-codegen/tests/suite/patch_check.rs"]
effort = "medium"
minutes = 25
+++

# `patch_check` leaves no crate behind when it fails

## Goal

`skein/testing/skein-codegen/tests/suite/patch_check.rs` makes `skein-patch-check-<pid>-<name>` under the
temporary directory and removes it after its assertions, so a failing test leaves it, and a reused pid then
fails `expect("unique scratch crate")`. `skein_scratch::Scratch` removes on drop, a failing test's included.
(Bug 18's other half, re-checked: holds for a failing test; every passing path removes its crate.)

Acceptance:

- `crate_files` returns `skein_scratch::Scratch`, label `patch-check-{name}`. Callers pass `.path()`. The six
  `remove_dir_all` calls go.
- The test below fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/testing.md`, 6 (`skein-scratch`).

## Owns and may change

- Owns: `tests/suite/patch_check.rs`. `skein-scratch` is already a dev-dependency.
- `crate_files`, `findings` and `assert_files` are private to the file. `patch_run.rs`, `patch_generated.rs`
  and `story.rs` roll their own directories too: not this bug, `## Left`.

## Steps

1. Write `a_failed_check_leaves_no_crate_behind`: `catch_unwind(AssertUnwindSafe(|| findings("leak", &[("schema/shelf-v1.schema",
   V1)], &Landed::none(), &[])))` with expectations that fail, assert it panicked, then scan
   `std::env::temp_dir()` for a name with `patch-check`, this process's id and `leak`, in either order. Run on
   the old code and quote the failure.
2. The change.
3. `heavy cargo nextest run -p skein-codegen --test suite -E 'test(/patch_check/)'`. `cargo fmt`, `heavy cargo
   clippy -p skein-codegen --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
