# real-loop-causes: forge history and byte searches

2026-11-07 UTC, baseline `1bb7c04a2a`, 16 CPUs, AMD EPYC-Genoa Processor,
Valgrind 3.24.0. Unoptimized test binaries, including dependencies.
`stories-go` is already in this baseline; all three requested stories remain.
The after arm combines forge-history reuse (`a01fc901c6`), terminal byte search
(`2c4b68aa95`) and the same byte search in case cue classification
(`d0754a7a00`).
Repair after/run 1 was captured at the collector step: terminal matching
and case cue classification have zero calls in that story. GitHub uses no
case oracle, so its final terminal samples also represent the combined path.
The two final case samples and whole-world native suites use all three fixes.

## Method

The procedure is `temper/docs/plan/limits-worlds/notes/measuring.md`.
GNU time is absent. The answer in
`temper/docs/plan/limits-worlds/tasks/figures-before.md`, Answers, permits
Python's `resource.getrusage(resource.RUSAGE_CHILDREN)` instead, for this task
too. Each native command is waited for; its user and system CPU include its
waited descendants. Wall seconds use `time.monotonic()`.

The measurement wrapper, inside `heavy`, is:

```python
import resource, subprocess, sys, time
started = time.monotonic()
code = subprocess.run(sys.argv[1:]).returncode
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
print("cpu", usage.ru_utime + usage.ru_stime,
      "user", usage.ru_utime, "system", usage.ru_stime,
      "wall", time.monotonic() - started, file=sys.stderr)
sys.exit(code)
```

For each arm, build and locate the suite with
`heavy cargo nextest list -p temper-real-loop-world --test suite --message-format json`.
Read its `binary-path` and `cwd`. Run that binary from `cwd`, setting
`CARGO_MANIFEST_DIR` to `cwd`, with `--exact NAME --nocapture`, through the
wrapper above. Native runs use the shipped 20-second patience.

For Callgrind alone, locally change `PATIENCE` in
`temper/tests/real-loop/src/lib.rs` to 3000 seconds and rebuild. Add
`valgrind --tool=callgrind --callgrind-out-file=OUTPUT.%p.out` before the
binary. No call-specific bound is changed. Run two samples per arm because
real-time scheduling changes the counts. Read each completed profile with
`callgrind_annotate --auto=no --inclusive=yes --threshold=100` and
`--inclusive=no`. The figure is its `PROGRAM TOTALS` Ir; these stories fork
no sweep children. Native CPU is measured separately from Callgrind CPU.

Restore the shipped patience for native validation and before committing.
Whole-world native measurements wrap warm `cargo nextest run -p
temper-real-loop-world`, with and without `--profile fuzzy`, inside `heavy`.
Those totals include cargo/nextest's warm driver overhead, consistently in
both arms; the first focused run rebuilt a binary and is excluded.

Partial profiles are retained under the measuring procedure's explicit
failure rule. Their instruction totals describe only the executed prefix,
not a completed story, and cannot establish a whole-story saving. The two
required samples measure this variability; they are not retries looking
for a passing verdict. No observation capacity is increased.

## Measurements

- `repair`: `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands`.
- `case`: `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4`.
- `github`: `github_land::a_chats_change_lands_on_connected_github`.

| Story | Arm | Run | Ir | Native CPU s (user + system, descendants) | Result |
|---|---|---:|---:|---:|---|
| `repair` | before | 1 | 26,131,909,456 | 9.540774 | Partial: frozen record count fits |
| `repair` | before | 2 | 28,677,321,021 | — | Partial: frozen record count fits |
| `repair` | after | 1 | 26,812,262,485 | 8.427590 | Partial: frozen record count fits |
| `repair` | after | 2 | 28,775,082,887 | — | Partial: frozen record count fits |
| `case` | before | 1 | 38,838,658,387 | 7.132480 | Partial: frozen record count fits |
| `case` | before | 2 | 38,632,038,555 | — | Partial: frozen record count fits |
| `case` | after | 1 | 25,606,221,778 | 6.322022 | 1 passed |
| `case` | after | 2 | 26,799,220,570 | — | Partial: see failure account |
| `github` | before | 1 | 15,391,832,129 | 3.277663 | 1 passed |
| `github` | before | 2 | 14,605,878,257 | — | 1 passed |
| `github` | after | 1 | 14,156,815,225 | 3.169261 | 1 passed |
| `github` | after | 2 | 13,727,034,832 | — | 1 passed |

Native CPU appears once per arm; both instruction samples are separate executions.
All native single-story runs selected exactly one test and passed.

## Profiling slowdown, polls and external children

`operation` below is `skein_forge_client::keep::poll::operation`.
Estimated native calls divide its instrumented calls by Callgrind wall/native wall,
as the measuring procedure requires. These estimates keep timer inflation out of
the ranking; neither polling intervals nor patience are shortened.
The collector runs on every judge pass, including io-driven passes, rather than
on this poll timer, so its entire count is not divided by the slowdown.

| Story | Arm/run | Callgrind wall s | Native wall s | Slowdown | Poll calls | Scaled calls | External spawns |
|---|---|---:|---:|---:|---:|---:|---:|
| `repair` | before/1 | 233.527 | 9.463 | 24.68 | 1,181 | 47.9 | 3 |
| `repair` | before/2 | 259.145 | 9.463 | 27.39 | 1,407 | 51.4 | 4 |
| `repair` | after/1 | 239.540 | 8.629 | 27.76 | 1,430 | 51.5 | 4 |
| `repair` | after/2 | 253.051 | 8.629 | 29.33 | 1,469 | 50.1 | 4 |
| `case` | before/1 | 366.654 | 7.996 | 45.86 | 3,750 | 81.8 | 0 |
| `case` | before/2 | 372.667 | 7.996 | 46.61 | 3,744 | 80.3 | 0 |
| `case` | after/1 | 212.771 | 8.574 | 24.82 | 1,359 | 54.8 | 0 |
| `case` | after/2 | 237.363 | 8.574 | 27.68 | 1,383 | 50.0 | 0 |
| `github` | before/1 | 148.115 | 3.249 | 45.59 | 383 | 8.4 | 0 |
| `github` | before/2 | 146.977 | 3.249 | 45.24 | 378 | 8.4 | 0 |
| `github` | after/1 | 157.280 | 3.185 | 49.38 | 395 | 8.0 | 0 |
| `github` | after/2 | 140.214 | 3.185 | 44.02 | 376 | 8.5 | 0 |

External spawns in these profiles come from Smith's `process::list`, running
`/usr/bin/rg --files` with the tool's selection and ignore rules. The `shell`,
`search` and `check` spawn paths have zero calls. Thus there are no observed
`sh` spawns to replace. Counts on partial profiles are counts of that prefix
alone. The fake git program stays hosted in-process. Native CPU includes the
external listing processes; Callgrind's Ir does not.

## Collector cost

Inclusive collector Ir includes its descendants. Prefixes can differ in how far
the story progressed; whole-story totals from partial runs are not comparable
savings. The collector's cost and read counts directly measure the removed work.

| Story | Arm/run | Collector calls | Collector Ir | Ir/call | Share | Open-pull read calls |
|---|---|---:|---:|---:|---:|---:|
| `repair` | before/1 | 11,893 | 833,682,304 | 70098.6 | 3.190% | 11,893 |
| `repair` | before/2 | 12,353 | 1,137,022,812 | 92044.3 | 3.965% | 12,353 |
| `repair` | after/1 | 12,781 | 6,401,309 | 500.8 | 0.024% | 20 |
| `repair` | after/2 | 12,876 | 5,874,866 | 456.3 | 0.020% | 14 |
| `case` | before/1 | 24,835 | 4,625,233,683 | 186238.5 | 11.909% | 24,835 |
| `case` | before/2 | 24,623 | 4,581,294,917 | 186057.5 | 11.859% | 24,623 |
| `case` | after/1 | 13,912 | 6,693,892 | 481.2 | 0.026% | 17 |
| `case` | after/2 | 13,802 | 6,763,495 | 490.0 | 0.025% | 17 |

The initial snapshot is always read. Subsequent reads happen after draining at
least one new forge observation, and the existing equality check still records
only a changed branch/pull snapshot. CI and metadata observations conservatively
trigger a read too. Domain observations, merged checks and the frozen main tree
are retained. Delivery collection consumes only the newly appended peer suffix.

The snapshot regression covers the seed, branch creation, pull opening, closing
and reopening, unchanged state, and the original change timestamp. Disabling
observation invalidation loses every subsequent branch/pull snapshot (only the seed remains).
The delivery regression covers extension and repeated unchanged history; removing
the cursor advance fails with `[Written, Written]` instead of `[Written]`.
Both faults were restored and both tests passed.

## `repair`: largest relevant inclusive Ir (before, run 1)

Runtime and dispatch ancestors are omitted; inclusive rows overlap.

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 11.92% | 3,113,797,904 | 2,936,545 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` |
| 10.25% | 2,678,942,911 | 2,936,517 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` |
| 9.12% | 2,382,499,895 | 959,318 | `skein_json::writer::escaped_len` |
| 5.44% | 1,422,311,880 | 2,936,545 | `skein_heap::allocated` |
| 3.19% | 833,682,304 | 11,893 | `temper_real_loop_world::forge::Seen::collect` |
| 1.09% | 283,673,736 | 31 | `jig_protocol_config::document::document` |
| 0.88% | 230,284,391 | 5 | `temper_real_loop_world::fixtures::own_defaults` |
| 0.76% | 198,140,997 | 1 | `temper_real_loop_world::terminal::operations` |
| 0.73% | 190,021,728 | 1 | `temper_real_loop_world::terminal::compose_with_reading` |
| 0.72% | 188,720,394 | 10,266,095 | `temper_real_loop_world::forge::Seen::collect::{{closure}}` |
| 0.58% | 150,949,093 | 3 | `temper_real_loop_world::fixtures::given` |
| 0.53% | 138,065,522 | 2 | `temper_real_loop_world::landing::admit` |

## `repair`: largest self Ir (before, run 1)

| Share | Ir | Function |
|---:|---:|---|
| 3.08% | 804,689,492 | `__memcpy_avx_unaligned_erms` |
| 2.91% | 759,578,632 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.76% | 719,936,227 | `skein_json::writer::escaped_len` |
| 2.57% | 670,767,911 | `skein_json::writer::escaped` |
| 2.41% | 628,547,582 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.71% | 446,000,459 | `core::ops::function::FnOnce::call_once'2` |
| 1.54% | 402,458,116 | `std::thread::local::LocalKey<T>::try_with` |
| 1.43% | 374,930,491 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.34% | 349,371,433 | `core::result::Result<T,E>::expect` |
| 1.33% | 346,510,658 | `skein_heap::padded` |

## `case`: largest relevant inclusive Ir (before, run 1)

Runtime and dispatch ancestors are omitted; inclusive rows overlap.

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 11.91% | 4,625,233,683 | 24,835 | `temper_real_loop_world::forge::Seen::collect` |
| 9.07% | 3,522,170,392 | 3,320,604 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` |
| 7.81% | 3,031,495,455 | 3,320,581 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` |
| 7.10% | 2,758,756,842 | 24,835 | `skein_fake_forge_domain::reads::open_pulls` |
| 4.14% | 1,608,391,652 | 3,320,604 | `skein_heap::allocated` |
| 3.37% | 1,307,983,563 | 628,282 | `skein_json::writer::escaped_len` |
| 2.54% | 984,889,128 | 21 | `temper_real_loop_world::fixtures::own_defaults` |
| 2.17% | 844,061,556 | 12 | `temper_real_loop_world::landing::admit` |
| 2.14% | 829,693,834 | 12 | `temper_real_loop_world::landing::limits` |
| 1.68% | 654,271,661 | 81 | `jig_protocol_config::document::document` |
| 1.20% | 466,226,665 | 22,091,722 | `temper_real_loop_world::forge::Seen::collect::{{closure}}` |
| 1.19% | 460,279,957 | 9 | `temper_real_loop_world::fixtures::given` |

## `case`: largest self Ir (before, run 1)

| Share | Ir | Function |
|---:|---:|---|
| 4.55% | 1,765,546,313 | `__memcpy_avx_unaligned_erms` |
| 2.69% | 1,044,536,332 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.63% | 634,239,734 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.59% | 617,807,445 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::find_map` |
| 1.46% | 567,418,595 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |
| 1.46% | 566,264,333 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.33% | 516,444,372 | `core::ops::function::FnOnce::call_once'2` |
| 1.17% | 455,102,719 | `std::thread::local::LocalKey<T>::try_with` |
| 1.07% | 416,233,424 | `core::option::Option<T>::expect` |
| 1.02% | 396,673,185 | `skein_json::writer::escaped_len` |

## `github`: largest relevant inclusive Ir (before, run 1)

Runtime and dispatch ancestors are omitted; inclusive rows overlap.

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 11.81% | 1,818,295,711 | 16,789 | `temper_real_loop_world::terminal::contains` |
| 8.83% | 1,359,280,781 | 1,269,748 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` |
| 7.57% | 1,164,801,885 | 1,269,746 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::dealloc` |
| 4.83% | 743,400,183 | 355,343 | `skein_json::writer::escaped_len` |
| 4.70% | 723,191,859 | 9,337,621 | `temper_real_loop_world::terminal::contains::{{closure}}` |
| 4.00% | 615,618,980 | 1,269,748 | `skein_heap::allocated` |
| 3.70% | 570,112,990 | 3 | `temper_real_loop_world::terminal::compose_with_reading` |
| 1.73% | 266,866,673 | 28 | `jig_protocol_config::document::document` |
| 1.48% | 227,727,964 | 5 | `temper_real_loop_world::fixtures::own_defaults` |
| 1.29% | 198,134,227 | 1 | `temper_real_loop_world::terminal::operations` |
| 0.97% | 148,918,453 | 3 | `temper_real_loop_world::fixtures::given` |
| 0.94% | 145,077,479 | 2 | `temper_real_loop_world::landing::fixture::{{closure}}` |

## `github`: largest self Ir (before, run 1)

| Share | Ir | Function |
|---:|---:|---|
| 4.14% | 637,006,709 | `__memcpy_avx_unaligned_erms` |
| 2.55% | 392,894,475 | `core::iter::traits::iterator::Iterator::try_fold'2` |
| 2.31% | 355,163,664 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.87% | 287,201,234 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.71% | 263,290,958 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.46% | 225,411,858 | `skein_json::writer::escaped_len` |
| 1.43% | 220,553,486 | `__memcmp_avx2_movbe` |
| 1.39% | 214,260,969 | `skein_json::writer::escaped` |
| 1.26% | 194,389,577 | `core::ops::function::FnOnce::call_once'2` |
| 1.23% | 189,645,021 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |

## Terminal matching cost

The collector-only GitHub samples keep the byte-window search and are
intermediate measurements. The final arm uses Skein's linear byte search.
The per-call comparison separates the algorithm from changing readiness-pass
counts. Empty-needle and byte-exact presence/absence semantics are unchanged;
the existing fresh-page test and all terminal stories pass.

| Stage | Run | Matching Ir | Share | Calls | Ir/call |
|---|---:|---:|---:|---:|---:|
| before | 1 | 1,818,295,711 | 11.81% | 16,789 | 108,302.8 |
| before | 2 | 1,121,880,763 | 7.68% | 10,582 | 106,017.8 |
| collector | 1 | 1,754,768,939 | 9.45% | 16,260 | 107,919.4 |
| collector | 2 | 1,116,655,635 | 6.71% | 10,589 | 105,454.3 |
| after | 1 | 261,862,313 | 1.85% | 9,531 | 27,474.8 |
| after | 2 | 286,108,646 | 2.08% | 10,380 | 27,563.5 |

## Case cue classification cost

The refreshed baseline figure (`067df0c8d9`) identifies `cued` at 4.498%
in the case sample that reached the ending oracle. This task's second
intermediate case sample measures the same cause at 4.04%, over 14 queries.
The original pair stopped before this path; zero means it was not reached.
The search still selects the earliest cue, retaining the enumerated priority
for ties. The regression also checks an opening cue and a system without cues.
Changing the comparison to keep the first enumerated match fails with
`Director` instead of `A`; the restored comparison passes.

| Stage | Run | Cue-search Ir | Share | Calls | Ir/call |
|---|---:|---:|---:|---:|---:|
| collector | 1 | 0 | 0.000% | 0 | — |
| collector | 2 | 1,134,762,687 | 4.037% | 14 | 81,054,477.6 |
| after | 1 | 220,243,602 | 0.860% | 13 | 16,941,815.5 |
| after | 2 | 220,314,688 | 0.822% | 13 | 16,947,283.7 |

## Whole-world native CPU

Warm cargo/nextest commands, user plus system including waited descendants.

| Profile | Arm | Passed tests | CPU s |
|---|---|---:|---:|
| default | before | 215 | 36.421265 |
| default | after | 218 | 33.799186 |
| fuzzy | before | 18 | 80.717788 |
| fuzzy | after | 18 | 74.607543 |

The focused name comparison loses none of the baseline 215 tests and adds only
the two forge-history regressions and the cue-selection regression. The fuzzy
name comparison is exactly the
same 18 names. These CPU totals are observations, not a claim that a difference
smaller than run-to-run variability is a saving.

## Causes, largest first

Shares below are inclusive Ir from each first baseline sample, in the order
repair / case 6.4 / GitHub, except for the cue row, whose ending oracle was
not reached in that baseline pair. Shared allocation and encoding rows overlap their
callers; they are not additive savings.

| Cause | Where | Share of each story | Decision |
|---|---|---|---|
| Allocation and accounting | `skein/testing/skein-heap/src/lib.rs`, `Counting::alloc` | 11.92% / 9.07% / 8.83% | `engine-causes`; includes the allocator, not all avoidable |
| Repeated forge-history collection | `temper/tests/real-loop/src/forge.rs`, `Seen::collect` | 3.19% / 11.91% / 0% | Fixed here: observe changes and append the new peer suffix |
| Terminal page matching | `temper/tests/real-loop/src/terminal.rs`, `contains` | 0% / 0% / 11.81% | Fixed here: shared linear byte search leaves 1.85–2.08%, with about 74% fewer instructions per query |
| JSON string-length pass | `skein/crates/skein-json/src/writer.rs`, `escaped_len` | 9.12% / 3.37% / 4.83% | `engine-causes` |
| Repeated cue searches at the ending oracle | `temper/tests/real-loop/src/case/oracle.rs`, `cued` | 0% / 4.04% in the intermediate sample that reached the oracle / 0% | Fixed here: shared byte search leaves 0.82–0.86% |
| Typed defaults and their configuration parsing | `temper/tests/real-loop/src/fixtures.rs`, `own_defaults` | 0.88% / 2.54% / 1.48% | The largest descendant is configuration parsing; keep each admission check |
| Admission of intermediate fixtures | `temper/tests/real-loop/src/landing.rs`, `admit` | 0.53% / 2.17% / 0.89% | Keep all twelve stage/host checks; their repeated configuration decoding is below the world, with `engine-causes` |
| Rewriting cluster documents | `temper/tests/real-loop/src/cluster.rs`, `Shape::write` | 0% / 0.09% / 0% | Below the stopping guide |

The fix adds one `usize` cursor to `Seen`. Unchanged passes retain their
snapshots, domain records, merged checks and main tree, while avoiding the
branch copies, open-pull inspection and repeated peer-history scan. The
collector measurements above show the saving at that cause. Partial program
totals and the native timing pairs do not establish a whole-story instruction
saving.

## Instrumentation limits

The baseline repair and case samples filled the forge peer's frozen record
table at `skein/crates/skein-fake-peers/src/forge/observe.rs` (`frozen record
count fits`). Both after repair samples reached that limit too.

The first collector-only case 6.4 sample instead reached the profiling judge's
2999-second diagnostic boundary, before the 3000-second world patience.
It was waiting at story step 14, call 10, for `task list --project 1 --json`
to contain `Repair landing branch CI`, after 1497 attempts. Four LLM queries
were outstanding, three tasks had ended, and the waiting goals were held.
Its loop thread used 2979.652 CPU seconds and waited runnable 18.411 seconds;
it was not a starved verdict. The complete failure log was read, with the
1496 identical successful hosted-call summaries collapsed for inspection.
The resulting 358,562,503,769 Ir is a partial prefix. The second intermediate
case sample reached the ending oracle, failing the complete-journal check for
task 1 at `temper/tests/real-loop/src/case/oracle.rs`. It collected
28,106,836,347 Ir. A native run alone in the fuzzy profile passed (one test,
9.410 seconds). The refreshed baseline figure also records this oracle
failure under Callgrind. These are diagnostic prefixes, not successful stories.

This sample's timer effects are large (using that stage's native wall of
8.912 seconds): 26,806 forge poll operations divided
by its 339.13-fold slowdown are about 79 native-equivalent calls, versus
about 82 in the first baseline case sample. Its 1497 condition-read attempts
scale to about 4.4. The late retained-host scans therefore do not justify
shortening the retry or changing the story.

The first final case sample passed under Callgrind; the second failed the
same ending-attempt promise for task 2 (`temper/tests/real-loop/src/case/oracle.rs:538`), collecting
26,799,220,570 Ir, marked partial. A native run alone in the fuzzy profile
then passed (one test, 8.662 seconds). Both ran on the 16-CPU EPYC-Genoa
machine named above. The failure and confirmation logs read:

```text
Callgrind: 6.4 Add "every attempt ended, none failed": task 2's complete journal ends every admitted attempt and permits only bounded placement rechecks
Callgrind: 0 passed; 1 failed; finished in 231.58s
Native fuzzy: PASS [8.662s] fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4
Native fuzzy: 1 test run: 1 passed, 44 skipped
```

The same case passes natively both alone and in the full fuzzy run. No call
bound, fixture timing, observation capacity, deadline or budget is changed.

## Left

- `skein/testing/skein-heap/src/lib.rs`,
  `skein/crates/skein-json/src/writer.rs` and the configuration codecs below
  the world: the shared costs in the cause table belong to `engine-causes`.
- `skein/crates/skein-fake-peers/src/forge/observe.rs` and the case's
  instrumented wait: partial profiling limits, not completed-story figures.
  Their bounds stay as supplied. The first final case profile completes
  successfully after the cue-search fix; earlier partial runs are retained
  as diagnostic evidence.
- The focused tree budget is still open. The real-loop command measured
  36.421265 CPU s before and 33.799186 after all three fixes. A final
  focused command that rebuilt after a different Cargo feature combination
  used 37.131810 CPU s and is excluded from the warm pair. The two-fix warm
  intermediate run was 33.975600 CPU s. The direct dependent `temper`
  passes 491 focused tests: 51.733666 CPU s on the
  pristine runtime base and 70.042554 on the final changed tree (both warm).
  Its earlier two-fix warm run used 56.914863 CPU s. Both are above
  the tree's 30-second budget; these warm-command totals include driver
  overhead and the native variation is substantial. Shared product and
  fixture costs outside this task remain with `engine-causes` and the
  other plans named by the request.

## Stopped at

Cue classification is 0.82–0.86% in the two final samples. Its average
query costs 16.94–16.95 million Ir versus 81.05 million in the intermediate
sample, about 79% less for the observed query mix; the final counts are 13
and 13, versus 14 before.

Terminal matching is now about 2% inclusive (1.85–2.08% across the two
samples), with 27,475–27,563 Ir per query versus 105,000–108,000 before.
New-page checks still need that shared search; eliminating even the whole
remaining function would save only about 2% of this story.

The next setup candidates are smaller. Terminal composition ran three
times: one admission count and two actual opens, including spent-link
recovery. Removing only the preflight is about 1.2% of the baseline story.
The case's twelve admissions check three successive configurations on
each of four hosts. They cost 2.17% in the baseline prefix and 3.30% in the
first completed after run; those are footprints, not removable work.
Dropping nine calls would remove checks (a hypothetical 1.63% of the
baseline prefix). All twelve stay. Their configuration-parser descendants
belong to `engine-causes`; no cross-story cache hides their checks.
Cluster rewriting is 0.09%.
The source's delegating host methods include product work and are not
removable harness costs. Their shared costs stay with `engine-causes`.
Timer-driven retries retain their settings and are ranked only after the
slowdown correction. These are the stopping candidates; no further
harness change is justified above the task's approximate 2% guide.
