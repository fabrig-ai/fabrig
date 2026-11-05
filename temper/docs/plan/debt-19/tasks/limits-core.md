+++
id = "limits-core"
part = "limits-once"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-tasks/src/domain.rs", "jig/crates/jig-domain-tasks/src/lib.rs", "jig/crates/jig-domain-fleet/src/limits.rs", "jig/crates/jig-domain-fleet/src/domain.rs", "jig/crates/jig-domain-fleet/src/needs.rs", "jig/crates/jig-domain-core/src/limits.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/profile/**", "jig/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 70
+++

# jig: the core alone holds the settings' revisions and charter versions

## Goal

The core's half of `jig/docs/development/debt/D009-limits-mapped-by-hand.md`. `tasks.revisions`, `tasks.versions` and `fleet.versions` are copies whose only check is equality with `settings`' (`jig-domain-core/src/limits.rs`, `valid`), so they go. The tasks child and the fleet take the two values when they're made, from the core, which owns them (`jig/docs/design/domain/engine.md`, 17.8). No configuration, stored or wire format changes.

Acceptance:

- `rg -n '\b(pub )?(revisions|versions): u32' jig/crates/jig-domain-tasks/src/limits.rs jig/crates/jig-domain-fleet/src/limits.rs` finds nothing, nor does `rg -n '(tasks\.revisions|tasks\.versions|fleet\.versions)' jig temper -g '*.rs'`.
- The same limits are refused as at base: `u32::MAX` revisions or versions fail at start, through `tasks::valid` and the core's `checked_valid`.
- `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-fleet -p jig-domain-core -p jig-protocol-config -p jig-tasks-world -p jig-fleet-world -p jig-core-world -p jig-store-world -p jig-protocol-store -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-domain-world` passes, assertions unchanged but for the exceptions below. `heavy cargo check --workspace --all-targets` compiles.
- By hand: size `holders` from `kept.versions` instead of `kept.revisions` (a tasks test goes red); pass `versions + 1` to the fleet (a fleet test goes red, or say which). Restore each.
- Net production at most +10 lines (estimate -5 to +10). `jig/docs/design/protocol/configuration.status.md`'s **Not yet** entry for 6 goes.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D009-limits-mapped-by-hand.md`
- `jig/docs/design/domain/engine.md`, 17.8; `jig/docs/design/protocol/configuration.md`, 6

## Owns and may change

- **Owns:** the files in `files`.
- **The shape:**

  ```rust
  // jig_domain_tasks
  /// What the core's settings hold, which this child sizes by (domain/engine.md, 17.8).
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub struct Kept { pub revisions: u32, pub versions: u32 }

  pub fn valid(limits: &Limits, kept: Kept) -> bool;
  impl Domain { pub fn new(limits: &Limits, kept: Kept, seed: u64, newest: u32, charters: .., efforts: ..) -> Domain; }

  // jig_domain_fleet
  pub fn valid(limits: &Limits, versions: u32) -> bool;
  impl Domain { pub fn new(limits: &Limits, versions: u32, needs: Needs, catalogue: ..) -> Domain; }
  ```

  The tasks `Domain` keeps `kept` for `Event::Charters`' check, and the fleet's keeps its `versions` for `needs.rs`' room. A child can't see up, so a derived method can't do this. The opposite (keep the children's, drop `settings.*`) leaves the fleet's copy and contradicts 17.8.
- **Reader files, compiler-forced:** the literals that set the copies: `jig/testing/jig-test-profile/src/lib.rs`, `jig/crates/jig-domain-core/src/tests.rs` and `src/settings/tests.rs`, `jig/crates/jig-protocol-store/src/tests/fixtures.rs` and `tests/codec/settings_scale.rs`, `jig/tests/store/src/root.rs`, `temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs`, `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`, `temper/tests/engine/domain/src/walking.rs`; and the `Domain::new` calls in `jig-domain-tasks`, `jig/tests/tasks`, `jig/tests/fleet`, the core's tests. Each edited only for the field or the argument. In temper these are test fixtures alone; no temper client file changes.
- **`jig-protocol-config`:** `overrides.rs` writes `settings.revisions` and `settings.versions` alone (about lines 150, 300, 586 to 590), and the operator's names `core.settings.revisions` and `core.settings.versions` don't change. `profile/small.rs` and `standard.rs` drop the copies.
- **The test exceptions, each because the state can't be written any more:**
  - `task_revisions_must_equal_settings_revisions`, `task_versions_must_equal_settings_versions` and `fleet_versions_must_equal_settings_versions` (`jig-domain-core/src/tests/limits.rs`) are deleted.
  - `jig-protocol-config`'s `tests/overrides.rs` (about lines 420 to 430) stops asserting the copies: what it asserts of `settings.*` stays.
  - `jig-domain-tasks/src/tests.rs`' "revision holders" tests (about lines 101, 104) reach the rule through `valid(&limits, kept)`, and still assert it.
- **If a memory ceiling trips** (`jig/tests/tasks/tests/suite/memory.rs`), stop and ask, with the measurement: `Domain` grows by `kept`, and no budget moves.
- **Stay off:** `jig-client-protocol-link` (`limits-link` owns it), `jig-domain-core/src/documents/small.rs`, `runs/brief_text.rs`.

## Steps

1. Tasks: `Kept`, `valid`, `Domain::new`, `validate_charters`, `charter_capacity`.
2. Fleet: `needs_bytes`, `Domain::new`.
3. Core: the equalities go, `checked_valid` takes the overflow checks, `lib.rs` (about lines 555, 578) passes the values.
4. Config, then the readers by compiler. Break the rules by hand, as the acceptance says. Delete the status entry.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` naming every package you changed `--all-targets -- -D warnings`; the tests above; `heavy cargo check --workspace --all-targets`. **This departs from the implementer's text** because 38 packages read `jig-domain-tasks`, and a literal's field is forced everywhere. The citations check. Commit and push.

## Report

The common note's items, what you broke and what failed, and the net lines.
