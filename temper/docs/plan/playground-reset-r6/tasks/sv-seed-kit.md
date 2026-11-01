+++
id = "sv-seed-kit"
part = "fs"
kind = "feature"
needs = ["fe-records"]
files = ["temper/testing/temper-github-app/**"]
effort = "medium"
minutes = 60
+++

## Goal

The worlds that drive serve in process need a forge without the document: a kit that keeps
a forge's record (`notes/contracts.md`, T1, T2) and its credential records in a
deployment's store, through jig's public store API, as `temper_github_app::seed` keeps a
GitHub App's records today. `sv-world-one-process` and `sv-world-real-loop` use it.

Acceptance:
- `temper_github_app::seed_forges(data: &Path, forges: &[ForgeRecord], records: &[(u64,
  Box<[u8]>)]) -> Result<(), String>`: one commit putting each record's `Forge` row (key
  `temper_protocol_store::keys::encode(&forge::Key::Forge(n), u32::MAX)`, value
  `temper_protocol_store::records::encode(&forge::Stored::Forge(record))`) and one
  `ForgeNumbers { next }` row past the highest number, then the secrets as `seed` keeps
  them; it returns once all is durable and every file is closed. It works where `seed`
  works (a store an earlier start made), and on an empty private data directory when
  jig's `Open` makes the deployment there (the report says whether it does: the worlds
  then seed before their first start, or after an empty one).
- `temper_github_app::forgejo_records(forge: u16, token: &[u8], user: u64, webhook_secret:
  Option<&[u8]>, git_token: &[u8], origin: SocketAddr) -> Box<[(u64, Box<[u8]>)]>`: the
  records a connected managed Forgejo keeps (`ApiToken`, `User`, `WebhookSecret` when given,
  `GitToken`, `Origin` of kind Forgejo at `origin`), encoded as `records` encodes the App's.
- `temper_github_app::forgejo(number: u16, api: SocketAddr, git: &[u8]) -> ForgeRecord`
  (Forgejo, `Mode::Connected`, plaintext, bot `temper`/`Temper`/`temper@localhost`, setup
  entry none) and `github(number: u16, api: SocketAddr, server_name: &[u8], trust:
  Option<&[u8]>) -> ForgeRecord` (`ForgeRecord::github_com()` with those, `Mode::Connected`).
- A test proves a seeded store restores: after `seed_forges`, a load of the application
  range (the kit's own `store::run` with `load`, extended to application rows) reads the
  rows back and `temper_protocol_store::records::row` decodes each to the record put.

## Inputs

- `temper/testing/temper-github-app/src/lib.rs` and `src/store.rs` (the precedent).
- `temper/docs/plan/playground-reset-r6/notes/fe-surface.md`, 4 and 6 (rows, keys).
- `temper/docs/design/protocol/credentials.md`, 2.1 (the records).
- `temper/docs/plan/playground-reset-r6/notes/common.md`.

## Owns and may change

- Owns: `temper/testing/temper-github-app/` (package `temper-github-app`).
- Pre-approved option: if jig's store refuses an application `Put` outside a root's commit
  numbering, use the commit number the deployment's header gives next (read it as the
  store's own open does) and say so.

## Steps

1. `src/forges.rs` (`pub use` from `lib.rs`): the three builders and `seed_forges`.
2. `src/store.rs`: the commit of application puts; the load of the application range for
   the test.
3. Test `the_seeded_forges_are_what_temper_restores` in `src/lib.rs`'s tests.
4. `heavy cargo nextest run -p temper-github-app -E 'test(=tests::the_seeded_forges_are_what_temper_restores)'`;
   `heavy cargo nextest run -p temper-github-app`; `cargo fmt`; `heavy cargo clippy -p
   temper-github-app --all-targets`; old forms. Commit and push.

## Report

The commit numbering used, and whether an empty data directory can be seeded.
