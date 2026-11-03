+++
id = "bounds-measured"
part = "bounds"
kind = "feature"
needs = ["playground-world", "one-process-story-a"]
files = [
  "temper/tests/playground/tests/memory.rs",
  "temper/tests/playground/Cargo.toml",
  "temper/tests/one-process/tests/memory.rs",
  "temper/tests/one-process/Cargo.toml",
  "temper/docs/development/playground-memory.md",
  "temper/docs/design/playground.status.md",
  "Cargo.lock",
  "temper/crates/temper-playground/src/story.rs",
  "temper/tests/one-process/src/world.rs",
  "temper/tests/one-process/src/fixtures.rs",
]
effort = "high"
minutes = 150
+++

# The playground world's memory, measured

## Goal

The playground is bounded by its documents, each just big enough for its stories; "measured
first: what reserves the world's memory at its first step and through each story, and which of it
the documents control" (`temper/docs/design/playground.md`, 8). This task measures, natively, and
writes the figures down; it sets no bound, and the memory's bar is the user's (`playground.md`,
11).

Acceptance:

- `temper/tests/playground/tests/memory.rs`, a test binary of its own declaring
  `#[global_allocator] static HEAP: skein_heap::Counting` (so no other binary is metered):
  `story_as_world_at_its_first_step_and_start`, story A's harness-free world
  (`temper_playground::world`), trace off, measuring each process's held and peak heap at its
  first step and once the world waits idle after the start (`skein_heap::Meter`/`Span` around the
  steps, or `skein_world::Memory::Checked`'s peaks if the world can be built on `skein_world`),
  and the fake disk's bytes apart (they live outside the processes' spans).
- `temper/tests/one-process/tests/memory.rs`, likewise its own binary:
  `story_a_at_its_high_water`, story A walked as `one-process-story-a` walks it, with
  `skein_world::Memory::Checked`, trace off, reporting each process's peak (`Outcome::peaks()`).
  If `Checked`'s release check fails for a process because the harness's captures are charged to
  it (the service's `Observe`), measure that process by `skein_heap::Span` around its iterations
  instead and say so; do not change the harness to suit the meter.
- Attribution: for each of the documents' knobs below, the measurement run again with the knob
  halved, and each peak that moves noted: skein-kv's `budget`, `commit` and `queued_bytes`
  (`skein_kv::Limits`, from jig's Small profile), the engine's and the worker's `memory_bytes`,
  smith's `llm_pool` and read window (the agent's profile), the fake LLM's and the fake forge's
  limits (`temper_playground::fakes`), the client's body bound. A knob halved past what the story
  needs fails the story: note the failure as the knob's floor.
- Both tests are `#[ignore = "a measurement, run by hand: temper/docs/development/playground-memory.md"]`
  and print their table with `--no-capture`; they assert only that each run measured something
  (a peak above zero). No ceiling is asserted: the bar is the user's.
- `temper/docs/development/playground-memory.md`: the date (`date -u +%F`), the commit measured,
  the machine (`unknown` if not known), the commands run; per process, the first step, the start
  and story A's high water; the fake disk; per knob, its value, whether it moved a peak, and its
  floor; and what reserves memory that no document controls.
- The status entry below is edited.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md`.
- `temper/docs/design/playground.md`, 8 and 11.
- `skein/docs/foundation/programming-model.md`, 6.3 (no process-wide worst case; nothing compares
  a memory declaration with a sum at start), read only.
- `skein/testing/skein-heap/src/lib.rs` (`Counting`, `Meter`, `Span`) and
  `skein/testing/skein-world/src/heap.rs` (`Memory`), read only.

## Owns and may change

- The two test files; each crate's `Cargo.toml` gains `skein-heap` (and `skein-world` for the
  playground world, if used) as a dev-dependency, and a `[[test]]` entry only if the crate's
  layout needs one for a second binary.
- `temper/docs/development/playground-memory.md`, new.
- Status, as `notes/common.md` says: `temper/docs/design/playground.status.md`, **Not yet**:
  `8: the documents' bounds, measured.` becomes `8: the documents' bounds, each just big enough
  for its stories (measured: \`../development/playground-memory.md\`).`, the line breaks kept
  where they are.
- Nothing in `temper-playground` or the harness changes; a read the measurement needs that they
  do not give is a question.

## Steps

1. The playground world's binary; `heavy cargo nextest run -p temper-playground-world --test
   memory --run-ignored only --no-capture`.
2. The one-process binary; `heavy cargo nextest run -p temper-one-process-world --test memory
   --run-ignored only --no-capture`.
3. The knobs, one run each; the figures into the report file.
4. The status edit; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground-world -p
   temper-one-process-world --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-playground-world -p temper-one-process-world` (the default runs, which skip the two).
   Commit; push.

## Report

As the implementer's text says, and: the table, in short (the three largest peaks and what
controls each).

## Answers

2026-11-03. Q: wait for owners to supply measurement access, or change the composition's and
harness's APIs here? A: change them here, in two small additive seams and nothing more; no other
task owns them and waiting supplies nothing. This replaces "Nothing in `temper-playground` or
the harness changes" for these seams alone; behaviour is kept and every existing caller reads as
before.

- **The playground world's meter,** `temper/crates/temper-playground/src/story.rs`:
  `pub fn world_metered<N: skein_sim_world::Meter>(start: Start, meter: N) -> World<Process<()>,
  Machine, N>`, which is today's `world` with `.with_meter(meter)` right after `with_machine`,
  before any process is admitted; `world(start)` becomes `world_metered(start, Bare)`. `capture`
  and `step` become generic over the world's meter (`N: Meter`). The test binary's meter is its
  own `skein_sim_world::Meter` impl over `skein_heap::Span` per process index (admit, around,
  release, restart), in `tests/memory.rs`: skein-world's `Metered` is crate-private and stays so.
- **The one-process runner's memory,** `temper/tests/one-process/src/world.rs` and
  `src/fixtures.rs`: `Settings` gains `pub memory: skein_world::Memory`, passed to
  `new_controlled` in place of the hard-coded `Memory::Unchecked`; `fixtures::calm` sets
  `Memory::Unchecked`. `Outcome` gains `pub peaks: Option<Vec<u64>>`, from skein-world's outcome's
  `peaks()`, with the process names in index order if the table needs them. Merge
  `origin/playground-temper/one-process-story-a` first: it edits the same file; keep its lines.
- **The knobs are halved by hand,** not by an API: for each run, halve the knob's value in the
  document that holds it (`temper-playground`'s documents, jig's Small profile as the playground
  takes it, the fakes' limits) as a local, uncommitted edit, run, record, and restore it with
  `git checkout -- <file>` before the next. The commands and the edited constant go in
  `playground-memory.md`; the pushed branch carries no knob and no edit to those files.
- **If story A does not walk yet** on the one-process world (its test still `#[ignore]`d for the
  playground's handover, `one-process-story-a`'s own gap): write `story_a_at_its_high_water` over
  the same entry as `tests/suite/story_a.rs`, record its high water and its knobs as
  `unmeasured: story A does not walk yet` in `playground-memory.md`, leave the status entry as it
  is, and report it under `## Left`. The playground world's first step and start, the fake disk,
  and each knob's effect on them are measured in full regardless.
