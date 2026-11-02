# live-temper-stories-2: the plan

From `inputs/request.md` (11-02): fab's after-landing pair at main 6ce795f9d ran both
owned-forge landing stories through the landing and the box's restart, then failed both at
`temper/crates/temper/tests/live.rs` ~35 (`ci_shown`): after the restart the stories expect
`CI at its head\nNot read since the restart`, and the change's page shows `Passed`. One direct
part, one task; shared rules in `notes/common.md`.

## Which is right

The design moved, and the code with it; the live story's expectation is stale.

- `temper/docs/design/domain/forge.md, 8.8` (6a4fdc65ec, 10-27): a live change's CI stays in
  memory and a restart forgets it, but "a change that has landed or ended keeps its last
  verdict at its exact head with its record (`ChangeRow`'s `ci`, written by the save its
  finish already makes), restored with the row".
- The code (a2fa22067a, 10-27): the panel reads a terminal row's saved `ci`
  (`temper-engine-domain-forge/src/panel.rs` ~164), so a landed change's page shows `Passed`
  after a restart.
- The process tier's two restart worlds were brought to it the same day (714b51308d):
  `support/first_repository.rs`'s `assert_task` reads `CI at its head\nPassed` on both runs.
  The live story kept its own copy of the old check, `ci_shown`, which no gate runs.

So no repair of the page or its state: the stories come to the design. The design text the
request asks for is written:

- `temper/docs/design/domain/testing.md, 2.7`: the owned-forge landing story's bullet says the
  change's page shows `Passed` for CI at its head on both runs, kept with the landed change's
  record through the restart (forge.md, 8.8), as 2.6's restart stories read it; its status
  gains the **Not yet** entry the task removes.
- `temper/docs/design/client/README.md, 5.10`: the **Gates at its head** bullet still gave the
  old reason for "Not read since the restart" ("the panel forgets CI at a restart, and a
  landed change's is not read again"), which contradicted forge.md 8.8 and is the source of
  the story's stale comment. It now says a landed change keeps the verdict last read at its
  head through a restart, so it is unknown only when no step read one between the engine's
  last start and the landing. The word and the code are unchanged; no status gap.

## Parts and tasks

- **landed-ci** (`temper/docs/design/domain/testing.md, 2.7`), slice 1.
  - `tasks/landed-ci-kept.md`, medium, ~45 minutes, needs nothing: one shared check,
    `support::first_repository::assert_landed_ci(run, page)` (`CI at its head\nPassed`),
    read by the process worlds' `assert_task` and by both live stories on both runs; `ci_shown`
    goes. The process world
    `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` runs in the
    slot through `tools/tiers/run temper process`, with `temper`'s `suite` `forge_land::`
    harness tests and the focused set.

## Critical path

`landed-ci-kept` (~45 minutes, up to ~90 with the buffer); then fab's end check on the trunk,
and fab's after-landing pair on a main holding it, which is the request's proof. What follows
`ci_shown` in the second run (same identity, forge, tasks, delegates and replies after the
restart) has not run live yet: the pair observes it for the first time.

## Overlaps

None found. No neighbouring trunk or ready branch (`reference-m8`, `reference-m7a`,
`reference-m1`, `v0.1.0-ladder`'s `ready/*`) touches `crates/temper/tests/live.rs`,
`support/first_repository.rs`, `process.rs`, `domain/testing.md` or its status, or
`client/README.md` against main 0ca3509cbf.

## Calls made

- **One check for both tiers.** The drift came from two copies of the same assertion; the
  live stories now call the process worlds' `assert_landed_ci`, so the next change of meaning
  moves both.
- **No red-before test.** The request asks for one only if the page were wrong; it is not.
  The process world reads the shared check green before and after; the live pair is fab's.
- **No contracts note:** one part, no shape crosses it.
