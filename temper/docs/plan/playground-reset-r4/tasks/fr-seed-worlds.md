+++
id = "fr-seed-worlds"
part = "forge-records"
kind = "migration"
needs = ["fr-first-intent", "fr-setup", "rm-stories", "rm-release-project", "pg-reset-crash", "pg-demo-out-store"]
files = [
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/record.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/**",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "temper/crates/temper-engine-domain/src/tests/**",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/tests/engine/**",
]
effort = "medium"
minutes = 50
+++

## Goal

The old constructor path goes (contracts 2: "no other crate builds
`Configured` by hand once the seed task lands"): `forge::Domain::new`
loses its `identities` and `forges: &[Configured]` parameters and
`temper_engine_domain::Config` its `identities` and `forges` fields;
every world and test that built `Configured` or `GitIdentity` by hand
builds its forges as `Forge` records in `Config.seed`. Behaviour does
not change.

Acceptance:

- No `Configured { .. }` literal and no `GitIdentity { .. }` literal is
  left outside `temper-engine-domain-forge/src/record.rs` (`rg` shows
  none); `Configured` is built by `Configured::from_record` alone.
- `record.rs` gains the builders the worlds need, a name this task
  creates: `Forge::connected(number: u16, kind: Kind, git: &[u8]) ->
  Forge` (records-only, no setup, identity number = forge number, empty
  login and author) and `Forge::with_setup(self, setup: Setup) -> Forge`;
  a world sets other fields directly.
- Every test of the connector, the root and the engine worlds
  (`temper/tests/engine/domain`, `forge`, `protocol-forge`) passes
  unchanged in what it asserts.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 2's
  last paragraph).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  section 3, "`Configured` reverse dependencies" (the test and world
  files that build it) and the `GitIdentity` list after it.
- The merged `fr-seed` report (the seed path and the old one).

The lead ordered this task after the playground's engine tasks
(`pg-start`, `pg-reset`, `pg-reset-crash`, `pg-demo-out-engine`,
`pg-demo-out-store`) and `rm-release-project`, which add tests in the
same trees: it starts from a trunk that holds them, and migrates their
literals too.

## Owns and may change

Owned: the files in the header, but another part's test files under
them (`temper-engine-domain-forge/src/tests/playground.rs`,
`temper/tests/engine/domain/tests/suite/playground.rs`: `pg-start`'s),
which are neighbours for their literals only.

Neighbours, each only to build forges as records in place of
`Configured`/`GitIdentity` literals (or to drop a removed field):
`temper-service/src/tests/*`, `temper/tests/client/**`,
`temper/tests/system/**`, `temper/tools/gallery/**`. The client world's
files are the playground part's: touch only those literals.

Pre-approved options:

- If a test asserts on a `Configured` value the old literal held that
  `from_record` cannot give (a `git` the record cannot carry), build the
  record that gives it, or assert on `from_record`'s value; list each.

## Steps

1. Remove the old path (`domain.rs` of both crates, `translate.rs`).
2. Builders in `record.rs`.
3. The literals, file by file; `heavy cargo check --tests` per package
   as you go.
4. `cargo fmt`; `heavy cargo clippy --all-targets -- -D warnings` for
   every package changed; the focused tests of
   `temper-engine-domain-forge`, `temper-engine-domain`,
   `temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-service`; `heavy cargo
   check --tests` for `temper`, `temper-client-world`,
   `temper-system-world`, `temper-one-process-world`,
   `temper-conformance-world`, `temper-real-loop-world`,
   `temper-client-gallery`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Domain::new`'s and `Config`'s
final shape, the builders.
