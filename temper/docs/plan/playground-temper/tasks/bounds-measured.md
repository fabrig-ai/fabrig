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
