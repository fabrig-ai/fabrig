# live-change-call: the plan

2026-11-07. From `inputs/request.md`: live-temper's `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` fails with "the chat's committed change call succeeded" unmet.

## Item 1: what the check reads, and why it's unmet

Read from the code; the run's log isn't in the repository.

- **What it reads.** `check_landing` (`temper/crates/temper/tests/support/forge_land.rs`) requires `change` among the chat's own committed calls that succeeded: `conversation::Evidence::tools`, filled from `task activity CHAT --json` by `successful_tools` (a call block with `failed == false` and an outcome). It reads the chat's activity alone, never a delegate's.
- **Why it's unmet.** The chat (task 1) called `goal`, not `change`, and its goal (task 2) called `change` (task 3). So `change` is in task 2's activity, never task 1's. The referee took the chat's one delegate, the goal, for the change (`find_delegates`: `self.tasks.change = delegates[0]`, executor never looked at). It watched the goal to its end, and then failed on a check whose words name nothing it saw. The goal ended within the story's 105 s, or the referee's `task show 2 --until ended` would not have been met. So "the goal then waited on nothing" is a goal parked between its change's start and its result; nothing here shows a fault in that.
- **Whose cause.** Not the story's. The check is current: the design's chat chooses `change` for a plain change (`temper/docs/design/domain/testing.md`, 2.7; `agent.md`, 4.3.1 and 4.3.2; the stock `chat.txt`: "Call goal only when they want to hear a plan or give a go-ahead ... or when they want a survey that changes nothing"). The goal-as-designer draft moves a goal's own landing, not a chat's. Not a fault in temper's landing either: the process tier's `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` runs chat → `change` on the owned Forgejo, and the root admits a chat's `goal` without `agree` by design. It is the model's choice (`gpt-6-luna`, the cheapest, since `fade293c2d`), against its instructions. temper's part in it is `goal`'s declaration: "Have one goal in your words tracked and reported; it may plan, ask you and call change". That says what a goal does and never when to start one, where `delegate`'s says "For a change to a repository, call change." Whether the chat's `change` was refused before its `goal` is unknown without the transcript. After this plan, the referee's failure says so.
- **Below the live tier.** A model's choice doesn't reproduce on a scripted model. What reproduces is the referee's half: `referee-words` commits red first a focused-suite row in which a chat's goal delegate is taken for the change, and one in which a missing `change` names no call.

## Parts and tasks

| Part | Task | Needs | Request item | Crates and files |
|---|---|---|---|---|
| `referee` | `referee-words` | none | 1 (the referee's half), 4 | `temper` test support: `tests/support/forge_land.rs`, `forge_land/conversation.rs`, `tests/suite/forge_land.rs` |
| `words` | `run-words` | none | 2, 3 | `temper-engine-smith`: `calls.rs`, `answers.rs`, `tests.rs`; `docs/help/reference/host-tools.md` regenerated |

- **`referee-words`:** `testing.md`, 2.7's new lines. The chat's one delegate must have a procedure executor; an agent's task (a goal) fails the story at once, naming the task and its charter. With no successful `change`, the failure names the chat's committed calls, those that succeeded and those that failed. Rows in the suite's existing referee tests, red first. The live story is compiled, not run.
- **`run-words`:** `agent.md`, 4.3.2's words for `goal`'s declaration: call it only for agreed work or a survey; for a change to a repository, call `change`. Also `agent.md`, 4.3's next reads after a refused branch read: `read_branches` `Empty` names a nonempty prefix, `read_settings` and `read_branch`; `read_branch` `Missing` names `read_settings`, `read_branches` with a prefix, and when a goal's or run branch exists. `read_branches`' `prefix` is declared nonempty. A table test of refusals and a declaration test, red first; the help page regenerated.

They share no crate and no file and start at once.

## Item 3: `read_branch` `Missing` and `read_branches` `Empty`

Read, not run (survey 2). Neither is a fault.

- `read_branches` with an empty prefix is refused `Error::Empty` by skein's design (`skein/docs/design/forge.md`, section 9; `skein-forge`'s Forgejo read plan), before any request. The declaration let a run send `""` ("The names' prefix.").
- A chat's `goal` holds no goal branch (`temper-engine-domain/src/route/goal.rs` builds it with no symbolic grants; stock template 2 names none). A charter 1 run has no `modify`, so it never pushes a run branch either. A goal reading `temper/<root>/g<task>` or `temper/<root>/r<task>` is rightly answered `Missing`. Which name it read is unknown without the log. Either way the words gave it nothing to read next, and the run-words task gives it that.

## Critical path

`run-words`, about 75 implementer-minutes, beside `referee-words`, about 60. The plan lands when both have merged and the trunk's end check is green. After landing, the live story shows whether `gpt-6-luna` now chooses `change` (`agent.status.md`, 4.3.2's entry on this request). If it still calls `goal`, the referee stops at once and says so, with the chat's calls.

## Decisions

- **The check stays.** The design has the chat choose `change`, and the story is the one place a real model's choice is held to it. The referee is made to stop at the wrong delegate and to say what it saw, not to accept the goal path.
- **A declaration's words, not a rail.** A root refusal of a chat's `goal` without `agree` would also refuse the survey goals the chat's words allow, and recorded real-loop stories run a chat's goal (`green_light`, B1). So the fix steers the model where it chooses, in `goal`'s declaration, as `delegate`'s already does.
- **The refusals keep their kinds.** `CallAnswer` and the store's records don't change. The next read is named by the renderer from the tool's name, as `read_compare`'s recovery already is. No stored format, protocol or wire change.
- **No live run in this plan.** The request allows running the live story, at most three times. Each task's failing test sits in a lower tier, and no step needs the live run. After-landing runs the live story.
- **Nothing dropped:** four items in two tasks.

## Overlaps

None with the drafts beside this one (`per-wake`, `tui-connected`, `tui-approval`: jig and skein). `temper-engine-domain` and `forge_route.rs` are untouched. Both tasks write status files under `temper/docs/design/domain/` (`testing.status.md`, `agent.status.md`), each removing only its own entries.
