+++
id = "http-kit"
part = "skein-kits"
kind = "migration"
needs = ["kit-packages"]
files = ["skein/testing/skein-http-kit/**", "skein/tests/http/src/lib.rs", "skein/tests/http/src/reference.rs", "skein/tests/http/Cargo.toml", "skein/tests/llm/Cargo.toml", "skein/tests/llm/tests/suite/fake.rs", "skein/tests/llm/tests/suite/bounded_peer_memory.rs", "Cargo.lock"]
effort = "medium"
minutes = 20
+++

# skein: the HTTP world's reference reader moves to a kit

## Goal

Skein D028's edge onto `skein-http-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`.

Acceptance:

- `skein-http-kit` holds `reference`, and nothing else unless it needs it. `skein-http-world` keeps the rest, with the re-export, so its six `crate::reference` readers and its seven test files stand.
- `skein-llm-world` reads the kit (dev: `tests/suite/fake.rs`, `tests/suite/bounded_peer_memory.rs`).
- `rg -n '^skein-http-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-http-world -p skein-http-kit -p skein-llm-world` passes; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header.

## Steps

The recipe; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-http-kit -p skein-http-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
