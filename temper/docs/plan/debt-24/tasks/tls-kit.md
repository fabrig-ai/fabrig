+++
id = "tls-kit"
part = "skein-kits"
kind = "migration"
needs = ["llm-kit", "hand-face"]
files = ["skein/testing/skein-tls-kit/**", "skein/tests/tls/src/**", "skein/tests/tls/Cargo.toml", "skein/tests/llm-connection/Cargo.toml", "skein/tests/llm-connection/tests/**", "skein/tests/forge/machine/Cargo.toml", "skein/tests/forge/machine/src/memory.rs", "skein/tests/forge/machine/src/peer.rs", "skein/tests/forge/machine/src/tls.rs", "skein/tests/forge/machine/src/machine.rs", "skein/tests/forge/setup/Cargo.toml", "skein/tests/forge/setup/src/memory.rs", "skein/tests/forge/setup/src/tls.rs", "skein/tests/link/Cargo.toml", "skein/tests/link/src/lib.rs", "skein/tests/oauth-accounts/Cargo.toml", "skein/tests/oauth-accounts/src/world.rs", ".config/fixture-readers.toml", "Cargo.lock"]
effort = "medium"
minutes = 35
+++

# skein: the TLS world's certificates, server, driver and meter move to a kit

## Goal

Skein D028's skein edges onto `skein-tls-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`.

Acceptance:

- `skein-tls-kit` holds `pki`, `server`, `drive` and `meter`. `skein-tls-world` keeps `world`, `native`, `native_pair`, `native_peer`, `records` and the re-export; its ten test files stand.
- **`tests/tls/fixtures/` stays where it is,** and `pki.rs` includes `../../../tests/tls/fixtures/...`, as `skein-fake-peers` does: fourteen fixture rows and two docs read that path. `.config/fixture-readers.toml` gains the kit's package where a row's readers need it (`tools/tiers/design.md`, 9).
- Readers on the kit: `skein-llm-connection-world`, `skein-forge-machine-world`, `skein-forge-setup-world`, `skein-link-world`, `skein-oauth-accounts-world`.
- `rg -n '^skein-tls-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-tls-kit -p skein-tls-world -p skein-llm-connection-world -p skein-forge-machine-world -p skein-forge-setup-world -p skein-link-world -p skein-oauth-accounts-world` passes; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `tools/tiers/design.md`, 9

## Owns and may change

- **Owns:** the files in the header.
- **Smith, jig, temper stay:** they read through the re-export until `skein-world-readers`.
- **`skein-forge-machine-world`'s suites under `tests/`** may be held by `hand-*` tasks at once: touch them only for a `use` that names `skein_tls_world`.

## Steps

The recipe; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-tls-kit -p skein-tls-world --all-targets -- -D warnings`; the tests above; `heavy cargo check -p smith-hosts-world --tests` for the re-export. Commit and push.

## Report

The common note's items, the kits note's lines; the fixture rows changed. A pure move: no rule to break by hand.
