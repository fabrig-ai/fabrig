# cuts-fork: a shared prefix for every live commit

2026-11-06/07 UTC, base `150688094b`, implementation `d775b0ad28`,
16 CPUs, AMD EPYC-Genoa Processor, valgrind 3.24.0. Unoptimized test builds;
simulated time, original patience. The after profiles precede formatting and
the final extraction of the fork-result match into `Harness::fork_cut`;
subsequent tests and clippy check that extraction.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | Before | 1 | 41,387,037,208 | 9.904 | 1 passed; all 78 live cuts |
| same | After, sum of parent + 78 children's files | 1 | 25,170,467,106 | 6.988 | 1 passed; every commit 4 through 81 |
| `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` | Before | 1 | 37,550,774,224 | 7.979 | 1 passed; all 77 live cuts |
| same | After, sum of parent + 77 children's files | 1 | 21,871,236,484 | 6.149 | 1 passed; every commit 4 through 80 |
| `temper-conformance-world`, all fuzzy tests, serial | Before | 1 | unprofiled | 51.795 | 15 passed |
| same | After | 1 | unprofiled | 38.549 | 15 passed |

The instruction reductions are 39.18% and 41.76%. Native CPU reductions are
29.44%, 22.94%, and 25.57% for the whole suite. Each simulated sweep was
profiled once per arm. These are single native timings; CPU seconds vary with
machine load, so the instruction counts are the primary comparison.

An uncut CI-repair focused run (seed 71, 81 commits and 504 iterations, as in
the seed-101 sweep) took 488,398,612 Ir. Its Ir times 78 cuts divided by two
is 19,047,545,868 Ir, 46.02% of the original sweep, above the request's 20%
threshold for building the fork. The fuzzy baseline used seed 101.

## Reproducing the measurements

Build and find the binary and cwd with
`heavy cargo nextest list -p temper-conformance-world --test suite --message-format json`.
For each test named in the first table, the before command is:

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=$PWD valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-worlds/cuts-fork/<label>.%p.out \
  <binary-path> --exact <test-name> --nocapture'
```

The after command adds `--zero-before=skein_fork::process::child`.
Callgrind follows fork without a follow-children flag. Version 3.24 inherits
its counters across fork: resetting at the child's entry excludes the
inherited world prefix, while leaving the parent's counters intact. A separate
focused comparison profile without this reset exposed the inherited counters;
its children's totals are not used for the sweep figures.

Read the parent and child files with `callgrind_annotate --inclusive=yes` and
`--inclusive=no`. For each after arm, sum every file's `summary:`: this is the
Ir that the annotator labels PROGRAM TOTALS. Include the parent once. These
sums conservatively retain about 5.9 million Ir of inherited libtest overhead
per child. The corresponding sums of collected `totals:` are 24,711,310,892
and 21,417,985,945 Ir. They are not substituted for the table's PROGRAM TOTALS.

The child annotations assign active zero-call stack edges to `clone` or
`FnOnce` in their self listing. The tables below aggregate the raw self and
call records by function name, treating even a `calls=0` record as a call
edge. The self sums equal the collected `totals:`. Inclusive frames overlap;
the tables retain the relevant work frames and omit forwarding wrappers.
Incoming calls already active at a child reset are not new calls. The
78 and 77 child files, and the sweep assertions, establish the cut counts.

Native CPU was measured with bash's `time`, which includes the children
waited for by the parent (`/usr/bin/time` is absent):

```sh
heavy bash -lc 'cd <cwd>
TIMEFORMAT="native user %U system %S wall %R"
time CARGO_MANIFEST_DIR=$PWD <binary-path> --exact <test-name> --nocapture'
```

For the whole suite, replace `--exact <test-name>` with `fuzzy:: --test-threads=1`.
No patience, build setting, guide or CPU budget was changed.

## CI repair: largest cause frames by inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| Before | 99.94% | 41,360,939,346 | 79 | `temper_conformance_world::stories::run` |
| Before | 85.31% | 35,308,380,066 | 43,157 | `jig_conformance::harness::Harness<A>::iterate` |
| Before | 65.80% | 27,232,626,905 | 79 | `temper_conformance_world::stories::positive_change` |
| Before | 44.58% | 18,450,509,387 | 155,975 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| Before | 21.02% | 8,697,603,455 | 6,942 | `temper_conformance_world::observations::snapshot` |
| Before | 17.36% | 7,183,060,354 | 79 | `temper_conformance_world::oracle::close` |
| Before | 16.09% | 6,657,489,306 | 6,942 | `jig_conformance::core::snapshot::snapshot` |
| Before | 11.32% | 4,683,381,568 | 504,648 | `jig_conformance::core::naming::naming` |
| After | 98.07% | 24,685,404,320 | 1 | `temper_conformance_world::stories::run` |
| After | 87.60% | 22,048,486,021 | 26,884 | `jig_conformance::harness::Harness<A>::iterate` |
| After | 66.04% | 16,621,525,819 | 10 | `temper_conformance_world::stories::positive_change` |
| After | 46.80% | 11,779,331,942 | 96,042 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| After | 28.39% | 7,146,206,247 | 78 | `temper_conformance_world::oracle::close` |
| After | 22.15% | 5,576,401,897 | 3,704 | `temper_conformance_world::observations::snapshot` |
| After | 16.96% | 4,268,048,276 | 3,704 | `jig_conformance::core::snapshot::snapshot` |
| After | 11.99% | 3,017,859,836 | 322,581 | `jig_conformance::core::naming::naming` |

## CI repair: largest self costs

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| Before | 6.07% | 2,510,494,121 | `__memcpy_avx_unaligned_erms` |
| Before | 4.18% | 1,731,923,454 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| Before | 3.57% | 1,475,748,577 | `core::ptr::copy_nonoverlapping::precondition_check` |
| Before | 3.39% | 1,403,951,090 | `std::thread::local::LocalKey<T>::try_with` |
| Before | 2.32% | 961,943,847 | `skein_heap::padded` |
| After | 5.29% | 1,331,584,046 | `__memcpy_avx_unaligned_erms` |
| After | 4.13% | 1,040,449,956 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| After | 3.54% | 889,919,072 | `core::ptr::copy_nonoverlapping::precondition_check` |
| After | 3.40% | 856,141,270 | `std::thread::local::LocalKey<T>::try_with` |
| After | 2.32% | 583,523,334 | `skein_heap::padded` |

## Changed judge: largest cause frames by inclusive Ir

| Arm | Share | Ir | Calls | Function |
|---|---:|---:|---:|---|
| Before | 99.94% | 37,527,688,714 | 78 | `jig_conformance::scenarios::run` |
| Before | 90.72% | 34,065,694,665 | 33,401 | `jig_conformance::harness::Harness<A>::iterate` |
| Before | 70.91% | 26,627,803,941 | 234 | `temper_conformance_world::scenarios::change` |
| Before | 51.64% | 19,389,461,955 | 122,470 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| Before | 26.56% | 9,971,839,087 | 7,119 | `temper_conformance_world::observations::snapshot` |
| Before | 19.74% | 7,414,329,395 | 7,119 | `jig_conformance::core::snapshot::snapshot` |
| Before | 13.58% | 5,098,837,507 | 544,839 | `jig_conformance::core::naming::naming` |
| After | 97.82% | 21,395,049,759 | 1 | `jig_conformance::scenarios::run` |
| After | 95.03% | 20,784,666,170 | 18,056 | `jig_conformance::harness::Harness<A>::iterate` |
| After | 73.96% | 16,176,300,997 | 128 | `temper_conformance_world::scenarios::change` |
| After | 57.67% | 12,613,209,455 | 66,343 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| After | 31.20% | 6,823,423,766 | 3,962 | `temper_conformance_world::observations::snapshot` |
| After | 23.27% | 5,088,805,664 | 3,962 | `jig_conformance::core::snapshot::snapshot` |
| After | 15.93% | 3,483,486,154 | 370,371 | `jig_conformance::core::naming::naming` |

## Changed judge: largest self costs

| Arm | Share | Ir | Function |
|---|---:|---:|---|
| Before | 5.59% | 2,099,333,852 | `__memcpy_avx_unaligned_erms` |
| Before | 4.14% | 1,554,119,022 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| Before | 3.52% | 1,322,869,236 | `core::ptr::copy_nonoverlapping::precondition_check` |
| Before | 3.35% | 1,257,753,710 | `std::thread::local::LocalKey<T>::try_with` |
| Before | 2.30% | 861,914,893 | `skein_heap::padded` |
| After | 4.56% | 996,529,200 | `__memcpy_avx_unaligned_erms` |
| After | 4.12% | 901,957,132 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| After | 3.52% | 770,860,032 | `core::ptr::copy_nonoverlapping::precondition_check` |
| After | 3.38% | 739,252,120 | `std::thread::local::LocalKey<T>::try_with` |
| After | 2.31% | 504,687,239 | `skein_heap::padded` |

## Causes, largest first

| Cause | Where | Share of each story after the change | Fixed by this task, or whose it is |
|---|---|---|---|
| A fresh deployment repeats each cut's prefix | The two fuzzy sweep helpers | Removed 39.18% / 41.76% of total Ir | This task; all cuts still checked |
| Independent durable snapshots and repeated naming | `jig/testing/jig-conformance/src/core/snapshot.rs`, `core/naming.rs`; `temper/tests/conformance/src/observations.rs` | Core snapshot 16.96% / 23.27%; naming 11.99% / 15.93%; application snapshot 22.15% / 31.20%, overlapping | `conformance-causes`; these files are outside this task |
| Copies and allocation metering below those frames | libc memcpy; `skein/testing/skein-heap/src/lib.rs` | memcpy self 5.29% / 4.56%; heap padding self 2.32% / 2.31% | Profile their callers in `conformance-causes`; heap kit belongs to `engine-causes` |

The fork kit's split/flush/wait path costs 152,412 / 150,458 inclusive Ir;
child hook setup costs 100,230 / 98,945. Dividing their sums by 78 / 77 cuts
gives 3,239 / 3,239 Ir per cut. This is user-space instruction cost, excluding
kernel page-table and copy-on-write work. Native system CPU rose from
0.048 to 0.274 s and from 0.004 to 0.344 s; for the whole suite, 0.303 to
2.300 s. No further product optimization was made across the next task's
ownership boundary; the remaining snapshot costs are well above 2%.

## The focused comparison's guide

`PersonWords` has 17 baseline commits, the fewest of the seven native stories
(`IssueCreation` has 21). The new focused test keeps all fourteen fresh cuts
before running the forked sweep, and compares every child's complete Outcome.
The first serial native measurement was 1.085 user + 0.048 system = 1.133 CPU s,
over the requested 1 s guide. The unreset comparison profile's parent alone
has 2,105,510,429 Ir: 690,379,316 (32.79%) in public readback/restart
`oracle::close`, and 512,416,169 (24.34%) in `Harness::new`. Those checks stay.
A seed-71 trial did not improve the test; it retains seed 101.
The final `d775b0ad28` binary measured 1.091 user + 0.064 system = 1.155 CPU s
(1.184 s wall), with all fourteen comparisons passing. The task's 2026-11-07
answer accepts cuts-fork with this guide unmet and assigns the repair to
`conformance-causes`. That task first checks whether comparing the first,
middle and last live cut holds the comparison's rule; the fuzzy sweeps still
check every commit. No budget or guide was raised, and this task retains all
fourteen comparisons.
