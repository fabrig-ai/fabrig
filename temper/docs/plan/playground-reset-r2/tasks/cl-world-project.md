+++
id = "cl-world-project"
part = "client"
kind = "feature"
needs = ["jp-delete", "cl-project", "cl-world-playground"]
files = [
  "temper/tests/client/tests/suite/project_settings.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/**",
]
effort = "medium"
minutes = 45
+++

## Goal

A story of temper's project settings page against the real engine
(draft 5, "Its client"): an owner deletes a project, confirmed by its
name; its chats end and leave the lists, the page leaves for the chats,
and the project is gone from `Me`; another party sees no Delete.

Acceptance (`temper-client-world`, new `tests/suite/project_settings.rs`):

- `an_owner_deletes_a_project_confirmed_by_its_name` (a project with a
  live chat: Delete, the name typed, confirmed; the chat ends; the page
  at `/chats`; `Me` lists the project no more; the forge's repository
  rows of the project erased, as `rm-project` does on
  `ProjectRemoved`).
- `delete_waits_for_the_exact_name` (a wrong name sends nothing).
- `another_party_sees_no_delete`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 1.
- `temper/docs/plan/playground-reset-r2/tasks/jig-ask.md`, `cl-project.md`
  and D1's deletion task (`jp-delete`, in `tasks/`).
- `temper/docs/design/draft/playground-reset.md`, 5.

## Owns and may change

Owns the new suite file and its `mod` line in `tests/suite/main.rs`; in
`temper/tests/client/src/` only what the story needs (a fixture with a
second party or an owned project; the world's in-process jig core
already serves `RemoveProject`). Never edits jig or the engine.

## Steps

1. The fixture: the world's owner, a project with one adopted repository
   and a live chat (as the suite's adoption stories make them), and a
   second party who is a member, not an owner.
2. The three stories above, through the fake person on
   `/projects/N/settings`.
3. Checks: `cargo fmt`; `heavy cargo clippy -p temper-client-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/project_settings/)'`, then
   the package's focused tests.
4. Old forms from `temper/`; commit and push.

## Report

- How long the deletion took to end in the world's time, and whether the
  page saw `ProjectRemoved` or the project leaving `Me` first.
