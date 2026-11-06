+++
id = "channel-kit"
part = "jig-kits"
kind = "migration"
needs = ["accounts-kit", "skein-world-readers"]
files = ["jig/testing/jig-channel-kit/**", "jig/tests/channel/src/**", "jig/tests/channel/Cargo.toml", "jig/tests/one-process/Cargo.toml", "jig/tests/one-process/src/**", "jig/testing/jig-test/Cargo.toml", "jig/testing/jig-test/tests/**", "jig/testing/jig-test/src/**", "jig/tests/boundary/src/tests/graph.rs", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 50
+++

# jig: the channel world's library moves to a kit

## Goal

Jig D031's last two edges. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`. `fixtures` needs `world`, `stories`, `referee` and `live_*`, and `world` needs it back, so all of `src/` moves.

Acceptance:

- `jig-channel-kit` holds all of the world's `src/`, about 6,260 lines. `jig-channel-world`'s `src/lib.rs` is the re-export and a `//!` line.
- Readers on the kit: `jig-one-process-world`, `jig-test` (dev).
- `graph.rs`' allowed name list gains the kit beside `jig-channel-world`.
- `PENDING` is empty of edges: the Measure reads 0. `rg -n '^jig-channel-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-channel-kit`, `jig-channel-world` and `jig-boundary` passes, and `heavy cargo check -p jig-one-process-world -p jig-test --tests` does; test counts hold. Seeds replay.
- `heavy cargo check -p temper-one-process-world --tests` (its name from `temper/tests/one-process/Cargo.toml`) passes through the re-export.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path. If the Measure isn't 0 here, a line another task should've cut stands: name it in your report as left; the item stays until `jig-kits-close`.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-channel-kit -p jig-channel-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines; the Measure. A pure move: no rule to break by hand.
