+++
id = "kinds-switch-rules"
part = "kinds-root"
kind = "feature"
needs = ["kinds-start", "kinds-requirements", "change-root", "change-story"]
files = ["temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/route/policy_translate.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/crates/temper-engine-domain/src/route/landing.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/tests/**", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/gates.rs", "temper/crates/temper-engine-domain-forge/src/judge.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests/requires.rs", "temper/crates/temper-engine-domain-forge/src/tests/**", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/startup/application/landing.rs", "temper/crates/temper-service/src/tests/root_checks.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-service/src/tests/system.rs", "temper/crates/temper-service/src/tests/fixtures/**", "temper/tests/engine/domain/src/walking.rs", "temper/tests/engine/domain/src/oracle/config.rs", "temper/tests/engine/domain/tests/suite/settings/**", "temper/docs/design/domain/root.status.md", "temper/docs/design/domain/forge.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 150
+++

# kinds-switch-rules: landing rules enforced by requirements 5 and 6, over the held rules

## Goal

The flip of 13.6's second step for the landing rules (`configuration.md`, 13.3): the rules'
default carries the forge's two standing requirements in place of the requirements 1 to 4 each
rule's criteria became; nothing compiles a landing rule any more (`root.md`, 6); a change's
gates and its procedure's waits come from the rules the forge holds (`kinds-held`,
`kinds-requirements`). The landing rules still come from the document, as the forge's
defaults (`kinds-start`), until `kinds-document`; so every world keeps its rules, now judged
by the forge.

Acceptance:

- The root's start (`domain.rs`, at the `build_landing` and `gate_policy` calls, and their
  twins in the configuration's validity check and at a project's policy change) no longer
  compiles `Config.landing`: no requirement 1 to 4 is added to the rules' default or to a
  seeded project's policy; no `deployment_judges`, `project_judges` or `gate_policy` is given
  to the forge. In their place, the rules' default (`config.core.defaults.rules`) gains, once,
  the two standing requirements on the forge's merge (connector the forge's, kind 4) over
  `{"segments": ["forge"], "last": "open", "value": ""}`: requirement 5, guarded and marked as
  one that must be guarded, parameters 0; requirement 6, observed with a freshness of 0,
  parameters 0 (13.3; the encoding jig's rules use for a connector's requirement today).
- The forge's holding: the root gives the forge `kinds_stock` at each start whether or not
  kinds are declared (only the event to the core stays conditional, `kinds-start`); where
  `kinds_stock` is empty, it gives the stock made from `Config.landing`'s rules and
  `Config.forge_charter` (and review charter 4) by `policy_translate::forge_stock`, each value
  sized 0 (a bridge for worlds that build `Config` themselves; it goes with `Config.landing`'s
  rules in `kinds-document`).
- `Config.landing: LandingPolicy` becomes `Config.landings: Map<u32, ProjectLanding>` and
  `Config.landing_rules: LandingRules { deployment, projects }` (the document's rules, read
  only by the bridge), with every literal and reader moved; `RootConfig.landings` as today.
  The domain world's builders that set `Config.landing` (`temper/tests/engine/domain/src/**`,
  `tests/suite/**`, found by the compiler) take the field's rename alone; `change-story`, which
  owns two of them, has landed before you start.
- The forge: `Domain::requires` reads the held rules (`held_requires`'s body; `held_requires`
  goes or stays as its alias); the root's `configured_gates` (`forge_route.rs`) takes
  `domain.forge.held_gates(..)` and no longer asks the core which gate a rule requires
  (`gate_required` goes); `GatePolicy`, `GateTemplate`, `GateCandidate`'s use by the root,
  `Judges`, `deployment_judges`, `project_judges`, `gate_policy` and the requirement 1 to 4
  criteria lookup go where nothing else reads them (requirements 1 to 4 judge `Wait`, never
  `Met`, for a rule a deployment's owners attach by hand: their parameters no longer index a
  table; say so in a comment citing `forge.md`, 8.7). The tests of the removed code go with it
  (`src/tests/requires.rs` reads the held rules instead).
- Status: `root.status.md` loses, from its first **Not yet** entry, "the two standing
  requirements" and the sentences from "and the landing rules are compiled as" to
  "(`temper-engine-domain/src/translate.rs`, `domain.rs`)"; `forge.status.md` 8.7 loses
  "requirements 5 and 6"; `configuration.status.md` 13.2, 13.3 loses "the rules carry the
  criteria's requirements 1 to 4 rather than 13.3's two standing requirements". Nothing else.
- The worlds pass: a change of meaning (how a merge is checked): `heavy cargo nextest run -p
  temper-engine-domain --lib`; `-p temper-engine-domain-forge --lib`; `-p temper-service
  --lib`; `-p temper-engine-domain-world --test suite`; `heavy cargo check --workspace
  --tests`.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.3, 13.4 (the rules' row; policy 1's
  "with no requirements"), 13.6 step 2 (as this cut wrote it); `temper/docs/design/domain/
  root.md`, 5.5 and 6 (**The landing policy**); `temper/docs/design/domain/forge.md`, 8.7.
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C3.

## Owns and may change

- Owns the files in the header for the edits above; in `forge_route.rs`, `configured_gates`
  and `gate_required` alone (`kinds-switch-charters` changes `start_change` after you). A
  temper-service test file the header does not list that the compiler names: the field's
  literal alone, and say so in the report (`kinds-ctl-form` may run beside you there).
- Options: if a world story asserts requirements 1 to 4 on a rule (the names `Ci`,
  `UpToDate`, a requirement number in an expectation), move it to 5 and 6 with the same
  expected outcome and list each in the report; a story whose outcome changes is a question.
  If jig's rules cannot carry a requirement with parameters 0 and "must be guarded", ask.
- Not this task's: the charters' read (`kinds-switch-charters`); the document and its writers
  (`kinds-document`).

## Steps

1. The root's start: the compile out, the two requirements in, the bridge.
2. The forge: requires and gates from the held rules; the removals.
3. `Config`'s split and its literals; the status lines.
4. Tests: `src/tests/kinds.rs` (root) gains `the_rules_default_carries_the_standing_requirements`
   and `a_document_rule_reaches_the_forge_as_a_default`; the world's light-rules and landing
   stories pass unchanged. Narrow first: `heavy cargo nextest run -p temper-engine-domain --lib
   -E 'test(/kinds::/)'`.
5. fmt, old forms, clippy over every package changed; the packages in Goal.
6. Commit ("Enforce landing rules by the forge's standing requirements") and push.

## Report

The implementer's text's report, plus: every story moved from requirements 1 to 4; what went
of `policy_translate.rs`.

## Answers

2026-11-03. **May the owner-policy story (`an_owner_adds_a_branch_requirement_and_later_changes_wait_for_it`) and its `owner_gate` builder move to the held landing rules, or should the story go?** Move it, keeping its outcome: the owner's addition becomes the project's own landing rule, enforcing (`forge.md`, 8.7: landing rules are settings, and a project adds a review to its rule; `configuration.md`, 13.3, kind 2), so requirements 5 and 6 hold the change until the review approves, then it lands. The world cannot yet change a setting at run time (that helper is `kinds-story`'s, after `kinds-document`, and its `a_landing_rule_changed_reaches_the_next_merge` tells the owner's change at run time), so the rule is the project's from the start, through the document as the forge's default (`kinds-start`'s bridge). You may change, beyond the header and the field's rename:

- `temper/tests/engine/domain/src/forge_routes.rs`: for `owner_gate`, the project's rule (`landing_rules.projects`, project 1) is enforcing (`enforce: true`, no more `!owner_gate`), still with gate 1 and no deployment rule; `change_world_funded`'s `people::Ask::ChangePolicy { change: PolicyChange::Requirements { .. } }` for `owner_gate` goes, and so does the builder's `limits.people.amendment_bytes = 2048` where only that ask needed it. The `paths::Change`/`paths::Gate` fields keep their names.
- `temper/tests/engine/domain/tests/suite/forge_routes.rs`: the story renamed `a_projects_own_enforced_rule_holds_landing_for_its_gate`, its assertions unchanged ("the owner-added review holds landing until its verdict" may say "the project's rule"). Nothing else in the file.

List both under the report's moved stories. The inherited `Refusal::Judged` match in `temper-protocol-store` stays its owner's (as `change-story`'s answer says): what it keeps from compiling is reported unobserved under `## Left`, and does not hold this task.
