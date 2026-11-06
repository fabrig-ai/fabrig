+++
id = "client-kit"
part = "jig-kits"
kind = "migration"
needs = ["core-kit", "seed-projects"]
files = ["jig/testing/jig-client-kit/**", "jig/tests/client/src/**", "jig/tests/client/Cargo.toml", "jig/tests/client-protocol/Cargo.toml", "jig/tests/client-protocol/src/**", "jig/tests/client-protocol/tests/**", "jig/tests/ops/Cargo.toml", "jig/tests/ops/src/**", "jig/tests/ops/tests/**", "jig/tests/conformance/Cargo.toml", "jig/tests/conformance/src/**", "jig/tests/conformance/tests/**", "jig/tests/engine-clients/Cargo.toml", "jig/tests/engine-clients/tests/**", "jig/tests/boundary/src/tests/graph.rs", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 70
+++

# jig: the client world's library moves to a kit

## Goal

Jig D031's four edges onto `jig-client-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`. `harness`, `joint`, `stories`, `scripted`, `observe`, `test`, `model` and `random` are mutually cyclic, and `jig/docs/design/client/domain/testing.md` has applications' client worlds run on them, so the whole `src/` moves.

Acceptance:

- `jig-client-kit` holds all of the world's `src/`, about 18,800 lines. `jig-client-world`'s `src/lib.rs` is the re-export and a `//!` line, and its suite stands.
- Readers on the kit: `jig-client-protocol-world`, `jig-ops-world`, `jig-conformance-world` (dev), `jig-engine-clients-world` (dev).
- `graph.rs`' `client_world_joins_only_the_client_and_test_roles` reads `jig-client-kit`'s dependencies, with the same list; `client_names`' allowed list gains the kit.
- The four `jig-client-world` lines leave `PENDING`; the Measure falls by four. `rg -n '^jig-client-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-client-kit`, `jig-client-world`, `jig-client-protocol-world`, `jig-engine-clients-world`, `jig-ops-world` and `jig-boundary` passes; test counts hold. `jig-conformance-world` runs `heavy cargo check --tests`.
- `heavy cargo check -p temper-client-world --tests` (its name from `temper/tests/client/Cargo.toml`) passes through the re-export.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/client/domain/testing.md`, 5

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path.
- **`jig-client-browser-world` and `jig-client-wire-world`** don't read this world: they stay.
- **A moved item's budget:** the kit's unit tests run in the focused suite, as they did in the world. Nothing's filter selects them by this world's name; if one does, stop and ask.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-client-kit -p jig-client-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines; the pin's edit. A pure move: no rule to break by hand.
