+++
id = "close"
part = "close"
kind = "docs"
needs = ["figures-before", "cuts-fork", "conformance-causes", "lower-stories", "stories-go", "engine-causes", "public-causes", "real-loop-causes"]
files = [
  "temper/docs/development/limits-worlds.md",
  "temper/docs/plan/limits-worlds/figures/close.md",
  "temper/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 75
+++

# What's left, for the user

## Goal

The account the request's section 4 asks for, with the figures taken on the plan's own trunk: each world's tests before and after, in instruction counts for the ones profiled; the causes found and not fixed; and what stays in the fuzzy suite and past its guides, with its numbers, for the user to decide. It changes no code.

Acceptance:

- `temper/docs/development/limits-worlds.md` holds:
  - **Before and after:** the request's table of worlds, with an *after* column. Each row holds the world's tests and its CPU seconds, from `figures/close.md`. A row whose before and after were measured differently says so.
  - **By instruction:** every story profiled, by name, first figure and last, with the figures it comes from.
  - **What went:** each story that went, what holds its behaviours, by test name, as `plan.md` has them. **What stays:** each story of the belonging table that stays, what only it holds, its CPU seconds and its instructions if profiled.
  - **Causes:** those fixed, with what each saved, and those found and not fixed, gathered from every figures file and from the tasks' `## Left`, each with its share and why it stands. A cause a profile showed no test can avoid says so.
  - **The suites against their limits:** temper's fuzzy against 120 and focused against 30, the real loop's focused against its 15 s ceiling, by the measured figures. If either stands over, say so plainly, and name what a next step could be and what it would need from the user. A budget is the user's.
- `temper/docs/plan/limits-worlds/figures/close.md` holds the runs behind it, in `notes/measuring.md`'s form.
- `temper/docs/design/domain/testing.status.md` holds an entry for what's over a limit today, as a **Departs** at 6.1 citing the account, or none if nothing is. It holds a second **Departs** at 5.3 and 6.1: the scripted stories that stay in the fuzzy suite, there because they were past the guide, which `skein/docs/foundation/testing-strategy.md`, 8.1, says isn't a reason; where they belong is the user's to decide, and the account says so with their numbers. A **Not yet** this plan wrote that still stands is your report's first item under `## Left`.
- `git diff` against your base holds those three files and nothing else.

## Inputs

- `temper/docs/plan/limits-worlds/inputs/request.md`, its section 4 and its table.
- `temper/docs/plan/limits-worlds/plan.md`, the belonging table, and `temper/docs/plan/limits-worlds/figures/*.md`.
- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1; `temper/docs/design/domain/testing.md`, 6.1.
- `temper/docs/development/browser-budget.md`, for a landed account's form.
- `DESIGN.md`, sections 2 and 4: a report goes in `docs/development/`, and a status file records gaps as they stand.

## Owns and may change

Owns the three files in the header. A departure from the implementer's text: `testing.status.md`'s entry is yours to write.

Options, each decided:

- **A world's CPU:** measure each row of the request's table as testing.md, 6.1, compares guides, with its own profile and `-j 1`, three runs and their median: `heavy bash -lc '/usr/bin/time -f "%U %S %e" cargo nextest run -p <package> --profile <default|fuzzy> -j 1 --no-fail-fast'`, once it's built. That includes nextest's own small cost and every test's descendants, and the request's figures are medians of gate runs: say so beside the column. The machine's load goes with each figure.
- **The whole suites:** the trunk's end check and the queue's gate reports measure them, and you don't run them. Use what your slot can read of them if there is one, and otherwise sum what you measured and say so. Never `tools/tiers/run`.
- **A cause task reported nothing fixable:** it's still a row, with its figures.
- **A slow tier's budget:** isn't read in a plan's run. Say that the browser, process and live tiers weren't run here, and that after-landing will say.
- **A figure you can't get:** `unknown`, never a guess.

## Steps

1. Read every figures file and `plan.md`.
2. Re-profile every profiled story that still exists, once on simulated time and twice on the real loop, as `notes/measuring.md` says.
3. Measure the worlds, as above.
4. Write the account, restore `PATIENCE`, delete `target/limits-worlds/`, run `python3 -I -B tools/citations/citations.py --base origin/main`, commit and push.

## Report

The account's first table, the suites against their limits, what stands for the user, and your implementation and check minutes apart.

## Rebase (2026-11-07, step merge, foreman limits-worlds/close/foreman-1)

reconcile the testing status conflict while preserving both plans’ findings
Rebase `limits-worlds/close` onto main; the conflict is in `temper/docs/design/domain/testing.status.md`.
Main commit `1096b2385590f907c86ebabb5ee5111e4324a5b9` adds the `limits-heavy` account and updates the same budget paragraph that close replaces.
Preserve main’s paragraph on the `limits-heavy` repairs, unexplained 19.3 CPU s and deferred causes.
Preserve its **Not yet:** 2.6 entry for the hangup/rebind and Linux killed-box stories: compiled, runtime results absent, process-tier budget fit unknown.
Replace the overlapping **Not yet:** 6.1 paragraph with close’s measured-budget **Departs:** 6.1 entry, retaining its qualification that whole-suite totals remain unknown.
Keep close’s separate **Departs:** 5.3 and 6.1 entry for the eighteen surviving fuzzy real-loop stories and the user’s placement decision.
Keep unrelated status entries intact; no measurement rerun is needed for this textual conflict.
Run `python3 -I -B tools/citations/citations.py --base origin/main` after resolving.
