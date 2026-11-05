+++
id = "hearth-kit-repository"
part = "hearth-kits"
kind = "migration"
needs = ["hearth-kits-lay"]
files = ["hearth/testing/hearth-test-repository/**", "hearth/tests/builder/src/fixtures.rs", "hearth/tests/builder/src/fixtures/binary.rs", "hearth/tests/builder/src/fixtures/resolution.rs", "hearth/tests/builder/src/fixtures/reserved.rs", "hearth/tests/builder/Cargo.toml", "hearth/crates/hearth/Cargo.toml", "hearth/crates/hearth/tests/end_to_end.rs", "hearth/crates/hearth/tests/refusals.rs", ".config/fixture-readers.toml", "Cargo.lock"]
effort = "high"
minutes = 75
+++

# hearth: `hearth`'s end-to-end tests stop depending on the builder world

## Goal

`hearth`'s `end_to_end` and `refusals` tests depend on the builder world for a scratch repository, a run of the built binary and a few constants. These move to `hearth-test-repository`, about 515 lines, and the edge goes. So does the cycle from `hearth` through the builder world and back, since the kit names no `hearth`.

Acceptance:

- `hearth/testing/hearth-test-repository` holds what the note's table gives it, and depends on no `hearth` lib or world.
- Its uses from `hearth`'s two tests are all there: `binary` (which returns `Ended`), `Repository::make` and `empty`, `work_tree`, `scratch`, `head`, `next`, `recorded_test_min`, `lines_starting`, `THIN`, `THIN_TEXT`, `MOTD`, `MOTD_KIND`, `MOTD_DIFF_TEXT`.
- Each of the two tests' first line reads `use hearth_test_repository::{self as fixtures, Repository};`, and no other line of theirs changes. `hearth/crates/hearth/Cargo.toml`'s dev-dependency on the builder world becomes one on the kit. It keeps `skein-scratch` for `src/tests.rs`.
- The builder world's fixtures keep `pub use` at their old paths, and no file under `hearth/tests/builder/tests/` changes.
- `.config/fixture-readers.toml`: the `hearth` readers of the builder's fixtures (`end_to_end`, `refusals`) go, and that block's comment drops "and hearth's binary tests".
- The item's **Measure** falls by one.
- `heavy cargo nextest run -p hearth --test end_to_end --test refusals`, `heavy cargo nextest run -p hearth-builder-world` and the latter under `--profile fuzzy` pass, with the same tests and no assertion changed.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`, binding
- `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9; `skein/docs/design/testing.md`, 6
- The shape to copy: `git show c721602b44` created `hearth-test-fixtures`
- `hearth/crates/hearth/tests/end_to_end.rs` and `refusals.rs`: the first lines, and `env!("CARGO_BIN_EXE_hearth")`, which stays in the tests and passes the path in

## Owns and may change

- **Owns:** the files in `files`.
- **The edits a move alone can't do, each allowed here:**
  - Six private-field sites become accessors (`fixtures.rs:223, 301, 303, 314, 518`, `resolution.rs:113`).
  - `git` and `git_output` become `pub`, for `fixtures.rs:489, 491, 506, 507`.
  - `reserved.rs` splits, with a glob re-export in the world.
  - The eight `include_bytes!` prefixes in `resolution.rs:16-23` become `../../repos/test/…`, resolved from the kit.
  - `Laid::Record` is spelled `hearth_lib::encode::Digest`, the same type, so that no line of the kit names `hearth::service`.
- **All code naming the `hearth` lib stays in the world:** `render`, `Holding`, `Scenario`. If a listed item turns out to need one, stop and ask.
- **`hearth-service-builder/src/lib.rs` stays out of it,** as the note says.

## Steps

1. The kit's root, then `binary`, `resolution`, `reserved`. The world's `pub use`.
2. `hearth`'s two tests and manifest; the readers file.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-test-repository -p hearth-builder-world -p hearth --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**; the test counts before and after.
