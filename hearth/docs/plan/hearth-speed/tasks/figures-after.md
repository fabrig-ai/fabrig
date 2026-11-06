+++
id = "figures-after"
part = "after"
kind = "docs"
needs = ["repeats"]
files = ["hearth/docs/plan/hearth-speed/figures.md", "hearth/docs/design/domain/testing.status.md", "hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 30
+++

# Before and after, on one machine

## Goal

The request's "What to report": hearth's focused suite before and after this plan by one method on one machine, what each commit saved, and the status files saying where it stands. **Nothing here raises the budget or changes a test.**

Acceptance:

- `figures.md` gains an "After" section: the method's two figures at the base commit and two at the merged trunk this branch starts from, all on this one machine, in one sitting. Both go through the same driver. If this isn't the machine "Before" names, that's why "Before" isn't reused: say so.
- **By family,** before beside after, with tests counted on each side. If the trunk has merged other plans' hearth tests, say how many and what they cost.
- **`fuzzy` once at the tip,** by the same driver with `prepare(DIR, "fuzzy", WRAPPER)`, for the status line.
- **What each commit saved,** in a table, from `git log` of this plan's commits.
- **The answer, in its last section:** the ratio against the aim of 85%; whether the method's figure is under 30; if the aim isn't met, what's left and by how much, and what's most costly now.
- **The queue's record:** the figure that counts, after landing, is out of this task's reach, since nothing here runs after landing. The section says that in a line.
- `hearth/docs/design/domain/testing.status.md`'s **Departs** on 9.1 gives this measurement's figures in the file's present-tense way, and its date moves. Its fuzzy figure and its CPU sentence on the large render follow only what you measured, and what's still unknown stays said.
  - **If the method's figure is under 30,** the part of that **Departs** on the focused suite's excess goes.
  - **If it's 30 or more,** that part stays, with its figure, and adds that the budget is the queue's figure, which for this revision is unknown until it lands.
- `hearth/docs/design/STATUS.md`'s clause on the focused suite follows it in the same words, and its date moves if it has one.
- Citations at 0, and `tests/boundary/tests/outward.rs` passes. The status files name no machine and no tool of this development, and no plan: what the status says now, with no history (`DESIGN.md`, 2).

## Inputs

- `hearth/docs/plan/hearth-speed/notes/common.md`
- `hearth/docs/plan/hearth-speed/figures.md`
- `hearth/docs/plan/hearth-speed/inputs/request.md`, "What to report"
- `hearth/docs/design/domain/testing.md`, 9.1, and `hearth/docs/design/domain/testing.status.md`, its **Departs**
- `tools/tiers/design.md`, 7, 7.3 and 10.2. `tools/tiers/budgets.toml`: read it, and don't change it
- `DESIGN.md`, 2

## Owns and may change

- **Owns:** the files under `files`.
- **Departs from your text:** this edits a status file, and adds no test.
- **A red is no one's to hide.** If a suite fails, record what failed, and report it. It isn't this task's to fix.
- **A figure you can't get is `unknown`,** with why.

## Steps

1. The base commit's figures, then the tip's, by `notes/common.md`'s method, one command at a time. `fuzzy` once at the tip.
2. The table of commits, from `git log`.
3. Write "After" and the answer. Edit the two status files.
4. `git status` shows only the files above. Run `heavy cargo nextest run -p fabrig-boundary --test outward`. Citations at 0. Commit and push.

## Report

The first line's answer: the ratio, and whether the method's figure is under 30. Then the figures, the families, what's left, anything `unknown`.
