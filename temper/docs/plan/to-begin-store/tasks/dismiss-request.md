+++
id = "dismiss-request"
part = "engine"
kind = "feature"
needs = ["dismissed-family", "dismiss-wire"]
files = ["temper/crates/temper-engine-domain-forge/src/dismissed.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/adoption.rs", "temper/crates/temper-engine-domain-forge/src/tests/dismissed.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/applications.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/tests/dismiss.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs"]
effort = "high"
minutes = 50
+++

# The request `Dismiss`, and a project's rows erased with it

## Goal

A party with a role in a project sends `Dismiss`: the row commits, the subject `dismissed` is marked
and the request is answered; a second is answered as the first and writes nothing. A project's
release erases its rows in its decision (contracts K4). The document is `dismissed-read`'s: until it
lands the mark wakes nobody.

Acceptance:

- K4's `DismissAnswer`, `Answered::Dismiss`, `Event::Dismiss`, `DISMISSED_SUBJECT`,
  `Application::Dismiss` and `ApplicationAnswer::Dismiss` exist.
- `to_root` turns `w::Request::Dismiss` into `root::Application::Dismiss`, `answer` turns
  `ApplicationAnswer::Dismiss` into K4's outcomes, replacing the forced arm `dismiss-wire` left.
- `applications::start` names the project of `what` as the scope, whatever scope the record came
  under: a party with a role in project 1 who sends `Dismiss { ToBegin { project: 2 } }` under the
  scope of project 1 is refused by jig, with no row.
- In `temper-engine-domain-forge/src/tests/dismissed.rs`:
  `a_dismissal_saves_its_row_marks_and_answers`,
  `a_second_dismissal_is_answered_as_the_first_and_writes_nothing`,
  `a_dismissal_at_the_bound_answers_too_large_and_writes_nothing`,
  `a_project_released_erases_its_dismissals_and_marks`,
  `a_release_with_no_dismissals_marks_nothing`.
- In a new `temper-engine-domain/src/tests/dismiss.rs`: `a_dismissal_is_of_the_project_it_names`,
  `a_party_with_no_role_is_refused_before_the_connector`,
  `a_project_being_deleted_refuses_a_dismissal`,
  `a_dismissal_is_answered_only_once_its_row_is_durable` (the pattern of
  `tests/setup.rs`, `a_new_binding_answers_only_after_its_commit_is_durable`).
- In `temper-service/src/tests/family.rs`, the request and both outcomes in the tables that hold
  `RemoveForge`'s.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-service`
  pass, with `-E` narrowing `temper-service` to `family`; `heavy cargo check --tests -p
  temper-client-world -p temper-engine-forge-world -p temper-engine-protocol-forge-world -p
  temper-one-process-world` pass.

## Inputs

- `temper/docs/plan/to-begin-store/notes/common.md`; `notes/contracts.md`, K1, K3 and K4.
- `temper/docs/design/domain/forge.md`, 5.5.
- `temper/docs/design/client/README.md`, 4.6 ("`Dismiss`") and 8 (the table's row).
- `temper/docs/design/protocol/store.md`, 2 ("A person's dismissals").
- jig's `jig/docs/design/domain/connectors.md`, 5.5 ("its keyed requests") and
  `jig/docs/design/domain/people.md`, 5.2 (a project being deleted).

## Owns and may change

- The files of the header. In `adoption.rs`, `release`: one call, beside `erase_project_settings`.
- `Application` and `Answered` matches, each its forced arm alone:
  `temper-engine-domain/src/route/playground.rs` (`forge::Answered`),
  `temper-engine-domain/src/admission.rs` (`forge::Event::Dismiss`, joining
  `forge::Event::Application`'s arm), `temper-engine-domain/src/domain.rs`,
  `temper-engine-domain/src/store.rs`, the worlds' matches under `temper/tests/engine/` and
  `temper/tests/client/src/`.
- `temper-engine-domain/src/tests.rs`: the `mod` line.
- `applications::admitted` takes `person`: `route/asks.rs` passes the core ask's field. It is the one
  caller.
- Pre-approved:
  - `Connect`'s retained answers (`d.connect.answer`) hold `Answered::Dismiss`, as `RemoveForge`'s
    does: it pushes `Request::ApplicationDone`, and the answer is memory alone. Keep it so.
  - If `a_project_being_deleted_refuses_a_dismissal` finds jig does not refuse the application
    under a project being deleted, stop and ask: that row would outlive the release.

## Steps

1. K4's domain types and the event; `dismissed.rs`'s two functions: the decision of
   `temper/docs/design/domain/forge.md`, 5.5, and the release's erasure, which marks once when any
   row went.
2. The root: the two enums, `start`, `admitted`, `answer`.
3. `family.rs`, both ways: `w::Dismissal` to `forge::Dismissal`; the outcomes.
4. The tests, each biting: drop the held-row check, drop the release's call, and name the record's
   scope in `start`, each once.
5. The implementer's checks; commit and push.

## Report

As the implementer's text asks; say which refusal `a_party_with_no_role_is_refused_before_the_connector`
saw.
