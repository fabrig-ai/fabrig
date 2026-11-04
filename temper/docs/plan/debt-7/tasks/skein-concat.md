+++
id = "skein-concat"
part = "smith-items"
kind = "interface"
needs = []
files = ["skein/crates/skein-lib/src/bytes.rs", "skein/crates/skein-lib/src/tests/bytes.rs", "skein/docs/design/lib.status.md", "skein/docs/development/debt/D007-joined-and-measure.md"]
effort = "medium"
minutes = 15
+++

# skein: `bytes::concat`, the one measured concatenation

## Goal

smith's `D016-retyped-primitives.md` cannot pay its byte `join` until skein has a concatenation, and
a missing generic mechanism goes into skein before its smith consumer. Add `skein_lib::bytes::concat`
and nothing else: skein's own `joined` copies and `Measure` stay, as `D007-joined-and-measure.md`, which
this task does not pay.

Acceptance:

- `skein_lib::bytes::concat(&[b"ab", b"", b"c"])` is a box of `abc`, of exactly that length.
- `lib.status.md` no longer lists `bytes::concat`.
- `D007-joined-and-measure.md`'s **What pays it back** says the copies go onto `bytes::concat`, and
  no longer cites smith's D016, so that `smith-primitives` can delete that item.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/lib.md`, 6 (the sentence on `bytes::concat` is on the trunk).
- `skein/docs/development/debt/D007-joined-and-measure.md`.

## Owns and may change

- Owns: `skein/crates/skein-lib/src/bytes.rs`, its test file `skein/crates/skein-lib/src/tests/bytes.rs`.
- `skein/docs/design/lib.status.md`: remove `6: `bytes::concat`.` from **Not yet**, and update its
  date line. This departs from the implementer's text.
- `skein/docs/development/debt/D007-joined-and-measure.md`: the **What pays it back** section only, as
  Step 3 words it. The item stays open and its `Measure` does not change.
- Not this task's: any `fn joined` or `struct Measure`, and any smith file.

## Steps

1. In `bytes.rs`, beside `copy_of`:

   ```rust
   /// The parts, in order, in a box of exactly their summed length.
   #[must_use]
   pub fn concat(parts: &[&[u8]]) -> Box<[u8]> {
       let mut length = 0_usize;
       for part in parts {
           length = length.checked_add(part.len()).expect("parts held in memory fit a length");
       }
       let mut writer = crate::Writer::new(length);
       for part in parts {
           writer.put(part).expect("the writer is the parts' length");
       }
       writer.finish()
   }
   ```

   Take the writer's path as the crate exports it. `Writer` is declared in `writer.rs`.
2. In `tests/bytes.rs`, one test: parts in order, an empty part, no parts at all (an empty box).
3. In `D007-joined-and-measure.md`, replace the section's text with: "`skein_lib::bytes::concat` is the
   one concatenation: delete the copies onto it, each overflow answer with them. Then a `Measure` in
   lib (or one in skein-http for the three writers), and delete those."
4. `cargo fmt`; `heavy cargo clippy -p skein-lib --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-lib --lib`; the citations check. Commit and push.

## Report

The common note's items. Break: swap the second loop's order (or drop a part) and name the failure.
