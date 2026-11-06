+++
id = "goal-real-story"
part = "real-story"
kind = "feature"
needs = ["goal-start"]
files = [
  "temper/tests/real-loop/tests/suite/goal_delivery.rs",
]
effort = "high"
minutes = 80
+++

# The real sides meet: real smith, a real worker and the engine

## Goal

The domain world's story has the fake forge's push stand in for a run. This one runs the rest for real, in `temper-real-loop-world`, which is focused: a scripted model edits a file in a real checkout, calls `deliver`, a real worker pushes to the run branch, and the model calls `change`, whose change lands into g. It proves what no domain test can: that smith admits this charter and offers `deliver` beside a report, and that the worker pushes where the engine named.

Acceptance:

- `a_goals_run_delivers_and_lands_through_a_change` in `temper/tests/real-loop/tests/suite/goal_delivery.rs` passes, by name, in the focused profile. Its scripted goal run calls in order: `edit` of a file; `deliver` with a title and a body; `change` with `{"words":"Land the design","branch":"run","landing":"goal"}`; `finish` with `form: report`, after the change has landed (a `wait` before it, as the world's stories wait).
- It asserts:
  - the run branch `temper/<root>/r<task>` holds the edit;
  - the goal branch `temper/<root>/g<task>` holds it after the change lands, read from the fake forge;
  - `deliver`'s answer is a delivery, naming the run branch in its receipt (`temper/docs/design/domain/agent.md`, section 6);
  - the task ends done with its report;
  - nothing reached `main`: `main_file` of the edited path is what it was.
- It stays within the focused profile's budget. Measure it, and report its time.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3, 4.3.1 and section 6.
- `temper/docs/design/domain/forge.md`, 8.1 and 11.1.
- `smith/docs/design/domain/run.md`, 8.1, 8.2 and 8.4.
- `temper/docs/plan/goal-delivery/notes/common.md`.

Read as worlds to follow: `temper/tests/real-loop/tests/suite/adoption_hold.rs` (`capture_forge`, `branch_file`, `tool(b"deliver", ...)`, and its `#[ignore]`d chat story, which is the same shape); `temper/tests/real-loop/src/fixtures.rs` (the charter with `modify`), `src/forge.rs` (`Fixture.configure`).

## Owns and may change

- **Owns:** `temper/tests/real-loop/tests/suite/goal_delivery.rs`, which you create.
- **May change:**
  - `temper/tests/real-loop/tests/suite/main.rs`: the `mod` line.
  - `temper/tests/real-loop/src/**`: a helper the story needs, added beside the existing ones. `model-catalogue-2/process-fixtures` writes `temper/tests/real-loop/tests/fixtures/**`, so stay out of that directory.
- **Not here:** the ignored chat story in `adoption_hold.rs` stays ignored: a chat's run is Open, and never delivers. Production code: a defect here goes to the report's first line as a `Question:`, unless it is under the option below.
- **Options:**
  - **Making a goal with its own branch:** no real-loop story has one yet. The goal's template carries the symbol: copy the shape of template 4 in `temper/docs/examples/reference/settings.json` (its `symbols` and its report contract) into this story's `Fixture.configure`, with `ended_by: result`, and start the task from that template as `temper help task` says. Change no shared fixture's defaults. If the settings path can't reach it, make the batch through the ctl's task creation, and say so.
  - **The project's checks:** the fake forge's files carry no mode (`skein_fake_forge_domain::api::File`), so an executable `.temper/pre-pr` can't be seeded this way. Leave it out, and name in the report that checks at delivery stand on `checks_must_pass` in the charter (`charter-delivery`, `goal-start`) and on smith's own tests. If the world can make one executable, add one that records it ran, and assert that.
  - **A small defect at the seam** (a limit in the world's fixture; a word the receipt assertion needs loosened): repair it and say so. One in smith's or the worker's production code is a question.

## Steps

1. Write the story, run it by name, and read where it fails. Its first failure on a trunk with `goal-start` should be in the fixture, not the grant: say which it was.
2. `heavy cargo nextest run -p temper-real-loop-world --test suite -E 'test(=goal_delivery::a_goals_run_delivers_and_lands_through_a_change)'`.
3. **A new test must bite:** break the grant once (`start.rs`'s `charter.delivers` false) and see the story fail because `deliver` isn't offered, then restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`.
5. Commit and push.

## Report

Its time; how the goal got its branch; what you did about checks; what the receipt said, word for word; and the bite.
