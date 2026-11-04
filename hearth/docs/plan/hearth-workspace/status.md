# hearth-workspace: status

Wed Nov  4 16:22:23 UTC 2026, at `2b79c5c224115a85eb778e3e57673c921870dc6a`, on `debian-16gb-hel1-6`. What hearth costs the gate, for the user’s ruling.

Measured on this task’s supplied base, with the other plan tasks present. The named early dependency, `hearth-workspace/fixture-readers` at `756553fa129a558541bf9ba38d5f96635e0ed6b4`, was checked and merged before measurement and checked again afterward; both merges were already up to date. No source content changed. Cargo regenerated four missing `skein-scratch` lockfile edges during builds; that incidental lockfile change was restored after measurement.

The host was shared: other Codex processes, and compiler/clippy workloads during this task’s tests and builds, were observed with `ps -eo comm=,pcpu=`. Nothing else launched by this task ran concurrently with a build or test. The identities, full overlap and per-run load of other jobs are unknown; these are shared-host measurements, not an idle-machine baseline.

All timed Cargo commands and collected test runs used `TEMPER_BUNDLE=none` and `NEXTEST_DOUBLE_SPAWN=0`, with closed stdin. Test CPU is user plus system time of test processes and descendants, summed by `tools/tiers/cpu.py`; wrapper CPU is excluded. Suite wall is the collector’s interval from the first test start to the final test end, excluding building and command startup. Build/rebuild wall and command wall below are differences between `date -u +%Y-%m-%dT%H:%M:%S.%NZ` timestamps around the command, including heavy admission and command overhead.

## Measurements

All figures are seconds. Cold / warm means first profile execution / warm 1 / warm 2. The slot’s target was cleaned once, then hearth’s tests were built before the six runs. Each profile’s warm runs followed its first run; fuzzy’s first run followed the three default runs. “Cold” does not mean flushed OS caches or an empty compiler cache; only this slot’s target was cleaned.

| What | CPU | Wall |
|---|---|---|
| `default`, cold / warm | 35.954231 s / 35.905218 s / 34.897518 s | 7.226331 s / 5.942260 s / 6.282762 s |
| `fuzzy`, cold / warm | 68.459108 s / 67.727156 s / 68.023353 s | 16.499871 s / 16.845440 s / 20.523381 s |
| Full gate’s test build: without hearth / added | unknown / unknown: date measures wall only | 372.006183 s / 26.950123 s |
| Full gate’s clippy: without hearth / added | unknown / unknown: date measures wall only | 63.340484 s / 8.971882 s |
| A skein-lib change: hearth’s rebuild | unknown: date measures wall only | 2.847632 s |
| `hearth-render-world` on every change: `default` / `fuzzy` / rebuild | 3.358087 s / 28.866113 s / unknown: rebuild CPU not collected | 1.581386 s / 10.003509 s / 3.050153 s |

Each full-gate pair starts with a separate clean target. “Added” is the second command’s incremental wall time over the completed warm rest, including its warm Cargo traversal. The cumulative pair totals are 398.956306 s for test building and 72.312366 s for clippy; subtracting the without-hearth stage gives the added figures above. A separate cold full-workspace command was not measured. Both stages of both pairs passed.

The initial clean hearth test build took 14.332784 s. After the clippy clean, an untampered prerequisite test build took 8.779646 s before either touch measurement. Touching `skein/crates/skein-lib/src/lib.rs` changed its mtime, not its contents. Compiler caches remained warm, so these rebuild figures measure that requested touch, not a source-content change.

| Run | Tests passed | Suite CPU | Suite wall | Whole command wall |
|---|---:|---:|---:|---:|
| `default-cold` | 2149 | 35.954231 s | 7.226331 s | 8.496699 s |
| `default-warm-1` | 2149 | 35.905218 s | 5.942260 s | 7.117988 s |
| `default-warm-2` | 2149 | 34.897518 s | 6.282762 s | 7.379805 s |
| `fuzzy-cold` | 59 | 68.459108 s | 16.499871 s | 17.346708 s |
| `fuzzy-warm-1` | 59 | 67.727156 s | 16.845440 s | 17.790929 s |
| `fuzzy-warm-2` | 59 | 68.023353 s | 20.523381 s | 21.633102 s |
| `render-default` | 109 | 3.358087 s | 1.581386 s | 2.606075 s |
| `render-fuzzy` | 22 | 28.866113 s | 10.003509 s | 11.098333 s |

All eight runs exited 0. Each test had one attempt. There were no failures, starved tests/runs, unobserved attempts, leaked descendants or incomplete CPU records. The collector used `LIMITS = {}`, so its verdict is “no budget set”; the provisional 30 s / 120 s comparison is recorded below.

## What a skein-only change now costs

Package reach is the transitive reverse closure of local workspace dependencies from `cargo metadata`, including normal, build, dev, optional and target-specific manifest edges. Workspace counts include the changed source package itself; hearth counts do not include any non-hearth package. This is manifest reach, not a claim that every reached package compiled in every command.

Before the plan: `a829ee545c85afa8d04bc2052c0d29bd09806df4` (the request’s starting revision), 279 workspace members, no hearth members. Now: the measured commit above, 310 members, including 30 hearth packages. `jig-protocol-worker-agents` and other dependency changes also arrived between these revisions; the workspace-count differences must not all be attributed to hearth.

| Changed crate | Workspace before | Workspace now | Hearth now |
|---|---:|---:|---:|
| `skein-lib` | 268 | 297 | 28 |
| `skein-io` | 171 | 184 | 12 |
| `skein-shell` | 145 | 157 | 10 |
| `skein-json` | 138 | 150 | 8 |
| `skein-sim` | 144 | 156 | 10 |
| `skein-world` | 129 | 140 | 10 |
| `skein-heap` | 158 | 174 | 13 |
| `skein-scratch` | 189 | 204 | 13 |

Exact hearth reach follows. Columns abbreviate the eight `skein-` sources: Lib, IO, Shell, JSON, Sim, World, Heap, Scratch. “yes” means reached; “—” means not reached.

| Hearth package | Lib | IO | Shell | JSON | Sim | World | Heap | Scratch |
|---|---|---|---|---|---|---|---|---|
| `hearth` | yes | yes | yes | yes | yes | yes | yes | yes |
| `hearth-accounts-world` | yes | — | — | — | — | — | yes | — |
| `hearth-builder-world` | yes | yes | yes | yes | yes | yes | yes | yes |
| `hearth-client-protocol-records` | yes | — | — | yes | — | — | — | — |
| `hearth-config-world` | yes | yes | yes | yes | yes | yes | yes | yes |
| `hearth-conformance-world` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-domain-build` | yes | — | — | — | — | — | — | — |
| `hearth-domain-builder` | yes | — | — | — | — | — | — | — |
| `hearth-domain-render` | yes | — | — | — | — | — | — | — |
| `hearth-domain-store` | yes | — | — | — | — | — | — | — |
| `hearth-domain-tree` | yes | — | — | — | — | — | — | — |
| `hearth-fake-git` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-fake-machine` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-fake-mmdebstrap` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-git-world` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-lib` | yes | — | — | — | — | — | — | — |
| `hearth-mmdebstrap-world` | yes | yes | yes | — | yes | yes | yes | yes |
| `hearth-protocol-accounts` | yes | — | — | — | — | — | — | — |
| `hearth-protocol-builder` | yes | yes | — | yes | — | — | — | — |
| `hearth-protocol-config` | yes | — | — | yes | — | — | — | — |
| `hearth-protocol-git` | yes | — | — | — | — | — | — | — |
| `hearth-protocol-mmdebstrap` | yes | — | — | — | — | — | — | — |
| `hearth-protocol-template` | yes | — | — | — | — | — | — | — |
| `hearth-protocol-tree` | yes | — | — | — | — | — | — | — |
| `hearth-recordings` | — | — | — | — | — | — | — | yes |
| `hearth-render-world` | yes | yes | yes | yes | yes | yes | yes | yes |
| `hearth-service-builder` | yes | yes | — | yes | — | — | — | — |
| `hearth-template-world` | yes | — | — | — | — | — | yes | yes |
| `hearth-test-mirror` | — | — | — | — | — | — | — | yes |
| `hearth-tree-world` | yes | — | — | — | — | — | yes | — |

The timed `skein-lib` touch build reported 35 `Compiling` packages: all 28 hearth packages marked in the Lib column, plus `skein-lib`, `skein-io`, `skein-json`, `skein-shell`, `skein-sim`, `skein-sim-world` and `skein-world`. Its test selection covers both hearth suites measured above. The render-only touch build is separately timed in the measurements table. Other individual crates’ rebuild times and subset-suite costs are unknown: this task measured the specified `skein-lib` touch and whole-hearth suites, not a touch/run for each source.

For the measured `skein-lib` change, rebuild plus both suites costs:

| Suite sample | Test CPU subtotal | Rebuild + suite wall | Rebuild + whole-command wall |
|---|---:|---:|---:|
| cold | 104.413339 s | 26.573834 s | 28.691039 s |
| warm 1 | 103.632374 s | 25.635332 s | 27.756549 s |
| warm 2 | 102.920871 s | 29.653776 s | 31.860539 s |

These sums combine separately measured runs, rather than timing a new continuous change-to-result sequence. Total CPU including the rebuild is unknown because rebuild CPU was not collected.

## Figures awaiting the user

- **The workspace’s `global-timeout`:** `default` 150 s and `fuzzy` 600 s in `.config/nextest.toml`, the sum of each tree’s, including hearth’s 30 s and 120 s, doubled provisionally on 10-22. These follow the file’s own rule until the user rules; they are wall backstops, not measured CPU totals.
- **`tools/tiers/budgets.toml`:** no entry for `conformance-hearth`, and none set for `default` or `fuzzy`. No budget or timeout was changed.
- **`.config/gate/always.txt`:** `hearth-render-world` runs on every change by that file’s rule, at the CPU, wall and rebuild costs above.
- **`hearth/scripts/fail_stop.py`:** kept to hearth’s packages, and no gate runs it. Applying its current check to the whole workspace identifies `temper-stage-one` and `temper-client-browser-world`’s `playground-serve`.
- **Hearth’s focused CPU budget remains red under `skein/docs/foundation/testing-strategy.md`, 8.1:** default measured 35.954231 / 35.905218 / 34.897518 CPU s against 30 s, with double spawn disabled. Fuzzy measured 68.459108 / 67.727156 / 68.023353 CPU s against 120 s. These shared-host results await the user’s ruling. Options: keep 30 s and repair the tests while retaining every assertion and case (the `join-worlds` recommendation), or the user sets a measured budget. No assertion, case, tier or budget was changed. The slowest tests are below.

Slowest focused tests by cold-run CPU, with all three observations:

| Test | Cold CPU | Warm 1 CPU | Warm 2 CPU |
|---|---:|---:|---:|
| `hearth-tree-world::suite frames::no_cut_of_a_generated_frame_reads_as_a_record` | 2.493540 s | 2.492295 s | 2.522405 s |
| `hearth-accounts-world::suite memory::a_read_at_its_limits_holds_no_more_than_its_worst_case` | 1.086886 s | 1.047194 s | 1.047173 s |
| `hearth-tree-world::suite memory::walk::walking_wide_and_deep_layers_holds_no_more_than_the_worst_case` | 0.978789 s | 0.960594 s | 1.001468 s |
| `hearth-builder-world::suite domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer` | 0.969830 s | 0.979147 s | 0.967920 s |
| `hearth-builder-world::suite domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath` | 0.914271 s | 0.889378 s | 0.881672 s |

Slowest fuzzy tests by cold-run CPU, with all three observations:

| Test | Cold CPU | Warm 1 CPU | Warm 2 CPU |
|---|---:|---:|---:|
| `hearth-tree-world::suite fuzzy::tree::each_edit_alone_walks_to_what_the_oracle_lists` | 9.998963 s | 9.941151 s | 10.121168 s |
| `hearth-tree-world::suite fuzzy::tree::random_layers_walk_against_their_edits_to_what_the_oracle_lists` | 7.130464 s | 7.170175 s | 7.115566 s |
| `hearth-tree-world::suite fuzzy::tree::random_layers_hash_to_what_the_oracle_lists` | 3.264931 s | 3.172065 s | 3.195169 s |

## Measurement sources

CPU collection used the throwaway caller of `tools/tiers/cpu.py` described in `tools/tiers/design.md`, 7.1 and 10.2: load with `runpy.run_path`, call `prepare(DIR, PROFILE, target/tiers-cpu-wrapper)`, append its configuration arguments to the nextest command, then call `execute(COMMAND, ENV, DIR, PROFILE, {})` and `summary(DIR, {}, PROFILE)`. The wrapper was built through `heavy cc -std=c11 -O2 -static -Wall -Wextra -Werror tools/tiers/cpu-wrapper.c -o target/tiers-cpu-wrapper`, and rebuilt after cleaning. Each run had a fresh directory. Its JUnit attempt inventory and collector records agreed.

All intervals below are UTC on 2026-11-04, on the machine and commit named above. CPU figures come from each run’s collector summary, build/rebuild figures from the timestamp differences. Every listed command exited 0.

| Sample | Source command (before collector arguments) | UTC start | UTC end |
|---|---|---|---|
| `hearth-initial-build` | `heavy cargo nextest run -p 'hearth*' --no-run` | 16:09:10.324678443Z | 16:09:24.657462234Z |
| `default-cold` | `heavy cargo nextest run -p 'hearth*' --profile default` | 16:09:44.000628380Z | 16:09:52.497327209Z |
| `default-warm-1` | `heavy cargo nextest run -p 'hearth*' --profile default` | 16:10:06.830861728Z | 16:10:13.948849787Z |
| `default-warm-2` | `heavy cargo nextest run -p 'hearth*' --profile default` | 16:10:14.009763086Z | 16:10:21.389568512Z |
| `fuzzy-cold` | `heavy cargo nextest run -p 'hearth*' --profile fuzzy` | 16:10:21.419215367Z | 16:10:38.765923362Z |
| `fuzzy-warm-1` | `heavy cargo nextest run -p 'hearth*' --profile fuzzy` | 16:10:38.815615777Z | 16:10:56.606544466Z |
| `fuzzy-warm-2` | `heavy cargo nextest run -p 'hearth*' --profile fuzzy` | 16:10:56.639545976Z | 16:11:18.272647779Z |
| `clean-full-tests` | `heavy cargo clean` | 16:11:25.616005024Z | 16:11:26.130952876Z |
| `full-tests-without` | `heavy cargo nextest run --workspace --exclude 'hearth*' --no-run` | 16:11:26.160123785Z | 16:17:38.166306794Z |
| `full-tests-with` | `heavy cargo nextest run --workspace --no-run` | 16:17:38.372598513Z | 16:18:05.322721102Z |
| `clean-full-clippy` | `heavy cargo clean` | 16:18:05.454093171Z | 16:18:06.560458123Z |
| `full-clippy-without` | `heavy cargo clippy --workspace --exclude 'hearth*' --all-targets -- -D warnings` | 16:18:06.635078987Z | 16:19:09.975563074Z |
| `full-clippy-with` | `heavy cargo clippy --workspace --all-targets -- -D warnings` | 16:19:10.029716522Z | 16:19:19.001598331Z |
| `hearth-rebuild-prerequisite` | `heavy cargo nextest run -p 'hearth*' --no-run` | 16:19:22.669135345Z | 16:19:31.448781687Z |
| `hearth-touch-rebuild` | `heavy cargo nextest run -p 'hearth*' --no-run` | 16:19:35.374649006Z | 16:19:38.222281091Z |
| `render-default` | `heavy cargo nextest run -p hearth-render-world --profile default` | 16:19:54.092969743Z | 16:19:56.699045225Z |
| `render-fuzzy` | `heavy cargo nextest run -p hearth-render-world --profile fuzzy` | 16:19:56.767206274Z | 16:20:07.865539066Z |
| `render-touch-rebuild` | `heavy cargo nextest run -p hearth-render-world --no-run` | 16:20:17.929478592Z | 16:20:20.979631652Z |

The initial `heavy cargo clean` preceded `hearth-initial-build` and removed 5.7 GiB; its exact UTC interval is unknown because it was not timed. The two later cleans removed 5.3 GiB and 26.8 GiB, respectively. Each touch immediately preceded its corresponding rebuild command.

Current reach source: `cargo metadata --format-version 1 --no-deps`. Baseline source: `git ls-tree -r --name-only a829ee545c` and `git show a829ee545c:<manifest>` reconstructed that revision’s tracked Cargo manifests and lockfile beneath an untracked directory in this slot, with empty placeholders for its tracked Rust target paths; `cargo metadata --offline --format-version 1 --no-deps --manifest-path .costs/base/Cargo.toml` read its workspace graph without building it. Closure used only metadata’s local path dependency edges between workspace members. No code or metadata from another slot was used.

Decision figures were checked against `.config/nextest.toml`, `tools/tiers/budgets.toml`, `.config/gate/always.txt`, and `hearth/scripts/fail_stop.py`. The fail-stop regular expressions were also applied read-only to metadata’s binary sources across the workspace, yielding exactly the two binaries named above. No reference to that script was found in `.config/`, `tools/` or `.temper/`.
