+++
id = "reaches-fuzz"
part = "fuzz"
kind = "feature"
needs = ["reaches-model"]
files = ["temper/tests/client/tests/suite/fuzzy/how.rs"]
effort = "high"
minutes = 60
+++

## Goal

Fuzz What reaches you's model against `temper/docs/design/draft/surfaces.md`,
section 12's rules (its section 14's fuzz test): over random tables of a
project's classes and their moves, every subject is placed exactly once,
no column is drawn empty, each column holds its class alone in the
settings order, and a move never hides a live subject; and the page's
parts follow their two new facts.

Acceptance:

- In `temper/tests/client/tests/suite/fuzzy/how.rs`:
  - `facts` draws `classes` and `several` too; `check_parts` asserts
    `reaches == facts.classes` and `several == facts.several`;
    `combination` and `FACT_COMBINATIONS` count them (four times today's);
    the existing test keeps its name and its 2,000 seeds.
  - A new test, `what_reaches_you_places_each_subject_once`, beside it:
    2,000 seeds, each a random table that `Classes::check` accepts, then
    16 moves of it, mixing: an item's class turned among `Decide`, `Ask`
    and `AskAway`; an item with words added; a set added, with its own
    item or none; an item removed; a set removed with its items; the
    table replaced by a stock way's (`Classes::stock`). Every table drawn
    is checked first (`check`), a move it refuses redrawn or skipped. On
    each table, `pages::how::reaches` against an oracle written here,
    independently of the function: every movable kind once, every item
    that is not a kind's own once, every set without its own item once,
    every fixed kind once, nothing else; `decide`'s lines all `Decide`,
    `ask`'s all `AskAway` then all `Ask`, `wait` the fixed kinds in
    `Kind::ALL`'s order; within each column the settings order of
    `notes/contracts.md`, section 1; each line's `item` and `class` the
    table's.
  - The sweep's coverage is asserted at its end, as the existing test
    asserts its combinations: tables with and without a `decide` line,
    with and without an `ask` line, and "Asks you" all `Ask`, all
    `AskAway` and mixed, were each drawn.
- Its CPU time stays small beside the fuzzy suite's 120 seconds: measure
  the two tests with `--profile fuzzy` and put the figures in the report.

## Inputs

- `temper/docs/plan/ui-p3b-reaches/notes/common.md`; `notes/contracts.md`,
  section 1.
- `temper/docs/design/draft/surfaces.md`, sections 12 and 14 (its fuzz
  test's rules); `temper/docs/design/client/README.md`, 5.13, *What reaches
  you* and *Its parts*; `temper/docs/design/domain/forge.md`, 5.10, *The
  table* (its bounds: 20 items, 120 bytes of words, 6 sets of 1 to 6
  patterns).
- Patterns: the same file's `sweep` and its replay message; the table
  generator of `temper/crates/temper-protocol-reaches/src/tests/fuzzy.rs`
  (`random_table`, `pattern`), which is private to that crate: write this
  file's own, smaller, from the kit's public bounds (`ITEMS`, `SETS`,
  `WORDS_BYTES`, `NAME_BYTES`, `PATTERNS`).

## Owns and may change

- **Owns:** the file in this spec's header.
- **Options, pre-approved:**
  - if the oracle needs a helper the kit doesn't export (a set's own item,
    `Kind::fixed`), write it in this file from the kit's public types;
  - if `pages::how::reaches` disagrees with the oracle on a table `check`
    accepts, the design (`README.md`, 5.13, *What reaches you*) and
    `notes/contracts.md`, section 1, decide which is wrong; a fault in the
    function is repaired in `temper-client-domain/src/pages/how.rs`, with
    a row added to `src/tests/pages/how.rs`' `reaches` table, and named in
    the report.

## Steps

1. The facts' draws and checks; then the new test, its generator, moves,
   oracle and coverage.
2. Break the rule once by hand (place a set with no item of its own in
   `decide`, in `reaches`), see the new test fail with its seed, restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-world
   --profile fuzzy -E 'test(/^fuzzy::how::/)'`, two tests selected. Commit
   and push.

## Report

As the implementer's text asks, with the two tests' CPU figures.
