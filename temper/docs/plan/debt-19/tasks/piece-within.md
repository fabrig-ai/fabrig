+++
id = "piece-within"
part = "link-translate"
kind = "interface"
needs = []
files = ["skein/crates/skein-codec/src/piece.rs", "skein/crates/skein-codec/src/tests/piece.rs", "skein/docs/design/codec.status.md"]
effort = "medium"
minutes = 20
+++

# skein: a codec splits a piece's items under a running sum

## Goal

The kit's half of `jig/docs/development/debt/D008-link-translate-split-by-number.md`. Everywhere a document's pieces are read, the same lines split an item, charge it and its length to a sum, and refuse past a bound. Those lines go into `skein_codec::piece`, before their consumers. Non-generic: closures, generics and traits are out of step code, and a decoded row has no shared trait.

Acceptance:

- `skein_codec::piece` gains the three functions below, each with step tests: `split_within` charges an item's bytes and its four-byte length, at `most` exactly passes and one past it is `Reason::Bound`, `u64` overflow is `Reason::Bound`, a partial length or an item past the end is `Reason::Malformed` and an empty reader is `Ok(None)` and charges nothing. `count` agrees with `write`'s items and refuses a malformed piece. `counted` returns each item of a counted piece.
- By hand: drop the `+ 4` (a test goes red); make `Bound` fire at `most` (a test goes red). Restore each.
- `heavy cargo nextest run -p skein-codec` passes. Production rises by at most 30 lines (estimate +24), which D008's later tasks pay many times over. `skein/docs/design/codec.status.md`'s **Not yet** entry goes, and it reads `No gap.` again if nothing else has joined it.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D008-link-translate-split-by-number.md`
- `skein/docs/design/codec.md`, 3: written, and binding
- `skein/docs/foundation/programming-model.md`, 10.3

## Owns and may change

- **Owns:** the files in `files`.
- **The shape, in `piece.rs`, beside `split`:**

  ```rust
  /// Splits the next item and charges its bytes and its length to `sum`;
  /// `Reason::Bound` past `most` or on overflow.
  pub fn split_within(reader: &mut Reader<'_>, sum: &mut u64, most: u64) -> Result<Option<Box<[u8]>>, Reason>;

  /// The items a piece holds; `Reason::Malformed` on a partial length or an item past the end.
  pub fn count(bytes: &[u8]) -> Result<u32, Reason>;

  /// The next item of a piece `count` has counted.
  pub fn counted(reader: &mut Reader<'_>) -> Box<[u8]>;
  ```

  `counted` panics, by `expect`, on a piece `count` didn't accept: the invariant relied on. `split` doesn't change. Order inside `split_within`: split first, then charge, so a malformed item is `Malformed` before any bound speaks, as every site has it.
- **`skein/crates/skein-codec/src/tests/piece.rs`** exists; add to it.

## Steps

1. The functions, their tests. Break the rules by hand. Delete the status entry.
2. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-codec --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-codec`. The citations check. Commit and push.

## Report

The common note's items, what you broke and what failed, and the net lines.
