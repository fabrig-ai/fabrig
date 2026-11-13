# README: status

2026-11-13. Where the tree's code lags or departs from [README.md](README.md).



- **Not yet:** 1: parameter 18, the source landings a follow change
  follows, its route `up` and no early start [in flight: change 99].

- **Not yet:** 3, *Its own writes are its own* and *Its branch moved*'s words: an update's made head is not recorded as its branch's head, so a goal's landing on its held branch takes it for drift and holds `BranchMoved`; a let-go update is judged against the base's tip read now, not the base it asked for; a close-out's or catch-up's merge into the goal's branch takes no writer slot the landing sees; and a `BranchMoved` hold names neither head (`temper-engine-domain-forge/src/domain.rs`, `change_outcome`, `change_facts`; `temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`).


- **Not yet:** 1, `reruns`, and 2, step 3, *The runner's failures run again*, with step 7's attempt in the landed result: a change's CI failed by the runner is repaired like any other, and no configuration names `forge.change_policy.reruns` (`temper-engine-domain-forge-change/src/domain.rs`, `checking`, `gating`).


- **Not yet:** 2, a change whose pushed head its landing branch already contains: it lives on, and only a contained catch-up ends with no change; nothing reads whether the base's tip contains a change's head.

- **Not yet:** 2, step 6: a resolution and a merge job wait for no producer or repair to end, and are cut at the last head the change knew, not its latest push (`repairs.md`).

- **Not yet:** 1, 3: a goal's runs push to its run
  branch r, and a `"branch": "run"` change lands r into g; g records no
  design writers, so a push to g by the goal's run would hold the goal as
  drift and a landing change on g as "branch moved"; a goal's run prepared
  before g is made and confirmed starts on the base, its run unable to
  deliver (`temper-engine-domain-forge/src/items.rs`,
  `task_workspace_item`).

`repairs.md` has no known implementation gap in a resolution's files:
its pull section carries a file whole, both sides, where it fits the
room the section has left, and names one that doesn't, with its sides'
bytes or `absent before` and `absent after`, for its checkout, the files
after it carried as they fit (`temper-engine-domain-forge/src/brief.rs`,
`name_conflict_if_past_room`; `src/tests/brief.rs`,
`a_resolutions_files_are_carried_whole_or_named`). The charter's fit
that can still cut the section's end is `../../agent/README.status.md`'s `../adoption.md`, 3; no
story takes a named file through its charter's start.

`../adoption.md`, section 5's `DismissMaking` is present: an owner can erase a failed row
and free its place, absence is kept, and running rows or another scope
are refused. The request carries `into`, so its keyed scope survives the
row's erasure. The connector's making tables cover these decisions.

Section 1 has no known implementation gap in bounding a pull request's
body at 65,536 bytes independently of `forge.client.op_bytes`, with a
marked cut and the change's stored words kept whole. The engine's batch
table admits 32 changes and refuses 33 at an authority batch bound of 32.

`../README.md`, section 5.2 has no known gap in a goal's topics from the core: they are
its landing branches' alone, and a goal's CI subscription held at
restart is dropped with its jig row (`temper-engine-domain-forge/src/domain.rs`,
`restored`, by `Request::GoalUntopic`), while a task's own stays.

`repairs.md` has no known gap in a merge job's failures: each is a line on
the worker that met it and on the engine, naming the job, its kind and why,
and the way out of its hold, out of tries, points at the engine's log,
which names each (`../../worker/jobs.status.md`). The composed system world runs
a change's merge job from the engine through its worker to the fake forge
(`../../../testing/README.status.md`, 2.1).

- **Not yet:**
  -  the request-to-repair fuzzy coverage remains `gates.status.md`, 1.
  -  the real-process update-recovery trial remains `../README.status.md`, 6 and 10.
  - 2, step 3: a head already behind taking its base first stands on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/update_first.rs`). The request
    `landing-fixes` of 2026-11-06 has its proof in the next landing plan on a real
    deployment, and no record shows one, so how much of a change's wait it saves,
    and how soon a moving base reaches the bound (`../README.md`, section 11), are unmeasured.
  -  the repair-cause and cleanup deployment proof remains `../branches.status.md` (Not yet, the whole document).
  - `questions.md`, 1, *Which target*: the real-model question-target proof remains `questions.status.md`, 1, *Which target*.
  - `questions.md`, 1, *The next producer merges the landing tip first*: the resumed-producer deployment proof remains `questions.status.md`, 1, *The next producer merges the landing tip first*.
  -  the browser readoption story remains `gates.status.md`, 3.
  -  the boxless removal gap remains `../adoption.status.md`, 1.
  -  the GitHub status and dispatch gap remains `gates.status.md`, 1.
  -  the native-queue gap remains `../queues.status.md`, 1.
  -  the findings projection gap remains `../issues.status.md` (Not yet, the whole document).
  -  the parked landing hold remains `../README.status.md`, 5.3.
  - 1: per-change bounds and gates are not caller overrides: `change` takes parameters 1
    to 6, 9 to 12, 14 and 16, its bounds, the questions it takes among
    them, are the deployment's `change_policy`, and its gates its landing
    branch's rules' (`temper-engine-domain/src/route/forge_route.rs`,
    `start_change`); its page's line that a change takes no words.
  - 1: the empty issue-title gap remains `../issues.status.md` (Not yet).
  - 2: the pull request's body naming its goal's issue.
  -  the approval, lens, parallel-gate and status gaps remain `gates.status.md`, 1.
  -  the held no-change words remain `repairs.status.md` (Not yet, the whole document).
  -  the stalled-hold result story remains `repairs.status.md` (Not yet, the whole document).
  -  the parallel replay proof remains `repairs.status.md` (Not yet, the whole document).
  -  the spend-widening gap remains `repairs.status.md` (Not yet, the whole document).
  -  the cancelled-delegate hold wording remains `repairs.status.md` (Not yet, the whole document).
  -  the brief-failure walk remains `repairs.status.md` (Not yet, the whole document).
  -  the review-brief refusal proof remains `repairs.status.md` (Not yet, the whole document).
  -  the failed-resolution release gap remains `repairs.status.md` (Not yet, the whole document).
  -  the remaining green-head repair stories remain `repairs.status.md` (Not yet, the whole document).
  -  the conflicting goal-landing story remains `../branches.status.md`, 1.
  - 3: a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
  -  the no-CI adoption choice remains `gates.status.md`, 2.
  - 5: the rule and criterion of a change's last wait or refusal, and
    its charter, on its panel.
  - 6: a change drained.
  -  the dependency-ancestry proof remains `questions.status.md`, 1.
  -  the message-room relay gap remains `questions.status.md`, 1.
  -  the repair/resolution relay stories remain `questions.status.md`, 1.
  -  the tier-branch CI gap remains `../queues.status.md` (Not yet, the whole document).
  -  the issue milestones remain `../issues.status.md` (Not yet, the whole document).
  -  the forge questions remain `../issues.status.md`, 1.
  -  participating objects and external merges remain `../README.status.md`, 7.
  -  upstream contributions remain `../README.status.md`, 8.
  -  the rule-change gate stories remain `gates.status.md`, 3.
- **Not yet:** 3: containment is read as a commit list or a
  comparison, not a `Range`: the change step's up-to-date and catch-up
  ranges (`Read::Commits`, `temper-engine-domain-forge/src/domain.rs`,
  `ancestry_status`), a landing member's main (`landing.rs`) and a lost
  run's observed head (`domain.rs`, `lost_read`) (skein's `forge/README.md`,
  3.2).

- **Design departs:** 2, step 3, from the request `landing-fixes` of 2026-11-06,
  item 1, decided by its plan's cut: the update first saves the change's wait on
  CI, where the request counts its 5:26 as CI's, because temper has no effect that
  stops a run the forge has started and a pull request comes before its update. The
  replaced head's run goes by the repository's own CI settings (`../README.md`, section 11).
