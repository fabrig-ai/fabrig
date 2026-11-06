+++
id = "utf8-one"
part = "utf8-one"
kind = "feature"
needs = []
files = ["skein/crates/skein-codec/src/lib.rs", "skein/crates/skein-oauth-issuer/src/documents.rs", "skein/crates/skein-forge/src/github/request.rs", "skein/docs/development/debt/D004-utf8-checks.md"]
effort = "medium"
minutes = 20
+++

# One whole-buffer UTF-8 check: `skein_lib::bytes::is_utf8`

## Goal

Pay skein's D004. `skein_codec::text_is_valid` stops being a second, hand-written RFC 3629 check and delegates to `skein_lib::bytes::is_utf8`; the step code that calls `from_utf8` under an expect only for the answer calls `is_utf8` instead.

Acceptance:

- `text_is_valid`'s body is the one call. Its name stays: generated code in every tree calls it, through `skein/testing/skein-codegen/src/emit_types.rs`, and generated code doesn't change.
- `skein-oauth-issuer/src/documents.rs`'s claim check and `skein-forge/src/github/request.rs` call `is_utf8`. `valid_text` is gone, and so is each `disallowed_methods` expect that only those lines needed.
- The item's measure reads 25: `rg -c 'from_utf8\(' skein/crates skein/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!*_tests.rs' | awk -F: '{s+=$2} END{print s}'`. The remainder is `is_utf8`'s own body and sites that use the `str`.
- `skein/docs/development/debt/D004-utf8-checks.md` is deleted.

## Inputs

- `skein/docs/development/debt/D004-utf8-checks.md`, whole.
- `skein/docs/design/lib.md`, 6.
- `skein/docs/design/codec.md`, 4, for text that isn't UTF-8.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. `skein-codec` already takes `skein-lib`. A manifest's dependency line for `skein-oauth-issuer` or `skein-forge`, if missing, is yours; both should have it.

The cut read the 27 sites. The other 25 use the `str`: `parse`, a JS string, or a `to_owned`. `skein-json`'s streaming `Utf8` is a different job and stays. If you find one more site that only tests validity, move it too, and report the reading.

## Steps

**Check the loop agrees before it goes.** A throwaway test, untracked and deleted before the push: run the hand loop and `is_utf8` over every input of 0 to 3 bytes, and over 4-byte inputs of a lead byte from `0xe0..=0xf5` followed by the edge values `0x7f, 0x80, 0x8f, 0x90, 0x9f, 0xa0, 0xbf, 0xc0`. They must agree on every one; a disagreement stops and asks, since the codec would then change meaning. Report the count.

Then the three edits. Narrow tests: `heavy cargo nextest run -p skein-codec`, the codec world (the package in `skein/tests/codec/Cargo.toml`), `-p skein-oauth-issuer`, `-p skein-forge`.

`cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`, `--write` if a count falls; `heavy cargo clippy -p skein-codec -p skein-oauth-issuer -p skein-forge --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says: the differential's count, the measure's reading, and the net lines.
