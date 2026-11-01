+++
id = "pg-demo-out-engine"
part = "playground"
kind = "migration"
needs = ["pg-reset", "pg-demo-out-wire"]
files = [
  "temper/crates/temper-engine-domain-forge/src/demo.rs",
  "temper/crates/temper-engine-domain-forge/src/playground.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/demo.rs",
  "temper/crates/temper-engine-domain/src/route/demo.rs",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/tests/engine/domain/tests/suite/demo.rs",
]
effort = "high"
minutes = 70
+++

## Goal

The demo goes from the engine (draft
`temper/docs/design/draft/playground-reset.md`, 6.3, "What goes"):
`Ask::StartDemo` and its answer, the `Demo` and `DemoAct` rows' use,
the demo's end on another forge's connect, its kept `Close` and
`RemoveAccount` acts (`ListDemoActs`, `SendDemoAct`, `SendDemoActs`),
the closed chats "Ended with the playground", and with them the
engine's use of `engine.acts`' `close`. The playground's start and
reset (`pg-start`, `pg-reset`) and the `Kept` family (`kept-acts`)
replace them. The store's families 17 and 19 stay in the schema until
`pg-demo-out-store`; nothing writes or reads them after this task but
the restore arms, which drop them.

Acceptance:

- `temper-engine-domain-forge`: no `Ask::StartDemo`,
  `Answered::StartDemo`, `Demo`/`DemoState`/`DemoAct`/`DemoActKind`
  used by any decision; `demo.rs` is gone, what of it the playground
  still needs (`DemoConfig::within`, `configure_demo`, `demo()`,
  `forge_ended` as `rm-remove-forge` left it, `demo_adopted` if
  `pg-start` reads it) moved to `playground.rs` with its doc comments
  citing draft 6.1; `Request::{ListDemoActs, SendDemoAct, DemoStart}`
  and `Event::{DemoActs, DemoActAnswered, SendDemoActs, DemoAnswered}`
  gone, and `Request::StopGroup` if nothing else sends it;
  `demo_act_bound` gone, `kept_bound` no longer counting it (its
  formula in `limits.rs` without the demo's term, `valid` unchanged
  otherwise); a restored `Stored::Demo` or `Stored::DemoAct` row is
  dropped (no table holds it);
- another forge's connect ends nothing (the demo's `connected` gone);
- `temper-engine-domain`: `route/demo.rs` gone; `ApplicationAnswer::StartDemo`,
  `Delivery::DemoActed` and the demo's fields of `Domain` gone; the
  route's demo arms (`applications.rs`, `forge_route.rs`, `held.rs`,
  `restart.rs`, `deadline.rs`, `route.rs`, `admission.rs`,
  `decision.rs`) gone; `Delivery::DemoNote` stays if `kept-acts` uses
  it (renaming it is not this task's);
- `temper-service`: `settings::Demo.state` and its setting in
  `engine.rs` gone; every `StartDemo`/`demo_state` reader gone;
- the engine worlds and tests name none of these: the demo's suites
  deleted where the playground's suites (`suite/playground*.rs`) cover
  them, each listed;
- every package that named them builds its tests and passes its
  focused tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D4; section
  2, "Demo (17) and DemoAct (19) stay until the playground part's task
  that removes the demo"; section 4, `close`).
- `temper/docs/design/draft/playground-reset.md`, 6.3.
- `temper/docs/plan/playground-reset-r4/notes/surveys/s2-playground.md`,
  sections 1 and 2 (every reader of the demo's items at the base; the
  run's tasks moved some).
- The merged reports of `pg-start`, `pg-reset`, `kept-acts`,
  `rm-remove-forge`.

## Owns and may change

Owned: the `files` above (deleted, or edited for what moves).

Neighbour files, edits allowed: every file of the temper tree naming
the removed items (the connector's `boundary.rs`, `domain.rs`,
`lib.rs`, `limits.rs`, `restore.rs`, `reconcile.rs`, `connect.rs`,
`tests.rs` and tests; the root's files named above and its tests;
`temper-service`'s `settings.rs`, `engine.rs`, `forge.rs`, `route.rs`,
`clients/translation.rs`, tests; `temper/tests/engine/**`,
`temper/tests/system/`, `temper/tests/client/`,
`temper/tests/one-process/`, `temper/crates/temper/tests/**`):
the use removed, or moved to the playground's item it now is. A
slow-tier test whose story is the demo's end
(`temper/crates/temper/tests/process.rs`,
`a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`
and its helpers; qa tests asserting "Ended with the playground") is
deleted, listed, not run.

Pre-approved:

- if `forge_ended` or `configure_demo` has a reader outside the
  connector that names the module path, keep the method on `Domain`
  (an `impl Domain` block in `playground.rs`);
- if a world fixture's `DemoConfig` literal named a configured forge for
  the demo, leave it: the field stays (`pg-config`);
- if `Request::StopGroup` is still sent by the setup's own paths, keep
  it.

## Steps

1. Connector: delete as the acceptance says; move the kept items.
2. Root: delete `route/demo.rs` and its arms; the playground's route
   stands alone.
3. Service, worlds, tests: the compiler's list.
4. Tests: the demo's unit tests and world suites deleted or moved to
   the playground's where they still say something true (a start's
   failure words, the deadline), each listed; add
   `another_forge_connecting_ends_nothing` to
   `tests/engine/domain/tests/suite/playground.rs` (a running
   playground and a configured real forge connecting: the row stays
   `Running`, no kept row) — a neighbour edit of that file, one test.
5. `cargo fmt`; clippy on every package changed; focused tests of
   `temper-engine-domain-forge`, `temper-engine-domain`,
   `temper-service`, `temper-engine-domain-world`,
   `temper-engine-forge-world`, `temper-engine-protocol-forge-world`,
   `temper-system-world`; `heavy cargo check --tests` for `temper`,
   `temper-one-process-world`, `temper-client-world`,
   `temper-conformance-world`, `temper-real-loop-world`,
   `temper-codecs-world`, one at a time. Commit, push.

## Report

fab's sections; every test deleted or moved, by file and name; what
moved to `playground.rs`.

## Rebase (2026-11-01, step merge, foreman playground-reset-r4/pg-demo-out-engine/foreman-1)

retain first_intent test registration while removing demo, and update its legacy demo fixture.
The merge conflicts in `temper/crates/temper-engine-domain-forge/src/tests.rs`.
Incoming commit `19fcb2f995` (“Keep first repository intents so setup resumes after restart”), merged by `8c2e4b3c6d`, adds `mod first_intent;` beside the deleted `mod demo;`.
Resolve that block with `mod connect_new;`, `mod first_intent;`, `mod fixtures;`; leave `mod demo;` deleted.
Preserve this branch’s removal of obsolete request match arms elsewhere in `tests.rs`.
Also adapt incoming `src/tests/first_intent.rs`, test `retained_repositories_take_the_project_even_when_the_forge_is_ended_or_removing`.
It restores `Stored::Demo(Demo { ... DemoState::Ended })`, which this branch correctly ignores.
Remove that test’s `configure_demo` and legacy Demo restore block.
Keep the removing Forge record fixture and the assertions that `forge_ended(1)` and `project_has_repository(5)` hold.
This preserves repository retention coverage using removal state, consistent with draft 6.3 and the task’s allowed neighbour edits.
No files changed or builds run; the read-only environment prevents writing report.md.
