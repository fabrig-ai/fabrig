+++
id = "setup-project-goes"
part = "wizard"
kind = "migration"
needs = ["welcome-ready", "no-repository", "no-seeded-project"]
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/src/lib.rs",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
  "temper/crates/temper-client-protocol-wire/src/tests/forgejo_admin.rs",
  "temper/crates/temper-client-protocol-wire/golden/",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/v11.rs",
  "temper/crates/temper-protocol-store/src/records/forges.rs",
  "temper/crates/temper-protocol-store/src/tests/records.rs",
  "temper/crates/temper-protocol-store/src/tests/fixtures.rs",
  "temper/crates/temper-protocol-store/src/tests/sizes.rs",
  "temper/crates/temper-protocol-store/golden/v11/",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/make.rs",
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/make.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/records.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/removal.rs",
  "temper/crates/temper-engine-domain/src/tests/setup.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_admin.rs",
  "temper/crates/temper-service/src/tests/setup_create.rs",
  "temper/crates/temper-service/src/tests/system.rs",
  "temper/crates/temper/src/host.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/forge/admin.rs",
  "temper/crates/temper-client-domain/src/forge/words.rs",
  "temper/crates/temper-client-domain/src/tests/forges.rs",
  "temper/crates/temper-client-domain/src/tests/connect.rs",
  "temper/crates/temper-client-domain/src/tests/admin.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/forges/connection.rs",
  "temper/crates/temper-client-tools/src/tests/forges.rs",
  "temper/crates/temper-client-tools/src/tests/forge_connection.rs",
  "temper/crates/temper-client-tools/src/tests/adopt.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs",
  "temper/tests/codecs/tests/suite/client_wire.rs",
  "temper/tests/codecs/tests/suite/fuzzy/store.rs",
  "temper/tests/engine/domain/tests/suite/first_repository.rs",
  "temper/tests/engine/forge/src/world.rs",
  "temper/tests/engine/forge/tests/suite/simulation.rs",
  "temper/tests/client/tests/suite/first_repository.rs",
  "temper/tests/client/tests/suite/forges.rs",
  "temper/tests/client/tests/suite/forges_edges.rs",
  "temper/tests/client/tests/suite/connect_forge.rs",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/forge.status.md",
  "temper/docs/design/protocol/store.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "medium"
minutes = 90
+++

# The set up's request names no project

## Goal

K3 in place: nothing carries the first repository's project any more. The client wire's `SetUp`
is `{ forge, org, repository, playground }` and `FirstAsk` `{ org, repository, label }`; the
row's `FirstRepository` is `{ org, repository, state, label, playground }`; `SetUpProject`,
`ExistingProject`, `NewProject` and `SetUpRefusal::new_project` go. The store's `FirstIntent` is
`{ playground, org, repository, label }`; `FirstProject`, `ExistingFirst` and `NewFirst` go. Both
schemas change in place, at their current versions (`CLIENT_VERSION` stays 11, the store stays
v11). `temper ctl forge setup` and `forge connect … --first` keep `--project`, held in the
call's own run (`temper/docs/design/client/tools.md`, 5.5).

Acceptance:

- The two schemas as above, each other field and variant in its place; generated code and goldens
  regenerated; the drift test passes.
- No `SetUpProject` (or a renamed twin) in the engine's forge domain, the service, the store, the
  client domain, ctl or their tests; the engine's `SetUpRefusal::NewProject` and its words in
  `crates/temper/src/host.rs` and `temper-client-domain`'s `forge/words.rs` go.
- `forge setup` and `forge connect … --first` behave as today from the outside: the same
  adoption into `--project` or a new project named after the repository, under
  `first-repository-F`, printing the same lines, but for `forge list`'s first repository, which
  names no project ("ORG/NAME made", and its JSON `first` with no `project`).
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`; `notes/contracts.md`, K3.
- `temper/docs/design/client/README.md`, 4.6: `SetUp`, its refusals, the row's `first`; section
  8's table, `ForgejoAdmin { first? }` and `FirstAsk`.
- `temper/docs/design/protocol/setup.md`, 6.7; `temper/docs/design/protocol/forge.md`, 2.1 (the
  intent); `temper/docs/design/protocol/store.md`, section 2 (family 25).
- `temper/docs/design/client/tools.md`, 5.5: `forge setup` and `forge connect … forgejo-admin`.

## Owns and may change

- **The wire** (`temper-client-protocol-wire`): the schema's lines 18-25, 39 and 65 as K3; its
  regeneration (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test
  suite -E 'test(drift::)'`); `lib.rs`'s bounds and checks for the fields that went
  (`set_up`'s and `first_ask`'s project bounds, if any); its tests and goldens: the goldens of
  removed types (`enum_set_up_project_*`, `enum_set_up_refusal_new_project_*`, `record_existing_project_*`,
  `record_new_project_*`)
  are deleted, the others regenerated on the tests that own them; read `git diff --stat` and keep
  only what this moves.
- **The store** (`temper-protocol-store`): the schema's lines 271-274 as K3; its regeneration;
  `records/forges.rs` (lines ~60 and ~129) maps the intent with no project; its tests and goldens
  as the wire's (`enum_first_project_*`, `record_existing_first_*`, `record_new_first_*` deleted,
  `record_first_intent_*`, `record_forge_*` regenerated).
- **The engine** (`temper-engine-domain-forge`): `boundary.rs`'s `Ask::SetUp`, `FirstAsk`,
  `FirstIntent` and `First` lose `project`, and `SetUpProject` and `SetUpRefusal::NewProject`
  go; `make.rs`'s check of a new project's name (line ~80) goes, the org's and repository's
  checks unchanged; `lib.rs`'s held bytes (line ~322) count no name; `connect.rs` (lines ~265,
  ~537) carries `first` with no project. `temper-engine-domain`'s `tests/setup.rs` follows.
- **The service** (`temper-service/src/family.rs`): the mappings at lines ~68, ~107, ~668 and
  ~880 lose the project; its tests: `tests/family.rs`'
  `setup_projects_and_first_repository_states_translate_without_losing_fields` becomes
  `setup_and_first_repository_states_translate_without_losing_fields`, and
  `new_project_and_invalid_setup_refusals_translate_to_the_wire` becomes
  `invalid_setup_refusals_translate_to_the_wire`; `setup.rs`, `setup_admin.rs`,
  `setup_create.rs`, `system.rs` drop the field from what they build.
- **`crates/temper/src/host.rs`** (line ~347): the `NewProject` arm goes.
- **The client domain**: `forge/forges.rs`' `SetUpProject`, `SetUp.project`, `First.project`
  and `SetUpRefusal::NewProject`; `forge/family.rs` (lines ~176, ~368) encode and decode with no
  project; `forge/admin.rs`' `FirstAsk` with no project; `forge/words.rs`' `first_repository`
  says "ORG/NAME made" and the `NewProject` refusal's words go. Their tests follow. In
  `temper-client-view`, `forge/connect.rs`, only arms and imports that named `SetUpProject` or
  `SetupField` if `welcome-ready` left any: nothing drawn changes.
- **ctl** (`temper-client-tools`): `forges.rs`' `Intent.project` is the call's own, from its
  `--project` (or a new project named after the repository, `widget` with `--playground`), set
  when the call starts or when a restored request (`restore`, line ~594) is taken up by the call
  that names its key, never decoded from the request; the match of a kept `SetUp` to a row
  (line ~1492) compares org, repository and playground alone; `first_json` prints no `project`;
  the `NewProject` refusal's word (line ~1082) goes; `forges/connection.rs` (lines ~150-181)
  builds `FirstAsk` with no project and keeps PROJECT for the continuation. Tests:
  `tests/forges.rs` (line ~678's `"first":{…,"project":{"existing":1}` becomes the JSON with no
  project), `tests/forge_connection.rs`, `tests/adopt.rs`. Pre-approved: if a restored `SetUp`
  is ever taken up by a call with no `--project` of its own, it adopts into a new project named
  after the repository, as a fresh call would; say so in the report.
- **Fixtures and worlds** that build the removed fields, the field dropped and nothing else:
  `temper-client-shell-cli/src/tests/fixtures/setup.rs` (line ~74),
  `temper/tests/codecs/tests/suite/{client_wire.rs, fuzzy/store.rs}`,
  `temper/tests/engine/domain/tests/suite/first_repository.rs`,
  `temper/tests/engine/forge/src/world.rs` and `tests/suite/simulation.rs`, and what
  `welcome-ready` left in `temper/tests/client/tests/suite/{first_repository,forges,forges_edges,
  connect_forge}.rs`. The process tier (`crates/temper/tests/process.rs`, its supports) calls
  ctl with `--project` and changes only if a JSON it compares names the first repository's
  project.
- Status, as `notes/common.md` says:
  - `temper/docs/design/protocol/setup.status.md`: remove the second **Not yet** bullet whole
    (`- **Not yet:** 6.7: \`SetUp\` and …`, all that `welcome-ready` left of it).
  - `temper/docs/design/protocol/forge.status.md`, **Not yet**: remove the sentence `2.1: a first
    set up's intent keeps a project (…).`
  - `temper/docs/design/protocol/store.status.md`, **Not yet**: remove the sentence `2: family
    25's first set up's intent keeps a project (…).`
  - `temper/docs/design/client/README.status.md`, **Not yet**: remove the lines `4.6, 8: \`SetUp\`,
    \`FirstAsk\` and the row's \`first\` carry a project, …` through `` client-wire-v1.schema`). ``
  - `temper/docs/design/client/tools.status.md`, **Not yet**: remove the sentence `5.5: \`forge
    setup\` and \`forge connect … --first\` send their project … (\`forges/connection.rs\`).`

## Steps

1. The wire's schema, regeneration, `lib.rs` and goldens; `heavy cargo nextest run -p
   temper-client-protocol-wire` and `heavy cargo nextest run -p temper-codecs-world --test suite
   -E 'test(drift::) | test(client_wire)'`.
2. The store's; `heavy cargo nextest run -p temper-protocol-store` and the codecs world's
   `test(records) | test(store)` filter.
3. The engine's forge domain; `heavy cargo nextest run -p temper-engine-domain-forge --lib`, `-p
   temper-engine-domain --lib -E 'test(setup)'`, `-p temper-engine-forge-world` and `-p
   temper-engine-domain-world --test suite -E 'test(first_repository)'`.
4. The service and host; `heavy cargo nextest run -p temper-service --lib -E 'test(family) |
   test(setup) | test(system)'`; `heavy cargo check -p temper --tests`.
5. The client domain and ctl; `heavy cargo nextest run -p temper-client-domain --lib`, `-p
   temper-client-tools --lib -E 'test(forges) | test(forge_connection) | test(adopt)'`, `-p
   temper-client-shell-cli --lib`, `-p temper-client-world --test suite -E 'test(first_repository)
   | test(forges) | test(connect_forge)'`.
6. The status edits; the citations check.
7. `cargo fmt`; old forms for temper; `heavy cargo clippy` naming every package changed,
   `--all-targets -- -D warnings`; the focused tests of each package changed and of its direct
   dependents in temper; `heavy cargo check --tests` for `temper-client-view`,
   `temper-client-shell-web` and `temper-client-shell-terminal`. Commit the schemas, generated
   code and goldens with their mappings and tests, then the status text; push.

## Report

As the implementer's text says, with the regenerated files' `git diff --stat`, the goldens
deleted, and each test renamed or rewritten.

## Rebase (2026-11-03, step merge, foreman first-run-projects/setup-project-goes/foreman-1)

resolve README.status.md by preserving both adjacent gap removals
The merge conflicts only in `temper/docs/design/client/README.status.md`.
Task commit `610719c9c6` removes the three-line `4.6, 8: SetUp, FirstAsk` project gap.
Incoming integration commit `3fb03466e3` removes the adjacent three-line `3.5, The first chat` gap.
Resolve by deleting both blocks, leaving the `4.4` picker entry followed directly by `5.3, Projects in the banner`.
Preserve all other status text; choosing either side wholesale would restore a closed gap.
`temper/tools/gallery/src/worlds.rs` auto-merged successfully; the refusal reports no conflict there.
