# Failure worlds: the room F7 made, measured

2026-11-21. temper's focused (`default`) and `fuzzy` suites, by layer and by affected binary, from the suites' own reports, before and after F7 of `temper/docs/design/draft/failure-worlds.execution.md`, section 1: the fuzzy copies of focused tests retired, and the real-loop and one-process stories cut to what only their tiers show, their other assertions held below (`temper/docs/design/testing/checks.md`, section 1). It follows the plan in `failure-worlds-budget.md`, whose figures were unmeasured, and moves no figure in `tools/tiers/budgets.toml`: the moves at its end are proposals for the user.

Only temper's tests change in F7, so only temper's packages are measured. The other four trees run the same code before and after.

## Method

- **Selection:** every package under `temper/` that `cargo metadata --no-deps --format-version 1` lists, through `tools/tiers/one-process default -p …` and `tools/tiers/one-process fuzzy -p …`, from the checkout's root, with `TEMPER_INTERNAL_BUNDLE` and `TEMPER_BUNDLE` both `none`. The hold is taken as the tool takes it, never bypassed. No slow profile, browser, walk or throwaway driver runs.
- **Figures:** each run's `results.json` (`tools/tiers/design.md`, 11.8): CPU is user plus system seconds of the `tests`, `fresh` and `alone` processes, waited-for descendants included, placed by the record's own `tree` and `layer`, and checked against `cpu-summary`'s totals. No test's CPU is apportioned from a shared process. The ceiling is `tools/tiers/budgets.toml` with its moves applied once (`tools/tiers/budgets.md`, sections 2 and 10), as `cpu-summary` gives it. Instruction counts stand beside CPU where the run counted.
- **Each run's record:** its commit, machine, lanes, wall, and every failed, starved, cancelled or unobserved test by name. A binary with one is **partial**, and its figure a lower bound.

## Before

At the commit F7 started from, before any of its changes. Filled by the before run.

## After

At F7's head once all of its changes are in. Filled by the after run, with each layer's and each affected binary's difference from the before figures.

## The moves for the user

Filled by the after run: for each suite, the moves (`from`, `to`, `cpu_s`, `why`, `tools/tiers/budgets.md`, 10) that the measured differences would support, strictly down `tools/tiers/layers.toml`'s order, for the user's decision. None is written to `tools/tiers/budgets.toml`.
