+++
id = "sv-world-real-loop"
part = "fs"
kind = "migration"
needs = ["sv-seed-kit", "sv-registry-setup"]
files = ["temper/tests/real-loop/**"]
effort = "high"
minutes = 110
+++

## Goal

The real-loop world's deployments name no forge (`notes/contracts.md`, T4;
`notes/sv-progress.md`, request 1 (c)): its documents carry no `application.forges`,
`identities` or `setup`, and the forge each story ran on is kept as a record with its
credential records in the deployment's store (`sv-seed-kit`), so every story runs as before
on the records path.

Acceptance:
- `temper/tests/real-loop/src/fixtures.rs` (`configuration`'s document and its
  `secrets/forge-1.*` files), `src/forge.rs` (the webhook declaration), `src/github.rs`
  (`write`'s GitHub forge and identity), `tests/suite/github.rs`: no `application.forges`,
  `identities` or `setup` member, and no `secrets/forge-1.*` file.
- Each deployment's forge is seeded with `temper_github_app::seed_forges`: the fake Forgejo
  as `forgejo(1, forge, b"http://<forge>")` with `forgejo_records(1, <today's api token>,
  <today's user>, <today's webhook secret or None>, <today's git token>, forge)`; the GitHub
  stories' forge as `github(1, address, ..)`, plaintext to its loopback address as today (an
  IP literal with `ApiTransport::Plaintext`: no resolve), seeded together with the App's
  records where `src/service.rs` seeds them today (`temper_github_app::seed`). The world's
  identity (`temper`, `Temper`/`temper@example.test`) is the record's bot.
- Every test of `temper-real-loop-world` (default profile; its `qa_all` binary is not run)
  passes; a test that asserted a document member that no longer exists asserts the record's
  equivalent.

## Inputs

- `temper/tests/real-loop/` (its module docs), `temper/docs/design/domain/testing.md` for
  its stories.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md`, T1, T4.
- `temper/docs/plan/playground-reset-r6/tasks/sv-seed-kit.md` and its report (when to seed:
  before the first start or after an empty one).
- `temper/docs/plan/playground-reset-r6/tasks/sv-registry.md` (how a seeded Forgejo record
  connects at start).

## Owns and may change

- Owns: `temper/tests/real-loop/` (package `temper-real-loop-world`; its `qa_all` binary at call
  sites only).
- Pre-approved options:
  - Seeding point: before the world's first start if `sv-seed-kit` says an empty data
    directory takes it; else the fixture runs one empty start first (as its GitHub seed does
    today) and seeds after it. Default as the kit's report says.
  - A story whose trace counts a start's steps may gain the empty start's; adjust its
    expected counts only for that, and list each.
  - If a story needs the document's forge in a way the records cannot give (a refusal of a
    document member), it stays on the old document until `sv-startup-refuse` and is listed
    under `## Left`.

## Steps

1. Fixtures, forge, github and service seeding as above.
2. `heavy cargo nextest run -p temper-real-loop-world` (whole, default profile); read
   each failure's log before a rerun.
3. `cargo fmt`; `heavy cargo clippy -p temper-real-loop-world --all-targets`; old forms.
   Commit and push.

## Report

- The seeding point taken; every story whose expectations changed and why.
