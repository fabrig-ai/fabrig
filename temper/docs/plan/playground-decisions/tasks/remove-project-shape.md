+++
id = "remove-project-shape"
part = "deletion-grace"
kind = "feature"
needs = ["remove-project-grace"]
files = [
  "jig/crates/jig-client/src/shapes.rs",
  "jig/crates/jig-client/src/tests/shapes.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "medium"
minutes = 25
+++

# `shapes::remove_project` fills a deletion's grace from the chat template

## Goal

A client gives a project's `RemoveProject` the chat template's grace, as `me` offers it now, as
`shapes::close_chat` gives a close's (`inputs/request.md`, decision 3; `notes/contracts.md`, C2).
jig's helper does it once, so no client derives it again. No client offers Delete yet; the shape
is the place the rule is code.

Acceptance:

- `jig_client::shapes::remove_project(me, project)` returns `Ask::RemoveProject { project, grace }`,
  `grace` the chat template's (`template(me, project, CHAT)`) or `None` once the template is gone;
  a project not in `me` gives `Err(NotOffered::Project)`.
- Its three unit tests below pass; the status entry below is gone.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md`.
- `jig/docs/design/client/domain/client.md`, 19 (`shapes::remove_project`, beside
  `shapes::close_chat`).
- `jig/docs/design/domain/people.md`, 5.2 (the paragraph on the deletion's grace).

## Owns and may change

- `jig/crates/jig-client/src/shapes.rs`, beside `close_chat`:

  ```rust
  /// A project's deletion with the chat template's grace, as a close gives one
  /// (client/domain/client.md, 19; domain/people.md, 5.2).
  pub fn remove_project(me: &Me, project: u32) -> Result<Ask, NotOffered> {
      // NotOffered::Project when no membership names it, as close_chat does;
      // grace: template(me, project, CHAT).and_then(|template| template.grace)
  }
  ```

  If `close_chat`'s membership check reads better as a shared private helper, factor it; its
  behaviour stays.
- `jig/crates/jig-client/src/tests/shapes.rs`, beside `close_chat_cancels_the_chat_last_with_its_template_s_grace`,
  with the file's `me()` fixture and `SHAPE_GRACE`:

  ```rust
  #[test]
  fn remove_project_gives_the_chat_template_s_grace() {
      // Ok(Ask::RemoveProject { project: PROJECT, grace: Some(SHAPE_GRACE) })
  }
  #[test]
  fn remove_project_gives_no_grace_once_the_chat_template_is_gone() {
      // me() with template CHAT removed from PROJECT's membership: grace None
  }
  #[test]
  fn remove_project_of_a_project_not_in_me_is_not_offered() {
      // Err(NotOffered::Project)
  }
  ```

- Status, as `notes/common.md` says: `jig/docs/design/client/domain/client.status.md`,
  **Not yet**: remove ` 19: \`shapes::remove_project\`: \`jig/crates/jig-client/src/shapes.rs\`
  has no shape for a project's deletion.` and nothing else.

## Steps

1. The shape and its tests: `heavy cargo nextest run -p jig-client --lib -E 'test(shapes::)'`.
2. The status edit; the citations check.
3. `cargo fmt`; old forms for jig; `heavy cargo clippy -p jig-client --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client`. The function is new, so no reader changes meaning.
   Commit the code and tests, then the status text; push.

## Report

As the implementer's text says.
