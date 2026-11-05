+++
id = "client-protocol-kit"
part = "jig-kits"
kind = "migration"
needs = ["engine-clients-kit"]
files = ["jig/testing/jig-client-protocol-kit/**", "jig/tests/client-protocol/src/**", "jig/tests/client-protocol/Cargo.toml", "jig/tests/channel/Cargo.toml", "jig/tests/channel/src/**", "jig/tests/one-process/Cargo.toml", "jig/tests/one-process/src/**", "jig/tests/ops/Cargo.toml", "jig/tests/ops/src/**", "jig/testing/jig-test/Cargo.toml", "jig/testing/jig-test/tests/**", "jig/testing/jig-test/src/**", "jig/tests/boundary/src/tests/kits.rs", "jig/tests/boundary/src/tests/graph.rs", "Cargo.lock"]
effort = "medium"
minutes = 50
+++

# jig: the protocol client moves to a kit

## Goal

Jig D031's four edges onto `jig-client-protocol-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`. `native` needs `front` and `fixtures`, and `fixtures` needs `client`, so everything moves but `public`.

Acceptance:

- `jig-client-protocol-kit` holds all of the world's `src/` but `public`, about 5,300 lines. `jig-client-protocol-world` keeps `public` and the re-export, and its suite stands.
- Readers on the kit: `jig-channel-world`, `jig-one-process-world`, `jig-ops-world`, `jig-test` (dev).
- The four lines leave `PENDING`; the Measure falls by four. `rg -n '^jig-client-protocol-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-client-protocol-kit`, `jig-client-protocol-world`, `jig-channel-world`, `jig-ops-world` and `jig-boundary` passes, and `heavy cargo check -p jig-one-process-world -p jig-test --tests` does; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path. `graph.rs`' allowed name list gains the kit, if it asks.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-client-protocol-kit -p jig-client-protocol-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
