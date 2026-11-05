+++
id = "hearth-kit-render"
part = "hearth-kits"
kind = "migration"
needs = ["hearth-kits-lay", "hearth-hostile"]
files = ["hearth/testing/hearth-test-render/**", "hearth/tests/render/src/**", "hearth/tests/render/Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 75
+++

# hearth: the goldens' readers and generated repositories become a kit

## Goal

The builder world depends on the render world for its goldens' readers, generated repositories and digest oracle. They move to `hearth-test-render`, about 1,330 lines. The render world uses the kit. `hearth-builder-switch` then cuts the edge.

Acceptance:

- `hearth/testing/hearth-test-render/src/` holds what the note's table gives it, with `git mv` where a file moves whole, so that `git log --follow` finds them. Its manifest lists the note's dependencies.
- `hearth/tests/render/src/lib.rs` and `src/fixtures.rs` keep `pub use` at the old paths, and no file under `hearth/tests/render/tests/` changes.
- The builder world and its manifest don't change: it still reaches them through the render world, until `hearth-builder-switch`.
- `heavy cargo nextest run -p hearth-render-world -p hearth-builder-world` and the same under `--profile fuzzy` pass, with the same tests and no assertion changed. Check the count: `hearth-test-render` has no tests of its own unless a moved inline `#[cfg(test)]` brought them.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`, binding
- `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9; `skein/docs/design/testing.md`, 6
- The shape to copy: `git show c721602b44` created `hearth-test-fixtures`

## Owns and may change

- **Owns:** the files in `files`.
- **`.config/fixture-readers.toml`:** `hearth/tests/render/src/fixtures/*.rs` still matches files that stay. Its builder reader goes in `hearth-builder-switch`. Don't change it here.
- **`.config/gate/always.txt`'s `hearth-render-world hearth/` stays:** `decoders.rs` stays in the render world.

## Steps

1. The moves, `pub use`s and manifest.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-test-render -p hearth-render-world -p hearth-builder-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the test counts before and after, per package and profile.
