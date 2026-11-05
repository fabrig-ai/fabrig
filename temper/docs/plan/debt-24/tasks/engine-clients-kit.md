+++
id = "engine-clients-kit"
part = "jig-kits"
kind = "migration"
needs = ["client-kit"]
files = ["jig/testing/jig-engine-clients-kit/**", "jig/tests/engine-clients/src/**", "jig/tests/engine-clients/Cargo.toml", "jig/tests/client-protocol/Cargo.toml", "jig/tests/client-protocol/tests/suite/settings.rs", "jig/tests/conformance/Cargo.toml", "jig/tests/conformance/src/**", "jig/tests/conformance/tests/**", "jig/tests/boundary/src/tests/kits.rs", "jig/tests/boundary/src/tests/graph.rs", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# jig: the engine clients' peer moves to a kit

## Goal

Jig D031's two edges onto `jig-engine-clients-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`.

Acceptance:

- `jig-engine-clients-kit` holds `peer` and `fixtures`, all of the world's `src/`. `jig-engine-clients-world`'s `src/lib.rs` is the re-export and a `//!` line.
- Readers on the kit: `jig-client-protocol-world` (dev: `tests/suite/settings.rs`), `jig-conformance-world` (dev). The world's dev edges onto the core and client kits stay on the world, as its tests read them.
- The two lines leave `PENDING`; the Measure falls by two. `rg -n '^jig-engine-clients-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-engine-clients-kit`, `jig-engine-clients-world`, `jig-client-protocol-world` and `jig-boundary` passes; `jig-conformance-world` runs `heavy cargo check --tests`; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path. `graph.rs`' allowed name list gains the kit, if it asks.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-engine-clients-kit -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
