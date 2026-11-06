# hearth-memory-6: figures

## Before

Measured at `64e35143f2eff5780434da052608d5fc9a74df13` on
2026-11-06 UTC, on `debian-32gb-hel1-14`. Every figure below is from
that commit. The timestamps are the output of `date -u` immediately
before each measurement (the counts were read at the timestamp shown).

| Figure | Value | How |
| --- | ---: | --- |
| hearth `default` CPU | 37.971859 s | `tools/tiers/cpu.py`; Fri Nov 6 06:49:28 UTC 2026 |
| hearth `fuzzy` CPU | 96.834378 s | `tools/tiers/cpu.py`; Fri Nov 6 06:49:40 UTC 2026 |
| `max_out` | 39 | `hearth/scripts/old-forms.txt`; Fri Nov 6 06:51:28 UTC 2026 |
| `worst_case` | 0 | `hearth/scripts/old-forms.txt`; Fri Nov 6 06:51:28 UTC 2026 |

Both CPU runs used `NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none`.
An untracked temporary driver loaded `tools/tiers/cpu.py` with
`runpy.run_path`, called `prepare`, appended its configuration arguments
to `heavy cargo nextest run --profile PROFILE -p PACKAGE ...`, then
called `execute` and `summary`. The static CPU wrapper was built once
through `heavy cc -std=c11 -O2 -static -Wall -Wextra -Werror` from
`tools/tiers/cpu-wrapper.c`. Compilation and collector overhead are
excluded from the CPU totals; the recorded scope is
`waited-and-reaped-descendants` (`tools/tiers/design.md`, 7.2).

Package selection came from `cargo metadata --no-deps --format-version 1`:
every workspace member whose manifest is under `hearth/`, 34 packages.
The root's `default` and `fuzzy` filters were preserved, without a test
filter or concurrency override. The packages were:

```text
hearth hearth-accounts-world hearth-builder-world hearth-client-protocol-records
hearth-config-world hearth-conformance-world hearth-domain-build
hearth-domain-builder hearth-domain-render hearth-domain-store hearth-domain-tree
hearth-fake-git hearth-fake-machine hearth-fake-mmdebstrap hearth-git-world
hearth-lib hearth-mmdebstrap-world hearth-protocol-accounts
hearth-protocol-builder hearth-protocol-config hearth-protocol-git
hearth-protocol-mmdebstrap hearth-protocol-template hearth-protocol-tree
hearth-recordings hearth-render-world hearth-service-builder hearth-template-world
hearth-test-fixtures hearth-test-git hearth-test-mirror hearth-test-render
hearth-test-repository hearth-tree-world
```

| Profile | Tests passed / attempts | User CPU s | System CPU s | Execution wall s |
| --- | ---: | ---: | ---: | ---: |
| `default` | 2183 / 2183 | 30.070981 | 7.900878 | 3.343354 |
| `fuzzy` | 60 / 60 | 96.462690 | 0.371688 | 17.949191 |

Both runs had zero failed tests, retries, starved attempts, unobserved
attempts, leaked descendants and incomplete records. The default
collector returned 1 for exceeding the existing 30 CPU s budget by
7.971859 s; this baseline is red on CPU, despite all assertions passing.
Fuzzy returned 0; its collector verdict was `no budget set` because
`tools/tiers/budgets.toml` has no hearth fuzzy entry. Its measured total
is below the 120 CPU s figure in
`skein/docs/foundation/testing-strategy.md`, 8.1.

### Ten most costly default tests

These are per-test user plus system CPU, one attempt each, from the
default run at the commit and timestamp above. Every binary is `suite`.

| Package | Test | CPU s |
| --- | --- | ---: |
| `hearth-tree-world` | `memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | 1.128163 |
| `hearth-tree-world` | `walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | 0.908517 |
| `hearth-builder-world` | `domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.838080 |
| `hearth-builder-world` | `domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.833521 |
| `hearth-builder-world` | `domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.821809 |
| `hearth-tree-world` | `walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | 0.733738 |
| `hearth-tree-world` | `hash::wide_and_deep_layers_hash_to_the_oracles` | 0.698183 |
| `hearth-tree-world` | `memory::walk::walking_generated_layers_against_a_parent_holds_no_more_than_its_ceiling` | 0.599966 |
| `hearth-builder-world` | `domain::a_second_build_of_the_same_spec_stages_and_runs_nothing` | 0.504747 |
| `hearth-tree-world` | `walk::generated_edits_walk_to_the_oracles_digests_and_file` | 0.499586 |

### Builder memory peaks

Measured at the same commit, with only a temporary print of
`Outcome::peaks()` after each scenario. Both test files were restored
after measurement; no scenario, allocator, limit or ceiling changed.
All five tests passed. Values are bytes. The peak column is the maximum
of the per-process peaks, since `CEILING` applies to each process rather
than their sum. The full vectors retain the other processes' peaks.

The first four rows are `memory::` tests in
`hearth/tests/builder/tests/suite/memory.rs`, measured on
Fri Nov 6 06:50:19 UTC 2026 through
`heavy cargo nextest run --profile default -p hearth-builder-world --test suite -E 'test(/^memory::/)' --success-output immediate`.
The last row is `fuzzy::memory::` in
`hearth/tests/builder/tests/suite/fuzzy/memory.rs`, measured on
Fri Nov 6 06:50:53 UTC 2026 through
`heavy cargo nextest run --profile fuzzy -p hearth-builder-world --test suite -E 'test(=fuzzy::memory::a_large_render_holds_no_more_than_its_ceiling)' --success-output immediate`.
Both used `NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none`.

| Test | Peak bytes | `CEILING` bytes | `Outcome::peaks()` |
| --- | ---: | ---: | --- |
| `the_builder_holds_no_more_than_its_ceiling_at_every_iteration` | 192615 | 250000 | `[192615, 1383, 2679, 5543, 5420]` |
| `queued_specs_hold_no_more_than_their_ceiling` | 191599 | 250000 | `[191599, 1383, 3690, 5543, 5420]` |
| `recording_resolutions_holds_no_more_than_their_ceiling` | 192615 | 250000 | `[192615, 1383, 2679, 5543, 5420, 15585]` |
| `building_holds_no_more_than_their_ceiling` | 1100668 | 1400000 | `[1100668, 1383, 2679, 5543, 5420, 15585]` |
| `a_large_render_holds_no_more_than_its_ceiling` | 2108490 | 2700000 | `[2108490, 1383, 826068, 17626, 17626]` |

### Comparison with the testing status

`hearth/docs/design/domain/testing.status.md`, the **Departs** on 9.1,
records 44.886618 CPU s for focused on 2026-11-05 before the last repair,
and conflicting earlier focused figures of 35 to 36, 48.292873, 30.2 and
about 32 CPU s. This baseline's 37.971859 CPU s is below 44.886618 and
48.292873, above the other figures, and still above 30. It does not
establish why those earlier figures differ or which are comparable.

The status records 68 CPU s fuzzy on 2026-11-04 before the large render
joined it, and 12.238013 CPU s for that render separately. This full
fuzzy baseline is 96.834378 CPU s with the large render included. Changes
in suite contents and measurement conditions prevent attributing the
difference here. The status discrepancy remains unresolved.

No requested baseline figure is unknown.

## After

Measured at `2916bc07d21c1eb4cd7210135da58460197847cf` on
2026-11-06 UTC, on `debian-32gb-hel1-14`, after merging the required
`close` branch (already included at this revision). The machine, driver
method, package selection, profile filters and peak-reading method are
the same as Before. The temporary driver again loaded `tools/tiers/cpu.py`
through `runpy.run_path`, used `prepare`, `execute` and `summary`, and
built the static wrapper once through `heavy cc`. All 34 hearth workspace
packages listed in Before were selected from fresh Cargo metadata.
`NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none` were set, with no test
filter or concurrency override on either full run.

The timestamps below are from `date -u` immediately before each CPU run.
Compilation, listing and wrapper overhead are excluded; the scope remains
`waited-and-reaped-descendants` (`tools/tiers/design.md`, 7.2).
The records were reconciled against JUnit by the collector.

| Profile | Timestamp (UTC, 2026-11-06) | Tests passed / attempts | User CPU s | System CPU s | Total CPU s | Execution wall s |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `default` | 08:20:12 | 2189 / 2189 | 34.555707 | 7.123153 | 41.678860 | 9.421157 |
| `fuzzy` | 08:20:44 | 60 / 60 | 100.135096 | 0.385572 | 100.520668 | 24.255479 |

Both runs have zero failures, retries, starved attempts, unobserved
attempts, leaked descendants and incomplete records. Default returned 1
solely for exceeding the existing 30 CPU s budget by 11.678860 s; its
assertions all passed. Fuzzy returned 0, with verdict `no budget set`:
`tools/tiers/budgets.toml` has no hearth fuzzy entry. Its measured CPU is
below the standing 120 CPU s figure
(`skein/docs/foundation/testing-strategy.md`, 8.1). No budget or timeout
changed.

### Before beside after

| Figure | Before | After | After minus before |
| --- | ---: | ---: | ---: |
| `default` CPU s | 37.971859 | 41.678860 | +3.707001 |
| `default` CPU s over 30 | 7.971859 | 11.678860 | +3.707001 |
| `default` tests | 2183 | 2189 | +6 |
| `fuzzy` CPU s | 96.834378 | 100.520668 | +3.686290 |
| `fuzzy` tests | 60 | 60 | 0 |
| `max_out` | 39 | 0 | -39 |
| `worst_case` | 0 | 0 | 0 |

The after counts were read from `hearth/scripts/old-forms.txt` at
08:22:45 UTC on 2026-11-06. This compares two revisions on the same
machine and by the same method, rather than repeated samples of unchanged
code. The merged trunk also includes other changes. How much of either
CPU difference comes from this round's code, other merged work or run
conditions is unknown: this measurement does not isolate those causes.
It does not reconcile the older, conflicting figures recorded in Before.

### Builder memory peaks

After peaks were read with the same temporary `Outcome::peaks()` prints
as Before, at the revision above. The five default tests ran at
08:21:50 UTC on 2026-11-06 through
`heavy cargo nextest run --profile default -p hearth-builder-world --test suite -E 'test(/^memory::/)' --success-output immediate`.
The large render ran at 08:22:15 UTC through
`heavy cargo nextest run --profile fuzzy -p hearth-builder-world --test suite -E 'test(=fuzzy::memory::a_large_render_holds_no_more_than_its_ceiling)' --success-output immediate`.
Both set `NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none`.
All six passed. Both test files were restored byte for byte before the
document edits; scenarios, limits, assertions and ceilings are unchanged.

Values are bytes. Each peak is the maximum per-process peak, since the
ceiling applies to each process independently. The new stuck-client
scenario has no Before peak or ceiling because it did not exist there.

| Test | Before peak | After peak | Before ceiling | After ceiling | After `Outcome::peaks()` |
| --- | ---: | ---: | ---: | ---: | --- |
| `the_builder_holds_no_more_than_its_ceiling_at_every_iteration` | 192615 | 193167 | 250000 | 250000 | `[193167, 1383, 2679, 5543, 5420]` |
| `queued_specs_hold_no_more_than_their_ceiling` | 191599 | 193167 | 250000 | 250000 | `[193167, 1383, 3690, 5543, 5420]` |
| `recording_resolutions_holds_no_more_than_their_ceiling` | 192615 | 193167 | 250000 | 250000 | `[193167, 1383, 2679, 5543, 5420, 15585]` |
| `building_holds_no_more_than_their_ceiling` | 1100668 | 1101220 | 1400000 | 1400000 | `[1101220, 1383, 2679, 5543, 5420, 15585]` |
| `a_large_render_holds_no_more_than_its_ceiling` | 2108490 | 2099206 | 2700000 | 2700000 | `[2099206, 1383, 826068, 17626, 17626]` |
| `a_client_that_stops_reading_holds_render_under_its_ceiling` | — | 238947 | — | 300000 | `[238947, 1383, 18666, 5543, 5461]` |

### Ceilings changed during this round

The history and diffs of
`hearth/tests/builder/tests/suite/memory.rs`,
`hearth/tests/builder/tests/suite/fuzzy/memory.rs` and
`hearth/tests/tree/tests/suite/memory.rs`, from the Before revision through
the measured revision on this plan's branches, show no change to an
existing measured ceiling. The only ceiling introduced and then changed
is the new stuck-client test's:

| Test | Previous ceiling | Final ceiling | Peak that set it | Why and commit |
| --- | --- | ---: | ---: | --- |
| `a_client_that_stops_reading_holds_render_under_its_ceiling` | `u64::MAX`, unmeasured placeholder | 300000 | 237107 | `c556690aea`: sixteen targets and thirty-two shared static files expose queued specs; a quarter above the measured held-output peak, rounded up to two significant digits. |

`deb1386e34` introduced the placeholder while scheduling prevented the
world reaching its release deadline, so its prior peak is unknown: that
world did not finish. `c556690aea` records that increasing the queue
watermark to 4096 instead peaks at 348482 bytes and fails the 300000-byte
ceiling. This is evidence from that commit's measurement, not a mutation
repeated here. The new test now costs 0.190608 CPU s in the full default
run; its after peak is 238947 bytes and still fits that same ceiling.
`713ae7effa` changes the tree memory driver's output queue to capacity one
when removing `MAX_OUT`, and changes no tree ceiling. The large-render
file has no change in this interval. No ceiling was raised by this task.

### Ten most costly default tests

These are user plus system CPU from the full default run above, one
attempt each, ordered by CPU. The binary IDs identify package and binary.

| Binary ID | Test | CPU s |
| --- | --- | ---: |
| `hearth-tree-world::suite` | `memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling` | 1.299843 |
| `hearth-tree-world::suite` | `walk::each_edit_alone_walks_to_the_oracles_digests_and_file` | 0.985476 |
| `hearth-builder-world::suite` | `domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.905413 |
| `hearth-builder-world::suite` | `domain::a_helper_request_that_fails_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.897948 |
| `hearth-builder-world::suite` | `domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.872006 |
| `hearth-tree-world::suite` | `hash::wide_and_deep_layers_hash_to_the_oracles` | 0.829022 |
| `hearth-tree-world::suite` | `walk::an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` | 0.786298 |
| `hearth-tree-world::suite` | `memory::walk::walking_generated_layers_against_a_parent_holds_no_more_than_its_ceiling` | 0.645605 |
| `hearth-builder-world::suite` | `goldens::each_error_case_prints_its_errors_after_the_healthy_spec` | 0.560335 |
| `hearth-builder-world::suite` | `domain::a_second_build_of_the_same_spec_stages_and_runs_nothing` | 0.540033 |

### Answer and costs for the next small plan

**No: hearth's focused suite takes 41.678860 CPU s, 11.678860 s over the
user's 30 CPU s.** All 2189 tests pass, but the CPU verdict is red.
The round does not bring the suite under its figure, and that figure
stays 30.

The largest families, grouped by binary without overlap, are:

| Binary ID | Tests | CPU s |
| --- | ---: | ---: |
| `hearth-builder-world::suite` | 106 | 12.008451 |
| `hearth-tree-world::suite` | 54 | 10.029600 |
| `hearth-builder-world::real` | 183 | 8.479085 |
| `hearth-render-world::suite` | 109 | 4.086128 |
| `hearth-domain-render` | 619 | 2.064762 |
| All remaining binaries | 1118 | 5.010834 |

The first four account for 34.603264 CPU s. Within those binaries,
the following module families refine the cost without adding it again:

| Family | Tests | CPU s |
| --- | ---: | ---: |
| Builder suite `domain::` | 13 | 4.139167 |
| Tree suite `walk::` | 16 | 3.612423 |
| Tree suite `memory::` | 11 | 2.833243 |
| Render suite `properties::` | 20 | 2.217494 |
| Builder suite `properties::` | 9 | 1.974704 |
| Tree suite `hash::` | 9 | 1.841482 |
| Builder suite `goldens::` | 22 | 1.774087 |
| Tree suite `trees::` | 13 | 1.623290 |

A small follow-up can start with the builder domain's repeated build and
failure scenarios, the tree's repeated edit/fault walks and wide/deep
memory scenario, and the builder real-git binary's integration coverage.
The top-test table locates the expensive individual scenarios; the family
tables also expose costs spread across many small tests. Check that each
belongs at its tier, then isolate repeated work or allocation costs before
changing it (`skein/docs/foundation/testing-strategy.md`, 8.1). These
measurements rank the work; they do not establish an inefficiency or how
much a repair can save. Moving tests to escape the budget would raise it.

The full fuzzy run's large render costs 14.633486 CPU s. Whether that cost
lies mainly in the harness or production code is unknown because no
profiling here separates them. Full-gate incremental build and clippy
costs are also unknown for this revision because this measurement covers
test execution. All requested after CPU, peak and count figures are known;
no reason for the CPU deltas or the conflicting older records is inferred.
