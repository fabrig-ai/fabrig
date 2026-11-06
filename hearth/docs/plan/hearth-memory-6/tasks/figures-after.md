+++
id = "figures-after"
part = "after"
kind = "docs"
needs = ["close"]
files = ["hearth/docs/plan/hearth-memory-6/figures.md", "hearth/docs/design/domain/testing.status.md", "hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 30
+++

# The gate's figures after this round

## Goal

The request's section 5: the figures after, beside `figures-before`'s, and a plain answer to whether hearth's focused suite is under the user's 30 CPU s, or what costs most if it isn't, for a small plan after this one. **Nothing here raises that figure.**

Acceptance:

- `hearth/docs/plan/hearth-memory-6/figures.md` gains an "After" section, measured in the same way as "Before", at the merged trunk this branch starts from: `default` and `fuzzy` CPU, the peaks and their ceilings (the new test's included), `max_out` and `worst_case`.
- It gives a table of before beside after, and every ceiling this round moved, with the peak that moved it and why, from `git log` of the memory tests' files on this plan's branches.
- Its last section answers: is the focused suite under 30 CPU s? If not, the largest costs, by test or by family of tests, with their CPU seconds, for a small plan after this one.
- `hearth/docs/design/domain/testing.status.md`'s **Departs** on 9.1 gives the figures of this measurement, in the file's present-tense way, and what is still unknown. If the suite is under 30, that **Departs** goes. `hearth/docs/design/STATUS.md`'s clause on the focused suite follows it.
- If a figure is `unknown`, it says why. The test code changes in no way.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/tasks/figures-before.md`: the method, and the same pre-approvals
- `hearth/docs/plan/hearth-memory-6/figures.md`, "Before"
- `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 5
- `tools/tiers/design.md`, 7, 7.3 and 10.2
- `tools/tiers/budgets.toml`: read it, and don't change it
- `DESIGN.md`, 2

## Owns and may change

- **Owns:** the files under `files`. `Departs from your text:` this edits a status file, and adds no test.
- **Same method as "Before":** the same driver and the same way to read peaks, in your slot, untracked, deleted before the push. If the machine or the method differs, say so: a comparison across them is labelled.
- **A red is no one's to hide.** If a suite fails, record what failed, and report it: that isn't this task's to fix.

## Steps

1. Measure `default`, then `fuzzy`, one command at a time.
2. Read the peaks, and restore every test file.
3. Rank the costs of `default`, by test and by family (a family: tests that run one scenario many times, or one binary).
4. Write "After", the comparison and the answer. Edit the status lines.
5. `git status` shows only the files above. Citations at 0. Commit and push.

## Report

The first line's answer: under 30 or not, with the figure. The comparison, the largest costs, anything `unknown`.
