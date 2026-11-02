+++
id = "goal-forge-story"
part = "goal-forge"
kind = "feature"
needs = ["goal-jig-holdings", "goal-forge-branch", "goal-forge-symbol"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs"]
effort = "high"
minutes = 75
+++

# goal-forge-story: a goal holding its own branch, end to end in temper's engine world

## Goal

The goal family's story, through temper's root, jig's core and the fake forge, in temper's
engine world (`temper/tests/engine/domain`): a tracked goal whose batch member names the
forge's *its own goal branch* becomes active holding `temper/<G>/g<G>`, made at the home's
default branch's tip; a change into that branch from outside the goal's subtree is refused
at its batch, naming the goal; a merge into it takes its writer slot; the goal's end keeps
the branch.

Acceptance, in a new module `forge_routes/goal_branch.rs` (declared in
`tests/suite/forge_routes.rs` beside `goal_tool`):

- `a_tracked_goal_naming_its_own_goal_branch_holds_it_made_at_mains_tip`: after adoption
  (`World::adopt`, `sign_in`), a person's `people::Ask::Create` with one tracked agent member
  (the goal template's shape: tracked, listed, a report) and `symbols: [Symbol { connector:
  <the forge connector>, symbol: 1 }]` makes task G; G's live record's holdings include a
  write hold on `temper/<G>/g<G>` in the home; the fake forge shows that branch at the
  default branch's tip (`World::external` with a branch read); G's first run is assigned.
- `an_untracked_task_naming_its_own_goal_branch_is_refused`: the same member untracked is
  refused (`people::Refusal::Limit { which: LimitKind::HoldKind }`), and no branch is made.
- `a_change_into_a_goal_branch_from_outside_its_goal_is_refused_naming_it`: with G active, a
  second person-made root (a chat) whose run makes a change into `temper/<G>/g<G>` (the
  change procedure's base, parameter 2, set to that branch: by the `change` tool's landing
  branch if the change family's `landing` parameter is on your base, otherwise by a
  person's `Create` of a change procedure task with that base) is refused, its problem
  naming G (`blocked_by: Some([G])`, `Refusal::HoldTaken`; or, through a person's batch,
  `LimitKind::HoldTaken`).
- `a_goals_end_keeps_its_own_branch`: G ends (its result, or its person's cancel, as the
  world's helpers allow); its holds are released and `temper/<G>/g<G>` still exists on the
  fake forge, at the tip it was made at.
- If the change family's `landing: "goal"` is on your base, also
  `a_change_by_the_goal_into_its_own_branch_lands_there_taking_its_writer_slot`: G's run
  calls `change` with `landing: "goal"`; the change is admitted, and its merge into g is in
  flight holding g's writer slot (a `Stored::Writer`-style row or the world's equivalent),
  then lands into g. Otherwise leave it out and name it under `## Left`.

## Inputs

- `temper/docs/design/domain/forge.md`, section 2 ("Holds", "Symbols"), section 11 and 11.1.
- jig's `jig/docs/design/domain/tasks.md`, 6.1 to 6.3, and `jig/docs/design/domain/
  connectors.md`, 3.1, 3.3 and 4.7.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C1, C2 and C3.

## Owns and may change

- Owns the files in the header. `temper/tests/engine/domain/tests/suite/forge_routes.rs` and
  `src/forge_routes.rs` are changed on plan reference-m8's trunk (`tiers-stories`): add the
  module line and any helper at the end, nothing else.
- **Options:** a helper the world lacks (a branch read on the fake forge, a tracked member
  with symbols) goes in `src/forge_routes.rs` as a small public function. If a story fails
  in a crate this plan's goal family changed, stop and report it (`Question:`) with the
  failing assertion and the log: the fix is that task's crate, never this test's.
- **Not yours:** every crate; the design.

## Steps

1. Read `forge_routes/goal_tool.rs` and `forge_routes/plain_change.rs` for the world's
   shapes, and `src/forge_routes.rs`'s `World`.
2. Write the stories; run each: `heavy cargo nextest run -p temper-engine-domain-world --test
   suite -E 'test(/forge_routes::goal_branch::/)'` (check the count).
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-engine-domain-world`.
4. Commit ("temper engine world: a goal's own branch, end to end"), push.

## Report

Each story and what it asserts, the tests and counts, and whether the inside change ran.
