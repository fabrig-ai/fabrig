# Hearth focused-suite figures

## Before

Base: `483dfed6aa8639dc7df93a4e381c6ae2458c4468`.
Machine: `debian-32gb-hel1-14` (`hostname` through heavy).
Date: 2026-11-06 UTC; build and measurements between 09:16:37 and 09:17:37
(`date -u`). These are builder measurements, not the queue's figures.

Method: `hearth/docs/plan/hearth-speed/notes/common.md`, “The method”, using
`tools/tiers/design.md`, 7.3 and 10.2. A detached checkout of the base was
built first. `cargo metadata --no-deps --format-version 1` selected every
workspace member whose manifest is under `hearth/`: 34 packages. A short
untracked driver, `.figures-before.py`, loaded `tools/tiers/cpu.py` with
`runpy.run_path`, used `prepare` with a fresh directory for each run, and
called `execute` and `summary`. It built the static wrapper once with
`heavy cc -std=c11 -O2 -static -Wall -Wextra -Werror` from
`tools/tiers/cpu-wrapper.c`. Both runs used:

```text
NEXTEST_DOUBLE_SPAWN=0 TEMPER_BUNDLE=none
heavy cargo nextest run --profile default -p <each of the 34 packages> <prepare arguments>
```

The preceding build used the same package list and profile with `--no-run`.
There was no test filter or concurrency override. CPU is user plus system
for the test and its waited/reaped descendants; wrapper CPU, compilation,
listing and nextest are excluded. Wall spans the first test start through
final wrapper settlement, rather than summing overlapping tests' elapsed
times. The two collector directories were retained through aggregation and
removed with the throwaway driver before pushing.

| Run | User s | System s | CPU s | Wall s | Tests |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 26.709478 | 6.318832 | 33.028310 | 3.259001 | 2,189 |
| 2 | 27.496458 | 6.649544 | 34.146002 | 3.209826 | 2,189 |

All 2,189 tests passed in each run: 2,189 attempts, no retries, failures,
starvation, unobserved attempts, leaks or incomplete records. Run 1 is the
lower CPU figure and supplies the rankings, shares and floors below; run 2
is 3.38% higher. Both exceed the existing 30 CPU s budget
(`hearth/docs/design/domain/testing.md`, 9.1): by 3.028310 and 4.146002 s.
This task records that existing departure and changes no code or budget.
The plan's same-machine 85% target would be 28.074064 CPU s against run 1.
The earlier 41.678860 builder figure and 31.7 queue figure are contextual
records, not interchangeable baselines; their difference from these runs
is unknown.

### By family

| Family | Tests | Run 1 CPU s | Share of run 1 | Run 2 CPU s |
| --- | ---: | ---: | ---: | ---: |
| Builder suite | 106 | 9.496237 | 28.75% | 9.760482 |
| Tree suite | 54 | 8.033831 | 24.32% | 8.168588 |
| Real git | 183 | 6.852550 | 20.75% | 7.106626 |
| Render suite | 109 | 3.110188 | 9.42% | 3.236249 |
| All the rest | 1,737 | 5.535504 | 16.76% | 5.874057 |

The four named families contain 452 tests (20.65% of the tests) and consume
83.24% of CPU. Shares are of total test CPU, not sampled self time.

### By binary

IDs without `::` are nextest's library test binaries. Binaries with no
selected tests contribute no CPU and are omitted.

| Nextest binary ID | Tests | Run 1 CPU s | Run 2 CPU s |
| --- | ---: | ---: | ---: |
| `hearth` | 15 | 0.023214 | 0.030764 |
| `hearth-accounts-world::suite` | 1 | 0.008530 | 0.011262 |
| `hearth-builder-world::real` | 183 | 6.852550 | 7.106626 |
| `hearth-builder-world::suite` | 106 | 9.496237 | 9.760482 |
| `hearth-client-protocol-records` | 154 | 0.264334 | 0.289618 |
| `hearth-config-world::suite` | 22 | 0.566553 | 0.613738 |
| `hearth-conformance-world::suite` | 3 | 0.004428 | 0.006745 |
| `hearth-domain-build` | 7 | 0.011357 | 0.012309 |
| `hearth-domain-builder` | 17 | 0.039432 | 0.038155 |
| `hearth-domain-render` | 619 | 1.534531 | 1.640351 |
| `hearth-domain-store` | 55 | 0.091285 | 0.097019 |
| `hearth-domain-tree` | 53 | 0.094011 | 0.093261 |
| `hearth-fake-git` | 25 | 0.048935 | 0.046981 |
| `hearth-fake-machine` | 74 | 0.136616 | 0.134131 |
| `hearth-fake-mmdebstrap` | 27 | 0.069002 | 0.064325 |
| `hearth-git-world::suite` | 20 | 0.368793 | 0.401134 |
| `hearth-lib` | 24 | 0.037801 | 0.037120 |
| `hearth-mmdebstrap-world::suite` | 23 | 0.579741 | 0.612821 |
| `hearth-protocol-accounts` | 21 | 0.109593 | 0.111128 |
| `hearth-protocol-builder` | 195 | 0.376488 | 0.386527 |
| `hearth-protocol-config` | 88 | 0.217072 | 0.235663 |
| `hearth-protocol-git` | 35 | 0.055474 | 0.063047 |
| `hearth-protocol-mmdebstrap` | 51 | 0.082720 | 0.091701 |
| `hearth-protocol-template` | 86 | 0.150557 | 0.165772 |
| `hearth-protocol-tree` | 75 | 0.135803 | 0.144894 |
| `hearth-recordings` | 5 | 0.009965 | 0.009705 |
| `hearth-render-world::suite` | 109 | 3.110188 | 3.236249 |
| `hearth-service-builder` | 8 | 0.025032 | 0.019074 |
| `hearth-template-world::suite` | 9 | 0.220996 | 0.219545 |
| `hearth-test-mirror` | 19 | 0.045416 | 0.044551 |
| `hearth-tree-world::suite` | 54 | 8.033831 | 8.168588 |
| `hearth::end_to_end` | 4 | 0.194126 | 0.207402 |
| `hearth::refusals` | 2 | 0.033699 | 0.045314 |

### Twenty most costly tests

Ranked by run 1 total CPU, including descendants. Binary aliases are B =
`hearth-builder-world::suite`, T = `hearth-tree-world::suite`.

| Rank | Binary | Test | CPU s |
| --- | --- | --- | ---: |
| 1 | T | `memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | 1.025999 |
| 2 | T | `walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | 0.782220 |
| 3 | B | `domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.714251 |
| 4 | B | `domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.710272 |
| 5 | B | `domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.694944 |
| 6 | T | `walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | 0.649863 |
| 7 | T | `hash::wide_and_deep_layers_hash_to_the_oracles` | 0.610796 |
| 8 | T | `memory::walk::walking_generated_layers_against_a_parent_holds_no_more_than_its_ceiling` | 0.518518 |
| 9 | B | `domain::a_second_build_of_the_same_spec_stages_and_runs_nothing` | 0.451076 |
| 10 | T | `walk::generated_edits_walk_to_the_oracles_digests_and_file` | 0.427494 |
| 11 | B | `goldens::each_error_case_prints_its_errors_after_the_healthy_spec` | 0.422682 |
| 12 | B | `domain::a_build_from_an_empty_store_publishes_the_empty_packages_software_and_config_layers` | 0.376966 |
| 13 | T | `walk::wide_and_deep_layers_walk_against_themselves_and_their_edits` | 0.367050 |
| 14 | T | `hash::generated_layers_hash_to_the_oracles_digests_and_file` | 0.355727 |
| 15 | T | `trees::a_change_for_another_path_is_refused` | 0.338907 |
| 16 | B | `properties::a_production_edit_leaves_the_lab_specs_through_real_git` | 0.328929 |
| 17 | T | `walk::a_walk_with_nothing_changed_is_no_change_with_the_parents_tree_digest_and_nothing_hashed` | 0.317632 |
| 18 | B | `protocol::a_full_pipe_holds_its_writer_until_read` | 0.293491 |
| 19 | T | `hash::each_bound_fails_at_its_path` | 0.289704 |
| 20 | T | `memory::walk::walking_a_layer_at_its_limits_against_a_parent_holds_no_more_than_its_ceiling` | 0.254713 |

Together these twenty tests cost 9.931234 CPU s, 30.07% of the suite.

### Floor estimate

Each row takes the cheapest test in that binary in run 1, without mixing
per-test minima across runs. These tests still do work: this is a startup
floor proxy, not an isolated measurement of empty-process overhead.

| Binary | Cheapest test | CPU s | Tests represented | Product CPU s |
| --- | --- | ---: | ---: | ---: |
| `hearth-builder-world::suite` | `goldens::errors_are_sorted_by_path_line_column_and_target` | 0.002354 | 106 | 0.249524 |
| `hearth-tree-world::suite` | `memory::the_bounds_are_none_past_the_encoders_depth` | 0.001593 | 54 | 0.086022 |
| `hearth-builder-world::real` | `a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` | 0.003360 | 183 | 0.614880 |
| `hearth-render-world::suite` | `golden_harness::a_spec_text_is_cut_into_its_targets_at_each_target_line` | 0.002120 | 109 | 0.231080 |
| `hearth-lib` | `tests::encode::a_value_sixty_four_containers_deep_encodes` | 0.001415 | 1,737 | 2.457855 |

The four family minima, applied within their own binaries, sum to 1.181506
CPU s over 452 tests. Applying the `hearth-lib --lib` minimum to the other
1,737 tests adds 2.457855 CPU s: **3.639361 CPU s, 11.02% of the suite**.
For comparison, applying that library minimum uniformly to all 2,189 tests
comes to 3.097435 CPU s (9.38%). Neither extrapolation establishes a saving.
The real-git minimum exercises a missing executable and expects a panic;
it does not include successful repository setup. The library minimum runs
64 encoder list heads and checks the measured size. The other minima check
golden text, decoder bounds, and target splitting. The loader,
allocator, harness and each test's own work remain combined in these figures.

## Profile

Profiling availability was checked on the builder through
`heavy bash -lc 'hostname; date -u; command -v perf; command -v valgrind; command -v cc; pwd'`.
Only `cc` was found; neither `perf` nor `valgrind` is on PATH. Per this task's
explicit fallback, nothing was installed and no substitute profiler was
introduced. There are no samples or callgrind results. An unavailable share
is **unknown**, not zero.

| Family | Crate shares of self time | Kernel / libc self time | Ten hottest functions |
| --- | --- | --- | --- |
| Builder suite | unknown | unknown / unknown | unknown |
| Tree suite | unknown | unknown / unknown | unknown |
| Real git | unknown | unknown / unknown | unknown |
| Render suite | unknown | unknown / unknown | unknown |

The intended crate grouping is the first segment of each demangled symbol,
with kernel and libc separate. Without samples, no such grouping can be
computed. For real git, the split between `git` processes and the test process
is **unknown**; test-process CPU before the first git invocation for the code
under test is also **unknown**. The collector's per-test records combine those
processes and have no function or child-exec timeline, so its 6.852550 CPU s
cannot supply either split.

## What it found

The timing establishes where costs sit. Source inspection identifies repeated
work, but its individual CPU share is unknown without a profiler. The enclosing
test costs below are measured scopes, not attributed costs or predicted savings.

- Tree parent construction repeats for the same seed inside the edit loop in
  `hearth/tests/tree/tests/suite/walk.rs`'s
  `each_edit_alone_walks_to_the_oracles_digests_and_file`: each edit recreates
  `Rng::new(0x7EE0_8300 + seed)` and `layers::layer(..., 30 + seed)` for each
  of ten seeds. The parent therefore repeats across edits at a given seed.
  The whole test costs 0.782220 CPU s, 9.74% of the tree family and 2.37% of
  the suite. Parent-generation self time and its share are unknown.
- Builder tests repeat baseline rendering while checking builds. In
  `hearth/tests/builder/tests/suite/domain.rs`,
  `a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer`
  calls `rendered_keys(seed)` inside `each_settled` after the initial build,
  before editing config. The whole test costs 0.710272 CPU s, 7.48% of the
  builder family and 2.15% of the suite. The share of repeated rendering
  itself is unknown; this figure includes the initial and restarted builds.
- Process and harness work recurs per test: the family-specific minimum proxy
  totals 3.639361 CPU s across the suite (11.02%), including real work in the
  minimum tests. Real-git's 183 tests cost 6.852550 CPU s (20.75%), averaging
  0.037446 CPU s each. How much is startup, repository construction, git or
  the code under test is unknown. No production inefficiency or repeated-work
  self-time share is established by this measurement.

The largest single test is the tree wide/deep memory walk, 1.025999 CPU s
(12.77% of the tree family, 3.11% overall); its source builds one 2,000-wide,
12-deep layer and checks two limit settings. Its setup and walking shares
remain unknown. Decisions about changes belong to the later tasks.
