# repairs: status

2026-11-12. Where the tree's code lags or departs from [repairs.md](repairs.md).

- **Not yet:** "A resolution", its reads on a forge catching up, and "A brief section's read that fails", a files read answered `Stale`: the brief counts it permanent, `the forge refused this read`, so a head the forge moves between the pull's read and its files' holds the task for good; and the forge world's fake answers a pull's files at the moved head, which skein's client refuses `InvalidAnswer` (`temper-engine-domain-forge/src/brief.rs`, `brief_answer`; `temper/tests/engine/forge/src/translate.rs`) [in flight: change 1229].

- **Not yet:** "Stalls" and "A stall's hold says where": a `Stalled` hold names neither the step nor what it waits on, and a goal's landing can sit at updating with no hold (`temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`; `temper-engine-domain-forge-change/src/domain.rs`, `updating`) [in flight: change 596].
- **Not yet:** "A repair", a runner's failure: the brief does not say a failure was the runner's, nor how many times its job ran again at the head, and such a failure is repaired, not run again (`temper-engine-domain-forge/src/brief.rs`) [in flight: change 601].

- **Not yet:** "A pull section is pinned at the change's head" and "A brief section's read that fails": a pull past the pin fails the brief, transient, whoever pushed it, the task's own earlier attempt included, until the task is held; a lost reread superseded by a later attempt of the same task records no head; the `brief read refused` line names no reason or heads, `error=None` for a refusal the forge answered (`temper-engine-domain-forge/src/brief.rs`, `brief_answer`) [in flight: change 262].
- **Not yet:** "A release", released from a resolution or a merge job that failed or was cancelled: the change keeps `Resolving { requested: true }` and holds again on the ended delegate's status (`temper-engine-domain-forge-change/src/domain.rs`, `released_state`) [in flight: change 263].

- **Not yet:** "A resolution", its "It waits for its producer": a conflict met while a task that pushes the change's branch still lives is resolved at once, its pull pinned at the last head the change knew, so a push by that task fails every attempt of the resolution's brief, `the change's head moved` (`temper-engine-domain-forge-change/src/domain.rs`, `conflict_due`; `temper-engine-domain/src/assemble.rs`, `finish_brief_plan`) [in flight: change 37].

- **Not yet:**
  - the whole document: the request-to-repair fuzzy coverage remains `gates.status.md`, 1.
  -  the real-process update-recovery trial remains `../README.status.md`, 6 and 10.
  - `README.md`, 2, step 3: the base-first deployment proof remains `README.status.md`, 2, step 3.
  - the whole document: the repair-cause and cleanup deployment proof remains `../branches.status.md` (Not yet, the whole document).
  - `questions.md`, 1, *Which target*: the real-model question-target proof remains `questions.status.md`, 1, *Which target*.
  - `questions.md`, 1, *The next producer merges the landing tip first*: the resumed-producer deployment proof remains `questions.status.md`, 1, *The next producer merges the landing tip first*.
  -  the browser readoption story remains `gates.status.md`, 3.
  -  the boxless removal gap remains `../adoption.status.md`, 1.
  -  the GitHub status and dispatch gap remains `gates.status.md`, 1.
  -  the native-queue gap remains `../queues.status.md`, 1.
  -  the findings projection gap remains `../issues.status.md` (Not yet, the whole document).
  -  the parked landing hold remains `../README.status.md`, 5.3.
  -  the per-change bounds and gates gap remains `README.status.md`, 1.
  -  the empty issue-title gap remains `../issues.status.md` (Not yet).
  -  the goal-issue link remains `README.status.md`, 2.
  -  the approval, lens, parallel-gate and status gaps remain `gates.status.md`, 1.
  - the whole document: a no-change on a change under another hold stays on the row, but the
    hold that stands does not say its words: `hold_change_decision`
    (`temper-engine-domain/src/route/forge_route.rs`) reads them for
    `Hold::Failed` alone, and they are in no panel. A release reads the result
    as a failed producer's, so its words first reach a person in the `Failed`
    hold that follows, and the change does not end. Seen by reading: the step
    test stops at the row
    (`a_questions_hold_keeps_the_result_until_release_and_says_no_change_words`).
  - the whole document: a result ending a stalled hold has no world: a live delegate does not
    stall, so `temper-engine-domain-forge/src/tests/change.rs` gives the held
    row by `Event::Change`, and only a green head's worlds run the release its
    `EndHold` asks for
    (`temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs`).
  - the whole document: the request `parallel-faults` of 2026-11-06 has its proof in a rerun of
    its parallel replay, and no record shows one, so no real run has shown a
    producer working past the stall or a stall's hold reaching anyone. It also
    asks why the replay's hold came about 57 minutes in, where the stall is 30,
    and that stays unknown: in a world the clock starts at the producer's
    request, after any wait for a dependency, and a hold shows only when a step
    runs, but no deployment's times were read. So is whether that row held with
    its task not, by the path `change_decision` now closes: a world shows that
    drop only by delivering the decision out of step, though the empty inbox,
    and `task show`'s "Waiting for news", which jig answers of a task not held
    (`jig/crates/jig-domain-core/src/runs/waits.rs`), fit it.
  - the whole document: widening exhausted spend with ctl before release: `task amend`'s
    `--budget` is not implemented (`../../../client/ctl/areas.md`). A spend
    hold offers cancellation or asking afresh instead.
  - the whole document: `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`), and the plan
    `first-goal-fixes` edits neither, the ui drafts holding them, so a cancel
    is said by `task show` and not by this line.
  - the whole document: the walk the request `brief-failure` of 2026-11-05 ends with has not
    passed: on a real Forgejo, a change whose CI goes red gets a repair that
    starts, and a repair whose brief can't be read ends held with the reason in
    `temper ctl`. Its QA rung is not drafted yet, so both stand on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`).
  - the whole document: `[the reviews could not be read: REASON]`, a gate's repair standing on
    its pull, is unobserved: it is built (`temper-engine-domain-forge/src/brief.rs`,
    `brief_answer`), but those stories run a CI repair's pair alone and no test
    holds its words.
  - the whole document: a released resolution or merge job that failed holds again, where the
    section has a resolution asked afresh: `released_state`
    (`temper-engine-domain-forge-change/src/domain.rs`) keeps `Resolving {
    requested: true }`, the row keeps the failed status
    (`temper-engine-domain-forge/src/domain.rs`, `change_standing`), and
    `delegated` reads it, so no fresh task is made. A failed producer and a failed
    repair take `requested: false`. This is from reading: no test or story
    releases a failed resolution. The `producer-failure` plan, whose request asks
    after a failure's shape and not its release, pins only its hold
    (`failed_repairs_and_resolutions_do_not_take_a_pushed_head`).
  - the whole document: a green head ending a repair has no story for a repair waiting for an
    answer to its question (none asks from a repair), a repair finding nothing
    to do (`NoChange`, read as failed), or a task closing or held with its
    attempt running, taken as a run in progress
    (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate_running`):
    the policy's tests hold a failed status and `delegate_running` alone
    (`temper-engine-domain-forge-change/src/tests/green.rs`).
  - the whole document: the conflicting goal-landing story remains `../branches.status.md`, 1.
  -  the retarget-on-release gap remains `README.status.md`, 3.
  -  the no-CI adoption choice remains `gates.status.md`, 2.
  -  the panel's wait and charter facts remain `README.status.md`, 5.
  -  the change drain remains `README.status.md`, 6.
  -  the dependency-ancestry proof remains `questions.status.md`, 1.
  -  the message-room relay gap remains `questions.status.md`, 1.
  -  the repair/resolution relay stories remain `questions.status.md`, 1.
  -  the tier-branch CI gap remains `../queues.status.md` (Not yet, the whole document).
  -  the issue milestones remain `../issues.status.md` (Not yet, the whole document).
  -  the forge questions remain `../issues.status.md`, 1.
  -  participating objects and external merges remain `../README.status.md`, 7.
  -  upstream contributions remain `../README.status.md`, 8.
  -  the rule-change gate stories remain `gates.status.md`, 3.
- **Design departs:** `repairs.md`, `questions.md`, 1 and jig's `domain/tasks/lifecycle.md`, 8, from the request
  `producer-failure` of 2026-11-05, each decided by its plan's cut, for the user
  to see:
  - `questions.md`, 1, and jig's `../attention.md`, 3: the finish-parking departure remains `questions.status.md`, 1.
  -  the ended-asker notice departure remains `questions.status.md`, 1.
  - the whole document: the request has its failing test in the forge world. Its stories run in
    the engine's domain world
    (`tests/engine/domain/tests/suite/forge_routes/producer_failure.rs`), where the
    core and the connector meet, with the policy's in
    `temper-engine-domain-forge-change/src/tests.rs`; `tests/engine/forge` tells
    none of it.

- **Design departs:** from `replay-repairs`' request item 2,
  decided by its plan's cut: a review starts with an omitted-page notice
  rather than including the oversized file. A files page already carries
  whole files, and the reference already sizes its answer bound from
  the room, so neither paging between files nor that room admits one
  file beyond the bound. The requested refusal's bound met and measured
  size remain `../README.md`, section 11's open decision: skein supplies only bare
  `TooLarge`, and carrying those facts changes its public interface and
  temper's stored answer. `read_pull_files` therefore still names
  `forge.client.rows or forge.client.answer_bytes` and says the measure
  is unknown; its host answer limit is a separate bound.

- **Design departs:** from the request `repair-green` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - The request's rule 1 ends a held repair when CI has passed at the head. A
    hold at gating, past the bound or short of authority, stays held
    (`temper-engine-domain-forge-change/src/domain.rs`, `held_on_ci`), because
    its state can't say whether it is a gate's or CI's, and the plan kept what a
    hold stores.
  - Rule 1's hold ends on a wake alone, and a poll wakes a change only when what
    it read of the pull request moved (`temper-engine-domain-forge/src/domain.rs`,
    the pull read's `wakes`). Under required contexts a check they don't name can
    keep the pull request's CI as it was, so that hold waits for a hint or a
    person's release, and where nothing delivers hints, for the release. It
    costs time and makes no fresh repair.
  - Rule 1 ends a retrying repair with no agent's work. A run ending into a retry
    wakes no change, and the change reads again within `poll`
    (`temper-engine-domain-forge/src/domain.rs`, `repair_poll`), so a backoff
    shorter than that starts another run, which rule 2 leaves to finish.
  - The request has its stories in the forge world or lower. The held and
    retrying ones run in the engine's domain world, above it
    (`tests/engine/domain/tests/suite/forge_routes/repair_green.rs`), where a
    task's phase, its cancel and the deployment's release meet; the forge world
    (`tests/engine/forge/tests/suite/green.rs`) tells the policy's.

- **Design departs:** from the request `brief-failure` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - The request's true capacity is a place, an account or a brief slot. `repairs.md` also
    pauses, with no bound, for a forge client `Busy` at the brief's deadline, a
    forge not connected and a connector with no room, so a repair waits out a
    disconnected forge unheld, though saying why, rather than making a person
    release each.
  - The request has a CI repair's stories in the forge world. They run in the
    engine's domain world (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`),
    where the core and the connector meet; the forge world
    (`tests/engine/forge/tests/suite/brief.rs`) tells a lone section's failure.
