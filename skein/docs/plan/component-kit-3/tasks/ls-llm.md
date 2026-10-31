+++
id = "ls-llm"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract"]
files = ["skein/crates/skein-llm/**"]
effort = "medium"
minutes = 20
+++

## Goal

skein-llm's hex-digit writer and its OpenAI decimal reader move onto lib. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `src/affinity.rs`: `fn hex(nibble: u8) -> u8` → `skein_lib::hex::digit`.
- `src/openai/common.rs`: `pub(crate) fn decimal(bytes: &[u8]) -> Option<u64>` and its callers → `Decimal::parse`.

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.

## Owns and may change

- Owns `skein/crates/skein-llm/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-llm`. Commit and push.

## Report

As `notes/lib-sites.md` says.
