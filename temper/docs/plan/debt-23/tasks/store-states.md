+++
id = "store-states"
part = "builder-lifecycles"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-domain-store/src/lib.rs", "hearth/crates/hearth-domain-store/src/start.rs", "hearth/crates/hearth-domain-store/src/enter.rs", "hearth/crates/hearth-domain-store/src/resolutions.rs", "hearth/crates/hearth-domain-store/src/staging.rs", "hearth/crates/hearth-domain-store/src/scan.rs", "hearth/crates/hearth-domain-store/src/tests/**"]
effort = "high"
minutes = 90
+++

# hearth-domain-store: `Store`'s phase, standing and flight are enums

## Goal

Hearth D006's last. `Store` has `phase`, and beside it `usable`, `said`, `finding`, `entering`, `writing`, `working`, `staged` and `decline`. About +10 net lines.

The sketch binds; no public signature moves:

```rust
enum Phase { Closed, Locking { said: bool }, Recording, Writing, Scanning, Ready }
enum Standing { Usable, Declining(Decline) }           // replaces usable + decline
enum Flight { Idle, Finding, Writing(Digest), Entering(Entering), Chain(Working) }
```

`staged: Option<Staged>` stays: it lives across a seal's chain, so it isn't part of `Flight`, and a one-slot `Option` is its whole state. `Store` keeps `#[derive(Hash)]`, as `Builder` derives it.

Acceptance:

- **A pin first, on today's code:** `enter.rs` sets `usable = false` without setting `decline`, so later finds answer `Unusable { List }`, not `Keep`. No test holds this. One row: after a failed entry, a find answers `Unusable { List }`. It passes first. Break it by answering `Keep`, see it fail, restore. This row stays.
- `rg '(usable|said|finding): bool|(entering|writing|working): Option' hearth/crates/hearth-domain-store/src/lib.rs` reads nothing.
- `heavy cargo nextest run -p hearth-domain-store --lib`, `-p hearth-domain-builder --lib` and `-p hearth-builder-world` pass, with no assertion changed but the new row's.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. The item's **Where** names only `lib.rs`, but the fields are read in the other five. `store.store.usable` in the fixtures changes how it reaches the code.
- **`MAX_OUT` stays as it is.**

## Traps

- **`Standing`:** a tidy `Declining(Unusable { Keep })` from `enter.rs` changes visible answers. If already declining, the old decline stays. Before `Ready`, the same default answers.
- **Dispatch order:** `Room`, `Record` and `Find` go before `finding` and `entering`, but after `working` and `writing`. Keep that order.
- **`store.finding = false`** also runs on a find answered at once. Only the `Finding` arms may write `Idle`, or a synchronous answer drops an in-flight arrival.
- **`Stage` during another flight** isn't refused today; only the protocol's "one store request" assert stops it. So a new `unreachable!` there keeps the crash, and that's fine.
- **`Failed`** must match `Locking { .. }`.

## Steps

1. The pin.
2. `Standing`; `Phase`; `Flight`.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-domain-store --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; what you broke for the pin and what failed.
