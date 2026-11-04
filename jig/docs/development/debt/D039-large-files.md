# D039: Eight production files pass 1,500 lines

- **Kind:** complexity
- **Where:** `jig-client-view/src/words/phrases/jig.rs`, `jig-protocol-config/src/members.rs`, `jig-protocol-store/src/codec/settings.rs`, `jig-domain-tasks/src/boundary.rs`, `jig-domain-core/src/reads/rows.rs`, `jig-client-view/src/words/phrases/english.rs`, `jig-domain-tasks/src/domain.rs`, `jig-domain-core/src/effects.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Signals in the step code": long functions, long tests, large files)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (cross-check: files of 800 lines or more, 66 to 134; A2 F8, A3 F2, A6 F11, F18)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** medium (files a reader can hold; fewer merge conflicts)
- **Size:** L
- **Status:** open
- **Paid when:** no hand-written production file in jig passes 1,500 lines (fabrig's one threshold for "too large to read"), except a table a comment says is kept whole on purpose.
- **Measure:** `find jig -name '*.rs' -not -path '*/generated/*' -not -path '*/tests/*' -not -path 'jig/testing/*' -not -name 'tests.rs' -not -name 'tests_*.rs' -not -name '*_tests.rs' -exec wc -l {} + | awk '$2 != "total" && $1 > 1500' | wc -l`: 8

## What

Eight hand-written production files are over 1,500 lines: `jig-client-view/src/words/phrases/jig.rs`
3,075 (D021); `jig-protocol-config/src/members.rs` 2,659, new since the backfill's first
count; `jig-protocol-store/src/codec/settings.rs` 2,254 (D025); `jig-domain-tasks/src/boundary.rs`
2,252 (D012); `jig-domain-core/src/reads/rows.rs` 1,837; `jig-client-view/src/words/phrases/english.rs`
1,630 (D021); `jig-domain-tasks/src/domain.rs` 1,588; `jig-domain-core/src/effects.rs` 1,507.

Test files are not counted, but the largest are of the same kind: `jig-domain-core/src/tests.rs`
3,899 and `jig-domain-people/src/tests.rs` 2,511 hold a whole crate's unit tests in one file;
`jig-protocol-store/src/tests/phase.rs` 1,898 holds four disk harnesses.

## Why it matters

Size alone is not debt, but these files mix concerns (a boundary of 77 events with
near-duplicate families, a crate's unit tests in one file), and they are where parallel
branches collide.

## What pays it back

Split by concept as the items above pay off (D012, D021, D025), and the others by the
design's parts; split the unit-test files by the module they test.
