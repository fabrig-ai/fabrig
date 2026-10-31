+++
id = "lib-llm"
part = "lib-sites"
kind = "migration"
needs = ["lib-kit", "counts"]
files = ["skein/crates/skein-llm/src/**", "skein/crates/skein-fake-llm-protocol/src/provider/inflate.rs"]
effort = "medium"
minutes = 30
+++

## Goal

skein-llm's affinity hex digit and OpenAI decimal parser, and the fake LLM protocol's test hex decoder move onto lib's primitives
(`skein/docs/design/draft/component-kit.md, 3.4`): each private copy
below goes, its callers calling lib, with no change of behaviour.

Acceptance:

- The copies below are gone, or kept for a reason `notes/migrations.md`
  allows (one that does more than the lib item), named in the report.
- Counts fall: `hex_decimal` by 2 or 3; none rises.
- The tests of every package you own pass, and its direct dependents
  within skein check (`heavy cargo check --tests -p ...`; `cargo tree -i
  <package> --workspace -e normal,dev --depth 1` finds them).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and
  `notes/migrations.md` (lib's primitives).
- lib's items, their docs and tests: `skein/crates/skein-lib/src/`
  (`time.rs`'s `earliest`, `lifecycle.rs`, `hex.rs`, `decimal.rs`'s
  `Decimal::parse`, `queue.rs`/`stack.rs`'s `clear`, `slab.rs`'s
  iterators).
- The copies (find each with `rg`; lines drift):
  - `crates/skein-llm/src/affinity.rs`: `fn hex(nibble) -> u8`;
  - `crates/skein-llm/src/openai/common.rs`: `pub(crate) fn decimal(&[u8]) -> Option<u64>`;
  - `crates/skein-fake-llm-protocol/src/provider/inflate.rs`: a test helper `fn hex(text: &str) -> Vec<u8>` (hex text to bytes: its digit values are `hex::value`).

## Owns and may change

- Owns `skein/crates/skein-llm/src/` and, in skein-fake-llm-protocol, `src/provider/inflate.rs` only.

## Steps

1. Move each copy, the narrowest test of its package after each: read the
   copy's body against the lib item's doc first; where they differ on an
   edge (case, empty input, overflow), keep the site's edge (common.md,
   "Behaviour stays").
2. `cargo fmt`; `heavy cargo clippy -p skein-llm -p skein-fake-llm-protocol --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-llm -p skein-fake-llm-protocol`; the dependents' checks.
3. `old_forms.py --write`; commit and push.

## Report

As common.md says, with each copy: removed, or kept and why.
