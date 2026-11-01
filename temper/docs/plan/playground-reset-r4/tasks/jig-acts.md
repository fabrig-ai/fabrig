+++
id = "jig-acts"
part = "project-deletion"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-people/src/**",
  "jig/crates/jig-protocol-config/src/acts.rs",
  "jig/crates/jig-protocol-config/src/tests/acts.rs",
  "jig/crates/jig-domain-core/src/connector.rs",
  "jig/crates/jig-domain-core/src/effects.rs",
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/tests/people/tests/suite/acts.rs",
]
effort = "high"
minutes = 50
+++

## Goal

jig's surface for deleting a project (`notes/contracts.md`, section 4):
`Ask::RemoveProject`, `Act::RemoveProject`, `Act::Cancel`, their
`ActKind`s and grant names, `Outcome::ProjectRemoved`, and
`connector::Ask::ReleaseProject` / `connector::Event::ProjectReleased`.
No body yet: each new request and act is refused *not offered* at the
people child's entrance, before any record is made, so no new value is
ever saved and jig's store does not move (its move is `pd-store`'s,
D2). Every application root, temper's included, compiles.

Acceptance:

- the items below exist with these names and shapes;
- `engine.acts`' `"remove_project"` and `"cancel"` parse to the new
  kinds; an unknown name is still refused;
- a granted `Act::RemoveProject` or `Act::Cancel`, and an owner's
  `Ask::RemoveProject`, answer `Refused(NotOffered)` and leave no
  `Acted`/`Answer` row and no flight (tests below);
- temper's root answers `ReleaseProject` with `ProjectReleased` at once;
  jig's test root and the ops example do the same;
- `heavy cargo check --tests` passes for every package that names
  `jig_domain_people::{Ask, Act, Outcome}` or `jig_domain_core::connector`
  (the list in Steps), and the focused tests of the packages changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1
  and 4).
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/people.md`, 5.1 (requests), 5.2, 5.3 (the
  deployment's acts: an ungranted act is refused *not offered*, unsaved).
- `jig/docs/design/domain/tasks.md`, section 7 (the compound request;
  `Cancel` is its deployment form).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s3-jig-project.md`,
  sections 4, 5 and 8 (reverse references of `Act`, `ActKind`, `Ask`,
  `Outcome::ProjectMade`).

## Owns and may change

Owned: the `files` above.

Neighbour files, each edited only for the compiler-forced arm or
literal of a type this task changes (the implementer text's allowance,
here across trees for the items this task adds):

- jig: `jig-domain-core` (any file: `routing/people.rs`,
  `routing/party_controls.rs`, `routing/vocabulary.rs`, `roles.rs`,
  `person_task.rs`, `acts/requests.rs`, `routing/application_routes.rs`,
  ...), `jig-protocol-store/src/codec/people/{asks,acts,outcomes}.rs`
  and `codec/people.rs`, `jig-protocol-engine-clients/src/{request.rs,
  translate/asks.rs, translate/answers.rs}`, `jig/testing/**`,
  `jig/examples/ops/**`, `jig/tests/**` (worlds' exhaustive matches).
- temper, call sites only:
  `temper/crates/temper-engine-domain/src/route/forge_route.rs`
  (`effect_ask`), `temper/crates/temper-engine-domain/src/store.rs`
  (the `Answer`/`Acted` sizing matches),
  `temper/crates/temper-engine-domain/src/route/demo.rs` (outcome
  matches), `temper/tests/engine/domain/tests/suite/root_routes.rs`,
  and any other temper line the compiler names for these types.

What the arms do, by default:

- **people child (owned):** `valid_ask` is false for
  `Ask::RemoveProject` and `invalid_ask` answers `Refusal::NotOffered`
  for it; `acts::valid` answers `Err(Refusal::NotOffered)` for
  `Act::RemoveProject` and `Act::Cancel`. Each such line carries a
  comment `// Refused until pd-close gives it a body.` (`pd-close` is
  the jig task that fills them). Every other people match gets the arm
  its nearest kin has: `RemoveProject` as `Rename` (project scope, the
  project's owners' `CapacityAction`, no model context, no words);
  `Act::Cancel`'s retained charge as `Close`'s reason (`words`);
  `Act::RemoveProject` charges nothing beyond the act's base.
  `Outcome::ProjectRemoved` is permanent, and its answer shape is
  `Expected::RemoveProject`'s (a new `Expected` variant).
- **codec, engine-clients (neighbours):** the new values cannot reach
  them yet; their encode arms return the module's existing bound
  problem (`codec/people.rs`'s `problem()`, the wire's equivalent), each
  with `// Encoded from pd-store on.` or `// Translated from pd-wire on.`.
  Never `unreachable!`/`panic!` in step code.
- **core (neighbour):** `deployment_acts::start` gives the new acts no
  digest; `deployment_acts::route` refuses both `NotOffered` (unreached:
  people refuses first); a party `Ask::RemoveProject` reaching the core
  is refused `NotOffered` the same way; `effects.rs` takes
  `connector::Event::ProjectReleased` as nothing yet (an arm beside
  `Adopted`'s), and `vocabulary.rs`' `effectconnector_new_work` says
  `false` for it.
- **roots:** each `connector::Ask` dispatcher answers `ReleaseProject {
  request, project }` at once by sending the core
  `Event::EffectConnector(connector::Event::ProjectReleased { request,
  project })`. In temper's `effect_ask` the arm carries the comment
  `// The forge connector erases its rows of the project first from
  rm-release-project on.` In jig's test root (`jig-test-domain`
  `routes.rs`) and the ops example (`jig-ops-domain` `route.rs`) it is
  their whole answer: if a connector there keeps a row naming a project,
  leave the answer at once and say so in the report.

Pre-approved: if the people child's `Expected`/answer-shape check or its
start fit needs a new arm elsewhere than named, add it; if a test kit's
referee (`jig/testing/jig-conformance`, `jig/tests/core/src/referee`)
classifies `Ask` or `Act` variants exhaustively, give the new ones the
arm of `Rename` / `Close` and say so.

## Steps

1. **people boundary** (`jig/crates/jig-domain-people/src/boundary.rs`):

   ```rust
   // Ask, beside MakeProject:
   /// The project's owner deletes it whole (people.md, 5.2; draft 5).
   RemoveProject { project: u32 },

   // Act, after Adopt:
   /// The deployment deletes a project whole (people.md, 5.3).
   RemoveProject { project: u32 },
   /// The deployment's compound cancel of one live task of the project,
   /// its delegates with it (tasks.md, section 7), whoever requested it.
   Cancel { project: u32, task: u64, grace: Option<Duration>, reason: Box<[u8]> },

   // ActKind, appended, and appended to ActKind::ALL in this order:
   RemoveProject,
   Cancel,

   // Outcome, beside ProjectMade:
   /// The project is closed and its deletion under way; it is gone to
   /// every request from this answer on (draft 5).
   ProjectRemoved { project: u32 },
   ```

   `Act::Cancel`'s answer is the compound's, as `Close`'s: no new
   outcome. Export nothing new from `lib.rs` beyond what the enums carry.
2. **people child:** the entrance refusals and arms above
   (`domain.rs`, `domain/acts.rs` `valid`/`kind`/`valid_outcome`,
   `domain/start.rs`, `domain/start/fit.rs`, `models.rs`, `words.rs`,
   `batch.rs`, `capacity.rs`).
3. **config** (`jig/crates/jig-protocol-config/src/acts.rs`, `grant`):
   `b"remove_project" => ActKind::RemoveProject`, `b"cancel" =>
   ActKind::Cancel`.
4. **core connector vocabulary**
   (`jig/crates/jig-domain-core/src/connector.rs`):

   ```rust
   // Ask:
   /// Erase the connector's rows of a deleted project (its adoptions,
   /// its settings of the project's scope); answered by ProjectReleased.
   ReleaseProject { request: Token, project: u32 },
   // Event:
   /// The connector holds no row of the project any more.
   ProjectReleased { request: Token, project: u32 },
   ```

   The core sends it later inside `vocabulary::Ask::Effect`, one
   request token per connector; nothing sends it yet.
5. **Every arm the compiler asks for** in the neighbour files, as
   above; temper's three files are call sites only.
6. **Tests:**
   - `jig/crates/jig-protocol-config/src/tests/acts.rs`:
     `remove_project_and_cancel_are_granted_by_name` (a document
     granting `["remove_project", "cancel"]` parses to those kinds), and
     extend the duplicate-kind case if it lists every kind.
   - `jig/tests/people/tests/suite/acts.rs` (`jig-people-world`), with
     `grants()` granting the two new kinds too:
     `remove_project_and_cancel_are_not_offered_and_unsaved` (each act,
     granted, with an owner, answers `Refused(NotOffered)`;
     `world.acts` stays empty; after `commit_all` no `Acted` row
     exists, read as the file's other unsaved cases read it);
     `a_project_owners_remove_project_is_not_offered_and_unsaved` (in
     the same world's person request path, or the closest existing
     people test of a refused-at-entrance request, e.g.
     `jig/crates/jig-domain-people/src/tests/projects.rs`: no `Answer`
     row, the same key then free for another ask).
7. Checks, once: `cargo fmt`; `heavy cargo clippy -p jig-domain-people
   -p jig-protocol-config -p jig-domain-core --all-targets -- -D
   warnings` and every other package you changed; focused tests:
   `heavy cargo nextest run -p jig-domain-people`, `-p
   jig-protocol-config`, `-p jig-people-world`, `-p jig-core-world
   --test suite -E 'test(/deployment_acts/)'`; then `heavy cargo check
   --tests` for each of `jig-domain-core jig-protocol-store
   jig-protocol-engine-clients jig-test-domain jig-test-service
   jig-ops-domain jig-core-world jig-conformance jig-conformance-world
   jig-client-world jig-store-world temper-engine-domain temper-service
   temper-engine-domain-world`, one at a time.
   Commit, push.

## Report

fab's sections; `## Interfaces` with every item of step 1 and step 4
(path and signature), and the refusing lines' file:line so `pd-close`,
`pd-cancel` and `pd-store` find them; each neighbour file edited and
why; any connector that keeps project rows in the test root or the ops
example.
