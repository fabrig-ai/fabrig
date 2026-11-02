+++
id = "tiers-stories"
part = "tiers-stories"
kind = "feature"
needs = ["tiers-root", "tiers-verification", "tiers-reports", "panel-steps"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/tiers.rs", "temper/tests/engine/domain/src/tier_runs.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/tests/engine/domain/src/lib.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 240
+++

# The engine's stories: a small change into main, verified by main's tier

## Goal

The engine, root and forge together, on the fake forge, walks what the milestone's walk will:
a chat's small change lands into main, main's tier runs on the new tip, and the change's result
comes at cleared; a red run reruns, blames and bisects, and the blamed change stays red until a
later landing's run passes its failures. These are after-landing.md 12's stories at the
engine's level.

Acceptance (`temper/docs/design/domain/after-landing.md`, sections 3 to 6 and 12;
`temper/docs/design/draft/reference-landing.md`, sections 6 and 7):

- A new story file `temper/tests/engine/domain/tests/suite/forge_routes/tiers.rs` (with support
  in `temper/tests/engine/domain/src/tier_runs.rs`: a world whose repository has one tier,
  `slow`, following main, its workflow failing tests by id from a file at the commit
  (`Cue::Tests`) and uploading a JUnit report), each story driven through the root as
  `forge_routes/landing.rs` and `forge_routes/result.rs` drive theirs:
  1. **green:** a chat's change lands into main; its task lives (`verifying`, its panel says so)
     until the tier's run passes; then it ends with its result at cleared, the `Cleared by slow
     run N.` line after its `Landed:` line; the tier task ended done with its words;
  2. **two landings, one run:** two changes land while one run is in flight; the next run covers
     both and clears both;
  3. **red, bisected:** three changes land in one range, one changing nothing the tier reads;
     two tests fail from the second's commit: the rerun confirms them, the probes (oldest first,
     the unchanged one skipped) blame the second, whose change goes `red`; the others end
     cleared;
  4. **repaired:** a later change lands whose run passes those two tests: the red change ends
     cleared, its result naming the run;
  5. **the bound:** the runner never reports (`silent`) past `verification_bound_ms`: the change
     ends landed and uncleared, saying so;
  6. **not followed:** a change into another branch of the repository (a goal's own branch,
     `temper/1/g1`) ends at landed at once, as today, and starts no run;
  7. **by turn:** story 1 where the project lands by turn.
- Each story asserts the change's panel steps through the panel's reads (`verifying`, `red`,
  `cleared`) and the tier task's words; seeds replay (`tests/clippy.toml`'s rules).
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
  'test(/forge_routes::tiers::/)'` passes (seven or more), then the whole
  `-p temper-engine-domain-world --test suite`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`; `notes/contracts.md`, 2 and 4.
- `temper/docs/design/domain/after-landing.md`, sections 3 to 6 and 12;
  `temper/docs/design/draft/reference-landing.md`, sections 6 and 7 (the walk's story);
  `skein/docs/design/fake-forge.md`, 2.3 (the fake's reports).
- The world: `temper/tests/engine/domain/src/forge_routes.rs` (the world's forge, workflows about
  686, limits; a landing project's configuration about 463 and 546 to 560), the stories in
  `tests/suite/forge_routes/{landing,result,plain_change}.rs`.
- The old code: `temper/docs/plan/reference-m8/notes/old/temper-2a-tests.diff`
  (`tests/engine/domain/src/tier_runs.rs`, `tests/forge_routes/landing.rs`'s tier stories) and
  `temper-t9-8140d6c33.diff` (the bound's story).

## Owns and may change

- Owns the files in the header. `forge_routes.rs` (src) only to add the tier's configuration
  and workflow to the world's options; the suite's `forge_routes.rs` only to add `mod tiers;`.
- Not this task's: any crate's code. A story that fails for a fault in a crate is reported, with
  the seed and the log, as `## Left` naming the crate and the owning task's area, and marked
  `#[ignore = "..."]` with that reason; never fixed here.

Pre-approved options:

- If the panel's reads are not reachable from this world, assert the change's state through
  the forge's rows as `forge_routes/result.rs` does, and say so.
- If story 7 needs a by-turn world this one lacks, build it as `forge_routes/landing.rs` does.

## Steps

1. The world's tier and workflow; then the stories one by one, each run alone first.
2. The suite whole.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`.
4. Commit (why: the milestone's story at the engine's level); push your branch.

## Report

As the implementer's text says, and also: each story's seed and simulated time; any story
ignored, why and whose.
