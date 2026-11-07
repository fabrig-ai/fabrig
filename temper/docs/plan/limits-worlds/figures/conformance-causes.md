# conformance-causes: what each forked replay still costs

2026-11-07 UTC, base `456f45b727`, 16 CPUs, AMD EPYC-Genoa Processor,
valgrind 3.24.0. Final code `7ee0712de1`. Unoptimized test builds; simulated
time; original patience.
Each sweep's Ir sums PROGRAM TOTALS from its parent and every child.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | Before | 1 | 25,170,053,373 | 6.918 | 1 passed; 78 children |
| same | After | 1 | 17,693,974,838 | 5.463 | 1 passed; 78 children |
| `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` | Before | 1 | 21,870,826,246 | 5.843 | 1 passed; 77 children |
| same | After | 1 | 15,122,795,638 | 4.372 | 1 passed; 77 children |

Final instruction reductions are 29.70% / 30.85%; single native CPU reductions
are 21.03% / 25.18%. The whole serial Temper fuzzy suite measured 27.634 CPU s
(24.938 user + 2.696 system), all fifteen passing. The upstream cuts-fork
after figure is 38.549 CPU s; this is an upstream reference, not a paired
before timing taken by this task. For that native command, replace
`--exact <test-name>` below with `fuzzy:: --test-threads=1`.

## Reproducing the measurements

Build and find the binary and cwd:
`heavy cargo nextest list -p temper-conformance-world --test suite --message-format json`.
For each test above, run the unoptimized binary from its cwd:

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=$PWD valgrind --tool=callgrind \
  --zero-before=skein_fork::process::child \
  --callgrind-out-file=<slot>/target/limits-worlds/conformance-causes/<arm>-<story>.%p.out \
  <binary-path> --exact <test-name> --nocapture'
```

Use the same command before and after. The child-entry reset excludes the
inherited prefix counters. Sum each file's `summary:` (PROGRAM TOTALS),
including the parent once: 79 files for CI repair and 78 for changed judge.
As in `temper/docs/plan/limits-worlds/figures/cuts-fork.md`, these summaries
conservatively retain inherited libtest overhead. Self and inclusive function
counts aggregate raw records by function name; even `calls=0` is a call edge.
Inclusive frames overlap. Forwarding and iterator wrappers are omitted from
the inclusive cause tables. Read annotations with
`callgrind_annotate --inclusive=no <file>` and `--inclusive=yes <file>`.

Native CPU uses bash time, which counts waited-for children (`/usr/bin/time`
is absent). The first and second columns in its output are added above:

```sh
heavy bash -lc 'cd <cwd>
TIMEFORMAT="native user %U system %S wall %R"
time CARGO_MANIFEST_DIR=$PWD <binary-path> --exact <test-name> --nocapture'
```

Native timing is a single run per arm; instructions are the primary comparison.
No budget, deadline, patience, build setting or cut was changed.

## First fix: keyed row projection

Commit `02e320c27a` replaces each changed row's contribution under its key and
retains unchanged naming and translated facts. Erasure removes the row's spend,
balances, writers and receipts; assembling current contributions retains shared
receipts. Cross-row projection ownership and proposal acceptances are resolved
against current rows. Cold observation, bulk writes and missing feeds rebuild
whole. The referee still receives a complete snapshot at each commit, and its
checks and commit boundaries are unchanged. This pays D034.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Keyed projection | 1 | 24,189,059,072 | 6.967 | 1 passed; 78 children |
| Changed judge | Keyed projection | 1 | 20,219,743,088 | 5.657 | 1 passed; 77 children |

The instruction savings are 3.90% and 7.55%. CI's single native CPU timing rose
slightly; it is not presented as a CPU gain. At this arm, CI's policy equality
still costs 3,135,675,258 Ir (12.96%); the projected adapter's borrowed-map work
costs 852,128,829 Ir (3.52%, its inclusive frame less projection and connector
translation). Those are the next causes, in that order.

## Second fix: immutable policy snapshots

Commit `2b0b663c86` gives the adapter and referee immutable, shared policy
values. Mutable access detaches the edited value. Every inspection still
refreshes the referee before processing the same observations; an adapter
without the optional snapshot retains the whole value comparison. Tests cover
both directions of edits and refresh after an external referee edit.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Policy snapshot | 1 | 21,050,322,762 | 6.359 | 1 passed; 78 children |
| Changed judge | Policy snapshot | 1 | 18,015,359,638 | 5.868 | 1 passed; 77 children |

This removes another 3,138,736,310 / 2,204,383,450 Ir, 12.98% / 10.90% of the
preceding arm. The two fixes together remove 16.37% / 17.63% of baseline Ir.
The single changed-judge CPU timing rose; no CPU gain is claimed for that arm.
The next common cause is reconstructing borrowed maps and resolving cross-row
facts: the adapter map alone costs 852 million / 1,012 million Ir in the
preceding arm. Keyed joins can read the changed durable rows directly.

## Third fix: direct durable lookup and keyed joins

Commit `0a8fa91711` consumes the whole row iterator only for a rebuild. Normal
commits look up their submitted keys directly; keyed histories, decisions and
amendment targets resolve the same cross-row facts in the same order. The
connector adapter reads accepted decisions from those current keyed summaries.
A focused test changes and erases an amendment while retaining its decision;
omitting the link fails whole-store equivalence at commit 2.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Direct feed | 1 | 19,849,743,944 | 6.555 | 1 passed; 78 children |
| Changed judge | Direct feed | 1 | 16,542,216,305 | 5.526 | 1 passed; 77 children |

This saves another 1,200,578,818 / 1,473,143,333 Ir, 5.70% / 8.18% of the
preceding arm. CI's single native timing rose slightly again. The remaining
adapter scans in effect evidence cost about 386 / 860 million inclusive Ir;
fixture construction for repeated public document watches is another cause
in CI repair. They are considered next, with independent full-walk comparisons
for indexed evidence and every public readback retained.

## Fourth fix: typed effect evidence lookups

Commit `04347dd69c` narrows entry and call discovery to their typed key ranges,
and looks up known calls, tasks and issues by key. Purpose, task, goal, phase
and origin predicates stay checked. Unit builds compare identities, outcomes,
issue evidence and the complete seen/held/settled accounting with full walks.
Inverting the entry match fails by relabeling a call as a procedure.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Indexed effect evidence | 1 | 19,437,631,801 | 6.779 | 1 passed; 78 children |
| Changed judge | Indexed effect evidence | 1 | 15,589,228,169 | 5.476 | 1 passed; 77 children |

This removes another 412,112,143 / 952,988,136 Ir, 2.08% / 5.76% of the
preceding arm. The native CI timing rose; instructions remain the comparison.
A compatibility check also holds the referee's existing Send and Sync traits;
the final arm uses atomic immutable policy sharing, with edits still detached.


## Fifth fix: reuse public watch setup

Commit `bbd33b6d40` constructs the fixture once per public readback/restart,
retains only its connector and window size, and reuses those two parameters.
All 954 document reads remain. In CI repair, `config::root` calls fall from
1,128 to 252; its inclusive instructions fall from 1,190,351,569 to 268,678,903.
The unit-test build compares the retained parameters with the full factory at
every watch; changing the connector makes that comparison fail.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| CI repair | Shared watch setup | 1 | 18,322,202,612 | 5.548 | 1 passed; 78 children |

This saves 1,115,429,189 Ir, 5.74% of the preceding arm. This arm also includes
atomic policy sharing and the replay-meter repair; it temporarily retains the
preceding row-cache representation to separate setup from cache compaction.
Changed judge does not run this document-readback path and has no separate
watch-setup measurement.

## Sixth and seventh fixes: compact facts and ordered expense totals

Commit `cddfae3cae` retains only each row's contributed facts, with a boxed task
where present, rather than carrying every empty snapshot collection in each
B-tree value. Every commit still assembles the same complete ordered snapshot.
Before this change, `Row::append` costs 1,534,257,813 Ir in the watch-setup CI
arm (8.37%), and 1,994,519,348 in the indexed changed-judge arm (12.79%).
Compaction also fixes the added cache's impact on the core world's replacement
story's heap baseline; its heap ceiling and every heap observation remain.

Commit `7ee0712de1` removes the temporary per-inspection task-spend B-tree.
It emits lost-turn evidence first, then sums the sorted attempt records by
task, retaining zero totals, the same order and every repeated observation.
The preceding indexed profiles put the map's direct entry, allocation,
consumption and destruction calls at 528,431,045 Ir (2.72%) / 354,854,401 Ir
(2.28%). The final arm includes both independent changes; the per-function
measurements below distinguish their work rather than assigning the combined
whole-story difference to either one.

## The focused comparison's guide

The upstream `cuts-fork` answer assigns this task the 1 s guide for
`stories::a_forked_cut_ends_as_a_fresh_cut_does`. With every fresh reference
retained it still measured 1.012 user + 0.070 system = 1.082 CPU s.
Commit `3a19335f0d` takes the answer's first option: compare complete outcomes
with fresh runs at commits 4, 10 and 17. All fourteen forked children still
run the story and referee, and the parent checks their complete ordered cut
list. Each child also checks its empty inherited-result list and one extra
cold cut. This pins the process boundary at its first, middle and last state;
the fourteen fuzzy sweeps still cover every commit.
The new native measurement is 0.451 user + 0.079 system = 0.530 CPU s
(0.842 s wall). Increasing the reference iteration count by one fails at
cut 4, 165 versus 166 iterations, and the parent names that failed child.
No guide, budget or timeout was raised.


## Final contribution measurements

The combined final arm removes another 628,227,774 CI Ir (3.43% of the watch
arm) / 466,432,531 judge Ir (2.99% of the indexed arm). Compaction's individual
instruction gain is modest; it is also the footprint repair that lets the
existing core replacement story keep its heap check. It is not credited with
the expense aggregation's saving.

| Function or context | CI preceding Ir | CI final Ir | Judge preceding Ir | Judge final Ir |
|---|---:|---:|---:|---:|
| `Row::append` | 1,534,257,813 | 1,456,542,617 | 1,994,519,348 | 1,886,847,604 |
| `Row::project` | 803,986,981 | 726,961,121 | 572,246,282 | 535,711,547 |
| `Temper::observed`, less its direct `projected` and `resolved` children | 1,144,897,654 | 702,233,371 | 867,182,346 | 568,308,677 |

The last row isolates the enclosing adapter work: it falls 442,664,283 /
298,873,669 Ir, 2.42% / 1.92% of the respective preceding stories. The new
`observations::expenses` includes the required emissions and two ordered
passes: 461,693,591 / 325,656,775 inclusive Ir; the temporary task map is gone.
The remainder of the whole-arm difference also includes cache compaction and
the compatibility repairs; it is not an isolated net compaction measurement.

The referee's call counts are unchanged:

| Function | CI before / after calls | Judge before / after calls |
|---|---:|---:|
| `Referee::observe` | 321,845 / 321,845 | 215,343 / 215,343 |
| `Referee::durable` | 3,549 / 3,549 | 3,886 / 3,886 |
| `Temper::build` | 157 / 157 | 78 / 78 |
| `restart::startup_page` | 1,600 / 1,600 | 810 / 810 |
| `oracle::document` | 954 / 954 | 0 / 0 |

The final native focused comparison is 0.431 user + 0.060 system = 0.491 CPU s
(0.536 s wall), with all fourteen children still completing.


## CI repair: largest work functions by inclusive Ir

Parent and all children summed; forwarding and thread/libtest wrappers omitted.
Frames overlap.

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| Before | 98.07% | 24,685,170,616 | 1 | `temper_conformance_world::stories::run` |
| Before | 87.60% | 22,048,264,905 | 26,884 | `jig_conformance::harness::Harness<A>::iterate` |
| Before | 66.04% | 16,621,336,529 | 10 | `temper_conformance_world::stories::positive_change` |
| Before | 46.80% | 11,779,329,017 | 96,042 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| Before | 37.82% | 9,518,643,629 | 338 | `temper_conformance_world::stories::wait` |
| Before | 28.39% | 7,146,168,650 | 78 | `temper_conformance_world::oracle::close` |
| Before | 27.31% | 6,873,064,491 | 96,042 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::observed` |
| Before | 22.15% | 5,576,400,969 | 3,704 | `temper_conformance_world::observations::snapshot` |
| Before | 16.96% | 4,268,047,348 | 3,704 | `jig_conformance::core::snapshot::snapshot` |
| Before | 16.04% | 4,037,784,696 | 28,706 | `temper_engine_domain::domain::step` |
| Before | 15.05% | 3,787,179,636 | 10 | `temper_conformance_world::scenarios::change` |
| Before | 14.14% | 3,559,282,995 | 23,482 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::step` |
| After | 97.24% | 17,205,536,448 | 1 | `temper_conformance_world::stories::run` |
| After | 88.22% | 15,609,402,097 | 26,884 | `jig_conformance::harness::Harness<A>::iterate` |
| After | 63.32% | 11,204,369,462 | 10 | `temper_conformance_world::stories::positive_change` |
| After | 34.78% | 6,154,795,724 | 338 | `temper_conformance_world::stories::wait` |
| After | 29.93% | 5,295,296,291 | 96,042 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| After | 29.12% | 5,152,814,838 | 78 | `temper_conformance_world::oracle::close` |
| After | 22.82% | 4,037,106,504 | 28,706 | `temper_engine_domain::domain::step` |
| After | 20.11% | 3,558,565,010 | 23,482 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::step` |
| After | 19.87% | 3,516,626,329 | 96,042 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::observed` |
| After | 19.13% | 3,384,364,500 | 18,601 | `temper_engine_domain::admission::try_step` |
| After | 18.65% | 3,299,877,392 | 65,497 | `temper_engine_domain::route::route_into` |
| After | 16.85% | 2,980,711,679 | 18,636 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::deliver` |

## CI repair: largest functions by self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| Before | 5.29% | 1,331,347,122 | `__memcpy_avx_unaligned_erms` |
| Before | 4.13% | 1,040,449,956 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| Before | 3.54% | 889,919,072 | `core::ptr::copy_nonoverlapping::precondition_check` |
| Before | 3.40% | 856,141,270 | `std::thread::local::LocalKey<T>::try_with` |
| Before | 2.97% | 748,134,764 | `core::ops::function::FnOnce::call_once'2` |
| Before | 2.32% | 583,523,334 | `skein_heap::padded` |
| Before | 2.02% | 508,311,288 | `std::thread::local::LocalKey<T>::initialize_with` |
| After | 6.48% | 1,145,820,230 | `__memcpy_avx_unaligned_erms` |
| After | 4.44% | 785,004,874 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| After | 3.73% | 659,574,770 | `core::ptr::copy_nonoverlapping::precondition_check` |
| After | 3.66% | 647,025,602 | `std::thread::local::LocalKey<T>::try_with` |
| After | 3.19% | 565,231,906 | `core::ops::function::FnOnce::call_once'2` |
| After | 2.50% | 443,050,116 | `skein_heap::padded` |
| After | 2.16% | 382,748,506 | `std::thread::local::LocalKey<T>::initialize_with` |


## Changed judge: largest work functions by inclusive Ir

Parent and all children summed; forwarding and thread/libtest wrappers omitted.
Frames overlap.

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| Before | 97.82% | 21,394,823,897 | 1 | `jig_conformance::scenarios::run` |
| Before | 95.03% | 20,784,440,924 | 18,056 | `jig_conformance::harness::Harness<A>::iterate` |
| Before | 74.50% | 16,294,350,543 | 227 | `temper_conformance_world::scenarios::<impl jig_conformance::scenarios::ScenarioApplication for temper_conformance_world::Temper>::action` |
| Before | 73.96% | 16,176,107,872 | 128 | `temper_conformance_world::scenarios::change` |
| Before | 57.67% | 12,613,194,806 | 66,343 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| Before | 38.61% | 8,444,972,563 | 66,343 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::observed` |
| Before | 31.20% | 6,823,423,585 | 3,962 | `temper_conformance_world::observations::snapshot` |
| Before | 23.27% | 5,088,805,477 | 3,962 | `jig_conformance::core::snapshot::snapshot` |
| Before | 17.31% | 3,786,803,055 | 18,862 | `temper_engine_domain::domain::step` |
| Before | 15.93% | 3,483,486,154 | 370,371 | `jig_conformance::core::naming::naming` |
| Before | 15.08% | 3,298,939,139 | 45,901 | `temper_engine_domain::route::route_into` |
| Before | 14.72% | 3,219,648,891 | 17,109 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::step` |
| After | 96.83% | 14,643,546,193 | 1 | `jig_conformance::scenarios::run` |
| After | 92.52% | 13,992,030,271 | 18,056 | `jig_conformance::harness::Harness<A>::iterate` |
| After | 73.62% | 11,133,764,833 | 227 | `temper_conformance_world::scenarios::<impl jig_conformance::scenarios::ScenarioApplication for temper_conformance_world::Temper>::action` |
| After | 72.99% | 11,038,078,970 | 128 | `temper_conformance_world::scenarios::change` |
| After | 38.19% | 5,775,304,411 | 66,343 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| After | 25.16% | 3,804,963,847 | 66,343 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::observed` |
| After | 25.04% | 3,786,896,640 | 18,862 | `temper_engine_domain::domain::step` |
| After | 21.82% | 3,299,168,285 | 45,901 | `temper_engine_domain::route::route_into` |
| After | 21.29% | 3,220,107,498 | 17,109 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::step` |
| After | 20.18% | 3,052,424,149 | 3,884 | `temper_conformance_world::observations::projected` |
| After | 19.85% | 3,002,388,043 | 12,372 | `temper_engine_domain::admission::try_step` |
| After | 18.68% | 2,825,103,599 | 6,275 | `temper_engine_domain::domain::route` |

## Changed judge: largest functions by self Ir

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| Before | 4.56% | 996,311,232 | `__memcpy_avx_unaligned_erms` |
| Before | 4.12% | 901,957,132 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| Before | 3.52% | 770,860,032 | `core::ptr::copy_nonoverlapping::precondition_check` |
| Before | 3.38% | 739,252,120 | `std::thread::local::LocalKey<T>::try_with` |
| Before | 2.95% | 645,807,790 | `core::ops::function::FnOnce::call_once'2` |
| Before | 2.37% | 517,960,653 | `alloc::collections::btree::navigate::…::next_unchecked closure (type arguments omitted)` |
| Before | 2.31% | 504,687,239 | `skein_heap::padded` |
| After | 5.31% | 802,448,218 | `__memcpy_avx_unaligned_erms` |
| After | 4.53% | 684,661,276 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| After | 3.77% | 570,186,180 | `core::ptr::copy_nonoverlapping::precondition_check` |
| After | 3.71% | 561,115,598 | `std::thread::local::LocalKey<T>::try_with` |
| After | 3.24% | 490,031,764 | `core::ops::function::FnOnce::call_once'2` |
| After | 2.55% | 384,894,406 | `skein_heap::padded` |
| After | 2.19% | 331,686,660 | `std::thread::local::LocalKey<T>::initialize_with` |


## Causes and ownership

Shares below name their identifying arm; overlapping frames are not added.

| Cause | Where | Share of CI / judge | Decision |
|---|---|---:|---|
| Whole-store core projection | `jig/testing/jig-conformance/src/core/snapshot.rs` | before 16.96% / 23.27% | D034 paid: keyed contributions, direct feed, keyed joins, full snapshots |
| Repeated policy equality | `jig/testing/jig-conformance/src/harness.rs` | projection arm 12.96% / 10.89% | immutable atomic policy snapshots, detached edits |
| Rebuilding borrowed maps and resolving joins | `temper/tests/conformance/src/observations.rs`, core projection | direct-feed arm saves 5.70% / 8.18% | keyed storage lookup and retained cross-row evidence |
| Fixture rebuilt per watch | `temper/tests/conformance/src/oracle.rs` | indexed CI factory 6.12%; judge path absent | share only the fixed watch parameters; keep all readback and restart |
| Effect-evidence scans | `temper/tests/conformance/src/observations.rs` | indexed arm saves 2.08% / 5.76% | typed ranges and exact lookups, full-walk focused oracle |
| Temporary spend map per inspection | `temper/tests/conformance/src/lib.rs` | indexed direct map calls 2.72% / 2.28% | ordered aggregation; every observation retained |
| Per-row empty snapshot payloads | core projection | enclosing append 8.37% / 12.79%; net compaction not isolated | compact facts; modest instruction gain and heap footprint repair |
| Engine stepping | `temper/crates/temper-engine-domain/src/domain.rs` | final 22.82% / 25.04% | `engine-causes`; required product work, unchanged calls |
| Allocation padding and metering | `skein/testing/skein-heap/src/lib.rs` | final self 2.50% / 2.55% | `engine-causes`; do not remove or weaken the meter |
| Complete snapshot task copies | core projection, referee vocabulary | final `Task::clone` 3.62% / 5.83%, includes `Scope::clone` | retain the owned complete observation; no changed-row-only verdicts |
| Cold build and restore pages | harness, adapters, engine | builds 157 / 78 and pages 1,600 / 810, unchanged | retain each restart; lower setup belongs to `engine-causes` |

## What stopped here

The next local naming work is below the threshold even if its entire frame
could disappear: `naming::naming` is 340,745,876 / 176,601,465 Ir, 1.93% /
1.17% of the final stories. The Result/Words walk's iterator alone is
119,951,542 / 147,839,883 Ir in the indexed arm (0.62% / 0.95%); its word
checks and emissions remain. Neither justifies another change of this size.
Referee diagnostic formatting is smaller still: its durable check's direct
format calls were 22,059,178 / 30,064,935 Ir in that arm. A check-skipping
change is not made. Core `Observer::inbound` and a neighbour's event-debug
retention are absent from these two profiles; neither gets an unmeasured fix.
The larger remaining copy frames materialize the agreed owned observations;
the engine, store, fake and heap-kit frames go to `engine-causes` and `close`.
Stopped at the next removable local cause below about 2%, retaining every
referee observation, restart, public readback and sweep commit.
