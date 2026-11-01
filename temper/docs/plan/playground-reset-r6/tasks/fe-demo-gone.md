+++
id = "fe-demo-gone"
part = "pg"
kind = "migration"
needs = ["fe-playground-forge", "fe-playground-root", "fe-world-remove"]
files = [
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain/**",
  "temper/crates/temper-protocol-store/**",
  "temper/tests/codecs/**",
  "temper/tests/engine/**",
]
effort = "medium"
minutes = 80
+++

## Goal

The demo goes from the engine (draft `temper/docs/design/draft/playground-reset.md`, 6.3
"What goes"; `notes/contracts.md` T2, T6): its `Demo` and `DemoAct` rows and families, its
state, its end's acts and selection, its own start code, now that the playground runs on
records (`fe-playground-forge`, `fe-playground-root`). `DemoConfig` stays: it is the
playground's configuration (`application.demo` keeps its name).

Acceptance:
- Gone from the forge connector: `Demo`, `DemoState`, `DemoAct`, `DemoActKind`,
  `Stored::{Demo, DemoAct}`, `Key::{Demo, DemoAct}`, `RecordKind::{Demo, DemoAct}`,
  `reconcile::Need::Demo`, `Event::{DemoActs, DemoActAnswered, SendDemoActs, DemoAnswered}`,
  `Request::{ListDemoActs, SendDemoAct, DemoStart}`, `demo_act_bound` (and its use in
  `valid`), `demo_state()`, the demo's maps and fields, and every function of `src/demo.rs`
  but `DemoConfig::within`, `configure_demo` and `demo()` (move them into
  `src/playground.rs` and delete `src/demo.rs` if nothing else is left).
- Gone from the root: `route/demo.rs`'s start (`start`, `begin`, `progress`, `reply`,
  `adopted`, `connection`, `done`'s demo parts) and end (`list_end_acts`, `send_end_act`,
  `end_acted`, `end_key`), the root's `demo_*` fields, `SendDemoActs` in `restart.rs`, and
  `root::Domain::demo_state()`; `route/demo.rs` is deleted if nothing is left (the playground's
  context handling moves to `route/playground.rs`). The `close` act is sent by nothing.
- Gone from the store: families 17 and 19 (their schema records `DemoState`, `Demo`,
  `DemoClose`, `DemoRemoveAccount`, `DemoActKind`, `DemoAct` and the `Value` variants
  `demo`, `demo_act`); a key of byte 17 or 19 decodes to `None`; the bytes stay unused (a
  comment in `keys.rs` says so, as for 20 and 22); generated code and goldens regenerated,
  the removed records' golden files deleted (`git rm`). `STORE_VERSION` stays 11.
- Every test of the owned packages passes; tests of the demo's end and old start are
  deleted (list them), their playground equivalents are `fe-world-reset`'s.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T2, T6;
  `notes/fe-surface.md` sections 4 and 6.
- `temper/docs/design/draft/playground-reset.md`, 6.3.

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`, `temper/crates/temper-engine-domain/`,
  `temper/crates/temper-protocol-store/`, `temper/tests/codecs/`, `temper/tests/engine/*`.
- May change, compiler-forced only (list each file):
  - `temper/crates/temper-service/`: readers of `root::Domain::demo_state()` or
    `forge::DemoState` (`src/route.rs`'s mirror into `settings::Demo.state`,
    `src/family.rs`'s demo read, `src/tests/store.rs`'s `Stored::Demo` test). If sv's tasks
    have not yet moved them to `root::Domain::playground()`: default, `settings::Demo.state`
    becomes `Option<forge::Playground>` filled from `playground()`, and the demo read's
    state is `running` for a `Running` row, `resetting` for a `Resetting` one (the wire's
    `ended` if `cl-wire` has not landed), `not_started` for none; the `Stored::Demo` test
    is deleted. Say so in the report.
  - `temper/crates/temper-fake/src/check.rs` (the removed `Stored` arms).
  - `temper/tests/client/`, `temper/tests/system/`, `temper/tests/conformance/`: arms of
    removed variants.
- Pre-approved options:
  - `Delivery::DemoNote` and `Delivery::DemoActed` (or the `Acted` name `fe-kept-root`
    chose) stay: the kept and reset acts use them. Renaming `DemoNote` to `Note` is allowed
    only if its readers outside the owned crates are compiler-forced arms; default, keep the
    name.
  - `forge::Request::StopGroup` stays (serve has it; nothing in the engine sends it now).

## Steps

1. Delete in the order forge, root, store; fix the compiler-forced arms.
2. Regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test
   suite -E 'test(drift::)'`, delete the stale demo goldens, rerun without the variable.
3. `temper/tests/codecs/tests/suite/fuzzy/store.rs`: the demo draws go; keys 17 and 19
   checked refused in `temper-protocol-store/src/tests/keys.rs`
   (`demo_family_bytes_are_unused`).
4. Run, one at a time: `heavy cargo nextest run -p temper-protocol-store --lib`; `-p
   temper-codecs-world --test suite`; `-p temper-engine-domain-forge --lib`; `-p
   temper-engine-domain --lib`; `-p temper-engine-domain-world --test suite`; `-p
   temper-engine-forge-world`; `-p temper-engine-protocol-forge-world`; `-p temper-service
   --lib`; `-p temper-fake --lib`; `heavy cargo check --workspace --all-targets`; `cargo fmt
   --all`; `heavy cargo clippy` on every changed package `--all-targets -- -D warnings`;
   `python3 -I -B scripts/old_forms.py` from `temper/`.
5. Commit, push your branch.

## Report

- Every file outside the owned crates edited, and under `## Left` every reader of the
  demo's removed items you did not own and could not route.

## Rebase (2026-11-01, step merge, foreman playground-reset-r6/fe-demo-gone/foreman-1)

resolve merge conflicts while preserving landed playground record handling and removing legacy demo types.
The merge failed in nine paths; this is the branch’s red, not a tool failure.
Conflicting paths:
- `temper/crates/temper-engine-domain/src/route/playground.rs`
- `temper/crates/temper-service/src/{engine,family,forge,route,settings}.rs`
- `temper/crates/temper-service/src/tests/{family,setup,setup_admin}.rs`

In root `route/playground.rs`, preserve integration commit `f91a8c5d5b`:
keep `Start.maker: u64`, resolve the starting person even with an existing project,
and retain both Party and Project account uses. Apply this branch’s
`demo_contexts` → `playground_contexts` rename and moved context helpers.

Service conflicts overlap `2d063cbd01` (read playground state from engine records).
Keep its `Demo.playground` field, refreshes in `engine::iterate` and `route::request`,
and `family::to_root` mapping state, project and forge from that row.
Remove obsolete `Demo.state`, its `demo_state()` refreshes and compiler-forced literals;
the spec’s replacement `state` field was conditional on sv not having migrated readers.
In `forge::ended`, retain this branch’s direct `engine.domain.forge_ended(forge)`.

Keep integration’s expanded playground family tests; remove their obsolete DemoState setup.
Setup conflicts also include fixture migration commit `0614a006ea`.
Preserve its registered-forge fixtures and Removing-record coverage.
Where an old test now exercises Removing records, remove only legacy demo setup,
Stored::Demo writes and DemoState assertions rather than deleting that coverage.
List every compiler-forced service file changed in the implementer’s report.

No builds or changes performed. The read-only filesystem prevents writing the requested `report.md`; this response supplies its contents.
