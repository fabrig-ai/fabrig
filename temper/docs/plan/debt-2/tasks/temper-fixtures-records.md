+++
id = "temper-fixtures-records"
part = "temper-forge-records"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/fixtures/**",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_adoption.rs",
  "temper/crates/temper-service/src/tests/setup_scopes.rs",
  "temper/crates/temper-service/src/tests/setup_reachable.rs",
  "temper/crates/temper-service/src/tests/forge.rs",
  "temper/crates/temper-service/src/tests/forge_runtime.rs",
  "temper/crates/temper-service/src/tests/system.rs",
]
effort = "high"
minutes = 130
+++

# temper D005, step 1: temper-service's fixtures connect their forges by records

## Goal

Every in-process fixture of `temper-service` gives its forges their access by records, as
the shipped path does (`temper/docs/design/protocol/credentials.md`, 2.1 and 5;
`temper-service/src/forge/records.rs`, `connect`), and none by `settings::Forge.api_token`
or `webhook_secret`. Product code does not change: the next task,
`temper-file-credentials-gone`, deletes the file-credential path once nothing fills it. This
is the first half of `temper/docs/development/debt/D005-forgejo-file-credentials.md`.

Acceptance:

- `fixtures::forges()` and every other fixture that builds a `settings::Forge` leave
  `api_token` and `webhook_secret` `None`; `rg -n 'api_token|webhook_secret'
  temper/crates/temper-service/src/tests` lists only: the `None`s of literals, the tests
  whose own subject is the file-credential path (a refusal naming `api_token`, `app_key`,
  `webhook_secret` or `user`; `tests/forge.rs`'s `HEAD_PROBLEMS`), which the next task deletes
  with that path, and the exemptions below. Report the list.
- A fixture forge that connected by file access connects by records seeded at start: the
  forge's rows (`Origin`, `User`, `ApiToken`, `GitToken`, and `WebhookSecret` where its test
  needs one) in `engine.forge_records.rows` and its values in the secrets store, from fixture
  constants (one value per forge where two forges must differ, as `fixtures.rs` ~473-477
  makes them today), never from `source.api_token`.
- A fixture forge that started without access (today `api_token = None`: the setup tests'
  forges the setup host connects, the GitHub forges) starts without records, as today,
  through one fixture knob that says so (the existing `ConfiguredEngine.records_only`, or the
  same knob renamed to what it means).
- Every `temper-service` test passes, no assertion changed but those the Report lists under
  the rule below; `heavy cargo nextest run -p temper-service --lib` counts above zero.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md`.
- `temper/AGENTS.md`.
- `temper/docs/development/debt/D005-forgejo-file-credentials.md`, whole.
- `temper/docs/design/protocol/credentials.md`, 2.1 and 5 (a forge's access is its records');
  `temper/docs/design/protocol/configuration.md`, 6.
- The shipped path: `temper-service/src/forge/records.rs` (`advance`, `connect`),
  `temper-service/src/forge.rs` (`connect`), `temper-service/src/forges.rs`
  (`record_settings`).
- The fixtures' records tools today: `tests/fixtures.rs` `seed_forge_rows` (~1453) and
  `configured_prepared` (~1309-1450, its secrets store `Keep`s ~1423-1446), `ConfiguredEngine`
  (~1257); `temper/tests/real-loop/src/fixtures/kept.rs` (`seed_forgejo`, `seed_record`,
  `forgejo_record`), which the process worlds use, for the rows' shapes.

## Owns and may change

- `temper-service/src/tests/fixtures.rs` and `tests/fixtures/**` (`forges()`, the two-forge
  tokens, `start_with_worker`, `started`, `ready`, `started_with_two_forges`,
  `configured_prepared`, `seed_forge_rows`, `fixtures/forge.rs` ~240 which takes
  `.access.as_mut()`: build the `Access` it needs directly; `fixtures/startup.rs`'s no-op
  `"webhook_secret":null` replace and `secrets/forge-1.*` files may go if nothing reads them).
- The tests that set a forge's `api_token` or `webhook_secret` for its fixture, or read
  `settings().access` filled from them: `tests/setup.rs` (~74, 469, 626, 671, 692, 788,
  1078, 1917), `setup_adoption.rs` (~14, 40, 42: a `WebhookSecret` row in place of `HOOK`),
  `setup_scopes.rs` (~100: build the `Access` directly), `setup_reachable.rs` (~261),
  `forge.rs` (~617, 635, 637, 805), `forge_runtime.rs` (~95: the assertion that
  `api_token` is `None` goes with the field's use), `system.rs` (~446).

Not yours: product code (`temper-service/src/**` outside `src/tests/`); `tests/limits.rs`
(another draft is changing it; its GitHub `api_token` line goes with the field in the next
task); `tests/startup.rs`; `tests/setup_github_render.rs` (`temper-unlinked-callers`').

Exemptions (they name the same words for something else, and stay): `Item::ApiToken` and the
records' rows; the owned forge's handover-directory file names (`api_token` in
`temper/crates/temper/tests/**`, `temper-forge-setup`); `tests/startup.rs` ~500 (a synthetic
`forges[0].user` path to test printing) and ~948-954 (the document refusing
`application.forges`).

Pre-approved:

- The fixture may wait, inside `started`, `ready` and their kin, until its forges' records
  have loaded and connected, so that each test begins where it began before; this is the
  default.
- A test whose own subject is the start (it counts the engine's steps or requests from the
  first) may take the new sequence when the only change is the records' load and connect:
  keep its other assertions, and list it in the report with the sequence before and after.
  Any other moved assertion is a question: stop and ask with the test and both values.
- If a forge cannot connect by records in a fixture without a product change, stop and ask,
  naming the forge's fixture and what the records path lacks.

## Steps

1. Sketch (the names are yours to keep or improve):

   ```rust
   // tests/fixtures.rs
   /// The API token each fixture forge's records hold (credentials.md, 5).
   pub(crate) const FORGE_TOKEN: &[u8] = b"...";       // today's default token's bytes

   fn seed_forge_rows(service: &mut crate::Service, forge: u16, token: &[u8]) { /* ... */ }
   ```

   Read the shipped path first, then today's `records_only` forges in `tests/setup.rs` ~787
   and `setup_adoption.rs` ~39, which already connect by records: generalize them.
2. Narrow tests, one at a time, each count above zero: `heavy cargo nextest run -p
   temper-service --lib -E 'test(/setup/)'` while you work on the setup tests, then `heavy
   cargo nextest run -p temper-service --lib`.
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`; `heavy cargo clippy -p
   temper-service --all-targets -- -D warnings`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commits: the fixtures' records; the tests' forges. Pushed.

## Report

As the implementer's text says, and also: the acceptance search's output; how a fixture now
says a forge starts without records; every test listed under the start rule, with its
sequences; the `temper-service --lib` count before and after.
