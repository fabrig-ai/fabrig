+++
id = "story-baseline"
part = "figures"
kind = "docs"
needs = []
files = ["temper/docs/plan/browser-budget/figures/story-baseline.md"]
effort = "medium"
minutes = 30
+++

# What the four stories cost before the repair, and where it goes

## Goal

The plan's *before*: each of the four stories' CPU and wall, with the machine's load, on the trunk as your slot has it, and where the long story's CPU goes. This task changes no code. The request's table came from a whole-tier run on `668f7cd3e503`; this is the same stories alone, with the load beside each figure, which is what the repair's *after* is compared with.

Acceptance:

- `temper/docs/plan/browser-budget/figures/story-baseline.md` holds three runs of each of the four stories and their medians, in `notes/measuring.md`'s form, failed or starved runs included.
- For `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`, it holds one run's CPU split by process family, and that run's count of the harness's accessibility queries with their wall time. Both as `notes/measuring.md` says.
- `git diff` holds that one file and nothing else.

## Inputs

- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/measuring.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1.
- `tools/tiers/design.md`, 7.3 and 10.1, for what `cpu-summary` reports.

## Owns and may change

Owns the figures file in the header. Everything else stays: the sampler and the counter are throwaway and gone before the push.

Options, each decided:

- **A story fails on the trunk:** write it down with its log's path, keep measuring the others, and report it under `## Left`. It isn't yours to repair here, and a failure on the trunk doesn't stop this task.
- **The sampler accounts for less than 80% of a run's total:** write down what it did account for, and say so. Don't build a better sampler.

## Steps

1. Run the four stories as `notes/measuring.md` says, three runs each.
2. One more run of the long story, with the sampler and the counter.
3. Write the figures, delete `target/browser-budget/`, make sure the throwaway code is gone, commit and push.

## Report

The table's medians beside the request's figures, the long story's split, and your implementation and check minutes apart.
