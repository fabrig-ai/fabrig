+++
id = "goal-story"
part = "story-red"
kind = "feature"
needs = []
files = [
  "temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs",
]
effort = "high"
minutes = 50
+++

# The failing story: a goal's run lands what it delivers

## Goal

Write the plan's story as one test in the engine's domain world, against the code as it is. It fails on the trunk at its first step, so it goes in `#[ignore]`d with a reason, and `goal-walks` lifts that once the code is there.

This departs from the implementer's text in one way: **no production code changes here, and the test you add does not pass.** It has no rule to break by hand. It is the plan's failing test, and the report names where it fails.

Acceptance:

- The test `a_goals_delivery_lands_into_its_goal_branch_by_a_change` exists in `goal_delivery.rs`, and it compiles against the trunk: it speaks `change` as JSON through the tool's decoder, so `ChangeBranch::Run` isn't named.
- Run by name with `--run-ignored only`, it fails at its first assertion, that the goal's charter grants `deliver`. The report quotes that failure.
- The ordinary suite's count of passing tests is unchanged.

## Inputs

- `temper/docs/plan/goal-delivery/notes/common.md` and `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 8.1, 8.5 and 11.1.
- `temper/docs/design/domain/agent.md`, 4.3 and 4.3.1.
- `temper/docs/design/draft/goal-as-designer.md`, 4.

Read these as the worlds to follow: `forge_routes/goal_landing.rs` (`mine`, `row`, `status`, `land_into_goal`, `assert_goal_holds`, `finish_run`), `forge_routes/goal_branch.rs` (`running_goal`, `branch_tip`, `writer`, `goal_request`), `forge_routes/early_start.rs` (a world with room for several producers).

## Owns and may change

- **Owns:** `temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs`, which you create.
- **May change:** `temper/tests/engine/domain/tests/suite/forge_routes.rs`, the one `mod goal_delivery;` line. `goal_landing.rs` and `goal_branch.rs`, a helper's visibility only (`pub(super)`).
- **Options:**
  - `running_goal`'s world is too small for the goal, a sibling and an early producer: build one in this file, as `early_start.rs` does with `StoryLimits { slots: Some(..), .. }`, and say so.
  - A later step can't be written without production code you don't have: write it as far as the world's helpers go, leave a one-line comment saying what the step holds, and name it in the report. `goal-walks` finishes it.

## Steps

The story, in this order, each step an assertion:

```rust
#[test]
#[ignore = "a goal's run delivers once goal-delivery lands: temper/docs/design/domain/forge.status.md"]
fn a_goals_delivery_lands_into_its_goal_branch_by_a_change() {
    // The goal's run, under a charter with modify, holding g.
    // 1. Its charter: grants.tools.modify; grants.deliver with checks_must_pass, fields title and body;
    //    outcome.change none; outcome.report some. (temper_engine_smith::charter(&goal))
    // 2. Its workspace: start ForgeStart::Base(g), push Some(r), r = temper/<root>/r<task>.
    //    Its writer is in r's slot, and none is in g's (goal_branch::writer).
    // 3. It delivers: the world makes r at g's tip and advances it (fake::advance), as a producer's push is
    //    made in goal_landing::producing_into_goal. Its head is H1. A hint lets the forge read it.
    // 4. While it runs, a sibling change lands into g (goal_landing::land_into_goal): merges go on.
    // 5. change {"words":"Land the design","branch":"run","landing":"goal"} is Delegated: C1.
    //    row(C1).branch is r; C1 has no delegate: no producer is made.
    // 6. change {"words":"Build on the design","landing":"goal","after":[C1]} is Delegated: C2.
    //    Its producer starts early: ForgeStart::Merged { branch: g, heads: [H1] }.
    // 7. C1 steps with the goal's run still in r's slot, and lands into g: State::Landed, and g holds H1's file.
    // 8. C1's close deletes nothing: r still exists at H1, and the goal still holds it (assert_goal_holds).
    // 9. The goal finishes with a report (finish_run, TaskResult::Report).
}
```

Then, once: `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`; run the test by name:

`heavy cargo nextest run -p temper-engine-domain-world --test suite --run-ignored only -E 'test(=forge_routes::goal_delivery::a_goals_delivery_lands_into_its_goal_branch_by_a_change)'`

Commit and push.

## Report

The failure as it reads, each step you could write whole and each you left as a comment, and the world you chose.
