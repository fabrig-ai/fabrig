+++
id = "tools-offered"
part = "charter-tools"
kind = "feature"
needs = ["tools-surface"]
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/start.rs", "temper/crates/temper-engine-smith/src/tests.rs"]
effort = "medium"
minutes = 45
+++

# tools-offered: the settings family offered only where the charter names it

## Goal

A run is declared `settings` and `propose_settings` only where its charter's `tools` and its
task's authority both hold jig's settings family (temper's `domain/agent.md`, 4.3; contract
C1). Every other host tool is declared as today. Until the stock gives its charters their own
masks (`stock-configuring`) every charter carries `RunPolicy::EVERY_FAMILY`, so no run's
offer moves on this branch.

Acceptance:

- `temper_engine_smith::calls::tools_for` takes the charter's mask beside the authority's:
  `pub(crate) fn tools_for(timeout: Duration, charter: tasks::Tools, authority: tasks::Tools) -> Box<[run::HostTool]>`;
  the settings declarations need `charter.0 & authority.0 & ToolFamilies::JIG.settings != 0`;
  message and plan stay the authority's alone, as now.
- `start.rs` passes `assignment.run.policy.tools`.
- The step test `settings_tools_are_offered_only_with_the_runs_settings_family` becomes a
  table over (charter mask, authority) of the four combinations of settings present or absent,
  the tools offered only where both hold it, the other declarations unchanged in each; its
  name stays, and a second test,
  `settings_tools_need_the_charters_family_too`, holds that a charter of `512` (the read
  family alone) under an authority of `1_539 | 1024` is offered no settings tool and every
  other tool the authority's offer has.
- The existing callers of `tools_for` in `tests.rs` pass `RunPolicy::EVERY_FAMILY` as the
  charter's mask, their assertions unchanged.
- The focused tests of `temper-engine-smith` pass; `heavy cargo check -p temper-service
  --tests` and `-p temper-engine-domain-world --tests` are clean (crate names as the
  workspace has them).

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3 ("Every host tool below, declared in every
  charter, ... but those of the `settings` family").
- `jig/docs/design/domain/engine.md`, 7.1.
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C1.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: `temper/crates/temper-engine-smith/src/calls.rs` (`tools_for`), `start.rs` (its one
  call), `tests.rs` (the tests above and the callers' new argument).
- Reader files, the call's new argument alone: any other caller of `tools_for` the compiler
  names (none outside the crate on the trunk at the cut; `tools(timeout)`, the full registry
  the `host-tools` page renders, is unchanged).
- Options, pre-approved:
  - If a test elsewhere (`temper-service/src/tests/stock_brief.rs`, the engine's domain world,
    the system world) asserts a run's offered tools and fails on this branch, the charter's
    mask is not reaching the run as `EVERY_FAMILY`: stop and report it with the test and its
    log; do not change that test.

## Steps

1. `calls.rs`: the signature above; the settings condition reads both masks. Keep the doc
   comment's two lines, the first now "Settings declarations require the charter's and the
   run's settings family; message and plan declarations the run's families."
2. `start.rs:129`: pass `policy.tools`.
3. The tests, in `tests.rs`, beside the existing ones.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-smith --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-smith`; the two checks above.
5. Commit ("temper-engine-smith: offer the settings family only where the charter names it
   (agent.md, 4.3)") and push.

## Report

As the implementer's text says.
