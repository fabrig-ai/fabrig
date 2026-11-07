+++
id = "limits-close"
part = "close"
kind = "docs"
needs = ["github-runs", "lib-counts", "sweeps", "machine-costs", "memory-worlds", "scenario-worlds"]
files = [
  "skein/docs/plan/skein-limits-heavy/plan.md",
  "skein/docs/development/skein-limits-heavy.md",
]
effort = "medium"
minutes = 40
+++

# The repair's account: the table and the report

## Goal

Build `plan.md`'s table from each task's figures, measure the largest test of each row once more on the merged trunk, and write the report for the user: before and after in instruction counts, what moved and what fell, and the causes found and not fixed. Don't leave out a figure because it's unflattering.

Acceptance:

- `skein/docs/plan/skein-limits-heavy/plan.md`, *The table*: one row per test touched, from every `figures/*.md`: the test, its before and after in instructions, its cause and what changed.
- `plan.md`'s *Tests that go* gains the commit each holder passed at, from the figures, and any other test that went.
- `skein/docs/development/skein-limits-heavy.md` holds, in this order:
  1. **The result:** a line for each of the request's rows and its sweeps: the request's CPU s, the largest test's instructions before, after, and on the merged trunk, and an estimate of the row's CPU s after, as its CPU s times after over before, marked as an estimate. Then the total, against the request's aim of at least 6 CPU s, marked likewise. The gate's own reading comes after landing, and isn't this page's.
  2. **What moved to fuzzy and which counts fell,** each with its reason and what fuzzy gains, from `plan.md` and the figures' *Counts*.
  3. **Each row:** the causes found and what fixing them did, in numbers.
  4. **Found and not fixed:** every such line from the figures, with its count and what fixing it would take, largest first.
  5. **What stays heavy and why,** in numbers, where the rule says the cost is what the test is for. Propose nothing about a budget: that's the user's.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes. The page names no machine, session or tool of whoever runs the plan: a machine by its CPU count and model.
- `python3 -I -B tools/citations/citations.py --base origin/main` finds nothing new.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, its table and item 5.
- `skein/docs/plan/skein-limits-heavy/figures/`, every file.
- `skein/docs/plan/skein-limits-heavy/plan.md` and `skein/docs/plan/skein-limits-heavy/notes/common.md`, *Measuring* and *The figures*.
- `skein/docs/foundation/testing-strategy.md`, 8.1.

## Owns and may change

Owns the files in the header. Changes no code, test, budget or timeout. No other part of `plan.md` changes, but for the additions above and a line saying the plan is complete.

Options, each decided:

- **A figures file is missing or thin:** say so in the page, in that row's place, and measure only that row's largest test. Don't redo its task's work.
- **A trunk figure is worse than its task's after:** that's a finding, and goes in the page with both numbers. Read the profile enough to say which merge moved it. Don't repair it here.
- **A test fails on the merged trunk:** read its log and report it under `## Left`, naming the task whose change you think reached it. Don't repair it here: the trunk's end check turns a red into a repair task.
- **A row's next cause is still over 0.05 CPU s:** that's a line in *Found and not fixed*, and the report's first line stays `Done:`.

## Steps

1. Read every figures file.
2. On the trunk, one callgrind run of the largest test still in the focused suite in each row, as `notes/common.md` says.
3. Write the table, *Tests that go* and the page.
4. Delete `target/skein-limits-heavy/`, run the two checks above, then commit and push.

## Report

The result's table as it stands in the page, the total, what's left, and your implementation and check minutes apart.
