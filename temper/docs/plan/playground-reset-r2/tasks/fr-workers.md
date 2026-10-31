+++
id = "fr-workers"
part = "forge-records"
kind = "feature"
needs = ["fr-registry"]
files = [
  "temper/crates/temper-service/src/workers.rs",
  "temper/crates/temper-service/src/worker_settings.rs",
  "temper/crates/temper-service/src/identities.rs",
  "temper/crates/temper-service/src/values.rs",
  "temper/crates/temper-service/src/translate.rs",
  "temper/crates/temper-service/src/assignment.rs",
  "temper/crates/temper-service/src/worker_route.rs",
  "temper/crates/temper-service/src/tests/registry_workers.rs",
  "temper/crates/temper-service/src/tests/worker*.rs",
]
effort = "high"
minutes = 60
+++

## Goal

A forge built from its record gets what a configured forge gets on the
workers' side (`temper/docs/design/draft/playground-reset.md, 3`): its
entry in the workers' forge list (checkout remotes, git program,
GitHub installation account) and its git identity's credential, so a
task on one of its repositories checks out and pushes as on a
configured forge. `crate::workers::add` (fr-registry's empty hook) does
it. The worker link's credential bound becomes a count of forges kept
(`forge.forges`), no longer the configured identities' count.

Acceptance, new tests in `temper-service`
(`src/tests/registry_workers.rs`):

- `a_made_forge_reaches_the_workers`: after `registry::made` for a
  Forgejo record whose bot token is in the secrets, an assignment on
  one of its repositories carries its checkout remote and its
  identity's credential, numbered as the forge.
- `the_credential_bound_is_the_forges_count`: the worker wire's
  `credentials` size is `limits.forge.forges`, whatever the seeds.
- The existing worker and assignment tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2 (the
  identity's number is the forge's) and 4 (limits as counts).
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/credentials.md`, the git identities.
- `tasks/fr-registry.md` (`Built`, `install`, the `add` hooks).

## Owns and may change

Owns, after `fr-registry`: the files above (`workers.rs`,
`worker_settings.rs`, `identities.rs`, `values.rs`, `translate.rs`,
`assignment.rs`, `worker_route.rs`). The other fr- consumer tasks run
at once and own `forge*`, `engine.rs`, `setup*`, `clients*`,
`family*`; `system.rs` is `fr-service`'s: if the credential bound's
source sits there, change only that expression and say so.

May change: the worker wire's limits literal sites the bound forces;
test fixtures; anything `notes/scope.md` lists as forced.

## Steps

1. `crate::workers::add(engine, built)`: push `built.forge` (as the
   `settings::Forge` the workers hold, `workers.rs:34`) into
   `Workers.forges` (a boxed slice today: make it a bounded list sized
   by `forge.forges` plus the seeds); add the identity
   `built.identity` to `values.identities` as `Values::fill` does for a
   configured one (`values.rs:132`), when its token is known; a
   records-only forge's bot token reaches it as today's records-only
   forges' does (read where `values.identities.put` is called after a
   setup host's grant; that path serves the added forge unchanged).
2. The credential bound (`workers.rs:515`, `credentials:
   settings.identities.len()`) becomes `limits.forge.forges` (at least
   the seeds' count).
3. Tests; `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-service`, then `-p temper-one-process-world -p
   temper-real-loop-world -p temper-system-world`; old forms.
4. Commit and push.

## Report

The standard report, plus: how a records-only forge's git token
reaches the identities table; any reader of `Workers.forges` that
assumed it never grows.
