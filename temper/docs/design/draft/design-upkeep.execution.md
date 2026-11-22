# design-upkeep: execution

2026-11-21. How [design-upkeep.md](design-upkeep.md) gets carried out: one mechanical change and two goals, each started on the user's word. This file designs nothing: each milestone writes its own design from the draft as it runs. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Needs | Its figure after landing | State |
|---|---|---|---|---|
| U1, the words: `DESIGN.md`, 2, and fabrig's goal charter no longer ask a goal for `STATUS.md`'s summaries. `DESIGN.md`, 4's `STATUS.md` bullet says no goal writes the page, and its reasons bullet puts a draft's new reasons beside it. `DESIGN.md`, 5, gives the mark, the pieces under 40 KB, the numbers that stay with a line where each section went, and no text cut to fit. Text alone, landed as a mechanical change. Then, on dev.fabrig.ai, fabrig's charter set applied and the live goals moved to its revision | 3.4, 4.1, 4.2, 6 | none | the goals landing after it that still edit a `STATUS.md` | landed, 2026-11-21 |
| U2, the page and the roll-up: `tools/citations/status.py`, with `--page`, and its tests (3.2). The unread status files brought to the form. Each tree's `STATUS.md` statements sorted (3.1) and the page replaced. The page check and the limit's new refusal in the citation checker (3.3, 4.2). `DESIGN.md`, 4's bullet naming the script, and the lines in temper's design README, 6, the trees' READMEs, hearth's `AGENTS.md` and the debt READMEs that say what the page holds. `client/web/mockups/STATUS.md` unchanged. One goal | 3, 4.1, 4.2, 5 | U1, applied on dev | the roll-up's output at its landing | landed |
| U3, the first split pass: temper's files over 48 KB split along their concepts, each piece under 40 KB and a map for each, one area's batch per change, the changes one after another. `draft/notes.md` split into a notes file beside each draft. The drafts that are done archived or deleted. One goal | 4.2, 4.3, 5 | U2 | each batch under the mark; the roll-up's due list for temper at its landing | not started |

## 2. Order

- **U1, then U2, then U3,** each on the user's word. U1 is text alone and lands as a mechanical change. Applying fabrig's charter set on dev and moving the live goals to its revision is ops', on the user's word. U2 starts once that's done, since a goal that was still asked for the page's summaries would run into the page check.
- **No priority mark:** the design's upkeep isn't one of v0.1.0's critical pieces. U2 and U3 start when places are free and marked work isn't about to arrive.
- **U3 while goals run:** its splits move text that goals in flight may cite or edit. The line left where each section was, and each split's map, let those goals find it (`design-upkeep.md`, 4.2 and 4.3).
