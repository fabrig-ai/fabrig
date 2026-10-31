+++
id = "fr-document"
part = "forge-records"
kind = "feature"
needs = ["fr-registry"]
files = [
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/startup/**",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/tests/fixtures/startup.rs",
  "temper/crates/temper-service/src/tests/startup*.rs",
  "temper/crates/temper-service/src/tests/document.rs",
  "temper/crates/temper/src/setup.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/seeds.rs",
  "temper/crates/temper-fake/src/demo.rs",
  "temper/crates/temper-fake/src/tests/demo.rs",
  "temper/crates/temper-init/template/temper.json",
]
effort = "high"
minutes = 90
+++

## Goal

The deployment's document gains the machine's members of
`notes/contracts.md` section 4 (`temper/docs/design/draft/playground-reset.md,
3`, "The document keeps the machine's"): `application.ports` and
`application.playground`, the latter replacing `application.demo`
(the scripted model's members and the project's name and policy; the
playground's behavior stays today's demo's until D4's `pg-` tasks).
Forges reach the service as seeds from three sources: the document's
own `application.forges` (a bridge `fr-retire` ends), seed files the
binary reads at serve start (`<data>/seed/forges/N.json`), and an
in-process world's given. `settings::Machine` is filled.

Acceptance:

- `temper-service` (`src/tests/document.rs`, new):
  `ports_and_playground_are_read` (a document with `ports` and
  `playground` gives `Machine.ports` and today's `settings::Demo` with
  the playground's members, its LLM listening on `ports.llm`);
  `a_document_with_demo_is_refused` (the old member names its path);
  `a_seed_resolves_as_a_document_forge` (a seed definition with
  credential-file members gives the same `settings::Forge` and
  `Identity` as the same forge in the document, and a `ForgeSeed` of
  its number). Forges stay required here; `fr-service` makes them
  optional.
- `temper` (the binary's unit tests): `seed_files_are_read_in_number_order`
  (two files under `<data>/seed/forges/` become two seeds; a file not
  named `N.json`, or not 0600, is refused naming it).
- `temper-fake` `src/tests/demo.rs`: its cases read `application.playground`.
- Every test that read `application.demo` reads `application.playground`,
  unchanged in meaning (`rg -l '"demo"' temper` for the fixtures).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  and 4.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.1.
- `temper/docs/design/protocol/configuration.md`, sections 3 and 8
  (the members and the startup checks);
  `temper/docs/design/protocol/setup.md`, 8.2 to 8.4 (init's facts, the
  default document, the demo's members).
- `tasks/fr-registry.md` (`Machine`, `Ports`, `ForgeSeed`, the readers'
  definition mode).

## Owns and may change

Owns, after `fr-registry` (`fr-service`, then `fr-retire`, own these
next; D4's `pg-` after them): `temper-service`'s `src/startup.rs`,
`src/startup/**` and `src/settings.rs`; the binary's `src/setup.rs`
and `src/lib.rs` and a new `src/seeds.rs`; `temper-fake/src/demo.rs`
and its tests.

May change:

- `temper-init/template/temper.json` (fr-init's next): `application.demo`
  becomes `application.playground` with contract 4's members, and
  `application.ports` added from the template's `$owned`, `$forge`,
  `$llm` (each as a number: add the placeholders' numeric form to
  `temper-init/src/template.rs` if it writes only strings). Nothing
  else of the template moves here; `temper-init`'s tests that read the
  demo member follow.
- Every fixture or test, in any crate or world, that writes or reads
  `application.demo` (temper-service `src/tests/fixtures/startup.rs`'s
  `demo_document`, `temper-fake`'s tests and `oauth/tests.rs`,
  `temper/tests/engine/domain/tests/fixtures/stock.rs`,
  `temper/tests/engine/domain/src/forge_routes.rs`, the binary's
  `tests/suite/*.rs` and `tests/support/*.rs`): the member renamed and
  its members moved, nothing else. The binary's process-tier tests
  (`tests/process.rs`, `tests/qa_all/`) are `fr-process`'s; edit their
  shared support only where the rename forces it.
- D3's files (`box_command.rs`, `forge.rs`, `fake.rs`, `fake/check.rs`,
  `forge_sign_in.rs`): only reads of `Deployment.demo`'s renamed
  members (`fake.rs:373`'s LLM configuration reads the playground's
  members, listening on `ports.llm`).
- Anything `notes/scope.md` lists as forced.

## Steps

1. **`application.ports`:** `{"owned": P, "forge": P, "llm": P}`, each
   a nonzero u16 on loopback, required when `application.playground`
   is given or a seed's definition is a stock one, otherwise optional;
   read into `settings::Machine.ports`. A setup entry's listen and the
   playground's LLM listen that differ from the ports are refused, so
   the box and serve agree.
2. **`application.playground`** (replacing `demo`, in
   `startup/application/demo.rs` and `temper-fake/src/demo.rs`, the
   module names kept): `{"provider", "key", "asked", "cued",
   "otherwise", "project": {"name", "policy"}}` or `null`; the LLM's
   listen is `127.0.0.1:ports.llm`. The bridge, until `pg-`: today's
   `settings::Demo` is built from it with `forge` the seed whose setup
   entry is fake (none: refused, "the playground has no fake forge"),
   and `project` the lowest-numbered `engine.projects` entry whose
   `policy` is the playground's (none: refused). `application.demo` is
   refused as an unknown member.
3. **Seeds:** `startup::startup`, `with_setup_given` and `with_setup`
   take `forge_seeds: &[settings::ForgeSeed]` (definitions, numbered);
   each is read by the forge, identity and setup readers in definition
   mode, its credential-file members named and resolved as the
   document's are (the binary reads the named files as today), and
   joins the document's forges (a number given twice is refused, naming
   both). `settings::Engine.forge_seeds` holds every seed, the
   document's own (fr-registry's) and these.
4. **Seed files** (the binary, new `src/seeds.rs`, called from
   `lib.rs`'s serve path): each `<data>/seed/forges/N.json`, `N` a
   decimal u16 above 0, 0600 in a 0700 directory, at most
   `forge.definition_bytes`; none and no directory is no seeds. Read at
   each serve start; a seed whose forge is kept is ignored by the
   domain (fr-domain), so a file may stay.
5. **The machine:** the binary fills `settings::Machine.person` from
   the effective user's login as `init.rs` takes it (`login`,
   `login@localhost`; none when unknown), and `Machine.github` by
   resolving GitHub's API host as `lib.rs`'s `forges` (`:342`) resolves
   a GitHub forge with a null `api` (TLS, the default trust); an
   unresolved host leaves it none. In-process worlds pass
   `Machine::none()` with ports when they need stock definitions.
6. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p temper-service -p
   temper -p temper-fake -p temper-init --all-targets -- -D warnings`;
   one at a time: `heavy cargo nextest run -p temper-service`, `-p
   temper-fake -p temper-init`, `-p temper` (its default-profile
   targets), `-p temper-one-process-world -p temper-real-loop-world -p
   temper-engine-domain-world`; `python3 -I -B scripts/old_forms.py`
   from `temper/`.
7. Commit (ports and playground, seeds, seed files, fixtures) and push.

## Report

The standard report, plus: every fixture the rename touched; how the
bridge finds the demo's forge and project; what `configuration.md`
section 3 should say of `ports`, `playground` and seed files (for its
designer).
