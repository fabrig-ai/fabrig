+++
id = "goal-forge-symbol"
part = "goal-forge"
kind = "feature"
needs = ["goal-jig-surface", "goal-forge-branch"]
files = ["temper/docs/design/domain/forge.status.md", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs"]
effort = "high"
minutes = 75
+++

# goal-forge-symbol: *its own goal branch* resolved for a tracked task; a change from outside refused

## Goal

The root's forge holdings (`task_holdings`, which answers jig's holdings asks) resolve the
forge's symbol 1, *its own goal branch* (contract C2's line; temper's `forge.md`, section
2), for a tracked task into a write hold on `<prefix><root>/g<task>` in the project's home,
and refuse it, naming the symbol, for any other; and refuse a change whose landing branch is
held by a task that is neither the change's requester nor an ancestor of it, naming the
holder (contract C3).

Acceptance:

- `forge_route.rs`: `pub(crate) const GOAL_BRANCH_SYMBOL: u16 = 1;` and `task_holdings`
  gains `tracked: bool, symbols: &[u16]` and returns `Result<Box<[tasks::Holding]>,
  Option<jig_domain_core::HoldingsRefusal>>` (`Err(None)` where it returns `None` today).
  - Each symbol: `GOAL_BRANCH_SYMBOL` on a tracked task adds a write holding on
    `tree_branch(home, root, b'g', number, None, ..)` in the project's home (reported with
    `report_resource`, as the run branch is), its kind the one a refusing hold needs (below);
    on an untracked task, a task of a project with no home, or any other symbol:
    `Err(Some(HoldingsRefusal::Symbol { symbol }))`.
  - A change procedure (the `code: 1 | CHANGE_CODE` arm) whose base, parameter 2, names a
    branch of its repository that `domain.core.tasks.holder` reports held by task `h`: when
    the change has a requester task (`parent`) and `h` is that task or an ancestor of it
    (walk the requester chain as `in_subtree` does), the batch goes on as today; otherwise
    `Err(Some(HoldingsRefusal::Held { holder: h }))`. A base nobody holds is as today.
- `asks.rs`: the three holdings arms pass `tracked` and `symbols` from the ask (or each
  `HoldingsNeed`), and answer `refused` from the error (`DelegateHoldings` and
  `ProcedureHoldings`: the first member's refusal, with `holdings: None`).
- **Refusing kind:** if the tasks hub admits a goal branch's hold as refusing from the
  resource's report alone (`GOAL_BRANCH_HOLD`, `goal-forge-branch`), the holding keeps kind
  1; if it decides by the holding's kind, register a second kind in `domain.rs` (near line
  409), `tasks::Kind { connector, kind: 2, hold: tasks::HoldKind::Exclusive { taken:
  tasks::Taken::Refuses } }`, and give a goal branch's holding kind 2. Say which in the
  report.
- Tests in `forge_route/tests.rs`, in the style of
  `delegations_with_one_repository_id_take_their_own_forges_holding`:
  `a_tracked_task_naming_its_own_goal_branch_holds_g_in_the_home`,
  `an_untracked_task_naming_its_own_goal_branch_is_refused_naming_the_symbol`,
  `an_unknown_forge_symbol_is_refused_naming_it`,
  `a_change_into_a_goal_branch_from_outside_its_goal_is_refused_naming_the_goal`,
  `a_change_from_inside_the_goals_subtree_into_its_branch_is_admitted`.
- In `temper/docs/design/domain/forge.status.md`, the **Not yet** sub-bullet "2, 11.1: the
  symbol *its own goal branch*: ..." goes.

## Inputs

- `temper/docs/design/domain/forge.md`, section 2 ("Holds", "Symbols"), section 11 (the `g`
  branch) and 11.1 (the bullets "Named by the goal's template" and "Written by change
  alone").
- jig's `jig/docs/design/domain/tasks.md`, 6.1 ("A held resource a queue orders") and 6.2
  ("What a task holds is its connector's to name"); `jig/docs/design/domain/connectors.md`,
  3.1 ("Symbols").
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C1, C2 (and its
  line under "Names the families write here") and C3.

## Owns and may change

- Owns the files in the header (in `forge.status.md`, the one sub-bullet above).
  `forge_route.rs`, `asks.rs` and `forge_route/tests.rs` are changed on plan reference-m8's
  trunk: keep each hunk to holdings.
- **Readers:** every caller of `task_holdings` (`asks.rs`; `forge_route.rs` near line 2605,
  a repair's holdings, which passes `tracked: false, symbols: &[]` and maps `Err(_)` as
  `None` was mapped).
- **Options:** if a change's requester is not at hand in `task_holdings` for the party path
  (`parent: None`), a person's change into a held branch is refused as from outside. If the
  change family's `landing: "goal"` (their task) resolves a base the root names differently
  from parameter 2, read it from where that task puts it once merged; until then parameter 2
  is the landing branch.
- **Not yours:** the forge crate (`goal-forge-branch`), `change`'s parameters (the change
  family), the jig core.

## Steps

1. Read `task_holdings`, `tree_branch` and `in_subtree` in `forge_route.rs`, and the
   holdings arms in `asks.rs`.
2. Symbols, then the refusal from outside, each with its tests:
   `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/goal_branch|forge_symbol|from_outside_its_goal|inside_the_goals_subtree/)'`
   (check five selected).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain`; `heavy cargo nextest run -p temper-engine-domain-world`.
4. Commit ("forge route: its own goal branch for tracked tasks; changes from outside a goal
   refused"), push.

## Report

The resolution, the refusals and their mapping, which hold kind the goal branch takes, the
tests and counts.
