+++
id = "close-figures"
part = "figures"
kind = "docs"
needs = ["service-fixture", "setup-steps", "view-words", "domain-fixture", "forge-routes-steps", "client-world", "suite-startup"]
files = [
  "temper/docs/plan/limits-fixtures/figures.md",
  "temper/docs/plan/limits-fixtures/plan.md",
]
effort = "medium"
minutes = 60
+++

# close-figures: before and after, per row of the request

## Goal

The request's item 3: before and after, per row, in instruction counts, with the cause of each and whether it's fixed, and a line for each cause not fixed. Each task measured from where it started, and later tasks started from earlier tasks' savings, so this measures every sampled test twice more on one machine: at the plan's base and at the trunk's tip.

**Acceptance:**

- `temper/docs/plan/limits-fixtures/figures.md` has one table with every row of the request's table and every test named in `figures/*.md`: `Ir` at the base, `Ir` at the tip, and the change in percent. The machine, the two commits and the date head it.
- It has one table of causes, gathered from the tasks' figures: the cause, where, what it cost, the task that fixed it or what fixing it would take.
- `plan.md` gets a `## Before and after` section: a line per row with its result, a line naming each cause not fixed, and `figures.md` by path.
- The checks of what hasn't moved (steps 3 and 4) are in `figures.md`, with their results.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: how to profile
- `temper/docs/plan/limits-fixtures/inputs/request.md`, item 3 and its table
- `temper/docs/plan/limits-fixtures/figures/*.md`: each task's figures

## Owns and may change

- **Owns:** `temper/docs/plan/limits-fixtures/figures.md`, `plan.md`.
- **Departs from your text:** you may change `docs/design/`, for one thing only. A task's report says a public type changed and which design section names it: that section, and its status file if it has a gap, get the line that says what's true now (`DESIGN.md`, 2). Nothing else in `docs/design/` changes, and no other file changes. No code changes here.
- **If a sampled test isn't at the base** (another plan added it) or isn't at the tip (another plan renamed it), its figure is `unknown` on that side, with why; take its new name if you can see it.
- **If the tip's reading is higher than a task's own after,** another plan's merge may have raised it. Say what you can see, by `git log`, and leave it: fixing it isn't this task's.
- **If a test has gone from one of these binaries,** by step 3, and it's this plan's doing, that's a red: first line `Failed:`, naming it. If another plan removed it, name that plan's commit.

## Steps

The plan's base is `7ffd748466`, the main it started from.

1. **At the tip:** your branch's start. For each test named in `figures/*.md`, take its `Ir` by the method in `notes/common.md`, one at a time, with `--trace-children=yes` for `temper`'s.
2. **At the base:** `git checkout --detach 7ffd748466` in your slot, and take the same counts. Then `git checkout` your branch again. Your figures stay untracked across the checkout, or are written after it.
3. **What hasn't moved, in names:** at each commit, `TEMPER_BUNDLE=none heavy cargo nextest list -p temper-service -p temper-client-view -p temper-engine-domain-world -p temper-client-world -p temper --message-format json`, kept under `target/lf/`. Every test name at the base is at the tip, or is accounted for as above.
4. **What hasn't moved, in settings:** `git diff 7ffd748466 HEAD -- .config/nextest.toml tools/tiers/budgets.toml`, and any `[profile` line in the diff of a `Cargo.toml`. Each change names its commit and plan. This plan's tasks should have made none: if one did, first line `Failed:`, naming it.
5. Write `figures.md` and `plan.md`. Citations (`python3 -I -B tools/citations/citations.py --base origin/main`) list nothing of yours. Commit and push.

This task runs no test but for the counts above, adds no test and changes no code: of your text's end-of-run steps, only the citations and `git status` apply.

## Report

The per-row table, the causes not fixed, and the result of the two checks.
