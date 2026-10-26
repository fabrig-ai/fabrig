# Findings: the skein step (lib.md 6.2: SHA-256, Hmac, equal)
At skein `main` ec686b8 (2026-10-16). Paths are skein's unless named.

## 1. What lib.md 6.2 asks
- **`lib::sha256`** (FIPS 180-4): `Sha256::new()`, `update(&[u8])` (cut anywhere),
  `finish(self) -> [u8; 32]`; one-shot `digest(&[u8]) -> [u8; 32]`. `Clone`;
  `Debug` shows the byte count only. State: eight words, a 64-byte block, the
  count; whole blocks compress from the caller's slice; no heap. The bound,
  2^64 - 1 bits, is asserted.
- **`Hmac`** (RFC 2104): `new(key: &[u8])`, `update`, `finish(self) -> [u8; 32]`;
  a key over 64 bytes is hashed first; no heap; `Debug` shows neither key nor
  state (what it shows is unstated; `Clone` too). "Two SHA-256 states and the
  padded key's block": two `Sha256` seeded with the ipad and opad blocks suffice.
- **`sha256::equal(a: &[u8], b: &[u8]) -> bool`:** time depends on the lengths
  alone. Silent on `core::hint::black_box` (stable, no unsafe).
- **Migrations named:** skein-io's `digest` keeps `Digest([u64; 4])`, `digest(&[u8])`
  as a thin layer; PKCE and `skein-ui-bundle` (gains a `skein-lib` dependency) use
  lib's; `skein-oauth`'s state comparison uses `equal`.
- **Tests (lib.md 10):** FIPS examples (empty, `abc`, 448- and 896-bit, a million
  `a`s); lengths 0..=200 vs the one-shot form, whole and split at every pair of
  points; fuzzy: 20,000 random messages cut at random. HMAC: RFC 4231 cases 1 to 7
  (5 truncated; 6, 7 a 131-byte key), whole and split at every point. `equal`:
  equal/unequal, same/different lengths. Step tests `src/tests/sha256.rs` (+ `mod
  sha256;` in `src/tests.rs`); fuzzy `tests/lib/tests/fuzzy_bytes.rs`. RFC 4231
  vectors are nowhere in the tree.
- **Subset (AGENTS.md; programming-model.md 10):** `forbid(unsafe_code)`, `no_std`
  (`lib.rs:11-12`), no external crates, `as`, indexing, `unwrap`, unchecked
  arithmetic, `copy_from_slice`, closures, `Default`, `Vec`; `for` loops; scoped
  `#[expect(.., reason)]` only (`crypto_rng.rs:90`); wasm32: checked conversions.

## 2. What exists
- **skein-lib has no SHA-256, HMAC or constant-time code** (graph and grep). It
  goes in `crates/skein-lib/src/sha256.rs`, `pub mod sha256;` beside `lib.rs:18`'s
  `pub mod base64;` (the model; no root re-export); crate doc `lib.rs:1-8`.
- **Three copies to replace:** `skein-io/src/digest.rs` (234 lines: `INITIAL`
  :20, `ROUND` :23-88, streaming `DigestState` :92-150, `digest` :153, `compress`
  :159-204, `#![expect]`s :6-14, tests :211-233; store users
  `file_layer/store.rs:15,74,211,853`); `skein-oauth/src/pkce.rs` (`K` :6-71,
  `sha256` :91-156); `skein-ui-bundle/src/sha256.rs` (126 lines; caller
  `digest.rs:17-33`; the golden directory `189a327712563817` must hold).
- **Constant-time compare:** `skein-oauth/src/machine.rs:668-677 constant_time_equal`
  (length check, OR of XORs; one use, :318) is `equal`'s body.
- **STATUS.md** (`docs/design/STATUS.md:34, 87-95`): temper's plan takes HMAC and
  `equal` "with lib's SHA-256 if hearth's M1 plan has not built it": it has not,
  so the step is all of 6.2. `StateHasher` (5.1) stays hearth's.

## 3. hearth's SHA-256 step: paused, unlanded, no code
- **Text:** `~/src/rust/tmp/plan/hearth-render/docs/development/plans/render/skein/01-sha256.md`
  (392 lines; untracked, worktree of branch `plan/render`, 175e92f).
- **Code: none** in any branch or worktree (hearth has no Rust; skein's 96
  branches and `tmp/plan/hearth-render-skein` have no `sha256.rs`).
- **Planned:** 01.1 `lib::sha256` (`Sha256`, `digest`; at most 205 lines, 66 of
  them round constants); 01.2 skein-io `digest` over it (<=40 lines); 01.3 PKCE
  and the bundler on it (<=30 lines). No HMAC, no `equal`.
- **Reusable:** its struct (`state: [u32; 8]`, `block: [u8; 64]`, `hashed: u64`,
  `const fn new`), `compress` moved from skein-io with its `#![expect]`s scoped
  to it, `update` via `as_chunks::<64>()` and `get_mut`/`zip`, `Debug` as
  `Sha256 { hashed: N }`, `#[cfg(test)] having_hashed(n)` for the bound test, its
  interface sketch, test list and literals (FIPS, padding edges, million `a`s
  `cdc76e5c...2cd0`), its Don'ts (no `Copy`/`PartialEq`/`Hash`/`Default`).
- **Raise with the designer:** lib.md 10's "every pair of points" over 0..=200 is
  about 1.37M hashes (est. 10 to 40 s unoptimised); hearth's plan cut it to every
  single cut point.

## 4. The gate
- `scripts/check.sh`: fmt; clippy workspace
  `-D warnings`; **`cargo check --target wasm32-unknown-unknown -p skein-lib -p
  skein-codec -p skein-http`** (the lib-specific line); `taste.sh`; `cargo check
  -p skein-tls`; nextest focused (15 s; base 6.46 s), fuzzy (60 s, `fuzzy_*`),
  browser (Chromium: x1 needs `SKEIN_TEST_CHROMIUM`, and
  `a_page_of_another_origin_is_opened_by_its_step` fails there, so the gate
  with it filtered, as hearth's plan did). Dev: `cargo nextest run -p
  skein-lib -p skein-lib-tests -p skein-io -p skein-oauth -p skein-ui-bundle`.

## 5. Downstream
- **No repo names `skein_lib::sha256`, HMAC or a constant-time compare.** skein-io's
  `Digest` is named by smith 9 files, jig 11, temper 6: API kept. **External `sha2`**
  (outside 6.2): jig tests (`Cargo.toml:155`), smith `benchmarks/src/grade.rs:10`,
  temper's domain crate `temper-engine-domain-forge-issues/src/domain.rs:8,25`
  (`temper/Cargo.toml:125`); moving them waits for a repin.
