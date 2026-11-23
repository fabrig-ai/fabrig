# repairs: status

2026-11-19. Where the tree's code lags or departs from [repairs.md](repairs.md).

A stall's gaps are `stalls.status.md`'s.

- **Not yet:** "A repair", a runner's failure: the brief does not say a failure was the runner's, nor how many times its job ran again at the head, and such a failure is repaired, not run again (`temper-engine-domain-forge/src/brief.rs`).

- **Not yet:**
  - the whole document: the request-to-repair fuzzy coverage remains `gates.status.md`, 1.
  -  the real-process update-recovery trial remains `../updates.status.md`, 1, and `../README.status.md`, 10.
  - `README.md`, 2, step 3: the base-first deployment proof remains `README.status.md`, 2, step 3.
  - the whole document: the repair-cause and cleanup deployment proof remains `../branches.status.md` (Not yet, the whole document).
  - `questions.md`, 1, *Which target*: the real-model question-target proof remains `questions.status.md`, 1, *Which target*.
  - `questions.md`, 1, *The next producer merges the landing tip first*: the resumed-producer deployment proof remains `questions.status.md`, 1, *The next producer merges the landing tip first*.
  -  the browser readoption story remains `gates.status.md`, 3.
  -  the boxless removal gap remains `../adoption-connection.status.md`, 1.
  -  the GitHub status and dispatch gap remains `gates.status.md`, 1.
  -  the native-queue gap remains `../queues.status.md`, 1.
  -  the findings projection gap remains `../issues.status.md` (Not yet, the whole document).
  -  the parked landing hold remains `../topics.status.md`, 1.3.
  -  the per-change bounds and gates gap remains `README.status.md`, 1.
  -  the empty issue-title gap remains `../issues.status.md` (Not yet).
  -  the goal-issue link remains `README.status.md`, 2.
  -  the approval, lens, parallel-gate and status gaps remain `gates.status.md`, 1.
  - the whole document: a green head ending a repair has no story for a repair waiting for an
    answer to its question (none asks from a repair), a repair finding nothing
    to do (`NoChange`, read as failed), or a task closing or held with its
    attempt running, taken as a run in progress
    (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate_running`):
    the policy's tests hold a failed status and `delegate_running` alone
    (`temper-engine-domain-forge-change/src/tests/green.rs`).
  - the whole document: the conflicting goal-landing story remains `../branches.status.md`, 1.
  -  the retarget-on-release gap remains `drift.status.md`, 1.
  -  the no-CI adoption choice remains `gates.status.md`, 2.
  -  the panel's wait and charter facts remain `panel.status.md`, 1.
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
- **Design departs:** `repairs.md`, `questions.md`, 1 and jig's `domain/tasks/lifecycle-endings.md`, 1, from the request
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
