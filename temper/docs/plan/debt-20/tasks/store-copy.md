+++
id = "store-copy"
part = "store-rules"
kind = "interface"
needs = []
files = ["jig/crates/jig-protocol-store/src/boundary.rs", "jig/crates/jig-protocol-store/src/commits.rs", "jig/crates/jig-protocol-store/src/tests/commits.rs", "jig/crates/jig-domain-core/src/settings/revisions.rs", "jig/crates/jig-domain-core/src/settings/tests.rs", "jig/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 40
+++

# jig: the store's copy write is generic

## Goal

The first half of `jig/docs/development/debt/D025-store-codec-domain-rules.md`. `jig_protocol_store::Write::CopySettingsRevision(u32)` becomes `Write::Copy { from, to }` over the core's keys (`temper/docs/plan/debt-20/notes/contracts.md`, 1): the core says which keys, and the store copies bytes. What's committed stays byte for byte.

Acceptance:

- `rg -n 'CopySettingsRevision' jig/crates/jig-protocol-store` and `rg -n 'store::Write::CopySettingsRevision' jig temper` find nothing. `core::Write::CopySettingsRevision` and the application roots' own stay (the contract's words).
- `commits.rs`'s `copy_settings` names no settings type; a missing `from` answers as a missing held revision does today.
- `settings::history_copy` has a test in `jig-domain-core/src/settings/tests.rs`: the number reaches both keys, the first `Held`, the second `History`.
- By hand: have `Write::Copy` put under `from` instead of `to`; `tests/commits.rs`'s copy test goes red. Swap the keys in `history_copy`; the new test goes red. Restore each.
- Production lines don't rise by more than the two routes' extra lines (estimate: +5). The item stays: `store-remaps` deletes it.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/contracts.md`, 1: binding
- `jig/docs/development/debt/D025-store-codec-domain-rules.md`
- `jig/docs/design/protocol/store.md`, 3 and 6 (this plan's text on the copy)

## Owns and may change

- **Owns:** the files in `files`. `store.status.md`: only the `3, 6:` clause this task closes, nothing else on the line.
- **Reader files, call sites only:**
  - `temper/crates/temper-service/src/store/route.rs`, `jig/testing/jig-test-store/src/route.rs`: the one arm that writes `store::Write::CopySettingsRevision`.
  - `jig/tests/store/src/model.rs`: its arm matches `Write::Copy { from, to }`, keeps its typed settings rows and reads the same state. If `root.rs` or another of its files asks, the same.
- **Not to change:** `jig/testing/jig-test-store-files/src/store.rs` and the other readers of `root::Write::CopySettingsRevision` (the root's write, not the store's). `temper/tests/**`.
- **If `copy`'s body wants a helper beside the `Put` arm's encoding,** that's yours.
- **`tests/commits.rs`'s test** changes only its spelling of the write, to `history_copy`'s keys. What it asserts stays.

## Steps

1. `settings::history_copy` with its test.
2. `Write::Copy`; `commits.rs`'s two arms and `copy`; its doc comment; the routes; the store model.
3. Break the rules by hand, as the acceptance says.
4. Remove this task's clause from `store.status.md`. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-store -p jig-domain-core -p jig-test-store -p jig-store-world -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-store -p jig-store-world -p jig-test-store`; `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`; `heavy cargo check -p temper-service --tests`. The citations check. Commit and push.

## Report

The common note's items; what you broke and what failed.
