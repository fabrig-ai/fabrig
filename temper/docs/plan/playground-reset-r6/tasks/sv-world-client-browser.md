+++
id = "sv-world-client-browser"
part = "fs"
kind = "migration"
needs = ["sv-seed-kit", "sv-registry-setup", "bx-init"]
files = ["temper/tests/client-browser/**"]
effort = "medium"
minutes = 70
+++

## Goal

The client-browser world's deployments name no forge (`notes/contracts.md`, T4;
`notes/sv-progress.md`, request 1 (c)). It runs the `temper` binary on documents it writes
(`src/configuration.rs`) or on a `temper init` it edits (`src/lib.rs`); after this task
neither names `application.forges`, `identities` or `setup`, and the forge a story ran on
(an unreached Forgejo, or the fake GitHub) is kept as a record in the deployment's store
before `temper serve` starts.

Acceptance:
- `src/configuration.rs`: the written document has no `forges`, `identities`, `setup`
  member and no `secrets/forge-1.*` file; its forge (`UNREACHED`) is seeded with
  `temper_github_app::seed_forges` as `forgejo(1, UNREACHED, ..)` with no credential
  record (an unreached forge connects nothing, as today), before the binary's first start
  or after an empty one, as `sv-seed-kit`'s report says.
- `src/lib.rs`: the GitHub substitution sets `application.machine.github`'s members on the
  init's document (bx-init writes `application.machine`) and seeds forge 1 as
  `github(1, ..)` with the App's records where it seeds them today; the lib's own tests
  (`tier_configuration`'s assertions) read the record's address instead of the document's.
- `heavy cargo nextest run -p temper-client-browser-world --lib` passes;
  `heavy cargo check -p temper-client-browser-world --tests` builds its `browser_all` and
  `qa_all` binaries (not run: they need a browser and are not in the gate).

## Inputs

- `temper/tests/client-browser/src/configuration.rs` and `src/lib.rs`.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md`, T1, T4.
- `temper/docs/plan/playground-reset-r6/tasks/sv-seed-kit.md` and its report;
  `tasks/bx-init.md` and its report (the init document's shape).

## Owns and may change

- Owns: `temper/tests/client-browser/` (package `temper-client-browser-world`); its
  `Cargo.toml` gains `temper-github-app` (workspace dependency, as the other worlds have it).
- Pre-approved option: if seeding a store the binary has not made yet is refused, run the
  binary's first start as the world's fixture already starts it, stop it, seed, and start
  again; say so.

## Steps

1. The two documents and the seeding.
2. `heavy cargo nextest run -p temper-client-browser-world --lib`; `heavy cargo check -p
   temper-client-browser-world --tests`.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets`; old
   forms. Commit and push.

## Report

- Whether the browser binaries were run anywhere (they are not in this task), and what in
  them still reads a document forge, under `## Left`.
