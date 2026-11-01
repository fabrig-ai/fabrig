+++
id = "jp-view"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-wire", "jp-close"]
files = ["jig/crates/jig-domain-core/src/acts.rs", "jig/crates/jig-domain-core/src/acts/**", "jig/crates/jig-domain-core/src/documents/offers.rs", "jig/crates/jig-client-protocol-wire/**", "jig/crates/jig-client-domain/src/**", "jig/crates/jig-client-protocol-link/src/**", "jig/crates/jig-protocol-engine-clients/src/**", "jig/crates/jig-client-view/src/**"]
effort = "high"
minutes = 80
+++

## Goal

Give a project's owners Delete on the project's settings in jig's
client, confirmed by typing the project's name (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "its client";
`jig/docs/design/client/domain/pages.md`, 6.5): the core offers the act
to the project's owners as it offers Rename, the offer crosses the wire,
and the view shows the control, its confirmation and its outcome.

Acceptance:

- The core's offer vocabulary `acts::Act::RemoveProject` (beside
  `Rename`, `acts.rs`), its standing an owner's (`acts/standing.rs`, as
  `Rename`'s), offered in the project's document to its owners
  (`documents/offers.rs`, line 134's list); the client contract's
  `ActKind::RemoveProject` and the wire's `ActKind` `remove_project`
  (`client-v19.schema`, in place), translated both ways where `rename`
  is.
- In `jig-client-view`, the project's settings (where Rename's owner
  form is: `page/project/owner_form.rs`, `sections/project/owner.rs`):
  a Delete control shown only when the project's document offers
  `RemoveProject`; it opens a confirmation (`confirm.rs` /
  `confirmation.rs`, as other owner acts confirm) that says what goes
  (the project, its tasks, chats and notes, at once; its spend stays in
  the deployment's totals) and is enabled only once the typed text equals
  the project's name (`people::same_name`'s rule, or the exact name if
  the client has no such rule: say which); confirmed, it sends
  `Ask::RemoveProject { project }`; its outcome `ProjectRemoved` shows in
  the owner outcome's words (`owner_outcome_words.rs`) and the client
  leaves the project's pages (the next document no longer lists it).
- Every exhaustive match over the new `ActKind` compiles in jig and
  temper (`temper-client-tools`' `admissions.rs`, `temper/tools/gallery`'s
  `worlds.rs`): forced arms.
- Tests: `jig-domain-core` unit test that an owner's project document
  offers `RemoveProject` and a member's does not; `jig-client-view` unit
  tests (beside `tests/owner_outcome.rs` and the owner form's): the
  control absent without the offer, the confirm disabled with a wrong
  name and enabled with the right one, the sent ask, the outcome's words;
  `jig-client-world`'s `suite` binary, one story in `settings.rs`: an
  owner deletes a project through the page and the project leaves the
  client's lists.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `jig/docs/design/client/domain/pages.md`, 6.5 (settings) and its
  confirmation rules; `jig/docs/design/client/domain/contract.md` (offers,
  `ActKind`).
- `tasks/jp-wire.md` (the plan directory) for the ask's client path.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  section 4, the `jig-client-view` readers.

## Owns and may change

- Owns the files in `files` (in `jig-domain-core`, the offer vocabulary
  and the project's offers only).
- May change: compiler-forced arms over `acts::Act` and the client
  `ActKind` anywhere in jig and temper; `jig/tests/client/tests/suite/settings.rs`
  and `jig/tests/client/src/**` for the story's helpers.
- Not yours: `jig-domain-core/src/removal/**`, `routing/**`.
- Options: if the client's pages hold their words in `phrases/` (as
  `phrases/english.rs` holds Rename's), add Delete's there. If a deployment
  owner who is not a project owner would be offered it by the standing
  rule, keep it to project owners (draft section 9, "who deletes") and
  say so.

## Steps

1. Offer vocabulary and the project's offers, with the unit test.
2. The wire's `ActKind` and its translations.
3. The view's control, confirmation and outcome, with unit tests:
   `heavy cargo nextest run -p jig-client-view --lib -E
   'test(/remove_project|delete/)'`.
4. The client world's story: `heavy cargo nextest run -p jig-client-world
   --test suite -E 'test(/settings/)'`.
5. `cargo fmt`; `heavy cargo clippy` naming every package changed, with
   `--all-targets -- -D warnings`; `heavy cargo check --tests -p
   temper-client-tools -p temper-gallery -p temper-client-view`; the
   focused tests of `jig-client-view`, `jig-client-domain`,
   `jig-client-protocol-link`, `jig-client-world`, `jig-domain-core`.
   Commit and push.

## Report

As `notes/common.md` says; name the name rule the confirmation uses.
