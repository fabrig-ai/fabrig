# hearth-apt-304: the plan

2026-11-07. From `inputs/request.md`: mmdebstrap 1.5.7 / apt 3.0.4's recordings and their expectations, beside apt 3.0.3's (`hearth/docs/design/domain/testing.md`, 4.5).

## Parts and tasks

| Part | Task | Needs | Request item | Owns |
|---|---|---|---|---|
| `recordings` | `recordings` | none | 1, 2, 3 | `hearth/testing/recordings/mmdebstrap/1.5.7-apt3.0.4/`, `hearth/tests/mmdebstrap/expect/1.5.7-apt3.0.4/` |

- **`recordings`:** copies `inputs/1.5.7-apt3.0.4/` byte for byte into the recordings, writes each of the seven expectations from the draft `recordings::every_recording_has_an_expectation_and_the_reference_agrees` prints, reviews them against 3.0.3's, and runs the focused and fuzzy suites of `hearth-mmdebstrap-world`, `hearth-builder-world` and `hearth-conformance-world`. Medium effort, about 30 implementer-minutes.

## Critical path

`recordings` alone. The plan lands when it has merged and the trunk's end check is green.

## Decisions

- **Copy, not move:** the inputs stay as the plan's record; the request limits the change to the two new directories.
- **No design or status change:** 4.5 already says mmdebstrap's runs are recorded under `<its version>-apt<apt's>/`, and a new version is re-recorded. Its status's **Not yet** for 4.5 is about M2's tools, not this.
- **No test change expected.** Checked by the cut on the trunk: against 3.0.3, the inputs differ only in the five `Fetched … (… /s)` rates and the two `I: success in … seconds` times; the 3.0.3 expectations are exactly the drafts the test prints (`fetched 10200000`, a byte count). No focused or fuzzy test assumes one version: `recordings::all` reads every version directory, sorted; `the_recorded_failures_have_the_causes_5_5_gives_them` takes each case's first recording (3.0.3's), and 3.0.4's causes are held by its expectations; the fuzzy recordings test spreads its 128 seeds over 14 cases. The builder's `recorded_closure` and the conformance world's `recorded_resolve` and `conformance_resolve` are read only by `conformance_*` and `recorded_*` binaries, outside focused and fuzzy; after-landing runs them, and the fake's `differ` compares no rate or time line.
- **Effort medium:** the request asks for low; a spec's efforts are high or medium.

## Overlaps

None known. `per-wake` writes jig alone; `look-round-17`'s cut is in flight and the queue's drafts are not cut, so their files are unknown. This plan writes only two new directories, which no other plan can already be changing.
