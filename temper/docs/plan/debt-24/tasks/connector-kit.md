+++
id = "connector-kit"
part = "jig-kits"
kind = "migration"
needs = ["kit-packages", "world-exemptions"]
files = ["jig/testing/jig-test-connector-kit/**", "jig/tests/test-connector/src/**", "jig/tests/test-connector/Cargo.toml", "jig/tests/core/Cargo.toml", "jig/tests/core/src/**", "jig/tests/core/tests/**", "jig/tests/conformance/Cargo.toml", "jig/tests/conformance/src/**", "jig/testing/jig-test-domain/Cargo.toml", "jig/testing/jig-test-domain/src/**", "jig/testing/jig-test-store/Cargo.toml", "jig/testing/jig-test-store/src/**", "jig/testing/jig-test-store-files/Cargo.toml", "jig/testing/jig-test-store-files/src/**", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# jig: the connector world's paths and kinds move to a kit

## Goal

Jig D031's five edges onto `jig-test-connector-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`.

Acceptance:

- `jig-test-connector-kit` holds `LIMITS`, `path`, `kind` and `config` from `src/world.rs`. `World` stays in the world, used only by its own tests. The kit depends on `jig-test-connector` and `skein-lib`, so `jig-test-domain`'s dev edge doesn't cycle.
- Readers on the kit: `jig-core-world`, `jig-conformance-world`, `jig-test-domain` (dev), `jig-test-store` (dev), `jig-test-store-files` (dev).
- The five `jig-test-connector-world` lines leave `PENDING`; the Measure falls by five. `rg -n '^jig-test-connector-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-test-connector-kit`, `jig-test-connector-world`, `jig-test-domain`, `jig-test-store`, `jig-test-store-files` and `jig-boundary` passes, and `-p jig-core-world` and `-p jig-conformance-world` do; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/domain/testing.md`, 7

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path. D035's `connector-names` takes the bare numbers afterward.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-test-connector-kit -p jig-test-connector-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.

## Answers

2026-11-05. May the scope include crate-path-only edits in `jig/tests/core/tests/**`? Yes. `jig-core-world` is a reader this spec names, and recipe step 4 renames the crate path in that package's `.rs` files; the header left its test files out, and the edge can't go without them. The header now lists them. Only the crate path changes there: its 41 uses across `jig/tests/core` are all `path` and `kind`, which the kit holds, so assertions, test names and counts stay. `rustfmt` may rewrap a line it shortens. Finish the fifth edge on the pushed branch: the core manifest line, `src/**`, `tests/**`, its `PENDING` line.
