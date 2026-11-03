+++
id = "change-tasks-limit"
part = "change-tasks"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs"]
effort = "high"
minutes = 75
+++

# A change's tasks bounded by a limit, `forge.change_policy.tasks`

## Goal

The tasks a change's delegate may make beside its foremen come from a limit of the forge's
change policy, `forge.change_policy.tasks`, 6 in `small`, in place of the constant 6, so a
deployment sized for a goal of a hundred changes gives each change about twelve
(`temper/docs/design/domain/agent.md`, 4.3.1). The stock's behaviour does not change.

Acceptance:

- `temper_engine_domain_forge_change::Limits` gains `pub tasks: u32`, documented as the tasks
  a change's delegate may make beside its foremen (`temper/docs/design/domain/agent.md`,
  4.3.1), placed after `questions`.
- `temper-service`'s `LIMITS` (`src/limits.rs`) sets it to 6, commented as `small`'s: a
  producer, a review, and two repairs each followed by its review. The override name
  `forge.change_policy.tasks` is accepted beside `forge.change_policy.questions`
  (`src/limits/names.rs`), a value outside `u32` refused as that one's is.
- The start refuses a configuration whose `forge.change_policy.tasks` is 0, naming the limit,
  as the start's other limit checks name theirs (find where `change_policy.questions` is
  checked against `QUESTIONS_OPEN`, `temper-engine-domain-forge/src/limits.rs`, and check
  beside it).
- `temper-engine-domain/src/route/forge_route.rs`, where the change's delegate is built
  (`chat_change`'s authority, today `6_u32` and `6`): the tasks are
  `env.limits.forge.change_policy.tasks`, plus `questions` with a foreman, then bounded as
  today by one fewer than the run has left.
- Tests:
  - an override test beside the existing `forge.change_policy.questions` one (find it with
    `rg 'change_policy.questions' temper/crates/temper-service/src/tests`): the name sets the
    field, a value past `u32` is refused;
  - a route test in `temper-engine-domain` (beside the existing test of a change's
    delegation, found with `rg 'delegation.tasks' temper/crates/temper-engine-domain/src/tests`)
    that a change made under `tasks` 8 and questions `foreman` gets 12 tasks, and under the
    stock 6 and no foreman gets 6.
- The focused tests of `temper-engine-domain-forge-change`, `temper-engine-domain-forge`,
  `temper-engine-domain` and `temper-service` pass.

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 4.3.1 (a change's authority: at most
  `forge.change_policy.tasks` tasks, and one more per question of its bound with a foreman).
- `temper/docs/design/protocol/configuration.md`, section 7 (overrides by dotted name; the
  limits module's constants each documented with what it bounds).

## Owns and may change

- Owns the files of this spec's header for what acceptance names.
- Every other `temper_engine_domain_forge_change::Limits { … }` literal in the tree (tests and
  fixtures in `temper-engine-domain-forge`, `temper-engine-domain`, `temper-fake`,
  `temper-protocol-store`, found with `rg 'forge_change::Limits \{'`): the new field only, 6
  where the literal copies the stock's values, compiler-forced.
- The tests named above, in their crates' test modules.
- If the store keeps the change policy's limits (a schema field), stop and ask: this task adds
  no store field.

## Steps

1. The field, the stock value and the override; the start's check.
2. The route; the tests; run each narrowly (`heavy cargo nextest run -p PKG --lib -E
   'test(=…)'`).
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge-change -p
   temper-engine-domain-forge -p temper-engine-domain -p temper-service --all-targets -- -D
   warnings`; the four packages' focused tests; `heavy cargo check -p temper-fake -p
   temper-protocol-store --tests`; commit and push.

## Report

As the implementer's text asks.
