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


## 6. K4: decoding alone

### 6.1 Trees, environment and method

2026-11-19. Base **A** is `4dccc50631531e27721aaba781a6f8d66e7fd477`,
after `60e03e20b2` (the decoded-list allocation change), including change
9879's `5bb267f7e24664eff21d6c59dd56cd144b2a930f` supplied before landing.
L is that tree's committed lean
`jig/crates/jig-protocol-store-tasks/src/generated/tasks.rs`. S is
`skein_codegen::generate_as(&schema, Form::Sealed).records` from the same
`jig/crates/jig-protocol-store/schema/tasks-v13.schema` and generator.

The coordinator's answer to question 213192 replaces separate S/L builds
with two modules in **one throwaway test process**, giving same-builder
alternation by construction without a builder-affinity control. The test
included L directly from its committed path and S as a second module;
no hand-written store readers were changed. Its schema parser, generator,
corpus construction, correctness checks and warm-up were outside timing.
The harness, sealed output and generator example are not retained.

The harness ran alone as `skein-codec-tests::k4_decode::decoding_alone`,
nextest's focused/default suite, once per measured profile and candidate:

```sh
cargo nextest run -p skein-codec-tests --test k4_decode \
  -E 'test(=decoding_alone)' --success-output immediate
# Optimized build: the same command with --cargo-profile release.
```

Each invocation selected and passed exactly one test, with zero skipped.
Cargo reported test profile unoptimized + debuginfo and release optimized;
the harness reported `debug_assertions=true` and `false` respectively,
`x86_64`, and 8 available logical CPUs. **Builder identity and CPU model
are unknown.** Each S/L pair ran on the same builder in the same process;
whether separate invocations used the same builder, and what other work
ran there, are unknown. Every command used cargo on PATH (heavy), one
at a time, with the deployment's no-bundle configuration. No local
wrapper elapsed time or compiler time enters these decode figures.

For each corpus, five rounds run S then L: S,L,S,L,S,L,S,L,S,L.
Each decoder gets an untimed pass over the corpus first. For every call,
`Reader::new` and black-boxed input/limits are prepared before
`Instant::now`; the timed expression is
`black_box(Row::decode(limits, &mut reader))`. Elapsed time is read
immediately afterward. Accumulating durations and dropping the returned
Result, including the decoded row's heap, are outside timing. Allocation
inside decode remains included. Clock-read and black-box overhead remain
in each observation; they are not subtracted, and matter especially for
early refusals in release. These are elapsed decode-call times, not CPU
times or whole-test durations. No sample is discarded; spread is the
observed minimum–maximum, not a confidence interval. Ratios below divide
medians, not the median of the five paired ratios.

The three prebuilt input sets are identical for S and L, all at CEILINGS:

| Corpus | Construction | Inputs | Bytes | Successful Row decodes | Repetitions per sample | Input digest |
|---|---|---:|---:|---:|---:|---|
| Fixed | All `skein_codegen::goldens`, sorted by name | 1,370 | 30,845 | 2 | 10 | `105c835c242c706c` |
| Mutated | The existing fuzzy test's sorted `_full.bin` fixtures, each original then 100 mutations, seed `0x5706_e014` | 69,185 | 2,009,799 | 83 | 1 | `9d3980b6912a33f7` |
| Full row | `record_row_full.bin`, the value the cited memory test decodes | 1 | 88 | 1 | 1,000 | `a7b25e8e7d46ed66` |

The mutated corpus uses `Rng::below(len)` to choose one byte and the low
byte of `next_u64` to replace it, skipping modification of empty inputs,
exactly as `fuzzy::decode::mutated_tasks_fixed_rule_values_never_panic`.
The cited `memory::a_full_tasks_row_decodes_within_its_decoded_bound` is
present in K3's `c145dbb87d206a8aabb560f4c214fd53ef985a01`, not this base's
`memory.rs`; its `check(values::TASKS, "row", ...)` picks precisely this
full fixed-rule golden. This does not substitute the newer largest-row
clamp test. Most Fixed/Mutated bytes describe declarations other than Row
and refuse at the version prefix, as in the existing fuzzy test. They
measure refusal throughput as well as successful decoding; Full row
isolates a successful decode.

The digest is FNV-1a-64 over each input's little-endian u64 length followed
by its bytes, in iteration order. Digests, counts, acceptance and bytes
matched in every measured invocation. Before timing, S and L agreed on
acceptance for every input, and successful rows re-encoded to each other
and to the original bytes. Encoding, equality and these checks are not
in the samples.

### 6.2 Samples and candidates

All samples below are **milliseconds**, each the sum of just the timed
calls for that corpus and its stated repetitions. Six decimal places
preserve the measured nanoseconds. Samples are in round order.

- **A:** unchanged base; both profiles establish the initial comparison.
- **B:** A plus only the body of `Record::decode`: return the read Result
  directly, with a tail check only on success, avoiding extraction and
  re-wrapping of the potentially large value in an unoptimized wrapper.
- **C:** B plus only the body of `field::version`: match the read u16
  directly, constructing Short/Version only on refusal, avoiding eager
  `path.problem`, `ok_or` and `?` layers. C was saved as
  `27d44a3278e29f482ea89c38490403c9d69e55d4` (unchecked).
- **D:** C plus forced inlining on `Record::decode`, `field::read`,
  `field::version`, `read_bytes` and the field shapes' `read` implementations.
  Question 213211's answer authorizes these attributes despite the
  body-only overlap restriction. No signature, byte, bound, refusal,
  schema, generated module or generator changes. B/C are diagnostic
  test-profile runs; release is measured on A and D.

**A test.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 3.075206, 1.446769, 1.437760, 1.435139, 1.452658 | 1.446769 [1.435139–3.075206] | 5.088138, 3.078187, 3.050638, 3.109108, 3.069189 | 3.078187 [3.050638–5.088138] | 2.1276 |
| Mutated | 7.229834, 7.279482, 7.228955, 7.237465, 7.361143 | 7.237465 [7.228955–7.361143] | 15.354976, 15.329331, 15.369279, 15.385970, 15.536042 | 15.369279 [15.329331–15.536042] | 2.1236 |
| Full row | 1.593269, 1.601069, 1.598669, 1.628478, 1.624548 | 1.601069 [1.593269–1.628478] | 2.014229, 1.991329, 1.999548, 2.036939, 2.033868 | 2.014229 [1.991329–2.036939] | 1.2581 |

**A release.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 2.331788, 1.092268, 0.992041, 1.170600, 1.171319 | 1.170600 [0.992041–2.331788] | 2.377748, 1.034938, 1.052639, 1.313908, 1.286699 | 1.286699 [1.034938–2.377748] | 1.0992 |
| Mutated | 5.743356, 4.855962, 4.970484, 4.984774, 4.889617 | 4.970484 [4.855962–5.743356] | 5.147494, 4.949687, 4.936468, 4.973906, 4.928575 | 4.949687 [4.928575–5.147494] | 0.9958 |
| Full row | 0.272819, 0.266650, 0.282740, 0.271610, 0.262450 | 0.271610 [0.262450–0.282740] | 0.214410, 0.217510, 0.210030, 0.213890, 0.211469 | 0.213890 [0.210030–0.217510] | 0.7875 |

**B test.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 1.801289, 1.498239, 1.476180, 1.506569, 1.477499 | 1.498239 [1.476180–1.801289] | 3.095419, 3.067426, 3.204110, 3.066286, 3.100987 | 3.095419 [3.066286–3.204110] | 2.0660 |
| Mutated | 7.625873, 7.746134, 7.286867, 7.769354, 8.582565 | 7.746134 [7.286867–8.582565] | 15.033491, 15.638620, 14.959221, 15.511872, 15.316589 | 15.316589 [14.959221–15.638620] | 1.9773 |
| Full row | 1.589300, 1.594248, 1.624199, 1.594118, 1.607759 | 1.594248 [1.589300–1.624199] | 1.857769, 1.869309, 1.849498, 1.844267, 1.880349 | 1.857769 [1.844267–1.880349] | 1.1653 |

**C test.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 1.693018, 1.687759, 1.529760, 1.540479, 1.518787 | 1.540479 [1.518787–1.693018] | 2.314558, 2.416748, 2.366067, 2.452028, 2.286139 | 2.366067 [2.286139–2.452028] | 1.5359 |
| Mutated | 7.407445, 7.865045, 7.327274, 7.484857, 7.394486 | 7.407445 [7.327274–7.865045] | 12.046418, 11.764939, 11.674591, 11.743178, 11.316333 | 11.743178 [11.316333–12.046418] | 1.5853 |
| Full row | 1.679129, 1.597549, 1.633070, 1.625049, 1.611819 | 1.625049 [1.597549–1.679129] | 1.777618, 1.817069, 1.799188, 1.762988, 1.817171 | 1.799188 [1.762988–1.817171] | 1.1072 |

**D test.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 1.448239, 1.431558, 1.461299, 1.428808, 1.439448 | 1.439448 [1.428808–1.461299] | 2.231069, 2.280018, 2.232388, 2.220208, 2.223567 | 2.231069 [2.220208–2.280018] | 1.5499 |
| Mutated | 7.168529, 7.189364, 7.149679, 7.081693, 7.115245 | 7.149679 [7.081693–7.189364] | 11.224253, 11.133491, 11.186938, 11.223071, 11.124592 | 11.186938 [11.124592–11.224253] | 1.5647 |
| Full row | 1.598100, 1.607619, 1.587218, 1.584979, 1.590448 | 1.590448 [1.584979–1.607619] | 1.772739, 1.775579, 1.785599, 1.784749, 1.786918 | 1.784749 [1.772739–1.786918] | 1.1222 |

**D release.**

| Corpus | Sealed samples, ms | Sealed median [spread], ms | Lean samples, ms | Lean median [spread], ms | Lean / sealed medians |
|---|---|---|---|---|---:|
| Fixed | 0.790169, 1.081229, 0.991810, 0.922449, 0.921088 | 0.922449 [0.790169–1.081229] | 1.150559, 1.576620, 1.895829, 2.120018, 1.941498 | 1.895829 [1.150559–2.120018] | 2.0552 |
| Mutated | 3.508067, 3.452639, 3.461847, 3.701066, 3.548357 | 3.508067 [3.452639–3.701066] | 5.862285, 5.845095, 5.790234, 5.895845, 5.952688 | 5.862285 [5.790234–5.952688] | 1.6711 |
| Full row | 0.251410, 0.269960, 0.250859, 0.263490, 0.269700 | 0.263490 [0.250859–0.269960] | 0.205799, 0.195960, 0.204020, 0.215860, 0.199809 | 0.204020 [0.195960–0.215860] | 0.7743 |

Harness UTC starts (from its SystemTime, not the controller's starts):
A test `1795093612`, A release `1795093642`, B test `1795093688`,
C test `1795093723`, D test `1795093893`, D release `1795094209`.
An initial D release command exhausted the shell's 120 s deadline while
compiling, with no test output or sample; that attempt is unobserved,
not passed. After suppressing the throwaway modules' unused-item warnings,
the completed release invocation above used a longer shell deadline,
not a changed test budget or timeout.

### 6.3 Cause and decision

There is no bounds-table walk or dynamic trait dispatch on this decode
path. The test-profile regression is in the extra unoptimized generic
helper/Result forwarding layers: the direct version check and Result
return reduce it, while the optimizing compiler already removes much of
that overhead. Full-row decoding improves with the helper changes.

A release is within about 10% on both corpus sets and faster on the full
row; A test is not. D test remains slower (1.55 / 1.56 / 1.12), and D
release **regresses the refusal-heavy corpora** (2.06 / 1.67 / 0.77).
The forced-inlining candidate therefore does not establish the requested
two-profile 10% criterion. Decision pending the coordinator's ruling on
these changed release figures; no further tuning is assumed.
