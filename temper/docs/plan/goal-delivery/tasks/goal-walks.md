+++
id = "goal-walks"
part = "story-green"
kind = "feature"
needs = ["goal-story", "goal-start"]
files = [
  "temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/domain/forge.status.md",
  "jig/docs/design/domain/connectors.status.md",
]
effort = "high"
minutes = 90
+++

# The story passes, and its walks: a second run, and a branch left holding what g lacks

## Goal

Lift the ignore on the plan's story and make it pass, then walk what it leaves out: a later run's delivery, a run branch left holding what g lacks, a delivery under an open change, a conflict, and the goal's close. This is where the plan's parts first meet, so a defect here is likely in the seams, and repairing it is this task's.

Acceptance, each a test in `goal_delivery.rs`:

- `a_goals_delivery_lands_into_its_goal_branch_by_a_change` runs with no `#[ignore]` and passes, every step in `goal-story` written whole, among them the early start at r's pushed head and C1 stepping with the goal's run in r's slot.
- `a_goals_second_run_delivers_from_g_with_its_run_branch_merged_in`: the first run delivers and its change lands. The goal parks and is woken. Its second run's start is `ForgeStart::Merge { branch: g, base: H1 }`, landing g, push r. It delivers, r fast-forwards to H2, and a second `"run"` change is admitted now that the first has ended, and lands. g holds both files.
- `a_run_branch_left_holding_what_g_lacks_reaches_the_next_run`: a run delivers and parks with no change. Its next run starts as above and makes the change, which lands what the first run delivered.
- `a_delivery_under_its_open_change_is_that_changes_new_head`: with C1 checking at H1 and the goal's run still going, a second delivery to H2 becomes C1's head, checked again, and C1 never merges H1.
- `a_conflict_on_a_run_branch_goes_to_the_merge_job_and_a_resolution`: g moves against r. With the goal's run still in the slot, the delegate waits. Once the run parks, it runs on r (`Writer::Ancestor`, holder the goal), and C1 goes on and lands. The goal's next run starts from what that left.
- `a_goals_close_takes_its_run_branch`: the goal finishes with a report, C1 having landed. The change's close left r in place, and the goal's close deletes it (`forge.md`, section 11).
- The status entries below are gone.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1, 8.5, 8.11 (*Early starts*), 11 and 11.1.
- `temper/docs/design/domain/agent.md`, 4.3 and 4.3.1.
- `temper/docs/design/draft/goal-as-designer.md`, 4.
- `temper/docs/plan/goal-delivery/notes/common.md` and `notes/contracts.md`.

Read as worlds to follow: `forge_routes/goal_landing.rs`, `goal_landing_held.rs` (a repair and a stay under a goal's landing), `early_start.rs`, `catch_up.rs` (the merge job).

## Owns and may change

- **Owns:** `forge_routes/goal_delivery.rs`, whole. It may change what `goal-story` wrote of the story's mechanics: steps may be moved and helpers added, but no step of the story dropped.
- **May change:**
  - `temper-engine-domain-forge/src/{domain,items}.rs`, `temper-engine-domain/src/route/forge_route.rs` (`change_branch`, `run_workspace` and `pending` in `forge_route/landing_route.rs`), `temper-engine-domain-forge-change/src/domain.rs`: **a repair of a seam the walks find,** in the fewest lines that hold the design, with a test in that crate. Name each in the report.
  - The story's early start: if it doesn't see r's head, and `run-branch` didn't take its option, take it here: the fresh read of a change on a run branch records the head in `d.heads` (`owned: false`) and wakes dependents as `branch_changed` does.
  - `temper/docs/design/domain/agent.status.md`, `forge.status.md`, `jig/docs/design/domain/connectors.status.md`: remove each **Not yet** entry naming `temper/docs/plan/goal-delivery/`, and nothing else. This departs from the implementer's text.
- **Stop and ask:** a walk that contradicts the design's words, where neither is plainly the slip. Say what the code does, what the section says, and your recommendation.
- **Options:**
  - The delegate's wait for r's slot passes the change's stall in the world's clock and it holds: release it in the walk, and report the numbers. That hold is the design's (`forge.md`, 8.5), not a fault.
  - A walk needs more slots or a longer step bound: its own world in this file, as `early_start.rs` does. Raise no limit in shared worlds.

## Steps

1. Lift the ignore, and run the story alone. Read its first failure, and keep going until it passes. Keep each repair's test beside its crate.
2. The walks, one at a time, each run alone before the next. Break the rule by hand for each: the writer rule in `change_facts` for the delivery under an open change; the merged start in `items.rs` for the second run.
3. The status entries.
4. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`, naming each production package you repaired.
5. `heavy cargo nextest run -p temper-engine-domain-world`, with `temper-engine-domain-forge`, `temper-engine-domain-forge-change` and `temper-engine-domain` if you repaired them. Then `heavy cargo nextest run -p temper-engine-domain-world --profile fuzzy`, since the slot's rule meets what the fuzzy walks hold.
6. Commit and push.

## Report

Each walk's rule broken by hand and its failure; each repair, with its crate and test; the stall's numbers if it held; and what the conflict walk saw of who ran.
