# counting: one reach of a thread’s counts, with every count unchanged

2026-11-07 UTC; base `ca05b26eec`, after `3d8dcd2669`; 16 CPUs, AMD EPYC-Genoa Processor; valgrind 3.24.0.

These are unoptimized Callgrind instruction readings, one test by exact name with `--nocapture`, in its nextest cwd and with `CARGO_MANIFEST_DIR` set. Every reading ran and passed exactly one test. `PROGRAM TOTALS` includes startup and teardown. No build settings, budgets or existing assertions changed; no existing test moved or left its suite.

## The six tests

The default selection run passed 1,792 tests and the fuzzy selection run passed 17. Each median is the middle passing duration printed by nextest with `--status-level all`, with equal durations ordered by name. The client protocol test is prescribed. Later readings keep these names.

| Key | Package | Profile | Test | Selection duration |
|---|---|---|---|---:|
| T1 | `jig-core-world` | `default` | `stories::address::a_person_task_is_readdressed_after_durability_and_the_previous_taker_cannot_answer_after_restart` | 0.042 s |
| T2 | `jig-client-protocol-world` | `default` | `client::account_keys::a_key_ask_carries_its_value_into_the_secrets_store` | prescribed |
| T3 | `skein-forge-machine-world` | `default` | `hand::github::github_installation_loss::terminal_deferred_viewer_status_stops_after_connected` | 0.014 s |
| T4 | `skein-llm-world` | `default` | `client::reuse_preserves_binding_and_rejects_other_authority` | 0.013 s |
| T5 | `skein-tls-world` | `default` | `handshake::a_chain_longer_than_the_records_held_is_too_long` | 0.02 s |
| T6 | `jig-ops-world` | `fuzzy` | `fuzzy::joint::p_accepts_the_restart_from_the_incidents_page` | 13.749 s |

## Whole-test totals

| Test | Before Ir | After Ir | Reduction |
|---|---:|---:|---:|
| T1 | 109,407,314 | 94,853,537 | 13.302% |
| T2 | 153,039,757 | 135,985,297 | 11.144% |
| T3 | 35,904,657 | 34,218,240 | 4.697% |
| T4 | 11,432,807 | 10,623,205 | 7.081% |
| T5 | 71,385,712 | 70,681,989 | 0.986% |
| T6 | 60,486,730,374 | 57,142,896,007 | 5.528% |

## Allocator paths by cause

The key names the package and exact test in the table above. Shares use the whole test as denominator. Each method’s four cause rows partition its inclusive total; the total row is their sum. Padding includes `padded` and layout construction, which are one callee before and separate callees after. The libc row follows the System call’s malloc/free or aligned-allocation edges. Rest includes the System wrapper, header access, count arithmetic, conversions and call overhead.

LocalKey/cell rows include get/set callees before. After, `LocalKey::with` also includes the entire accounting closure: that closure’s inclusive count is subtracted, then its direct Cell get/set edges are added. This counts access machinery without attributing the accounting body to TLS or counting it twice. The highs are idle in these selected paths; their existing step tests are checked separately.

| Package | Test | Arm | Total Ir | Cause | Cause’s inclusive Ir | Share |
|---|---|---|---:|---|---:|---:|
| `jig-core-world` | T1 | before | 109,407,314 | `alloc`: total | 23,852,956 | 21.80% |
| `jig-core-world` | T1 | before | 109,407,314 | `alloc`: LocalKey/cell reads and writes | 9,016,644 | 8.24% |
| `jig-core-world` | T1 | before | 109,407,314 | `alloc`: padding and layout construction | 5,300,545 | 4.84% |
| `jig-core-world` | T1 | before | 109,407,314 | `alloc`: malloc/free, including aligned allocation | 1,874,169 | 1.71% |
| `jig-core-world` | T1 | before | 109,407,314 | `alloc`: rest | 7,661,598 | 7.00% |
| `jig-core-world` | T1 | before | 109,407,314 | `dealloc`: total | 20,058,324 | 18.33% |
| `jig-core-world` | T1 | before | 109,407,314 | `dealloc`: LocalKey/cell reads and writes | 5,586,222 | 5.11% |
| `jig-core-world` | T1 | before | 109,407,314 | `dealloc`: padding and layout construction | 5,300,304 | 4.84% |
| `jig-core-world` | T1 | before | 109,407,314 | `dealloc`: malloc/free, including aligned allocation | 1,870,602 | 1.71% |
| `jig-core-world` | T1 | before | 109,407,314 | `dealloc`: rest | 7,301,196 | 6.67% |
| `jig-core-world` | T1 | after | 94,853,537 | `alloc`: total | 14,980,780 | 15.79% |
| `jig-core-world` | T1 | after | 94,853,537 | `alloc`: LocalKey/cell reads and writes | 2,056,904 | 2.17% |
| `jig-core-world` | T1 | after | 94,853,537 | `alloc`: padding and layout construction | 2,771,235 | 2.92% |
| `jig-core-world` | T1 | after | 94,853,537 | `alloc`: malloc/free, including aligned allocation | 1,874,169 | 1.98% |
| `jig-core-world` | T1 | after | 94,853,537 | `alloc`: rest | 8,278,472 | 8.73% |
| `jig-core-world` | T1 | after | 94,853,537 | `dealloc`: total | 14,362,137 | 15.14% |
| `jig-core-world` | T1 | after | 94,853,537 | `dealloc`: LocalKey/cell reads and writes | 1,869,405 | 1.97% |
| `jig-core-world` | T1 | after | 94,853,537 | `dealloc`: padding and layout construction | 3,013,032 | 3.18% |
| `jig-core-world` | T1 | after | 94,853,537 | `dealloc`: malloc/free, including aligned allocation | 1,870,602 | 1.97% |
| `jig-core-world` | T1 | after | 94,853,537 | `dealloc`: rest | 7,609,098 | 8.02% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `alloc`: total | 27,828,465 | 18.18% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `alloc`: LocalKey/cell reads and writes | 10,588,530 | 6.92% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `alloc`: padding and layout construction | 6,189,115 | 4.04% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `alloc`: malloc/free, including aligned allocation | 2,094,881 | 1.37% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `alloc`: rest | 8,955,939 | 5.85% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `dealloc`: total | 23,440,827 | 15.32% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `dealloc`: LocalKey/cell reads and writes | 6,522,720 | 4.26% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `dealloc`: padding and layout construction | 6,188,874 | 4.04% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `dealloc`: malloc/free, including aligned allocation | 2,203,953 | 1.44% |
| `jig-client-protocol-world` | T2 | before | 153,039,757 | `dealloc`: rest | 8,525,280 | 5.57% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `alloc`: total | 17,412,933 | 12.81% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `alloc`: LocalKey/cell reads and writes | 2,405,669 | 1.77% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `alloc`: padding and layout construction | 3,235,800 | 2.38% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `alloc`: malloc/free, including aligned allocation | 2,094,881 | 1.54% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `alloc`: rest | 9,676,583 | 7.12% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `dealloc`: total | 16,789,707 | 12.35% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `dealloc`: LocalKey/cell reads and writes | 2,182,800 | 1.61% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `dealloc`: padding and layout construction | 3,518,154 | 2.59% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `dealloc`: malloc/free, including aligned allocation | 2,203,953 | 1.62% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | `dealloc`: rest | 8,884,800 | 6.53% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `alloc`: total | 2,703,492 | 7.53% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `alloc`: LocalKey/cell reads and writes | 1,046,310 | 2.91% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `alloc`: padding and layout construction | 612,857 | 1.71% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `alloc`: malloc/free, including aligned allocation | 156,922 | 0.44% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `alloc`: rest | 887,403 | 2.47% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `dealloc`: total | 2,297,257 | 6.40% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `dealloc`: LocalKey/cell reads and writes | 645,668 | 1.80% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `dealloc`: padding and layout construction | 612,616 | 1.71% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `dealloc`: malloc/free, including aligned allocation | 195,983 | 0.55% |
| `skein-forge-machine-world` | T3 | before | 35,904,657 | `dealloc`: rest | 842,990 | 2.35% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `alloc`: total | 1,674,156 | 4.89% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `alloc`: LocalKey/cell reads and writes | 238,072 | 0.70% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `alloc`: padding and layout construction | 320,412 | 0.94% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `alloc`: malloc/free, including aligned allocation | 156,922 | 0.46% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `alloc`: rest | 958,750 | 2.80% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `dealloc`: total | 1,638,879 | 4.79% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `dealloc`: LocalKey/cell reads and writes | 216,070 | 0.63% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `dealloc`: padding and layout construction | 348,248 | 1.02% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `dealloc`: malloc/free, including aligned allocation | 195,983 | 0.57% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | `dealloc`: rest | 878,578 | 2.57% |
| `skein-llm-world` | T4 | before | 11,432,807 | `alloc`: total | 1,317,226 | 11.52% |
| `skein-llm-world` | T4 | before | 11,432,807 | `alloc`: LocalKey/cell reads and writes | 512,184 | 4.48% |
| `skein-llm-world` | T4 | before | 11,432,807 | `alloc`: padding and layout construction | 286,302 | 2.50% |
| `skein-llm-world` | T4 | before | 11,432,807 | `alloc`: malloc/free, including aligned allocation | 100,224 | 0.88% |
| `skein-llm-world` | T4 | before | 11,432,807 | `alloc`: rest | 418,516 | 3.66% |
| `skein-llm-world` | T4 | before | 11,432,807 | `dealloc`: total | 1,077,311 | 9.42% |
| `skein-llm-world` | T4 | before | 11,432,807 | `dealloc`: LocalKey/cell reads and writes | 301,498 | 2.64% |
| `skein-llm-world` | T4 | before | 11,432,807 | `dealloc`: padding and layout construction | 286,061 | 2.50% |
| `skein-llm-world` | T4 | before | 11,432,807 | `dealloc`: malloc/free, including aligned allocation | 96,156 | 0.84% |
| `skein-llm-world` | T4 | before | 11,432,807 | `dealloc`: rest | 393,596 | 3.44% |
| `skein-llm-world` | T4 | after | 10,623,205 | `alloc`: total | 814,642 | 7.67% |
| `skein-llm-world` | T4 | after | 10,623,205 | `alloc`: LocalKey/cell reads and writes | 112,750 | 1.06% |
| `skein-llm-world` | T4 | after | 10,623,205 | `alloc`: padding and layout construction | 149,682 | 1.41% |
| `skein-llm-world` | T4 | after | 10,623,205 | `alloc`: malloc/free, including aligned allocation | 100,224 | 0.94% |
| `skein-llm-world` | T4 | after | 10,623,205 | `alloc`: rest | 451,986 | 4.25% |
| `skein-llm-world` | T4 | after | 10,623,205 | `dealloc`: total | 769,878 | 7.25% |
| `skein-llm-world` | T4 | after | 10,623,205 | `dealloc`: LocalKey/cell reads and writes | 100,895 | 0.95% |
| `skein-llm-world` | T4 | after | 10,623,205 | `dealloc`: padding and layout construction | 162,613 | 1.53% |
| `skein-llm-world` | T4 | after | 10,623,205 | `dealloc`: malloc/free, including aligned allocation | 96,156 | 0.91% |
| `skein-llm-world` | T4 | after | 10,623,205 | `dealloc`: rest | 410,214 | 3.86% |
| `skein-tls-world` | T5 | before | 71,385,712 | `alloc`: total | 1,163,821 | 1.63% |
| `skein-tls-world` | T5 | before | 71,385,712 | `alloc`: LocalKey/cell reads and writes | 446,682 | 0.63% |
| `skein-tls-world` | T5 | before | 71,385,712 | `alloc`: padding and layout construction | 242,177 | 0.34% |
| `skein-tls-world` | T5 | before | 71,385,712 | `alloc`: malloc/free, including aligned allocation | 118,613 | 0.17% |
| `skein-tls-world` | T5 | before | 71,385,712 | `alloc`: rest | 356,349 | 0.50% |
| `skein-tls-world` | T5 | before | 71,385,712 | `dealloc`: total | 919,060 | 1.29% |
| `skein-tls-world` | T5 | before | 71,385,712 | `dealloc`: LocalKey/cell reads and writes | 255,016 | 0.36% |
| `skein-tls-world` | T5 | before | 71,385,712 | `dealloc`: padding and layout construction | 241,936 | 0.34% |
| `skein-tls-world` | T5 | before | 71,385,712 | `dealloc`: malloc/free, including aligned allocation | 89,258 | 0.13% |
| `skein-tls-world` | T5 | before | 71,385,712 | `dealloc`: rest | 332,850 | 0.47% |
| `skein-tls-world` | T5 | after | 70,681,989 | `alloc`: total | 726,217 | 1.03% |
| `skein-tls-world` | T5 | after | 70,681,989 | `alloc`: LocalKey/cell reads and writes | 96,259 | 0.14% |
| `skein-tls-world` | T5 | after | 70,681,989 | `alloc`: padding and layout construction | 126,602 | 0.18% |
| `skein-tls-world` | T5 | after | 70,681,989 | `alloc`: malloc/free, including aligned allocation | 118,613 | 0.17% |
| `skein-tls-world` | T5 | after | 70,681,989 | `alloc`: rest | 384,743 | 0.54% |
| `skein-tls-world` | T5 | after | 70,681,989 | `dealloc`: total | 659,061 | 0.93% |
| `skein-tls-world` | T5 | after | 70,681,989 | `dealloc`: LocalKey/cell reads and writes | 85,340 | 0.12% |
| `skein-tls-world` | T5 | after | 70,681,989 | `dealloc`: padding and layout construction | 137,520 | 0.19% |
| `skein-tls-world` | T5 | after | 70,681,989 | `dealloc`: malloc/free, including aligned allocation | 89,295 | 0.13% |
| `skein-tls-world` | T5 | after | 70,681,989 | `dealloc`: rest | 346,906 | 0.49% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `alloc`: total | 5,561,656,650 | 9.19% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `alloc`: LocalKey/cell reads and writes | 2,059,047,360 | 3.40% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `alloc`: padding and layout construction | 1,226,513,581 | 2.03% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `alloc`: malloc/free, including aligned allocation | 507,925,599 | 0.84% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `alloc`: rest | 1,768,170,110 | 2.92% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `dealloc`: total | 4,698,742,458 | 7.77% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `dealloc`: LocalKey/cell reads and writes | 1,290,952,968 | 2.13% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `dealloc`: padding and layout construction | 1,224,880,565 | 2.03% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `dealloc`: malloc/free, including aligned allocation | 495,522,067 | 0.82% |
| `jig-ops-world` | T6 | before | 60,486,730,374 | `dealloc`: rest | 1,687,386,858 | 2.79% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `alloc`: total | 3,534,085,014 | 6.18% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `alloc`: LocalKey/cell reads and writes | 474,163,697 | 0.83% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `alloc`: padding and layout construction | 641,247,761 | 1.12% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `alloc`: malloc/free, including aligned allocation | 507,925,599 | 0.89% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `alloc`: rest | 1,910,747,957 | 3.34% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `dealloc`: total | 3,382,377,030 | 5.92% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `dealloc`: LocalKey/cell reads and writes | 432,011,820 | 0.76% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `dealloc`: padding and layout construction | 696,301,397 | 1.22% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `dealloc`: malloc/free, including aligned allocation | 495,522,067 | 0.87% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | `dealloc`: rest | 1,758,541,746 | 3.08% |

## Decisions inside the shape

The following alternatives each ran T1 once, with exactly one passing test. They change only the allocator variant. Whole-test totals include startup; allocator-path totals provide the comparison.

| Variant | Total Ir | alloc inclusive Ir | dealloc inclusive Ir | allocated inclusive Ir | freed inclusive Ir |
|---|---:|---:|---:|---:|---:|
| Combined highs, per-field Cells, unchecked alloc layout plus explicit bound | 95,064,809 | 15,200,720 | 14,362,137 | 4,330,510 | 3,958,740 |
| Combined highs, per-field Cells, safe alloc layout (chosen) | 94,854,633 | 14,980,780 | 14,362,137 | 4,330,510 | 3,958,740 |
| Combined highs, one Cell over copied counters, safe alloc layout | 97,279,105 | 15,996,784 | 15,769,689 | 5,346,514 | 5,366,292 |
| Separate highs, per-field Cells, safe alloc layout | 94,851,044 | 14,980,780 | 14,362,137 | 4,330,510 | 3,958,740 |

- **Highs beside the counters.** Separate highs and combined highs have identical alloc/dealloc/allocated/freed instruction totals on T1. Their whole-test totals differ by 3,589 Ir (0.0038%), outside the allocator paths. Combining therefore costs nothing on the measured allocator paths outside a step and removes the second TLS reach while recording/handing highs. All 15 unchanged meter/runtime/span tests pass, including compaction, narrow peaks and ownership hand-outs. `Counts` is const-initialised, with a const no-drop assertion; `RUNTIME` retains its destructor.
- **A Cell per field.** The copied-counter variant adds 1,016,004 alloc Ir and 1,407,552 dealloc Ir on T1. Its whole-test total is 2.56% higher than per-field Cells. Keep the default per-field form.
- **Safe layout on alloc.** `Layout::from_size_align` costs 77 Ir/call, against 88 for `from_size_align_unchecked`, whose debug preconditions still run. The safe variant saves 219,940 alloc Ir on T1 (10 Ir/call in the complete path), so alloc keeps the safe constructor. `padded` computes size/offset without a checked-add round trip: Layout already bounds the sum below usize overflow. The safe constructor’s only condition not guaranteed by the incoming layout and power-of-two offset is that the padded size, rounded up, fits isize::MAX.
- **Unchecked layout on dealloc.** Allocation established the padded layout’s conditions. Deallocation reconstructs it inside the unsafe impl with a SAFETY comment, without a fallible branch or silent return. The unchecked constructor still costs 88 Ir/call in this unoptimized build; it removes the impossible error handling, not Rust’s debug UB checks. No package optimization override was added.

## What remains

These rows are subsets of the partition above, not additional costs to sum with it. Header access carries allocation numbers, required by the unchanged hand-out accounting. Layout construction still carries unoptimized library/precondition work that this change cannot remove: even unchecked construction executes UB preconditions. Required size conversion and call boundaries remain too. These readings leave those costs visible; build settings stay unoptimized.

| Package | Test | Arm | Total Ir | Cause | Cause’s inclusive Ir | Share |
|---|---|---|---:|---|---:|---:|
| `jig-core-world` | T1 | after | 94,853,537 | header write_unaligned (within rest) | 4,223,328 | 4.45% |
| `jig-core-world` | T1 | after | 94,853,537 | header read_unaligned (within rest) | 4,310,148 | 4.54% |
| `jig-core-world` | T1 | after | 94,853,537 | size conversion (within rest) | 2,155,363 | 2.27% |
| `jig-core-world` | T1 | after | 94,853,537 | dealloc unchecked layout (within padding) | 1,935,384 | 2.04% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | header write_unaligned (within rest) | 4,931,232 | 3.63% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | header read_unaligned (within rest) | 5,032,800 | 3.70% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | size conversion (within rest) | 2,516,689 | 1.85% |
| `jig-client-protocol-world` | T2 | after | 135,985,297 | dealloc unchecked layout (within padding) | 2,259,840 | 1.66% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | header write_unaligned (within rest) | 489,210 | 1.43% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | header read_unaligned (within rest) | 497,278 | 1.45% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | size conversion (within rest) | 249,165 | 0.73% |
| `skein-forge-machine-world` | T3 | after | 34,218,240 | dealloc unchecked layout (within padding) | 223,696 | 0.65% |
| `skein-llm-world` | T4 | after | 10,623,205 | header write_unaligned (within rest) | 228,584 | 2.15% |
| `skein-llm-world` | T4 | after | 10,623,205 | header read_unaligned (within rest) | 232,164 | 2.19% |
| `skein-llm-world` | T4 | after | 10,623,205 | size conversion (within rest) | 116,375 | 1.10% |
| `skein-llm-world` | T4 | after | 10,623,205 | dealloc unchecked layout (within padding) | 104,456 | 0.98% |
| `skein-tls-world` | T5 | after | 70,681,989 | header write_unaligned (within rest) | 193,438 | 0.27% |
| `skein-tls-world` | T5 | after | 70,681,989 | header read_unaligned (within rest) | 196,306 | 0.28% |
| `skein-tls-world` | T5 | after | 70,681,989 | size conversion (within rest) | 98,441 | 0.14% |
| `skein-tls-world` | T5 | after | 70,681,989 | dealloc unchecked layout (within padding) | 88,352 | 0.12% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | header write_unaligned (within rest) | 977,139,942 | 1.71% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | header read_unaligned (within rest) | 996,167,946 | 1.74% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | size conversion (within rest) | 498,416,240 | 0.87% |
| `jig-ops-world` | T6 | after | 57,142,896,007 | dealloc unchecked layout (within padding) | 447,259,296 | 0.78% |

## Refusal and invariants

The new refused test checks byte-vector requests at isize::MAX, MAX−7, MAX−8 and MAX−15, plus page-aligned requests at the largest original layout and the next smaller one. Every try_reserve_exact refuses and Meter::held remains zero. The largest valid padded requests reach the system allocator; the larger padded requests are refused before it.

The rule was broken by removing the explicit bound in the unchecked trial, and again in the chosen form by replacing its checked layout decision with unchecked construction. Both runs selected one test and failed by SIGABRT: the layout constructor reported that the rounded-up allocation size exceeded isize::MAX. Restoring the decision passed the same test. The final 16-test heap suite passes. Meter, runtime and span test files have no diff; Counting, Meter, Measured, Span and Grown keep their public signatures.

The final focused consumer check passed 1,072 tests across skein-world-tests, skein-oauth-accounts-world, skein-forge-machine-world, jig-store-world, smith-hosts-world and hearth-render-world. No count, peak or hand-out assertion changed. Clippy and the skein old-form ratchet pass. No slow tier ran.

The repeated T1 reading of the chosen code differs from its option trial by 1,096 Ir (0.0012%); allocator-path totals are identical.
