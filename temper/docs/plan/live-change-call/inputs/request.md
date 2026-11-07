# live-change-call: a chat's change lands on the owned forge with a real model

live-temper's `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` fails at `temper/crates/temper/tests/live.rs:110` with "the chat's committed change call succeeded" unmet, at `21a9819cc3`, after `forge-read-answer` stopped `serve` panicking in it. The story's own check is in `temper/tests/support/forge_land` (`run_live`). Its log is outside this repository, so the words here are what you have. **Run, unless marked read.**

**What the log shows.** The chat (task 1) made a goal (task 2), which made one change (task 3), "Append the exact line `Hello from temper, ...` to the end of README.md", funded by task 2. Task 3 ended about a minute later with nothing spent of its own and 53 spent below, and the forge's runner took three CI jobs on the repository. The goal then waited on nothing. Along the way the goal's reads were refused twice: `read_branch` "forge read failed" `Missing`, and `read_branches` `Empty`. A walk of the goal-as-designer charters met the same `read_branches` `Empty`. The story ran 105 s.

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **Find what the story's check reads and why it's unmet.** Which call the check means by "the chat's committed change call", what the run did instead, and whether the cause is temper's (a change's landing, a goal's or chat's tools, a forge read), the story's (a check that no longer matches what a chat does since the goal took over landing), or the model's. Reproduce it below the live tier where the cause allows: the real loop's scripted model, or the engine world.
2. **Fix the cause**, with its failing test first in the lowest tier that holds it.
3. **`read_branch` `Missing` and `read_branches` `Empty`:** say whether either is a fault, as a branch the goal expected (its own goal or run branch, or main) wasn't found or a listing came back empty, and fix it if it is, or say why the refusal is right and make its words say what to read instead.
4. **The live story's own words:** if the check is stale, bring it current, and make its failure say what it saw.

**You may run this live story by name, alone, to reproduce and to prove the fix** (`AGENTS.md`, 5: a red's repair). It spends a provider's credit; run it as few times as the fix needs, at most three. If your builder has no provider login for it, stop and say so: don't move or copy a login. Everything else runs focused and fuzzy alone.

## Not in this plan

The Claude Code twin's own failures, unless the same cause; a box's surviving child after the story, which is another owner's.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first. A stored format or protocol change is a question first; both main areas write `temper-engine-domain` and `forge_route.rs`. Formats change in place before v0.1.0. `walk = false`.
