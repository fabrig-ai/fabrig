# ui-p3f-design: the rules every task keeps

Every spec of this plan names this note. A spec wins where it says otherwise.

## The plan's rules

From the request (`temper/docs/plan/ui-p3f-design/inputs/request.md`, *For every
task of this plan*):

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or
  live tier or one of their stories, a walk, a render check or a throwaway driver:
  a plan is complete when it lands. A story of a slow tier is written and
  compiled (`heavy cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table
  test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a
  story goes beside its siblings' fixture. temper's focused and fuzzy suites go to
  30 and 120 CPU seconds in the gate.
- **Decide, don't ask.** A stored format, a protocol or a public interface may
  change freely, keeping no compatibility; say so in the report. A small edit a
  task needs just outside its files (a re-export, a `mod` line, a golden) is made
  and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** your change makes stale (`temper/docs/help/reference/*`,
  a schema's generated code and goldens, prints, the gallery's goldens, counted
  baselines, journey recordings) are yours to regenerate with their own tool in
  the same task, never by hand. The client wire's and the store's generated
  Rust and goldens come from their schemas by the codecs world's drift test:
  `TEMPER_BUNDLE=none SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
  temper-codecs-world --test suite -E 'test(drift::)'`, then the same without
  `SKEIN_CODEGEN_WRITE`; regenerated, never hand-merged, on a catch-up too.
  `docs/help/reference/*` pages come from their tests with
  `SKEIN_CODEGEN_WRITE=1` (`temper/crates/temper/src/tests/settings_fields.rs`,
  `temper/crates/temper/src/tests/commands.rs`,
  `temper/crates/temper-client-tools/src/tests/help.rs`). `temper-client-journeys
  again` is generation, not a walk: run it where a journey's focused replay check
  fails for your change; a re-record that stops on "a key mark caught a moving
  page" is retried up to three times, each on a fresh fake box.
- **Help pages** (`temper/docs/help/`) a change grows past their bound are that
  task's to shorten, losing no fact; never raise a bound.
- **A test's oracle or helper** that your change's correct behaviour trips is
  yours to repair, keeping what it checks for everything else; name it in the
  report.
- **Before calling a focused or fuzzy failure outside the change,** run it once on
  the pristine base. If main passes and your branch fails, it's yours.
- **`temper.css`:** this plan adds no rule.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** regenerated, not
  hand-merged; `names.txt` stays sorted as it is.

## The words

The words this plan ships, a stop's words, its card's `who` and facts and the
"none kept" answers, are written in `notes/contracts.md` and in
`temper/docs/design/domain/forge.md`, 8.13, byte for byte: a task copies them,
never rewrites them.

## Status files

`docs/design/` isn't an implementer's, but for this: a task that builds what a
**Not yet** entry names deletes that entry, the one bullet naming the task, in the
commit that builds it: the bullet's own lines alone, leaving the lines around it,
so that two tasks' deletions in one file merge. If what it built leaves part of
the entry true, it keeps that part and says so in the report.

## Beside this plan

Two plans in flight add kinds of temper's by the same mechanism and walk the
same files: ui-reaches (kind 8, a project's classes) and ui-p3e-words (kinds 9
and 10, a setting's words). Neither has landed when this plan is cut.

- **Kind 11 is this plan's;** 8, 9 and 10 are theirs. On a catch-up that meets
  theirs in a file of yours, keep both: each list gains its own lines, and a
  `match` its own arms.
- **`SettingValue`'s order is a stored format:** `goal` stays the store schema's
  last variant, after whatever main holds then (`classes`, `line`, `words`).
- **Never edit** what those plans own for their kinds: `ProjectWays` and kind 7's
  client reads (ui-reaches' `ways-away`), the stock's instructions
  (`temper/crates/temper-service/src/stock/*.txt`, `coordinator.md`), and the
  client's How work runs page (`temper-client-view/src/pages/how.rs`,
  `temper-client-domain/src/pages/how.rs`).
- **This plan writes no words into Ask temper's instructions** (`ask.txt`): they
  are at their 8,192-byte bound once both plans land. Ask temper reaches kind 11
  through its tools' declarations and `see settings template-goal N`.
