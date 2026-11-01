+++
id = "sv-world-real-loop"
part = "fs"
kind = "migration"
needs = ["sv-seed-kit", "sv-registry-setup"]
files = ["temper/tests/real-loop/**", "temper/crates/temper-service/src/setup/lifecycle.rs", "temper/crates/temper-service/src/tests/registry_setup.rs", "temper/crates/temper-service/src/forge/records.rs"]
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

## Answers

2026-11-01. Q: the records-only webhook story panics with `same forge writer` because
`setup::lifecycle::connected` reports `config.user` (0) for a registered Forgejo without a
setup entry; may this task fix it in `temper-service`? A: yes, this task fixes it. The
owners cannot take it in time: `sv-registry-setup` is merged, `sv-startup-refuse` waits on
this task, and `sv-resolve` is about held records. The task's `files` now name
`temper/crates/temper-service/src/setup/lifecycle.rs` and
`temper/crates/temper-service/src/tests/registry_setup.rs`, and nothing else of the service.
- In `connected`'s last branch (no GitHub, no `Local`): when the forge's records hold
  `Item::User`, report that id, as the `Local` branch does; take the login from the
  registered record's `bot.login` when the forge is registered (the registry
  `crate::forges` keeps; the same login `setup.rs` gives the host's `Given` for a
  record with no entry); otherwise, for a document forge, keep `config.user` and the
  identity table as today. Change nothing else of `connected` or the file.
- Add one case to `src/tests/registry_setup.rs`: a registered Forgejo record with no setup
  entry and a seeded `User` connects as `Connected { id: <the record's User>, login:
  <its bot login> }`. Run it with `heavy cargo nextest run -p temper-service --lib -E
  'test(/tests::registry_setup::/)'` (a module of the lib's tests, not a test binary), then this task's own acceptance (the whole `temper-real-loop-world`
  default run).
- Keep the empty-start seeding point you took. Leave the other service readers your report
  lists under `## Left` (`startup/application.rs`, `forge.rs::config`,
  `setup/settings.rs::fixed`) to `sv-startup-refuse`.

2026-11-01. Q (run 2): the answer names `temper-service/tests/registry_setup.rs`, which does
not exist; may the case go into `src/tests/registry_setup.rs`? A: yes, that path was the
answer's slip, corrected above in `files` and the bullets: add the case to the lib's
existing module `src/tests/registry_setup.rs` and run it with `heavy cargo nextest run -p
temper-service --lib -E 'test(/tests::registry_setup::/)'`. `sv-world-one-process` now
waits on this task for the `connected` fix, so land it before anything else.

2026-11-01. Q (run 3): may this task also fix the records-only Forgejo's webhook access and
git-credential publication, or should `sv-startup-refuse` supply them? A: this task fixes
both, narrowly. `sv-startup-refuse` and `sv-world-one-process` wait on this task, so they
cannot supply what its acceptance needs. The task's `files` now add
`temper/crates/temper-service/src/forge/records.rs`; `src/forge.rs` stays unchanged, and
`setup/lifecycle.rs` may now change beyond `connected`'s last branch only as below.
- Webhook access: in `forge/records.rs`'s restored branch (a registered record with no
  setup entry), pass `records::access`'s `webhooks` as true for a Forgejo record whose rows
  hold `Item::WebhookSecret`; GitHub's call and every other caller stay as they are.
- Git credential: move the identity-publishing block of `setup/lifecycle.rs::connect_local`
  (the `GitToken` read and check, `translate::identity`, `values.identities.put`,
  `root::Event::Credential`) into one `pub(crate)` helper taking the forge's
  `settings::Identity` and its rows; `connect_local` calls it as before, and the restored
  Forgejo branch of `forge/records.rs` calls it with `crate::forges::identity(record)` (its
  number is the forge's) before `crate::forge::connect`, when its rows hold `GitToken`.
- Two cases in `src/tests/registry_setup.rs`: a registered Forgejo with no setup entry and a
  seeded `WebhookSecret` connects with the secret in its `Connect` access; with a seeded
  `GitToken` it publishes one `Credential` for identity <the forge's number> and the
  identities table holds it. Run the module as above, then the whole
  `temper-real-loop-world` default run.
- If `forced_overdue_observes_real_tasks_operations_and_dispatch` still stalls after both
  fixes, do not widen further: report its log and diagnosis as a question.
