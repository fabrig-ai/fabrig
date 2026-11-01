+++
id = "sv-world-one-process"
part = "fs"
kind = "migration"
needs = ["sv-seed-kit", "sv-registry-setup", "sv-world-real-loop"]
files = ["temper/tests/one-process/**", "temper/crates/temper-service/src/**"]
effort = "high"
minutes = 110
+++

## Goal

The one-process world's deployments name no forge (`notes/contracts.md`, T4;
`notes/sv-progress.md`, request 1 (c)): its documents carry no `application.forges`,
`identities` or `setup`, and the forge each story ran on is kept as a record with its
credential records in the deployment's store (`sv-seed-kit`), so every story runs as before
on the records path.

Acceptance:
- `temper/tests/one-process/src/fixtures.rs` (`configuration()` and `files()`),
  `src/stories.rs` (its forge and identity mutation), `src/lower/github.rs`,
  `src/restart_control.rs`: no `application.forges`, `identities` or `setup` member, and no
  `secrets/forge*` file; `application.machine` only where a story needs a machine
  template (none expected).
- Each deployment's forge is seeded with `temper_github_app::seed_forges`: the fake Forgejo
  of `fixtures::FORGE` as `forgejo(1, FORGE, b"http://FORGE")` with `forgejo_records(1,
  <today's api token>, 1, Some(HOOK_SECRET), <today's git token>, FORGE)`; the lowered
  GitHub stories' forge as `github(1, FORGE, ..)` (plaintext as today: a GitHub record whose
  api is an IP literal with `ApiTransport::Plaintext`, so no resolve is needed) with its
  App's records as today. The world's worker identity (`login` `worker`, author
  `Worker`/`worker@example.invalid`) is the record's bot.
- `restart_control.rs` reads the forge's connection rows from what the engine and serve
  now hold (the registry, `engine.workers.forges`), not from `Named.forges`/`identities`.
- Every test of `temper-one-process-world` (default profile) passes; a test that asserted a
  document member that no longer exists asserts the record's equivalent.

## Inputs

- `temper/tests/one-process/` (its `README` or module docs), `temper/docs/design/domain/testing.md`
  for its stories.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md`, T1, T4.
- `temper/docs/plan/playground-reset-r6/tasks/sv-seed-kit.md` and its report (when to seed:
  before the first start or after an empty one).
- `temper/docs/plan/playground-reset-r6/tasks/sv-registry.md` (how a seeded Forgejo record
  connects at start).

## Owns and may change

- Owns: `temper/tests/one-process/` (package `temper-one-process-world`).
- Pre-approved options:
  - Seeding point: before the world's first start if `sv-seed-kit` says an empty data
    directory takes it; else the fixture runs one empty start first (as real-loop's GitHub
    seed does) and seeds after it. Default as the kit's report says.
  - A story whose trace counts a start's steps may gain the empty start's; adjust its
    expected counts only for that, and list each.
  - If a story needs the document's forge in a way the records cannot give (a refusal of a
    document member), it stays on the old document until `sv-startup-refuse` and is listed
    under `## Left`.

## Steps

1. Fixtures, stories, lower/github, restart control as above.
2. `heavy cargo nextest run -p temper-one-process-world` (whole, default profile); read
   each failure's log before a rerun.
3. `cargo fmt`; `heavy cargo clippy -p temper-one-process-world --all-targets`; old forms.
   Commit and push.

## Report

- The seeding point taken; every story whose expectations changed and why.

## Answers

2026-11-01. Q: the suite is red on service defects of the records path (`connected`'s
writer 0, the GitHub workers held, the held-change restart); route them to the service's
owners, or widen this task? A: both, by defect. Your pushed branch stays the start of the
next run; keep the empty-start seeding point you took.
- **`setup::lifecycle::connected` (writer 0, `same forge writer`):** not yours.
  `sv-world-real-loop` was answered the same fix (its spec's `## Answers`), and this task
  now needs it, so your next run starts from a trunk that has it. Do not edit
  `setup/lifecycle.rs`.
- **The GitHub and M15 stories held with no agents:** yours. `workers::limits` sizes
  `credentials` by `settings.identities.len()`, zero once the document names no forge.
  Size it by the forges kept at most, `settings.limits.forge_component.forges` (the draft,
  section 3: limits are counts), and add a case to `src/tests/worker.rs` that a settings
  with no document identity still gives `credentials` that count. Then confirm the four
  stories; if one stays red on another service reader the document fed, the next bullet
  applies to it.
- **The held-change restart, and any story still red after these:** find its cause. When
  it is a temper-service reader of the document's forges, identities or setup that the
  records leave empty or zero, fix that reader to read the registry or the records, with a
  lower-tier test in `src/tests/`, and list each under `## Changes`. Touch no other
  service file and no service behaviour beyond such a reader; `sv-startup-refuse` deletes
  the document halves later, so make no deletion here. A cause that is anything else
  (an engine or domain fault, a wider change) goes in your report as a `Question:` with
  its diagnosis.
- `Cargo.lock`: leave it to the trunk as you did.

