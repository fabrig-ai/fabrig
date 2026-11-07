# What every world pays to count its heap and keep its wire's trace

2026-11-07 02:27:10 UTC, base `ca05b26eec`, after `7d40b8bf6f`. Instruction counts under callgrind, each test by its name, unoptimized as the suites run (`skein/docs/foundation/testing-strategy.md`, 8).

## The tests

Each reading ran exactly one test and passed, in its nextest working directory with `CARGO_MANIFEST_DIR` set. Before is the pristine-base arm, not an intermediate allocator-only or trace-only result. After includes all four shared-helper fixes and both earlier accounting/trace changes. Totals include startup and teardown. No budget, timeout, build setting, suite membership or existing assertion changed.

| Package | Test | Profile | Before | After | Change |
|---|---|---|---:|---:|---:|
| `jig-core-world` | T1: `stories::address::a_person_task_is_readdressed_after_durability_and_the_previous_taker_cannot_answer_after_restart` | `default` | 109,407,314 | 94,854,180 | −13.30% |
| `jig-client-protocol-world` | T2: `client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store` | `default` | 153,039,757 | 115,566,545 | −24.49% |
| `skein-forge-machine-world` | T3: `hand::github::github_installation_loss::terminal_deferred_viewer_status_stops_after_connected` | `default` | 35,904,657 | 31,721,364 | −11.65% |
| `skein-llm-world` | T4: `client::reuse_preserves_binding_and_rejects_other_authority` | `default` | 11,432,807 | 10,419,957 | −8.86% |
| `skein-tls-world` | T5: `handshake::a_chain_longer_than_the_records_held_is_too_long` | `default` | 71,385,712 | 59,871,433 | −16.13% |
| `jig-ops-world` | T6: `fuzzy::joint::p_accepts_the_restart_from_the_incidents_page` | `fuzzy` | 60,486,730,374 | 44,528,952,467 | −26.38% |

## The causes

T1–T6 name the exact tests above. Counts are inclusive; wrappers and their children overlap and must not be added. For each allocator method, its TLS, padding/layout, libc and rest rows partition its total. TLS means the `LocalKey::with` edge minus the accounting closure, plus that closure’s Cell get/set edges; the accounting body is not charged twice.

Before arms are identified here so their denominators remain explicit: allocator rows use the pristine totals above; Wire’s T2 rows use its independent pristine reading of 153,256,084 Ir (0.14% above the other pristine reading); T3–T5 helper rows use task base `7fc50fc4ac` (31,721,051, 10,622,141 and 70,685,306 Ir). T6’s empty-write rows use `99327d988b` (55,548,693,627 Ir); its text-run rows use `f89237da2f` after the empty guard (44,854,311,773 Ir). A dash means that subcause was not isolated in that before reading. Every after share uses the final total above.

Locations: H = `skein/testing/skein-heap/src/lib.rs`; W = `skein/testing/skein-world/src/stream.rs`; L = `skein/testing/skein-llm-kit/src/lib.rs`; T = `skein/testing/skein-tls-kit/src/drive.rs`; P = `skein/testing/skein-fake-person/src/tree.rs`.

| Cause | Where | Test | Share before | Share after | Fixed |
|---|---|---|---:|---:|---|
| `alloc`: total | H | T1 | 23,852,956 Ir (21.80%) | 14,980,780 Ir (15.79%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T1 | 9,016,644 Ir (8.24%) | 2,056,904 Ir (2.17%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T1 | 5,300,545 Ir (4.84%) | 2,771,235 Ir (2.92%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T1 | 1,874,169 Ir (1.71%) | 1,874,169 Ir (1.98%) | No; required System allocation |
| `alloc`: rest | H | T1 | 7,661,598 Ir (7.00%) | 8,278,472 Ir (8.73%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T1 | 20,058,324 Ir (18.33%) | 14,362,137 Ir (15.14%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T1 | 5,586,222 Ir (5.11%) | 1,869,405 Ir (1.97%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T1 | 5,300,304 Ir (4.84%) | 3,013,032 Ir (3.18%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T1 | 1,870,602 Ir (1.71%) | 1,870,602 Ir (1.97%) | No; required System allocation |
| `dealloc`: rest | H | T1 | 7,301,196 Ir (6.67%) | 7,609,098 Ir (8.02%) | No; required headers, conversions and calls |
| `alloc`: total | H | T2 | 27,828,465 Ir (18.18%) | 14,144,042 Ir (12.24%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T2 | 10,588,530 Ir (6.92%) | 1,956,541 Ir (1.69%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T2 | 6,189,115 Ir (4.04%) | 2,636,544 Ir (2.28%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T2 | 2,094,881 Ir (1.37%) | 1,675,854 Ir (1.45%) | No; required System allocation |
| `alloc`: rest | H | T2 | 8,955,939 Ir (5.85%) | 7,875,103 Ir (6.81%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T2 | 23,440,827 Ir (15.32%) | 13,637,050 Ir (11.80%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T2 | 6,522,720 Ir (4.26%) | 1,778,540 Ir (1.54%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T2 | 6,188,874 Ir (4.04%) | 2,866,582 Ir (2.48%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T2 | 2,203,953 Ir (1.44%) | 1,752,704 Ir (1.52%) | No; required System allocation |
| `dealloc`: rest | H | T2 | 8,525,280 Ir (5.57%) | 7,239,224 Ir (6.26%) | No; required headers, conversions and calls |
| `alloc`: total | H | T3 | 2,703,492 Ir (7.53%) | 1,301,811 Ir (4.10%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T3 | 1,046,310 Ir (2.91%) | 184,468 Ir (0.58%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T3 | 612,857 Ir (1.71%) | 247,206 Ir (0.78%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T3 | 156,922 Ir (0.44%) | 128,325 Ir (0.40%) | No; required System allocation |
| `alloc`: rest | H | T3 | 887,403 Ir (2.47%) | 741,812 Ir (2.34%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T3 | 2,297,257 Ir (6.40%) | 1,265,754 Ir (3.99%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T3 | 645,668 Ir (1.80%) | 166,685 Ir (0.53%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T3 | 612,616 Ir (1.71%) | 268,651 Ir (0.85%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T3 | 195,983 Ir (0.55%) | 152,672 Ir (0.48%) | No; required System allocation |
| `dealloc`: rest | H | T3 | 842,990 Ir (2.35%) | 677,746 Ir (2.14%) | No; required headers, conversions and calls |
| `alloc`: total | H | T4 | 1,317,226 Ir (11.52%) | 800,212 Ir (7.68%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T4 | 512,184 Ir (4.48%) | 110,660 Ir (1.06%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T4 | 286,302 Ir (2.50%) | 146,910 Ir (1.41%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T4 | 100,224 Ir (0.88%) | 99,022 Ir (0.95%) | No; required System allocation |
| `alloc`: rest | H | T4 | 418,516 Ir (3.66%) | 443,620 Ir (4.26%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T4 | 1,077,311 Ir (9.42%) | 755,808 Ir (7.25%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T4 | 301,498 Ir (2.64%) | 99,025 Ir (0.95%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T4 | 286,061 Ir (2.50%) | 159,599 Ir (1.53%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T4 | 96,156 Ir (0.84%) | 94,582 Ir (0.91%) | No; required System allocation |
| `dealloc`: rest | H | T4 | 393,596 Ir (3.44%) | 402,602 Ir (3.86%) | No; required headers, conversions and calls |
| `alloc`: total | H | T5 | 1,163,821 Ir (1.63%) | 713,915 Ir (1.19%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T5 | 446,682 Ir (0.63%) | 94,470 Ir (0.16%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T5 | 242,177 Ir (0.34%) | 124,208 Ir (0.21%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T5 | 118,613 Ir (0.17%) | 117,677 Ir (0.20%) | No; required System allocation |
| `alloc`: rest | H | T5 | 356,349 Ir (0.50%) | 377,560 Ir (0.63%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T5 | 919,060 Ir (1.29%) | 646,760 Ir (1.08%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T5 | 255,016 Ir (0.36%) | 83,725 Ir (0.14%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T5 | 241,936 Ir (0.34%) | 134,917 Ir (0.23%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T5 | 89,258 Ir (0.13%) | 87,786 Ir (0.15%) | No; required System allocation |
| `dealloc`: rest | H | T5 | 332,850 Ir (0.47%) | 340,332 Ir (0.57%) | No; required headers, conversions and calls |
| `alloc`: total | H | T6 | 5,561,656,650 Ir (9.19%) | 3,299,278,416 Ir (7.41%) | Partly; exact accounting retained |
| `alloc`: LocalKey/cell reads and writes | H | T6 | 2,059,047,360 Ir (3.40%) | 437,428,193 Ir (0.98%) | One TLS reach; access remains |
| `alloc`: padding and layout construction | H | T6 | 1,226,513,581 Ir (2.03%) | 592,437,755 Ir (1.33%) | Repeated validation removed; layout work remains |
| `alloc`: malloc/free, including aligned allocation | H | T6 | 507,925,599 Ir (0.84%) | 505,801,755 Ir (1.14%) | No; required System allocation |
| `alloc`: rest | H | T6 | 1,768,170,110 Ir (2.92%) | 1,763,610,713 Ir (3.96%) | No; required headers, conversions and calls |
| `dealloc`: total | H | T6 | 4,698,742,458 Ir (7.77%) | 3,123,241,542 Ir (7.01%) | Partly; exact accounting retained |
| `dealloc`: LocalKey/cell reads and writes | H | T6 | 1,290,952,968 Ir (2.13%) | 399,084,435 Ir (0.90%) | One TLS reach; access remains |
| `dealloc`: padding and layout construction | H | T6 | 1,224,880,565 Ir (2.03%) | 643,230,200 Ir (1.44%) | Repeated validation removed; layout work remains |
| `dealloc`: malloc/free, including aligned allocation | H | T6 | 495,522,067 Ir (0.82%) | 456,418,987 Ir (1.02%) | No; required System allocation |
| `dealloc`: rest | H | T6 | 1,687,386,858 Ir (2.79%) | 1,624,507,920 Ir (3.65%) | No; required headers, conversions and calls |
| `Wire::take` | W | T2 | 6,567,946 Ir (4.29%) | 258,516 Ir (0.22%) | Yes; defer trace formatting |
| `Wire::answer` | W | T2 | 16,606,637 Ir (10.84%) | 2,266,347 Ir (1.96%) | Yes; defer trace formatting |
| `alloc::fmt::format` from `Wire::take` | W | T2 | 6,346,129 Ir (4.14%) | 0 Ir (0.00%) | Yes; no formatting on these passing paths |
| `alloc::fmt::format` from `Wire::answer` | W | T2 | 12,549,691 Ir (8.19%) | 0 Ir (0.00%) | Yes; no formatting on these passing paths |
| `alloc::fmt::format`, whole process | library/fixtures | T2 | 20,967,655 Ir (13.68%) | 1,576,664 Ir (1.36%) | Wire part fixed; fixture formatting remains |
| temporary fragment collection | W | T2 | 1,772,991 Ir (1.16%) | 0 Ir (0.00%) | Yes; copy queue slices |
| temporary fragment vector drop | W | T2 | 670,432 Ir (0.44%) | 0 Ir (0.00%) | Yes; no temporary fragment vector |
| `Wire::write` | W | T2 | 2,288,388 Ir (1.49%) | 180,322 Ir (0.16%) | Empty calls skipped; nonempty extension retained |
| RSA signing | `skein/crates/skein-tls/src/rs256.rs` | T3 | 22,580,773 Ir (71.19%) | 22,580,773 Ir (71.18%) | No; product/cryptography |
| `Harness::drain` | `skein/tests/forge/machine/src/hand.rs` | T3 | 6,164,510 Ir (19.43%) | 6,164,510 Ir (19.43%) | No; test-local fixture, includes product calls |
| `Wire::answer` | `skein/testing/skein-world/src/stream.rs` | T3 | 408,028 Ir (1.29%) | 408,028 Ir (1.29%) | Retained; each individual child below 1% |
| `Trace::clear` | `skein/testing/skein-world/src/stream.rs` | T3 | 229,890 Ir (0.72%) | 229,890 Ir (0.72%) | Retained; below 1%, frees required trace storage |
| `Wire::take` | `skein/testing/skein-world/src/stream.rs` | T3 | 88,050 Ir (0.28%) | 88,050 Ir (0.28%) | Retained; below 1% |
| `World::run` | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 6,341,707 Ir (59.70%) | 6,342,593 Ir (60.87%) | Wrapper; mostly product execution |
| `client::resume` | `skein/crates/skein-llm/src/client.rs` | T4 | 5,733,259 Ir (53.97%) | 5,733,937 Ir (55.03%) | No; product |
| JSON `collector::up` | `skein/crates/skein-json/src/collector.rs` | T4 | 2,571,482 Ir (24.21%) | 2,571,630 Ir (24.68%) | No; product |
| `Client::prepare` | `skein/crates/skein-llm/src/client.rs` | T4 | 2,371,343 Ir (22.32%) | 2,371,720 Ir (22.76%) | No; product |
| JSON `string::decode` | `skein/crates/skein-json/src/string.rs` | T4 | 834,576 Ir (7.86%) | 834,408 Ir (8.01%) | No; another plan’s string path |
| `text_response` | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 308,759 Ir (2.91%) | 104,518 Ir (1.00%) | Its event assembly fixed; response framing retained |
| formatted byte-by-byte event assembly | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 230,192 Ir (2.17%) | 25,923 Ir (0.25%) | Yes; assemble literal slices |
| `World::next_turn` | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 181,051 Ir (1.70%) | 181,259 Ir (1.74%) | Retained; product callees and required scheduling |
| `World::take` | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 81,471 Ir (0.77%) | 81,471 Ir (0.78%) | Retained; below 1% |
| `response` framing | `skein/testing/skein-llm-kit/src/lib.rs` | T4 | 76,699 Ir (0.72%) | 76,727 Ir (0.74%) | Retained; below 1% |
| `Pair::settle` | `skein/testing/skein-tls-kit/src/drive.rs` | T5 | 62,297,304 Ir (88.13%) | 51,483,987 Ir (85.99%) | Wrapper; both TLS implementations dominate |
| `Server::receive` | `skein/testing/skein-tls-kit/src/server.rs` | T5 | 21,968,215 Ir (31.08%) | 21,967,127 Ir (36.69%) | Wrapper; mostly rustls |
| `Wire::answer`, drain collection replaced | `skein/testing/skein-tls-kit/src/drive.rs` | T5 | 10,977,233 Ir (15.53%) | 165,190 Ir (0.28%) | Yes; copy both queue slices and drain prefix |
| server configuration | `skein/testing/skein-tls-kit/src/pki.rs` | T5 | 1,391,687 Ir (1.97%) | 1,391,687 Ir (2.32%) | Retained; required certificate/key setup |
| `Wire::pull` | `skein/testing/skein-tls-kit/src/drive.rs` | T5 | 354,155 Ir (0.50%) | 354,154 Ir (0.59%) | Retained; below 1% |
| TLS `client::up` | `skein/crates/skein-tls/src/client.rs` | T5 | 28,938,273 Ir (40.94%) | 28,938,182 Ir (48.33%) | No; product |
| loader `_dl_start` | libc loader | T3 | 1,731,277 Ir (5.46%) | 1,733,601 Ir (5.47%) | No; process startup |
| loader `_dl_start` | libc loader | T4 | 691,578 Ir (6.51%) | 692,201 Ir (6.64%) | No; process startup |
| loader `_dl_start` | libc loader | T5 | 1,132,364 Ir (1.60%) | 1,133,192 Ir (1.89%) | No; process startup |
| drain collection, within `answer` | T | T5 | 10,926,497 Ir (15.46%) | 0 Ir (0.00%) | Yes; no per-byte drain iterator collection |
| peer packet processing | rustls | T5 | 21,577,873 Ir (30.53%) | 21,576,785 Ir (36.04%) | No; rustls dependency |
| certificate/key configuration | rustls | T5 | 1,339,731 Ir (1.90%) | 1,339,731 Ir (2.24%) | No; rustls dependency |
| formatting within `events` | L | T4 | 37,322 Ir (0.35%) | 0 Ir (0.00%) | Yes; no formatted intermediate strings |
| empty/nonempty write dispatch | W | T6 | 11,384,603,071 Ir (20.49%) | 690,200,616 Ir (1.55%) | Empty extension fixed; required calls remain |
| trace length observations | W | T6 | 551,300,960 Ir (0.99%) | 551,300,960 Ir (1.24%) | Retained; fixture polling and direct accessor |
| byte-by-byte text-run assembly | P | T6 | 491,251,309 Ir (1.10%) | 165,995,676 Ir (0.37%) | Yes; per-run slice copies |
| Unicode case-insensitive matching | P | T6 | 479,127,155 Ir (1.07%) | 479,127,155 Ir (1.08%) | Retained; required matching, not an identified 1% gain |
| tree query wrapper | P | T6 | 1,132,730,481 Ir (2.53%) | 807,474,435 Ir (1.81%) | Retained; required matching, not an identified 1% gain |
| `Person::next` | skein/testing/skein-fake-person/src/lib.rs | T6 | 1,038,201,860 Ir (1.87%) | 721,826,120 Ir (1.62%) | Wrapper around the same tree matching |

The helper changes keep byte order, RNG draws and draw order, trace/failure text, demands, fill lengths and stream counters. No public signature or caller changed. Three new tests pin wrapped fills, exact event bytes and empty-write scheduling. Each was made to fail for its rule: omit the second queue slice, remove one framing newline, or consume an RNG draw on an empty write; restoring the rule passes. The text-run change is covered by the unchanged joined-run, Unicode, region and drawing-fallback tests.

## What the code can't reach

`Counting` still pays for one TLS reach, Cell operations, layout construction/preconditions, allocation-number headers, size conversions, System allocation and call boundaries. The partition above gives every cause for every test. A `[profile.dev.package.skein-heap]` override would reach the remaining unoptimized work: the user’s to sign off under `skein/docs/foundation/testing-strategy.md`, 8, with these numbers. None is added. It would not remove the required libc allocation itself. These subsets of the partition show the retained library work; do not add them to its totals.

| Test | Header write | Header read | Size conversion, alloc + free | Dealloc unchecked layout |
|---|---:|---:|---:|---:|
| T1 | 4,223,328 Ir (4.45%) | 4,310,148 Ir (4.54%) | 2,155,363 Ir (2.27%) | 1,935,384 Ir (2.04%) |
| T2 | 4,018,080 Ir (3.48%) | 4,100,624 Ir (3.55%) | 2,050,601 Ir (1.77%) | 1,841,312 Ir (1.59%) |
| T3 | 377,464 Ir (1.19%) | 383,596 Ir (1.21%) | 192,227 Ir (0.61%) | 172,568 Ir (0.54%) |
| T4 | 224,360 Ir (2.15%) | 227,852 Ir (2.19%) | 114,219 Ir (1.10%) | 102,520 Ir (0.98%) |
| T5 | 189,790 Ir (0.32%) | 192,582 Ir (0.32%) | 96,579 Ir (0.16%) | 86,680 Ir (0.14%) |
| T6 | 902,762,790 Ir (2.03%) | 920,241,270 Ir (2.07%) | 460,452,902 Ir (1.03%) | 413,169,768 Ir (0.93%) |

- Allocation keeps the safe padded layout constructor: the earlier measured choice costs 77 Ir/call, against 88 Ir/call for unchecked construction with its debug preconditions. Deallocation already knows the conditions and uses unchecked reconstruction, but its preconditions still execute. Headers must preserve allocation numbers for exact ownership hand-outs. No counter or peak assertion was weakened.
- T3 spends 22,580,773 Ir (71.19%) in product RSA signing. Its own `Harness::drain` fixture spends 6,164,510 Ir (19.43%), including product calls; it is under `skein/tests/forge/machine/src/hand.rs`, outside this helper scope. This median is dominated by code under test. Startup shares for all three profiled medians are in the table.
- T4’s product resume, admission, JSON collection and string decoding costs are in the table. `skein/crates/skein-json/src/string.rs` is another plan’s. Its remaining `World::next_turn` cost includes product intake/state queries; each individual scheduling or copy operation is below 1%. Its response framing is also below 1%.
- T5’s product TLS client and peer rustls processing dominate after the fill fix. Almost all server configuration is rustls certificate/key validation. Caching configurations would change the ownership/lifecycle of the setup exercised by the tests; the wrapper outside that library call is only 51,956 Ir (0.09%). Each other TLS helper cause is below 1%.
- T6’s remaining `Wire::write` wrapper still receives 27,565,048 calls. Only nonempty slices extend the queue; that extension is now just 1,067,371 Ir (0.0024%). The remaining dispatch and slice-emptiness work is the unoptimized public call path; removing it would require compiler inlining or fewer fixture calls. Changing the fixture’s polling belongs to `jig/testing/jig-client-protocol-kit/src/net.rs`; that file stays unchanged. The earlier nonempty `.iter().copied()` queue trial was more expensive (2,541,817 Ir versus 2,288,388 Ir in T2 before), so the nonempty path stays as it was.
- T6’s `Trace::len` has 55,130,096 calls: 275,650,480 Ir (0.62%) is the wrapper and the same amount is `Vec::len`. Removing the wrapper alone cannot save 1% and the public length observation remains required. Polling twice per turn is the fixture’s decision.
- T6’s Unicode case-insensitive matcher remains 479,127,155 Ir (1.08%): lowercasing and temporary-string drops are 194,430,050 Ir (0.44%) and 211,932,781 Ir (0.48%). No single identified removable cause reaches 1%. Preserving Unicode matching requires more than deleting those calls; an ASCII-only comparison would change the contract. The 1.81% tree query and its `Person::next` caller include this matching and the now 0.37% text assembler. The last small slice fix saved 325,359,306 Ir (0.73%) of T6, and no further identified helper gain reaches about 1%.
- T1 store request handling: 8,000,475 Ir (8.43%) in `skein/crates/skein-kv/src/store.rs`. This is outside the shared-helper scope; wrapper totals include their callees.
- T2 fixture naming: 1,155,305 Ir (1.00%) in `jig/testing/jig-conformance/src/core/naming.rs`. This is outside the shared-helper scope; wrapper totals include their callees.
- T6 joint byte-pump fixture: 22,252,285,111 Ir (49.97%) in `jig/tests/ops/src/joint/bytes.rs`. This is outside the shared-helper scope; wrapper totals include their callees.
- T2’s retained fragment work is slice extension 416,122 Ir (0.36%), queue slices 123,912 Ir (0.11%), queue drain 91,104 Ir (0.08%), drain drop 45,716 Ir (0.04%). The formatter’s remaining fixture cost is shown above; none is from passing Wire answer/take paths.
