+++
id = "forge-routes"
part = "forge-routes"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_changes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_landing_held.rs", "temper/tests/engine/domain/tests/suite/forge_routes/landing.rs"]
effort = "medium"
minutes = 70
+++

## Goal

Three ignored forge-route stories of the engine-domain world run again
in the focused suite. By reading, none waits on product work: one
asserts a merge commit no design asks of the fake, one closes before the
landing news its merge caused has been heard, and one waited on jig work
that is on main.

Acceptance:

- In `temper/tests/engine/domain/tests/suite/forge_routes/` (package
  `temper-engine-domain-world`, binary `suite`), these lose their
  `#[ignore]` and pass:
  - `goal_changes.rs`:
    `a_goal_makes_two_changes_into_its_branch_the_second_after_the_first`;
  - `goal_landing_held.rs`:
    `a_goals_landing_held_past_its_repairs_lands_after_a_change_into_its_branch`;
  - `landing.rs`: `a_change_created_with_a_dependency_joins_before_production`.
- `goal_changes.rs` is rewritten to the design as it stands (below),
  every other assertion of the story kept.
- `heavy cargo nextest run -p temper-engine-domain-world` passes.

## Inputs

- `temper/docs/plan/ignored-engine/notes/common.md`: every rule there
  holds.
- `temper/docs/design/domain/forge.md`, 4.4 (the merge style shown at
  adoption: the repository's), 11.1 (a branch per goal), 16 (Restart:
  startup's fresh reads find what moved).
- `skein/docs/design/fake-forge.md`, 2.2: the fake's merges are squashes
  at an exact head, one commit on the base's tip.
- `jig/docs/design/domain/connectors.md`, 6.1, and its status (no 6.1
  gap left): a declared procedure made with a pending dependency starts
  kept (`jig-domain-tasks/src/domain.rs`, `dependencies_ready`; temper
  declares its change procedure in `temper-engine-domain/src/domain.rs`).
- `temper/docs/design/domain/testing.md`, 4.2 (the fake forge) and 5.2
  (the referee).

## Owns and may change

- Owns the three stories and the private helpers of their own files.
- **`goal_changes.rs`, the landing's shape.** The `assert_landed`
  helper's `assert_eq!(commit.parents.as_ref(), &[base, head], ...)`
  asks a merge commit, which no design asks: the product merges by the
  repository's style (`MergeHow::Style`, forge.md 4.4), and the fake's
  style is a squash (fake-forge.md 2.2). The ignore's "C5" is the
  reference case's change of that name
  (`temper/docs/design/draft/reference-case.md`, rows C5 and 12), whose
  merge job merges g into the change's own branch; it asks nothing of a
  landing's parents. Rewrite the assertion to what the design gives: the
  landing is one commit whose parents are `[base]` (g's tip before it),
  its tree the producer's head's tree, read through the fake's commit
  read the helper already makes (compare the commits' tree ids if the
  fake's `Commit` carries one; otherwise read the paths the producer
  wrote at both). Its message says why, citing fake-forge.md 2.2 and
  forge.md 4.4. Keep the `Write::Merge { head, how: Style }` assertion,
  `pull.commit == head` ("lands without an update"), the pending CI
  status, the branch tip and the goal's hold.
- **`goal_landing_held.rs`, the close.** `World::close` crashes the
  store and compares the restarted state with the one before
  (`temper/tests/engine/domain/src/forge_routes.rs`, `close`; its
  oracle compares `Deployment.messages`). The landing's merge moved
  main, and the story closes before the forge has heard main move;
  the restart's fresh read then publishes landing news, which takes a
  message number. Before `world.close()`, do what the sibling
  `goal_landing.rs` does: take `news_count` for the goal, send a
  `forge_top::Event::Hint` for main's branch, and
  `await_new_forge_news` (both in the parent module,
  `tests/suite/forge_routes.rs`, reached as `super::`).
- **`landing.rs`:** the `#[ignore]` alone, unless it fails.
- Options, each pre-approved:
  - If `goal_changes.rs`'s second change then conflicts or needs an
    update because the first was squashed (its head is no longer an
    ancestor of g's tip), and g's rule asks no update, the fake's
    three-way merge decides: if it conflicts, the story is wrong
    about the design only if the design says the second lands without
    an update; read forge.md 11.1 and 9, follow what they say, and say
    so. Never change the fake's merge for this story.
  - If `goal_landing_held.rs` still differs at close after the news is
    heard, read the oracle's diff: a row the restart rebuilds from the
    forge's fresh reads that the design says it rebuilds (forge.md 16)
    is heard first, as above; a durable record that differs is a product
    fault, fixed by its cause in `temper-engine-domain` or
    `temper-engine-domain-forge`, with a row in that crate's own tests,
    and named in the report. Never normalise `Deployment.messages` in
    the oracle.
  - If `landing.rs`'s story fails, the cause is read against
    connectors.md 6.1 and forge.md 9 (a change kept by a dependency):
    a jig fault is reproduced in jig's tests (`jig/tests/core` or
    `jig-domain-tasks`' own) and fixed there; a temper fault in
    `temper-engine-domain/src/route/forge_route.rs`'s `start_change` or
    `forge_route/landing_route.rs`, with a row in that crate's tests.
- `temper/tests/engine/domain/src/forge_routes.rs`: added helpers only,
  placed beside the helpers they resemble; the `restore-stories` task
  may also add one there.

## Steps

1. Remove the three `#[ignore]` lines; commit. Run each story alone
   (count 1) and record its first failure or its pass.
2. Rewrite `goal_changes.rs`'s landing assertion; run it alone.
3. Hear main's move before `goal_landing_held.rs`'s close; run it alone.
4. Fix `landing.rs`'s story if it failed; run it alone.
5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain-world [-p each product
   package you changed] --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-engine-domain-world [-p each product package
   you changed and its direct dependents]`.
6. Commit, run `python3 -I -B tools/citations/citations.py --base
   origin/main`, push.

## Report

The implementer's report, with: each story's first failure as it stood
(or its pass), the cause found and the fix, the oracle's diff if one
was read, and each story's time.
