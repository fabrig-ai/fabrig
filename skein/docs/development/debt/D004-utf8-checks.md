# D004: UTF-8 is checked by a hand loop in codec and by `from_utf8` under expects

- **Kind:** duplication
- **Where:** `skein-codec/src/lib.rs`, `skein-lib/src/bytes.rs`, step crates calling `core::str::from_utf8`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the UTF-8 row)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** skein has one whole-buffer UTF-8 check, `skein_lib::bytes::is_utf8`; `skein_codec::text_is_valid` delegates to it or is gone, and step code calls it instead of `from_utf8` under a scoped expect where it only needs the answer.
- **Measure:** `rg -c 'from_utf8\(' skein/crates skein/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!*_tests.rs' | awk -F: '{s+=$2} END{print s}'`: 27 at base (some need the `str`, so the floor is above 0)

## What

- lib has `bytes::is_utf8` and `utf8_prefix` since 10-31, used 4 times.
- `skein_codec::text_is_valid` (`skein-codec/src/lib.rs`) is a second,
  hand-written RFC 3629 check, referenced 26 times.
- Step code outside tests calls `from_utf8` 27 times, each under a
  `disallowed_methods` expect.
- skein-json's streaming `Utf8` (`skein-json/src/utf8.rs`) is a
  different job, a check across chunks, and is not part of this item.

## Why it matters

Two definitions of valid text can disagree on an edge (overlong forms,
surrogates), and each `from_utf8` site carries an exception to the
rules for what one call would say.

## What pays it back

Point `text_is_valid` at `is_utf8` (keeping its name if generated code
calls it), then move the `from_utf8` sites that only test validity.
