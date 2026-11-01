+++
id = "pd-client-view"
part = "pd"
kind = "feature"
needs = ["pd-client-wire"]
files = [
  "jig/crates/jig-client-view/**",
  "jig/tests/client/**",
]
effort = "high"
minutes = 100
+++

## Goal

jig's client offers **Delete project** on a project's settings page (`/projects/N/settings`) to
an owner when the engine offers it, confirmed by typing the project's name (contracts J7).

Acceptance:
- The Settings section shows a **Delete project** control when the owner document's acts
  offer `ActKind::RemoveProject` as allowed (`OwnerProject.acts`, as `owner_form::service_offered`
  reads `MakeService`), and nothing when it is not offered.
- Pressing it opens a confirmation in the page (a new state of `owner_form::Form`): words that
  say what goes ("Deleting {name} ends its tasks and erases them, its notes, roles and
  adoptions. Spend already made stays in the totals."), a text field asking for the project's
  name, **Delete project** and **Dismiss**. **Delete project** is disabled until the typed text
  equals the project's current name (`OwnerProject.name`, compared exactly, byte for byte);
  a new owner document with another name disables it again until it matches the new one.
- Sending asks `Ask::RemoveProject { project }` through the page's intents, as the settings
  form sends (`owner_form::send`, `IntentPurpose::ProjectSettings`); while pending the control
  shows it and sends nothing more.
- On the answer `ProjectRemoved`, if the page is still the one that sent it
  (`handle == domain.page_handle()`, as `page/project/goals.rs` `answered` checks), the page
  leaves for the root address `Address::Inbox { project: None }`, whose frame lists the
  projects (jig's client has no projects-list address; the root is its nearest), with
  `page::routing::navigate`; otherwise a kept notice says the project was deleted. A refusal
  shows on the confirmation as the form's refusals show; an unknown answer as
  `owner_form::unknown`.
- Node and size budgets (`sections/project/owner.rs` `max_nodes_with_card`, `required`) cover
  the new parts.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J7.
- `temper/docs/design/draft/playground-reset.md`, section 5 ("Its client").
- `jig/docs/design/client/domain/contract.md` (the owner project document, its acts).
- The code: `page/project/owner_form.rs` (`Form`, `review`, `send`, `answered`, `unknown`),
  `sections/project/owner.rs` (`section` ~531, `forms` ~822, `service_form` ~1176,
  `review_form` ~1040), `binding.rs` (~208, typed input to an act), `ui.rs` (~553, ~631),
  `page/conversations.rs` (~149, answer dispatch), `page/project/goals.rs` (~252-297, leaving
  after an answer), `page/routing.rs` `navigate`.

## Owns and may change

Owned: `jig/crates/jig-client-view/**` and `jig/tests/client/**`. Nothing else: the contract
and wire are `pd-client-wire`'s.

Options, pre-approved:
- New phrases take the next free ids of the view's phrase tables; say which.
- If `owner_form::answered` cannot reach `navigate` without the env, handle and output, pass
  them in as the `CreateGoal` arm of `page/conversations.rs` does.

## Steps

1. `owner_form.rs`: the delete state (`typed`, `pending`, `refusal`), `delete_offered`,
   `open_delete`, `type_name`, `send_delete`, the answer arm.
2. `sections/project/owner.rs`: the control and the confirmation, beside `service_form`;
   bindings and acts for opening, typing, sending, dismissing.
3. Tests:
   - `src/tests/owner_project.rs` (beside the service-form tests ~746, ~1001):
     `delete_is_offered_only_when_the_engine_offers_it`,
     `delete_stays_disabled_until_the_typed_name_matches_exactly` (a prefix, a case change,
     a trailing space: disabled; the exact name: enabled),
     `a_renamed_project_disables_a_matching_confirmation_again`,
     `deleting_sends_one_request_and_leaves_for_the_root_on_its_answer`,
     `a_refused_deletion_shows_on_the_confirmation`.
   - `jig/tests/client/tests/stories/projects.rs`: `an_owner_deletes_a_project_by_typing_its_name`
     (beside `an_owner_renames_a_project_and_me_says_the_new_name`).
   Commands: `heavy cargo nextest run -p jig-client-view --lib -E 'test(/delete/)'`,
   `heavy cargo nextest run -p jig-client-world --test stories -E 'test(/deletes_a_project/)'`,
   then `heavy cargo nextest run -p jig-client-view --lib` and
   `heavy cargo nextest run -p jig-client-world --test suite`.
4. `cargo fmt`; `heavy cargo clippy -p jig-client-view -p jig-client-world --all-targets`;
   old forms; commit and push.

## Report

The phrase ids used; the address the page leaves for.
