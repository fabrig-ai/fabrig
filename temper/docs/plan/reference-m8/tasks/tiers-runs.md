+++
id = "tiers-runs"
part = "tiers-forge"
kind = "feature"
needs = ["tiers-surface", "tiers-policy", "tiers-config", "change-verifying"]
files = ["temper/crates/temper-engine-domain-forge/**", "temper/crates/temper-protocol-store/**", "temper/tests/codecs/**"]
effort = "high"
minutes = 360
+++

# The forge's tiers follow their branch, run whole, and are judged

## Goal

The forge connector runs each configured tier on the tip of the branch it follows: one run per
tier at a time, the next at the newest tip, its range the landings since the tier's last judged
tip; a tip whose key has a receipt judged with no run; each run a `tier` procedure task the root
makes on the forge's request, its branch `temper/tier/NAME.N`, dispatched, followed and
judged by the pure policy; a runner's faults retried up to `tier_retries`, then the run held.
Reading a run's failing tests from its artifact, reruns and probes are `tiers-reports`'s: here a
failed whole run is judged as `failed(None)`, a fault.

Acceptance (`temper/docs/design/domain/after-landing.md`, sections 2, 3, 4's fault, 10):

- **Following:** each configured tier's branch (its `branch`, else the repository's landing
  branch as adoption names it) joins the client's watch set wherever it is computed (the
  forge's `domain.rs`: `names`, `unnamed`, and the restore and settle rebuild, by a survey about
  lines 1749, 1822 and 5200), under an owner of the forge's own, as long as a tier names it. The
  first tip seen of a tier's branch with no judged run starts the tier's baseline (section 2: its
  failures blamed on no landing); a tip move with no run in flight starts the next run; one in
  flight waits, and the run after it takes the newest tip. A move of a branch no tier follows
  starts nothing.
- **Range:** the landings in a run are the observed tip moves since the last judged tip, each
  with its commit and the tier's key at it (the gates' key over the tier's definition: name,
  workflow, ignored, artifact, format and `branch`; `temper-engine-domain-forge-landing`'s
  `key.rs`), temper's own merges (`Stored::Landed`) and others' pushes alike; at most `2 *
  changes + 1` landings, the oldest two adjacent outside landings merging past it.
- **Receipts:** a tip whose key has a tier receipt is judged by it (`start`'s `receipt`): still a
  tier task (every effect needs a task), but no branch and no dispatch; whole runs that end make
  receipts with their failures (`Check::Tier`); reruns and probes never do.
- **A run:** `Request::TierRun` once per run; on `Event::TierTask` the task owns the run; its
  writes, each with `EffectPurpose::Tier` (tag 8): the branch at the tip (kind 9, `Write::CreateBranch`),
  the dispatch (kind 16, inputs `temper_commit` and `temper_key`), a fault's rerun or new dispatch
  (17 or 16), followed through `runs.rs`'s `Follow` as the gates' are; `describe_tier_effect`
  real; `Event::VetoTier` and `StepTier` handled as the landing's veto and step are. The run's
  branch is deleted when the run ends, by the release a change's branch takes at its close
  (`temper/docs/design/domain/forge.md`, section 11), never by an effect of kind 15.
- **Judged:** `step`'s `Next::Judged` saves the tier's record (`judged`) and ends the task,
  `TierDecision::Done` with contracts.md, 4's words; then calls
  `crate::verification::judged(d, repository, tier, &verdict)`, a function `tiers-verification`
  fills (here it does nothing). `Next::Held`: `TierDecision::Held` with contracts.md, 4's words;
  on `StepTier` with `released`, the run is dispatched again with its faults counted from 0, its
  landings still verifying. `Next::Dispatch` of a `Rerun` or `Probe` part cannot arise until
  `tiers-reports` (a test pins that a failed run with no report is a fault).
- **Settings:** retries from `Domain::after_landing(project)`; the constant `change-verifying`
  put in the facts stays for `tiers-verification`.
- **Restart:** the tier's record and each operation's state are kept on `TierRow` (add the
  operational fields it needs: pending tips, the baseline, the branch's state, the flight, the
  held fault; in the store's record too), so a cold restart settles what was in flight; a story
  writes a tier mid-run, restarts cold and goes on.
- Unit stories in the forge crate (`src/tests/tiers.rs`, on the crate's fixtures): a green run
  clears its range and makes a receipt; two moves during a run give one next run at the newest
  tip; the baseline; a documentation landing judged by a receipt with no dispatch; an outside
  push in a range; a fault retried, then green; faults past `tier_retries` held, then released
  and green; a branch no tier follows starts no run; the cold restart.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
  `-p temper-protocol-store --lib`; `-p temper-codecs-world --test suite`;
  `-p temper-engine-forge-world --test suite`; `heavy cargo check -p temper-engine-domain
  -p temper-engine-domain-world --tests`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 1, 3 and 4.
- `temper/docs/design/domain/after-landing.md`, sections 2, 3, 4, 10;
  `temper/docs/design/domain/landing.md`, 6.1, 6.2, 7.1 and 7.4 (a gate's key, receipts,
  dispatch, faults); `temper/docs/design/domain/forge.md`, 7.2 and section 11.
- Today's code, by a survey: gate runs in `src/landing.rs` (key reads about 1227, dispatch about
  1234 to 1291 with inputs about 1384, receipts `put_receipt`/`finish_flight` about 1491 to
  1572 with `Check::Tier` refused there today, effect kinds about 2082 to 2133), `src/runs.rs`
  (`Follow`, `follow`, `retry`, `write`, `read`, `answered`, `verdict`), a branch's tip in
  `domain.rs` (`branch_changed` about 4556 publishes nothing for a first sighting: a tier's
  baseline needs it), merges kept as `Stored::Landed` (`domain.rs` about 1653).
- The old code, to adapt: `temper/docs/plan/reference-m8/notes/old/temper-2a-forge.diff`
  (`tiers.rs`: watch, following and range, tree walk, the run's task, dispatch, receipts,
  restart; `purpose.rs`), `temper-t9-1f11a39df.diff` (tier branches released through the
  forge client's hand-off) and `temper-t9-8140d6c33.diff` (held runner faults released by a
  person, faults from 0). The old code hard-wires the landing branch: N5's `branch` replaces it.
  The old mission's decisions: probes oldest first; a receipt-judged verdict still has a tier
  task; branches released by release writes with no kind 15.

## Owns and may change

- Owns the forge crate and the store for this wave (`tiers-verification` and `tiers-reports`
  come after), and the codecs world's store tests.
- Readers: `temper/tests/engine/forge/src/*.rs` (the world's limits and workflows: a second
  workflow for a tier, more `workflow_runs`), only as the stories need; `temper-fake/src/check.rs`
  arms.
- Not this task's: the root (`temper-engine-domain`, `tiers-root`), the change's verification
  facts (`tiers-verification`), artifact reports, reruns and probes (`tiers-reports`), the panel
  (`panel-steps`).

Pre-approved options:

- If deleting the run's branch by the release a change's branch takes needs a hold the tier
  task cannot take, delete it by the forge client's release write as the old t9 commit does,
  name the departure from after-landing.md 10 in the report, and go on.
- If the old range bound (`2 * changes + 1`) does not fit the store's row bound, use the bound
  that fits and give both.
- If a figure under `temper/crates/temper/tests/figures/` or a store size pin moves, update it as
  its test says, and list it.

## Steps

1. Read the old `tiers.rs` whole, then today's `landing.rs` gate path and `runs.rs`; write
   `tiers.rs` against today's shapes and the pure crate.
2. The watch set, the first sighting, the range; then runs, receipts, faults and the hold; then
   restart.
3. The unit stories, then the forge world's suite.
4. `cargo fmt`; clippy over every package you changed; the tests in Goal, each count reported.
5. Commit in steps that say why; push your branch.

## Report

As the implementer's text says, and also: the fields added to `TierRow`; how the branch is
deleted; what of the old code you took; anything `tiers-verification` or `tiers-reports` needs
that you left as a hook.

## Answers

### 2026-11-02: Should this run resume after tiers-surface pushes its required interface (recommended), or should tiers-runs take over the missing surface work? (reference-m8/tiers-runs-1, foreman reference-m8/tiers-runs/foreman-1)

Resume tiers-runs after tiers-surface pushes its required interface; the missing surface work remains tiers-surface’s.
`tiers-runs.md` declares: `needs = ["tiers-surface", "tiers-policy", "tiers-config", "change-verifying"]`.
`tiers-surface.md`, Goal: “After it, `tiers-runs` (the forge's bodies) and `tiers-root` (the root's joins) run side by side.”
Its acceptance explicitly owns the missing tier types, requests, events, purpose tag 8, store families, receipt failures and read-method stubs.
The next run consumes that prerequisite, implements tiers-runs and completes its acceptance checks.

### 2026-11-02: the tier task's deadline (asked by reference-m8/tiers-root-2)

Widened: `notes/contracts.md`, 6 adds `Domain::wait_tier(env, task, until)` and
`Domain::offered_tier(task)` to `tiers.rs`, as no-op stubs on the trunk; fill them here, as
`landing.rs`'s `wait_landing` and `offered_landing` are: `until`, else `env.wall` plus
`limits.client.poll`; a deadline map of the tiers' own, sized to the tier tasks live at once
(one per configured tier, so the repositories' bound times `limits.tiers.tiers`), never
`change_due` or the landing's `due`; counted in `next_wall_deadline` and fired in `fire` as
`Request::WakeProcedure { task }`; dropped when the task ends and rebuilt or cleared on restore
as the landing's are. Add one unit story: a tier task waited with a deadline is woken at it.

## Rebase (2026-11-02, step merge, foreman reference-m8/tiers-runs/foreman-2)

Resolve tiers.rs conflict while preserving tier runs and implementing the required tier deadline hooks.
The merge failed in `temper/crates/temper-engine-domain-forge/src/tiers.rs`.
Trunk commit `567a90d4ba5a92373db4ce832f5e3a180c07c5c4` adds `wait_tier` and `offered_tier` stubs; branch commit `ac01e4524c` replaces that file with the run implementation.
Preserve the branch’s run implementation and retain both public methods, implementing them per `notes/contracts.md`, section 6 and the task’s deadline answer.
Trunk’s tier route, added by `32c1362409`, calls both methods, so choosing the branch file wholesale would leave missing methods.
Add a separate tier wall-deadline map, preferably in `tiers::State`, bounded by `limits.repositories * limits.tiers.tiers`.
Implement `wait_tier` like `landing.rs::wait_landing`: use `until`, otherwise `env.wall` plus `limits.client.poll`; `offered_tier` removes the task’s deadline.
Include the map in `domain.rs::next_wall_deadline`; expire deadlines in `fire`, removing them and emitting `Request::WakeProcedure { task }`.
Remove deadlines when tier tasks end and clear or rebuild them on restore, preserving `tiers::restart`’s existing task wakes.
Keep wall deadlines separate from the existing monotonic walk and flight timers and from change and landing deadline maps.
Add the required story in `src/tests/tiers.rs`: a waiting tier task wakes at its deadline.
Validate the focused story and the spec’s forge library and domain integration checks through `heavy`; keep changes within the forge crate.
