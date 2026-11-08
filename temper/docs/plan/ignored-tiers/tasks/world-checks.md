+++
id = "world-checks"
part = "world-checks"
kind = "feature"
needs = []
files = ["temper/tests/playground/tests/memory.rs", "temper/tests/one-process/tests/memory.rs", "temper/tests/engine/domain/tests/suite/forge_routes/hints.rs", "temper/docs/development/playground-memory.md", "temper/docs/design/playground.status.md"]
effort = "medium"
minutes = 70
+++

## Goal

Three measurements run by hand become checks in the focused suite,
where their crates' binaries already sit. The playground world's native
heap at start is held under bounds. Story A's one-process high water is
held the same way if the story walks. The forge's two-change hint
profile asserts what it printed.

Acceptance:

- `temper/tests/playground/tests/memory.rs`,
  `story_as_world_at_its_first_step_and_start` (package
  `temper-playground-world`, binary `memory`): no `#[ignore]`, and it
  passes in the focused suite. Each figure that
  `temper/docs/development/playground-memory.md`'s *Measured bytes*
  records is a named bound in the test: the first step's and start's
  held and peak bytes per process, the fake disk's retained bytes at
  each, and the shared forge's before start. Each bound is the figure
  plus a quarter (`figure + figure / 4`). The doc gives no margin, so the
  request's 25% holds. The test asserts each measured value is at most
  its bound, naming the process, the phase and both numbers when it
  fails. It also asserts which processes it measures at each phase (three
  at the first step, four at start, by `name`), since the bounds are
  keyed by them. It keeps its existing assertions. The `println!` tables
  may stay, for whoever reads `--no-capture`.
- `temper/tests/one-process/tests/memory.rs`, `story_a_at_its_high_water`
  (package `temper-one-process-world`, binary `memory`). First run
  `story_a_is_walked_from_the_welcome_to_the_csv_change_landed` alone
  (`heavy cargo nextest run -p temper-one-process-world --test suite
  --run-ignored only -E 'test(=story_a_is_walked_from_the_welcome_to_the_csv_change_landed)'`,
  a focused test, count 1).
  - If it fails (the expected case, `temper/docs/design/playground.status.md`,
    9), the memory test keeps its `#[ignore]`, and its reason becomes
    `"story A does not walk in the one-process world (playground.status.md, 9): no high water to bound"`.
    Its body doesn't change.
  - If it passes, run the memory test alone with `--run-ignored only
    --no-capture`, and record its per-process peaks in the doc (a table
    under *Measured bytes*, dated by `date -u +%F` with the commit). Bind
    the test to them as above, and remove its `#[ignore]`.
    `story_a.rs` stays as it is: it is not this plan's.
- `temper/tests/engine/domain/tests/suite/forge_routes/hints.rs`,
  `two_change_hint_request_profile` (package `temper-engine-domain-world`,
  binary `suite`): no `#[ignore]`, and it passes in the focused suite. Its
  printed profile becomes assertions (forge.md, 5.2, *Which change a
  hint wakes*):
  - each of the three hints wakes the changes it printed (`temper/a`:
    51; `release`: 52; `unmatched-or-renamed`: 51 and 52);
  - the requests each hint's steps send are the woken changes' fresh
    reads and no more: their count is the sum of `reads_for(...).len()`
    over the woken changes;
  - each landing's requests are counted. The second change's landing
    (52) sends no more requests than the first's (51).
  Run it once alone with `--run-ignored only --no-capture` before
  editing, to read what it prints. Use the figures it shows, and say in
  the report what each assertion holds. If a printed line holds nothing
  stable to assert, the test says so in a comment above it and asserts
  the smallest thing it can: a count's upper bound, never a print.
- `temper/docs/development/playground-memory.md`: its first paragraph no
  longer says "No memory ceiling is set here". It says the playground
  binary's test holds each figure below with a quarter's margin, as a
  regression bound (`../design/playground.md`, section 9), and that
  sections 8 and 11 leave the memory's bar itself to the user. *Commands
  and scope*: the playground command loses `--run-ignored only`, since
  the test is in the focused suite. The one-process command keeps it
  while that test is ignored. *Measured bytes*' closing paragraph says
  what holds for the one-process binary: ignored with the reason above,
  or bound to its new table.
- `temper/docs/design/playground.status.md`, **Not yet**:
  - the 9 entry that begins `9: the native heap at start:` is removed
    whole if both memory tests are bound. If the one-process test stays
    ignored, it is replaced by:

    ```
      - 9: story A's high water in the one-process world:
        `temper/tests/one-process/tests/memory.rs` is ignored and prints
        it by hand, the story stalling there (above).
    ```
  - in the 8 entry, `and sets no bound;` becomes
    `and the playground world's test holds them within a quarter (9);`.
- `heavy cargo nextest run -p temper-playground-world -p
  temper-one-process-world -p temper-engine-domain-world` passes. Report
  each changed test's time from nextest.

## Inputs

- `temper/docs/plan/ignored-tiers/notes/common.md`: every rule there
  holds.
- `temper/docs/design/playground.md`, 8 and 9 (*The native heap at
  start, in the gate*).
- `temper/docs/development/playground-memory.md`: the figures and what
  they cover.
- `temper/docs/design/domain/forge.md`, 5.2 (*Which change a hint
  wakes*) and 5.3.
- `skein/docs/foundation/testing-strategy.md`, 8 (focused tests hold
  "memory under the scenarios' ceilings") and 8.1.

## Owns and may change

- Owns: the files in the header. In `hints.rs`, the profile test alone:
  its siblings and `reads_for` keep their assertions, and a helper they
  share may be added.
- Not yours: `temper/tests/one-process/src/**` and
  `temper/tests/playground/src/**`. The memory tests measure the worlds
  as they are, and the `ui-p1-rail` plan is changing
  `temper/tests/one-process/src/`.
- Options, each pre-approved:
  - If a playground figure on your base is already above its bound (the
    doc measured at `79d1b7c27b`, 2026-11-03), don't loosen the margin.
    Measure the trunk (`--no-capture`) and add a dated table with the
    commit under *Measured bytes*. Keep the old table and say which one
    the test binds. Bind that figure plus a quarter, and name each
    growth over 25% in the report as a finding for the user.
  - If world-step counts or process order differ from the doc (147
    steps; service, fake LLM, fake forge HTTP, hosted git), keep the
    bounds keyed by `name`. Report the difference, and correct the
    doc's words if they are wrong.
  - If the hint profile's second landing sends more requests than the
    first, it fails on the trunk. That is a finding in the product, not
    the test. Keep the assertion, keep `#[ignore]` with the reason
    `"forge.md, 5.2: the second change's landing sends N requests to the first's M"`
    (your figures), and report it first in your report.
  - If the playground test runs over its guide (1 s), keep it where it
    is and report its time and cause (8.1, question 2).

## Steps

1. Run each of the three tests once as it stands, alone, `--run-ignored
   only --no-capture`, and the one-process story as above. Keep their
   outputs out of the branch.
2. Turn the playground measurement into bounds. Run it alone, then break
   one bound once (a bound below its figure) to see it fail with its
   message, and restore it.
3. The one-process test, by the story's result.
4. The hint profile: assertions in place of prints. Run it alone. Break
   it once (invert the landing comparison) and restore.
5. The doc and the status lines.
6. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-playground-world -p
   temper-one-process-world -p temper-engine-domain-world --all-targets
   -- -D warnings`; the three packages' focused run.
7. Commit, run the citations check (`notes/common.md`), push.

## Report

The implementer's report, with: each test's first output's figures (or
first failure) and its time now; the bounds as named; the one-process
story's result and what followed; what each hint assertion holds; any
option taken.
