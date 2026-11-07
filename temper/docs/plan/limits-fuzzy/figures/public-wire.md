# public-wire: the public records' decoder sweep

2026-11-07 UTC, base commit `0a921f03b1`, 16 CPUs, AMD EPYC-Genoa Processor,
Valgrind 3.24.0. The after arm is that base with this change to
`temper/tests/codecs/tests/suite/fuzzy/public_wire.rs` (source blob `5cbd3d3d927d2b32a520ba2fc1707ca313814821`).
Both binaries use the unchanged unoptimized test profile.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---:|---|---:|---:|---:|---|
| public records | 2,000 seeds; 24,000 input checks | Base | 1 | 31,243,278,730 | 4.625519 (4.609839 + 0.015680) | 1 passed |
| public records | 2,000 seeds; 24,000 input checks | Fixed inputs once | 1 | 18,975,274,038 | 2.767242 (2.767242 + 0) | 1 passed |

Saved **12,268,004,692 Ir (39.27%)** and **1.858277 native CPU s (40.17%)**.
Each arm has one Callgrind run and one separate native run. Native wall times
were 4.974628 and 2.772042 s; load averages were respectively
29.99/14.78/12.90 and 36.39/40.73/27.71. Instruction counts decide the gain;
CPU seconds are observations under load. Callgrind wall times were 172.76 and
115.85 s. Each profile has one process, so no totals need summing.

## Reproduce

From the slot root, build and obtain `binary-path` and `cwd` from the
`rust-suites` entry (46 test names, including five fuzzy names):

```sh
heavy cargo nextest list -p temper-codecs-world --test suite --message-format json
mkdir -p target/limits-fuzzy/public-wire
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-fuzzy/public-wire/<arm>.%p.out \
  <binary-path> --exact fuzzy::public_wire::public_records_decoders_never_panic_and_reencode_the_same --nocapture'
callgrind_annotate --inclusive=yes --threshold=100 --auto=no <profile>
callgrind_annotate --inclusive=no --threshold=100 --auto=no <profile>
```

Also annotate with source enabled (omit `--auto=no`) to read each `check`
call's inclusive count and calls. Run the same binary and exact name natively
through `heavy`, from `cwd`, with `CARGO_MANIFEST_DIR` set. GNU time was absent;
use `notes/measuring.md`'s Python `RUSAGE_CHILDREN` wrapper, untracked under
`target/limits-fuzzy/`, and remove it before pushing. All libtest runs must
report `1 passed`. Commands receive `</dev/null`. Compare the base revision and
this change in this slot; do not change compiler or profile settings.

## Public records: the largest by inclusive Ir

Runtime/test/unwind envelopes are omitted here; they contain the sweep rather
than explaining its cost. Shares overlap down the call tree and must not be
added. Calls are summed from Callgrind's incoming call edges; inline fragments
can have their own rows. The test's `decode` envelope is 97.31% before and
96.29% after, including successful records' measurement and re-encoding.

### Base

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 58.35% | 18,230,811,883 | 12,000 | `PublicLandings::decode` |
| 58.34% | 18,228,023,642 | 12,000 | `PublicLandings::decode_from` |
| 42.38% | 13,242,352,144 | 352,927 | `PublicQueue::decode_from` |
| 23.21% | 7,252,266,490 | 31,657,018 | `List<T>::push` |
| 21.66% | 6,768,117,360 | 3,980 | `PublicLandings::encode` |
| 19.52% | 6,098,506,240 | 254,720 | `PublicQueue::encode` |
| 17.57% | 5,488,005,635 | 22,400,023 | `PublicQueued::decode_from` |
| 16.33% | 5,102,551,040 | 16,302,080 | `PublicQueued::encode` |
| 13.98% | 4,368,622,792 | 479,413 | `PublicLandingLine::decode_from` |
| 12.03% | 3,757,506,060 | 3,980 | `PublicLandings::measure` |
| 11.56% | 3,613,203,200 | 254,720 | `PublicQueue::measure` |
| 11.03% | 3,446,828,303 | 31,657,018 | `List<T>::grow` |
| 10.47% | 3,271,872,627 | 56,244,704 | `Writer::put` |
| 8.89% | 2,778,279,683 | inline | `List<T>::grow` (fragment in `raw_vec/mod.rs`) |
| 7.30% | 2,282,291,200 | 16,302,080 | `PublicQueued::measure` |
| 4.96% | 1,551,193,882 | 31,657,018 | `List<T>::room` |

### Fixed inputs once

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 63.80% | 12,105,826,351 | 8,002 | `PublicLandings::decode` |
| 63.79% | 12,103,535,955 | 8,002 | `PublicLandings::decode_from` |
| 44.29% | 8,403,245,090 | 224,991 | `PublicQueue::decode_from` |
| 25.53% | 4,845,168,571 | 20,894,402 | `List<T>::push` |
| 18.35% | 3,481,969,155 | 14,212,119 | `PublicQueued::decode_from` |
| 17.75% | 3,368,753,892 | 1,981 | `PublicLandings::encode` |
| 16.89% | 3,204,158,018 | 351,477 | `PublicLandingLine::decode_from` |
| 16.00% | 3,035,462,528 | 126,784 | `PublicQueue::encode` |
| 13.38% | 2,539,737,088 | 8,114,176 | `PublicQueued::encode` |
| 12.29% | 2,331,468,192 | 20,894,402 | `List<T>::grow` |
| 9.94% | 1,885,228,362 | inline | `List<T>::grow` (fragment in `raw_vec/mod.rs`) |
| 9.86% | 1,870,256,157 | 1,981 | `PublicLandings::measure` |
| 9.48% | 1,798,431,040 | 126,784 | `PublicQueue::measure` |
| 8.55% | 1,623,029,461 | 27,898,884 | `Writer::put` |
| 5.99% | 1,135,984,640 | 8,114,176 | `PublicQueued::measure` |
| 5.40% | 1,023,825,698 | 20,894,402 | `List<T>::room` |

## Public records: the largest by self Ir

These are the first 12 self-cost rows of each annotation; compiler/library
helpers appear under the names Callgrind gives them.

### Base

| Share | Ir | Function |
|---:|---:|---|
| 9.91% | 3,094,947,000 | `<core::result::Result<T,E> as core::ops::try_trait::Try>::branch` |
| 6.45% | 2,016,002,070 | `PublicQueued::decode_from` |
| 5.27% | 1,645,069,954 | `__memcpy_avx_unaligned_erms` |
| 5.22% | 1,631,912,128 | `core::option::Option<T>::ok_or` |
| 3.59% | 1,120,137,856 | `core::num::<impl u32>::checked_add` |
| 3.41% | 1,064,115,412 | `PublicQueue::decode_from` |
| 3.39% | 1,058,082,840 | `core::option::Option<T>::expect` |
| 3.34% | 1,043,333,120 | `PublicQueued::encode` |
| 3.30% | 1,031,779,452 | `core::result::Result<T,E>::expect` |
| 3.24% | 1,010,947,905 | `List<T>::push` |
| 3.06% | 956,159,968 | `Writer::put` |
| 2.35% | 733,337,010 | `alloc::vec::Vec<T,A>::push_mut` |

### Fixed inputs once

| Share | Ir | Function |
|---:|---:|---|
| 9.63% | 1,827,529,026 | `<core::result::Result<T,E> as core::ops::try_trait::Try>::branch` |
| 6.74% | 1,279,090,710 | `PublicQueued::decode_from` |
| 5.57% | 1,056,226,115 | `core::option::Option<T>::ok_or` |
| 5.57% | 1,056,073,545 | `__memcpy_avx_unaligned_erms` |
| 3.64% | 691,095,879 | `core::result::Result<T,E>::expect` |
| 3.56% | 675,189,972 | `PublicQueue::decode_from` |
| 3.51% | 666,975,977 | `List<T>::push` |
| 2.93% | 555,588,272 | `core::num::<impl u32>::checked_add` |
| 2.77% | 524,775,627 | `core::option::Option<T>::expect` |
| 2.74% | 519,307,264 | `PublicQueued::encode` |
| 2.55% | 484,261,610 | `alloc::vec::Vec<T,A>::push_mut` |
| 2.50% | 474,281,028 | `Writer::put` |

## Causes, largest first

| Cause | Where | Share of each sweep | What it saved, or whose it is |
|---|---|---|---|
| Repeated bound-record decode, measure and re-encode | `public_wire.rs`, `sweep`'s "at every bound" | 12,153,841,267 Ir / 38.90% before; 6,078,958 Ir / 0.032% after | 6,000 calls become three; saved 12,147,762,309 Ir on this check edge. |
| Decoding and re-encoding retained mutated bound records | Generated `public_v1.rs`; `skein_lib` | After: `PublicLandings::decode` 63.80%, encode 17.75%, measure 9.86% (overlapping lower paths) | The sweep exists to check these inputs; retained. Outside this task; details below. |
| Repeated fixed ceiling-refusal input and its reconstruction | `public_wire.rs`, "length past ceiling" | Check edge 8,698,000 Ir / 0.028% before; 4,385 Ir after; old extra refill 137,557,746 Ir / 0.44% | 6,000 checks become three. Check-edge saving 8,693,615 Ir, plus removed repeated copying/prefix setup. The helper still checks each kind. |
| Resetting scratch bytes between drawn mutations | `sweep`, two `extend_from_slice` calls | After: 275,167,198 Ir / 1.45% together; initial `to_vec` another 0.75% | Initial materialization stays. Undoing mutations might avoid the two refills, but their entire footprint is below 2%; stopped. |
| Arbitrary byte generation | `temper/tests/codecs/src/lib.rs`, `drawn` | 95,558,174 Ir / 0.31% before; 95,557,775 Ir / 0.50% after | Below 2% even if the entire function vanished. No shared helper change; other four sweeps are untouched. |
| Per-check unwind envelope | `check` and standard unwind helpers | After: about 2.30 million self Ir including the decode closure / 0.012% | Inclusive 96% is decoder work, not unwind overhead. Retained for seed/kind/case/input diagnostics. |

The whole-test saving includes allocator/copy-path differences and the small
new cost of formatting one context per seed (1,155,518 Ir / 0.006% after).
The edge savings are attribution, not separate native timing experiments.
There is one fix arm; no claim of independent CPU seconds for its subparts.

## Inputs and assertions retained

Each seed in `1..=2000` still makes one arbitrary draw, shared across the three
kinds, then for each kind checks arbitrary, flipped, inserted and removed
bytes: 12 checks per seed, 24,000 in all. Every RNG call, its argument and its
order are unchanged; `drawn`, fixture construction and the decoder are unchanged.
Neither moved fixed check consumed the RNG. Each kind's fixed pair now runs
once before the seed loop, through the same `check` and decode macro: six fixed
checks instead of 12,000. Successful bound records must decode and encode back
to identical bytes; ceiling-plus-one lengths must be refused at
`PublicChangeGates`, `PublicLandingsLandings`, or `PublicLandingMembers`.

Before changing the sweep, temporary tracing inside the existing `check`
captured seed 1's 12 raw drawn inputs and, for all seeds, one row containing
seed, kind, case, byte length and `DefaultHasher` digest of the input bytes.
Afterward the same tracing captured the same 24,000 rows in the same order.
The manifests compare byte for byte; their SHA-256 is
`98dc42ab0e751065e402378ae1cf52ec117b8a20aa85c3ddca93e32088b6d49e`.
All 12 seed-1 raw byte files also compare byte for byte. This supplements the
unchanged RNG/data-flow argument; the digest comparison alone is not a proof
against hash collisions. Seed 1's arbitrary input has 290 bytes; the bound
record lengths are 822 (`Change`), 67,405 (`Landings`) and 1,562 (`Landing`),
with inserted/removed lengths one larger/smaller.

A seed context formatted once per sweep supplies the same seeded failure text.
`check` still catches decoding panics and prints seed, kind, case and full
input bytes; re-encoding equality prints the compared bytes. Fixed cases name
kind and case without a seed. There are no added, removed or renamed tests,
and the base/final nextest lists have the same 46 names. All capture code and
artifacts are removed before pushing.

## Left

For `under-causes`, all figures below are inclusive after-arm footprint, not
predicted removable work, and overlap:

- `temper/crates/temper-client-protocol-wire/src/generated/public_v1.rs`:
  `PublicLandings::decode` is 12,105,826,351 Ir (63.80%);
  `PublicQueue::decode_from` 8,403,245,090 (44.29%);
  `PublicQueued::decode_from` 3,481,969,155 (18.35%). Queues at their ceilings
  decode thousands of scalar records. `PublicLandings::encode` is
  3,368,753,892 (17.75%); `measure` 1,870,256,157 (9.86%). Measurement visits
  every queued scalar record even though `PublicQueued` has a fixed wire size.
- `skein/crates/skein-lib/src/list.rs`: `List::push` is 4,845,168,571 Ir
  (25.53%), including `grow` 2,331,468,192 (12.29%). Reading shows every push
  checks room and calls `grow`; physical storage grows while the logical cap
  was set from the validated wire count. These are decoder/list mechanics,
  not fixture construction repeated by the test.
- `skein/crates/skein-lib/src/writer.rs`: `Writer::put` is 1,623,029,461 Ir
  (8.55%), checking remaining room and extending for each scalar write.
- `skein/crates/skein-lib/src/reader.rs`: scalar reads together are
  797,987,918 Ir (4.21%): u16 469,459,905, u64 220,379,625,
  u8 93,076,568, u32 15,071,820. These check/split the input then convert
  big-endian bytes. Generated `Result`/`Option` propagation also appears
  prominently in self Ir. No saving is assumed from removing required checks.

## Stopped at

The next large cause is the decoder's work on retained bound-record mutations:
the decode/measure/encode envelope is 96.29%, and removing it would remove the
checks. It stays. Within this world's removable setup, even eliminating both
remaining scratch refills entirely offers only 1.45%; `drawn` offers at most
0.50%, and the unwind envelope about 0.012%. No next local cause offers about
2% of this sweep. The shared helpers and the world's other four sweeps stay
unchanged; only their ordinary package acceptance runs are needed.
