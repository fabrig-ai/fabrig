+++
id = "pd-client"
part = "project-deletion"
kind = "feature"
needs = ["pd-wire"]
files = [
  "jig/crates/jig-client-domain/src/requests.rs",
  "jig/crates/jig-client-domain/src/tests/remove_project.rs",
  "jig/testing/jig-client-scripted-engine/src/requests/projects.rs",
  "jig/testing/jig-client-scripted-engine/src/tests/remove_project.rs",
  "jig/tests/client/tests/stories/projects.rs",
]
effort = "high"
minutes = 40
+++

## Goal

jig's client's side of a request to delete a project (draft 5, "Its
client"): a client sends `Ask::RemoveProject { project }` through its
request table as it sends `MakeProject`, keeps it by its key, and
hears `ProjectRemoved`; after it the client's `me` and documents no
longer name the project. The scripted engine (the client worlds'
engine) answers it as the engine does once `pd-close` lands: the
project's owner gets `ProjectRemoved` and the project leaves `me`; a
member who is no owner is refused as the engine's `project_admission`
refuses. temper's Delete button, confirmed by the project's name, is
the playground designer's, on this surface.

Acceptance: tests below; `pd-wire`'s *not offered* answer in the
scripted engine is gone.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4).
- `jig/docs/design/client/domain/root.md`, section 4 (the request
  table), `jig/docs/design/client/domain/contract.md` (`me`, owner
  outcomes).
- The model: `MakeProject` and `Rename` in `jig-client-domain`
  `requests.rs` and the scripted engine's `requests/projects.rs`
  (`project_request`, `change_project`, `rename_changes`).
- `pd-wire`'s report (`## Interfaces`).

## Owns and may change

Owned: the `files` above. Neighbours: the scripted engine's
`requests.rs` (its dispatch arm), `truth.rs` (a project dropped from
its truth), `payload/owner_outcome.rs` (the outcome's payload), its
`tests.rs` (`mod` line); `jig-client-domain`'s `tests.rs` (`mod`
line); `jig/tests/client/src/**` helpers, an arm or helper each.

Pre-approved: if `me` is pushed by a watch the scripted engine
re-renders on a project change, use that path; if the client domain
needs no change beyond `pd-wire`'s for the request to be kept, say so
and leave `requests.rs` but its test.

## Steps

1. `jig-client-domain` `requests.rs`: `Ask::RemoveProject` kept and
   retried as `Ask::MakeProject` is; its answer an owner outcome.
2. Scripted engine `requests/projects.rs`: `RemoveProject` for the
   project's owner removes the project from its truth (its roles, its
   documents, `me`) and answers `ProjectRemoved { project }`; a
   non-owner is refused as the engine refuses; an unknown project as
   an unknown project is.
3. Tests:
   - `jig-client-domain` `src/tests/remove_project.rs`:
     `a_remove_project_request_is_kept_by_its_key_and_answered_once`;
   - scripted engine `src/tests/remove_project.rs`:
     `an_owner_removes_a_project_and_it_leaves_me`,
     `a_member_who_is_no_owner_is_refused`;
   - `jig-client-world` `tests/stories/projects.rs`:
     `an_owner_deletes_a_project_and_it_leaves_the_project_list`.
4. Checks: `cargo fmt`; clippy on `jig-client-domain`,
   `jig-client-scripted-engine`; their `--lib` tests; `heavy cargo
   nextest run -p jig-client-world --test suite -E
   'test(/projects/)'`; `heavy cargo check --tests -p jig-client-view
   -p temper-client-domain -p temper-client-world`. Commit, push.

## Report

fab's sections; `## Interfaces`: the request's path through the
client domain; what the scripted engine's truth drops.
