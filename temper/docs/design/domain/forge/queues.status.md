# queues: status

2026-11-11. Where the tree's code lags or departs from [queues.md](queues.md).

- **Not yet:**
  -  the request-to-repair fuzzy coverage remains `change/gates.status.md`, 1.
  -  the real-process update-recovery trial remains `README.status.md`, 6 and 10.
  - `change/README.md`, 2, step 3: the base-first deployment proof remains `change/README.status.md`, 2, step 3.
  -  the repair-cause and cleanup deployment proof remains `branches.status.md` (Not yet, the whole document).
  - `change/questions.md`, 1, *Which target*: the real-model question-target proof remains `change/questions.status.md`, 1, *Which target*.
  - `change/questions.md`, 1, *The next producer merges the landing tip first*: the resumed-producer deployment proof remains `change/questions.status.md`, 1, *The next producer merges the landing tip first*.
  -  the browser readoption story remains `change/gates.status.md`, 3.
  -  the boxless removal gap remains `adoption.status.md`, 1.
  -  the GitHub status and dispatch gap remains `change/gates.status.md`, 1.
  - 1: the forge's own merge queue beyond a queued merge kept
    pending: `Pull.queued` and `dequeued` are not read, a queued change
    keeps temper's turn, an ejection makes no repair, the panel shows no
    place in it, and adoption's words leave out the queue, signatures,
    linear history, code owners and the App's bypass
    (`temper-service/src/family.rs`).
  -  the findings projection gap remains `issues.status.md` (Not yet, the whole document).
  -  the parked landing hold remains `README.status.md`, 5.3.
  -  the per-change bounds and gates gap remains `change/README.status.md`, 1.
  -  the empty issue-title gap remains `issues.status.md` (Not yet).
  -  the goal-issue link remains `change/README.status.md`, 2.
  -  the approval, lens, parallel-gate and status gaps remain `change/gates.status.md`, 1.
  -  the held no-change words remain `change/repairs.status.md` (Not yet, the whole document).
  -  the stalled-hold result story remains `change/repairs.status.md` (Not yet, the whole document).
  -  the parallel replay proof remains `change/repairs.status.md` (Not yet, the whole document).
  -  the spend-widening gap remains `change/repairs.status.md` (Not yet, the whole document).
  -  the cancelled-delegate hold wording remains `change/repairs.status.md` (Not yet, the whole document).
  -  the brief-failure walk remains `change/repairs.status.md` (Not yet, the whole document).
  -  the review-brief refusal proof remains `change/repairs.status.md` (Not yet, the whole document).
  -  the failed-resolution release gap remains `change/repairs.status.md` (Not yet, the whole document).
  -  the remaining green-head repair stories remain `change/repairs.status.md` (Not yet, the whole document).
  -  the conflicting goal-landing story remains `branches.status.md`, 1.
  -  the retarget-on-release gap remains `change/README.status.md`, 3.
  -  the no-CI adoption choice remains `change/gates.status.md`, 2.
  -  the panel's wait and charter facts remain `change/README.status.md`, 5.
  -  the change drain remains `change/README.status.md`, 6.
  -  the dependency-ancestry proof remains `change/questions.status.md`, 1.
  -  the message-room relay gap remains `change/questions.status.md`, 1.
  -  the repair/resolution relay stories remain `change/questions.status.md`, 1.
  - the whole document: a branch a tier follows still pauses its queue on its own CI's
    red and makes a repair, where its tiers judge it instead
    (`temper-engine-domain-forge-change/src/domain.rs`, `base_ci`).
  -  the issue milestones remain `issues.status.md` (Not yet, the whole document).
  -  the forge questions remain `issues.status.md`, 1.
  -  participating objects and external merges remain `README.status.md`, 7.
  -  upstream contributions remain `README.status.md`, 8.
  -  the rule-change gate stories remain `change/gates.status.md`, 3.
- **Not yet:** the whole document, *The first owns the queue*: the first keeps its turn
  only until the next ready change steps: `queue_first`
  (`temper-engine-domain-forge/src/domain.rs`) chooses among `queue_ready`'s
  changes, which leave out one at `Updating`, `Checking`, `Gating` or `Landing`,
  so a queued change stepped while the first is there takes `First` too; several
  ways of leaving the queue keep `owns_turn`; and only a change leaving `Queued`
  or `First` wakes the next, not one giving up its turn elsewhere or landing
  [in flight: change 594].
