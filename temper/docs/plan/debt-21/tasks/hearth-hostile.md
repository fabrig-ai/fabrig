+++
id = "hearth-hostile"
part = "hearth-kits"
kind = "migration"
needs = []
files = ["hearth/testing/hearth-test-fixtures/src/hostile.rs", "hearth/testing/hearth-test-fixtures/src/lib.rs", "hearth/tests/config/src/generate.rs", "hearth/tests/config/tests/suite/fuzzy/goldens.rs", "hearth/tests/config/tests/suite/fuzzy/documents.rs", "hearth/tests/config/Cargo.toml", "hearth/tests/render/src/repositories.rs", "hearth/tests/render/Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# hearth: the render world stops depending on the config world

## Goal

`hearth-render-world` uses one thing of `hearth-config-world`: `generate::mutate`. `mutate` and `HOSTILE` go to `hearth-test-fixtures`, and the edge goes.

Acceptance:

- `hearth/testing/hearth-test-fixtures/src/hostile.rs` holds `mutate` and `HOSTILE`, moved whole. `lib.rs` has `pub mod hostile;`, and its opening comment's sense still fits.
- `hearth/tests/render/src/repositories.rs` calls `hearth_test_fixtures::hostile::mutate`. `hearth/tests/render/Cargo.toml` has no `hearth-config-world` line, and has `hearth-test-fixtures` if it lacked one.
- The config world's two callers (`tests/suite/fuzzy/goldens.rs`, `tests/suite/fuzzy/documents.rs`) call it there too, and `goldens.rs`'s `use` drops `generate` if nothing else needs it. No re-export is left in `generate`. A `[dev-dependencies]` line in the config world's manifest stays where it is, if it's already there.
- The item's **Measure** reads 3.
- `heavy cargo nextest run -p hearth-test-fixtures -p hearth-config-world -p hearth-render-world` and the same under `--profile fuzzy` pass, with no assertion changed: the same draws from the same seeds give the same bytes.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`, binding
- `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9; `skein/docs/design/testing.md`, 6
- The shape to copy: `git show c721602b44` created `hearth-test-fixtures`

## Owns and may change

- **Owns:** the files in `files`.
- **`hearth-test-fixtures` already depends on `skein-lib`,** which holds `Rng`. It gains no dependency.
- `hearth/tests/render/src/repositories.rs` itself moves in `hearth-kit-render`, which waits for this.

## Steps

1. The move, the callers, the manifests.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-test-fixtures -p hearth-config-world -p hearth-render-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**.
