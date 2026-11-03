+++
id = "mine-story-held"
part = "mine"
kind = "feature"
needs = ["mine-story-moves"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_landing_held.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 90
+++

# mine-story-held: the goal's landing held past its repairs lands after a change into g

## Goal

`temper/docs/design/draft/reference-landing.md`, section 9's second story, in the root's
engine world: the goal's landing L, its CI red, repaired three times (each repair pushing g
under G's hold), is held past its repairs and escalated to G; G makes a change into g that
lands; G releases L; L checks the new head and lands (`temper/docs/design/domain/forge.md`,
8.4 and 8.5; `reference-landing.md`, section 3, "Bounds").

Acceptance, as one story (and its helpers) in a new
`forge_routes/goal_landing_held.rs`:
- `a_goals_landing_held_past_its_repairs_lands_after_a_change_into_its_branch`:
  - the world's landing rules as the reference's (`temper/docs/examples/reference/landing.json`
    after task `reference-landing-config`): main asks CI and up to date, no gates; goal
    branches neither; the change policy's repairs 3
    (`World::configured_limits` or the override the suite uses, `rg 'change_policy'
    temper/tests/engine/domain`);
  - G holds g; G's plan change lands into g; G makes L (`branch mine`);
  - CI fails at L's head three times; each time a repair task is delegated by L, its
    workspace starts from g, it pushes g (assert g's writer slot is the repair's while it
    runs and G's hold of g stays G's), and CI is read again at the new head;
  - CI fails a fourth time: L is held for its repairs, saying so; G hears it (its escalation
    in G's inbox, or G woken with it: assert what the world shows);
  - G makes a change into g (`landing goal`) that lands; L, held, does not move;
  - G releases L (G's run's `release` call if the world drives one; otherwise a party's
    `people::Ask::Release` as `held_change.rs` does; say which); L takes g's tip as its head,
    CI is read there and passes; L lands at exactly that head.
- Every `forge_routes` story passes.

## Inputs

- `temper/docs/design/draft/reference-landing.md`, section 3 ("Bounds") and section 9.
- `temper/docs/design/domain/forge.md`, 8.1 ("A change on its requester's held branch"), 8.4
  (a repair; "Bounded apart"; "A release") and 8.5 ("On its requester's held branch").
- `temper/docs/plan/reference-m2/notes/contracts.md`, C3 to C5.
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns the files in the header: `forge_routes.rs` takes one `mod goal_landing_held;` line
beside `mod goal_landing;`.

- The suite's helpers in `goal_branch.rs`, `goal_changes.rs`, `goal_landing.rs`,
  `change_tool.rs`, `ci_poll.rs` and `reviews.rs` may become `pub(super)`: visibility only.
- A fault the story finds in the plan's code (the forge-change phases, the forge's
  `workspace_item`, `claim`, `drift` or `change_facts`, the root's `start_change`): fix it in
  the least lines with a unit test beside it, and say so in the report. A fault elsewhere
  (jig's escalation, the world): mark the story `#[ignore = "<the fault in one line>"]`,
  list it under `## Left` with what the story showed, and push.

## Steps

1. The story, step by step, run alone as it grows: `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E
   'test(=forge_routes::goal_landing_held::a_goals_landing_held_past_its_repairs_lands_after_a_change_into_its_branch)'`
   (check it selected 1).
2. Then `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::/)'`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings` (and each package you fixed).
   Commit ("story: the goal's landing held past its repairs lands after a change into its
   branch") and push.

## Report

As the implementer's text says.
