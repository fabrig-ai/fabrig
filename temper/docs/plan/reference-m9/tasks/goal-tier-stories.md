+++
id = "goal-tier-stories"
part = "stories"
kind = "feature"
needs = ["goal-tier-forge", "repair-stories"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_tier.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/tier_runs.rs"]
effort = "high"
minutes = 120
+++

# Stories: g's tier checks a goal's branch along the way

## Goal

The root's world tells `temper/docs/design/draft/reference-code-production.md`, section 10 end
to end on the fake forge: a goal holding its own branch, a tier following goal branches with a
span, changes landing into the branch and cleared as they land, a red told to the goal as news,
marked new then still failing, while merges go on, and a repair into the branch turning it
green.

Acceptance (`temper/docs/design/domain/after-landing.md`, 3, 5.4, 7 and 12: "g's tier"):

- A new `temper/tests/engine/domain/tests/suite/forge_routes/goal_tier.rs` (its `mod` line in
  `forge_routes.rs`), on the tiers world with a second tier, `goal`, `Follows::Goals`, span 20
  minutes (`tier_runs.rs`'s world gains the option):
  - `changes_into_a_goal_branch_are_cleared_as_they_land`: a goal made with its own goal branch
    makes two changes into it with `"landing": "goal"`; both land within the span, each result
    ends at `Landed` with no tier line, and one run of `goal` starts once the span has passed,
    at the branch's newest tip (the world's clock advanced); main's tier starts no run.
  - `a_red_of_the_goal_tier_reaches_the_goal_as_news`: the run fails a test and its rerun fails
    it again; the goal's inbox holds news starting `Red on temper/R/gG: tier goal run N at
    HEX12: 1 new, 0 still failing.` with `new: <id>`, waking it; a third change lands meanwhile
    (merges go on); the next run, failing the same test, sends news with `still failing: <id>`,
    kept, not waking.
  - `a_repair_into_the_goal_branch_turns_it_green`: the goal's repair change into its branch
    lands; the next run passes; no news is sent.
  - `a_goals_close_ends_its_branchs_following`: the goal ends with a run in flight on its
    branch; the run ends with no news.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::goal_tier/)'`
  passes, within the tier's budget (`skein/docs/foundation/testing-strategy.md`, 8.1).

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, sections 5 and 6.
- `temper/docs/design/domain/after-landing.md`, sections 2, 3, 5.4, 7 and 12;
  `temper/docs/design/draft/reference-code-production.md`, section 10.

## Owns and may change

- Owns the files of this spec's header for the stories named; `tier_runs.rs`: the world's
  second tier and helpers, keeping its existing callers' behaviour.
- A story that fails for code this plan's other tasks own (`goal-tier-forge`, `red-news-tell`):
  mark it `#[ignore = "<the fault, file and function>"]`, report it under `## Left`, and do not
  fix that code.

## Steps

1. The world's option and helpers; the stories, one at a time, each run alone first.
2. `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::(goal_tier|tiers|repairs)/)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   commit and push.

## Report

As the implementer's text asks; give each story's wall time.
