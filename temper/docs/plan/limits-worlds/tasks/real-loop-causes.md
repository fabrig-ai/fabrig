+++
id = "real-loop-causes"
part = "real-loop"
kind = "feature"
needs = ["figures-before", "stories-go"]
files = [
  "temper/tests/real-loop/src/**",
  "temper/tests/real-loop/tests/suite/**",
  "temper/docs/plan/limits-worlds/figures/real-loop-causes.md",
]
effort = "high"
minutes = 150
+++

# What a real-loop story costs in its own harness

## Goal

The real-loop world is 85 CPU s of the fuzzy suite and 30 of the focused one, on one harness: its setup, its judge's iteration, its forge and LLM peers' hosting and its fixtures. Fix what `figures/figures-before.md` puts in `temper/tests/real-loop/src/**`, largest first, until the next one's gain is under about 2% of a story. Every story keeps every check it has.

Acceptance:

- `temper/docs/plan/limits-worlds/figures/real-loop-causes.md` holds, in `notes/measuring.md`'s form, instructions before and after for `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands`, `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` and `github_land::a_chats_change_lands_on_connected_github`, two runs per arm, and ends with what you stopped at.
- Each fix has its own commit, saying what it costs and what it saved.
- `heavy cargo nextest run -p temper-real-loop-world` and the same `--profile fuzzy` pass, with the same tests by name, and native CPU seconds for the whole world in each profile are in the file, before and after.
- No patience, deadline, bound or budget changed in what you commit: `git diff` holds none.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `temper/docs/plan/limits-worlds/figures/figures-before.md`: your list and your *before*. `stories-go` has landed since, so check that a story you read still exists.
- `temper/docs/design/domain/testing.md`, 2.4, 4.1 and 4.2; `skein/docs/foundation/testing-strategy.md`, 2.8, 2.10 and 8.
- `temper/tests/real-loop/src/world.rs`, `src/service.rs`, `src/forge.rs`, `src/fixtures.rs`.

## Owns and may change

Owns the files in the header. `temper/tests/real-loop/tests/fixtures/**` stays as it is: `model-catalogue-2/process-fixtures` holds it. A cause under this world is `engine-causes`' or another task's: name it under `## Left`, with its figures.

Where to look first, and what you decide. The profile decides whether any of these is the cause:

- **What the judge does on every iteration:** `service.seen.collect(...)` in `world.rs` reads the forge's branches and open pulls on each pass, through the forge's own API, to record a snapshot only when they've changed. It may be that a change marker on the forge's side, or `pop_observation`, says when to look. Every snapshot a story reads stays.
- **Setup rebuilt:** `green_light::fixture()`, `landing::fixture()` and `repair::fixture()` are called again from one another and for each field; each decodes its documents and builds a tree. `Shape::write` in `cluster.rs` reads and writes each worker's documents by `serde_json`. Build each once per story.
- **A poll where an event would do:** a `Deadline` or a fixed wake in the world's hosts, or a call's retry. Remember that a poll looks bigger under callgrind (`notes/measuring.md`).
- **A real program where a fixture would do:** count the spawns of `sh` and what they do. The fake git program is in-process, and the design wants it so (`temper/docs/design/domain/testing.md`, 4.3).

Options, each decided:

- **A wake that exists to bound a real-clock wait:** the patience rule keeps it (`skein/docs/foundation/testing-strategy.md`, 2.10). Don't remove it; say what it costs.
- **Shortening the path by giving a story fewer turns or acts:** that's a change to what it checks. Don't, unless a check it has doesn't read them, and say so.
- **`temper_fake`'s documents, or a kit under `temper/testing/**`,** at the top of the profile: `engine-causes`' by `notes/common.md`, and `## Left` here.
- **Timings you might be tempted to shrink** (a fixture's `tasks.retries`, a bounded wait): they're a test's settings. A story's shorter setting that keeps its path is the design's way (`temper/docs/design/domain/testing.md`, 5.3, "Time keeps the case's order, not its ratio"), and allowed where the story's own row says it's only the wait. A profile's seconds of idle cost no CPU anyway: only spend time on this if the CPU moves.

## Steps

1. Re-profile the three stories on your base, as `notes/measuring.md` says.
2. Fix the largest, re-profile, repeat.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`; the runs in the acceptance; `python3 -I -B tools/citations/citations.py --base origin/main`; restore `PATIENCE`, delete `target/limits-worlds/`, commit and push.

## Report

Instructions and CPU seconds before and after, each cause and what it saved, what you left and whose it is, and your implementation and check minutes apart.
