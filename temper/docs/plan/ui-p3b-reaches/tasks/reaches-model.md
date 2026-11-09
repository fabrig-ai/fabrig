+++
id = "reaches-model"
part = "model"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/how.rs", "temper/crates/temper-client-domain/src/tests/pages/how.rs"]
effort = "high"
minutes = 50
+++

## Goal

Give How work runs' What reaches you its model in `temper-client-domain`:
the parts' two new facts and one pure function laying a project's classes
in three columns, each subject once, so that the view, the gallery and the
fuzz test all build on one rule (`temper/docs/design/client/README.md`,
5.13, *What reaches you* and *Its parts*). No drawing here.

Acceptance:

- `pages::how::Facts` and `Parts` gain `classes`/`several` and
  `reaches`/`several` as `temper/docs/plan/ui-p3b-reaches/notes/contracts.md`,
  section 1, has them; `parts` sets `reaches` from `classes` and `several`
  from `several`; `facts` gathers both.
- `pages::how::{Line, Reaches, reaches}` exist exactly as that section
  types them, in the settings order it gives, total on any table.
- The domain's How table test (`src/tests/pages/how.rs`,
  `parts_and_facts_follow_the_project_rules`) holds the new facts and
  parts as rows of its existing loops, and a new table of `reaches` rows:
  Agents review's stock (decide wording 1, retry 2, tests 3; ask
  implementation 4, dependency 5, departure 6, all `Ask`); You approve's
  (no decide line; ask the six movable kinds, item 0, `AskAway`); a table
  with an item with words, a set with its own item, a set with none and an
  item at `AskAway` (the `AskAway` lines first in "Asks you", the words
  item after the kinds, the bare set last); every movable kind at
  `Decide` (no ask line); and an unchecked table with a fixed kind's item,
  an item at `Wait` and an item naming a missing set, each left out. Every
  row's `wait` is the four fixed kinds in `Kind::ALL`'s order.
- `facts` rows: kind 8 unread is `classes: false`; read (the fixture's
  kind 8 answer, with or without a value) is `true`, for the open project
  alone; `me` with one project is `several: false`, with two `true`.
- Delete `temper/docs/design/client/README.status.md`'s **Not yet** entry
  `5.13, *Its parts*` (the one naming `pages::how::reaches`).

## Inputs

- `temper/docs/plan/ui-p3b-reaches/notes/common.md`; `notes/contracts.md`,
  section 1.
- `temper/docs/design/client/README.md`, 5.13, *What reaches you* and *Its
  parts*; `temper/docs/design/domain/forge.md`, 5.10, *The table*.
- Patterns: `temper/crates/temper-client-domain/src/forge/classes.rs`
  (`asking`, the same walk over kinds then items), and
  `temper/crates/temper-client-view/src/cards/proposal.rs`, `class_objects`
  (the settings order this function's columns keep); the domain's kind 8
  read, `Forge::classes` (`src/forge/ways.rs`).
- Fixtures: `src/tests/pages/how.rs`' `fixture` and `me`; how kind 8 is
  answered in a domain test: `src/tests/away.rs` (its classes' reads).

## Owns and may change

- **Owns:** the files in this spec's header.
- **Readers, call sites only:** every literal of `pages::how::Facts` or
  `Parts` the new fields break, in any file: today
  `temper/tests/client/tests/suite/fuzzy/how.rs` (`facts` and the
  literals it builds) and the domain's own tests. Give them `classes:
  false, several: false` (and `reaches: false, several: false`) and
  nothing more: the fuzz test's rows for them are `reaches-fuzz`'s.
- **Options, pre-approved:**
  - if a reader outside these files builds `Facts` or `Parts` (`rg
    'how::Facts|how::Parts|Facts \{ roles'`), give it the same defaults
    and name it in the report;
  - if `reaches` reads more clearly split into a private walk per step of
    the settings order, split it; the public surface stays the note's.

## Steps

1. The fields, `parts` and `facts`; the `Line`, `Reaches` and `reaches`
   of contracts, section 1, with a module-level citation of README.md,
   5.13, *What reaches you*.
2. The rows above in `src/tests/pages/how.rs`. Break the order once by
   hand (put the `Ask` lines before the `AskAway` ones) and see the
   mixed row fail; restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-client-domain --lib -E 'test(/pages::how::/)'`, then
   `heavy cargo check -p temper-client-world --tests` and `heavy cargo
   check -p temper-client-view --tests` for the readers. Commit and push.

## Report

As the implementer's text asks; name every reader whose literal you
widened.
