+++
id = "reset-grace"
part = "deletion-grace"
kind = "feature"
# playground-restart-resets edits the line after this task's in setup.status.md.
needs = ["remove-project-grace", "kept-names", "playground-restart-resets"]
files = [
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/tests/engine/domain/tests/suite/reset.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 45
+++

# The playground's reset deletes its project with the chat template's grace

## Goal

The user decided (`inputs/request.md`, decision 3) that a deleted project's conversations are
cancelled with their close's grace, the chat template's. jig's core reads no template, so the
application fills the grace (`notes/contracts.md`, C2): temper's root, translating the forge
connector's `ResetAct::RemoveProject { project }` into jig's act, gives it the current grace of
temper's chat template, template 1, as it reads template 2 for a goal
(`temper-engine-domain/src/route/goal.rs:110`). The forge connector's `ResetAct` stays as it is.

Acceptance:

- `route::playground::reset` sends `people::Act::RemoveProject { project, grace }`, `grace`
  being `domain.core.current.template(CHAT_TEMPLATE)`'s grace, or `None` when the settings hold
  no template 1.
- The number is named once in the crate, `const CHAT_TEMPLATE: u32 = 1;` in
  `route/playground.rs`, its doc citing `temper/docs/design/protocol/configuration.md, 2`
  (temper's templates: chat, template 1). temper's engine crates link no `jig-client`, so
  `jig_client::shapes::CHAT` is not used here.
- The world story below passes; the status entry below is gone.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md` and `notes/contracts.md`, C2.
- `temper/docs/design/protocol/setup.md`, 8.4, "Its reset", step 1 (`project`), and "Kept until
  whole".
- `temper/docs/design/protocol/configuration.md`, 2: "Templates are jig's two defaults".
- `jig/docs/design/domain/people.md`, 5.2 (the paragraph on the deletion's grace) and 7
  ("Closed with its project"), read only.

## Owns and may change

- `temper/crates/temper-engine-domain/src/route/playground.rs`: `CHAT_TEMPLATE` and `reset`'s
  `ResetAct::RemoveProject` arm (`:291`), which `remove-project-grace` left as `grace: None`.
  Nothing else: `acted` already moves the row on for every refusal but *not offered* and
  lacking the role, so a key conflict after a restart with a changed grace passes the step, as
  setup.md, 8.4 says.
- `temper/tests/engine/domain/tests/suite/reset.rs`, a new story with the file's helpers
  (`driver`, `start`, `row`, `application`):

  ```rust
  #[test]
  fn the_reset_deletes_the_project_with_the_chat_templates_grace() {
      let (mut d, sign_in) = driver(true);
      start(&mut d, sign_in);
      let project = row(&d).expect("running playground").project.expect("playground project");
      application(&mut d, sign_in, 82, forge::Ask::EndPlayground, false);
      // Apply the store's pending writes one at a time, as `run` does, until one saves
      // engine::Record::Core(core::Record::People(people::Stored::Acted { act, .. })) with
      // act == people::Act::RemoveProject { project, grace }; assert
      // grace == Some(Duration::from_millis(5000)), the world's chat template
      // (temper_engine_domain_world::fixtures::templates).
  }
  ```

  If the act's saved row is erased before you can read it (the deletion ends in the same
  pass), read the write as it is applied, not `d.store.rows`.
- Status, as `notes/common.md` says: `temper/docs/design/protocol/setup.status.md`, **Not yet**:
  remove ` 8.4: the reset's \`RemoveProject\` carries no grace, where it carries the chat
  template's: temper's root fills none (\`temper-engine-domain/src/route/playground.rs\`,
  \`reset\`).` and nothing else on the line; touch no other line but the date.

## Steps

1. `CHAT_TEMPLATE` and `reset`; `heavy cargo nextest run -p temper-engine-domain --lib -E
   'test(playground)'`.
2. The world story: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(reset::)'`.
3. The status edit; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-domain-world --all-targets -- -D warnings`. The reset's act now carries a
   grace, so run its readers' tests: `heavy cargo nextest run -p temper-engine-domain -p
   temper-engine-domain-world -p temper-engine-protocol-forge-world`. Commit the code and test,
   then the status text; push.

## Report

As the implementer's text says.
