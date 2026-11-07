+++
id = "heavy-close"
part = "close"
kind = "docs"
needs = ["box-lifetime", "fake-backends", "view-limits", "forge-rate", "gallery-print", "scale-restart", "small-rows"]
files = [
  "temper/docs/plan/limits-heavy/plan.md",
  "temper/docs/development/limits-heavy.md",
]
effort = "medium"
minutes = 45
+++

# The repair's account: the table and the report

## Goal

Build `plan.md`'s table from each task's figures, measure the largest test of each row once more on the merged trunk, and write the report for the user: before and after in instruction counts, what moved and what that costs, and the causes found and not fixed. Don't leave out a figure because it's unflattering.

Acceptance:

- `temper/docs/plan/limits-heavy/plan.md`, *The table*: one row per test touched, from every `figures/*.md`: the test, its before and after in instructions, its cause and what changed.
- `plan.md` gains *Tests that went*: each that went, by test name, with what holds it now and the commit it passed at, from the figures. If none went, it says so. `a_hangup_stops_a_box_and_leaves_nothing_running` and `a_killed_box_takes_its_steps_with_it` aren't among them, since they moved, and *What the move costs the process tier* stays, with `box-lifetime`'s finding on their overlap added.
- `temper/docs/development/limits-heavy.md` holds, in this order:
  1. **The result:** a line for each of the request's rows and its four named tests: the request's CPU s, the largest test's instructions before, after, and on the merged trunk, and an estimate of the row's CPU s after, as its CPU s times after over before, marked as an estimate. Then the total for the rows, against the request's about 89, marked likewise. The gate's own reading comes after landing, and isn't this page's.
  2. **What moved,** and what it costs the process tier, from `plan.md`. Those two tests were compiled, not run, here.
  3. **Each row:** the causes found and what fixing them did, in numbers.
  4. **Found and not fixed:** every such line from the figures, with its count and what fixing it would take, largest first.
  5. **What stays heavy and why,** in numbers, where the rule says the cost is what the test is for. Propose nothing about a budget: that's the user's.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes. The page names no machine, session or tool of whoever runs the plan: a machine by its CPU count and model.
- `python3 -I -B tools/citations/citations.py --base origin/main` finds nothing new.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`, its table and item 3.
- `temper/docs/plan/limits-heavy/figures/`, every file.
- `temper/docs/plan/limits-heavy/plan.md` and `temper/docs/plan/limits-heavy/notes/common.md`, *Measuring* and *The figures*.
- `temper/docs/development/browser-budget.md`, for the shape of such a page here.
- `skein/docs/foundation/testing-strategy.md`, 8.1.

## Owns and may change

Owns the files in the header. Changes no code, test, budget or timeout. No other part of `plan.md` changes, but for the additions above and a line saying the plan is complete.

Options, each decided:

- **A figures file is missing or thin:** say so in the page, in that row's place, and measure only that row's largest test. Don't redo its task's work.
- **A trunk figure is worse than its task's after:** that's a finding, and goes in the page with both numbers. Read the profile enough to say which merge moved it. Don't repair it here.
- **A test fails on the merged trunk:** read its log and report it under `## Left`, naming the task whose change you think reached it. Don't repair it here: the trunk's end check turns a red into a repair task.
- **A row's next cause is still over 0.2 CPU s:** that's a line in *Found and not fixed*, and the report's first line stays `Done:`.

## Steps

1. Read every figures file.
2. On the trunk, one callgrind run of the largest test still in the focused suite in each row, as `notes/common.md` says. No slow tier: the two moved tests aren't run.
3. Write the table, *Tests that went* and the page.
4. Delete `target/limits-heavy/`, run the two checks above, then commit and push.

## Report

The result's table as it stands in the page, the total, what's left, and your implementation and check minutes apart.
