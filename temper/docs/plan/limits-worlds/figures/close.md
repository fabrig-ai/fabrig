# close: deployment worlds on the combined plan trunk

2026-11-07 UTC, commit `bcddfe1ff1`, 16 CPUs, AMD EPYC-Genoa Processor,
Valgrind 3.24.0. Unoptimized test builds, including dependencies.
All builds and test executions used `heavy`, one at a time, with
`TEMPER_BUNDLE=none`. No browser, process or live tier ran.

## Method and exact names

| Key | Package | Exact test in `suite` |
|---|---|---|
| C | `temper-conformance-world` | `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` |
| J | `temper-conformance-world` | `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` |
| Q | `temper-conformance-world` | `stories::a_forked_cut_ends_as_a_fresh_cut_does` |
| B | `temper-conformance-world` | `stories::failed_ci_is_repaired_and_the_new_head_is_reviewed` |
| U | `temper-one-process-world` | `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` |
| O | `temper-one-process-world` | `public::every_owner_version_and_kind_answers_as_declared` |
| M | `temper-one-process-world` | `m15::whole_run_and_renewal_scan_every_fake_issued_token_byte` |
| R | `temper-real-loop-world` | `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` |
| P | `temper-real-loop-world` | `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` |
| G | `temper-real-loop-world` | `github_land::a_chats_change_lands_on_connected_github` |

Native CPU is Python's before/after difference of
`resource.getrusage(resource.RUSAGE_CHILDREN)`, user plus system; wall is
`time.monotonic()`. The wrapper waits for each command, so CPU includes
all waited-for descendants. GNU time is absent; the answer in
`temper/docs/plan/limits-worlds/tasks/figures-before.md`, Answers, explicitly
permits this substitute for every task of this plan. No tool was installed.
For a command, the measurement wrapper is equivalent to:

```python
import os, resource, subprocess, sys, time
before = resource.getrusage(resource.RUSAGE_CHILDREN)
started = time.monotonic()
load_before = os.getloadavg()
code = subprocess.run(sys.argv[1:]).returncode
used = resource.getrusage(resource.RUSAGE_CHILDREN)
print("user", used.ru_utime - before.ru_utime,
      "system", used.ru_stime - before.ru_stime,
      "wall", time.monotonic() - started,
      "load", load_before, os.getloadavg())
sys.exit(code)
```

Build and locate the binaries with:

```sh
heavy cargo nextest list -p temper-conformance-world -p temper-one-process-world \
  -p temper-real-loop-world --test suite --message-format json
```

Each story runs from its listed cwd, with `CARGO_MANIFEST_DIR` set to that
cwd, `--exact NAME --nocapture`. Each native story precedes its first
profile. Simulated stories run once under Callgrind; R/P/G run twice.
Every completed invocation must report exactly `1 passed`; partial
failures are recorded, not interpreted as completed-story totals.

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=$PWD valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-worlds/close/<key>-<run>.%p.out \
  <binary> --exact <name> --nocapture'
```

C/J/Q add `--zero-before=skein_fork::process::child`. Their figures sum
PROGRAM TOTALS (`summary:`) for the parent and every child, retaining
the inherited libtest overhead as in `cuts-fork.md`. Other figures come
from one file. Both `callgrind_annotate --inclusive=yes` and
`--inclusive=no`, with `--auto=no --threshold=100`, were read. Function
tables aggregate identical demangled symbols across contexts and, for
forked tests, files. Calls sum incoming edges, including zero-call
active-stack edges. Inclusive frames overlap; wrappers are omitted from
the work-function tables. Self Ir partitions collected work, which can
differ from PROGRAM TOTALS after child resets.

Only `temper/tests/real-loop/src/lib.rs`'s `PATIENCE` was temporarily
raised from 20 to 3000 seconds for profiling, including the R/P/G native
references built for those profiles. No call-specific bound changed.
Comparisons with preceding real-loop profiles use that same patience.
Patience is restored before shipped-patience survivor and world runs,
and before committing. No capacity, assertion, budget, timeout, compiler
setting or tier membership is changed.

## Closing profiles and native references

| Story | Arm | Run | Ir | Native CPU s (user + system, descendants) | Result |
|---|---|---:|---:|---:|---|
| C | combined trunk | 1 | 14,810,315,805 | 5.029580 | 1 passed |
| J | combined trunk | 1 | 12,794,367,523 | 4.859550 | 1 passed |
| Q | combined trunk | 1 | 1,060,592,335 | 0.508428 | 1 passed |
| B | combined trunk | 1 | 291,672,676 | 0.112383 | 1 passed |
| U | combined trunk | 1 | 10,996,373,926 | 3.899453 | 1 passed |
| O | combined trunk | 1 | 3,792,362,463 | 1.388872 | 1 passed |
| M | combined trunk | 1 | 11,756,350,787 | 2.714692 | 1 passed |
| R | combined trunk | 1 | 23,471,805,382 | 7.350897 | partial: frozen record count fits |
| R | combined trunk | 2 | 22,906,059,880 | 7.350897 | partial: frozen record count fits |
| P | combined trunk | 1 | 21,430,390,341 | 5.430119 | 1 passed |
| P | combined trunk | 2 | 21,249,528,979 | 5.430119 | 1 passed |
| G | combined trunk | 1 | 11,561,761,382 | 2.639637 | 1 passed |
| G | combined trunk | 2 | 11,595,415,230 | 2.639637 | 1 passed |

Native CPU is one separate invocation per story, repeated here only as
its reference for the paired profiles. It is not Callgrind's CPU.

| Story | Native user s | Native system s | Native wall s | Load before → after |
|---|---:|---:|---:|---|
| C | 4.649442 | 0.380138 | 5.555693 | 21.01 / 12.19 / 12.17 → 22.77 / 12.71 / 12.34 |
| J | 4.433195 | 0.426355 | 4.891337 | 1.48 / 10.85 / 15.64 → 1.84 / 10.77 / 15.58 |
| Q | 0.429022 | 0.079406 | 0.595446 | 26.38 / 10.41 / 11.27 → 26.38 / 10.41 / 11.27 |
| B | 0.080916 | 0.031467 | 0.211977 | 23.98 / 14.85 / 12.83 → 23.98 / 14.85 / 12.83 |
| U | 3.823464 | 0.075989 | 4.114746 | 23.61 / 15.21 / 12.98 → 21.96 / 15.01 / 12.93 |
| O | 1.364857 | 0.024015 | 1.390145 | 4.95 / 10.85 / 11.67 → 4.95 / 10.85 / 11.67 |
| M | 2.674829 | 0.039863 | 2.726712 | 2.65 / 9.00 / 10.97 → 2.52 / 8.87 / 10.91 |
| R | 6.915841 | 0.435056 | 7.574938 | 2.52 / 6.56 / 9.77 → 2.56 / 6.50 / 9.73 |
| P | 5.110230 | 0.319889 | 8.442376 | 3.99 / 3.97 / 6.94 → 4.95 / 4.17 / 6.99 |
| G | 2.483185 | 0.156452 | 2.638655 | 0.44 / 1.87 / 4.87 → 0.57 / 1.87 / 4.85 |

| Profile | Wall s | Load before → after | Files | Result |
|---|---:|---|---:|---|
| C-1 | 384.729164 | 22.77 / 12.71 / 12.34 → 34.31 / 33.21 / 22.25 | 79 | 1 passed |
| J-1 | 299.533759 | 1.84 / 10.77 / 15.58 → 1.44 / 5.34 / 11.95 | 78 | 1 passed |
| Q-1 | 33.679164 | 26.38 / 10.41 / 11.27 → 34.25 / 14.10 / 12.47 | 15 | 1 passed |
| B-1 | 10.761024 | 23.98 / 14.85 / 12.83 → 23.84 / 15.12 / 12.94 | 1 | 1 passed |
| U-1 | 109.657843 | 21.96 / 15.01 / 12.93 → 5.97 / 11.34 / 11.83 | 1 | 1 passed |
| O-1 | 51.454203 | 4.95 / 10.85 / 11.67 → 2.83 / 9.36 / 11.11 | 1 | 1 passed |
| M-1 | 108.617515 | 2.52 / 8.87 / 10.91 → 2.84 / 7.10 / 10.06 | 1 | 1 passed |
| R-1 | 216.519479 | 2.56 / 6.50 / 9.73 → 2.16 / 4.25 / 8.15 | 1 | partial: frozen record count fits |
| R-2 | 206.982122 | 1.91 / 4.03 / 7.98 → 5.13 / 4.06 / 7.15 | 1 | partial: frozen record count fits |
| P-1 | 193.849192 | 4.95 / 4.17 / 6.99 → 1.76 / 3.84 / 6.42 | 1 | 1 passed |
| P-2 | 190.967692 | 1.14 / 3.46 / 6.20 → 1.00 / 2.27 / 5.21 | 1 | 1 passed |
| G-1 | 112.028010 | 0.57 / 1.87 / 4.85 → 0.93 / 1.60 / 4.42 | 1 | 1 passed |
| G-2 | 106.769026 | 0.68 / 1.46 / 4.27 → 1.13 / 1.40 / 3.94 | 1 | 1 passed |

## Real-clock effects and partial results

Real-clock poll counts divide by Callgrind wall / native wall before ranking.
Their raw shares are screened by the same ratio; mixed event-driven work is
not automatically a timer. For a partial profile this ratio compares a failed
prefix with a complete native reference, so it is diagnostic, not a measured
successful-story slowdown. No timer was shortened. The poll-operation frame excludes later
asynchronous work; its count normalizes cadence, not the cost of a whole poll.

| Profile | Native wall s | Profile wall s | Ratio | Forge poll calls | Scaled poll calls | Raw poll share | Scaled share | External spawns | Hosted starts |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| R-1 | 7.574938 | 216.519479 | 28.58 | 1,478 | 51.71 | 0.007% | 0.000% | 4 | 69 |
| R-2 | 7.574938 | 206.982122 | 27.32 | 1,473 | 53.91 | 0.008% | 0.000% | 4 | 69 |
| P-1 | 8.442376 | 193.849192 | 22.96 | 1,363 | 59.36 | 0.006% | 0.000% | 0 | 86 |
| P-2 | 8.442376 | 190.967692 | 22.62 | 1,336 | 59.06 | 0.006% | 0.000% | 0 | 85 |
| G-1 | 2.638655 | 112.028010 | 42.46 | 371 | 8.74 | 0.004% | 0.000% | 0 | 45 |
| G-2 | 2.638655 | 106.769026 | 40.46 | 367 | 9.07 | 0.004% | 0.000% | 0 | 45 |

External children execute outside the instruction count. Native CPU includes
their waited-for work. Spawn counts describe each instrumented run or
prefix, not a missing successful-story instruction count. Hosted starts run inside the measured parent and must
not be mistaken for external children. C/J/Q fork children are included by
summing their own files, not by attributing their instructions to the parent.

The forge collector runs on event/readiness passes as well as timer wakes.
Its per-call algorithm is measured directly in real-loop-causes; no raw
polling footprint here is credited as an avoidable native saving.

**Outcomes:** R-1: partial; R-2: partial; P-1: 1 passed; P-2: 1 passed; G-1: 1 passed; G-2: 1 passed.


The policy-deadline scan mixes io readiness and timer wakes. Dividing
its whole cost by slowdown is a conservative screen, not an observed
native call count or a removable fraction. Raw inclusive function tables
are footprints; this screened path is not ranked as an avoidable cause.

| Profile | Policy-deadline calls | Scaled calls | Raw share | Screened share |
|---|---:|---:|---:|---:|
| R-1 | 42,522 | 1487.63 | 4.289% | 0.150% |
| R-2 | 42,595 | 1558.85 | 4.402% | 0.161% |
| P-1 | 292,520 | 12739.61 | 9.538% | 0.415% |
| P-2 | 289,386 | 12793.29 | 9.525% | 0.421% |
| G-1 | 28,730 | 676.69 | 5.862% | 0.138% |
| G-2 | 28,799 | 711.73 | 5.856% | 0.145% |

R: observed count spread 2.47%; failed prefixes do not establish a successful-story error bound or saving.
P: both profiles pass; observed instruction spread 0.85% of the smaller count.
G: both profiles pass; observed instruction spread 0.29% of the smaller count.

### R-1: unsuccessful profile

```text
thread 'fuzzy::repair::a_change_failing_ci_is_repaired_and_lands' (2116867) panicked at skein/crates/skein-fake-peers/src/forge/observe.rs:15:9:
frozen record count fits
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 210.75s
```

### R-2: unsuccessful profile

```text
thread 'fuzzy::repair::a_change_failing_ci_is_repaired_and_lands' (2120829) panicked at skein/crates/skein-fake-peers/src/forge/observe.rs:15:9:
frozen record count fits
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 201.49s
```

R also prints Valgrind warnings for unhandled syscall 424. These samples
do not establish that warning as the cause of the observation-cap assertion.

The complete logs were read before any second required sample. These pairs
are the prescribed variability measurements, not retries replacing a failure.
Failure logs are retained with this run report. Native exact-name references
pass, as do all three shipped-patience runs of the real-loop fuzzy world
(18 tests each); those native results do not turn a partial Ir into a full one.
No observation cap, journal promise, patience in committed code or call bound changed.

## Largest functions

### C: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 88.35% | 13,084,750,991 | 26,884 | `jig_conformance::harness::Harness<A>::iterate` |
| 1 | 28.49% | 4,219,087,002 | 78 | `temper_conformance_world::oracle::close` |
| 1 | 23.27% | 3,447,037,096 | 28,706 | `temper_engine_domain::domain::step` |
| 1 | 23.14% | 3,426,405,414 | 28,706 | `temper_engine_domain::route::step_routed` |
| 1 | 22.60% | 3,347,073,889 | 6,514 | `temper_engine_domain_world::store::Store::request` |
| 1 | 21.70% | 3,213,521,261 | 33,565 | `temper_engine_domain_world::store::Store::drive` |
| 1 | 19.38% | 2,869,558,866 | 18,601 | `temper_engine_domain::admission::try_step` |
| 1 | 18.76% | 2,778,313,795 | 65,497 | `temper_engine_domain::route::route_into` |

### C: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 6.65% | 985,248,524 | `__memcpy_avx_unaligned_erms` |
| 1 | 5.27% | 779,826,804 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 4.42% | 654,897,118 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 2.51% | 371,074,410 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |
| 1 | 2.02% | 298,635,944 | `core::ub_checks::maybe_is_nonoverlapping::runtime` |
| 1 | 1.85% | 274,452,329 | `free` |

### J: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 92.36% | 11,816,585,192 | 18,056 | `jig_conformance::harness::Harness<A>::iterate` |
| 1 | 73.42% | 9,393,307,463 | 227 | `temper_conformance_world::scenarios::<impl jig_conformance::scenarios::ScenarioApplication for temper_conformance_world::Temper>::action` |
| 1 | 72.80% | 9,314,767,836 | 128 | `temper_conformance_world::scenarios::change` |
| 1 | 25.37% | 3,246,436,127 | 18,862 | `temper_engine_domain::domain::step` |
| 1 | 25.27% | 3,233,069,783 | 18,862 | `temper_engine_domain::route::step_routed` |
| 1 | 21.92% | 2,804,714,218 | 45,901 | `temper_engine_domain::route::route_into` |
| 1 | 19.99% | 2,557,962,223 | 12,372 | `temper_engine_domain::admission::try_step` |
| 1 | 19.85% | 2,539,142,426 | 3,884 | `temper_conformance_world::observations::projected` |

### J: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 5.71% | 730,981,645 | `__memcpy_avx_unaligned_erms` |
| 1 | 5.34% | 683,679,090 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 4.45% | 569,209,768 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 2.46% | 315,073,350 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |
| 1 | 2.03% | 259,380,218 | `core::ub_checks::maybe_is_nonoverlapping::runtime` |
| 1 | 1.90% | 243,704,535 | `_int_malloc` |

### Q: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 70.19% | 744,391,190 | 2,551 | `jig_conformance::harness::Harness<A>::iterate` |
| 1 | 39.95% | 423,724,836 | 16 | `temper_conformance_world::oracle::close` |
| 1 | 24.56% | 260,437,007 | 2,999 | `temper_engine_domain::domain::step` |
| 1 | 24.32% | 257,959,541 | 2,999 | `temper_engine_domain::route::step_routed` |
| 1 | 22.69% | 240,695,478 | 3,454 | `temper_engine_domain_world::store::Store::drive` |
| 1 | 22.57% | 239,348,214 | 831 | `temper_engine_domain_world::store::Store::request` |
| 1 | 19.00% | 201,506,782 | 875 | `jig_protocol_store::down` |
| 1 | 18.37% | 194,837,746 | 2,171 | `temper_engine_domain::admission::try_step` |

### Q: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 7.82% | 82,965,501 | `__memcpy_avx_unaligned_erms` |
| 1 | 4.81% | 51,020,130 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 4.05% | 43,006,305 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 2.33% | 24,699,450 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |
| 1 | 1.85% | 19,631,990 | `core::ub_checks::maybe_is_nonoverlapping::runtime` |
| 1 | 1.71% | 18,171,120 | `free` |

### B: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 82.60% | 240,922,730 | 504 | `jig_conformance::harness::Harness<A>::iterate` |
| 1 | 27.53% | 80,301,571 | 603 | `temper_engine_domain::domain::step` |
| 1 | 27.38% | 79,868,170 | 603 | `temper_engine_domain::route::step_routed` |
| 1 | 27.12% | 79,111,386 | 1 | `temper_conformance_world::scenarios::change` |
| 1 | 23.19% | 67,637,125 | 1,304 | `temper_engine_domain::route::route_into` |
| 1 | 22.68% | 66,151,569 | 389 | `temper_engine_domain::admission::try_step` |
| 1 | 21.42% | 62,474,091 | 142 | `temper_engine_domain_world::store::Store::request` |
| 1 | 20.17% | 58,843,166 | 654 | `temper_engine_domain_world::store::Store::drive` |

### B: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 6.77% | 19,745,154 | `__memcpy_avx_unaligned_erms` |
| 1 | 5.44% | 15,865,038 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 4.54% | 13,245,927 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 2.50% | 7,278,420 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |
| 1 | 2.07% | 6,029,294 | `core::ub_checks::maybe_is_nonoverlapping::runtime` |
| 1 | 1.95% | 5,695,700 | `_int_malloc` |

### U: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 22.50% | 2,474,546,299 | 23,298 | `temper_service::system::iterate` |
| 1 | 22.38% | 2,460,451,009 | 116,490 | `temper_service::system::iterate_stage` |
| 1 | 15.71% | 1,727,875,266 | 23,298 | `temper_service::engine::iterate` |
| 1 | 11.58% | 1,273,720,302 | 9,086 | `smith_agent_service::service::iterate` |
| 1 | 9.60% | 1,055,783,935 | 9,086 | `smith_agent_service::service::down::domain_down` |
| 1 | 9.26% | 1,017,762,672 | 111 | `smith_agent_service::service::request::domain_request` |
| 1 | 9.24% | 1,016,531,870 | 15 | `smith_agent_service::service::program::complete` |
| 1 | 9.24% | 1,016,530,160 | 15 | `smith_protocol_llm::component::Component::from_domain` |

### U: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 4.90% | 538,772,792 | `__memcpy_avx_unaligned_erms` |
| 1 | 2.50% | 275,183,912 | `skein_json::string::after_plain` |
| 1 | 2.47% | 271,767,637 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 2.27% | 249,155,510 | `core::option::Option<T>::expect` |
| 1 | 2.07% | 227,208,784 | `core::result::Result<T,E>::expect` |
| 1 | 1.88% | 206,938,867 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |

### O: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 22.36% | 847,857,223 | 7,938 | `temper_service::system::iterate` |
| 1 | 22.23% | 843,054,733 | 39,690 | `temper_service::system::iterate_stage` |
| 1 | 15.73% | 596,574,204 | 7,938 | `temper_service::engine::iterate` |
| 1 | 11.26% | 427,029,608 | 3,200 | `smith_agent_service::service::iterate` |
| 1 | 9.30% | 352,681,954 | 3,200 | `smith_agent_service::service::down::domain_down` |
| 1 | 8.95% | 339,281,055 | 37 | `smith_agent_service::service::request::domain_request` |
| 1 | 8.94% | 338,867,120 | 5 | `smith_agent_service::service::program::complete` |
| 1 | 8.94% | 338,866,550 | 5 | `smith_protocol_llm::component::Component::from_domain` |

### O: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 4.90% | 185,662,838 | `__memcpy_avx_unaligned_erms` |
| 1 | 2.53% | 96,088,068 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 2.43% | 92,141,852 | `skein_json::string::after_plain` |
| 1 | 2.24% | 84,827,804 | `core::option::Option<T>::expect` |
| 1 | 2.03% | 76,932,708 | `core::result::Result<T,E>::expect` |
| 1 | 2.03% | 76,919,453 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |

### M: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 99.90% | 11,744,368,500 | 1 | `temper_one_process_world::m15::run::delivery` |
| 1 | 99.11% | 11,651,589,689 | 1 | `temper_one_process_world::m15::run::delivery::{{closure}}` |
| 1 | 32.12% | 3,775,882,260 | 2,867 | `temper_one_process_world::m15::observer::Observer::check_since` |
| 1 | 32.12% | 3,775,698,772 | 5,734 | `temper_one_process_world::m15::observer::Observer::leak_since` |
| 1 | 31.99% | 3,760,948,162 | 4 | `temper_one_process_world::m15::checkpoint` |
| 1 | 31.93% | 3,754,096,012 | 5 | `temper_one_process_world::m15::observer::Observer::scan` |
| 1 | 31.53% | 3,706,478,451 | 4 | `temper_one_process_world::m15::observer::checkpoint` |
| 1 | 31.53% | 3,706,478,087 | 4 | `temper_one_process_world::m15::observer::checkpoint::{{closure}}` |

### M: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 8.71% | 1,023,518,225 | `aho_corasick::automaton::try_find_overlapping_fwd` |
| 1 | 7.11% | 835,514,806 | `__memcpy_avx_unaligned_erms` |
| 1 | 6.96% | 818,210,829 | `<alloc::string::String as core::fmt::Write>::write_str` |
| 1 | 3.86% | 453,553,000 | `<usize as core::slice::index::SliceIndex<[T]>>::index` |
| 1 | 3.14% | 369,406,175 | `<alloc::vec::Vec<T,A> as core::ops::index::Index<I>>::index` |
| 1 | 2.30% | 270,560,304 | `<core::fmt::builders::DebugStruct>::field` |

### R: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 99.95% | 23,459,496,207 | 1 | `temper_real_loop_world::repair::run` |
| 1 | 22.36% | 5,248,827,916 | 12,549 | `temper_service::system::iterate` |
| 1 | 22.33% | 5,241,235,771 | 62,745 | `temper_service::system::iterate_stage` |
| 1 | 15.69% | 3,681,592,250 | 2,575 | `jig_client_shell_native::shell::Transport::event` |
| 1 | 15.68% | 3,681,146,982 | 2,421 | `jig_client_shell_native::connections::Connections::receive` |
| 1 | 15.58% | 3,656,825,988 | 2,472 | `jig_client_shell_native::connections::Connections::pump` |
| 1 | 15.51% | 3,639,640,573 | 1,349 | `jig_client_protocol_http::stream::Stream::body` |
| 1 | 15.44% | 3,623,970,226 | 3,103 | `temper_client_shell_cli::host::OneShot::events` |
| 2 | 99.95% | 22,893,749,555 | 1 | `temper_real_loop_world::repair::run` |
| 2 | 23.18% | 5,309,333,488 | 12,734 | `temper_service::system::iterate` |
| 2 | 23.15% | 5,301,629,418 | 63,670 | `temper_service::system::iterate_stage` |
| 2 | 15.19% | 3,478,662,542 | 2,496 | `jig_client_shell_native::shell::Transport::event` |
| 2 | 15.18% | 3,478,228,031 | 2,341 | `jig_client_shell_native::connections::Connections::receive` |
| 2 | 15.08% | 3,454,684,537 | 2,392 | `jig_client_shell_native::connections::Connections::pump` |
| 2 | 15.01% | 3,438,057,549 | 1,282 | `jig_client_protocol_http::stream::Stream::body` |
| 2 | 14.98% | 3,431,528,143 | 12,734 | `temper_service::engine::iterate` |

### R: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 4.07% | 955,486,113 | `__memcpy_avx_unaligned_erms` |
| 1 | 2.91% | 682,652,406 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 2.38% | 557,825,305 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 2.21% | 518,515,602 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 1.96% | 459,074,944 | `skein_json::string::after_plain` |
| 1 | 1.71% | 402,444,911 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 4.15% | 951,423,439 | `__memcpy_avx_unaligned_erms` |
| 2 | 2.91% | 667,417,864 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 2 | 2.40% | 548,764,597 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 2.20% | 504,824,869 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 2 | 1.85% | 424,094,099 | `skein_json::string::after_plain` |
| 2 | 1.76% | 402,756,564 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |

### P: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 21.43% | 4,592,348,675 | 14,661 | `temper_service::system::iterate` |
| 1 | 21.39% | 4,583,478,770 | 73,305 | `temper_service::system::iterate_stage` |
| 1 | 14.01% | 3,002,136,798 | 12,494 | `temper_service::engine::iterate` |
| 1 | 9.54% | 2,044,110,329 | 292,520 | `temper_service::system::Service::next_policy_deadline` |
| 1 | 8.05% | 1,725,428,302 | 3,759 | `jig_client_shell_native::shell::Transport::event` |
| 1 | 8.05% | 1,724,678,281 | 3,307 | `jig_client_shell_native::connections::Connections::receive` |
| 1 | 7.88% | 1,687,995,117 | 3,457 | `jig_client_shell_native::connections::Connections::pump` |
| 1 | 7.76% | 1,663,609,787 | 778 | `jig_client_protocol_http::stream::Stream::body` |
| 2 | 21.30% | 4,527,045,652 | 14,469 | `temper_service::system::iterate` |
| 2 | 21.26% | 4,518,291,907 | 72,345 | `temper_service::system::iterate_stage` |
| 2 | 13.93% | 2,960,032,788 | 12,303 | `temper_service::engine::iterate` |
| 2 | 9.52% | 2,023,954,646 | 289,386 | `temper_service::system::Service::next_policy_deadline` |
| 2 | 8.15% | 1,732,266,354 | 3,704 | `jig_client_shell_native::shell::Transport::event` |
| 2 | 8.15% | 1,731,528,210 | 3,261 | `jig_client_shell_native::connections::Connections::receive` |
| 2 | 7.98% | 1,695,405,199 | 3,408 | `jig_client_shell_native::connections::Connections::pump` |
| 2 | 7.87% | 1,671,379,274 | 778 | `jig_client_protocol_http::stream::Stream::body` |

### P: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 5.24% | 1,123,069,616 | `__memcpy_avx_unaligned_erms` |
| 1 | 2.58% | 552,826,371 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 2.55% | 546,674,004 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 1.82% | 390,191,347 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1 | 1.58% | 339,502,951 | `core::array::iter::iter_inner::PolymorphicIter<[core::mem::maybe_uninit::MaybeUninit<T>]>::next` |
| 1 | 1.48% | 316,674,859 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 5.23% | 1,110,600,132 | `__memcpy_avx_unaligned_erms` |
| 2 | 2.57% | 546,174,562 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 2.55% | 542,847,594 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 2 | 1.82% | 387,600,468 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 2 | 1.58% | 336,032,525 | `core::array::iter::iter_inner::PolymorphicIter<[core::mem::maybe_uninit::MaybeUninit<T>]>::next` |
| 2 | 1.47% | 312,843,156 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |

### G: largest work functions by inclusive Ir

| Run | Share | Ir | Calls | Function |
|---:|---:|---:|---:|---|
| 1 | 25.30% | 2,924,736,185 | 8,275 | `temper_service::system::iterate` |
| 1 | 25.25% | 2,919,729,810 | 41,375 | `temper_service::system::iterate_stage` |
| 1 | 14.31% | 1,654,015,489 | 8,275 | `temper_service::engine::iterate` |
| 1 | 8.63% | 997,701,627 | 1,928 | `jig_client_shell_native::shell::Transport::event` |
| 1 | 8.63% | 997,297,507 | 1,703 | `jig_client_shell_native::connections::Connections::receive` |
| 1 | 8.46% | 978,420,396 | 1,777 | `jig_client_shell_native::connections::Connections::pump` |
| 1 | 8.35% | 965,769,803 | 479 | `jig_client_protocol_http::stream::Stream::body` |
| 1 | 8.15% | 941,706,314 | 479 | `jig_client_protocol_http::stream::Stream::piece` |
| 2 | 25.08% | 2,908,539,238 | 8,288 | `temper_service::system::iterate` |
| 2 | 25.04% | 2,903,524,998 | 41,440 | `temper_service::system::iterate_stage` |
| 2 | 14.19% | 1,645,898,705 | 8,288 | `temper_service::engine::iterate` |
| 2 | 8.48% | 983,045,714 | 1,911 | `jig_client_shell_native::shell::Transport::event` |
| 2 | 8.47% | 982,647,523 | 1,689 | `jig_client_shell_native::connections::Connections::receive` |
| 2 | 8.31% | 963,898,069 | 1,762 | `jig_client_shell_native::connections::Connections::pump` |
| 2 | 8.20% | 951,367,602 | 471 | `jig_client_protocol_http::stream::Stream::body` |
| 2 | 8.00% | 927,664,481 | 471 | `jig_client_protocol_http::stream::Stream::piece` |

### G: largest functions by self Ir

| Run | Share | Ir | Function |
|---:|---:|---:|---|
| 1 | 5.48% | 633,300,621 | `__memcpy_avx_unaligned_erms` |
| 1 | 2.53% | 292,692,074 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1 | 2.37% | 274,569,006 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 1.97% | 228,158,394 | `skein_lib::bytes::TwoWay::find` |
| 1 | 1.77% | 205,158,048 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1 | 1.74% | 200,689,031 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 2 | 5.44% | 630,405,690 | `__memcpy_avx_unaligned_erms` |
| 2 | 2.70% | 313,178,866 | `skein_lib::bytes::TwoWay::find` |
| 2 | 2.51% | 290,896,156 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 2 | 2.36% | 273,741,055 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 1.76% | 203,897,506 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2 | 1.72% | 199,450,651 | `core::ptr::copy_nonoverlapping::precondition_check` |


## Each world's three warm serial runs

All nine request rows were built before timing. Each uses its own profile,
`-j 1 --no-fail-fast`, shipped patience, and the CPU wrapper above:

```sh
heavy bash -lc '<resource wrapper> cargo nextest run -p <package> \
  --profile <default|fuzzy> -j 1 --no-fail-fast'
```

Loads are one-/five-/fifteen-minute averages at each command's start and
end, not utilization percentages. An independently idle machine was not
established; load is retained rather than assuming quiet-run guide measurements. CPU includes
the warm cargo/nextest driver and every waited-for descendant, unlike
per-test gate accounting. Before is the request's three-gate median near
`50053b79a2`; its machine load is unknown. Every before/after row therefore
compares different measurement methods. A median is taken by CPU, and its
wall and load come from the same run, not an independently sorted wall.

### Fuzzy

| Package | Run | Passed tests | User s | System s | CPU s | Wall s | Load before → after |
|---|---:|---:|---:|---:|---:|---:|---|
| `temper-real-loop-world` | 1 | 18 | 57.621427 | 3.739574 | 61.361001 | 67.148499 | 1.06 / 1.28 / 3.59 → 0.93 / 1.19 / 3.39 |
| `temper-real-loop-world` | 2 | 18 | 56.497947 | 3.641054 | 60.139001 | 65.996120 | 0.93 / 1.19 / 3.39 → 0.87 / 1.12 / 3.22 |
| `temper-real-loop-world` | 3 | 18 | 56.073142 | 3.391468 | 59.464610 | 64.996200 | 0.87 / 1.12 / 3.22 → 0.95 / 1.09 / 3.06 |
| `temper-conformance-world` | 1 | 15 | 27.805836 | 2.780852 | 30.586688 | 30.446288 | 2.24 / 1.35 / 3.13 → 1.75 / 1.32 / 3.06 |
| `temper-conformance-world` | 2 | 15 | 28.026566 | 2.723077 | 30.749643 | 30.618427 | 1.75 / 1.32 / 3.06 → 1.66 / 1.34 / 3.01 |
| `temper-conformance-world` | 3 | 15 | 26.461450 | 2.642869 | 29.104319 | 28.966094 | 1.66 / 1.34 / 3.01 → 1.52 / 1.33 / 2.95 |
| `temper-engine-domain-world` | 1 | 26 | 27.187403 | 1.013688 | 28.201091 | 28.115880 | 2.72 / 1.59 / 3.02 → 2.19 / 1.56 / 2.97 |
| `temper-engine-domain-world` | 2 | 26 | 26.804011 | 1.033113 | 27.837124 | 27.716566 | 2.19 / 1.56 / 2.97 → 1.72 / 1.50 / 2.90 |
| `temper-engine-domain-world` | 3 | 26 | 27.844565 | 1.149873 | 28.994438 | 28.861318 | 1.72 / 1.50 / 2.90 → 1.44 / 1.45 / 2.84 |
| `temper-one-process-world` | 1 | 4 | 11.143395 | 0.509741 | 11.653136 | 11.899252 | 1.40 / 1.45 / 2.83 → 1.41 / 1.45 / 2.82 |
| `temper-one-process-world` | 2 | 4 | 10.808402 | 0.467131 | 11.275533 | 11.169282 | 1.41 / 1.45 / 2.82 → 1.35 / 1.43 / 2.80 |
| `temper-one-process-world` | 3 | 4 | 10.712882 | 0.423093 | 11.135975 | 11.032505 | 1.35 / 1.43 / 2.80 → 1.29 / 1.42 / 2.78 |

### Focused

| Package | Run | Passed tests | User s | System s | CPU s | Wall s | Load before → after |
|---|---:|---:|---:|---:|---:|---:|---|
| `temper-one-process-world` | 1 | 51 | 26.374421 | 1.524786 | 27.899207 | 27.679666 | 1.27 / 1.41 / 2.77 → 1.25 / 1.39 / 2.72 |
| `temper-one-process-world` | 2 | 51 | 26.405461 | 1.491474 | 27.896935 | 27.702061 | 1.25 / 1.39 / 2.72 → 1.15 / 1.35 / 2.67 |
| `temper-one-process-world` | 3 | 51 | 26.594039 | 1.574352 | 28.168391 | 27.960920 | 1.15 / 1.35 / 2.67 → 1.10 / 1.32 / 2.62 |
| `temper-real-loop-world` | 1 | 219 | 26.624945 | 5.144536 | 31.769481 | 50.246599 | 1.10 / 1.32 / 2.62 → 0.74 / 1.21 / 2.51 |
| `temper-real-loop-world` | 2 | 219 | 26.317748 | 5.276242 | 31.593990 | 49.723030 | 0.74 / 1.21 / 2.51 → 0.57 / 1.09 / 2.41 |
| `temper-real-loop-world` | 3 | 219 | 27.131879 | 5.371853 | 32.503732 | 50.415705 | 0.57 / 1.09 / 2.41 → 0.77 / 1.07 / 2.33 |
| `temper-system-world` | 1 | 48 | 9.061631 | 1.268294 | 10.329925 | 10.240492 | 0.79 / 1.07 / 2.32 → 0.82 / 1.07 / 2.31 |
| `temper-system-world` | 2 | 48 | 9.602885 | 1.203230 | 10.806115 | 10.694047 | 0.82 / 1.07 / 2.31 → 0.92 / 1.08 / 2.30 |
| `temper-system-world` | 3 | 48 | 9.822240 | 1.316740 | 11.138980 | 11.014682 | 0.92 / 1.08 / 2.30 → 0.94 / 1.08 / 2.28 |
| `temper-playground-world` | 1 | 6 | 5.649725 | 0.416373 | 6.066098 | 5.991735 | 1.03 / 1.10 / 2.28 → 1.02 / 1.09 / 2.26 |
| `temper-playground-world` | 2 | 6 | 5.447404 | 0.394208 | 5.841612 | 5.746172 | 1.02 / 1.09 / 2.26 → 1.02 / 1.09 / 2.25 |
| `temper-playground-world` | 3 | 6 | 5.460139 | 0.377391 | 5.837530 | 5.760296 | 1.02 / 1.09 / 2.25 → 1.02 / 1.09 / 2.25 |
| `temper-conformance-world` | 1 | 36 | 5.034894 | 0.823041 | 5.857935 | 5.768169 | 1.02 / 1.09 / 2.25 → 1.02 / 1.09 / 2.24 |
| `temper-conformance-world` | 2 | 36 | 4.826559 | 0.735451 | 5.562010 | 5.480497 | 1.02 / 1.09 / 2.24 → 1.01 / 1.08 / 2.23 |
| `temper-conformance-world` | 3 | 36 | 4.693314 | 0.883230 | 5.576544 | 5.492422 | 1.01 / 1.08 / 2.23 → 1.01 / 1.08 / 2.23 |

### Medians and comparison

| Suite | Package | Before tests | Before gate CPU s | After tests | Median serial CPU s | Run |
|---|---|---:|---:|---:|---:|---:|
| fuzzy | `temper-real-loop-world` | 22 | 85 | 18 | 60.139001 | 2 |
| fuzzy | `temper-conformance-world` | 15 | 44 | 15 | 30.586688 | 1 |
| fuzzy | `temper-engine-domain-world` | 26 | 26 | 26 | 28.201091 | 1 |
| fuzzy | `temper-one-process-world` | 4 | 10 | 4 | 11.275533 | 2 |
| focused | `temper-one-process-world` | 49 | 29 | 51 | 27.899207 | 1 |
| focused | `temper-real-loop-world` | 241 | 30 | 219 | 31.769481 | 1 |
| focused | `temper-system-world` | 42 | 8 | 48 | 10.806115 | 2 |
| focused | `temper-playground-world` | 6 | 5 | 6 | 5.841612 | 2 |
| focused | `temper-conformance-world` | 29 | 3.5 | 36 | 5.576544 | 3 |

Whole-temper suites were not run. No readable combined-trunk end-check
or gate total is present in the supplied plan reports. The account sums
these request rows, labels the sums incomplete, and retains the unknown
whole-suite totals. No slow-tier budget is measured; after-landing owns
the browser, process and live results.

## Native CPU for every surviving belonging-table story

The two profiled fuzzy stories use their native references above, with
profiling patience. Every other survivor uses its exact name from the
same binary listing, once, with restored shipped patience, and must
report exactly `1 passed`. These single runs describe story cost,
not the world's three-run median.

| Test | User s | System s | CPU s | Wall s | Load before → after | Result |
|---|---:|---:|---:|---:|---|---|
| `fuzzy::case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 4.069187 | 0.185146 | 4.254333 | 4.806458 | 0.90 / 1.31 / 3.77 → 0.99 / 1.32 / 3.77 | 1 passed; patience 20 s |
| `fuzzy::case_4_4::the_pace_question_reaches_the_person_and_its_answer_is_kept` | 2.119057 | 0.127943 | 2.247000 | 2.710994 | 0.99 / 1.32 / 3.77 → 0.99 / 1.32 / 3.77 | 1 passed; patience 20 s |
| `fuzzy::case_4_5_limited::a_limited_account_holds_its_work_until_its_reset` | 0.720101 | 0.052297 | 0.772398 | 1.743164 | 0.99 / 1.32 / 3.77 → 0.99 / 1.31 / 3.75 | 1 passed; patience 20 s |
| `fuzzy::case_4_5_refused::a_refused_key_holds_its_work_until_it_is_replaced` | 0.794510 | 0.068216 | 0.862726 | 0.889667 | 0.99 / 1.31 / 3.75 → 0.99 / 1.31 / 3.75 | 1 passed; patience 20 s |
| `fuzzy::case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 1.496534 | 0.100304 | 1.596838 | 1.631129 | 0.99 / 1.31 / 3.75 → 0.99 / 1.31 / 3.75 | 1 passed; patience 20 s |
| `fuzzy::case_5_1::a_held_director_keeps_the_owners_question_and_its_fresh_run_reads_the_new_rules` | 1.847135 | 0.136825 | 1.983960 | 2.004391 | 0.99 / 1.31 / 3.75 → 0.99 / 1.31 / 3.73 | 1 passed; patience 20 s |
| `fuzzy::case_5_2::a_stopped_owner_holds_its_delegates_and_its_release_resumes_them_and_its_news_waits_for_its_release_without_a_park_or_a_retired_charter` | 3.613531 | 0.205451 | 3.818982 | 3.819889 | 0.99 / 1.31 / 3.73 → 0.99 / 1.31 / 3.73 | 1 passed; patience 20 s |
| `fuzzy::case_5_4::a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_with_an_overlap_wake_and_a_closing_comment` | 5.021798 | 0.289025 | 5.310823 | 5.296514 | 0.99 / 1.31 / 3.73 → 0.99 / 1.30 / 3.71 | 1 passed; patience 20 s |
| `fuzzy::case_5_5::an_owners_answered_plan_starts_a_sub_owner_under_a_deadline_that_cannot_delegate_without_the_deadline_firing_or_a_fresh_run_only_at_the_milestones_end` | 1.730193 | 0.087504 | 1.817697 | 2.629771 | 0.99 / 1.30 / 3.71 → 0.99 / 1.30 / 3.71 | 1 passed; patience 20 s |
| `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | 5.110230 | 0.319889 | 5.430119 | 8.442376 | 3.99 / 3.97 / 6.94 → 4.95 / 4.17 / 6.99 | 1 passed; patience 3000 s |
| `fuzzy::case_6_5::a_budget_question_climbs_two_owners_to_the_persons_inbox_and_a_raised_investigation_hands_its_report_to_an_advisor_without_the_director_asking_the_person` | 2.749939 | 0.088448 | 2.838387 | 3.093700 | 0.99 / 1.30 / 3.71 → 0.99 / 1.29 / 3.69 | 1 passed; patience 20 s |
| `fuzzy::case_6_6::a_kit_change_lands_before_the_apps_early_repin_without_relay_without_the_a3_a4_epilogue` | 4.119857 | 0.208397 | 4.328254 | 4.318091 | 0.99 / 1.29 / 3.69 → 1.07 / 1.30 / 3.68 | 1 passed; patience 20 s |
| `fuzzy::case_6_7::a_sub_owner_past_its_deadline_is_held_at_its_next_run_and_escalated_through_its_owner_to_the_director_who_splits_it_by_amendment_and_release_without_a_grace_for_its_delegates` | 2.312900 | 0.136053 | 2.448953 | 3.345711 | 1.07 / 1.30 / 3.68 → 1.07 / 1.30 / 3.68 | 1 passed; patience 20 s |
| `fuzzy::case_6_8::m1s_end_message_starts_m2_beside_a_tester_without_forge_effects_whose_test_lands_only_through_a1s_change` | 3.383440 | 0.188862 | 3.572302 | 3.583671 | 1.07 / 1.30 / 3.68 → 1.07 / 1.30 / 3.67 | 1 passed; patience 20 s |
| `fuzzy::green_light::a_green_light_in_the_inbox_lets_the_change_land` | 5.660671 | 0.316454 | 5.977125 | 6.037068 | 1.07 / 1.30 / 3.67 → 1.06 / 1.29 / 3.65 | 1 passed; patience 20 s |
| `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 2.010890 | 0.143921 | 2.154811 | 2.383468 | 1.06 / 1.29 / 3.65 → 1.06 / 1.29 / 3.64 | 1 passed; patience 20 s |
| `fuzzy::program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | 2.004563 | 0.161009 | 2.165572 | 2.261611 | 1.06 / 1.29 / 3.64 → 1.06 / 1.29 / 3.64 | 1 passed; patience 20 s |
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | 6.915841 | 0.435056 | 7.350897 | 7.574938 | 2.52 / 6.56 / 9.77 → 2.56 / 6.50 / 9.73 | 1 passed; patience 3000 s |

## Causes, largest first

This task changes no code and claims no isolated cause saving. The
account gathers every earlier cause, fixed and retained, with its
identifying arm and why it stands. Every committed figures file was
read, together with the supplied figures-before, cuts-fork (both runs),
conformance-causes (both runs), stories-go and real-loop-causes reports.
The early figures-before report's tool block is superseded by its
recorded answer and completed committed figures. Engine/public/lower
external reports are not present in the supplied run directory; the
engine/public figures' own Left sections and the plan supply those
findings. No missing report is treated as a new passing result.

| Cause | Closing footprint | Why it remains |
|---|---|---|
| Engine steps | C 3,447,037,096 Ir (23.27%); J 3,246,436,127 Ir (25.37%) | Required execution; 28,706 / 18,862 calls remain, as in conformance-causes. |
| Owned referee task copies | C 499,891,919 Ir (3.38%); J 687,376,183 Ir (5.37%) | Complete owned observations; no changed-row-only verdict replaces them. |
| Durable task-record copies | C 742,518,055 Ir (5.01%); J 637,407,683 Ir (4.98%) | Current-row translation still copies owned records. Borrowed translation would need separate representation work and complete-observation proofs; no fix is attempted in this documentation task. |
| Engine deadline lookup | U 671,174,344 Ir (6.10%); O 231,673,466 Ir (6.11%); M 240,538,559 Ir (2.05%) | Ir exactly matches public-causes; the larger shares reflect other work removed. No demonstrated local deadline-maintenance repair. |
| Remaining checked heap header layout | C 365,419,943 Ir (2.47%); J 319,118,826 Ir (2.49%) | Required counted-allocation layout; retain the meter and its checks. The original unforked engine profile identifies the unavoidable leaf separately. |
| Remaining naming | C 259,912,415 Ir (1.75%); J 134,760,887 Ir (1.05%) | Below the approximately 2% local stop rule. |
| JSON parser after intervening repairs | U 960,772,391 Ir (8.74%); O 325,732,852 Ir (8.59%); M 453,012,378 Ir (3.85%) | The trunk includes bc0d8f4c4f and ae9260a7e2. This plan has no paired arms isolating their savings. Remaining independent input validation stays; schema representation is separate work. |
| Remaining string measurement | U 295,507,163 Ir (2.69%); O 98,930,459 Ir (2.61%); M 126,516,238 Ir (1.08%) | The earlier validating writer trial was discarded at 0.76%. Intervening trunk code takes plain runs and omits duplicate writing measurement. Do not treat the old duplicate implementation as still unpaid. |
| Real-loop parser and string measurement | R parser 2,196,130,356 Ir (9.36%) / 2,022,949,717 Ir (8.83%), measure 476,259,806 Ir (2.03%) / 438,111,340 Ir (1.91%); P parser 1,540,803,627 Ir (7.19%) / 1,540,796,281 Ir (7.25%), measure 274,449,081 Ir (1.28%) / 273,973,161 Ir (1.29%); G parser 772,656,369 Ir (6.68%) / 772,494,869 Ir (6.66%), measure 138,033,833 Ir (1.19%) / 137,724,991 Ir (1.19%) | Footprints overlap their callers; R prefixes are partial. Retain validation and bounded encodings; no further cause repair is owned here. |
| Remaining forge collector | R 6,392,543 Ir (0.03%) / 6,433,867 Ir (0.03%); P 6,511,806 Ir (0.03%) / 6,450,697 Ir (0.03%) | The earlier change-triggered snapshots and history cursor remain. Readiness counts vary; no whole-story saving from a partial prefix is inferred. |
| Remaining byte searches | G terminal 209,765,189 Ir (1.81%) / 294,857,026 Ir (2.54%); P cues 220,243,602 Ir (1.03%) / 220,243,602 Ir (1.04%) | The earlier per-query repairs remain. Raw timer/readiness counts need the preceding correction; these are not promises that the whole footprint can disappear. |

## Cleanup and checks

All 27 warm world commands passed: 1,269 test selections, with no rebuild
inside a timed command. All 26 native exact-name invocations passed. Eleven
of thirteen Callgrind invocations passed; both repair profiles stopped at
the recorded observation cap and remain partial. C/J/Q retain all 78 / 77 /
14 forked live cuts. No test, assertion, limit, budget or committed build
setting changed here; no test was added.

Patience is restored to 20 seconds, with no source diff. Raw profiles and
annotations under `target/limits-worlds/` were deleted after these figures
were written; structured measurements and logs remain with the run report.
Formatting, the temper old-form ratchet and committed-tree citations are
checked for the final documentation commit; outcomes are in its run report.
