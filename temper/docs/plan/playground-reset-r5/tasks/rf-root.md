+++
id = "rf-root"
part = "remove-forge"
kind = "feature"
needs = ["rf-family", "jp-acts"]
files = ["temper/crates/temper-engine-domain/src/**"]
effort = "high"
minutes = 90
+++

## Goal

The root carries the removal between the connector, jig and the service
(`notes/remove-forge.md`, sections 2 to 5): every kept row's act is sent
in the deployment's name and answered back by key, whatever made it; jig's
`ProjectRemoved` reaches the connector; `RemoveForge` is admitted as the
deployment's forge acts are; the connector's removal requests become the
root's requests to the service.

Acceptance:

- `route/demo.rs`'s kept-row sender (renamed by `rf-family`; move it to a
  new `route/kept.rs` if that reads better) sends `KeptAct::Cancel {
  project, task }` as `people::Act::Cancel { project, task, grace: None,
  reason: b"The forge was removed." }` and `KeptAct::RemoveProject {
  project }` as `people::Act::RemoveProject { project }` (the variants
  `jp-acts` added), beside today's `Close` and `RemoveAccount`. The answer
  policy (`end_acted`) holds for all four: kept on `Again`, a missing
  grant or no first owner; erased on the act's outcome (`Compound`,
  `ProjectRemoved`, account removed), *ended*, *unknown*, and a refusal
  naming an unknown project or task (jig's note, section 7: a
  `RemoveProject` sent again after the deletion is refused unknown, and is
  done).
- `core::Request::ProjectRemoved { project }` (jp-acts' variant, which the
  root ignores today) is routed to the connector as
  `forge::Event::ProjectRemoved { project }` in the same decision.
- `Ask::RemoveForge` is admitted by `route/applications.rs` at the
  deployment's scope as `Connect` is, and its `Answered::RemoveForge`
  translated in the application answer (`applications.rs`, `answer`) as
  the connect answers are.
- Root requests `Request::ForgeEnded { forge }`, `Request::RemoveGroup {
  forge }`, `Request::EraseSecrets { forge }` beside `ForgeKept`, forwarded
  from the connector's (`route/forge_route.rs`, replacing `rf-family`'s
  dropping arms), each with door room for `Limits.forge.forges` at once
  (a restart resends every removal's). Root events `Event::GroupRemoved {
  forge }`, `Event::SecretsErased { forge }` admitted and passed to the
  connector (`admission.rs`, as `FirstMade` is). Forced arms in the
  service (`route.rs`: drop with a comment `// rf-service serves it`) and
  the worlds' collectors (ignored as `ForgeKept` is).
- Tests (root library tests, a new `src/tests/removal.rs`):
  `kept_cancels_and_project_removals_are_sent_in_the_deployments_name`,
  `a_kept_rows_answer_reaches_the_connector_by_key_and_unknown_is_done`,
  `a_removed_project_reaches_the_connector_in_its_closing_decision`,
  `remove_forge_is_admitted_for_owners_at_the_deployments_scope`,
  `the_removals_requests_reach_the_service_and_its_events_come_back`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 1
  and 3), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`,
  `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`
  (sections 3 and 7).
- `tasks/jp-acts.md` (the jig act and request names).
- jig's `domain/people.md`, 5.3 (deployment acts); temper's
  `docs/design/protocol/setup.md`, 8.4 (today's end acts).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 1, "End selection", "End execution", "End answer policy".

## Owns and may change

- Owns: `temper/crates/temper-engine-domain/src/**`. Runs at once with
  `rf-domain` (`temper-engine-domain-forge`): touch nothing of that
  crate.
- May change, forced arms only: `temper-service/src/route.rs` and
  `src/clients/translation.rs`; the engine, client and system worlds'
  collectors (`rg 'Request::ForgeKept'` finds them).
- Options: if `people::Act::Cancel`'s grace cannot be `None`, use the
  conversation template's grace as `Close` does and say so.

## Steps

1. The kept sender's two new acts; the answer policy.
2. `ProjectRemoved` to the connector.
3. `RemoveForge` admission and answer.
4. The removal's requests and events; forced arms.
5. Tests. Narrow: `heavy cargo nextest run -p temper-engine-domain --lib -E
   'test(/removal/)'`.
6. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain --all-targets
   -- -D warnings`; focused tests of `temper-engine-domain`; `heavy cargo
   check --tests` for `temper-service`, `temper-engine-domain-world`,
   `temper-client-world`, `temper-system-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; every file outside the crate you edited.
