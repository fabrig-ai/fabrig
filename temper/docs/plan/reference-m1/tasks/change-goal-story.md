+++
id = "change-goal-story"
part = "change"
kind = "feature"
needs = ["change-words", "goal-forge-symbol", "goal-forge-branch", "goal-forge-story"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_changes.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 90
+++

# change-goal-story: a goal's changes land into its own branch, one after another

## Goal

One story runs both real sides of the goal's own branch and `change`: a tracked goal G
holding its branch g (the goal family's tasks in `needs`) makes two changes into g in one
`change` call, the second after the first; each opens its pull request into g, waits for no
CI and takes no update under the goal branches' rule, and merges at its head; each result
carries its producer's report; G ends and g is kept. This is the slice M1's walk follows
(steps 3, 7 and 19 of `temper/docs/design/draft/reference-case.md`, in their plainest form).

Acceptance: the story below passes, with every story of `forge_routes`; no code changes.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 ("What a plan decides").
- `temper/docs/design/domain/forge.md`, 8.2 steps 3, 6 and 7; 8.7 ("What the procedure asks
  of the rules"); 11 and 11.1 (the goal's own branch, as the goal family wrote them).
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C1, C3, C4, C5, C6.

## Owns and may change

Owns the files in the header: a new story file and its `#[path =
"forge_routes/goal_changes.rs"] mod goal_changes;` line. Helpers the goal family's story
added (making G from a template naming its own goal branch, and reading g's hold) are used,
never copied; a helper that is private to their file may be made `pub(super)` there, that
edit alone.

Options: if the code fails the story, stop at the first failing assertion: the fix is not
this task's. Report it as `Done:` with the story marked `#[ignore = "<the fault>"]` and the
fault under `## Left`, naming the task whose code it is.

## Steps

1. `a_goal_makes_two_changes_into_its_branch_the_second_after_the_first`, in
   `temper/tests/engine/domain/tests/suite/forge_routes/goal_changes.rs`:
   - The world's deployment landing rules, set as `tests/suite/settings/b2.rs` sets them: main's
     rule on the repository's `branch` segment with the last segment `main` exact, CI passed
     and up to date, enforced; the goal branches' rule on the same segments with the last
     segment `temper` open, asking neither, enforced.
   - G: a tracked goal holding g (the goal family's way), its authority with the control
     family; active, g made at main's tip.
   - G's run calls `change` with `{"changes":[{"name":"a","words":"Add A","landing":"goal"},
     {"words":"Add B","landing":"goal","after":["a"]}]}`: two change tasks in one batch, B's
     dependency A.
   - A's producer pushes and finishes with words `Done: added A.\n## Left\n- none`; A opens
     its pull request into g; its CI at the head stays pending; it merges at its head as a
     merge commit; its result words hold `Landed: pull request #N merged into temper/R/gR`,
     then `Report:` and those words.
   - main moves on meanwhile; B starts only after A is done, its producer pushes, and B merges
     at its head with no update effect.
   - G finishes; g is kept (the branch still exists, no delete effect).
2. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/goal_changes::/)'` (1). Then `heavy cargo nextest run -p temper-engine-domain-world
   --test suite -E 'test(/forge_routes::/)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D
   warnings`. Commit ("Show a goal's changes landing into its own branch") and push.

## Report

As the implementer's text says.
