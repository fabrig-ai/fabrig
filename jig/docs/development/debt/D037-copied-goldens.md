# D037: jig keeps copies of smith's goldens that nothing compares

- **Kind:** tests
- **Where:** `jig-protocol-transcript/tests/data`, `testing/jig-test-worker/src/tests/fixtures.rs`, `testing/jig-client-test-records/src/activity_reads.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Fixtures that move with the code": four copies of smith's goldens)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A7: transcript goldens unchecked against smith's)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Test kits and worlds": copied fixture constants that nothing checks against their owners)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (a change to smith's goldens reaches jig's tests by itself)
- **Size:** S
- **Status:** open
- **Paid when:** jig's tests read smith's goldens from `smith/` (or a test compares each copy with its source), and no README names a smith revision to copy from.

## What

- `record_turn_full.bin` and `record_turn_smallest.bin` in `jig-protocol-transcript/tests/data` are byte-equal at base to `smith/crates/smith-transcript/golden/v6/`, and their README names a smith revision to copy from;
- `SMALLEST: [u8; 121]` (`testing/jig-test-worker/src/tests/fixtures.rs`, line 70) is smith's smallest v4 charter as a literal, the size of `smith/crates/smith-charter/golden/v4/record_charter_smallest.bin`;
- `FULL_TURN` (`activity_reads.rs`, line 4) includes jig's copy.

smith is in the same repository now, so the copies are no longer needed to build.

## Why it matters

Every change to smith's goldens must be copied by hand; a missed copy keeps jig's tests
green on an old format.

## What pays it back

`include_bytes!` from `smith/`'s golden paths in the test kits, the copies and their
README deleted.
