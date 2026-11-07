+++
id = "close"
part = "close"
kind = "docs"
needs = ["public-wire", "one-process-sweeps", "state-hash", "engine-sweeps", "under-causes"]
files = [
  "temper/docs/development/limits-fuzzy.md",
  "temper/docs/plan/limits-fuzzy/figures/close.md",
  "temper/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 60
+++

# The account: temper's fuzzy suite against its limit

## Goal

The report the request's point 4 asks for, with figures taken on the plan's own trunk: each sweep touched, before and after in instruction counts, the causes not fixed, and where temper's fuzzy suite now stands against 120 CPU s. It changes no code.

Acceptance:

- `temper/docs/development/limits-fuzzy.md` holds:
  - **Each sweep touched,** by test name: its worlds or inputs drawn, its instructions before and after, and the figures file they come from. Those counts come from the tasks' figures: re-profile a sweep only if a later task changed its path, and say so. The worlds or inputs drawn are the same number on both sides.
  - **The worlds,** the request's table with an *after* column: tests and CPU s per world. A row whose before and after were measured differently says so.
  - **Causes:** those fixed, with what each saved, and those found and not fixed, gathered from every figures file's `## Left` and `## Stopped at`, each with its share and why it stands. A cause a profile showed no test can avoid says so.
  - **Against the limit:** the temper fuzzy total against 120, with the request's aim of about 15 out beside it. State the margin and how far these figures swing between runs. Say plainly whether it holds. The user turns limits on once a tree holds them, by a gate entry in `tools/tiers/budgets.toml`: this plan adds none, and says what the figures support. If it doesn't hold, name what a next step could be and what it would need from the user. A budget is the user's.
- `temper/docs/plan/limits-fuzzy/figures/close.md` holds the runs behind it, in `notes/measuring.md`'s form.
- `temper/docs/design/domain/testing.status.md`: in its **Departs** at 6.1, the sentence on the fuzzy request-world sum of 130.202313 CPU s says what's so now and cites this account, or goes if the suite holds. "The whole-temper suite totals remain unknown" comes to say that of the focused suite alone. Its sentences on the focused suite, and the second **Departs** on the scripted stories, otherwise stay as they are.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`, `temper/docs/plan/limits-fuzzy/inputs/request.md`.
- Every file in `temper/docs/plan/limits-fuzzy/figures/` and your needs' reports' `## Left`, where you can read them.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `temper/docs/design/domain/testing.md`, 6.1.

## Owns and may change

Owns the account, its figures and that status entry. It changes no code, no test, and no budget. A red it meets is under `## Left`, and the trunk's end check takes it.

## Steps

1. Build, then run temper's fuzzy suite alone, three times serially, warm, under `notes/measuring.md`'s wrapper:

   ```sh
   heavy bash -lc '<wrapper> cargo nextest run --workspace --profile fuzzy -j 1 --no-fail-fast \
     -E "package(/^temper(-|$)/) | package(=git-transcripts)"'
   ```

   This is the one departure from the implementer's text, which keeps a task from the whole workspace: the profile's default filter still applies, so this selects temper's fuzzy suite and nothing else, and the limit is that suite's. Check the count. Take the median by CPU, and give every run, its passed tests and its load.
2. Run each of the request's five worlds three times the same way (`-p <package> --profile fuzzy -j 1 --no-fail-fast`), for the table.
3. Say how these compare with the gate's figures: these are serial and include the driver, and the gate's are per test at eight at once. The request's *before* is the gate's. Whether a gate run holds it is the queue's record to show, and the trunk's end check gives the first one.
4. Write the account. `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

The suite's three runs, the margin, whether it holds, what's left, and your minutes.
