+++
id = "rail-temper"
part = "charter-rail"
kind = "feature"
needs = ["rail-jig"]
files = ["temper/crates/temper-engine-smith/src/start.rs", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 15
+++

# temper leaves the deliver rail to jig

## Goal

`rail-jig` put the rail in `jig_protocol_charter::charter`. temper's root narrowed the same grant
after it; a root decides none of it (`jig/docs/design/domain/hosts.md`, 8), so temper's line goes
and its tests prove jig's rail holds for temper.

This is a temper task of a jig plan: `temper/AGENTS.md` applies beside `jig/AGENTS.md`.

Acceptance:

- `start.rs` has no `tools.modify` line, nor its comment.
- `a_report_run_cannot_modify_without_delivery`, `an_open_run_cannot_modify_even_when_policy_requests_it`
  and `a_finite_change_run_keeps_its_modify_policy` in `temper/crates/temper-engine-smith/src/tests.rs`
  pass unchanged: they are this task's test, red had `rail-jig` not landed.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 4.3; `jig/docs/design/domain/hosts.md`, 8.

## Owns and may change

- `temper/crates/temper-engine-smith/src/start.rs`: the comment and line that narrow `modify`.
- `temper/docs/design/domain/agent.status.md`: this plan's **Not yet** entry for 4.3.
- Other lines of `start.rs` stay; plan `fabrig-scale` works near it.

## Steps

1. Remove the two lines. `let mut charter` stays: the `charter.grants.host_tools` loop below needs
   it.
2. `heavy cargo nextest run -p temper-engine-smith -p temper-service`.
3. `cargo fmt`; old forms; `heavy cargo clippy -p temper-engine-smith --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks.
