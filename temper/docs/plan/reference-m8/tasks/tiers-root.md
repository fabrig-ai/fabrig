+++
id = "tiers-root"
part = "tiers-root"
kind = "feature"
needs = ["tiers-surface", "jig-procedure-seed", "change-verifying"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route/tier_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/landing_route.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/crates/temper-engine-domain/src/tests/**"]
effort = "high"
minutes = 300
+++

# The root makes and steps the tier task, and words a change cleared

## Goal

The root answers the forge's `Request::TierRun` by making the run's `tier` procedure task
(code 4) with jig's zero-budget deployment seed, steps it, carries its effects through
admission, holds it with its words, ends it with its verdict's words, and gives a change's
result its verification line. The landing task by turn moves to the same seed.

Acceptance (`temper/docs/design/domain/after-landing.md`, sections 3, 6 and 10;
`notes/contracts.md`, 4):

- A new `route/forge_route/tier_route.rs`, as `landing_route.rs` is for the landing:
  `Request::TierRun { repository, tier, number, tip }` → `domain.core.procedure_seed(project)`
  (the repository's project) → `jig_domain_core::Event::ConnectorRepair` with
  `Executor::Procedure { connector: forge, code: AFTER_LANDING_CODE }`, a spec naming the
  repository, the tier and the run, a report contract, no holdings, the task's words `Tier NAME
  run N`; then `forge::Event::TierTask`; no seed (no policy, no number left) →
  `forge::Event::TierRefused`. `AFTER_LANDING_CODE` loses its `#[expect(dead_code)]`.
- Starting and stepping: `route/asks.rs`'s `StartProcedure` branch for code 4 → a step
  (`forge::Event::StepTier` with `TierHeard`), as the landing's `start_landing_step`.
  `Request::TierDecision`: `Wait` waits (with its deadline); `Held { words }` holds the task as
  the landing's fault hold does (`tasks::Hold::Procedure`, its words); `Done { words }` ends it
  done with the words as its report.
- Effects: `describe_tier_effect` → an effect flight of the tier's own (`EffectFlight::Tier`,
  beside `EffectFlight::Landing`), admitted, judged and vetoed as the landing's are
  (`forge::Event::Committed`, `forge::Event::VetoTier`), the procedure's effects unpriced.
  `admission.rs` classifies the tier events as the landing's.
- The landing task by turn (`landing_route.rs`, `start_landing`) makes its task with
  `procedure_seed` instead of `repair_seed`; the queue's repair (`forge_route.rs`,
  `start_queue_repair`) keeps `repair_seed` (its producer is an agent: jig's
  `jig/docs/design/domain/authority.md`, 6).
- A change's result (`change_landing_words`, `landed_change`, `finish_change_step`): a change
  that finishes `Cleared` or `Uncleared` gets contracts.md, 4's line after its `Landed:` line,
  naming the tiers' runs (from `Domain::tier_task`/the verification the forge gives; if the
  forge gives no run number, the line names the tiers alone); `Landed` as today.
- Unit stories in the crate (`route/forge_route/tests.rs` or `src/tests/`): a tier run's task is
  made while the period is reserved whole (the seed's point); a refused seed answers
  `TierRefused`; a held decision holds with its words and a release steps it again; a done
  decision ends it with its words; a landing task by turn starts under a reserved period; the
  result words of `Cleared` and `Uncleared`. `temper-engine-domain/src/tests/setup.rs`'s
  `repair_seed` assertion stays true.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain --lib`;
  `-p temper-engine-domain-world --test suite -E 'test(/forge_routes/)'` (landing by turn and its
  results must not change; `a_chats_change_lands_by_turn`,
  `a_ready_change_waits_when_landing_room_is_full` among them).

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 2 and 4.
- `temper/docs/design/domain/after-landing.md`, sections 3 (the held run's words), 6 (the
  result at cleared) and 10 (a run is a task, its authority, its funding);
  `jig/docs/design/domain/authority.md`, 6 (`procedure_seed`).
- Today's code, by a survey: `landing_route.rs` (`start_landing` about 155 to 219,
  `start_landing_step` about 255, `describe_landing_effect`'s use about 269 to 307,
  `landing_decision` about 309 to 398 with its fault hold about 332 to 373); `forge_route.rs`
  (codes about 13 to 16; `task_holdings` about 711 to 760; the effect flight about 3399 to 3538;
  admit, drop and judge about 3788 to 3968; `Request::Outcome` about 3242; `WakeProcedure`
  about 3045; result words about 2623 to 2881); `asks.rs` about 348 to 366; the stock already
  grants kinds 9, 16, 17 and delegates `Procedure(4)` (`temper-service/src/stock.rs`).
- The old code: `temper/docs/plan/reference-m8/notes/old/temper-2a-routes.diff`
  (`start_tier_run`, `start_tier`, `tier_decision`, `describe_tier_effect`'s flight; it held by
  `Hold::Effects`: follow today's `Hold::Procedure` with words) and
  `temper-t9-8140d6c33.diff` for the held run's release.

## Owns and may change

- Owns the files in the header for this wave.
- `temper/tests/engine/domain/src/forge_routes.rs` and its `tests/suite/forge_routes/landing.rs`:
  only where a landing task's start now needs no unreserved spend (a fixture that funded the
  period for it may stay).
- Not this task's: the forge crate (`tiers-runs`, `tiers-verification`, `tiers-reports` own it),
  the panel, help.

Pre-approved options:

- If jig's `procedure_seed` refuses for a reason the old code did not meet, fall back to
  `repair_seed` for the tier task alone, retried on the tier's next move, and say so in the
  report; do not change jig.
- If a domain-world story pins a landing task's budget or reservation that the new seed
  changes, update the pinned value to the seed's (budget 0) and list it.

## Steps

1. `tier_route.rs` and its joins; the landing's seed; the result words.
2. Unit stories; then the domain world's forge routes.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain --all-targets -- -D warnings`; the
   tests in Goal, counts reported.
4. Commit (why: tier runs are deployment procedures the root makes and steps); push.

## Report

As the implementer's text says, and also: the task's spec parameters you chose; the words as
printed; any landing story whose numbers moved.

## Answers

2026-11-02. The forge surface (`notes/contracts.md`, 4) is absent from `tiers-surface`'s branch:
wait for it, or supply it here? Wait. `tiers-surface` owns that surface and its second run is
under way; this task stays out of the forge crate. Start once `tiers-surface` has pushed it: merge
`origin/reference-m8/tiers-surface` (or the trunk, once it has landed there) and check that
`Request::{TierRun, TierDecision}`, `TierDecision`, `Event::{TierTask, TierRefused, StepTier,
VetoTier}`, `TierHeard`, `TierRow` and the stubs `Domain::{tier_task, describe_tier_effect,
change_verification}` are there. Code against the stubs as contracts.md, 4 gives them: a story
that needs a filled `tier_task` or a non-`Unfollowed` verification builds it through the forge's
public events where it can, else is left to `tiers-stories` and listed under Left. If the surface
is still absent, stop again with the names missing; never add them in the forge crate.

2026-11-02. `TierDecision::Wait` carries a deadline, but the forge has no tier wait and the
change's and landing's deadline maps are sized to their own tasks: supply it here, or have the
forge's owner supply it? Neither crosses: the answer added `Domain::wait_tier(env, task, until)`
and `Domain::offered_tier(task)` to the forge's `tiers.rs` as no-op stubs (`notes/contracts.md`,
6); `tiers-runs` fills them. Call them as `landing_route.rs` calls `wait_landing` and
`offered_landing`: `offered_tier` where the tier task is started or stepped, `wait_tier` on
`Wait { until }`; reuse neither `wait_change` nor `wait_landing`. The forge crate stays out of
this task. A story that needs the deadline to fire (a wake at `until`) waits on the stubs being
filled: leave it to `tiers-stories` and list it under Left; the unit stories in Goal need none.
