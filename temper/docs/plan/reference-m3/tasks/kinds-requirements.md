+++
id = "kinds-requirements"
part = "kinds-forge"
kind = "feature"
needs = ["kinds-held"]
files = ["temper/crates/temper-engine-domain-forge/src/standing.rs", "temper/crates/temper-engine-domain-forge/src/tests/standing.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs"]
effort = "high"
minutes = 100
+++

# kinds-requirements: requirements 5 and 6, and the gates, over the rules the forge holds

## Goal

The forge judges its two standing requirements over the landing rules it holds
(`configuration.md`, 13.3; `forge.md`, 8.7): 5, *the landing rules* (guarded), and 6, *up to
date by the landing rules* (observed). And it reads a landing branch's procedure waits and its
gates from the same held rules, a rule that does not enforce giving advice alone. All new
methods and arms: the root keeps calling today's compiled `requires` and `gate_candidates`
until `kinds-switch` moves it, and no change's behaviour moves while nothing holds rules.

Acceptance:

- A new module `src/standing.rs` (`mod standing;` in `lib.rs`), module doc citing
  `temper/docs/design/protocol/configuration.md`, 13.3 and `domain/forge.md`, 8.7, with an
  `impl Domain` block:
  - `pub fn held_requires(&self, repository: &Repository, branch: &[u8]) -> Requires`: as
    `Domain::requires` but over `landing_rules(repository.project)` (deployment's then the
    project's) through `settings::requires`, with `requires`' own resource name (move its
    inner `branch_resource_name` to a private function both use, unchanged).
  - `pub fn held_gates(&self, limits: &Limits, repository: &Repository, branch: &[u8]) ->
    Option<Box<[change::Gate]>>`: the gates of every held rule covering the branch, the
    deployment's then the project's, each a `GateCandidate { number, parameters: 0, blocking,
    project_scope, freshness }` (`project_scope` true for a project's rule), required where its
    rule enforces, through `gates::selected` as `configured_gates` does: an enforcing rule's
    blocking gate blocks; a gate of a rule that does not enforce is asked for and never holds
    the merge (13.3, **A rule that does not enforce is advice**).
- `Domain::judge_landing` gains requirements 5 and 6, before its criterion lookup (their
  parameters are 0 and name no criterion): over the enforcing held rules covering the change's
  landing branch (a change row's repository and `base`; a landing row's member's, as
  `landing::judge` finds it):
  - 5: each rule's criteria, `Ci` where it asks CI, `Gate { number, freshness }` for each
    blocking gate, `Approval { role, people, freshness }` for each approval, each judged as
    requirements 1, 3 and 4 are today (`judge::judge`, or `landing::judge` for a landing row);
    `Refuse` where one refuses, else `Wait` where one waits, else `Met`; `Met` where no
    enforcing rule covers the branch.
  - 6: `UpToDate` judged once where some such rule asks up to date, `Met` where none does.
- `Domain::guards_landing` is true for requirement 5 and false for 6, whatever the parameters.
- Requirements 1 to 4 are judged as today.
- Tests in `src/tests/standing.rs` pass; the crate's focused tests pass unchanged.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.3 (all four bullets);
  `temper/docs/design/domain/forge.md`, 8.7 (the two tables and the bullets **Enforced, and
  waited for** and **What the procedure asks of the rules**) and 8.3 (gates).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C3.
- `temper/docs/plan/reference-m1/tasks/settings-landing-rules.md` and `change-requires.md`'s
  meaning of `requires`, kept.

## Owns and may change

- Owns `standing.rs` and its tests, new. `domain.rs`: the two arms in `judge_landing` (calling
  into `standing.rs`), the one in `guards_landing`, and the moved resource-name helper, nothing
  else. `lib.rs`, `tests.rs`: one `mod` line each.
- Not this task's: the rule and criterion a change last waited on, kept on the change and
  shown on its panel (`forge.md`, 8.8: its status entry stays); removing the compiled
  `GatePolicy` and `Judges` (`kinds-switch`).
- Options: if `gates::selected` needs a candidate's `parameters` for anything but the core's
  requirement lookup, keep it 0 and say what reads it. If a landing row's members name several
  landing branches, judge each member's as `landing::judge` walks them and fold as above.
- Other plans: reference-m2, -m5, -m6, -m7b change `domain.rs`: keep the hunks to the arms.

## Steps

1. The module, the arms, the helper.
2. Tests in `src/tests/standing.rs`, table-driven where they can be, rules kept through
   `keep_settings` (kinds-held) on the crate's fixtures:
   - `the_landing_rules_meet_wait_and_refuse_as_their_criteria`: one enforcing rule with CI,
     a blocking gate and an approval: all met `Met`; CI pending `Wait`; the gate asking for
     changes `Refuse`; one waiting and another refusing `Refuse`.
   - `rules_that_do_not_enforce_hold_nothing`: a covering rule with `enforce` false and CI
     failed: 5 and 6 `Met`.
   - `a_branch_no_enforcing_rule_covers_meets_both`.
   - `up_to_date_by_the_landing_rules`: a rule asking up to date, the head lacking the base's
     tip `Refuse`, unread `Wait`; no rule asking it `Met`.
   - `project_rules_add_to_the_deployments`: a deployment rule met and the project's failing
     `Refuse`.
   - `held_gates_block_only_under_enforcing_rules`: an enforcing rule's blocking gate is
     blocking; the same gate under a rule that does not enforce is not.
   - `held_requires_reads_the_held_rules`: as M1's `requires` tests, through the domain.
   - `requirement_5_is_guarded_and_6_observed`.
   Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/standing::/)'` (check the count: eight).
3. fmt, old forms, clippy on `temper-engine-domain-forge`; its focused tests; `heavy cargo
   check -p temper-engine-domain --tests`.
4. Commit ("Judge the landing rules' standing requirements over the held rules") and push.

## Report

The implementer's text's report, plus: how a landing row's members name their branches, as
you found them.
