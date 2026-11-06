+++
id = "close"
part = "close"
kind = "docs"
needs = ["last-round"]
files = ["hearth/docs/plan/hearth-speed-2/figures.md", "hearth/docs/design/domain/testing.status.md", "hearth/docs/design/domain/testing.md", "hearth/docs/design/STATUS.md"]
effort = "medium"
minutes = 40
+++

# Before and after, on one machine, and the status says the result

## Goal

The request's section 3: hearth's focused suite before and after this plan by one method on one machine, what each commit saved, and the design's status saying where it stands. **Nothing here raises the budget or changes a test.** If the aim isn't met, this is where the measurements go to the user: `skein/docs/foundation/testing-strategy.md`, 8.1 ends over budget in a ruling with measurements.

Acceptance:

- `figures.md` gains `## After`: the method's two figures at the base commit and two at the merged trunk this branch starts from, all on this one machine, in one sitting, by the driver of `notes/common.md`. If this isn't the machine `## Before` names, that's why `## Before` isn't reused: say so.
- **By family,** before beside after, user and system apart, with tests counted on each side. If the trunk merged other plans' hearth tests, say how many and what they cost. The whole inventory of (binary, test) is compared: a test gone is a red, not a saving.
- **`fuzzy` once at the tip,** by the same driver (`.measure.py fuzzy DIR`), for the status line.
- **What each commit saved,** in a table from `git log` of this plan's commits, as each commit's own message gives it. Their machines differ, so the rows aren't summed.
- **`## The answer`, its last section:** the ratio against the aim of 85%; whether the method's figure is under 30. If the aim isn't met: what's left and by how much, what's most costly now, which repairs were tried with what each saved, and any cost in a shared kit with its share, so the user can rule on the measurements. Also, either way, the inefficiencies in hearth's production code this plan found and stopped before repairing (`hearth/docs/plan/hearth-speed-2/tasks/last-round.md`'s options that weren't taken), named in a line each.
- **The queue's record after landing is out of this task's reach,** since nothing here runs after landing. `## The answer` says that in a line.
- `hearth/docs/design/domain/testing.status.md`:
  - **Its Departs on 9.1 holds this measurement,** in the file's present-tense way, with its date moved: the method in a sentence (all of hearth's workspace packages, `NEXTEST_DOUBLE_SPAWN=0`, measured as `tools/tiers/design.md`, 7.3 has it), the figures, the families. A figure measured outside the queue says so.
  - **If the method's figure is under 30,** the sentence that the suite is over its budget goes, and the status adds that the budget reads the queue's figure, which for this revision is unknown until it lands.
  - **If it's 30 or more,** that **Departs** stays, with the figure and what's left by how much.
  - **The sentence that no profile of the suite has been taken goes:** the suite's CPU by function is now known. What's still unknown stays said: the causes of variation across machines, and the large render's split, unless a task measured it.
  - **The `Not yet` entries on 9.1 this plan's design added:** each one built goes. One the plan stopped before building stays, as `Not yet`.
- `hearth/docs/design/domain/testing.md`, 11's "The focused suite's way to its figure" says what's open now and nothing built: what the profile showed that's left, if anything, stands as the open question, a cost in a shared kit among it with its measured share. It still says the figure stands and a run over it is recorded, never refused, and that it depends on the queue's record and on the user's ruling. Its other questions don't change. 9.1's list doesn't change: a repair the plan stopped before making stays in it, and stays `Not yet`.
- `hearth/docs/design/STATUS.md`'s clause on the focused suite follows the status in the same words.
- Citations at 0, and `tests/boundary/tests/outward.rs` passes. The status files name no machine and no tool of this development, and no plan: what's true now, with no history (`DESIGN.md`, 2). `tools/tiers/budgets.toml` doesn't change.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`
- `hearth/docs/plan/hearth-speed-2/inputs/request.md`, 3
- `hearth/docs/design/domain/testing.md`, 9.1 and 11, and `hearth/docs/design/domain/testing.status.md`, its entries on 9.1
- `tools/tiers/design.md`, 7, 7.3 and 10.2
- `DESIGN.md`, 2

## Owns and may change

- **Owns:** the files under `files`, in those sections alone.
- **Departs from your text:** this edits design and status files, because the request's section 3 asks it to, and adds no test.
- **A red is no one's to hide.** If a suite fails, record what failed and report it. It isn't this task's to fix.
- **A figure you can't get is `unknown`,** with why.

## Steps

1. The base's figures, then the tip's, by the method, one command at a time. `fuzzy` once at the tip.
2. The table of commits, from `git log`.
3. Write `## After` and `## The answer`. Edit the status and design files.
4. `git status` shows only the files above, with `.measure.py` deleted. `heavy cargo nextest run -p fabrig-boundary --test outward`. Citations at 0. Commit and push.

## Report

The first line's answer: the ratio, and whether the method's figure is under 30. Then the figures, the families, what's left, anything `unknown`.
