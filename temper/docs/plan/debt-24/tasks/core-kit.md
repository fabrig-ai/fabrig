+++
id = "core-kit"
part = "jig-kits"
kind = "migration"
needs = ["connector-kit"]
files = ["jig/testing/jig-core-kit/**", "jig/tests/core/src/**", "jig/tests/core/Cargo.toml", "jig/tests/client/Cargo.toml", "jig/tests/client/src/**", "jig/tests/client-protocol/Cargo.toml", "jig/tests/client-protocol/src/**", "jig/tests/conformance/Cargo.toml", "jig/tests/conformance/src/**", "jig/tests/engine-clients/Cargo.toml", "jig/tests/engine-clients/tests/**", "jig/tests/ops/Cargo.toml", "jig/tests/ops/src/joint/engine.rs", "jig/tests/store/Cargo.toml", "jig/tests/store/tests/suite/readers.rs", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 50
+++

# jig: the core world's library moves to a kit

## Goal

Jig D031's six edges onto `jig-core-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`. `world`, `peers`, `reads` and `referee` cycle, and `fixtures` needs `stories`, so nearly all of `src/` moves together.

Acceptance:

- `jig-core-kit` holds all of the world's `src/`, about 8,500 lines, but what nothing outside the world reads and nothing moved reads: `agents_tools` and `reference`, if a read shows that. `jig-core-world` keeps those and the re-export, and its 87 test files stand, including those other drafts hold (`tests/suite/stories.rs`, `tests/stories/note_room.rs`).
- Readers on the kit: `jig-client-world`, `jig-client-protocol-world`, `jig-conformance-world`, `jig-engine-clients-world` (dev), `jig-ops-world`, `jig-store-world` (dev).
- The six `jig-core-world` lines leave `PENDING`; the Measure falls by six. `rg -n '^jig-core-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-core-kit`, `jig-core-world`, `jig-store-world`, `jig-engine-clients-world`, `jig-conformance-world` and `jig-boundary` passes, and `heavy cargo check -p jig-client-world -p jig-client-protocol-world -p jig-ops-world --tests` does; test counts hold. Seeds replay.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/domain/testing.md`, 7

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path.
- **`jig/tests/client` and `jig/tests/client-protocol` may be held by `seed-projects` at once:** you touch only a crate's name in them; fab merges.
- **Temper's one reader** stays until `temper-kit-names`.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-core-kit -p jig-core-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
