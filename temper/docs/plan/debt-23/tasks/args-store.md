+++
id = "args-store"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-store/src/generations.rs", "jig/crates/jig-protocol-store/src/directory.rs", "jig/crates/jig-protocol-store/src/patching/switch.rs", "jig/crates/jig-protocol-store/src/patching/phase/walking.rs", "jig/crates/jig-protocol-store/src/tests/generations.rs", "jig/crates/jig-protocol-store/src/tests/format.rs"]
effort = "medium"
minutes = 20
+++

# jig-protocol-store: `manifest` and `seal` take their arguments folded

## Goal

Part of jig D016: `generations.rs`'s `manifest` takes (store, secrets) folded, and `patching/phase/walking.rs`'s `seal` a `Sealing`, by `notes/arguments.md`. `manifest` is `pub(crate)`, so it's no seam, despite its reason.

Acceptance:

- `rg -o 'too_many_arguments' jig/crates/jig-protocol-store/src -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` reads 0.
- `heavy cargo nextest run -p jig-protocol-store --lib`, `-p jig-store-world` and `-p jig-store-codecs-world` pass, with no assertion changed. **The stored bytes don't move,** and the golden files prove it: `manifest`'s output stays byte for byte.
- About +8 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header. Tests only in `manifest`'s calls.
- **Stays off:** the store's schemas and `generated/`, which other drafts are changing.

## Steps

1. The two folds.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-protocol-store --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items.
