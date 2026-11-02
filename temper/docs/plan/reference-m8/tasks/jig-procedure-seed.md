+++
id = "jig-procedure-seed"
part = "procedure-funding"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/effects/connector.rs", "jig/crates/jig-domain-core/src/effects.rs", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/tests/core/tests/suite/procedure_seeds.rs", "jig/tests/core/tests/suite/main.rs"]
effort = "high"
minutes = 60
+++

# jig: a deployment-started procedure that runs no agent, funded at the effect

## Goal

A procedure task the deployment starts on a project's behalf that runs no agent (a forge's
landing or tier run) is made with a budget of zero, so that a period its people have reserved
whole never stops it; each priced effect it asks for is funded at the effect from the
period's unreserved spend, and waits while there is none (jig's
`jig/docs/design/domain/authority.md`, 6). Today temper makes such tasks with
`Core::repair_seed`, which returns `None` when nothing of the period is unreserved, so the
task silently never starts. temper's tier and landing tasks switch to the new seed in a
later task of this plan.

Acceptance:

- `Core::procedure_seed(&mut self, project: u32) -> Option<RepairSeed>` exists, public,
  beside `repair_seed` in `jig/crates/jig-domain-core/src/effects/connector.rs`: the
  project's policy ceiling as its authority with spend 0, budget 0; `None` only when the
  project is not open, has no policy, or no task number is left. It calls `start_input()`
  as `repair_seed` does.
- A task made from it through the existing connector-repair route
  (`routing/task.rs`, `route_connector_repair`) is made even when the period is reserved
  whole; a zero-budget seed with an agent or person executor is refused there (`Decided`,
  no task).
- Its priced effects: admitted at the described price against every permission and both
  ceilings (`connector_effect_admit`), then `Wait` while the period's unreserved spend is
  below the price; when admitted, charged to the period (`effects.rs`, `charge`), not to the
  task. Every other task's admission and charging is unchanged.
- A story file `jig/tests/core/tests/suite/procedure_seeds.rs` in `jig-core-world`'s
  `suite` binary covering: made with a period reserved whole; an effect waits with no
  unreserved spend and is admitted and charged to the period once some is; a seed with an
  agent executor and budget 0 makes no task; a `repair_seed` task is funded as before.
  `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/procedure_seeds::/)'`
  passes with the count in the report.
- jig's `domain-core` unit tests and `jig-core-world`'s `suite` binary still pass, and
  temper's two `repair_seed` callers still compile:
  `heavy cargo check -p temper-engine-domain --tests`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`.
- `jig/docs/design/domain/authority.md`, 6 (the deployment's procedure tasks) and 7
  (effective authority, funding); `jig/docs/design/domain/tasks.md`, the funder of a task.
- The old commit: `temper/docs/plan/reference-m8/notes/old/jig-procedure-seed.patch`
  (`procedure_seed`, `procedure_effect_funder`, its admission and charge, the route's
  zero-budget refusal, a 382-line story file). Paths are re-rooted under `jig/`, but its
  files moved since: `src/connector.rs` is now `src/effects/connector.rs`,
  `src/routing/tasks.rs`'s `route_connector_repair` is now in `src/routing/task.rs`, the
  test belongs in `tests/core/tests/suite/` as a module of `main.rs`, and the old test's
  fixtures (`effects::fixture`, `fixtures::seeded_project_rows`) are not in today's
  `jig-core-world`: build the core from today's fixtures (`fixtures.rs`, `world.rs`; see
  `suite/funding.rs`).

## Owns and may change

- Owns the files in the header. `procedure_effect_funder` stays `pub(crate)`.
- Reader of `RepairSeed`: `jig/crates/jig-domain-core/src/vocabulary.rs` (unchanged: the
  seed's type does not change).
- Read only: temper's callers of `repair_seed`
  (`temper/crates/temper-engine-domain/src/route/forge_route/landing_route.rs:188`,
  `route/forge_route.rs:2587`, `src/tests/setup.rs:553`): this task does not switch them.

Pre-approved options:

- If today's `connector_effect_admit` or `charge` has a shape the old hunks do not fit, keep
  the meaning (zero-budget deployment procedure: checked at the price, funded from the
  period, `Wait` while short) in today's shape and say how.
- If the story needs a fixture `jig-core-world` lacks (a period reserved whole, a
  procedure executor), add it to `jig/tests/core/src/` as a small helper, and list it.

## Steps

1. Read the old patch; port `procedure_seed` and `procedure_effect_funder` into
   `effects/connector.rs`, the admission into `connector_effect_admit`, the funder into
   `effects.rs`'s `charge`, the zero-budget check into `route_connector_repair`. Doc
   comments cite `jig/docs/design/domain/authority.md, 6`.
2. Write the story file; run it alone first, then `jig-domain-core`'s lib tests
   (`heavy cargo nextest run -p jig-domain-core --lib`).
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world --all-targets --
   -D warnings`; `heavy cargo nextest run -p jig-core-world --test suite`;
   `heavy cargo check -p temper-engine-domain --tests`.
4. Commit (why: deployment procedures start under a reserved period); push your branch.

## Report

As the implementer's text says, and also: the stories and their count; how the admission's
`Wait` is reached (which caller retries it); anything of authority.md 6 left unbuilt.
