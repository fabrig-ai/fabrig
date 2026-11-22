# Failure worlds: the room F7 made, measured

2026-11-21. temper's focused (`default`) and `fuzzy` suites, by layer and by affected binary, from the suites' own reports, before and after F7 of `temper/docs/design/draft/failure-worlds.execution.md`, section 1: the fuzzy copies of focused tests retired, and the real-loop and one-process stories cut to what only their tiers show, their other assertions held below (`temper/docs/design/testing/checks.md`, section 1). It follows the plan in `failure-worlds-budget.md`, whose figures were unmeasured, and moves no figure in `tools/tiers/budgets.toml`: the moves at its end are proposals for the user.

Only temper's tests change in F7, so only temper's packages are measured. The other four trees run the same code before and after.

## Method

- **Selection:** every package under `temper/` that `cargo metadata --no-deps --format-version 1` lists, through `tools/tiers/one-process default -p …` and `tools/tiers/one-process fuzzy -p …`, from the checkout's root, with `TEMPER_INTERNAL_BUNDLE` and `TEMPER_BUNDLE` both `none`. The hold is taken as the tool takes it, never bypassed. No slow profile, browser, walk or throwaway driver runs.
- **Figures:** each run's `results.json` (`tools/tiers/design.md`, 11.8): CPU is user plus system seconds of the `tests`, `fresh` and `alone` processes, waited-for descendants included, placed by the record's own `tree` and `layer`, and checked against `cpu-summary`'s totals. No test's CPU is apportioned from a shared process. The ceiling is `tools/tiers/budgets.toml` with its moves applied once (`tools/tiers/budgets.md`, sections 2 and 10), as `cpu-summary` gives it. Instruction counts stand beside CPU where the run counted.
- **Each run's record:** its commit, machine, lanes, wall, and every failed, starved, cancelled or unobserved test by name. A binary with one is **partial**, and its figure a lower bound.

## Before

**Unmeasured: package discovery was blocked waiting for disk, then timed out.** The intended measurement commit is `ac898e16f970d628425df798b581348375be23e0`, read with `git rev-parse HEAD` before any work and verified unchanged after the attempt. No suite results were produced; this is not a zero-cost baseline or an affordability claim.

The attempt ran from machine **hel1-8** (the identifying suffix of the observed hostname) on 2026-11-21. At 20:12:39 UTC the initial checkout and time were recorded. The prerequisite `cargo metadata --no-deps --format-version 1`, piped to a reader selecting manifests under `temper/`, was then started through Cargo on PATH. It produced no metadata. Its only captured diagnostic was `heavy: disk: waiting: 4556435456 bytes short`; the enclosing command timed out at its 1800 s limit. At 20:42:51 UTC the checkout still had no target directory. The observed outcome is the command timeout, not a verified Cargo exit 75; neither a builder identity nor a completed disk-wait outcome was reported.

Consequently neither prescribed `tools/tiers/one-process --out DIR default -p …` nor its `fuzzy` counterpart started. The package selection and Cargo target directory could not be obtained, so there are no run directories, `results.json`, `run.json` or `cpu-summary` outputs. No hold was acquired or bypassed (the environment's hold was `cargo --hold`), no disk workaround or unchanged retry was attempted, and no slow profile, browser, walk, render check or throwaway driver ran. This is the departure from **Method**: failure in discovery prevented both suite invocations, rather than leaving a partly completed suite.

| Suite | Lanes | Suite wall s | Exit status | Instructions | temper CPU s |
| --- | --- | --- | --- | --- | --- |
| Focused (`default`) | unmeasured | unmeasured | not started | not counted; no run | unmeasured |
| `fuzzy` | unmeasured | unmeasured | not started | not counted; no run | unmeasured |

### Focused (`default`)

No layers were placed by a run. There are no CPU figures to compare with `cpu-summary`, and no measured room or ceilings with moves applied.

| Layer | CPU s | Ceiling | Room |
| --- | --- | --- | --- |

### Fuzzy

No layers were placed by a run. There are no CPU figures to compare with `cpu-summary`, and no measured room or ceilings with moves applied.

| Layer | CPU s | Ceiling | Room |
| --- | --- | --- | --- |

### Affected binaries and completeness

All seven requested binaries are **unmeasured**, not completed observations or measured lower bounds. No binary has a process record to mark **partial**; every test is unobserved. There are no reported failed, crashed, starved or cancelled tests, and no standalone failure diagnostic to run. Without a listing or `results.json`, the selected test names and counts are unknown; this must not be read as an empty selection or as passing tests.

| Binary | Focused CPU s | Focused selected tests | Fuzzy CPU s | Fuzzy selected tests |
| --- | --- | --- | --- | --- |
| `temper-engine-domain-world::suite` | unmeasured | unknown | unmeasured | unknown |
| `temper-engine-forge-world::suite` | unmeasured | unknown | unmeasured | unknown |
| `temper-worker-domain-world::suite` | unmeasured | unknown | unmeasured | unknown |
| `temper-system-world::suite` | unmeasured | unknown | unmeasured | unknown |
| `temper-one-process-world::suite` | unmeasured | unknown | unmeasured | unknown |
| `temper-one-process-world` (library tests) | unmeasured | unknown | unmeasured | unknown |
| `temper-real-loop-world::suite` | unmeasured | unknown | unmeasured | unknown |

Every `fuzzy::` test's wall time in `temper-real-loop-world::suite` is likewise unmeasured; no result supplies their selected names or individual times. No shared-process CPU has been apportioned, no historical figure substituted, and no budget, move, reference or test changed.

## After

Measured **2026-11-22**, at commit `9f9c7462852131328fbb55452ffdcace568dd471`, on **hel1-18** (the identifying suffix of the observed hostname). The checkout includes the duplicate retirements, the two library referee controls, the composed-system holders, the worker hosting repair, and the reconciled twelve-story real-loop library. Before running, the fuzzy listing confirmed no engine-domain `fuzzy::removal::` or `fuzzy::reset::`, no four fixed-cut one-process `fuzzy::upgrade::power_lost_at_*` wrappers, and no real-loop 4.4, 4.5, 5.5, 6.5 or 6.7 story. It listed 28 engine-domain, three one-process and twelve real-loop fuzzy tests.

Both commands selected all **132 packages** whose manifest from `cargo metadata --no-deps --format-version 1` is under `temper/`, with `TEMPER_INTERNAL_BUNDLE=none TEMPER_BUNDLE=none`. They used **`--layers all`** in addition to Method's command: since the layer-selection change, one-process otherwise omits real-loop and end-to-end (`tools/tiers/design.md`, 11.9). This keeps the intended coverage of both profiles, including the requested real-loop observations; it does not select a slow profile or a browser/process/live tier. Before never reached selection, so there is no executed before selection to reproduce. Neither the hold nor a disk check was bypassed.

The directories, relative to the checkout root and beneath Cargo's discovered target directory, are:

- `target/failure-worlds-after-default-20261122T1759` — command started at 17:58:56 UTC, returned at 18:04:34 UTC, exit **0**.
- `target/failure-worlds-after-fuzzy-20261122T1805` — command started at 18:04:40 UTC, returned at 18:06:05 UTC, exit **0**.

Each directory's `results.json` records the measured commit above and exactly this `selected_layers`:
`["step", "domain", "system", "machine", "protocol", "io", "simulated", "real-loop", "end-to-end", "checks"]`.
Each records `counting = {"counted": true, "why": null}`. Both used **four lanes** (`run.json`). Suite wall below is the first process start through the last process end, derived from `results.json` and matching `run.json`, not build/discovery/hold elapsed time. All process records are kind `tests`; there are no `fresh`, `alone`, reference or empty-process costs to add or exclude.

| Suite | Selected / passed tests | Suite wall s | Exit status | Instructions | temper CPU s, all layers | temper CPU s, replaying layers only | Difference from Before |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Focused (`default`) | 15607 / 15607 | 203.192323 | 0 | 4558986435053 | 687.486916 | 528.779495 | unknown: Before unmeasured |
| `fuzzy` | 120 / 120 | 79.555779 | 0 | 1288286319334 | 225.069200 | 159.475917 | unknown: Before unmeasured |

Replaying totals exclude `real-loop` and `end-to-end`, the layers now omitted by the gate. All-layer and per-layer CPU sums were independently summed from user plus system seconds in `results.json` and checked against `tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --json` on each same directory (both exited 0). The ceilings below are that summary's, with the existing moves applied once; room is ceiling minus observed CPU, **not** main's held budget or a measured saving. CPU and wall are rounded to six decimal places; counts are exact. All differences from Before, including selected-count and instruction differences, are **unknown**, not zero.

### Focused (`default`)

| Layer | CPU s | Ceiling s | Room s | Instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | --- |
| step | 170.169973 | 254.600000 | 84.430027 | 1478737120232 | unknown; two referee controls moved here, worker hosting repair |
| domain | 156.324531 | 142.800000 | -13.524531 | 762856825362 | unknown; engine-domain/forge holders added for cut or retired stories |
| system | 59.519176 | 111.000000 | 51.480824 | 360074012071 | unknown; composed chat/merge and story holders added |
| machine | 1.648948 | 4.000000 | 2.351052 | 13879328717 | unknown; no F7 move attributed |
| protocol | 10.236883 | 9.000000 | -1.236883 | 66647786230 | unknown; no F7 move attributed |
| io | unmeasured: no process | none | unknown | not counted: no process | unknown; no F7 move attributed |
| simulated | 130.879984 | 470.000000 | 339.120016 | 865876840778 | unknown; two supplied-value referee controls moved out |
| real-loop | 95.351008 | 76.700000 | -18.651008 | 625331552150 | unknown; stories cut or retired; focused helper users retained |
| end-to-end | 63.356413 | 36.900000 | -26.456413 | 385582969513 | unknown; no F7 move attributed |

### Fuzzy

| Layer | CPU s | Ceiling s | Room s | Instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | --- |
| step | 26.224402 | 20.600000 | -5.624402 | 254078453440 | unknown; moved referee controls are focused, not fuzzy |
| domain | 93.643688 | 85.400000 | -8.243688 | 420252783685 | unknown; eleven duplicate removal/reset imports retired; lower holders added |
| system | 11.567764 | 13.000000 | 1.432236 | 73020540869 | unknown; new focused holders replace two upper fuzzy wrappers |
| machine | 0.378642 | 2.000000 | 1.621358 | 2124390771 | unknown; no F7 move attributed |
| protocol | 0.115828 | 2.000000 | 1.884172 | 559000238 | unknown; no F7 move attributed |
| io | unmeasured: no process | none | unknown | not counted: no process | unknown; no F7 move attributed |
| simulated | 27.545593 | 196.000000 | 168.454407 | 178930649388 | unknown; four fixed-cut upgrade and two chat/merge wrappers retired |
| real-loop | 65.593283 | 183.000000 | 117.406717 | 359320500943 | unknown; twelve survivors cut to their seams; other stories retired |
| end-to-end | unmeasured: no process | 1.000000 | unknown | not counted: no process | unknown; zero selected, not a timed observation |

No process means no row in `cpu-summary`; those explicit rows above satisfy the layer inventory without inventing measurements. They add no CPU to either sum. The non-gate summaries warn on default domain/protocol instruction ceilings and real-loop/end-to-end CPU ceilings, and fuzzy step/domain/system instruction ceilings. These are observations, not permission to increase any budget. Positive CPU room does not imply instruction room.

### Affected binaries and completeness

Every selected test passed. **Failed, crashed, starved, cancelled and unobserved selected tests: none**, in either suite (`unobserved = []`, `cancelled = null`; every process exited 0, with no signal or leak). Therefore **no measured binary is partial**, and there is no failing test to diagnose alone. The zero-selected fuzzy library row is not a partial execution: there was no process. Slow-profile tests were not selected and are not claimed as observations.

| Binary | Focused CPU s | Focused selected tests | Focused instructions | Fuzzy CPU s | Fuzzy selected tests | Fuzzy instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `temper-engine-domain-world::suite` | 138.721038 | 1400 | 660356734319 | 39.044897 | 28 | 191564702277 | unknown; duplicate fuzzy removal/reset registrations retired, focused story holders added |
| `temper-engine-forge-world::suite` | 6.563226 | 359 | 37587132357 | 7.210454 | 10 | 41071967444 | unknown; focused story holders added |
| `temper-worker-domain-world::suite` | 0.214666 | 76 | 1376580289 | 0.393680 | 3 | 2600514867 | unknown; hosting repair is in the worker library; no retirement credited here |
| `temper-system-world::suite` | 14.033373 | 66 | 110186948777 | 1.549208 | 1 | 7394755043 | unknown; focused chat/merge and story holders added |
| `temper-one-process-world::suite` | 82.917321 | 62 | 532293147978 | 27.428055 | 3 | 177979666742 | unknown; two focused controls moved out, four upgrade and two chat/merge fuzzy wrappers retired |
| `temper-one-process-world` (library tests) | 0.720108 | 26 | 5745159428 | unmeasured: no process | 0 | not counted | unknown; two focused referee controls moved in |
| `temper-real-loop-world::suite` | 50.975621 | 31 | 283188280757 | 65.593283 | 12 | 359320500943 | unknown; five requested retirements plus 6.7 retired, nine scripted survivors cut; backend and twins retained |

The real-loop library now has twelve fuzzy stories: nine scripted (4.1/4.3 combined, 4.6, 5.1, 5.2, 5.4, 6.4, 6.6, 6.8, kept-delegate landing), the program backend and the two recorded twins. The following are their **individual wall times**, not apportioned CPU. Each name is under `temper-real-loop-world::suite`; all passed. Every wall difference from Before is unknown.

| Fuzzy test | Wall s |
| --- | ---: |
| `fuzzy::case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 7.343441 |
| `fuzzy::case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 4.403056 |
| `fuzzy::case_5_1::released_runs_read_the_pushed_guide_without_the_dropped_rule` | 4.401456 |
| `fuzzy::case_5_2::a_stopped_owners_reviewed_change_lands_and_its_news_arrives_after_release` | 6.605569 |
| `fuzzy::case_5_4::a_pushed_change_is_cancelled_under_its_closed_pull_and_an_overlapping_landing_wakes_b_after_release` | 9.198825 |
| `fuzzy::case_6_4::a_red_tip_is_repaired_before_a_waiting_change_updates_and_lands` | 8.403207 |
| `fuzzy::case_6_6::kit_merges_before_apps_first_push_and_both_land_green_with_the_pin` | 6.519898 |
| `fuzzy::case_6_8::a_testers_test_lands_only_through_its_owners_change_without_tester_forge_writes` | 6.312759 |
| `fuzzy::green_light::a_green_light_in_the_inbox_lets_the_change_land` | 8.956867 |
| `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 4.589322 |
| `fuzzy::program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | 3.845024 |
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | 8.878377 |

## The moves for the user

**Focused (`default`): none supported by measured differences. Fuzzy: none supported by measured differences.** Before is wholly unmeasured, so no donor layer has a measured freed amount. After's positive room is not a before/after retirement credit, and the largest layer or binary differences cannot be ranked. No positive `cpu_s` can be certified as no greater than a donor's measured saving.

The candidate moves in `failure-worlds-budget.md`, *The moves proposed*, remain unfunded proposals, not results of this measurement:

| Suite | from | to | Candidate cpu_s | Why not proposed from these runs |
| --- | --- | --- | ---: | --- |
| default | `temper.real-loop` | `temper.domain` | 12 | Cut stories have lower holders, but baseline M and retirement credit D are unknown. |
| default | `temper.simulated` | `temper.step` | 0.1 | The two referee controls moved, but the donor's freed CPU is unknown. |
| fuzzy | `temper.real-loop` | `temper.domain` | 20 | Survivors were cut and stories retired, but their before cost was not measured. |
| fuzzy | `temper.simulated` | `temper.system` | 2 | Chat/merge holders replaced wrappers, but the simulated donor's saving is unknown. |

Each candidate direction is strictly downward in `tools/tiers/layers.toml`'s order, as `tools/tiers/budgets.md`, 10 requires. None satisfies the measured-funding condition here. In the budget note's formula, only After is observed; M, D and the marginal L/H/S costs cannot be separated from these whole-process figures. Duplicate retirement within domain or simulated would free that same layer, not itself be an inter-layer move. No historical envelope, shared-process apportionment or positive After room has been substituted for those unknowns. **No budget, reference, move or held-budget file was changed.**
