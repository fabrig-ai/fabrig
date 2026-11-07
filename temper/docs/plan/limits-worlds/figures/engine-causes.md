# engine-causes: allocator accounting shared by the worlds

2026-11-07 UTC; base `28a0a63aad`, with the allocator arm described below;
16 CPUs, AMD EPYC-Genoa Processor; Valgrind 3.24.0. Test binaries stay
unoptimized. `figures-before.md` supplies the ranked cause list.

## Method and stories

| Key | Package | Exact test in `suite` |
|---|---|---|
| C | `temper-conformance-world` | `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` |
| G | `temper-real-loop-world` | `github_land::a_chats_change_lands_on_connected_github` |

Every build, native run and Callgrind run used `heavy`, sequentially, with
`TEMPER_BUNDLE=none`. `cargo nextest list --test suite --message-format json`
located each binary and working directory. Each binary ran from its package
with `CARGO_MANIFEST_DIR` set, `--exact NAME --nocapture`, and reported
exactly `1 passed`. Native CPU is Bash `time`'s user plus system seconds,
including waited-for descendants; the profiler's user time is not native CPU.
G was built and run with the measuring note's local `PATIENCE = from_secs(3000)`;
no call-specific bound changed. Patience is restored before each commit.
C uses simulated time and needs no patience edit.

Each invocation writes one Callgrind file: Ir is its `PROGRAM TOTALS`, with
no fork sums. Both `callgrind_annotate --inclusive=yes --auto=no --threshold=100`
and `--inclusive=no` were read. Tables group identical function symbols across
inline file contexts. Inclusive rows overlap and must not be added. These
are focused/fuzzy stories; no browser, process or live tier ran.

## Before and after the allocator change

| Story | Arm | Run | Ir | Native CPU s (user + system, descendants) | Native wall s | Callgrind wall s | Result |
|---|---|---:|---:|---:|---:|---:|---|
| C | before | 1 | 41,386,232,328 | 11.545 | 12.204 | 340.599 | 1 passed |
| C | heap | 1 | 37,822,070,501 | 9.535 | 9.537 | 305.871 | 1 passed |
| G | before | 1 | 14,902,421,987 | 3.315 | 3.296 | 147.046 | 1 passed |
| G | before | 2 | 15,299,060,625 | 3.315 | 3.296 | 155.268 | 1 passed |
| G | heap | 1 | 14,041,380,369 | 3.376 | 3.654 | 138.659 | 1 passed |
| G | heap | 2 | 14,794,365,686 | 3.376 | 3.654 | 133.980 | 1 passed |

C saves **3,564,161,827 Ir (8.61%)**. It still checks 81 durable commits,
504 baseline iterations and 79 baseline/live-cut story runs. Its local
before differs from the supplied baseline by only 2,206 Ir (0.000006%).
G's before spread is 2.66% and its heap spread is 5.36%, measured against
the smaller count in each arm. The after spread exceeds the
median reduction: **no end-to-end G gain is established**. Its native CPU also does
not show a gain. The shared cause is established by C and the bookkeeping
paths below, not by choosing G's fastest run.

C has no native or hosted process spawns. Every G profile has 45 hosted
starts and no native `skein_shell::linux::process::spawn` call. There are no
unmeasured native shell instructions on these paths. G's wall/native-wall
ratios are 44.61 / 47.11 before and 37.95 / 36.67 after. We do not rank
real-clock polls from their instrumented raw shares: the timer corrections
in `figures-before.md` remain the screen for those separate causes.

## The fixed cause

One const-initialized, destructor-free thread-local `Counts` groups the
live bytes, peak, allocation sequence and phase. Each allocation/free
reaches it once, then accesses its cells directly. High-water records keep
their separate bounded store. Allocation layout, numbering, signed arithmetic,
handoff accounting, peaks and public API keep their meanings. No cache,
new retained payload or memory ceiling is introduced.

| Story / arm | Allocations | Frees | `allocated` inclusive Ir | `freed` inclusive Ir | `padded` inclusive Ir |
|---|---:|---:|---:|---:|---:|
| C-before | 8,152,067 | 8,152,066 | 3,947,360,116 | 2,673,877,648 | 3,929,295,579 |
| C-heap | 8,152,067 | 8,152,066 | 1,590,012,185 | 1,467,371,880 | 3,929,295,579 |
| G-before-1 | 1,299,201 | 1,299,199 | 629,874,232 | 426,137,272 | 626,214,374 |
| G-before-2 | 1,274,115 | 1,274,113 | 617,732,608 | 417,909,064 | 614,122,922 |
| G-heap-1 | 1,273,314 | 1,273,312 | 248,512,710 | 229,196,160 | 613,736,840 |
| G-heap-2 | 1,283,503 | 1,283,501 | 250,499,565 | 231,030,180 | 618,647,938 |

C allocates and frees the same numbers of blocks in both arms. Its
bookkeeping saving is 3,563,853,699 Ir; the remaining 308,128 Ir saving
comes from meter/span access. The header calculation is unchanged,
3,929,295,579 Ir (9.49% of before C): checking and deriving a System layout
for the allocation header is required work, left as the task's hot-leaf
exception. No build override is requested.

The new `skein-heap` test
`tests::shared_counters_keep_handoffs_and_signed_spans_exact` passes,
as do all 15 `skein-heap-tests` focused tests. Removing the peak update
made the new test fail with `(0, 80)` instead of `(192, 80)`; restoring it
made the test pass. Thus the test holds peak accounting, not just final
live bytes. It also holds handoff subtraction and a negative span release.

## C-before: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.94% | 41,360,124,810 | 79 | `temper_conformance_world::stories::run` |
| 85.34% | 35,320,500,135 | 4,615 | `jig_conformance::harness::Harness<A>::drain` |
| 85.34% | 35,320,449,370 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from` |
| 85.34% | 35,318,875,655 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from::{{closure}}` |
| 85.34% | 35,318,811,045 | 4,615 | `jig_conformance::harness::Harness<A>::drain_iterations` |
| 85.31% | 35,307,745,684 | 43,157 | `jig_conformance::harness::Harness<A>::iterate` |
| 65.80% | 27,231,996,797 | 79 | `temper_conformance_world::stories::positive_change` |
| 44.59% | 18,453,085,444 | 155,975 | `jig_conformance::harness::Harness<A>::inspect` |

## C-before: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 6.07% | 2,510,458,176 | `__memcpy_avx_unaligned_erms` |
| 4.18% | 1,731,914,448 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 3.57% | 1,475,748,577 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 3.39% | 1,403,951,090 | `std::thread::local::LocalKey<T>::try_with` |
| 2.96% | 1,226,420,472 | `core::ops::function::FnOnce::call_once'2` |
| 2.32% | 961,943,847 | `skein_heap::padded` |
| 2.04% | 844,834,528 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.01% | 831,816,052 | `std::thread::local::LocalKey<T>::initialize_with` |

## C-heap: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.94% | 37,797,540,027 | 79 | `temper_conformance_world::stories::run` |
| 85.50% | 32,338,536,461 | 4,615 | `jig_conformance::harness::Harness<A>::drain` |
| 85.50% | 32,338,485,696 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from` |
| 85.50% | 32,336,911,981 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from::{{closure}}` |
| 85.50% | 32,336,847,371 | 4,615 | `jig_conformance::harness::Harness<A>::drain_iterations` |
| 85.47% | 32,326,970,378 | 43,157 | `jig_conformance::harness::Harness<A>::iterate` |
| 65.77% | 24,875,712,323 | 79 | `temper_conformance_world::stories::positive_change` |
| 45.37% | 17,160,127,972 | 155,975 | `jig_conformance::harness::Harness<A>::inspect` |

## C-heap: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 6.64% | 2,510,543,486 | `__memcpy_avx_unaligned_erms` |
| 4.58% | 1,731,914,448 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 3.90% | 1,475,748,577 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 2.54% | 961,943,847 | `skein_heap::padded` |
| 2.23% | 844,834,528 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.08% | 786,317,166 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` |
| 2.03% | 766,294,251 | `core::alloc::layout::Layout::from_size_align` |
| 1.97% | 744,328,260 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |

## G-before-1: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.08% | 14,765,921,427 | 1 | `temper_real_loop_world::run_github` |
| 99.08% | 14,765,899,545 | 1 | `temper_real_loop_world::world::run_github` |
| 94.60% | 14,097,699,283 | 1 | `skein_world::real::World<P,R>::run` |
| 94.58% | 14,095,115,242 | 1 | `skein_world::real::Running<P,R>::run` |
| 85.81% | 12,787,621,639 | 9,091 | `skein_world::real::Running<P,R>::drive` |
| 78.60% | 11,713,231,771 | 48,702 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 22.66% | 3,376,917,744 | 8,558 | `temper_service::system::iterate` |
| 22.63% | 3,371,740,154 | 42,790 | `temper_service::system::iterate_stage` |

## G-before-1: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.41% | 657,696,944 | `__memcpy_avx_unaligned_erms` |
| 2.84% | 423,651,938 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.34% | 349,421,400 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.81% | 270,348,440 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.79% | 267,261,921 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.74% | 259,011,433 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.66% | 247,452,118 | `skein_json::writer::escaped_len` |
| 1.57% | 233,395,334 | `core::ptr::copy_nonoverlapping::precondition_check` |

## G-before-2: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.13% | 15,165,650,918 | 1 | `temper_real_loop_world::run_github` |
| 99.13% | 15,165,629,036 | 1 | `temper_real_loop_world::world::run_github` |
| 94.76% | 14,497,428,774 | 1 | `skein_world::real::World<P,R>::run` |
| 94.74% | 14,494,844,733 | 1 | `skein_world::real::Running<P,R>::run` |
| 84.04% | 12,857,362,263 | 8,854 | `skein_world::real::Running<P,R>::drive` |
| 77.13% | 11,799,525,842 | 47,417 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 21.17% | 3,239,461,472 | 8,351 | `temper_service::system::iterate` |
| 21.14% | 3,234,409,117 | 41,755 | `temper_service::system::iterate_stage` |

## G-before-2: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.14% | 633,161,662 | `__memcpy_avx_unaligned_erms` |
| 2.81% | 429,709,158 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.72% | 416,125,483 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.45% | 374,955,111 | `core::iter::traits::iterator::Iterator::try_fold'2` |
| 2.24% | 342,512,620 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.75% | 267,036,861 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.70% | 260,580,235 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.61% | 245,893,014 | `skein_json::writer::escaped_len` |

## G-heap-1: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.08% | 13,912,855,042 | 1 | `temper_real_loop_world::run_github` |
| 99.08% | 13,912,836,808 | 1 | `temper_real_loop_world::world::run_github` |
| 94.60% | 13,283,232,193 | 1 | `skein_world::real::World<P,R>::run` |
| 94.58% | 13,280,657,054 | 1 | `skein_world::real::Running<P,R>::run` |
| 85.38% | 11,988,233,380 | 8,825 | `skein_world::real::Running<P,R>::drive` |
| 78.07% | 10,962,333,764 | 47,379 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 21.86% | 3,068,785,438 | 8,330 | `temper_service::system::iterate` |
| 21.82% | 3,063,745,788 | 41,650 | `temper_service::system::iterate_stage` |

## G-heap-1: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.51% | 633,380,053 | `__memcpy_avx_unaligned_erms` |
| 2.95% | 414,805,640 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.44% | 342,158,460 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.90% | 266,886,856 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.90% | 266,090,486 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.85% | 259,253,559 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.75% | 245,847,968 | `skein_json::writer::escaped_len` |
| 1.63% | 229,245,754 | `core::ptr::copy_nonoverlapping::precondition_check` |

## G-heap-2: the largest by inclusive Ir

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 99.13% | 14,665,744,863 | 1 | `temper_real_loop_world::run_github` |
| 99.13% | 14,665,726,606 | 1 | `temper_real_loop_world::world::run_github` |
| 94.87% | 14,036,098,705 | 1 | `skein_world::real::World<P,R>::run` |
| 94.86% | 14,033,523,711 | 1 | `skein_world::real::Running<P,R>::run` |
| 83.87% | 12,407,724,521 | 8,915 | `skein_world::real::Running<P,R>::drive` |
| 76.90% | 11,376,172,122 | 47,819 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 20.84% | 3,083,648,663 | 8,387 | `temper_service::system::iterate` |
| 20.81% | 3,078,574,528 | 41,935 | `temper_service::system::iterate_stage` |

## G-heap-2: the largest by self Ir

| Share | Ir | Function |
|---:|---:|---|
| 4.30% | 635,966,570 | `__memcpy_avx_unaligned_erms` |
| 2.87% | 425,272,617 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.82% | 417,398,144 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.51% | 370,978,460 | `core::iter::traits::iterator::Iterator::try_fold'2` |
| 2.33% | 344,072,786 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.81% | 268,057,642 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.76% | 260,925,190 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.67% | 246,765,228 | `skein_json::writer::escaped_len` |

## Causes, largest first

Footprints below are the supplied ranked list's shares, not promised savings.
R and P are partial unsuccessful instrumented paths; C, G and U completed.

| Cause | Where | Share of each story | Fixed by this task, or whose it is |
|---|---|---|---|
| Allocation bookkeeping and header layout | `skein/testing/skein-heap/src/lib.rs` | C 25.49%; R 14.27 / 14.22%; P 12.30 / 10.96%; G 11.44 / 10.79%; U 0% | Counter lookup fixed here; required header-layout leaf remains, 9.49% of C. |
| Reparse schema/request/configuration JSON | `skein-json::parse`, `string`; `skein-llm::Json::from_bytes` | R 12.85 / 13.12%; P 9.81 / 6.03%; G 10.03 / 9.75%; C 0%; U 10.15% | Left: the parser must validate independent inputs. Avoiding schema reconstruction/reparsing needs a change to producer representation or bounded retained schema state keyed by the varying charter, namespace and grants, beyond the cause's module and its callers. |
| Validate and count strings again on the writing pass | `skein-json::writer` | R 11.09 / 10.52%; P 7.53 / 4.15%; G 6.81 / 7.03%; C 0%; U 9.37% | Next candidate: measure a single validating escaping pass before deciding. |
| Repeated whole-prompt measurement | `smith-protocol-llm::component::observed_prompt` and `skein-llm::client` | R 7.35 / 6.20%; P 3.98 / 2.03%; G 3.70 / 4.26%; C 0%; U 5.80% | Left: preserving refusal before observation needs an admitted measurement transferred through the prompt-observation and connection-reservation boundaries; removing the early check changes behaviour, and that protocol change is beyond this local cause repair. |
| Derive store-family bounds again | `jig-protocol-store::codec::limits` | C 2.17%; R 0.16 / 0.14%; P 0.23 / 0.18%; G 0.29 / 0.25%; U 0.15% | Left pending the next candidate's stopping screen; production Store already retains its codec view. C's incoming costs split into independently constructed `Limits::codec_view` (1.50%) and `sizes::largest` / `sizes::frame` (0.33% each). |

## Left

- U's durable-row scan (19.86%) belongs to `public-causes`.
- C's referee snapshots/naming (16.09%), policy comparisons (12.30%) and live-cut fixture setup (7.91%) belong to `conformance-causes` / `cuts-fork`.
- P's oracle cue scans (4.50%) and deployment-default rebuilds (3.07 / 2.50%) belong to `real-loop-causes`.
- The supplied partial R/P profiles' retained forge-observation assertion and ending oracle assertion remain diagnostic only; this task compares completed C/G stories and changes neither fixture observation caps nor oracle expectations.

Next candidate: writing-pass string recount, U 9.37% (R's partial footprint 11.09 / 10.52%); decision awaits its instruction measurement.
