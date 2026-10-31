+++
id = "pg-gone"
part = "playground"
kind = "migration"
needs = ["pg-reset", "pg-models", "rm-world"]
files = [
  "temper/crates/temper-engine-domain-forge/src/demo.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/demo.rs",
  "temper/crates/temper-engine-domain/src/route/demo.rs",
  "temper/tests/engine/domain/tests/suite/demo.rs",
  "temper/crates/temper-protocol-store/**",
]
effort = "medium"
minutes = 75
+++

## Goal

What goes (draft 6.3), engine and store side: the demo's record and its
end on another forge's connect, with everything only they used. After
this task temper has no `Demo`, `DemoState`, `DemoAct`, `DemoActKind`,
`DemoConfig`, no `Stored::Demo`/`DemoAct` family, no `engine.acts`
`close` grant in what `temper init` writes (the reset's grants
`remove_project`, `remove_account` and `cancel` stay), and no code that
ends the playground when another forge connects. Names that the
playground and the removal still use lose "demo".

Acceptance:

- `rg -n 'DemoState|DemoAct|DemoConfig|Stored::Demo|Key::Demo|demo_act_bound|ListDemoActs|SendDemoAct|SendDemoActs|StopGroup' temper/crates temper/tests`
  finds nothing (the client crates' own `demo` names are D5's and not
  this task's: exclude `temper-client-*`, `temper/tests/client`).
- `temper-protocol-store`'s tests pass with the two families gone
  (in place, their key bytes 17 and 19 left unused and commented so in
  `keys.rs`), goldens regenerated; the codec world's store tests pass.
- A forge connecting while the playground runs leaves the playground
  running (new test in `temper-engine-domain-forge/src/tests/playground.rs`:
  `another_forge_connecting_leaves_the_playground_running`).
- The init template's `engine.acts` grants the forge connector
  `remove_account`, `cancel`, `remove_project` and not `close`;
  `temper-init`'s tests and every fixture reading the template pass.
- Every workspace package builds and its focused tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 ("`Demo`
  and `DemoAct` stay until the `playground` part removes them").
- `temper/docs/design/draft/playground-reset.md`, 6.3 ("What goes").
- `temper/docs/design/protocol/store.md`, 2 and 4 (families in place
  until the first release).
- Survey of readers: every `rg` hit of the acceptance's pattern, and
  `temper-protocol-store`'s `schema/store-v10.schema` (`Demo`,
  `DemoAct`), `src/records.rs`, `src/keys.rs`, `golden/*demo*`.

## Owns and may change

Owns the files in the header (the two `demo.rs` modules of the domains
are deleted, or emptied to nothing and removed from their `mod` lists).

May change, as the work needs, to delete or rename what goes and only
that: the forge crate (`boundary.rs`, `domain.rs`, `lib.rs`,
`limits.rs`, `restore.rs`, `reconcile.rs`, `connect.rs`, `remove.rs`,
`playground.rs`, `reset.rs`, tests); the root (`boundary.rs`,
`domain.rs`, `decision.rs`, `restart.rs`, `route.rs`,
`route/forge_route.rs`, `route/held.rs`, `route/kept.rs`,
`route/reset.rs`, `store.rs`, tests); `temper-service` (`settings.rs`,
`startup/`, `engine.rs`, `route.rs`, `system.rs`, `clients.rs`,
`clients/`, `lib.rs`, tests); `temper-init` (template and tests);
`temper-fake` (`src/demo.rs`'s names only where they name a type that
goes; its tests); the binary's `src/setup.rs`, `src/fake.rs` and
`tests/` fixtures that grant `close` for the demo end (drop `close`
from the grant only); every test world (`temper/tests/**`, including
`temper/tests/client` for forced arms only). `notes/scope.md` applies.
A help page under `temper/docs/help/` naming the demo's end may be
edited only to drop that name (a departure from the standing rules,
allowed here).

Renames (`notes/scope.md`'s forced arms follow): root
`Delivery::DemoActed` → `Delivery::Acted`, `Delivery::DemoNote` →
`Delivery::Note`; the root's `demo_*` fields that pg-start and rm-domain
still use → names without `demo`. Keep a name if renaming it would touch
a client crate beyond a forced arm.

Pre-approved: if `Request::StopGroup` still has a producer other than
the demo's end (bx-serve's), keep it and drop it from the acceptance's
pattern, saying so.

## Steps

1. Delete the demo's end in the forge connector (`connected`, `acts`,
   `send_acts`, `act_answered`, the `demo*` fields, `forge_ended`'s
   demo clause, `retire_ended`'s remains of it) and the families
   (`Stored::Demo`, `Stored::DemoAct`, their keys, `stored_key`,
   `stored_bytes`, restore and `RecordKind` arms, `demo_act_bound`).
2. Delete the root's end (`list_end_acts`, `send_end_act`, `end_acted`,
   `end_note`, the `SendDemoActs` push in `restart.rs`, the demo's
   delivery accounting) and route `Delivery::Acted` by reply token to
   `kept` or `reset` alone.
3. The store codec: remove the two values and keys from the schema in
   place, regenerate (the crate's generator command, as forge-families
   used it), delete their goldens, adjust the size tests.
4. The service and init: drop the demo settings' remains, its state
   mirror and the startup checks naming `application.demo`; the
   template's grant. Fixtures across `temper/tests/**` and the binary's
   tests drop `close` from `engine.acts` where only the demo end used
   it (a test that grants `close` for its own `Act::Close` keeps it).
5. Tests: the forge crate's and the root's demo-end tests and the world
   suite's `demo.rs` go (each named in the report); the new
   `another_forge_connecting_leaves_the_playground_running`.
6. Then, one at a time:
   - `heavy cargo nextest run -p temper-protocol-store -p temper-codecs-world`;
   - `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain`;
   - `heavy cargo nextest run -p temper-service -p temper-init -p temper-fake`;
   - `heavy cargo nextest run -p temper-engine-domain-world -p temper-engine-forge-world -p temper-engine-protocol-forge-world -p temper-system-world -p temper-conformance-world -p temper-one-process-world -p temper-real-loop-world -p temper-client-world -p temper`;
   - `heavy cargo nextest run --workspace --no-run` (every binary, the
     process tier's included, builds);
   - `cargo fmt`; `heavy cargo clippy` over every package changed,
     `--all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`, lowering with
     `--write` what the deletion lowers.
7. Commit (store families; engine; service and init; tests) and push.

## Report

The standard report, plus: each test deleted, by package and name;
each rename and the files its forced arms touched; whether `StopGroup`
went; the old-forms counts lowered.
