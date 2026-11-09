+++
id = "bounds-clear"
part = "bounds-clear"
kind = "feature"
needs = ["differs-says"]
files = ["temper/crates/temper-playground/stories/a/deployment.json", "temper/tests/playground/tests/suite/walk.rs", "temper/tests/playground/tests/fixtures/story-a.log", "temper/docs/design/playground.status.md"]
effort = "medium"
minutes = 120
+++

## Goal

Story A's world decides nothing at a byte bound natively, so the
browser's wasm32 world, which counts the same content in fewer bytes,
says what the native replay of its log says
(`temper/docs/design/playground.md`, 8, *Clear of the bounds that count
slots*, and 9, *Clear of its byte bounds*; skein's
`skein/docs/design/playground.md`, 6, *Another build's slots*). This is
the request's items 1 and 3: the cause, fixed where it is, and a
deterministic test in the lowest world that fails before the fix.

What the cut found, by reading (`temper/docs/plan/red-87-replay/plan.md`,
*What the cut found*): the replay's loop, the page's log and the world's
inputs agree, and the world reads no clock, randomness, hash order,
float or target-gated code. What differs between the two builds is the
word size: jig's and temper's byte bounds add `size_of` of a slot (a
row, a record, a box) to each item's owned bytes (for instance
`temper-engine-domain`'s `loads.rs` cuts a page at
`size_of::<Record>()` plus each record's bytes, and jig's
`jig-domain-core` `reads.rs` and `limits.rs` count the same way), and
story A's deployment (`temper/crates/temper-playground/stories/a/deployment.json`,
the same bytes on both builds) keeps several byte bounds tight:
`core.load_bytes` 8192, `core.run_bytes` 16384, `core.policy_bytes`
8192, `documents.text_bytes` 128, `views.text_bytes` 1024,
`hub.event_bytes` 1024, `people.amendment_bytes` 1101 and others. Near
such a bound the native world cuts or refuses where the wasm32 world
does not, which the person's timing makes happen now and then.

Acceptance:

- **The test** (Steps, 1): `walk.rs`'s
  `story_a_kept_log_replays_natively` replays the kept log twice: over
  story A's deployment as kept, and over it with each `*_bytes` override
  of its engine, its worker and its application doubled. Both replay to
  the end and pass `assert_main`.
- **It fails before the fix,** on the kept log as it stands on your base,
  and the report names the first bound the doubling showed the story
  reaching, with the `Differs` the doubled replay gave (the line's input
  and last lines, from `differs-says`).
- **The fix** (Steps, 3): story A's deployment gives each byte bound the
  story reaches room past its native high water; the kept log is
  recorded again by its own test; the doubled replay then passes.
- `temper-playground-world`'s `memory` binary passes unchanged, and the
  readers of the kept log pass (Steps, 4).
- The **Not yet** entries 8, *Clear of the bounds that count slots*, and
  9, *Clear of its byte bounds*, in
  `temper/docs/design/playground.status.md` are removed, their lines
  alone, and the date line set to the day you land. This departs from
  the implementer's text, which closes `docs/design/` to you: the
  entries are this task's.

## Inputs

- `temper/docs/design/playground.md`, 8 (*Its documents bound it*;
  *Clear of the bounds that count slots*) and 9 (*A log replayed*;
  *Clear of its byte bounds*).
- `skein/docs/design/playground.md`, 6 (*Another build's slots*) and 6.2
  (the replay's loop; *What a difference says*).
- `temper/docs/plan/red-87-replay/inputs/request.md` and
  `temper/docs/plan/red-87-replay/plan.md`, *What the cut found*.
- `temper/docs/plan/red-87-replay/notes/common.md`.

## Owns and may change

- Owns the files in `files`. The kept log changes only by its own test's
  writer (`SKEIN_CODEGEN_WRITE=1`, Steps, 3), never by hand.
- Reader files, assertion edits only if the re-recorded log moves what
  they assert: `temper/crates/temper-playground/src/tests/world.rs` (it
  replays the kept log's first inputs) and
  `temper/tests/one-process/tests/suite/replay.rs` (the kept log under
  the forge referee). Name each edit in the report.
- May run `browser_playground::story_a_walks_on_the_built_playground`,
  by name and alone, through the browser tier's runner filtered to it
  (`temper/scripts/browser.sh -E
  'test(=browser_playground::story_a_walks_on_the_built_playground)'`),
  at most three times before the fix and three after, reading no CPU
  figure: the request's one exception (notes, *Only the focused and
  fuzzy suites run*).
- Pre-approved, each named in the report:
  - A doubled bound that startup refuses (a relation between limits):
    keep it at its value in the test, by name, with the refusal's words.
  - A bound the story reaches by its payload alone (counted without any
    `size_of`, so the same on both builds): leave it out of the doubling
    by name, with the line of code that counts it.
  - Room: raise each reached bound in `deployment.json` to twice its
    value, and again while the doubled replay still shows it; never
    more than the doubled test needs.
  - If `memory` fails because of the raised bounds, take the smallest
    raise that keeps the doubled replay passing; if none keeps both, stop
    with a `Question:` giving the bound, its values and the heap figures
    measured, since the memory figures are the user's.
  - If the doubled replay of the kept log differs nowhere (the story
    reaches no overridden byte bound natively at the kept walk's pace):
    run the browser story as above. If a run fails, its `Differs` names
    the line's input and the last lines; find the bound or the cause
    they point at, and fix that at its cause: a byte bound by room as
    above (and the test still doubles it); anything else in the crate
    that holds it, with a test in the lowest world that shows it, and
    the design sections above left as they are (the report says what
    the code taught). If no run fails, stop with a `Question:` giving
    the experiments, their counts and results, and the options: more
    browser runs, or land the doubled test alone.
- Not yours: `browser_playground.rs` (the story keeps its replay check
  and its `panic!`), skein's replay and log (the `differs-says` task's),
  any limit's accounting in jig or temper (a word-size-free accounting
  is not this plan's), `temper/docs/development/playground-memory.md`'s
  figures, any `docs/design/` but the status entries above.

## Steps

1. **The test,** in `temper/tests/playground/tests/suite/walk.rs`: a
   helper beside `kept_log` that takes the deployment's bytes and returns
   them with every `*_bytes` value under `/engine/limits/overrides`,
   `/workers/*/limits/overrides` and `/application/limits/overrides`
   doubled (`serde_json`, as `temper-playground` reads the deployment);
   and `story_a_kept_log_replays_natively` loops over two rows, the kept
   deployment and the doubled one, each building its world from
   `A.start(read.seed, read.start)` with `start.deployment` replaced for
   the doubled row, replaying the kept log as it does now and asserting
   `assert_main`. A row's failure names the row and prints the
   `Differs` with `{:#?}`. One test, two rows, one more replay: no new
   world.

2. **Before the fix,** on your base with `differs-says` merged:
   `heavy cargo nextest run -p temper-playground-world --test suite -E
   'test(=walk::story_a_kept_log_replays_natively)'`. Expect the doubled
   row to differ. From its `Differs` (the input, the last lines) and
   the deployment, find which bound the story reaches: double one bound
   at a time, starting with those the last lines' read points at, until
   the doubled replay of that one alone differs. Keep the failing run's
   output for the report. This is the bite: the test fails for the rule
   (the story reaches a byte bound) before the fix.

3. **The fix:** raise the reached bounds in `deployment.json` (the
   options above; keep its formatting and key order), then record the
   kept log again: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   temper-playground-world --test suite -E
   'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`,
   and run it once more without the variable to see it keep the same
   bytes. Then the doubled test passes; if it still differs, the next
   bound is reached: repeat 2 and 3 for it.

4. **Runs,** checking each count:
   - `heavy cargo nextest run -p temper-playground-world --test suite -E 'test(/^walk::/)'`
   - `heavy cargo nextest run -p temper-playground-world --test memory`
   - `heavy cargo nextest run -p temper-playground --lib`
   - `heavy cargo nextest run -p temper-one-process-world --test suite -E 'test(=replay::story_a_kept_log_replays_under_the_referee)'`
   - the browser story, by name and alone, once after the fix (more,
     up to three, if you ran it before the fix), saying what each run
     gave.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-playground-world --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` after
   committing; commit saying why (story A reached a byte bound counted
   by slot size, so the browser's wasm32 world and the native replay
   decided differently there); push.

## Report

As the implementer's text has it, and: the bounds the story reached,
each with the doubled replay's `Differs` that showed it and its old and
new values; any bound left out of the doubling and why; the kept log's
re-recording (its length and input count before and after, from the
walk's `eprintln`); the browser story's runs and what each gave; the
memory binary's figures if they moved; the readers you edited.
