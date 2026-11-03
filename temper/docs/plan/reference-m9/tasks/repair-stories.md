+++
id = "repair-stories"
part = "stories"
kind = "feature"
needs = ["repairs-call", "repairs-verdict", "red-news-tell"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/repairs.rs", "temper/tests/engine/domain/tests/suite/forge_routes/tiers.rs", "temper/tests/engine/domain/src/tier_runs.rs"]
effort = "high"
minutes = 120
+++

# Stories: a red after landing reaches its goal, and a repair naming it clears it

## Goal

The root's world tells `temper/docs/design/draft/reference-landing.md`, section 9's third story
end to end, through the real forge connector and change procedure on the fake forge: a goal's
change lands into main, main's tier reddens it, the goal hears it as news, makes a repair into
main naming it, and the next run passing the failures clears the goal's landing, repaired by
the repair.

Acceptance (`temper/docs/design/domain/after-landing.md`, 7 and 12: "a red to its goal", "a
repair naming a red landing clears it"):

- In `temper/tests/engine/domain/tests/suite/forge_routes/repairs.rs` (made by `repairs-call`),
  on the tiers world (`temper/tests/engine/domain/src/tier_runs.rs`):
  - `a_red_after_landing_reaches_its_goal_as_news`: a goal (a tracked task holding no branch, or
    the world's chat where it is a goal's subtree) makes a change into main that lands; main's
    tier `slow` fails two tests at the new tip, its rerun fails them again; the goal's inbox
    holds one news message whose words start `Red after landing: tier slow run N blames landing
    HEX on main: 2 failing.`, list both ids, name `Run: ` with the fake forge's run url, and
    carry `End of job ` with the failed job's log's end; the goal wakes on it.
  - `a_repair_naming_its_red_clears_it`: from there, the goal's `change` into main with
    `repairs` naming the red landing's commit lands; the next run passes; the red change's
    result carries `Cleared by slow run N, which passed the failures blamed on it; repaired by
    HEX12.`, HEX12 the repair's merge commit's first twelve digits, and the repair's own result
    `Cleared by slow run N.` as today.
  - `a_repair_into_another_branch_is_refused`, if `repairs-call` did not already write it:
    skip it then.
- In `forge_routes/tiers.rs`: try `a_repaired_result_names_the_run_that_passed_its_failures`
  without its `#[ignore]`; if it passes with this plan's words, remove the ignore; if it fails
  for `tier_landing_run` (another plan's or a later fix's), leave it ignored and say so.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::(repairs|tiers)/)'`
  passes, within the tier's budget (`skein/docs/foundation/testing-strategy.md`, 8.1).

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, sections 2, 3 and 5.
- `temper/docs/design/domain/after-landing.md`, sections 7 and 12;
  `temper/docs/design/draft/reference-landing.md`, section 9 (the third story).

## Owns and may change

- Owns the stories named, in the files of this spec's header; `tier_runs.rs`: helpers (a goal,
  the news in a task's inbox, the fake run's url and job log) as the stories need them.
- A story that fails for code this plan's other tasks own (`repairs-call`, `repairs-verdict`,
  `red-news-tell`): mark it `#[ignore = "<the fault, file and function>"]`, report it under
  `## Left`, and do not fix that code.

## Steps

1. The world's helpers; the stories, one at a time, each run alone first
   (`-E 'test(=forge_routes::repairs::NAME)'`).
2. The ignored story's trial.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   commit and push.

## Report

As the implementer's text asks; give each story's wall time.
