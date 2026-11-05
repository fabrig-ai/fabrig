+++
id = "making-root"
part = "making-engine"
kind = "feature"
needs = ["making-decide"]
files = ["temper/crates/temper-engine-domain/src/route/making.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/applications.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/tests/making.rs", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/src/tests/documents.rs"]
effort = "high"
minutes = 70
+++

# The root's part: scope, requests and the connector's act

## Goal

The engine's root carries a making from the party to the connector and back, and turns a made row into
jig's connector act (`temper/docs/design/domain/forge.md`, 4.5; contracts T5).

Acceptance:

- `temper/tests/engine/domain/tests/suite/making.rs`, on `first_repository.rs`'s and `removal.rs`'s pattern:
  - `make_scope_is_the_project_into_names`, and the deployment's with none: a client's own scope is not
    trusted, the project being inside the record;
  - `a_kept_making_is_answered_and_marks_forges`;
  - `made_into_a_project_is_adopted_by_the_connectors_act`: after `MakingAdopt`, jig's `Act::Adopt` is
    decided, the repository is adopted, owned, and the row is erased;
  - `a_restart_during_the_act_joins_it_and_adopts_once`;
  - `a_document_with_no_grant_says_so_in_the_row`: `Refused(NotOffered)` fails the row, the words naming
    `adopt` and `engine.acts`, as `route/playground.rs` names `remove_project`;
  - `jig_busy_is_asked_again`: `Again` becomes `MakingAgain`;
  - `a_deleted_project_fails_the_row`.
- `temper init`'s document grants `adopt` (`temper/crates/temper-init/template/temper.json`).
- `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world -p temper-init` pass;
  `heavy cargo check --tests -p temper-service -p temper-system-world` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T5 and T6.
- `temper/docs/design/domain/forge.md`, 4.5; `temper/docs/design/protocol/setup.md`, 8.3, "Its acts".
- jig's `jig/docs/design/domain/people.md`, 5.3.

## Owns and may change

- The files of the header; the new test file and its `mod` line in `temper/tests/engine/domain/tests/suite/main.rs`.
- **The scope:** `route/applications.rs`, 24 to 38: `Ask::MakeRepository { into: Some(into), .. }` is
  `ApplicationScope::Project(into.project)`, and `into: None` `Deployment`.
- **`route/making.rs`, on `route/playground.rs`'s `reset` and `acted`:**
  - `Request::MakingAct` becomes `core::Event::Act { reply_to, connector: domain.config.forge_connector,
    key, act: Box::new(people::Act::Adopt { project, adoption }), carried: None }`, the people's
    `Adoption` built as `forge_route.rs`'s `adopt_repository_ask` builds its `Ask`. Share that body;
    no second copy.
  - The act's answer: `Outcome::Adopted` is `MakingActed { key, outcome: None }`; `Outcome::Again(_)` is
    `MakingAgain`; a refusal, a `MakingFailed` at `Adoption`, `NotAllowed` for jig's and its words.
- **The ask back:** `route/asks.rs`, 143: `core::Ask::Adopt { request, project, adoption }` whose
  `application_part_request(request)` is `None` and which names a row `Adopting` is an act's:
  `Work::Forge(forge::Event::MakingAsked { .. })`, the forge's `Adoption` read back from the people's
  options. `domain.adoption_restore` holds it under `request`, as a party's is, so that
  `forge::Request::Adopted` (`forge_route.rs`, 4146) seeds the parties and answers `people::Event::Decided`
  unchanged. Anything else stays `NotOffered`.
- **`engine.acts`:** `temper-init/template/temper.json` and its test, `temper-init/src/tests/documents.rs`.
  Other documents naming the grant follow: `rg '"remove_account"' temper`.
- Remove "8.3: `"adopt"` in `engine.acts`." from `temper/docs/design/protocol/setup.status.md`.
- Pre-approved:
  - `Request::MakingCopied` routes to the service as `Request::SetUp` does (`forge_route.rs`, 4035): a
    `Delivery`, or a root `Request` of its own; `making-run` handles it.
  - If `act` needs `amendment_bytes` room, it is what `adoption_bytes` charges a party's.

## Steps

1. The scope. 2. `route/making.rs`. 3. The ask back. 4. The grant. 5. The tests, each biting: trust the
client's scope, and treat `Again` as refused, each once. 6. The implementer's checks; commit and push.

## Report

As the implementer's text asks; how the people's options are read back.
