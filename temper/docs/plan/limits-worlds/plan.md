# limits-worlds: the real loop and the worlds on it, within the suites' CPU limits

The request: `inputs/request.md`. This is the repair of `skein/docs/foundation/testing-strategy.md`, 8.1, for the worlds that run whole deployments. No budget, timeout, deadline or build setting changes, and no test moves into the focused suite.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `figures` | `tasks/figures-before.md` | | The five stories' *before* by instruction, and one ranked list of causes across the worlds |
| `cuts` | `tasks/cuts-fork.md` | | Every live commit still crashed, each forked from one baseline |
| `conformance` | `tasks/conformance-causes.md` | `cuts-fork` | What a child's replay still costs, at the harness and the adapter |
| `engine` | `tasks/engine-causes.md` | `figures-before` | The causes the worlds share, fixed in the crates under them |
| `public` | `tasks/public-causes.md` | `figures-before` | The largest focused test, at its cause in its world |
| `lower` | `tasks/lower-stories.md` | | The two behaviours only `case_6_1` and `case_6_2` hold, in the engine's domain world |
| `belonging` | `tasks/stories-go.md` | `lower-stories` | Four stories gone, and 5.3 saying what is so |
| `real-loop` | `tasks/real-loop-causes.md` | `figures-before`, `stories-go` | What a real-loop story costs in its own harness |
| `close` | `tasks/close.md` | all of the above | The account for the user: `temper/docs/development/limits-worlds.md` |

Critical path, by the specs' estimates: `lower-stories`, `stories-go`, `real-loop-causes`, `close`, about 340 minutes. `figures-before`, `engine-causes`, `close` and `cuts-fork`, `conformance-causes`, `close` are each about 315. That's nine tasks against the request's "about eight": `lower-stories` is small and has to land before `stories-go`, so it stays apart.

## Belonging

**How this was read.** Four read-only surveys listed each story's assertions and looked for another test asserting each one, in the lower worlds, the real loop's other stories and the process tier. The lead read the holders of the four that go, and the code behind the findings below. The rows that stay weren't all re-read: their only effect is that a story stays.

**A story's path counts as its own only if it crosses a boundary of the assembled stack that no other real-loop or process story crosses** (the worker, the agent's host, git, the forge's wire, a provider's wire, the real clock). Domain decisions in a new order aren't a path of their own: they're the lower world's (`temper/docs/design/domain/testing.md`, 2.4).

**"A small cost" for a lower story is taken as about 60 lines in an existing test file.** The figures in the last column are the surveys' estimates of the lines a lower story would take, and the path a story alone crosses can't move down at any cost. `close` puts them with their CPU seconds for the user.

| Story (`fuzzy::`) | CPU s | What only it holds, by reading | The process tier | Verdict |
|---|---:|---|---|---|
| `case_4_1_4_3` | | Placement on ws, s1 and s2 with s2 added late, the stall, and a timer-driven cancel on the cluster; a created subtree's authority within its parents'; a Markdown-only tree | Its documented story is gone; `a_task_runs_on_each_worker` and `a_worker_started_later_takes_the_waiting_task` hold the placement alone | **Stays.** Path its own; about 265 to 350 lines for the rest |
| `case_4_4` | | A model amendment reaching A's next query; a new delegate paying from the account a person ranked first, over the wire | Documented story gone | **Stays.** Path its own; about 95 to 145 lines |
| `case_4_5_limited` | | A provider's limit and reset on the wire, the wait, and work resumed past it with no act | Documented story gone | **Stays.** Path its own; about 80 to 130 lines |
| `case_4_5_refused` | | A provider's refusal on the wire, `account replace-key`, and work resumed on the new credential | Documented story gone | **Stays.** Path its own; about 80 to 120 lines |
| `case_4_6` | | A worker stopped under a committed, unanswered question; the retry elsewhere; one answer; a worker's two lives | Its process story exists and is `#[ignore]`d: it doesn't run | **Stays.** Path its own; about 80 to 120 lines |
| `case_5_1` | | A guide replaced on main reaching fresh runs' system text through checkout preparation; the fresh brief's rendered tail | Nothing | **Stays.** Path its own; about 120 to 180 lines |
| `case_5_2` | | A running reviewed change stopped, released, and landed; B's news once, after the release | Nothing | **Stays.** Path its own; about 110 to 180 lines |
| `case_5_4` | 6.8 | A produced change's cancel against the forge's records, at its pushed head; a held owner's overlap news once, after the release | Nothing | **Stays.** Path its own; about 80 to 140 lines |
| `case_5_5` | | A depth refusal as a hosted agent's tool output, with no task made; a fresh run recovering its plan by a note's recall | Nothing | **Stays, the closest call.** About 90 to 150 lines, and then its path is nearly held |
| `case_5_6` | | Nothing | Nothing | **Goes** |
| `case_6_1` | | One batch's two local references, as dependency facts | Nothing | **Goes,** after `lower-stories` |
| `case_6_2` | | A kept delegate's journal and committed turns, across Met | Nothing | **Goes,** after `lower-stories` |
| `case_6_4` | 6.7 | The red tip's deployment repair: requested by the deployment, first in the forge's merge order, the pause | Nothing | **Stays.** Path its own; about 60 to 105 lines for its checks |
| `case_6_5` | | The inbox whole before and after one answer; an effort's amendment reaching the next run after its decision; an advisor's report, model and one turn | Nothing | **Stays, a close call.** About 95 to 145 lines, and no path that's clearly its own |
| `case_6_6` | | Two repositories: an atomic batch across them, the early producer, the pin's bytes against the real merge | Nothing | **Stays.** Path its own; about 100 to 150 lines |
| `case_6_7` | | A deadline waking a parked task on the real clock, its hold's escalation, the split by amendment and release | Nothing | **Stays.** Path its own; about 85 to 130 lines |
| `case_6_8` | | A tester whose forge call is refused, with no effect; a message starting M2 beside live A1; the tester's report landing through a change | Nothing | **Stays.** Path its own; about 175 to 270 lines |
| `repair::a_change_failing_ci_is_repaired_and_lands` | 8.2 | **Rung B1's twin:** a real model's six recorded transcripts replayed on today's code (`temper/docs/development/acceptance.md`, 7). No scripted story can hold that. Also the repair's whole wording and a bounded result | `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` holds the repair's path, without the inbox | **Stays** |
| `green_light::a_green_light_in_the_inbox_lets_the_change_land` | 7.4 | **Rung B0's twin** (`temper/docs/development/acceptance.md`, 6). Also the `goal(advisor: true)` path, and the stock review brief as delivered | The same story holds the change's leg | **Stays** |
| `landing::a_chat_lands_its_change_on_the_forge` | | Nothing | `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` holds its path | **Goes** |
| `landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | | A landing dependency added to a report delegate, which first runs after the real landing | Nothing | **Stays,** and now holds the Forgejo landing's path. About 60 to 100 lines for its checks |
| `program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | | Claude Code as the backend through a whole change; its activations, and its Read and Edit | Nothing | **Stays.** Path its own |

A blank CPU figure is one the request doesn't name. `close` measures each. By 8.1 the scripted stories that stay don't belong in the fuzzy suite for being past a guide, and this plan moves none of them, as the request says. Where they belong is the user's next call, and `close` gives it to the user with the numbers.

### What holds them now

Each is a test by name, run and passing before its story goes (`tasks/stories-go.md`).

**`fuzzy::landing::a_chat_lands_its_change_on_the_forge`:**

- Its changed-digest and replied watch: `both_requirements_must_hold_on_the_same_delivery` (`temper/crates/temper-client-tools/src/tests/jig/wait.rs`, focused).
- The chat's ended watch and its result, "The change landed.": `github_land::a_chats_change_lands_on_connected_github` (real loop, focused).
- A green merge with the edit on main, on the Forgejo layer: `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` (real loop, fuzzy).

**`fuzzy::case_5_6`.** Its own header says it's in the fuzzy suite to make room, at about 1.56 s.

- A tracked helper at priority 1: `an_agent_changes_a_live_delegates_controls_and_replays_the_answer` (`temper/tests/engine/domain/tests/suite/root_routes.rs`, focused); `fuzzy::case_4_1_4_3` end to end.
- A timer armed through a fresh run; a person's correction waking that run with nothing of the earlier one; the cancel, its party and reason: `an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation` (`root_routes/timer_cancellation.rs`, focused), which is this story's own domain, and `a_cancel_closes_three_levels_with_runs_live_deepest_first` (`root_routes.rs`, focused).
- Nothing landed on main: this helper's turn is held and it holds no change, so it has nothing to land. A cancelled helper with a change is `fuzzy::case_5_4`, whose oracle has main without x3's file.
- A note written, searched for and recalled: `a_note_written_corrected_by_a_person_and_recalled_is_durable` (`root_routes.rs`, focused), and `fuzzy::case_4_6`, whose script recalls by search, end to end. That this one comes after a cancel is no behaviour: notes don't read cancels.
- The paths: a timer armed and read in `fuzzy::case_6_4`, a helper cancelled in `fuzzy::case_4_1_4_3`, a fresh run in `fuzzy::case_5_1`.

**`fuzzy::case_6_1`:**

- The hierarchy and its deadlines; the question and answer; complete journals: `fuzzy::case_6_5`.
- A batch's local references as dependency facts: `one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth`, new in `lower-stories`.
- Three delegates running, the fourth gated, unrelated work going on: `both_prerequisite_orders_and_completion_orders_gate_only_the_fourth_delegate` (`root_routes/added_dependencies.rs`, focused).
- The paths: a batch-local dependency in `fuzzy::case_6_6`; an unstarted dependent starting after Met in `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands`; each worker holding a run in `fuzzy::case_4_1_4_3`.

**`fuzzy::case_6_2`:**

- Numbered questions up two levels and down again, and a delegation only after the answer: `fuzzy::case_6_5`.
- Dependencies added to a live delegate, by whom, at its next revision, and kept: `a_live_delegate_is_kept_until_both_added_dependencies_are_met` (`root_routes/added_dependencies.rs`, focused).
- One Met, a new run, and its results: `a_kept_delegate_is_woken_by_its_last_dependencys_met` (`jig/tests/core/tests/stories/dependencies.rs`, focused).
- The journal as a prefix, and the committed turns at the next attempt: `a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole`, new in `lower-stories`.
- The change landing green and the delegate going on after it: `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands`.
- A resumed attempt keeping its history on the real loop: `fuzzy::case_4_6`.
- Complete journals: `fuzzy::case_6_6`.

### Found by the reading

- **The process tier no longer holds what 5.3 says.** Of the stories its rows name, only 4.6's exists, and it's `#[ignore]`d, waiting on jig's fleet. `1425451fb7` removed the others. So "the process tier also runs this case" holds for no step today, and nothing here leaned on it. `stories-go` corrects 5.3. Whether to run 4.6's again is jig's fleet's question.
- **`repair` and `green_light` are twins.** The surveys counted neither's recorded transcripts as a behaviour, and read `repair`'s domain checks as movable. They stay for that reason alone.
- **5.3's rows have drifted from the code** in 5.1, 5.2, 5.5 and 6.6: a story's name, and what its oracle reads. `stories-go` corrects them against the code.

## What this plan decided

- **Temper's sweeps fork; they don't take jig's `ColdSnapshot`.** jig has a snapshot that resumes a story at a cut, and it needs each story written as a resumable state machine (`jig/tests/conformance/src/replay.rs`, `Continuation`), and `Clone` on every peer and fake. Temper's fourteen stories are straight-line functions with about forty nested drains, on a fake forge that isn't `Clone`. Rewriting them would cost more than the repair and make them harder to read. A forked process holds the position with no rewrite, and its child goes through the same `crash()` a fresh cut does. A child's `Outcome` is compared to a fresh cut's in a focused test, and the focused cuts stay fresh. The kit is `unsafe`, in one small module on the `ring` profile (`notes/contracts.md`). Its task has a way out: if the profile shows a small prefix, it doesn't build the fork.
- **Valgrind's copy is private.** The request names callgrind, and a builder was found without it today (commit `0a4dafa2f6`). `notes/measuring.md` has each task keep a copy in its slot's `target/` with `apt-get download` and `dpkg -x`, so no machine changes. If that fails, the task asks.
- **The real loop's patience is raised locally while profiling,** and never committed. Callgrind is 30 to 60 times slower, and the 20 s patience runs on the real clock. Polls look bigger under it, so `notes/measuring.md` has them read by call count.
- **Cause tasks are cut by crate, not by cause.** The causes aren't known until a profile runs, and a lead doesn't wait for one. So `figures-before` ranks them across the worlds, and each cause task owns a layer, fixes what the list gives it, and stops at the request's 2%. A cause in another's layer goes under `## Left`.
- **D034 may be paid here.** jig's `jig/docs/development/debt/D034-world-replay-cost.md` measured the whole-store projection at each commit at about a tenth of jig's fuzzy conformance suite. `conformance-causes` pays it only if the profile puts it first and over a tenth of the story.
- **5.3's table changes only where a story goes or a row is wrong.** The rows' numbers stay. The rule "one past the guide is a fuzzy test" in 5.3 and act 3's "whatever its cost" stand until the user decides where the stories belong, and `close` records that as a **Departs**.

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, lists them.

- `figures-before` changes no code, and `cuts-fork`, `conformance-causes` and `lower-stories` are in the kits and worlds no other draft holds.
- `engine-causes` can't avoid them: a cause may sit in `temper-engine-domain/src/route/**` or `jig-protocol-store`, which `goal-delivery` and `model-catalogue-2` hold, so it keeps its hunks small and keeps their changes at a merge.
- `stories-go` meets `goal-delivery/goal-real-story` in one `mod` line of `temper/tests/real-loop/tests/suite/main.rs`, and `model-catalogue-2/process-fixtures` not at all: `tests/fixtures/**` stays, since the twins read it.
