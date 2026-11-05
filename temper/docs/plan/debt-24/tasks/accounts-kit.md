+++
id = "accounts-kit"
part = "jig-kits"
kind = "migration"
needs = ["client-protocol-kit"]
files = ["jig/testing/jig-protocol-accounts-kit/**", "jig/tests/protocol-accounts/src/**", "jig/tests/protocol-accounts/Cargo.toml", "jig/tests/one-process/Cargo.toml", "jig/tests/one-process/src/accounts.rs", "jig/testing/jig-test/Cargo.toml", "jig/testing/jig-test/tests/live.rs", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# jig: the accounts' fixtures move to a kit

## Goal

Jig D031's two remaining edges onto `jig-protocol-accounts-world`. By the recipe of `temper/docs/plan/debt-24/notes/kits.md`, and `notes/worlds.md`.

Acceptance:

- `jig-protocol-accounts-kit` holds `fixtures` and `world::Story`, and what those read. **`fixtures` has an `owner` link the survey didn't read:** if it drags `owner`, `world`, `referee` and `visitor`, they all move, about 2,000 lines, and that's the task's call. `jig-protocol-accounts-world` keeps the rest and the re-export.
- Readers on the kit: `jig-one-process-world` (`fixtures::{ROOM, limits}`, `world::Story`), `jig-test` (dev: `tests/live.rs`).
- The two lines leave `PENDING`; the Measure falls by two. `rg -n '^jig-protocol-accounts-world\b' jig -g Cargo.toml` reads nothing.
- `heavy cargo nextest run` on `jig-protocol-accounts-kit`, `jig-protocol-accounts-world` and `jig-boundary` passes, and `heavy cargo check -p jig-one-process-world -p jig-test --tests` does; test counts hold.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/notes/worlds.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header, in readers only a manifest line and a crate path.
- **`accounts-lending` and `accounts-entry`** change `skein-oauth-accounts`, which this world reads. They don't change its public surface: nothing here meets them.

## Steps

The recipe; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-protocol-accounts-kit -p jig-protocol-accounts-world -p jig-boundary --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items, the kits note's lines; how far `fixtures` dragged. A pure move: no rule to break by hand.
