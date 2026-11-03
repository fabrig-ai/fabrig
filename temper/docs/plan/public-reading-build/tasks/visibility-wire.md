+++
id = "visibility-wire"
part = "jig-public"
kind = "interface"
needs = ["visibility-surface"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/*.bin", "jig/crates/jig-client-protocol-wire/src/tests/fixtures.rs", "jig/crates/jig-client-domain/src/contract/asks.rs", "jig/crates/jig-client-domain/src/contract/me.rs", "jig/crates/jig-client-domain/src/contract/outcome.rs", "jig/crates/jig-client-domain/src/requests.rs", "jig/crates/jig-client-protocol-link/src/translate/asks.rs", "jig/crates/jig-client-protocol-link/src/translate/compound.rs", "jig/crates/jig-client-protocol-link/src/translate/store.rs", "jig/crates/jig-client-protocol-link/src/translate/answers.rs", "jig/crates/jig-client-protocol-link/src/translate/documents_part0.rs", "jig/crates/jig-client-protocol-link/src/tests/projects.rs", "jig/crates/jig-protocol-engine-clients/src/request.rs", "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs", "jig/crates/jig-protocol-engine-clients/src/translate/answers.rs", "jig/crates/jig-protocol-engine-clients/src/translate/owner.rs", "jig/crates/jig-protocol-engine-clients/src/tests/asks.rs", "jig/crates/jig-protocol-engine-clients/src/tests/answers.rs", "jig/crates/jig-protocol-engine-clients/src/tests/documents.rs", "jig/crates/jig-domain-core/src/documents.rs", "jig/crates/jig-domain-core/src/documents/small.rs", "jig/tests/client-wire/tests/suite/set_visibility.rs", "jig/tests/client-wire/tests/suite/main.rs"]
effort = "medium"
minutes = 130
+++

# SetVisibility and `me`'s public identifier across the client wire, the link and the engine end

## Goal

A client can send `SetVisibility { project, public }` and read whether a project is public in
`me`: the contract's request and outcome, the client wire v19 in place, both translations
(the link's contract <-> wire, the engine end's wire -> people), and the `me` field from the
core's frame to the client, every literal edited for the field alone
(`jig/docs/design/client/domain/contract.md`, 3.2's `SetVisibility` row and 5.1's `me`;
`jig/docs/design/domain/people.md`, 13.2). Nothing decides it yet: the people child refuses
it (`visibility-surface`) until `visibility-core`, and `MeProject.public` is always `None`
until `public-component` fills it.

Acceptance:

- `jig_client_domain::contract::Ask::SetVisibility { project: u32, public: bool }`, appended
  last to `Ask` (`contract/asks.rs`), its project typed as `RemoveProject`'s is, doc citing
  contract.md 3.2 and people.md 13.2; `contract::DocumentOutcome::VisibilitySet { project:
  u32, public: bool }` appended last (`contract/outcome.rs`), doc: the project public or
  private from that decision; `contract::MeProject` gains `pub public: Option<u64>`, appended
  last, doc: its public identifier while it is public, none while private (contract.md, 5.1;
  people.md, 13.8).
- `jig_domain_people::Outcome::VisibilitySet { project: u32, public: bool }` appended last
  (`jig-domain-people/src/boundary.rs`, the outcome alone), kept as the people family keeps
  every outcome: `people-v14.schema` in place, `enum Outcome` gains
  `visibility_set: OutcomeVisibilitySet` appended last, `record OutcomeVisibilitySet {
  project: u32 public: bool }`, its codec both ways in `codec/people/outcomes.rs`, generated
  code and goldens regenerated. Nothing produces it yet; `visibility-core` does.
- The client wire v19, in place (as `64c006515c` carried `RemoveProject`): `record
  SetVisibility { project: u32 public: bool }` beside `RemoveProject`, `Ask`'s
  `set_visibility: SetVisibility` appended last (`client-v19-part1.schema`);
  `record OutcomeVisibilitySet { project: u32 public: bool }` and `DocumentOutcome`'s
  `visibility_set` appended last; `MeProject` gains `public: option u64`, appended last.
  Generated code and goldens regenerated through the drift test, never by hand.
- The engine end: `w::Ask::SetVisibility` -> `people::Ask::SetVisibility { project, public }`
  (`translate/asks.rs`), sorted in `request.rs` wherever `RemoveProject` is (a project's own
  act, by its owner); `people::Outcome::VisibilitySet` -> `w::DocumentOutcome::VisibilitySet`
  (`translate/answers.rs`); `MeProject.public` written `None` from every frame
  (`translate/owner.rs`, one line, commented: filled by the public names, people.md 13.8).
- The link: `c::Ask::SetVisibility` <-> `w::Ask::SetVisibility` (`translate/asks.rs`,
  `store.rs`, `compound.rs` grouped with `RemoveProject`: not an operation of a compound);
  `w::DocumentOutcome::VisibilitySet` -> `c::DocumentOutcome::VisibilitySet`
  (`translate/answers.rs`); `w::MeProject.public` -> `c::MeProject.public`
  (`translate/documents_part0.rs`).
- The core's `FrameProjectRole` gains `pub public: bool` (`documents.rs`, doc citing people.md
  13.2), written `false` in `documents/small.rs` (`visibility-core` fills it from state).
- Tests, each with both values of `public` and the largest project:
  - `jig/tests/client-wire/tests/suite/set_visibility.rs` (a module line in `main.rs`), as
    `remove_project.rs` is: `a_set_visibility_ask_round_trips`,
    `a_visibility_set_outcome_round_trips`, `a_me_project_round_trips_with_and_without_its_public_identifier`
    (`Some(u64::MAX)`, `Some(0)`, `None`);
  - engine-clients `src/tests/asks.rs`: `set_visibility_reaches_the_people_child_as_sent`;
    `src/tests/answers.rs`: `visibility_set_answers_as_its_document_outcome`;
    `src/tests/documents.rs`: `me_names_no_public_identifier_until_the_names_are_held`
    (a frame with `public: true` still gives `None`);
  - link `src/tests/projects.rs`: `set_visibility_crosses_the_link_both_ways` and
    `a_public_identifier_in_me_reaches_the_contract`;
  - `jig-protocol-store`'s `src/tests/codec.rs`: `a_visibility_set_outcome_round_trips_as_kept`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 2.
- `jig/docs/design/client/domain/contract.md`, 3.2 (the `SetVisibility` row) and 5.1
  (`me`); `jig/docs/design/domain/people.md`, 13.2 and 13.8. `jig/docs/design/protocol/client.md`,
  section 19, is not this task's.
- As they stand: the precedent commit `64c006515c` (`git show --stat`, then its hunks in the
  files above); `jig-client-protocol-wire`'s two v19 schemas; `contract/asks.rs`, `me.rs`,
  `outcome.rs`; `jig-protocol-engine-clients/src/translate/{asks,answers,owner}.rs`,
  `request.rs`; the link's `translate/`; `jig-domain-core/src/documents.rs` (`FrameProjectRole`)
  and `documents/small.rs`; `jig-protocol-store/src/codec/people/outcomes.rs`.

## Owns and may change

- Owns the files in the header, and, of files `visibility-surface` owned (merged before you
  start): in `jig/crates/jig-domain-people/src/boundary.rs` the `Outcome` enum's new variant
  alone; in `jig/crates/jig-protocol-store/` `schema/people-v14.schema` (the outcome's record
  and variant alone), `src/generated/people_v14.rs` and `golden/people/**` (regenerated),
  `src/codec/people/outcomes.rs` and `src/tests/codec.rs` (the outcome's arms and test).
- **Compiler-forced arms** for `contract::Ask::SetVisibility`, `contract::DocumentOutcome::VisibilitySet`,
  `people::Outcome::VisibilitySet` and `w::Ask`/`w::DocumentOutcome`'s new variants, each
  grouped with `RemoveProject`'s (an ask) or `Renamed`'s / `ProjectRemoved`'s (an outcome),
  doing what those do or nothing:
  - jig: `jig-client-view/src/{state/activity/sending.rs, state/page/project/owner_form.rs,
    tree/sections/project/owner.rs, state/outcome.rs, tree/fragments/outcome.rs}`,
    `jig-client/src/thread/model.rs`, `jig/testing/jig-client-test-kit/src/check.rs`,
    `jig/testing/jig-client-scripted-engine/src/{requests.rs, requests/projects.rs,
    codec/values/outcome.rs}` (the scripted engine does not decide `SetVisibility`:
    `project_request` returns `None` for it, as for the other asks it does not script),
    `jig/tests/client/src/{observe.rs, harness/referee.rs, harness/referee/offers.rs}`,
    `jig/tests/client-protocol/src/front/observe.rs`, `jig-protocol-engine-clients/src/tests/fixtures.rs`,
    the people child's and the core's outcome matches (`jig-domain-people/src/{capacity.rs,
    domain/acts.rs, domain/deletion.rs, domain/requests.rs, domain/start.rs, domain/start/fit.rs}`,
    `jig-domain-core/src/{routing/party/people.rs, start/links.rs}`), `jig/testing/jig-test-domain/src/routes.rs`;
  - temper (required arms, each edited only for the arm): `temper-client-tools/src/jig/render/outcome.rs`
    (the ask's name `set-visibility`, beside `remove-project`; `public-ctl` owns what it
    prints), `temper-client-tools/src/jig/render/document/act_answers.rs`,
    `temper-client-domain/src/pages/switcher.rs`, `temper-engine-domain/src/{store.rs,
    route/playground.rs}`.
  Any other arm the compiler names in jig or temper: the same, listed in the report.
- **Literals,** each edited only to add the field, `public: None` (contract `MeProject` and
  wire `MeProjectParts`) or `public: false` (`FrameProjectRole`): `jig-domain-core/src/tests.rs`,
  `jig-protocol-engine-clients/src/tests/{llm.rs, fixtures/documents.rs}`,
  `jig-client-protocol-wire/src/tests/fixtures.rs`, `jig-client/src/tests/fixtures.rs`,
  `jig-client-view/src/tests/{state/domain/frame_facts.rs, state/domain/pages.rs, tree/cards.rs}`,
  `jig-client-protocol-link/src/tests/llm_accounts.rs`, `jig/testing/jig-client-scripted-engine/src/{tests.rs,
  requests/projects.rs, codec/values/me.rs}`, `jig/testing/jig-client-test-records/src/owner.rs`,
  `jig/testing/jig-client-test-domain/src/tests/root/navigation.rs`, `jig/tests/client/src/stories.rs`,
  `jig/tests/ops/tests/suite/client_stories/orders.rs`; temper (a required field):
  `temper-client-domain/src/tests/{connect.rs, fixtures/chats.rs, pages/welcome.rs}`,
  `temper-client-domain/tests/suite/{change.rs, task_pages.rs}`, `temper-client-tools/src/tests/fixtures.rs`,
  `temper-client-view/src/tests/fixtures.rs`, `temper-client-view/tests/suite/{change_page.rs,
  task_pages.rs}`, `temper-client-shell-cli/src/tests/fixtures/recovery.rs`,
  `temper/tools/gallery/src/worlds.rs`, `temper/tests/engine/domain/src/forge_routes.rs`,
  `temper/tests/client/src/component.rs`. Any other the compiler names: the same.
- Not this task's: an offered act kind (`ActKind`) for `SetVisibility` (no design names one),
  the public identifier's value (`public-component`), deciding the request (`visibility-core`),
  ctl's command (`public-ctl`).
- Pre-approved: if a variant cannot be appended last because a schema's enum is ordered
  otherwise, follow the schema's order and say so. If configuring-p2-build's variants have
  reached main first, append after them.

## Steps

1. The contract's variants and field, the people outcome; the core's field; forced arms and
   literals until `heavy cargo check -p jig-client-domain -p jig-domain-core --tests` passes.
2. The wire schemas and the people family's outcome; regenerate both families
   (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(/drift::/)'`,
   then again without the variable); the translations; forced arms in jig and temper until
   `heavy cargo check -p jig-protocol-engine-clients -p jig-client-protocol-link -p jig-client-view -p temper-client-tools -p temper-client-domain -p temper-engine-domain --tests`
   passes.
3. The tests above, each run alone first
   (`heavy cargo nextest run -p jig-client-wire-world --test suite -E 'test(/set_visibility::/)'`,
   `-p jig-protocol-engine-clients --lib -E 'test(/visib|public_identifier/)'`, and the
   link's and the store's likewise).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of `jig-client-protocol-wire`, `jig-client-domain`, `jig-client-protocol-link`,
   `jig-protocol-engine-clients`, `jig-protocol-store`, `jig-domain-people`, `jig-domain-core`,
   `jig-client-view`, `jig-client-scripted-engine`, `jig-client-wire-world`,
   `jig-store-codecs-world`, and `heavy cargo check --tests` of every temper package you
   touched; the citations check.
5. Commit (why: a project's visibility as a client request and `me`'s public identifier, which
   the core, the component and ctl build on); push your branch.

## Report

As the implementer's text says, and also: the schemas' new lines as written, every forced arm
and literal by file, and what the scripted engine does with `SetVisibility`.
