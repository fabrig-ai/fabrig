+++
id = "story-a-walks"
part = "story-a-walks"
kind = "feature"
needs = []
files = ["temper/crates/temper-playground/src/story.rs", "temper/crates/temper-playground/src/tests/world.rs", "temper/tests/one-process/src/world.rs", "temper/tests/one-process/tests/suite/story_a.rs", "temper/tests/one-process/tests/memory.rs", "temper/docs/development/playground-memory.md", "temper/docs/design/playground.status.md"]
effort = "medium"
minutes = 120
+++

## Goal

Story A walks in the playground world but not in the one-process
world, where the terminal client stalls at the playground's Starting
and then exhausts the world's iteration bound at 0 simulated ns
(`temper/docs/development/playground-memory.md`, *Measured bytes*).
The likely cause is the world's wiring: `temper-playground`'s
`story::world_metered` lays the deployment's handovers on the fake disk
after `machine::setup` (the demo LLM key, each setup entry's
`admin_token`, and the playground forge's admin token at its fake
`playground-N` entry), and the one-process world calls
`machine::setup` alone, so the playground forge's start finds no
handover. Lay them the same way in both worlds, find any further cause,
and bound story A's native high water.

Acceptance:

- `temper_playground::story::prepare` (sketch) lays the fake disk for a
  story's deployment, `machine::setup` and every handover
  `world_metered` lays now; `world_metered` calls it, and so does the
  one-process world for `Story::PlaygroundA`, in place of its
  `machine::setup(&mut disk)` (`temper/tests/one-process/src/world.rs`,
  where `temper_playground::stories::A.start` is taken).
- `temper/tests/one-process/tests/suite/story_a.rs`,
  `story_a_is_walked_from_the_welcome_to_the_csv_change_landed`
  (package `temper-one-process-world`, binary `suite`,
  `story_a::story_a_is_walked_from_the_welcome_to_the_csv_change_landed`):
  no `#[ignore]`, its assertions unchanged, passing alone and then with
  its package.
- `temper/tests/one-process/tests/memory.rs`, `story_a_at_its_high_water`
  (binary `memory`): no `#[ignore]`. Run it once alone with
  `--no-capture` and read each process's peak. Record them in
  `playground-memory.md` (below), and bind each in the test to
  `figure + figure / 4`, keyed by the process's index and name as the
  printed table and the playground test key them; a failure names the
  process and both numbers. It keeps its existing assertions. The test
  also asserts how many processes it measures.
- `temper-playground`'s `story_a_lays_the_playground_forges_handover_before_starting`
  (`src/tests/world.rs`) keeps passing through `prepare`, and
  `heavy cargo nextest run -p temper-playground -p
  temper-playground-world -p temper-one-process-world` passes. Report
  each changed test's time; story A in the one-process world is a
  world's walk, held to its world's guide
  (`temper/docs/design/domain/testing.md`, 6.1) as the playground
  world's walk is.
- `temper/docs/development/playground-memory.md`:
  - *Commands and scope*: the one-process command loses `--run-ignored
    only`.
  - *Measured bytes*: a new table, dated by `date -u +%F` with the
    trunk commit you measured at, of story A's one-process high water,
    peak bytes by process (index and name), and a line saying the
    one-process memory test binds each to its figure plus a quarter.
    The paragraph that begins `The one-process binary takes the same
    entry` says what holds now: the story walks, and its memory test is
    bound to that table. Drop its sentences on the `Bounded` failure
    and the ignore. The 2026-11-03 table's last column keeps its words
    (it is history); nothing else in the file changes.
- `temper/docs/design/playground.status.md`, **Not yet**:
  - the two entries that begin `9: Story A's walk in the one-process
    world` and `9: story A's high water in the one-process world` are
    removed whole;
  - in the 8 entry, `and the playground world's test holds them
    within a quarter (9); through the story there is only the browser
    tier's high water, which does not say what reserves it.` becomes
    `and the playground world's test holds them within a quarter (9),
    as the one-process world's test holds story A's native high water
    by process (9); in a browser there is only the browser tier's high
    water, which does not say what reserves it.`;
  - its date line becomes the day you commit.

## Inputs

- `temper/docs/plan/ignored-misc/notes/common.md`: every rule there
  holds.
- `temper/docs/design/playground.md`, 5.2 (story A), 9 (*The native heap
  at start, in the gate*: "Story A's high water in the one-process world
  ... is bound the same way once the story walks there") and 4 (*No
  box*: the playground's setup entry, "whose handover the composition
  lays on the disk"; the world is the one-process world's composition
  without its harness).
- `temper/docs/design/domain/testing.md`, 2.5 (the one-process world)
  and 6.1 (a world's guide).
- `temper/docs/development/playground-memory.md`: how the figures are
  measured.

## Owns and may change

- Owns: the files in the header.
- Sketch, binding the outer layer:

  ```rust
  // temper/crates/temper-playground/src/story.rs
  /// Lay a story's fake disk before its first start: the machine's
  /// files, then every handover its deployment names
  /// (temper/docs/design/playground.md, 4).
  pub fn prepare(disk: &mut skein_fake_machine::Machine, deployment: &[u8], seed: u64);
  ```

  `world_metered` keeps calling `machine::fresh()` itself;
  `prepare` takes the disk as it is. If `machine::path`'s roots need
  `fresh()`'s reset to lay a handover on the one-process world's disk,
  `prepare` does what laying needs and says so in its doc.
- Options, each pre-approved:
  - If the story still stalls once the handovers are laid, find the
    cause. One in the one-process world's wiring is yours
    (`temper/tests/one-process/src/**`). One in the product whose
    design section exists (cite it in the commit), in a file no other
    plan's task lists (the brief's neighbours change
    `temper-client-domain/src/forge.rs` and `forge/ways.rs`, not
    `forge/playground.rs`), is yours too: the narrowest fix, with a
    step test in that crate. A cause that needs a design section that
    does not exist keeps both tests' `#[ignore]`, each reason naming
    the missing section and what you found; the status entries then
    stay, their words replaced by that cause; the doc's paragraph says
    the same; and the report says it first.
  - If the walk is over its world's guide, report its time and cause
    (8.1, question 2); never move or ignore it for its cost.
  - If the memory test's peaks differ run to run on one seed, the bound
    is the largest of three runs plus a quarter, and the report says
    so.
- Not yours: `temper/tests/playground/**` (its bounds stay as they
  are), the playground world's walk, and every other ignored test.

## Steps

1. Run the story alone with `--run-ignored only` to see where it stops
   now. Keep the log out of the branch.
2. `prepare`, used by both worlds. Run the story alone again; follow
   any further cause (options).
3. Remove the story's `#[ignore]`; run it alone, then its package.
4. Run the memory test alone with `--run-ignored only --no-capture`;
   record its peaks; bind them; remove its `#[ignore]`. Break one bound
   once (a bound below its figure), see it fail with its message,
   restore.
5. The doc and the status lines.
6. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-playground -p temper-playground-world
   -p temper-one-process-world --all-targets -- -D warnings`; the three
   packages' focused run.
7. Commit, run the citations check, push.

## Report

The implementer's report, with: where the story stopped before and
what made it walk; its time now; the memory figures and their bounds;
any option taken.

## Answers

- 2026-11-08. *May story A's one-process deadline (the referee's
  `story_deadline`, 30 s simulated) become 60 s?* Yes: set it to 60 s
  for `Story::PlaygroundA`, and keep the playground script's per-reply
  `within` at 60 s too, the playground world's `WAIT`
  (`temper/tests/playground/src/stories.rs`), where the same story
  completes at 44.8 simulated seconds. This is not a timeout under
  `notes/common.md`'s rule, which names `tools/tiers/budgets.toml` and
  `.config/nextest.toml` alone: it is the world's patience, there to
  end a hung run (`temper/docs/design/domain/testing.md`, 6.1), and the
  30 s was set for a story that never walked. Its comment says why
  (the story's simulated length in the playground world). If the walk
  still does not finish within 60 s, that is a cause to find under the
  wiring option, never a further raise. The wall guide is separate:
  report the walk's time, examine it under 8.1, question 2, and never
  move or ignore it for its cost. Continue from the pushed branch
  (`97e4acf6d3`) with the outstanding acceptance: both `#[ignore]`s,
  the memory bounds, the doc and the status lines.
