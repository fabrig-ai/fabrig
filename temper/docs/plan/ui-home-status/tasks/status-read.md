+++
id = "status-read"
part = "status-read"
kind = "feature"
needs = ["status-wire", "project-facts"]
files = [
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/route/panels.rs",
  "temper/crates/temper-engine-domain/src/tests/looked.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/tests/engine/domain/tests/suite/panels.rs",
]
effort = "high"
minutes = 60
+++

# The root and the service carry the look and the document

## Goal

A client's `Looked` reaches the forge and its answer comes back; a client's watch of `status` gets its own document. `temper/docs/design/client/README.md`, 4.6 and section 8's row.

Acceptance:

- `notes/contracts.md`, *The root*, holds. `Application::Looked { project }` is admitted of `Project(project)`, whatever scope the record came under, from a party with a role there, in a project not being deleted, as `Dismiss` is.
- `family.rs`: `w::Request::Looked` to the application, replacing `status-wire`'s refusal; `w::Watch::Status(subject)` to `Watched::Application { scope, connector: wire::FORGE, subject: Token::new(forge::status_subject(subject.project())) }`, as `Watch::Changes`; `forge::Document::Status` to `wire::project_status`, replacing `project-facts`' `None`; `LookedAnswer::Looked` to `w::Outcome::Looked`, `TooLarge` to the forge refusal `Dismiss`'s takes.
- A watcher's document is its own party's; another project's subject is gone; no role, refused.
- `temper/docs/design/domain/forge.status.md`'s 5.6 entry and `client/README.status.md`'s words `the request Looked and the subject status` go.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 4.6, section 8's table; `temper/docs/design/domain/forge.md`, 5.6.
- The models: `Dismiss` in `route/applications.rs`, `boundary.rs`, `family.rs`; `Watch::Changes` in `family.rs` and `temper/tests/engine/domain/tests/suite/panels.rs`.

## Owns and may change

Owns, as `files` lists them. May change the status lines above.

Options, each pre-approved:

- **`temper-client-tools`' `connection.rs` and `lifecycle.rs`:** a `ctl` client neither sends `Looked` nor watches `status`: their arms stay as `status-wire` left them.
- **A `document` that can't be encoded** answers `None`, as the others do: bounds hold by the schema's ceilings, and a `branch` past 1024 bytes can't be, as `Repository`'s are bounded alike. If a test finds one, cut it with `text::fit`.

## Steps

1. **The root:** the application and its answer, `route/applications.rs`'s scope, push and answer arms.
2. **The service:** `family.rs`'s four places.
3. **Tests:** `tests/looked.rs`, shaped as `tests/dismiss.rs`: `a_look_is_of_the_project_it_names`, `a_party_with_no_role_is_refused`, `a_project_being_deleted_refuses_a_look`. `tests/family.rs`: `looked_and_status_translate_both_ways`, beside the `changes_subject` case at `:2805`. `suite/panels.rs`: `a_status_document_is_the_watchers_own`, `a_status_subject_for_another_project_is_gone`, `a_look_remakes_the_document`.
4. **Check,** the implementer's text, with the citations check, then push.

## Report

As `notes/common.md`, Your report.
