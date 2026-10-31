+++
id = "fr-connections"
part = "forge-records"
kind = "feature"
needs = ["fr-registry", "fr-skein-connection"]
files = [
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/forge/records.rs",
  "temper/crates/temper-service/src/forge/tokens.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/tests/forge*.rs",
  "temper/crates/temper-service/src/tests/registry_connections.rs",
]
effort = "high"
minutes = 75
+++

## Goal

A forge built from its record gets what a configured forge gets at
start from the forge connection side
(`temper/docs/design/draft/playground-reset.md, 3`): its connection in
skein's forge component (by `fr-skein-connection`'s `Request::Add`),
its secrets loaded and its access handed from them, and, for GitHub,
its installation-token state. `crate::forge::add` (fr-registry's empty
hook) does it, at start for restored records and at runtime for a
forge just made.

Acceptance, new tests in `temper-service`
(`src/tests/registry_connections.rs`):

- `a_restored_record_connects_from_its_secrets`: a store holding a
  `Forge` record (a Forgejo, records-only, plaintext loopback to the
  test's fake forge) and its secrets (`temper-protocol-records` items:
  API token, user, webhook secret) starts with that forge connected:
  calls on one of its repositories are served by the fake forge.
- `a_made_forge_is_added_and_calls_wait_for_its_access`: after
  `registry::made` for a new Forgejo record, the component holds it
  (`Added { refused: None }`) and its calls are answered
  `NotConnected` until its access is handed (as a configured forge
  without access is today).
- `a_made_github_forge_holds_its_token_state`: a GitHub record made at
  runtime has its installation-token state, as a configured GitHub's.
- The existing forge tests pass unchanged (`src/tests/forge.rs` and
  the others `rg -l 'forge::' src/tests` names).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/protocol/credentials.md`, section 2 (the forge's
  secret items, numbered by forge).
- `temper/docs/design/domain/root.md`, 9.1 (public restore, then
  secrets) for the start order.
- `skein/docs/design/forge-connection.md`, 3.5 (access at runtime).
- `tasks/fr-registry.md` (`Built`, `install`, the `add` hooks) and
  `tasks/fr-skein-connection.md` (`Request::Add`, `Event::Added`).

## Owns and may change

Owns, after `fr-registry` (`fr-service` and D4's `rm-service` own them
next): `src/forge.rs`, `src/forge/records.rs`, `src/forge/tokens.rs`,
and in `src/engine.rs` the forge component's event arms (`Added`) and
the secret loader's construction. The other fr- consumer tasks run at
once with this one and own `workers.rs`, `values.rs`, `setup*`,
`clients*`, `family*`: call into them only through what exists.

May change: `src/tests/fixtures*.rs` (a helper that seeds a `Forge`
record and its secrets into a test store; if the test store cannot be
seeded before start, make the record by a `forge::Event::Seed` stepped
in the test and the secrets by the store's `Keep`, and say so);
anything `notes/scope.md` lists as forced.

## Steps

1. `crate::forge::add(engine, built)`:
   - the connection: `skein_forge_connection::settings::Forge` from
     `built.forge` as `forge::settings` (`forge.rs:42`) makes one today
     (address from `built.forge.api`: an `Addr::Ip` as given; GitHub's
     from the machine's GitHub endpoint (`Machine.github`, as
     fr-registry keeps the machine for `read`; add
     `Registry::machine(&self) -> &Machine` in `registry.rs` if it has
     no accessor); otherwise `HELD_ADDRESS`), with no
     access; send `Request::Add`. `Event::Added { refused: Some(..) }`
     goes to the diagnostics as a forge that fails its checks.
   - the secrets: the loader's `Rows` takes the number
     (`Rows::configure(number)`, new): before the load ends, its rows
     load with the others; after it (a forge made at runtime), there is
     nothing in the store for it yet.
   - for GitHub, the token state `forge::tokens` holds per GitHub forge
     (`tokens.rs:36`), added for this forge (`Tokens::add`, new).
2. The loader's `Connect` phase hands each loaded forge's access, as it
   does for configured records-only forges; a forge added after it
   gets its access from the setup host's connect as today (nothing to
   do here).
3. `forge::settings` and the start path stay for seeded forges.
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-service`, then `-p temper-one-process-world -p
   temper-real-loop-world`; `python3 -I -B scripts/old_forms.py` from
   `temper/`.
5. Commit and push.

## Report

The standard report, plus: how the GitHub endpoint reaches `add`;
whether the loader's phases needed reordering; what a held address
means for a restored record.
