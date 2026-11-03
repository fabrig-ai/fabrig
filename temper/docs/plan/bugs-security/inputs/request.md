# Request: bugs that leak a secret, fixed with tests that fail on the old code

From fabrig/debt's backfill via the director, 2026-11-03 ~21:00Z (the user approved the
routing; source `coord/debt/for-director.md` on hel8, its "Bugs" section). Each fix carries a
test, in the lowest world that shows it, that fails before the fix and passes after. No other
change; a fix that needs a design change stops and asks.

1. **jig, `jig/testing/jig-test-service/src/program_sources.rs`:** `Resources` derives `Debug`, and
   its environment entries may hold credentials, so a debug print or trace of it can show them.
   Replace the derive with a hand-written `Debug` that shows only counts (as temper's copy does).
   Test: a `Resources` holding a canary value formats without it.
2. **skein, skein-io's `file::Request`:** it derives `Debug`, so a trace of `Store { bytes }` prints
   a stored record (a token record among them) in full. Give the request a `Debug` that shows the
   byte count, never the bytes (and the same for any other variant carrying file contents). Test:
   formatting a `Store` with a canary shows its length, not the canary.
3. **skein, skein-lib's `CryptoRng` derives `Clone`,** so a clone repeats the random stream, against
   `skein/docs/design/lib.md` 8. Remove `Clone` (and any by-value copy path), fixing its users to
   pass the generator by reference or draw a fresh one. Test: a compile-fail or trait check that
   `CryptoRng: !Clone`, plus the users' existing tests.
4. Before fixing, search each tree for any other `derive(Debug)` on a type that holds a token,
   key, password or credential bytes, and fix those the same way, listing each in the report;
   leave anything that needs a design decision for a question.

Status files: each tree's status file for the crate says what changed, if it recorded the gap.
