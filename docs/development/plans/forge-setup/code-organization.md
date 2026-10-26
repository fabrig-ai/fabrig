# forge-setup: code organization

2026-10-16. Crates, names, shared interfaces, fixtures, the plan's
"Don't" list, and its size limits and count. Three crates are added
(`setup.md`, section 11); none is renamed or removed.

## 1. Crates

### New

| Crate | Kind | What it holds | Sessions |
|---|---|---|---|
| `crates/temper-forge-setup` | step code, workspace lints (the subset) | `Setup`, its checks and `setup.` limits; `Step`, `steps`; app.ini, the runner's configuration, `state`, the layout; the preparation's machine and the processes' specs; reconcile's machine; `temper forge run`'s and the upgrade's steps. Depends on temper-forge-forgejo, jig-protocol-config, skein-lib, skein-json. Names no host | 07, 08, 09, 15 |
| `crates/temper-forge-setup-shell` | ordinary Rust, its own lints as `crates/temper/Cargo.toml`'s | the hosts of prepare, run and reconcile, on one skein-io loop: the preparation's commands on skein-io's processes (bounded, masked output), reconcile's HTTP on skein-http; the one std module of the lock and the exec alone (`src/process.rs`; designer's answer 32) | 08, 10 |
| `crates/temper-box` | step code, workspace lints; its host written within the subset (designer's answer 22) | the box's machine, `box.` limits, its host on skein-io's processes, the boundary test of setup.md 7.3 | 12 |

### Changed

| Crate | What changes | Sessions |
|---|---|---|
| skein `crates/skein-lib` | `sha256::{Sha256, digest, Hmac, equal}` | skein 01 |
| skein `crates/skein-io`, `crates/skein-oauth`, `crates/skein-ui-bundle` | their private SHA-256 and constant-time comparison moved onto skein-lib's; every public item kept | skein 01 |
| `crates/temper-forge-forgejo` | `Method::Put`; booleans in the body decoder; the setup's documents (admin users, the admin's token routes, orgs, teams, members, org hooks) and the deliveries' documents, both sides; the pin (`src/pin.rs`) | 01, 02, 03, 04 |
| `testing/temper-throwaway-forgejo` | re-exports the pin; Forgejo's command line on a work path; `[webhook] ALLOWED_HOST_LIST`; `attach` | 03, 04, 08 |
| `tools/forgejo-conformance` | the facts `setup-…`, `cli-…`, `runner-…`, `delivery-…`; recorded deliveries in `deliveries/`, beside `transcripts/` | 01, 02, 03, 04, 05 |
| `testing/temper-fake-forge-protocol` | the setup's routes over a `Store` beside its routes (designer's answer 26); signed deliveries rendered for the world to write; the route tally's capacity | 01, 02, 05, 09 |
| `testing/temper-fake-forge-domain` | hooks as Forgejo's: a job's end, the item's body, the run's payload, a "checking" phase | 06 |
| `crates/temper-engine-protocol-forge` | the hooks' listener: connections, the head bound, the streamed HMAC, the answers, the hint | 05 |
| `crates/temper-engine-domain-forge`, `-forge-client` | `settle`: the window in the change's procedure; `settle` and its check in the client's limits | 06 |
| `crates/temper-service` | `application.webhooks`, `forges[].webhook_secret`, `forges[].user` as a file, `application.setup` handed on undecoded with its overrides, the hook limits in the profile | 05, 07 |
| `crates/temper` | `temper forge prepare|run|reconcile`, `temper box` | 08, 10, 13 |
| `crates/temper/tests` (the process tier) | Forgejo by prepare and run; provisioned by reconcile; CI; webhooks; the box's tests; the owned example | 08, 10, 11, 13, 14 |
| `tests/forge-setup/reconcile` (new world) | reconcile's machine against the fake forge | 09 |
| `tests/engine/forge`, `tests/engine/protocol-forge`, `tests/codecs` | settle's stories; the hints' joint story; the new transcripts and deliveries replayed | 01, 02, 04, 05, 06 |
| `docs/examples/owned.json`, `docs/examples/owned/`, `docs/guide/box.md` | the owned example, its non-secret files, the guide (decisions.md, 14) | 14 |

## 2. Names

- **The pin:** `temper_forge_forgejo::pin::{VERSION, RUNNER_VERSION,
  names_forgejo, names_runner}`; the throwaway re-exports them. Nothing
  else names a Forgejo or runner version (decisions.md, 4).
- **Requests,** flat beside the existing ones: `UserOf`, `CreateUser`,
  `Tokens`, `CreateToken`, `DeleteToken` (on the admin's routes,
  designer's answer 14; a token is deleted by its listed id), the org, team, member and hook
  requests of temper 02. A user read for the setup decodes as
  `Kind::Setup(SetupKind::Account)`; embedded users keep `id` and
  `login`.
- **Facts** by group (decisions.md, 11): `setup-…`, `cli-…`,
  `runner-…`, `delivery-…`.
- **The bot's marker** `temper:bot`, in its full name, set by the
  request that makes it (designer's answer 15).
- **The listener's path** for a forge's deliveries, `SUBSCRIBED` (the
  events) and the two header names: `temper_forge_forgejo::delivery`.
- **Tests** are named for the behaviour, about eight words
  (development.md, 5.6).

## 3. Shared interfaces and their owners

Each is defined by one increment; the others use it by name, and never
redefine it.

| Interface | Owner |
|---|---|
| `Method::Put`, booleans in `BodyDecoder`, `User`'s `is_admin` and `full_name`, `Kind::Setup` | temper 01.1 |
| the conformance's setup recorder (`setup::record`, `setup::facts`), the fake's route capacity (`ROUTES`) | temper 01.2 |
| the token documents and requests on the admin's routes | temper 01.3 |
| the org, team and member documents | temper 02.1 |
| the org hook documents | temper 02.2 |
| `pin::{VERSION, RUNNER_VERSION, names_forgejo, names_runner}` | temper 03.1 |
| the throwaway's work path and command line (`src/cli.rs`) | temper 03.2 |
| the deliveries' documents both sides; the listener's path, `SUBSCRIBED`, the header names | temper 04.2 |
| the fake's signed deliveries | temper 05.1 |
| the component's hook limits and the listener | temper 05.2 |
| `crates/temper/tests/support/webhooks.rs` | temper 05.3 |
| `settle` (the window and the client's limit) | temper 06.2 |
| `Setup`, its checks, `setup.` limits, `Step`, `steps` | temper 07.1 |
| the layout (`Place`), app.ini, the runner's configuration, `state` | temper 07.2 |
| `forges[].user` as a file | temper 07.3 |
| the preparation's machine, the specs (`spec::Command`, `spec::Which`) | temper 07.4 |
| the lock-and-exec module (`temper-forge-setup-shell/src/process.rs`) and its boundary test; `crates/temper/src/forge.rs`; `crates/temper/tests/support/owned.rs`, its first functions | temper 08.1 |
| `Instance::attach` | temper 08.3 |
| reconcile's machine (`Need::Send { request, token }`) | temper 09.1 |
| the fake's setup `Store` | temper 09.2 |
| reconcile's host and exchange; `support::owned::{run, start, reconcile}`, `Running::into_child` | temper 10.1 |
| the process tier's owned configuration, `support::owned::tier_configuration(directory) -> SocketAddr` | temper 10.2 |
| `support::owned::binaries()` | temper 10.2 |
| `support::owned::{application, example_document, lay}` | temper 14.1; an increment of 11 landed before it builds its own members, and 14.1 converts them |
| temper-box's machine and `box.` limits | temper 12.1 |

## 4. Fixtures

Each crate's fixtures are one module, made by the first session that
tests the crate and extended by the later ones, never redefined.

- `crates/temper-forge-setup/src/tests/fixtures.rs` (temper 07.1;
  extended by 07.2, 07.4, 08, 09, 15).
- `crates/temper-forge-setup-shell/tests/support/mod.rs` (temper 08.1:
  its tests need `CARGO_TARGET_TMPDIR`, which integration tests alone
  have; extended by 08.2, 10.1).
- `crates/temper-box/src/tests/fixtures.rs` (temper 12.1; extended by
  12.2).
- `tests/forge-setup/reconcile/src/` (the world's own, temper 09.2).
- The existing ones grow: `crates/temper-forge-forgejo/src/tests.rs`,
  `tests/codecs/src/forgejo.rs`'s cases, `crates/temper/tests/support/`,
  `crates/temper-engine-protocol-forge/src/tests/fixtures.rs`,
  `tests/engine/protocol-forge/src/fixtures.rs`, `tests/engine/forge/`.

Limits in tests come from the shipped profile's builders
(`crates/temper-service/src/limits.rs` and its peers); a test that needs
another value names it beside the profile's, with the reason.

## 5. The plan's "Don't" list

No increment:

- adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment;
- adds a field or a predicate that its counterpart across a boundary
  lacks: the fake's server and the clients decode and encode the same
  documents, from `temper-forge-forgejo`;
- writes a test that checks several behaviours under one name;
- adds a lint exception in tests;
- writes an expected value computed by the code under test;
- writes the words the pulse forbids (README, "Pulse");
- edits a transcript, a recorded delivery or `facts.txt` by hand, or
  inserts a capture before an existing one;
- names a Forgejo or runner version in code, but the pin's;
- puts `std::process` or a file lock in any module of `crates/*/src`
  but `temper-forge-setup-shell/src/process.rs` (test files aside;
  `std::process::ExitCode` in the binary's `main` allowed), or runs a
  command through std's `Command::output`;
- puts a secret in a command's argument, a log line or a message;
- signs in as the bot, or keeps its password (designer's answer 14);
- makes the fake forge better than Forgejo: the fake answers what the
  conformance's facts record (`forge.md`, section 9);
- moves temper's skein but in temper 05.1;
- writes a document or request literal in a third test of a crate without
  a constructor for it in that crate's `tests/fixtures.rs` (steer of
  the check of 2026-10-16T23:49Z).

## 6. Size limits and the count

- **Aims:** a file of at most 800 lines and a test function of at most
  40; a miss is noted in the landing's report, and the landing stands.
- **A file over 800 lines at the base grows by at most 40 lines** in one
  increment (`git diff --numstat origin/main...HEAD -- <file>`, added
  minus removed); code past that goes in a new module beside it. They
  include `crates/temper-engine-domain-forge/src/domain.rs` (3463),
  `testing/temper-fake-forge-domain/src/tests.rs` (2150),
  `tests/engine/forge/tests/simulation.rs` (1830),
  `crates/temper-engine-domain-forge-client/src/keep.rs` (1377),
  `src/outbox.rs` (1079), `src/api.rs` (850); in skein,
  `crates/skein-io/src/kernel.rs` (1537), `stream.rs` (1741),
  `file_layer.rs` (1274), `file_layer/store.rs` (906),
  `crates/skein-oauth/src/machine.rs` (832).
- **An increment's size** is its hand-written production lines, counted
  once before it lands, in its worktree. In temper:

  ```sh
  git diff -M --numstat origin/main...HEAD -- crates testing tools \
    ':!*/tests.rs' ':!*/tests/*' ':!*/generated/*' ':!*/golden/*' \
    ':!*/schema/*' ':!tools/*/transcripts/*' ':!tools/*/deliveries/*' \
    ':!tools/*/facts.txt' | awk '{a+=$1} END {print a}'
  ```

  In skein, the same with `crates` alone. An increment whose Size is
  the process tier's support (temper 08.3, 10.2, 11.1, 11.2, 13.2,
  14.1) counts those files instead:

  ```sh
  git diff -M --numstat origin/main...HEAD -- crates/temper/tests/support \
    | awk '{a+=$1} END {print a}'
  ```

  Each landing's report gives the number.
- **The tolerance** is the implementation rules; a Size marked "(codec)"
  takes its codec tolerance.
