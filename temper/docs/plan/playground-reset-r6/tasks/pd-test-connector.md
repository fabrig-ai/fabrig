+++
id = "pd-test-connector"
part = "pd"
kind = "feature"
needs = ["pd-vocabulary"]
files = [
  "jig/testing/jig-test-connector/**",
  "jig/testing/jig-test-domain/src/routes.rs",
  "jig/tests/test-connector/**",
]
effort = "medium"
minutes = 60
+++

## Goal

jig's testing application's connector releases a deleted project (contracts J5): told
`ReleaseProject { project }`, it erases every row it keeps that names the project, durably,
then answers `ProjectReleased`. The conformance world (`pd-conformance`) then sees no
connector row naming a deleted project.

Acceptance:
- `jig_test_connector::Event::Release { project: u32 }` (new, appended): the connector drops
  from memory and erases (`Request::Erase`) every one of its records naming the project:
  `Record::Adoption { project, .. }` (and its `adoptions` entries), `Record::Task { project, .. }`,
  `Record::Proposal { project, .. }` and its outbox proposals, and every other record whose
  value names the project (the implementer lists them; a record keyed by a task of the project
  counts as naming it); then `Request::Released { project }` (new, appended), in the same
  decision. A project it holds nothing for is answered at once. Repeating it is harmless.
- jig-test-domain's root routes `core::Ask::ReleaseProject { project }` for connector `number`
  to `connector::Event::Release { project }` and `connector::Request::Released { project }` to
  `core::Event::ProjectReleased { project, connector: number }` (replacing the answer-at-once
  arm `pd-vocabulary` wrote), the erases joining the same decision as every connector `Erase`.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J5.
- The code: `jig/testing/jig-test-connector/src/{boundary.rs,domain.rs,outbox.rs}` (records
  ~782, `adopt` ~422, `restore` ~494), `jig/testing/jig-test-domain/src/routes.rs`
  (`route_ask` ~116, `route_connector` ~581).

## Owns and may change

Owned: the `files` above. Compiler-forced arms on the connector's `Event`/`Request` elsewhere
(common.md). The core's shapes are `pd-vocabulary`'s.

## Steps

1. The event, request and `release(domain, project, out)` in the connector.
2. Routing in jig-test-domain.
3. Tests in `jig/tests/test-connector/tests/suite/` (new `release.rs`, registered in
   `main.rs`): `a_released_project_leaves_no_connector_row` (adopt into projects 1 and 2, keep
   a task row and a proposal for project 2; `Release { project: 2 }` erases project 2's rows,
   answers `Released`, keeps project 1's; a restart restores nothing of project 2) and
   `releasing_a_project_without_rows_answers_at_once`.
   Commands: `heavy cargo nextest run -p jig-test-connector-world --test suite -E 'test(/release/)'`,
   then `heavy cargo nextest run -p jig-test-connector-world --test suite` and
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion_vocabulary/)'`
   (its release test now goes through the connector and must still see the answer; do not
   edit `jig/tests/core`, which `pd-core-decide` writes: if that test fails, report it).
4. `cargo fmt`; `heavy cargo clippy -p jig-test-connector -p jig-test-domain -p jig-test-connector-world --all-targets`;
   old forms; commit and push.

## Report

The connector records you found naming a project, and which of them you erase.
