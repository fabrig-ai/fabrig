+++
id = "digits"
part = "digits"
kind = "feature"
needs = ["paths"]
files = ["hearth/crates/hearth-client-protocol-records/src/form/text.rs", "hearth/crates/hearth-client-protocol-records/src/form/json.rs", "hearth/crates/hearth-client-protocol-records/src/parts/config.rs", "hearth/crates/hearth-client-protocol-records/src/parts/config/directives.rs", "hearth/crates/hearth-client-protocol-records/src/parts/secrets.rs", "hearth/crates/hearth-client-protocol-records/src/parts/software.rs", "hearth/crates/hearth-protocol-builder/src/cache.rs", "hearth/crates/hearth-protocol-builder/src/resolve.rs", "hearth/crates/hearth-protocol-builder/src/store/names.rs", "hearth/crates/hearth-protocol-mmdebstrap/src/packages.rs", "hearth/crates/hearth-protocol-git/src/listing.rs", "hearth/crates/hearth-domain-render/src/kinds/software.rs", "hearth/crates/hearth-domain-render/src/kinds/files.rs", "hearth/crates/hearth-domain-render/src/kinds/secrets.rs", "hearth/crates/hearth-domain-render/src/schema/forms.rs", "hearth/crates/hearth-domain-render/src/tests/fixtures/digests.rs", "hearth/crates/hearth-domain-render/Cargo.toml", "hearth/crates/hearth-lib/src/tests/fixtures.rs", "hearth/crates/hearth-lib/Cargo.toml", "hearth/crates/hearth-protocol-tree/src/tests/fixtures.rs", "hearth/crates/hearth-protocol-tree/Cargo.toml", "hearth/testing/hearth-test-fixtures/**", "hearth/testing/hearth-fake-machine/src/hearth/universe.rs", "hearth/scripts/old-forms.txt", "Cargo.lock", "hearth/docs/development/debt/D005-private-hex-decimal-octal.md"]
effort = "high"
minutes = 90
+++

# hearth's digits are lib's

## Goal

skein-lib has `bytes::hex_digit`, `bytes::hex_value` and `bytes::decimal`; no hearth file calls them. Step code calls
them, a mode's digits are written and read in one place each, and the fixtures' `hex` is one helper in the kit. Not a
byte of a record, a store name or a refusal changes.

Acceptance:

- `git grep -E '\bfn\s+(hex|decimal)\b' -- 'hearth/*.rs'` finds 8 lines: `Lines::decimal`, `Oid::hex`, fake git's `hex`,
  the kit's `hex`, and the four references in `tests/config/src/reference.rs`, `tests/git/src/expect.rs`,
  `tests/git/src/reference.rs` and `tests/tree/tests/suite/hash.rs`. Those references stay independent on purpose.
  `hex_decimal_fn` in `hearth/scripts/old-forms.txt` reads 8, by `--write`.
- `git grep -nE "(saturating|checked)_add\(10\)" -- 'hearth/crates/*/src/*.rs' ':!*/tests/*'` finds nothing but the
  line that isn't digits, if any: name it in the report.
- `git grep -n '& 7' -- 'hearth/crates/hearth-client-protocol-records/src'` finds one line, and
  `git grep -n 'checked_mul(8)' -- 'hearth/crates/hearth-domain-render/src/kinds' 'hearth/crates/hearth-domain-render/src/schema'`
  finds one. `hearth-domain-render/src/tests/fixtures/software.rs` keeps its own writer: a fixture's expected text
  stays independent of what it checks.
- `heavy cargo nextest run` passes for `-p hearth-client-protocol-records -p hearth-protocol-builder -p
  hearth-protocol-mmdebstrap -p hearth-protocol-git -p hearth-domain-render -p hearth-lib -p hearth-protocol-tree -p
  hearth-fake-machine`, and for the worlds that read them: `-p hearth-builder-world -p hearth-render-world -p
  hearth-git-world -p hearth-mmdebstrap-world -p hearth-tree-world -p hearth`. No golden moves. A meaning changes
  nowhere, but these are the bytes of every record: so the whole list, not a filter.
- The branch's net lines are negative, production and tests each. The kit's `digits.rs` is at most 15 lines.
- `hearth/docs/development/debt/D005-private-hex-decimal-octal.md` is gone.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `hearth/docs/development/debt/D005-private-hex-decimal-octal.md`
- `skein/crates/skein-lib/src/bytes.rs`: `hex_digit`, `hex_value`, `decimal`, and what each accepts

## Owns and may change

- **Owns:** the files in the spec's `files` list, for the changes below.
- **May change:** `hearth/crates/hearth-domain-render/Cargo.toml`, `hearth/crates/hearth-lib/Cargo.toml`,
  `hearth/crates/hearth-protocol-tree/Cargo.toml`: a `[dev-dependencies]` section with
  `hearth-test-fixtures.workspace = true`, above `[lints]`. `Cargo.lock`: what cargo writes.
- **Not to touch:** `hearth-domain-build/src/tests.rs`, `tests/builder/src/domain/store.rs` and fake git's `objects.rs`,
  which read digits for themselves in tests or fakes and are outside the item. Any `MAX_OUT` form.
- **Options:**
  - `kinds/software.rs` has a local `bytes`: call `skein_lib::bytes::hex_value` by its full path, or rename the local.
  - If the count can't reach 8 because the trunk added a `fn hex`, take the count as you find it and name the extra.
  - If a function you'd share has a second reader this spec doesn't name, its call site is yours to change.

## Steps

1. **`hearth-client-protocol-records`.** In `form/text.rs`, `fn hex` goes: `quote` and `hex12` call
   `skein_lib::bytes::hex_digit`, which masks to the low four bits. In `form/json.rs`, `digest_hex` calls it too.

   `parts/config.rs`'s `octal` moves to `form/text.rs`, unchanged in signature and body, as the one writer of a mode's
   digits. `Lines::mode` becomes `self.put(&octal(mode))`. `parts/secrets.rs`'s `json_place` calls `octal` in place of
   its copy. `parts/config.rs`, `parts/config/directives.rs` and `parts/software.rs` take the new path.
2. **`hearth-protocol-builder`.** `cache.rs`'s `hex` becomes:

   ```rust
   /// A byte's two lower-case digits.
   pub(super) fn hex_pair(writer: &mut Writer, byte: u8) {
       writer.put(&[bytes::hex_digit(byte >> 4_u8), bytes::hex_digit(byte)]).expect("hex room");
   }
   ```

   `resolve.rs` and `Cache::new` take the name, and `store/names.rs`'s `removal` calls it in place of its loop. Check
   `Writer::put` of two bytes behaves as two puts of one for a writer with room: each `Writer::new` size here counts
   both digits.

   `names.rs`'s `digit` keeps refusing an upper-case digit, which lib's `hex_value` reads:

   ```rust
   /// A digit of a name the store wrote: lower-case, so another name is not read as the store's.
   fn digit(byte: u8) -> Option<u8> {
       if byte.is_ascii_uppercase() { None } else { bytes::hex_value(byte) }
   }
   ```
3. **`hearth-protocol-mmdebstrap`.** `packages.rs`: `decimal` goes, its caller calls `bytes::decimal`. `digest`'s match
   becomes `bytes::hex_value(*b)?`: both accept either case.
4. **`hearth-protocol-git`.** `listing.rs`'s `number` keeps its refusal of a leading zero, and calls lib for the rest:

   ```rust
   pub(crate) fn number(input: &[u8]) -> Option<u64> {
       if input.len() > 1 && input.starts_with(b"0") {
           return None;
       }
       bytes::decimal(input)
   }
   ```

   An empty input is `None` from lib.
5. **`hearth-domain-render`.** `kinds/software.rs`: `checked(sha, Format::Sha256, ..)` has refused all but lower-case hex
   by then, so `hex_value(*digit).expect("a checked digit")` replaces the arithmetic.

   The mode's reader goes in `schema/forms.rs`, beside `Format::Mode`'s check:

   ```rust
   /// The value of a mode's octal digits, `check` having passed them.
   pub(crate) fn octal(digits: &[u8]) -> u16;
   ```

   `kinds/files.rs`'s `mode` and `kinds/secrets.rs`'s `mode_with_default` call it. Its body is their loop.
6. **`hearth-fake-machine`.** `universe.rs`: `decimal` goes, its callers call `skein_lib::bytes::decimal`, which it
   matches on an empty value, a non-digit and an overflow. `hex` becomes `sha256`, with `hex_value` for `to_digit(16)`:
   both accept either case.
7. **The kit.** `hearth/testing/hearth-test-fixtures`: `skein-lib.workspace = true` in `[dependencies]`, `pub mod
   digits;` in `lib.rs`, and `src/digits.rs`:

   ```rust
   //! Digits a test writes as text.
   use skein_lib::bytes;

   /// The bytes `text` spells in pairs of hex digits, of either case; spaces don't count.
   #[must_use]
   pub fn hex(text: &str) -> Box<[u8]>;
   ```

   Ordinary Rust: an iterator over the digits, a `chunks_exact(2)`. It keeps the fixture's two panics, on a byte that
   isn't a digit and on an odd count (`"hex pairs are whole"`).

   In `hearth-lib/src/tests/fixtures.rs`, `hearth-protocol-tree/src/tests/fixtures.rs` and
   `hearth-domain-render/src/tests/fixtures/digests.rs`, the copy goes, for `pub use hearth_test_fixtures::digits::hex;`.
   Callers don't change.
8. **Bite.** Two guards here are what's left of a rule lib doesn't hold:

   - Remove `is_ascii_uppercase` from `names.rs`'s `digit`: `tests/store.rs`'s row
     `b"store.record.skein-0123456789abcdeF"`, `Shape::Stray`, must go red.
   - Remove the leading-zero check from `listing.rs`'s `number`: a test of `listing` must go red. If none does, add the
     row to the listing's tests that holds it, and show that row red.

   Restore both.
9. `cd hearth && python3 -I -B scripts/old_forms.py --write`. Delete D005's file. The tests above, `cargo fmt`, clippy on
   every package changed. Commit and push.

## Report

The 8 lines left. What each bite printed. Anything you found that reads digits by hand and isn't listed. Net lines,
production and tests apart.
