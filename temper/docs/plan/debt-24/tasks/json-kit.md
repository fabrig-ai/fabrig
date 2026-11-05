+++
id = "json-kit"
part = "skein-kits"
kind = "migration"
needs = ["kit-packages"]
files = ["skein/testing/skein-json-kit/**", "skein/tests/json/src/**", "skein/tests/json/Cargo.toml", "skein/crates/skein-llm/Cargo.toml", "skein/crates/skein-llm/src/translate.rs", "skein/tests/http/Cargo.toml", "skein/tests/http/tests/suite/stack.rs", "skein/tests/protocol/Cargo.toml", "skein/tests/protocol/src/world.rs", "skein/tests/protocol/src/scenario.rs", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# skein: the JSON world's reference and generator move to a kit

## Goal

Skein D028's three edges onto `skein-json-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`.

Acceptance:

- `skein-json-kit` holds `generate`, `reference` and, from the world's `lib.rs`, `write`, `Outcome`, `Decoded`, `COLLECTOR_CAPS` and what those need. `skein-json-world` holds `transcript`, `world` and the re-export.
- Readers on the kit: `skein-llm` (dev: `src/translate.rs`, `generate`), `skein-http-world` (dev: `tests/suite/stack.rs`), `skein-protocol-world` (`src/world.rs`, `src/scenario.rs`).
- `rg -n '^skein-json-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-json-world -p skein-json-kit -p skein-llm --lib`, and `-p skein-http-world`, `-p skein-protocol-world`, pass; test counts hold as the note says.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`
- `skein/docs/design/testing.md`, 6

## Owns and may change

- **Owns:** the files in the header. `skein-http-world`'s other readers of its own `generate` aren't this.
- **The cycle:** `skein-llm`'s dev edge is safe, since the kit depends only on `skein-json` and `skein-lib`. If it would need more, stop and ask.

## Steps

The recipe; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-json-kit -p skein-json-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
