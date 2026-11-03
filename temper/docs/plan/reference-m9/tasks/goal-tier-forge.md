+++
id = "goal-tier-forge"
part = "goal-tier"
kind = "feature"
needs = ["store-surface", "goal-tier-config", "red-news-tell"]
files = ["temper/crates/temper-engine-domain-forge/src/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tiers/words.rs", "temper/crates/temper-engine-domain-forge/src/tiers/goals.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers.rs", "temper/crates/temper-engine-domain-forge/src/tests/tiers/goals.rs"]
effort = "high"
minutes = 300
+++

# g's tier: a tier follows each goal's own branch, paced by its span, and tells its reds

## Goal

A tier following goal branches (`Follows::Goals`) follows each goal branch of its repository
held by a live goal, keeps one run at a time across them, starts a branch's run at most once
per its span, judges no landing, and tells the goal holding the branch what its rerun failed
again, each test marked new or still failing. Any tier with a span waits it between runs.

Acceptance (`notes/contracts.md`, section 6; `temper/docs/design/domain/after-landing.md`, 2,
3, 5.4 and 7):

- A new module `tiers/goals.rs` (its doc cites `temper/docs/design/domain/after-landing.md, 3`)
  holds what is the goal branches' own; `tiers.rs` calls it where its paths branch.
- **Followed:** `tiers::watched` and `add_watches` add, for each `Goals` tier, each name of its
  repository that a live goal holds and `items::goal_branch` parses; a hold's release drops
  that goal's `TierGoal` (where the forge drops the hold, it calls into `tiers`; a run in
  flight for that goal then ends with its words and tells nothing). `tiers::observed`, on a
  move of such a branch, makes or updates the goal's `TierGoal` (`tip`), appends no landing,
  and calls `start_due`. The first tip the forge records for the branch, its making at the
  default branch's tip as the hold is taken, is the entry's `ran`, never run: only a later
  move (a change landing into it) makes it due.
- **Due and started:** with no run in flight, the due entry (contracts.md, 6) with the oldest
  `started`, `None` first, then the lowest `goal`, starts: `TierRow.goal`, the entry's
  `started`, a run at its `tip` with no landings and no receipt (`p::start(.., Box::new([]),
  None, ..)`), numbered by `record.sequence` as any run. A due entry within its span sets the
  forge's deadline (`tiers::deadline`) at `started + span`, and `tiers::fire` starts it then.
  No receipt is made from a goal tier's run.
- **A tier following one branch with a span:** `TierRow.started` is set as each run starts; a
  move within the span waits on the same deadline, and the run then takes the newest tip.
  Without a span, nothing changes.
- **Finish:** for a goal's run, no `verification::judged`; the failures that failed again are
  the verdict's `failing`; `new` and `still` against the entry's `failing` as contracts.md, 6
  says; red tells the goal through `tiers/red.rs` (`RedOn::Goal { branch, tip, new, still }`,
  the run's number and url, the failed jobs' logs' ends), to the subscriber of
  `Topic::Landings { repository, branch }` whose `task` is the goal, `Class::Wakes` when `new`
  is non-empty or `Many`, `Class::Kept` otherwise; green tells nothing; then the entry's `ran`
  and `failing`, and `TierRow.goal = None`. Flakes are kept on the record as any run's.
- **Words:** `tier_specification` (`tiers/words.rs`) adds `Goal branch: BRANCH` after the
  repository line, for a goal's run alone.
- **Bytes:** `row_bytes` counts `goals`, `goal` and `started`.
- **Restart:** the row's `goals` and `goal` come back from the store; a run in flight resumes
  as any; a deadline pending at the restart is set again.
- Unit stories in a new `src/tests/tiers/goals.rs` (its `mod` line in `tests/tiers.rs`), on
  that file's helpers and `tests/tiers/red.rs`'s:
  - a change landing into `temper/1/g1`, held by goal 1, starts one run of the goal tier at its
    tip, and the change ends unfollowed;
  - a second move within the span starts no run; `fire` at the span's end starts one at the
    newest tip;
  - a red whose rerun fails a test again tells goal 1 `RedOn::Goal` with it `new`, waking; the
    next run failing it again tells it `still`, kept; a green run tells nothing;
  - two goals' branches: one run at a time, the second branch's after the first ends;
  - goal 1's hold released while its run is in flight: the run ends, nothing is told, and its
    entry is gone;
  - a goal's branch whose goal made it but no change landed into is never run;
  - a main tier with a span waits it between two moves;
  - a restart with a goal's run in flight resumes it and tells once.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, sections 1, 4, 5
  and 6.
- `temper/docs/design/domain/after-landing.md`: section 2 (goal branches, a tier's span), 3 (on
  a goal's branch), 5.4 (no verdict there), 7 (a red of a tier following goal branches), 10 (the
  store); `temper/docs/design/domain/forge.md`, 11.1 (a branch per goal, its hold).

## Owns and may change

- Owns the files of this spec's header for what acceptance names (`tiers.rs` whole: no other
  task of this plan edits it now).
- `temper-engine-domain-forge/src/domain.rs`: the one call into `tiers` where a goal's hold is
  released, and the deadline's wiring if `tiers::deadline`/`fire` do not already reach it;
  nothing else there.
- `tiers/red.rs` is `red-news-tell`'s: call its telling; a change it needs to serve
  `RedOn::Goal` (a parameter, a visibility) is allowed, its behaviour for a red after landing
  unchanged, and named in the report.
- `TierRow`'s fields are `store-surface`'s: add none. If the design needs one it lacks, stop
  and ask.
- The world stories (`temper/tests/engine/**`) are `goal-tier-stories`': not yours.

## Steps

1. `tiers/goals.rs`: following, the entries, due and the span; their stories.
2. `start_due`, `deadline`, `fire`, `finish` and the release hook; the words; `row_bytes`.
3. Tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/tiers/)'`, then
   `--lib` whole; `-p temper-engine-forge-world --test suite -E 'test(/tiers/)'`;
   `-p temper-engine-domain-world --test suite -E 'test(/forge_routes::tiers/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D
   warnings`; commit and push.

## Report

As the implementer's text asks; say how the span's deadline is set and kept across a restart.
