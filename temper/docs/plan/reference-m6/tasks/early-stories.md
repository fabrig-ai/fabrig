+++
id = "early-stories"
part = "early"
kind = "feature"
needs = ["early-route"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs", "temper/tests/engine/domain/src/**"]
effort = "high"
minutes = 90
+++

# early-stories: the case's early starts, as the root's stories

## Goal

Stories of the reference case's early starts (`temper/docs/design/draft/reference-case.md`,
steps 8, 10, 13 and 17) and of 8.11's other rules, run through the root with its fakes, so
each rule is shown whole: when a producer starts early, the once rule, the refusal on a
conflict, the next producer merging the landing tip first, the refused amendment and a
dependency into another repository.

Acceptance: the stories below pass in
`temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`, beside task
`early-route`'s, and every story of the world's `forge_routes` passes. Each is one thread
and one loop, and a seed replays.

- `a_dependent_waits_for_its_dependencys_first_push`: C3 made after C2 while C2's producer
  has pushed nothing: C3 is active and kept with no producer, and past the change's stall it
  is not held; C2's first push starts C3's producer early.
- `a_dependency_kept_by_its_own_dependency_holds_the_early_start`: C1, C2 after C1, C3 after
  C2; C2 starts early on C1's head and pushes; C3 has no producer while C2 is kept by C1;
  C1 lands; C3's producer starts on C2's head.
- `an_early_start_that_conflicts_waits_for_its_dependency_to_land` (step 17): C8's
  producer's preparation fails as a conflict naming C7's head; C8's producer is cancelled,
  nothing is held or escalated to G; C7 lands; C8's next producer starts from g's tip
  (`Base`), and C8 lands.
- `a_failed_early_producer_waits_and_the_next_merges_the_landing_tip` (step 13): C6's early
  producer pushes a head, then fails; nothing is held; C4 lands; C6's next producer starts
  from a merge in progress, C6's branch with g's tip as its base; it pushes and C6 lands.
- `an_early_producer_done_while_kept_waits_then_merges` (step 10): C3's producer finishes
  while C2's still produces; C3 is not ready; C2 lands; C3 merges with no further run.
- `an_amendment_dropping_a_dependency_started_from_is_refused_naming_it`: G amends C3's
  dependencies to none: refused, naming C2; G cancels C3 and makes it again: admitted.
- `a_dependency_into_another_repository_does_not_wait_for_a_push`: a change into a second
  repository after a change into the first, both pending: its producer starts at once from
  its own landing branch (`Base`), and no dependency is marked started from.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11; `temper/docs/design/domain/landing.md`, 4.1.
- `temper/docs/design/draft/reference-case.md`, section 4 (Act 2: steps 8, 10, 13 and 17),
  for the stories' shape only.
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C5 and C6.

## Owns and may change

Owns the files in the header. A story that finds a fault in the root's or the forge's early
start (task `early-route`'s code: `temper-engine-domain/src/route/forge_route*`,
`temper-engine-domain-forge/src/**`, `temper-engine-domain-forge-change/src/**`) may fix it
there, the fix alone, with a forge unit test or a pure test that shows it, and says so in the
report. A fault elsewhere is a question.

- Options: how the world fakes a conflicting preparation: as its preparation stories do
  (`temper/tests/engine/domain/tests/suite/preparation.rs`), with the worker's answer
  carrying `Preparation::Conflicted`; add a helper to the world's `src/` if it has none. A
  story the world cannot tell without a new world mechanism beyond such a helper is a
  question.

## Steps

1. The stories, one at a time, each run alone first.
2. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D
   warnings` (and each package you fixed); the world's `forge_routes` stories and the
   focused tests of each package you fixed. Commit, push.

## Report

As the implementer's text says; name each fault found and fixed.
