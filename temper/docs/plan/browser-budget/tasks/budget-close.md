+++
id = "budget-close"
part = "close"
kind = "docs"
needs = ["story-baseline", "history-ctl", "window-seam", "window-story", "find-backoff", "delivery-cost"]
files = [
  "temper/docs/development/browser-budget.md",
  "temper/docs/plan/browser-budget/figures/budget-close.md",
]
effort = "medium"
minutes = 30
+++

# The repair's account: before, after, and what's left

## Goal

Measure the four stories on the trunk with every lever merged, and write the account that goes to the user: a before-and-after table for each story with the machine's load, what each lever did, and, where a target isn't met, what's left and where its CPU goes, in numbers. Don't leave out a measurement because it's unflattering.

Acceptance:

- `temper/docs/plan/browser-budget/figures/budget-close.md` holds three runs of each of the four stories on the merged trunk, and the long story's CPU split, as `notes/measuring.md` says.
- `temper/docs/development/browser-budget.md` holds, in this order:
  1. **The result:** the request's figure, `story-baseline`'s median and the final median for each story, with the load, beside the request's targets: the long story about 12, `latest_returns` about 6, `earlier_words` about 5. Each target met or not, in plain words.
  2. **The whole tier isn't measured here:** after-landing proves it on the next main. Say what the four add up to now, and what that leaves the other thirteen on the request's figures (45 to 70 s), against 85 and against the target of 70.
  3. **Each lever:** what it did, in numbers, from `figures/history-ctl.md`, `figures/window-story.md` and `figures/find-backoff.md`, and whether the third landed and why.
  4. **A delivery's cost,** from `figures/delivery-cost.md`: does each delivery re-render or re-measure the whole window, what that costs a person at the whole bound, and what was or wasn't fixed.
  5. **Belonging,** from `temper/docs/plan/browser-budget/plan.md`, *Belonging*, carried over as it stands.
  6. **What's left and where it goes,** from the final split, by process family, in numbers.
- `rg -n 'Not yet:\*\* 4\.3, \*(The tier|A chat)' temper/docs/design/client/browser.status.md` and `rg -n 'window_bytes' temper/docs/design/client/README.status.md` find nothing. If one does, its task left it: remove it, and name it in your report.
- `temper/docs/development/browser-budget.md` passes `heavy cargo nextest run -p fabrig-boundary --test outward`. It names no machine, session or tool of whoever runs the plan: a machine by its CPU count and model, a run by `tools/tiers/run --cpu`.
- `python3 -I -B tools/citations/citations.py --base origin/main` finds nothing new.

## Inputs

- `temper/docs/plan/browser-budget/inputs/request.md`, *Measuring*.
- `temper/docs/plan/browser-budget/figures/`, every file, and `temper/docs/plan/browser-budget/plan.md`.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.
- `temper/docs/development/playground-figures.md`, for the shape of a figures page here.
- `skein/docs/foundation/testing-strategy.md`, 8.1.

## Owns and may change

Owns the files in the header. May remove a status entry its task left, as above. Changes no code, story, budget or timeout.

Options, each decided:

- **A target isn't met:** that isn't a question and doesn't stop this task. These are targets, not limits, and the tier as a whole isn't measured here. Write what's left, report `Done:`, and put the same numbers under `## Left`, where they reach the user. Propose nothing about a budget: that's the user's.
- **A story fails on the merged trunk:** read its log; it's in the table with that log's path. Report it under `## Left` naming the task whose change you think reached it. Don't repair it here.
- **A figures file is missing or thin:** say so in the page, in that lever's place, rather than measuring it again. Your own three runs are the only measuring this task does.

## Steps

1. Three runs of each story on the trunk, and the long story's split.
2. Write both files. Delete `target/browser-budget/`, make sure the sampler is gone.
3. The two checks above, then commit and push.

## Report

The result's table as it stands in the page, what's left, and your implementation and check minutes apart.
