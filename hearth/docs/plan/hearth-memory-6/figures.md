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
