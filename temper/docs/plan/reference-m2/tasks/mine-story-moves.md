+++
id = "mine-story-moves"
part = "mine"
kind = "feature"
needs = ["mine-start", "mine-forge", "goal-release"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_landing.rs", "temper/docs/design/domain/forge.status.md", "temper/docs/design/domain/after-landing.status.md"]
effort = "high"
minutes = 110
+++

# mine-story-moves: the goal's landing takes g's moves as its new head; G's close deletes g

## Goal

End-to-end stories in the root's engine world for what `mine-start`, `mine-procedure`,
`mine-forge` and `goal-release` built apart: a change into g landing while L is checked is
L's new head (`temper/docs/design/domain/forge.md`, 8.5); a move of g by another identity is
drift and holds L; the goal closing after L landed deletes g, and closing after L was
cancelled keeps it (section 11; `temper/docs/design/domain/after-landing.md`, section 6).

Acceptance, as stories appended to `forge_routes/goal_landing.rs` (created by `mine-start`):
- `a_change_into_the_goal_branch_while_its_landing_is_checked_is_its_new_head`: G's plan
  change landed into g; L made (`branch mine`) and checking, CI pending at g's tip A; G makes
  a second change into g (`landing goal`) that lands at B; L's head is B, CI is read at B,
  never held `BranchMoved`, its repairs 0; CI passes at B; L lands at exactly B.
- `a_move_of_the_goal_branch_by_another_identity_holds_its_landing_for_drift`: with L
  checking, an outside push to g (`world.external` / the fake forge's raw git as another
  identity, as `reviews.rs`'s `observed_move` does) holds L for its branch moved, naming the
  move; nothing is merged.
- `a_goal_closing_after_its_landing_deletes_its_branch`: L landed (its result given: cleared
  or past the bound as the world's tiers allow); G finishes; its release deletes g (a
  `DeleteBranch` of g), and G closes.
- `a_goal_closing_after_cancelling_its_landing_keeps_its_branch`: L cancelled before its
  merge; G finishes; g is kept, no `DeleteBranch` of g.
- `a_goal_branch_read_sent_before_a_merge_into_it_does_not_hold_its_landing`, the old
  branch `o/forgejo-land-drift-world`'s idea for 8.5, carried here: with L checking, a read
  of g is sent and held unanswered (delay that one branch's reads, as the old world did with
  a `slow_branch: Option<Box<[u8]>>` knob that kept `Read::Branch` of that branch pending
  twenty seconds; use the current world's way to delay one read, `rg 'slow_|hold_.*answers'
  temper/tests/engine/domain/src`); a change into g lands meanwhile; the stale read then
  answers the old tip; L is not held and ends at the new head. If the world offers no way
  to delay one branch's reads, adding one (a field on the world, its read path) is
  allowed; if that would change more than the world's read path, skip this story and say so
  under `## Left`.
- Every `forge_routes` story passes.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 ("A change on its requester's held branch"), 8.5
  ("On its requester's held branch"), section 11 ("Deleted as their tasks close").
- `temper/docs/design/domain/after-landing.md`, section 6.
- `temper/docs/plan/reference-m2/notes/contracts.md`, C2 to C5.
- `temper/docs/plan/reference-m2/notes/common.md`.
- `temper/docs/design/domain/forge.status.md`: its **Not yet** "8.1, 8.5, 11: a change on
  its requester's held branch: ...", and `after-landing.status.md`'s "a goal's landing holds
  its goal's branch as a change holds its own (...)", both of which this task removes once
  its stories pass.

## Owns and may change

Owns the files in the header (appending to `goal_landing.rs`; `mine-start`'s stories stay as
they are).

- The suite's helpers in `goal_branch.rs`, `goal_changes.rs`, `change_tool.rs` and
  `reviews.rs` may become `pub(super)`: visibility only.
- The world's crate (`temper/tests/engine/domain/src/`): a knob delaying one branch's reads,
  for the stale-read story alone, if the world has none.
- A story that fails on code the plan's tasks wrote: if the fault is in one of
  `temper-engine-domain-forge-change`'s phases, the forge's `drift`/`change_facts`/
  `workspace_item`, the root's `start_change`/`change_batch`, or `goal_branch::landed`, fix
  it there in the least lines, add a unit test beside that code's, and say so in the report;
  anything else is a story marked `#[ignore = "<the fault in one line>"]` and listed under
  `## Left`.

## Steps

1. The stories, one at a time, each run alone first:
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(=forge_routes::goal_landing::<name>)'` (check it selected 1).
2. Then `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::/)'`.
3. Remove the two **Not yet** entries above. `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-world
   --all-targets -- -D warnings` (and each package you fixed). Commit ("stories: the goal's
   landing takes its branch's moves; the goal's close deletes it") and push.

## Report

As the implementer's text says.
