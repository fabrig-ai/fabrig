+++
id = "fr-setup"
part = "forge-records"
kind = "feature"
needs = ["fr-registry", "fr-skein-setup", "fr-domain"]
files = [
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/**",
  "temper/crates/temper-service/src/tests/setup*.rs",
  "temper/crates/temper-service/src/tests/registry_setup.rs",
]
effort = "high"
minutes = 90
+++

## Goal

A forge built from its record gets what a configured forge gets from
the setup host (`temper/docs/design/draft/playground-reset.md, 3`): its
entry in skein's setup host (by `fr-skein-setup`'s `Request::Add`), its
local start state when it has a setup entry (owned, fake), and its
first repository's intent restored after a restart, so the host makes
the first repository at the next connect instead of losing it
(`temper/docs/design/protocol/setup.md, 6.7`, now kept;
`playground-reset.md, 3`, "The intent is kept").
`crate::setup::add` (fr-registry's empty hook) does the first two.

Acceptance, new tests in `temper-service`
(`src/tests/registry_setup.rs`), over the fake forge the setup tests
use (`src/tests/setup_create.rs`'s `Forge`):

- `a_made_owned_forge_is_set_up_by_the_host`: a SetUp with
  `Target::New(stock::owned(..))` under the box (the box link answering
  `started`) reconciles the new forge and makes its first repository,
  as a configured owned forge's SetUp does today
  (`setup_create.rs`'s cases are the model).
- `a_made_github_forge_serves_its_manifest`: `Connect { target:
  New(stock::github(..)), how: GitHubManifest }` answers a link served
  by the host for the new forge.
- `a_restart_before_the_first_repository_makes_it_at_the_next_connect`:
  a SetUp cut after reconcile and before Create, restarted over its
  store, makes the first repository with its names after the restart's
  reconcile, and the forge's row shows it made.
- The existing setup tests (`src/tests/setup*.rs`) pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.4.
- `temper/docs/design/protocol/setup.md`, 5.1 (the host), 6.5 to 6.7
  (owned start, the first repository).
- `skein/docs/design/forge-setup.md`, 6.3.
- `tasks/fr-registry.md` (`Built`, `install`, `setup_member`),
  `tasks/fr-skein-setup.md` (`Request::Add`), `tasks/fr-domain.md` (the
  intent kept in the record and restored as making).

## Owns and may change

Owns, after `fr-registry` (D4's `rm-service`, `pg-` and D3's `bx-serve`
work here next; `bx-serve` only through call sites): `src/setup.rs`,
`src/setup/**`. The other fr- consumer tasks run at once and own
`forge*`, `engine.rs`, `workers.rs`, `values.rs`, `clients*`,
`family*`.

May change: setup test fixtures; anything `notes/scope.md` lists as
forced.

## Steps

1. Factor `setup/settings.rs`'s `fixed` (`:8`) so one forge's
   `host::Forge` is made from one `settings::Forge` and its `Setup`
   (`fn host_forge(..) -> Result<Option<host::Forge>, Problems>`), and
   `local_starts` (`:253`) so one setup entry's local state is made
   alone (`fn local_start(..)`). `fixed` and `local_starts` call them for
   the seeded forges as today.
2. `crate::setup::add(engine, built)`: `host_forge` from `built`; when
   the host component runs, `host::Request::Add`; before it runs (its
   settings still pending, `Layer.pending`), push it into the pending
   settings' forges. Insert into `served`. When `built.setup` is some,
   its local state by `local_start` (a records-only entry: no
   handover). `host::Event::Added { refused: Some(..) }` goes to the
   diagnostics.
3. The intent after a restart: in `setup/lifecycle.rs`'s `restored`
   (`:824`), for each forge whose record has `first: Some(i)`
   (`engine.domain.forge_record(n)`), set its local `first` to
   `Intent { playground, label, org, repository }` from `i`, as
   `lifecycle::first` (`:1330`) does at a SetUp. The host's Create then
   runs after the restart's reconcile as it does after a SetUp's. A
   Create that finds the repository already made (a restart after
   Create, before adoption) counts as made: read the Create plan's
   outcome for an existing repository and keep its id; if skein's
   create refuses an existing repository, answer the intent's state as
   made by reading the repository instead, and say which.
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-service`, then `-p temper-one-process-world`; old forms.
5. Commit and push.

## Report

The standard report, plus: how a Create over an existing repository
answers; whether the pending host settings path was needed; what
`setup.md` 6.7 should now say (for its designer).
