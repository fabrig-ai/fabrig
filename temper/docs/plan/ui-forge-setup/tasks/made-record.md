+++
id = "made-record"
part = "app-made"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-protocol-records/schema/secrets-v1.schema",
  "temper/crates/temper-protocol-records/src/generated/v1.rs",
  "temper/crates/temper-protocol-records/golden/**",
  "temper/crates/temper-protocol-records/src/lib.rs",
  "temper/crates/temper-protocol-records/src/codec.rs",
  "temper/crates/temper-protocol-records/src/forge.rs",
  "temper/crates/temper-service/src/forge/gone.rs",
  "temper/crates/temper-service/src/tests/forge_gone.rs",
  "temper/tests/codecs/tests/suite/records.rs",
  "temper/docs/design/protocol/credentials.status.md",
]
effort = "medium"
minutes = 40
+++

# temper can keep `AppMade` in the secrets' store

## Goal

A manifest-made App will be marked by a new secret item, `AppMade`, written first by skein's `Convert` and kept with `AppId`. Give temper its record first, so that nothing is ever written that temper can't keep: code 12, its schema row, its codec, its check and its erasure. Nothing writes it yet: `setup-answers` does.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 5), because the design already says what this task builds.

Acceptance:

1. `Item::AppMade`'s code is 12, `parse` answers it, and code 11 stays refused: the `for code in [4, 11]` loop in `temper/tests/codecs/tests/suite/records.rs` passes unchanged.
2. `records::encode` and `decode` round-trip `AppMade`'s bytes, and a row of 8 zero bytes makes `access` answer `Unreadable`, a new row of that test's table of unreadable items.
3. `access` connects a GitHub forge with a good `AppMade` row, and without one, as `complete` holds none.
4. A removal erases `AppMade` after `Origin`, in the expected list of `temper/crates/temper-service/src/tests/forge_gone.rs`.
5. The entry *2.1: `AppMade` has no record...* is gone from `temper/docs/design/protocol/credentials.status.md`.
6. Focused tests pass in `temper-protocol-records`, `temper-codecs-world` and `temper-service`, and `heavy cargo check --tests` passes for `temper-client-domain`, `temper-github-app` and `temper-real-loop-world`. `temper-real-loop-world` is compiled, never run.

## Inputs

- `temper/docs/plan/ui-forge-setup/notes/contracts.md`, *The record*: it binds.
- `temper/docs/plan/ui-forge-setup/notes/common.md`.
- `temper/docs/design/protocol/credentials.md`, 2.1: the items' table, the values and *Erased with the forge*.

## Owns and may change

**Owned:** the `files` above.

**Readers:** any other exhaustive `match` on `temper_protocol_records::Item` or `v1::Item` the compiler names gets its arm; the survey found `lib.rs`, `codec.rs` (`generated`, `raw`, `pasted_value`, `pasted_item`), `forge.rs` and `gone.rs`, and none beyond them. `temper-service/src/setup.rs`'s `item_record` maps skein's items and isn't touched here.

**Options, each with its default:**

- `ForgeRows::values` is `[Option<Secret>; 10]`, indexed by `code - 1`. Default: 12 slots, with the one for code 11 never filled, as `parse` refuses it. If a private slot function reads better, use it. Update the `expect` words that say "ten".
- `access`'s check walks `values` beside a `kit` list of skein's items. skein has no `Item::AppMade` until `setup-answers`, so check this row as `item::Item::AppId`, whose bytes it shares; `setup-answers` changes that one word.
- A pasted bundle never carries `AppMade`, but `PastedItem` mirrors `Item` (the contract), so `pasted_value` and `pasted_item` each gain their arm. A golden of the pasted record that regenerates differently is expected: regenerate, and say so.
- A temper test that counts a removal's erases by number, not by the list: change the number, and say so.

## Steps

1. The schema's lines, as the contract gives them, then regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`. Check with `git status` that the existing goldens' bytes didn't move, the pasted record's aside.
2. `lib.rs`:

   ```rust
   pub enum Item { /* ... */ Origin, AppMade = 12 }
   // code(): Self::AppMade => 12
   // parse(): 12 => Item::AppMade
   ```

3. `codec.rs` as `AppId`'s arms are, `v1::AppMade` for `v1::AppId`. `forge.rs` per the options.
4. `gone.rs`, `next`:

   ```rust
   Item::Accepted => Item::Origin,
   Item::Origin => Item::AppMade,
   Item::AppMade => {
       *item = None;
       answered(engine, forge);
       return false;
   }
   ```

5. The tests of acceptance 1 to 4. Round-trip wherever `records.rs` or the crate's own tests round-trip an item's bytes; the table of acceptance 2 is the `for (item, bytes) in [(Item::AppId, vec![0; 8]), ...]` loop.
6. Narrow tests, then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-protocol-records -p temper-service -p temper-codecs-world --all-targets -- -D warnings`, acceptance 6, commit and push.

## Report

As the implementer's text says; also the code and slot form you chose, and any golden that moved.
