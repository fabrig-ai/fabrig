# ui-drafter: the rules every task keeps

Every spec of this plan names this note. A spec wins where it says otherwise.

## The plan's rules

From the request (`temper/docs/plan/ui-drafter/inputs/request.md`, *For every task of this plan*):

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or
  live tier or one of their stories, a walk, a render check or a throwaway driver:
  a plan is complete when it lands.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table
  test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a
  story goes beside its siblings' fixture. temper's focused and fuzzy suites go to
  30 and 120 CPU seconds in the gate.
- **Decide, don't ask.** A stored format, a protocol or a public interface may
  change freely, keeping no compatibility; say so in the report. A small edit a
  task needs just outside its files (a re-export, a `mod` line, a golden) is made
  and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** your change makes stale (`temper/docs/help/reference/*`,
  prints, goldens, counted baselines, journey recordings) are yours to regenerate
  with their own tool in the same task. `temper-client-journeys again` is
  generation, not a walk; a re-record that stops on "a key mark caught a moving
  page" is retried up to three times.
- **Help pages** (`temper/docs/help/`) a change grows past their bound are that
  task's to shorten, losing no fact; never raise a bound.
- **A test's oracle or helper** that your change's correct behaviour trips is
  yours to repair, keeping what it checks for everything else; name it in the
  report.
- **Before calling a focused or fuzzy failure outside the change,** run it once on
  the pristine base. If main passes and your branch fails, it's yours.

## The drafter's words

`temper/docs/plan/ui-drafter/inputs/drafter.txt` is the drafter's instructions,
taken exactly as written: no task edits its words, and the stock's copy is its
bytes (`notes/contracts.md`, *The charter*). If a stock bound or a test refuses
any of it, stop with `Question:`, naming the bound and the bytes.

## Status files

`docs/design/` isn't an implementer's, but for this: a task that builds what a
**Not yet** entry names deletes that entry, the one bullet its spec names, in the
commit that builds it: the bullet's own lines alone, leaving the blank lines
around it, so that two tasks' deletions in one file merge. If what it built
leaves part of the entry true, it keeps that part and says so in the report.

## Not to touch

- `temper/crates/temper-client-domain/src/pages.rs`: another plan changes it now.
  A change there is a question.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`, but for the one
  call `no-workspace` changes.
- `temper/crates/temper-client-shell-web/temper.css`, the gallery and its
  `names.txt`: nothing here draws anything new.
- The rail's menu and its words (`temper-client-view`): its "Goal" item already
  leads to `NewChat { draft: true }`.
