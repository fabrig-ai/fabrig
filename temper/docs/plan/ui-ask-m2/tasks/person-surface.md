+++
id = "person-surface"
part = "person"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/reads/person.rs", "jig/testing/jig-core-kit/src/**", "jig/tests/core/tests/suite/person_reads.rs", "jig/tests/core/tests/suite/main.rs"]
effort = "high"
minutes = 90
+++

## Goal

Give jig's core the surface of a run's read as its person
(`jig/docs/design/domain/engine.md`, 7.3, *A run's read as its person*):
the named action `PersonRead`, its answer `Now::PersonAnswer`, its routing
and its first checks, the asking family and a person requester, with every
admitted read answered unknown until task `person-reads` makes them. Small
and certain, so that temper's root can build on it at once.

Acceptance:

- `NamedAction::PersonRead(crate::reads::Read)` and `Now::PersonAnswer {
  to, key, answer: Box<crate::reads::Answer> }` exist as
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 1 gives them,
  with their doc comments.
- `routing/run/calls.rs`'s `named_action` sends `PersonRead` to
  `crate::reads::person::run(core, env, to, key, read)` before replay and
  call admission, as it sends `NamedAction::Settings`; `named_tool_kind`
  maps it to `ToolKind::Ask`.
- `reads/person.rs`'s `run`: a run whose authority lacks the asking family
  is answered `Now::EffectAnswer` with the part `authorize_tool` gives, as
  `reads::settings::run` refuses; a run whose task's requester is not
  `Party::Person` is answered `Now::PersonAnswer` with
  `Answer::None(Why::Unknown)`; every other read, in this task, is answered
  the same unknown. Nothing is committed and no call record is kept.
- Every match the new `Now` variant forces handles it as the same site
  handles `Now::SettingsAnswer` (relayed, captured or dropped), and
  jig-core-kit's world captures it in `person_answers: Vec<(Token,
  core::CallKey, core::reads::Answer)>` beside `settings_answers`, with a
  helper that sends a `PersonRead` as the assigned run's call:
  `World::person_read(&mut self, read: core::reads::Read) -> Token`.
- One story in jig's core world, `jig/tests/core/tests/suite/person_reads.rs`,
  `a_run_reads_as_its_person_only_with_the_asking_family`: the settings
  stories' run (`jig/tests/core/tests/settings/support.rs`,
  `settings_run_with_limits`) given the asking family reads
  `Read::Project { project }` of its own project and is answered
  `PersonAnswer` (unknown, in this task), and asked again under the same
  call name is answered again; the same run without the asking family is
  answered beyond authority, naming the family; the store's rows are
  unchanged by both.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 1.
- `jig/docs/design/domain/engine.md`, 7.3: the table's last row and *A run's
  read as its person*; `jig/docs/design/domain/authority.md`, section 3 (the
  asking family).
- The pattern to follow: `jig/crates/jig-domain-core/src/reads/settings.rs`,
  `run` and `refused`; `routing/run/calls.rs`, `named_action` (lines 414 to
  460) and `named_tool_kind`.
- The settings run read's stories and fixtures:
  `jig/tests/core/tests/suite/settings_reads.rs`,
  `jig/tests/core/tests/settings/support.rs` (`run_configuration`,
  `settings_run_with_limits`, `settings_goal`).

## Owns and may change

- **Owns:** `vocabulary.rs`'s two variants; `calls.rs`'s routing and tool
  kind; the new `reads/person.rs` and its `mod` line in `reads.rs`; the
  new story file and its `mod` line in `jig/tests/core/tests/suite/main.rs`;
  jig-core-kit's capture and helper.
- **Forced arms** of `Now::PersonAnswer`, each edited only for that, in:
  `jig/crates/jig-domain-core/src/tests/runs/completion_retry.rs`,
  `jig/crates/jig-protocol-store/src/ended.rs`,
  `jig/examples/ops/domain/src/route.rs`,
  `jig/testing/jig-client-kit/src/joint/engine.rs`,
  `jig/testing/jig-core-kit/src/client_peer.rs`,
  `jig/testing/jig-test-domain/src/routes.rs`,
  `jig/testing/jig-test-service/src/clients/translation.rs`,
  `jig/testing/jig-test-service/src/route.rs`,
  `jig/testing/jig-test-store/src/route.rs`,
  `jig/tests/ops/src/joint/engine.rs`, and any other the compiler names.
- **temper's one line, pre-approved across the tree boundary:** in
  `temper/crates/temper-engine-domain/src/route/now.rs`, the arm
  `core::Now::PersonAnswer { .. } => unreachable!("temper's root asks no person read yet")`,
  which task `see-route` replaces; nothing else of temper.
- **Options, pre-approved:** if the asking family has no configured name a
  jig fixture can grant, add it to the fixture's charter as
  `run_configuration` adds the settings family, and say so. If the kit's
  protocol stand-in cannot send a named action directly, add the smallest
  sending helper beside `relay_call`, and say so.

## Steps

1. The two variants and the routing; `reads/person.rs` with `run`.
2. The forced arms, then the kit's capture and helper.
3. The story; break the family check once (route `PersonRead` without
   `authorize_tool`) to see its refusal row fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-kit -p
   jig-core-world -p temper-engine-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-domain-core --lib`; `heavy cargo nextest
   run -p jig-core-world --test suite -E 'test(/person_reads|settings_reads/)'`;
   `heavy cargo check --tests` on each other package whose file you changed
   (name them). Commit and push.

## Report

As the implementer's text asks; name each forced arm and what it does.
