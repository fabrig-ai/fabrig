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

At F7's head once all of its changes are in. Filled by the after run, with each layer's and each affected binary's difference from the before figures.

## The moves for the user

Filled by the after run: for each suite, the moves (`from`, `to`, `cpu_s`, `why`, `tools/tiers/budgets.md`, 10) that the measured differences would support, strictly down `tools/tiers/layers.toml`'s order, for the user's decision. None is written to `tools/tiers/budgets.toml`.
