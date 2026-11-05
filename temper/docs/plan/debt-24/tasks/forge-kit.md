+++
id = "forge-kit"
part = "skein-kits"
kind = "migration"
needs = ["tls-kit", "hand-face"]
files = ["skein/testing/skein-forge-kit/**", "skein/tests/forge/machine/src/**", "skein/tests/forge/machine/Cargo.toml", "skein/tests/forge/client/src/world.rs", "skein/tests/forge/client/src/translate.rs", "skein/tests/forge/client/src/lib.rs", "skein/tests/forge/protocol/Cargo.toml", "skein/tests/forge/protocol/src/**", "skein/tests/forge/protocol/tests/suite/protocol.rs", "skein/tests/forge/setup/Cargo.toml", "skein/tests/forge/setup/src/**", "skein/tests/forge/setup/tests/**", "Cargo.lock"]
effort = "medium"
minutes = 45
+++

# skein: the forge machine world's library moves to a kit

## Goal

Skein D028's edges onto `skein-forge-machine-world` and `skein-forge-client-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`. `joint` needs `hooks`, `github` and `pipe`, and a temper test reads `machine`, so the kit takes the whole library but what's the world's alone.

Acceptance:

- `skein-forge-kit` holds `fixtures`, `github`, `hooks`, `joint`, `machine`, `peer`, `pipe` and `tls`, and `client`, a new module with the forge client world's `LIMITS` and `translate::commit`, whose world reads them from it. `skein-forge-machine-world` keeps `memory` and `hand` and the re-export; its suites stand.
- Readers on the kit: `skein-forge-protocol-world` and `skein-forge-setup-world`, on the kit's `client`, `joint`, `pipe`, `fixtures` and `github`.
- `rg -n '^skein-forge-(machine|client)-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-forge-kit -p skein-forge-machine-world -p skein-forge-client-world -p skein-forge-protocol-world -p skein-forge-setup-world` passes; test counts hold. Seeds replay: nothing changes an order of draws.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `skein/docs/design/testing.md`, 6

## Owns and may change

- **Owns:** the files in the header.
- **Temper stays:** `temper-service`'s tests and `temper-engine-protocol-forge-world` read through the re-export until `skein-world-readers`.
- **`hand.rs` stays in the world:** nothing else reads it. It names the kit where it reads a module that moved. The `hand-*` tasks hold it and `tests/suite/hand/**` at once: touch only `hand.rs`'s `use` lines.
- **`memory`** may read a kit item that's `pub(crate)`: it becomes `pub`.

## Steps

The recipe; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-kit -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above; `heavy cargo check -p temper-service --tests` for the re-export. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
