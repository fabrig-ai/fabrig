+++
id = "hearth-builder-switch"
part = "hearth-kits"
kind = "migration"
needs = ["hearth-kit-render", "hearth-kit-repository"]
files = ["hearth/tests/builder/Cargo.toml", "hearth/tests/builder/src/fixtures/goldens.rs", "hearth/tests/builder/src/fixtures/budgets.rs", "hearth/tests/builder/src/fixtures/properties.rs", "hearth/tests/builder/tests/suite/goldens.rs", "hearth/tests/builder/tests/suite/properties.rs", ".config/fixture-readers.toml", ".config/gate/always.txt", "Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# hearth: the builder world stops depending on the render world

## Goal

The builder world's readers name `hearth-test-render`, and the last edge from it to a world goes. It still reads the goldens by path, so the gate learns that.

Acceptance:

- `hearth/tests/builder/src/fixtures/goldens.rs:3`, `budgets.rs:6`, `properties.rs:2, 13, 27, 47`, `tests/suite/goldens.rs:2` and `tests/suite/properties.rs:2` name `hearth_test_render`, each changed only in its path. No other line of them changes. `include_*!("../../../render/…")` stays.
- `hearth/tests/builder/Cargo.toml` has `hearth-test-render` and no `hearth-render-world`. The root `Cargo.toml`'s `hearth-render-world` workspace-dependency line goes, if no manifest uses it.
- `.config/gate/always.txt` gains `hearth-builder-world hearth/tests/render/` (`.config/gate/README.md`, its lists): the builder reads goldens by path, and loses reaching them through the dependency. The `hearth-render-world hearth/` line stays.
- `.config/fixture-readers.toml`: the `hearth-builder-world` reader of `hearth/tests/render/src/fixtures/*.rs` goes, and that block's comment drops "builder suite/properties.rs also reads generated repositories".
- The item's **Measure** falls by one.
- `heavy cargo nextest run -p hearth-builder-world` and the same under `--profile fuzzy` pass, with the same tests and no assertion changed.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`, binding
- `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9; `skein/docs/design/testing.md`, 6
- The shape to copy: `git show c721602b44` created `hearth-test-fixtures`
- `.config/gate/README.md`, its lists

## Owns and may change

- **Owns:** the files in `files`.
- **A `use` of an item the kit lacks** (the render world's other modules): that's a question. The survey found none.

## Steps

1. The paths, manifests and two config files.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-builder-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**; the test counts before and after.
