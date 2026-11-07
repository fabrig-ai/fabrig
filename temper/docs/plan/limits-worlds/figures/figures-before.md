# figures-before: where the five worlds spend instructions

2026-11-06–07 UTC; measured checkout `e4c4f9cf969a8291b6b07a2f0173ae4970e658b2`, containing integration base `73a713a9b72e269e09f14aac0a36ef63c644d877`; 16 CPUs, AMD EPYC-Genoa Processor; Valgrind 3.24.0. Unoptimized test binaries. The fork optimization is absent from this checkout.

## Method and story keys

| Key | Package | Exact test in `suite` | Suite |
|---|---|---|---|
| R | `temper-real-loop-world` | `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | fuzzy |
| P | `temper-real-loop-world` | `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | fuzzy |
| G | `temper-real-loop-world` | `github_land::a_chats_change_lands_on_connected_github` | focused |
| C | `temper-conformance-world` | `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | fuzzy |
| U | `temper-one-process-world` | `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` | focused |

All builds and executions used `heavy`, sequentially, with `TEMPER_BUNDLE=none`. `cargo nextest list -p temper-real-loop-world -p temper-conformance-world -p temper-one-process-world --test suite --message-format json` located the binaries. Each invocation used that binary with `--exact NAME --nocapture`, its listed working directory and `CARGO_MANIFEST_DIR`. No slow tier, walk driver, optimization override or product change was used.

Native CPU is user plus system of the test and its waited-for descendants, measured by Python `resource.getrusage(resource.RUSAGE_CHILDREN)` before and after `subprocess.run`; wall time uses `time.monotonic()`. This is the task’s 2026-11-06 answer in place of unavailable GNU time. No download or machine change was needed: Valgrind and `callgrind_annotate` were already installed.

First, all five stories ran once with the unchanged 20-second patience and passed. Then `temper/tests/real-loop/src/lib.rs` alone was temporarily changed to `PATIENCE = from_secs(3000)`, rebuilt, and the three real-loop native measurements and both profiles per story used that same build. No call-specific timeout or answer bound was changed. Patience was restored to 20 seconds before committing.

Callgrind arguments were `--tool=callgrind --callgrind-out-file=.../LABEL.cgN.%p.out`. Each run produced one file; each Ir below is its `PROGRAM TOTALS`, not a sum of forked cuts. Native shell children are outside Ir; hosted programs execute in the measured process. Function costs and incoming calls are grouped by identical demangled symbol across Callgrind contexts, including inline contexts; self totals still partition the complete Ir. The raw files were read with `callgrind_annotate --inclusive=yes --auto=no --threshold=100` and `--inclusive=no` with the same options. All profile files and annotations under `target/limits-worlds/` were deleted after recording these figures.

## Native results on unchanged source

| Story | Native CPU s, descendants included | Native wall s | Result |
|---|---|---|---|
| R | 8.526526 | 8.525244 | 1 passed |
| P | 7.316409 | 9.430814 | 1 passed |
| G | 3.323168 | 3.317098 | 1 passed |
| C | 10.483911 | 10.491085 | 1 passed |
| U | 4.840739 | 4.842804 | 1 passed |

## Before profiles

| Story | Arm | Run | Ir | Native CPU s (user + system, descendants) | Native wall s | Callgrind wall s | Wall ratio | Result |
|---|---|---|---|---|---|---|---|---|
| R | before | 1 | 25,889,333,321 | 9.461922 | 9.498569 | 238.222627 | 25.08× | Partial: frozen record count |
| R | before | 2 | 28,824,912,164 | 9.461922 | 9.498569 | 250.095339 | 26.33× | Partial: frozen record count |
| P | before | 1 | 32,077,040,893 | 6.982877 | 9.347564 | 273.783157 | 29.29× | Partial: oracle assertion |
| P | before | 2 | 39,395,136,937 | 6.982877 | 9.347564 | 359.511294 | 38.46× | Partial: frozen record count |
| G | before | 1 | 14,649,575,831 | 3.039783 | 3.099840 | 152.791068 | 49.29× | 1 passed |
| G | before | 2 | 16,436,998,947 | 3.039783 | 3.099840 | 168.345666 | 54.31× | 1 passed |
| C | before | 1 | 41,386,230,122 | 10.483911 | 10.491085 | 353.138831 | 33.66× | 1 passed |
| U | before | 1 | 18,514,602,590 | 4.840739 | 4.842804 | 174.586156 | 36.05× | 1 passed |

R1/R2 and P2 failed at `skein/crates/skein-fake-peers/src/forge/observe.rs:15`, “frozen record count fits”: the retained forge observation cap is 2,048 (`temper/tests/real-loop/src/forge.rs`). P1 reached the ending oracle and failed at `temper/tests/real-loop/src/case/oracle.rs:555`: `6.4 Add "every attempt ended, none failed": task 2's complete journal ends every admitted attempt and permits only bounded placement rechecks`. Their complete profiler output describes unsuccessful execution, so these four figures are marked partial and cannot stand for a completed successful story. No patience expiry fired. R also printed Valgrind warnings for unhandled syscall 424; the measurements do not establish that warning as the assertion’s cause.

R instruction spread: 11.34% of the smaller count. The two partial failures do not establish a completed-story error bound.

P instruction spread: 22.81% of the smaller count. The two failed runs are diagnostic only; P reached different endpoints.

G instruction spread: 12.20% of the smaller count. Treat this as the observed run-to-run error; a smaller apparent gain is not evidence of improvement.

C passed with 81 durable commits and 504 baseline iterations. The profile records 79 calls to `stories::run` / `Harness::start`, including its baseline and selected live cuts; the pre-live fixture commits are excluded by the test. U passed with its three service worlds (one calibration, then direct and proxied identifier checks). Each passing invocation reported exactly `1 passed`; each unsuccessful profile reported `0 passed; 1 failed`.

## Children and real-clock corrections

| Profile | Native children: calls to `skein_shell::linux::process::spawn` | Hosted starts: `skein_world::real::Running::spawn` |
|---|---|---|
| R1 | 3 | 70 |
| R2 | 4 | 71 |
| P1 | 0 | 88 |
| P2 | 0 | 67 |
| G1 | 0 | 45 |
| G2 | 0 | 45 |
| C1 | 0 | 0 |
| U1 | 0 | 0 |

These are observed spawn-path calls in the instrumented parent, not a descendant instruction count. R’s native shell children run uninstrumented; G/P use hosted git and agent programs on these paths. C/U have simulated process spawning, with no native child starts on this path. Native CPU above includes all waited-for descendants.

For real-loop timer/readiness paths, use the profile’s wall/native-wall ratio S: equivalent calls = observed calls / S, adjusted share = raw inclusive share / S. For partial profiles S is only an elapsed-time ratio to the complete native run, not a completed-story slowdown estimate. Event-loop scans are mixed event/timer work; applying the same division to their whole cost is a conservative screen, not an observed native call count. No timer correction is applied to deterministic simulated C/U.

| Profile | Path | Observed calls | Equivalent calls | Raw share | Adjusted share |
|---|---|---|---|---|---|
| R1 | `temper_service::system::Service::next_policy_deadline` | 38,520 | 1535.9 | 3.45% | 0.14% |
| R1 | `temper_real_loop_world::forge::Seen::collect` | 11,915 | 475.1 | 3.11% | 0.12% |
| R2 | `temper_service::system::Service::next_policy_deadline` | 39,789 | 1511.2 | 3.26% | 0.12% |
| R2 | `temper_real_loop_world::forge::Seen::collect` | 12,364 | 469.6 | 4.06% | 0.15% |
| P1 | `temper_service::system::Service::next_policy_deadline` | 318,856 | 10886.5 | 6.95% | 0.24% |
| P1 | `temper_real_loop_world::forge::Seen::collect` | 15,161 | 517.6 | 5.88% | 0.20% |
| P2 | `temper_service::system::Service::next_policy_deadline` | 528,390 | 13738.5 | 9.43% | 0.25% |
| P2 | `temper_real_loop_world::forge::Seen::collect` | 25,172 | 654.5 | 11.97% | 0.31% |
| G1 | `temper_service::system::Service::next_policy_deadline` | 28,571 | 579.7 | 4.60% | 0.09% |
| G1 | `temper_real_loop_world::terminal::contains` | 8,778 | 178.1 | 6.25% | 0.13% |
| G2 | `temper_service::system::Service::next_policy_deadline` | 29,942 | 551.3 | 4.31% | 0.08% |
| G2 | `temper_real_loop_world::terminal::contains` | 16,765 | 308.7 | 11.00% | 0.20% |

The table stays below 25 rows. These screened paths are all below 2% after correction, so they are not ranked as causes. `forge::Seen::collect` rebuilds branch/pull snapshots and rescans all peer observations; `terminal::contains` uses byte windows repeatedly in readiness checks; service deadline checks traverse engine and worker deadlines. Their raw prominence is useful for diagnosis, but is not a native saving estimate.

## Self-instructions by crate

Attribution is by the symbol’s first Rust path segment, including the implementing type for `<crate::Type as Trait>`. Generic `core`/`alloc` work stays there, even when called by product code. Unscoped C, primitive and unknown symbols are grouped separately. Every crate reaching 1% in any individual profile appears. Cells are billions of self Ir and percentage of that run’s total; paired entries are run 1 / run 2, never summed.

| Crate / symbol group | R | P | G | C | U |
|---|---|---|---|---|---|
| `core` | 9.931 / 11.047 B (38.36 / 38.33%) | 12.779 / 15.553 B (39.84 / 39.48%) | 5.879 / 6.874 B (40.13 / 41.82%) | 15.190 B (36.70%) | 5.574 B (30.11%) |
| `alloc` | 2.720 / 3.111 B (10.51 / 10.79%) | 3.676 / 6.048 B (11.46 / 15.35%) | 1.598 / 1.691 B (10.91 / 10.29%) | 9.533 B (23.03%) | 5.336 B (28.82%) |
| `[unscoped/native]` | 2.058 / 2.297 B (7.95 / 7.97%) | 3.213 / 4.245 B (10.02 / 10.77%) | 1.697 / 1.934 B (11.58 / 11.76%) | 6.278 B (15.17%) | 1.726 B (9.32%) |
| `skein_json` | 3.632 / 3.989 B (14.03 / 13.84%) | 3.266 / 2.471 B (10.18 / 6.27%) | 1.379 / 1.556 B (9.41 / 9.47%) | 0.000 B (0.00%) | 2.451 B (13.24%) |
| `std` | 1.272 / 1.410 B (4.91 / 4.89%) | 1.359 / 1.487 B (4.24 / 3.78%) | 0.577 / 0.611 B (3.94 / 3.72%) | 3.631 B (8.77%) | 0.004 B (0.02%) |
| `skein_lib` | 1.913 / 2.155 B (7.39 / 7.48%) | 1.975 / 2.160 B (6.16 / 5.48%) | 1.056 / 1.119 B (7.21 / 6.81%) | 0.537 B (1.30%) | 0.995 B (5.37%) |
| `skein_heap` | 1.059 / 1.175 B (4.09 / 4.08%) | 1.131 / 1.238 B (3.53 / 3.14%) | 0.481 / 0.508 B (3.28 / 3.09%) | 3.027 B (7.31%) | 0.000 B (0.00%) |
| `temper_real_loop_world` | 0.421 / 0.455 B (1.62 / 1.58%) | 1.115 / 1.477 B (3.48 / 3.75%) | 0.215 / 0.280 B (1.47 / 1.70%) | 0.000 B (0.00%) | 0.000 B (0.00%) |
| `skein_http` | 0.608 / 0.658 B (2.35 / 2.28%) | 0.472 / 0.337 B (1.47 / 0.85%) | 0.275 / 0.291 B (1.87 / 1.77%) | 0.000 B (0.00%) | 0.102 B (0.55%) |
| `temper_one_process_world` | 0.000 / 0.000 B (0.00 / 0.00%) | 0.000 / 0.000 B (0.00 / 0.00%) | 0.000 / 0.000 B (0.00 / 0.00%) | 0.000 B (0.00%) | 0.313 B (1.69%) |
| `jig_protocol_store` | 0.059 / 0.062 B (0.23 / 0.21%) | 0.083 / 0.116 B (0.26 / 0.30%) | 0.043 / 0.045 B (0.30 / 0.27%) | 0.690 B (1.67%) | 0.106 B (0.57%) |
| `skein_forge_connection` | 0.215 / 0.266 B (0.83 / 0.92%) | 0.294 / 0.543 B (0.92 / 1.38%) | 0.222 / 0.238 B (1.51 / 1.45%) | 0.000 B (0.00%) | 0.099 B (0.53%) |
| `jig_conformance` | 0.000 / 0.000 B (0.00 / 0.00%) | 0.000 / 0.000 B (0.00 / 0.00%) | 0.000 / 0.000 B (0.00 / 0.00%) | 0.605 B (1.46%) | 0.000 B (0.00%) |
| `temper_service` | 0.128 / 0.136 B (0.50 / 0.47%) | 0.241 / 0.412 B (0.75 / 1.05%) | 0.090 / 0.096 B (0.62 / 0.58%) | 0.013 B (0.03%) | 0.208 B (1.12%) |

## Causes, largest first

Ranked by the largest measured share in any story. These are cost footprints and candidates for their owners, not promised savings. Inclusive categories overlap: allocator bookkeeping is inside snapshots/parsing, and prompt measurement includes string writing. Do not add the rows. Partial R/P shares describe their failed paths only. “0.00%” means the named path was absent or below the displayed precision. No code was fixed by this task.

| Cause | Where / measured footprint | Share of each story (run 1 / run 2) | Whose it is |
|---|---|---|---|
| Per-allocation header-layout recomputation and thread-local live/peak bookkeeping | `skein/testing/skein-heap/src/lib.rs`: `padded`, `allocated`, `freed`; their three disjoint inclusive subpaths, excluding System allocation itself | R 14.27% / 14.22%; P 12.30% / 10.96%; G 11.44% / 10.79%; C 25.49%; U 0.00% | `engine-causes` |
| Scan every durable row again while waiting for a landed record | `temper/tests/one-process/src/public/owners.rs`: `Walk::landed`, reached by `ready` in every referee observation | R 0.00% / 0.00%; P 0.00% / 0.00%; G 0.00% / 0.00%; C 0.00%; U 19.86% | `public-causes` |
| Rebuild the complete referee snapshot, including formatting each key into an owned string | `jig/testing/jig-conformance/src/core/snapshot.rs` and `core/naming.rs`; `snapshot` includes `format!("{key:?}")` per row | R 0.00% / 0.00%; P 0.00% / 0.00%; G 0.00% / 0.00%; C 16.09%; U 0.00% | `conformance-causes` (jig-conformance); application projection in `temper/tests/conformance/src/observations.rs` also belongs there |
| Reparse complete schema/request/configuration JSON into retained documents, byte by byte | `skein/crates/skein-json/src/parse.rs`, `string.rs`; callers include `skein/crates/skein-llm/src/wire/json.rs::Json::from_bytes`, fake LLM request decoding and configuration documents | R 12.85% / 13.12%; P 9.81% / 6.03%; G 10.03% / 9.75%; C 0.00%; U 10.15% | `engine-causes` |
| Compare the full nested referee policy on every inspection even when unchanged | `jig/testing/jig-conformance/src/harness.rs::inspect_outside`: `referee.policy != *policy` | R 0.00% / 0.00%; P 0.00% / 0.00%; G 0.00% / 0.00%; C 12.30%; U 0.00% | `conformance-causes` |
| Validate UTF-8 and count escapes on every string pass, including the writing pass | `skein/crates/skein-json/src/writer.rs::Encoder::put_string` always calls `escaped_len`, then `write_escaped` on writing; nested prompt/schema passes repeat this work | R 11.09% / 10.52%; P 7.53% / 4.15%; G 6.81% / 7.03%; C 0.00%; U 9.37% | `engine-causes` |
| Reconstruct fixture/store/domain setup independently for every live-cut replay | `jig/testing/jig-conformance/src/harness.rs::start`, called by `temper/tests/conformance/tests/suite/fuzzy/stories.rs`; counted setup only, not all later replay work | R 0.00% / 0.00%; P 0.00% / 0.00%; G 0.00% / 0.00%; C 7.91%; U 0.00% | `cuts-fork`, then `conformance-causes` |
| Measure whole prompts repeatedly for observation and preparation/admission | `smith/crates/smith-protocol-llm/src/component.rs::observed_prompt`; `skein/crates/skein-llm/src/client.rs::measure` and `translate.rs::measure_request` | R 7.35% / 6.20%; P 3.98% / 2.03%; G 3.70% / 4.26%; C 0.00%; U 5.80% | `engine-causes` |
| Rescan system text for every agent cue when classifying each query for the ending oracle | `temper/tests/real-loop/src/case/oracle.rs::cued`, reached by `Told::new`; measured only in P1 because P2 failed before the ending oracle | R 0.00% / 0.00%; P 4.50% / 0.00%; G 0.00% / 0.00%; C 0.00%; U 0.00% | `real-loop-causes` (after `stories-go`) |
| Clone deployment JSON and parse configuration again when rebuilding defaults | `temper/tests/real-loop/src/fixtures.rs::own_defaults`; P invokes it 21 times | R 0.89% / 0.80%; P 3.07% / 2.50%; G 1.55% / 1.39%; C 0.00%; U 0.00% | `real-loop-causes` (after `stories-go`) |
| Re-derive all generated store-family bounds repeatedly | `jig/crates/jig-protocol-store/src/codec.rs::limits`, including tasks, notes, core, settings and people; 2,061 calls in C | R 0.16% / 0.14%; P 0.23% / 0.18%; G 0.29% / 0.25%; C 2.17%; U 0.15% | `engine-causes` |

C’s snapshots visit 504,648 row names across 6,942 snapshots; `naming` alone costs 11.32% inside the 16.09% snapshot row. Policy equality runs 155,975 times (12.30%). U calls `landed` 91,773 times, visiting its scan closure 8,773,340 times (19.86% including iteration). C makes 8,152,067 counted allocations: `padded` runs 16,304,133 times across allocation/free. R makes roughly 2.85–3.17 million counted allocations in its partial paths. These identify repetition in the tree rather than merely a hot function name.

P1’s `Told::new` entry is 4.50%, of which `cued` is 4.498% and `with_agent` is only 0.002%. `cued` scans the complete system text with byte windows for each candidate agent cue. Text/history copying is therefore not the measured oracle cause.

The JSON parse row includes configuration and LLM paths; in R, the `Json::from_bytes -> parse` edge alone is 11.75% / 12.13%, G 8.04% / 7.97%, and U 9.62%. `Json::from_document` additionally validates the retained document and measures its writing. The string-writing row is shared by real-loop and public worlds. The common engine and codec costs therefore remain relevant even though the partial stories cannot establish end-to-end improvements.

Stop at the 2% screen. Examples below it: GitHub’s credential-leak scan is 0.44% in G1; `stories::given` is 1.22% in U; the corrected real-loop scans/deadline checks above stay under 0.32%. Ordinary required execution, store commits and oracle readback have inclusive costs, but their parent functions alone do not establish an avoidable cause. No fix, reduced assertion, tier move or larger budget is inferred from those totals.

## R1 (partial): the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.95% | 25,876,834,603 | 1 | `temper_real_loop_world::repair::run` |
| 99.95% | 25,876,834,493 | 1 | `temper_real_loop_world::run_forge_with` |
| 99.95% | 25,875,487,969 | 1 | `temper_real_loop_world::world::run_forge` |
| 97.37% | 25,208,009,453 | 1 | `skein_world::real::World<P,R>::run` |
| 97.36% | 25,205,516,231 | 1 | `skein_world::real::Running<P,R>::run` |
| 89.53% | 23,179,153,743 | 11,915 | `skein_world::real::Running<P,R>::drive` |
| 85.03% | 22,012,877,087 | 67,561 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 80.62% | 20,872,374,120 | 67,561 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 80.28% | 20,783,263,650 | 67,561 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 21.37% | 5,532,777,470 | 11,227 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 19.12% | 4,949,586,449 | 11,227 | `temper_service::system::iterate` |
| 19.09% | 4,942,794,114 | 56,135 | `temper_service::system::iterate_stage` |
| 18.72% | 4,847,459,459 | 2,812 | `<temper_real_loop_world::calls::recorded::Recorded as skein_shell::drive::Host>::iterate` |
| 18.72% | 4,846,179,081 | 2,812 | `<temper_client_shell_cli::host::OneShot as skein_shell::drive::Host>::iterate` |
| 16.62% | 4,301,688,201 | 1,024 | `<skein_sim_world::writer_pipe::PipeHost<P> as skein_shell::drive::Host>::iterate` |

## R1 (partial): the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 3.41% | 883,835,302 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 3.10% | 802,107,566 | `__memcpy_avx_unaligned_erms` |
| 3.06% | 792,849,036 | `skein_json::writer::escaped_len` |
| 2.63% | 679,950,395 | `skein_json::writer::escaped` |
| 2.41% | 624,467,699 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.36% | 610,175,386 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.95% | 504,701,047 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.90% | 491,237,353 | `std::thread::local::LocalKey<T>::try_with` |
| 1.78% | 461,716,412 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.68% | 433,760,388 | `core::ops::function::FnOnce::call_once'2` |
| 1.36% | 352,082,457 | `core::result::Result<T,E>::expect` |
| 1.30% | 336,880,973 | `skein_heap::padded` |

## R2 (partial): the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.96% | 28,812,394,302 | 1 | `temper_real_loop_world::repair::run` |
| 99.96% | 28,812,394,192 | 1 | `temper_real_loop_world::run_forge_with` |
| 99.95% | 28,810,930,087 | 1 | `temper_real_loop_world::world::run_forge` |
| 97.64% | 28,143,450,740 | 1 | `skein_world::real::World<P,R>::run` |
| 97.63% | 28,140,957,312 | 1 | `skein_world::real::Running<P,R>::run` |
| 89.00% | 25,654,323,295 | 12,364 | `skein_world::real::Running<P,R>::drive` |
| 84.63% | 24,393,365,712 | 70,951 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 80.41% | 23,179,303,380 | 70,951 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 80.07% | 23,079,039,013 | 70,951 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 21.39% | 6,166,537,776 | 11,700 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 19.37% | 5,582,511,901 | 11,700 | `temper_service::system::iterate` |
| 19.34% | 5,575,433,401 | 58,500 | `temper_service::system::iterate_stage` |
| 17.87% | 5,150,614,726 | 2,866 | `<temper_real_loop_world::calls::recorded::Recorded as skein_shell::drive::Host>::iterate` |
| 17.86% | 5,149,312,646 | 2,866 | `<temper_client_shell_cli::host::OneShot as skein_shell::drive::Host>::iterate` |
| 14.43% | 4,159,774,489 | 788 | `<skein_fake_peers::llm::Peer as skein_shell::drive::Host>::iterate` |

## R2 (partial): the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 3.45% | 993,790,067 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 3.09% | 891,939,834 | `__memcpy_avx_unaligned_erms` |
| 2.89% | 832,712,895 | `skein_json::writer::escaped_len` |
| 2.50% | 719,285,192 | `skein_json::writer::escaped` |
| 2.38% | 684,742,029 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.34% | 673,982,250 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 2.04% | 587,178,163 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.89% | 544,849,237 | `std::thread::local::LocalKey<T>::try_with` |
| 1.77% | 509,935,260 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.67% | 480,750,674 | `core::ops::function::FnOnce::call_once'2` |
| 1.41% | 407,681,098 | `core::result::Result<T,E>::expect` |
| 1.30% | 373,661,160 | `skein_heap::padded` |

## P1 (partial): the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 95.09% | 30,501,790,501 | 1 | `temper_real_loop_world::cluster::run_cluster_forge` |
| 95.09% | 30,501,790,398 | 1 | `temper_real_loop_world::cluster::run_with` |
| 95.09% | 30,501,706,021 | 1 | `temper_real_loop_world::cluster::run_cluster_forge::{{closure}}` |
| 95.09% | 30,501,705,956 | 1 | `temper_real_loop_world::world::run_cluster_forge` |
| 89.53% | 28,717,044,767 | 1 | `skein_world::real::World<P,R>::run` |
| 89.51% | 28,713,224,508 | 1 | `skein_world::real::Running<P,R>::run` |
| 78.41% | 25,152,599,467 | 15,161 | `skein_world::real::Running<P,R>::drive` |
| 71.41% | 22,904,959,613 | 129,773 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 65.16% | 20,901,773,812 | 129,773 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 64.76% | 20,773,380,435 | 129,773 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 21.34% | 6,844,033,384 | 16,003 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 18.51% | 5,938,677,967 | 16,003 | `temper_service::system::iterate` |
| 18.48% | 5,928,996,152 | 80,015 | `temper_service::system::iterate_stage` |
| 13.00% | 4,170,862,543 | 5,055 | `<temper_real_loop_world::calls::recorded::Recorded as skein_shell::drive::Host>::iterate` |
| 13.00% | 4,168,533,206 | 5,055 | `<temper_client_shell_cli::host::OneShot as skein_shell::drive::Host>::iterate` |

## P1 (partial): the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 3.89% | 1,247,878,116 | `__memcpy_avx_unaligned_erms` |
| 3.23% | 1,036,476,727 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 3.05% | 979,260,574 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2.16% | 693,539,596 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 2.00% | 640,613,025 | `skein_json::writer::escaped_len` |
| 1.93% | 618,126,097 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.76% | 564,813,890 | `skein_json::writer::escaped` |
| 1.64% | 524,619,142 | `std::thread::local::LocalKey<T>::try_with` |
| 1.56% | 500,885,034 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.46% | 468,733,181 | `core::ops::function::FnOnce::call_once'2` |
| 1.33% | 426,787,472 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |
| 1.18% | 380,069,995 | `core::array::iter::iter_inner::PolymorphicIter<[core::mem::maybe_uninit::MaybeUninit<T>]>::next` |

## P2 (partial): the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.97% | 39,382,587,871 | 1 | `temper_real_loop_world::cluster::run_cluster_forge` |
| 99.97% | 39,382,579,241 | 1 | `temper_real_loop_world::cluster::run_with` |
| 99.96% | 39,380,640,301 | 1 | `temper_real_loop_world::cluster::run_cluster_forge::{{closure}}` |
| 99.96% | 39,380,640,270 | 1 | `temper_real_loop_world::world::run_cluster_forge` |
| 95.43% | 37,595,971,345 | 1 | `skein_world::real::World<P,R>::run` |
| 95.42% | 37,592,276,667 | 1 | `skein_world::real::Running<P,R>::run` |
| 76.24% | 30,033,570,208 | 25,172 | `skein_world::real::Running<P,R>::drive` |
| 66.59% | 26,234,894,015 | 208,323 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 58.03% | 22,859,530,808 | 208,323 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 57.46% | 22,634,727,707 | 208,323 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 25.09% | 9,884,009,884 | 25,217 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 22.77% | 8,969,472,209 | 25,217 | `temper_service::system::iterate` |
| 22.73% | 8,954,215,924 | 126,085 | `temper_service::system::iterate_stage` |
| 13.12% | 5,169,161,939 | 23,519 | `temper_service::engine::iterate` |
| 12.57% | 4,951,784,131 | 25,172 | `<temper_real_loop_world::world::Judge as skein_world::referee::Referee<temper_real_loop_world::world::Proc>>::act` |

## P2 (partial): the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 4.51% | 1,775,134,515 | `__memcpy_avx_unaligned_erms` |
| 3.16% | 1,245,335,848 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.26% | 891,295,379 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 2.19% | 863,684,420 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,Type>>::find_key_index` |
| 2.07% | 815,890,210 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.79% | 706,153,150 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::find_map` |
| 1.49% | 588,026,678 | `core::array::iter::iter_inner::PolymorphicIter<[core::mem::maybe_uninit::MaybeUninit<T>]>::next` |
| 1.46% | 574,165,609 | `std::thread::local::LocalKey<T>::try_with` |
| 1.44% | 568,054,962 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.42% | 558,446,293 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.32% | 519,108,816 | `core::ops::function::FnOnce::call_once'2` |
| 1.16% | 458,433,891 | `alloc::collections::btree::search::<impl alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>>::search_tree` |

## G1: the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.09% | 14,516,676,054 | 1 | `temper_real_loop_world::run_github` |
| 99.09% | 14,516,654,179 | 1 | `temper_real_loop_world::world::run_github` |
| 94.53% | 13,848,409,237 | 1 | `skein_world::real::World<P,R>::run` |
| 94.51% | 13,845,825,230 | 1 | `skein_world::real::Running<P,R>::run` |
| 86.16% | 12,622,441,185 | 8,764 | `skein_world::real::Running<P,R>::drive` |
| 78.98% | 11,570,583,969 | 46,966 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 73.16% | 10,717,410,240 | 46,966 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 72.78% | 10,661,589,002 | 46,966 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 26.88% | 3,937,929,764 | 8,236 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 22.10% | 3,238,275,802 | 8,236 | `temper_service::system::iterate` |
| 22.07% | 3,233,293,022 | 41,180 | `temper_service::system::iterate_stage` |
| 12.56% | 1,839,551,443 | 2,062 | `<temper_real_loop_world::calls::recorded::Recorded as skein_shell::drive::Host>::iterate` |
| 12.55% | 1,838,589,709 | 2,062 | `<temper_client_shell_cli::host::OneShot as skein_shell::drive::Host>::iterate` |
| 12.42% | 1,819,091,826 | 8,236 | `temper_service::engine::iterate` |
| 10.03% | 1,468,946,544 | 468 | `skein_json::parse::parse` |

## G1: the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 4.32% | 633,197,736 | `__memcpy_avx_unaligned_erms` |
| 2.91% | 426,252,378 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.36% | 345,770,702 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.89% | 277,318,103 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.83% | 268,335,227 | `skein_json::writer::escaped_len` |
| 1.80% | 263,073,536 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.59% | 233,228,078 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.59% | 233,205,883 | `skein_json::writer::escaped` |
| 1.59% | 232,396,436 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.52% | 222,855,672 | `std::thread::local::LocalKey<T>::try_with` |
| 1.36% | 199,266,146 | `core::iter::traits::iterator::Iterator::try_fold'2` |
| 1.35% | 198,086,645 | `core::ops::function::FnOnce::call_once'2` |

## G2: the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.17% | 16,299,826,061 | 1 | `temper_real_loop_world::run_github` |
| 99.17% | 16,299,804,186 | 1 | `temper_real_loop_world::world::run_github` |
| 95.10% | 15,631,559,244 | 1 | `skein_world::real::World<P,R>::run` |
| 95.08% | 15,628,975,237 | 1 | `skein_world::real::Running<P,R>::run` |
| 84.68% | 13,918,521,350 | 9,226 | `skein_world::real::Running<P,R>::drive` |
| 78.07% | 12,832,868,472 | 49,432 | `skein_world::real::Running<P,R>::drive::{{closure}}` |
| 72.59% | 11,932,291,093 | 49,432 | `<temper_real_loop_world::world::Proc as skein_shell::drive::Host>::iterate` |
| 72.24% | 11,873,639,128 | 49,432 | `<temper_real_loop_world::world::Kind as skein_shell::drive::Host>::iterate` |
| 25.14% | 4,132,404,984 | 8,703 | `<temper_real_loop_world::service::ServiceHost as skein_shell::drive::Host>::iterate` |
| 20.88% | 3,432,247,556 | 8,703 | `temper_service::system::iterate` |
| 20.85% | 3,426,982,241 | 43,515 | `temper_service::system::iterate_stage` |
| 11.82% | 1,942,641,215 | 2,102 | `<temper_real_loop_world::calls::recorded::Recorded as skein_shell::drive::Host>::iterate` |
| 11.81% | 1,941,662,677 | 2,102 | `<temper_client_shell_cli::host::OneShot as skein_shell::drive::Host>::iterate` |
| 11.65% | 1,914,257,985 | 8,703 | `temper_service::engine::iterate` |
| 11.03% | 1,812,558,707 | 1,604 | `temper_real_loop_world::terminal::Typist::ready` |

## G2: the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 4.07% | 668,446,835 | `__memcpy_avx_unaligned_erms` |
| 2.81% | 461,387,873 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.74% | 450,359,567 | `<core::slice::iter::Windows<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.39% | 392,986,701 | `core::iter::traits::iterator::Iterator::try_fold'2` |
| 2.20% | 362,319,854 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 1.91% | 313,699,711 | `skein_json::writer::escaped_len` |
| 1.84% | 302,238,065 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 1.70% | 279,322,823 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.64% | 270,286,875 | `skein_json::writer::escaped` |
| 1.48% | 244,056,878 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 1.43% | 235,773,044 | `std::thread::local::LocalKey<T>::try_with` |
| 1.28% | 210,005,986 | `__memcmp_avx2_movbe` |

## C1: the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.94% | 41,360,124,803 | 79 | `temper_conformance_world::stories::run` |
| 85.34% | 35,320,500,135 | 4,615 | `jig_conformance::harness::Harness<A>::drain` |
| 85.34% | 35,320,449,370 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from` |
| 85.34% | 35,318,875,655 | 4,615 | `jig_conformance::harness::Harness<A>::drain_from::{{closure}}` |
| 85.34% | 35,318,811,045 | 4,615 | `jig_conformance::harness::Harness<A>::drain_iterations` |
| 85.31% | 35,307,745,684 | 43,157 | `jig_conformance::harness::Harness<A>::iterate` |
| 65.80% | 27,231,996,797 | 79 | `temper_conformance_world::stories::positive_change` |
| 44.59% | 18,453,085,444 | 155,975 | `jig_conformance::harness::Harness<A>::inspect` |
| 44.58% | 18,449,965,944 | 155,975 | `jig_conformance::harness::Harness<A>::inspect_outside` |
| 26.92% | 11,139,208,463 | 483 | `temper_conformance_world::stories::wait` |
| 25.97% | 10,749,210,358 | 79 | `temper_conformance_world::scenarios::change` |
| 25.75% | 10,657,846,402 | 155,975 | `<temper_conformance_world::Temper as jig_conformance::harness::Application>::observed` |
| 22.22% | 9,196,744,128 | 8,152,067 | `<skein_heap::Counting as core::alloc::global::GlobalAlloc>::alloc` |
| 21.02% | 8,697,333,495 | 6,942 | `temper_conformance_world::observations::snapshot` |
| 18.83% | 7,792,182,478 | 50,947 | `temper_engine_domain::domain::step` |

## C1: the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 6.07% | 2,510,458,060 | `__memcpy_avx_unaligned_erms` |
| 4.18% | 1,731,914,448 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
| 3.57% | 1,475,748,577 | `core::ptr::copy_nonoverlapping::precondition_check` |
| 3.39% | 1,403,951,090 | `std::thread::local::LocalKey<T>::try_with` |
| 2.96% | 1,226,420,472 | `core::ops::function::FnOnce::call_once'2` |
| 2.32% | 961,943,847 | `skein_heap::padded` |
| 2.04% | 844,834,528 | `<core::option::Option<T> as core::ops::try_trait::Try>::branch` |
| 2.01% | 831,816,052 | `std::thread::local::LocalKey<T>::initialize_with` |
| 1.90% | 786,317,166 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` |
| 1.85% | 766,294,251 | `core::alloc::layout::Layout::from_size_align` |
| 1.80% | 744,328,260 | `core::ptr::alignment::Alignment::new_unchecked::precondition_check` |
| 1.68% | 694,770,492 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_kv` |

## U1: the largest by inclusive Ir

Top 15 repository functions; libtest/thread/standard-library dispatch is omitted. World and story dispatch remain. Calls aggregate incoming edges with the same demangled symbol; inclusive costs include callees and are not additive.

| Share | Ir | Calls | Function |
|---|---|---|---|
| 99.94% | 18,502,716,896 | 1 | `temper_one_process_world::public::walk` |
| 99.94% | 18,502,716,743 | 2 | `temper_one_process_world::public::walk::{{closure}}` |
| 99.94% | 18,502,715,690 | 3 | `temper_one_process_world::public::service::run` |
| 99.93% | 18,501,397,676 | 3 | `temper_one_process_world::public::service::phase` |
| 99.71% | 18,461,095,892 | 3 | `temper_playground::programs::with` |
| 99.71% | 18,461,092,282 | 9 | `temper_one_process_world::public::service::phase::{{closure}}` |
| 97.38% | 18,029,327,103 | 3 | `skein_world::world::World<P,R,M>::run` |
| 97.38% | 18,029,326,503 | 3 | `skein_world::world::World<P,R,M>::run_with_faults` |
| 97.38% | 18,029,325,903 | 3 | `skein_world::world::World<P,R,M>::run_inner` |
| 89.27% | 16,528,158,282 | 23,298 | `skein_sim_world::world::World<P,M,H>::step_with` |
| 67.66% | 12,526,752,728 | 159,383 | `skein_sim_world::world::World<P,M,H>::turn` |
| 58.60% | 10,849,388,722 | 161,999 | `skein_sim_world::world::World<P,M,H>::turn::{{closure}}` |
| 57.95% | 10,728,799,995 | 159,386 | `<skein_world::heap::Metered as skein_sim_world::meter::Meter>::around` |
| 51.79% | 9,588,742,513 | 159,383 | `<temper_playground::programs::Process<H> as skein_shell::drive::Host>::iterate` |
| 20.70% | 3,832,389,155 | 159,386 | `<skein_world::world::Observing<R,F> as skein_sim_world::Turns<P>>::turned` |

## U1: the largest by self Ir

| Share | Ir | Function |
|---|---|---|
| 3.48% | 643,524,927 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_unchecked::{{closure}}` |
| 3.39% | 626,778,858 | `__memcpy_avx_unaligned_erms` |
| 3.15% | 583,062,626 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::Leaf>,alloc::collections::btree::node::marker::Edge>>::next_kv` |
| 3.00% | 555,139,762 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 2.80% | 518,993,564 | `alloc::collections::btree::navigate::<impl alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<BorrowType,K,V,alloc::collections::btree::node::marker::LeafOrInternal>,alloc::collections::btree::node::marker::KV>>::next_leaf_edge` |
| 2.61% | 482,821,319 | `skein_json::writer::escaped_len` |
| 2.29% | 423,924,206 | `alloc::collections::btree::node::Handle<alloc::collections::btree::node::NodeRef<alloc::collections::btree::node::marker::Immut,K,V,NodeType>,alloc::collections::btree::node::marker::KV>::into_kv` |
| 2.29% | 423,245,247 | `alloc::collections::btree::navigate::LazyLeafRange<BorrowType,K,V>::init_front` |
| 2.27% | 420,918,083 | `core::iter::traits::iterator::Iterator::try_fold` |
| 2.18% | 403,653,496 | `skein_json::writer::escaped` |
| 2.09% | 386,168,395 | `<alloc::collections::btree::map::Iter<K,V> as core::iter::traits::iterator::Iterator>::next` |
| 1.91% | 353,281,871 | `alloc::collections::btree::mem::replace` |

## Left

- R1/R2/P2: forge observation-cap assertion under Callgrind (`skein/crates/skein-fake-peers/src/forge/observe.rs`, fixture cap in `temper/tests/real-loop/src/forge.rs`). Partial profiles retained as authorized; `engine-causes` / `real-loop-causes` own the code. Native runs on unchanged source passed.
- P1: ending-attempt oracle assertion (`temper/tests/real-loop/src/case/oracle.rs:555`), owned by `real-loop-causes`; its native runs passed. This task changes figures only.
- Every ranked cause remains for its named owner. No product, harness, observation limit, timeout, budget or stored format was changed. Native timings are isolated stories, not full-suite CPU totals; no whole-suite budget pass is claimed.
