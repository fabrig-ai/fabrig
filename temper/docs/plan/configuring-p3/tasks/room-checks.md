+++
id = "room-checks"
part = "room"
kind = "feature"
needs = ["room-table", "stock-configuring", "agent-kinds"]
files = ["temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/tests/room_checks.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/startup/problems.rs", "temper/crates/temper-service/src/tests/room.rs", "temper/crates/temper-service/src/tests.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 150
+++

# room-checks: no charter leaves its brief less than 2,048 bytes

## Goal

temper refuses a charter that leaves its brief less than 2,048 bytes of smith's host part,
naming the charter and both figures, at a start and at the three entrances of a change of
settings (temper's `domain/agent.md`, 4.3, "Checked before a run meets it"; contract C2),
by `room-table`'s `HostPart`.

Acceptance:

- **The figure reaches the root.** `temper-service` builds
  `temper_engine_smith::room::host_part(&agent.run_limits)` where the engine has an agent
  configuration (`settings::Agent`), and gives it to the engine root at its construction as
  `Option<temper_engine_domain::room::HostPart>` (a field of whatever the root is built from;
  `None` in a world that names no agent); a root given `None` checks nothing.
- **At a start:** each charter of the current settings is checked before the core admits
  anything: the defaults in `startup/checks.rs` beside `checks::agent` (the stock's or a
  world's), refused as a start problem naming the charter as the start names a stock setting
  (`charter 3`) with `Short::words()`; and the stored changes' charters, once the store has
  loaded them, refusing the start the same way (the path
  `a_stored_change_that_no_longer_checks_refuses_the_start` in the engine's domain world
  takes).
- **At the entrances:** the root's `ChangeSettings` and `ProposeSettings` routes
  (`route.rs`) refuse a set whose charter item fails, as
  `Refusal::Judged { at, setting, why }`, before the core sees it; a run's `propose_settings`
  (`route/tools.rs`) is answered `SettledAnswer::Host { error: true, body }` with the words,
  the run going on, before the core sees it.
- Tests: in `temper-service/src/tests/room.rs`, a start whose own defaults hold a charter of
  instructions long enough to leave 2,047 bytes is refused naming it and both figures, and one
  leaving 2,048 starts; in `temper-engine-domain/src/tests/room_checks.rs`, a party's change
  and proposal with such a charter refused `Judged` naming the item, an accepted stored
  proposal and a reset not checked, a run's `propose_settings` answered as an error and the
  run's next call decided, and a root given no `HostPart` checking nothing. The stock's five
  charters start in the small profile (`temper-service`'s existing start tests).
- temper's `docs/design/domain/agent.status.md`: the sentence "4.3: the check that each
  charter leaves its brief 2,048 bytes, at start and at the three entrances of a change of
  settings; only fitting exists (`temper-engine-smith/src/start.rs`)." removed.
- The focused tests of `temper-engine-domain`, `temper-service`, the engine's domain world and
  the system world pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.
- `temper/docs/design/protocol/configuration.md`, section 7 (the small profile's host part)
  and 13.4.
- `jig/docs/design/domain/engine.md`, 17.2 (a change and its refusals), 17.6.
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C2.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`.
- Reader files: the root's constructor's callers, the new argument or field alone (worlds'
  builders in `temper/tests/*/src`, `temper-playground`'s, `temper/crates/temper`'s), each
  passing `None` unless it configures an agent whose figure it wants checked; list each.
- Options, pre-approved:
  - If `route.rs`'s settings routes have no reply path for a refusal before the core, refuse
    through the people child's refusal of the request as `translation.rs` does for a kind's
    value, and say which.
  - If the stored changes' charters are only reachable inside jig's start, check them in the
    root's first decision after the store's settings load, refusing the start as a stored
    change that no longer checks is refused, and say how.
  - A world whose own defaults hold a charter that now fails the check: report it with its
    figures; do not lengthen or shorten its instructions.

## Steps

1. The figure to the root.
2. The start's checks.
3. The two routes and the run's entrance.
4. The tests; the status sentence.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` naming every
   package you changed, `--all-targets -- -D warnings`; the focused tests above; the citations
   check.
6. Commit ("temper: a charter leaves its brief 2,048 bytes, checked at a start and at a
   change's entrances (agent.md, 4.3)") and push.

## Report

As the implementer's text says, and each stock charter's room in the small profile.
