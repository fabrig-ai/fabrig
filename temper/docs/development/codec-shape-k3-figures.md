# K3: lean tasks records, measured

2026-11-19. Source, compile and decode figures for jig's tasks store family, for `temper/docs/design/draft/codec-shape.md`, sections 2 and 3.3. **The line criterion holds. The compile criterion is unproven on builder elapsed time: the two store-build spreads overlap. User+system CPU is not available through the build wrapper.** The isolated tasks build and clippy are faster beyond their spreads; the selected decode-test medians are slower. These figures do not establish the whole 3.3 gate for K4.

## 1. Trees, environment and method

- **S, sealed:** `80a02a1f87e290afdcd262eb0e085a07cad35a17`.
- **L, lean:** `c145dbb87d206a8aabb560f4c214fd53ef985a01`, the head of change 8787 supplied to this measurement, not landed when measurement began. No landing merge was then available whose first parent could be used; S is that change's base instead.
- Both revisions were checked out in separate temporary worktrees under `/tmp`, in the same measurement session. Both had empty `git status --short --untracked-files=all` and clean diffs after the measurements; both worktrees were removed. No code, schema, test, tool or budget change is retained.
- The controlling shell's machine exposed 8 cores / logical CPUs, AMD EPYC-Genoa, one thread per core; its hostname is omitted to keep the repository's outward-name boundary. **Builder identity is unknown:** cargo's output and timing reports do not name its machine. Every measured compile timing report says `jobs=4 ncpu=8`. Same-builder affinity cannot be verified through the wrapper, so these are the nearest available paired measurements, not a controlled same-machine CPU experiment.
- One cargo invocation at a time was launched by this measurement. Whether other work ran on the builders is unknown; exclusive use was not established. Ops' stated cap was at most two check runs per builder, four CPUs each. The third sealed mutated-corpus invocation reported package-cache lock waits before the test.
- Every build and test used `cargo` on PATH, with `TEMPER_INTERNAL_BUNDLE=none` and `TEMPER_BUNDLE=none`. Compile reports identify dev profile, unoptimized + debuginfo, and `rustc 1.101.0-nightly (19833af72 2026-10-20)`, x86_64-unknown-linux-gnu. Nextest used the test profile, also unoptimized + debuginfo.
- Each scenario alternated S, L, S, L, S, L. All samples are retained below; spread means observed minimum–maximum, not a confidence interval. No run was discarded or repeated to narrow its spread.

**CPU limitation.** Local process accounting measures the cargo wrapper, not the builder's compiler tree. An exploratory invocation reported Cargo elapsed 0.84 s while local elapsed was 78.17 s and local child CPU was 0.539 user + 0.263 system s. Those local numbers are not used in the comparison. Builder user+system CPU is **not measured** for any compile scenario. The nearest figures are Cargo's own `Finished … in …` elapsed seconds and its `--timings` unit durations. These are elapsed times, not CPU seconds.

## 2. Lines

File on both sides: `jig/crates/jig-protocol-store-tasks/src/generated/tasks.rs`. Literal newline counts, including comments and blanks: **S 46,328; L 18,090**. The measured sealed count is four lines more than the brief's post-K2 reference of 46,324; the measured 46,328 is the second denominator below.

| Baseline | Baseline lines | Lean lines | Lean / baseline | Reduction | At most half? |
|---|---:|---:|---:|---:|---|
| Survey, the draft's “today” (`codec-shape.md`, 2) | 42,813 | 18,090 | 0.4225 (42.25%) | 57.75% | yes |
| Post-K2 sealed tree, measured S (reference was 46,324) | 46,328 | 18,090 | 0.3905 (39.05%) | 60.95% | yes |

Lean parts partition the entire file. Each section includes its marker, attributes and following blank lines up to the next section marker. The 333 `// Type:` and 333 `// Record:` blocks alternate from line 5,016 to EOF; count each block up to the next marker.

| Lean part | Lines | Counting boundary / contents |
|---|---:|---|
| Header, imports, version | 9 | 1–9 |
| Limits | 525 | 10–534: `Limits`, `CEILINGS`, `LIMIT_NAMES`, `values`, `check_limits` |
| Paths | 2,864 | 535–3,398: path constants, name table and `name` |
| Bounds table and family wrappers | 1,617 | 3,399–5,015: `TABLE`, `DECLARATIONS`, `CEILING_VALUES`, family encoded/decoded bound functions |
| Types | 2,841 | all `// Type:` blocks, public structs/enums |
| `Record` implementations | 10,234 | all `// Record:` blocks, including declaration-bound wrappers and read/write |
| **Total** | **18,090** | no unassigned lines |

These are generated-module lines, not the shared helper implementation's lines or machine-code size.

## 3. Compile: builder elapsed, not CPU

Dependencies were built before sampling: each tree first built `jig-protocol-store`; each tree's clean store build was warmed again before B, and clippy was warmed before C. Those warm-ups are excluded.

Touching can be lost at a content-syncing wrapper boundary. To guarantee a changed input, each sample temporarily appended one trailing comment, `// K3 compile measurement <scenario> <iteration>.`, identical on both sides for that sample, and restored the exact original bytes immediately afterward. The comment is a rebuild stimulus, not a schema or behaviour change. Each A sample's output said `Compiling jig-protocol-store-tasks`, each B sample said `Compiling jig-protocol-store-records` but not tasks, and each C sample said `Checking jig-protocol-store-tasks`.

Commands (all with `--timings`):

| Scenario | Temporarily changed file | Command |
|---|---|---|
| A-store, schema proxy through store | tasks generated module above | `cargo build -p jig-protocol-store --timings` |
| A-tasks, schema proxy alone | same module | `cargo build -p jig-protocol-store-tasks --timings` |
| B, no schema change | `jig/crates/jig-protocol-store-records/src/codec/tasks/value.rs`, same path both sides | `cargo build -p jig-protocol-store --timings` |
| C, clippy | tasks generated module | `cargo clippy -p jig-protocol-store-tasks --timings` |

### 3.1 Whole-command elapsed

Seconds from Cargo's `Finished` line; three samples per side. CPU user+system is **not measured** in all four rows.

| Scenario | Sealed samples | Sealed median [spread] | Lean samples | Lean median [spread] | Comparison beyond spread |
|---|---|---|---|---|---|
| A-store | 8.86, 8.49, 9.25 | 8.86 [8.49–9.25] | 7.41, 6.60, 8.81 | 7.41 [6.60–8.81] | unproven: overlap |
| A-tasks | 5.25, 4.68, 5.02 | 5.02 [4.68–5.25] | 3.21, 3.26, 3.56 | 3.26 [3.21–3.56] | holds on builder elapsed time |
| B | 4.81, 4.63, 5.09 | 4.81 [4.63–5.09] | 4.95, 5.08, 4.82 | 4.95 [4.82–5.08] | unproven: overlap |
| C | 4.01, 3.90, 4.12 | 4.01 [3.90–4.12] | 2.56, 2.18, 2.57 | 2.56 [2.18–2.57] | holds on builder elapsed time |

A-store's median falls by 16.4%, A-tasks' by 35.1%, C's by 36.2%; B's rises by 2.9%, inside the observed spread. A lower median alone does not establish a gain beyond overlapping ranges.

### 3.2 Rebuilt units

Elapsed seconds from the `UNIT_DATA` in Cargo's timing reports, median [minimum–maximum]. `tasks`, `records`, `store` abbreviate `jig-protocol-store-tasks`, `jig-protocol-store-records`, `jig-protocol-store`. A dash means fresh/not rebuilt, not a zero-duration compilation. Pipelined units can overlap; their durations must not be summed as command elapsed or CPU.

| Scenario | Unit | Sealed | Lean |
|---|---|---:|---:|
| A-store | tasks | 4.39 [4.35–4.57] | 2.89 [2.81–3.46] |
| A-store | records | 1.86 [1.86–2.49] | 2.02 [1.74–2.26] |
| A-store | store | 3.27 [2.98–3.45] | 3.29 [2.96–4.33] |
| A-tasks | tasks | 4.77 [4.40–4.94] | 2.95 [2.87–3.17] |
| B | tasks | — | — |
| B | records | 1.84 [1.77–1.91] | 1.78 [1.59–1.86] |
| B | store | 3.09 [3.04–3.16] | 3.26 [3.23–3.51] |
| C | tasks | 3.75 [3.63–3.85] | 2.24 [1.85–2.25] |

### 3.3 Run times and provenance

All dates below are **2026-11-19 UTC**, from each timing report's builder-side `Build start`, rounded here to milliseconds. Builder name: **unknown for every run**. Each row is one S/L pair; pairs ran in row order within each scenario. Concurrency was one unit for A-tasks/C, two for A-store/B; all reports gave `jobs=4 ncpu=8`.

| Scenario | Pair | Sealed start | Lean start |
|---|---:|---|---|
| A-store | 1 | 04:15:20.349 | 04:15:29.428 |
| A-store | 2 | 04:16:04.977 | 04:16:13.674 |
| A-store | 3 | 04:16:20.517 | 04:16:30.054 |
| A-tasks | 1 | 04:17:02.706 | 04:17:08.221 |
| A-tasks | 2 | 04:17:11.659 | 04:17:16.558 |
| A-tasks | 3 | 04:17:20.098 | 04:17:25.351 |
| B | 1 | 04:17:31.722 | 04:17:36.751 |
| B | 2 | 04:17:41.917 | 04:17:46.775 |
| B | 3 | 04:17:52.122 | 04:17:57.430 |
| C | 1 | 04:18:18.751 | 04:18:23.002 |
| C | 2 | 04:18:25.823 | 04:18:30.019 |
| C | 3 | 04:18:38.457 | 04:18:42.875 |

## 4. Decode-test times

Each test ran alone three times per tree, alternating S/L. All 24 invocations selected exactly one test in `jig-store-codecs-world::suite`; all 24 passed, each skipping the other 44. The times below are the individual nextest **PASS** durations, not its summary, Cargo build duration, or local wrapper elapsed time.

Command pattern:

```sh
cargo nextest run -p jig-store-codecs-world --test suite \
  --profile fuzzy -E 'test(=fuzzy::decode::NAME)'
cargo nextest run -p jig-store-codecs-world --test suite \
  -E 'test(=memory::NAME)'
```

Names, under `jig/tests/store-codecs/tests/suite/`:

- **Arbitrary:** `fuzzy::decode::arbitrary_tasks_rows_never_panic_and_round_trip_when_they_decode`.
- **Mutated:** `fuzzy::decode::mutated_tasks_fixed_rule_values_never_panic`.
- **Full row:** `memory::a_full_tasks_row_decodes_within_its_decoded_bound`.
- **Additions:** `memory::added_dependencies_kept_lines_and_circle_payloads_fit_their_heap_bounds`.

| Test | Sealed samples, s | Sealed median [spread], s | Lean samples, s | Lean median [spread], s | Lean / sealed medians |
|---|---|---|---|---|---:|
| Arbitrary | 0.018, 0.027, 0.018 | 0.018 [0.018–0.027] | 0.040, 0.074, 0.073 | 0.073 [0.040–0.074] | 4.06 |
| Mutated | 0.127, 0.045, 0.047 | 0.047 [0.045–0.127] | 0.188, 0.071, 0.070 | 0.071 [0.070–0.188] | 1.51 |
| Full row | 0.025, 0.027, 0.026 | 0.026 [0.025–0.027] | 0.045, 0.044, 0.044 | 0.044 [0.044–0.045] | 1.69 |
| Additions | 0.076, 0.064, 0.064 | 0.064 [0.064–0.076] | 0.086, 0.092, 0.104 | 0.092 [0.086–0.104] | 1.44 |

The arbitrary test feeds 10,000 seeded byte strings, each at most 128 bytes, to the row decoder and round-trips successful decodes. The mutated test constructs the schema's full fixed-rule fixtures and makes 100 seeded mutations per fixture. The memory tests construct full fixed-rule values at run time: `Row`, and separately `TaskDefinition`, `KeptLine`, `Circle`. They check decoded heap bounds and released allocations. These are the two tasks tests in `memory.rs`, not a newly invented largest-value benchmark.

Nextest measures whole tests: schema parsing, fixture generation, bounds calculation, allocation checks and round-trip encoding where present are included, not just decode calls. These figures therefore establish selected-test runtime, not decoder-only throughput. All four lean medians are higher; arbitrary, full-row and additions ranges are wholly above sealed, while mutated ranges overlap. Section 3.3 asks to measure decode runtime but states no decoder-runtime pass threshold.

Nextest output does not name builder hosts or test start timestamps. For reproducibility, these are the **controlling shell's invocation starts**, not builder starts, on 2026-11-19 UTC (three runs in order):

| Test | Sealed local starts | Lean local starts |
|---|---|---|
| Arbitrary | 04:18:56.194, 04:21:06.060, 04:21:27.572 | 04:20:42.976, 04:21:25.270, 04:21:28.698 |
| Mutated | 04:21:29.975, 04:21:32.399, 04:21:37.376 | 04:21:31.144, 04:21:36.420, 04:21:38.363 |
| Full row | 04:21:39.338, 04:21:41.245, 04:21:43.243 | 04:21:40.272, 04:21:42.191, 04:21:44.190 |
| Additions | 04:21:45.153, 04:21:47.168, 04:21:49.252 | 04:21:46.149, 04:21:48.145, 04:21:50.308 |

## 5. Verdict for K4

- **Lines: holds.** 18,090 is 42.25% of the survey's 42,813, and 39.05% of the measured post-K2 sealed 46,328. The survey denominator, not the later larger file, decides the half-size criterion.
- **Compile: unproven on builder elapsed time; user+system CPU not available through the build wrapper.** Isolated tasks build and clippy hold on builder elapsed time, with disjoint faster ranges. Both the schema-through-store and no-schema-through-store comparisons have overlapping ranges, so neither proves the required non-regression beyond spread. No whole-command compile regression beyond spread is established either. The B store unit alone is slower beyond its range, as the unit table shows.
- **Decode: measured, all tests pass, all four medians slower.** No runtime threshold was supplied; this is evidence for K4's decision, not grounds to hide the regression or invent a new threshold.

Thus the source-size condition is demonstrated, but these nearest available measurements do **not** demonstrate that all of 3.3 holds. K4 must not treat this report as a proven full compile gate. Byte/fingerprint equivalence is outside this measurement-only change; these four existing tests are not a replacement for the conversion's drift and `every` checks.
