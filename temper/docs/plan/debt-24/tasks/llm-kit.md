+++
id = "llm-kit"
part = "skein-kits"
kind = "migration"
needs = ["http-kit"]
files = ["skein/testing/skein-llm-kit/**", "skein/tests/llm/src/**", "skein/tests/llm/Cargo.toml", "skein/tests/tls/Cargo.toml", "skein/tests/tls/tests/suite/fake_peer.rs", "skein/tests/oauth/Cargo.toml", "skein/tests/oauth/tests/suite/**", "skein/tests/fake-peers/Cargo.toml", "skein/tests/fake-peers/tests/suite/llm.rs", "skein/tests/llm-connection/Cargo.toml", "skein/tests/llm-connection/src/**", "skein/tests/llm-connection/tests/**", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# skein: the LLM world's library moves to a kit

## Goal

Skein D028's skein edges onto `skein-llm-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`.

Acceptance:

- `skein-llm-kit` holds all of the world's `src/`: `lib.rs`'s items and `fake`, with `fake`'s unit tests. `skein-llm-world`'s `src/lib.rs` is the re-export and a `//!` line, and its 29 test files stand.
- Readers on the kit: `skein-tls-world` (dev), `skein-oauth-world` (dev), `skein-fake-peers-tests` (dev), `skein-llm-connection-world`.
- `rg -n '^skein-llm-world\b' skein -g Cargo.toml` reads nothing.
- `heavy cargo nextest run -p skein-llm-kit -p skein-llm-world -p skein-tls-world -p skein-oauth-world -p skein-fake-peers-tests -p skein-llm-connection-world` passes; test counts hold. `tools/tiers/packages.toml`'s `live-skein` row still selects the world's live binary: its name is unchanged.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header.
- **Smith, jig, temper stay:** they read through the re-export until `skein-world-readers`.
- **`smith/benchmarks/tasks/repository/shell-cwd/grader`** is outside the workspace and pinned: leave it.

## Steps

The recipe; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm-kit -p skein-llm-world --all-targets -- -D warnings`; the tests above; `heavy cargo check -p smith-protocol-llm-world --tests` for the re-export. Commit and push.

## Report

The common note's items, the kits note's lines. A pure move: no rule to break by hand.
