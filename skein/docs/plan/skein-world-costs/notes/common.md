# skein-world-costs: the rules every task names

The request is `skein/docs/plan/skein-world-costs/inputs/request.md`: read *What's wanted* and *For every task of this plan*, which bind. `skein/docs/plan/skein-world-costs/notes/measuring.md` holds how a test is measured and written down, and binds too.

This plan is `skein/docs/foundation/testing-strategy.md`, 8.1's second question for `Counting` and `Wire`: they belong, so they are made cheaper at the cause.

## What holds for each task

- **Focused and fuzzy alone.** Never `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh` or a story of a slow tier. No walk, render check or throwaway driver.
- **The profile comes first:** a task reads its figures on its base before it changes anything (`skein/docs/foundation/testing-strategy.md`, 8, "A slow test is understood, never hidden").
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. No `[profile.dev.package.skein-heap]`: that is the user's to sign off, with the numbers this plan reports.
- **Nothing is lost.** No test leaves its suite, and no assertion weakens.
- **Decide, don't ask.** A small edit just outside your files is made and named in your report. `Question:` is only for what truly blocks the task.
- **Before you call a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **No design text changes in this plan.** The form of the counters and the trace is the code's; what `skein/docs/design/testing.md`, 5, says of them stays true.

## A departure from the implementer's text

**Direct dependents.** `skein-heap` has 52 direct dependents and `skein-world` 107, so your spec names the packages to run instead of all of them. Run those. The trunk's end check runs the wider set, and a red it raises comes back as a repair.

## Not this plan

- `skein-json`'s string paths, and what a process costs before a test starts: other plans'.
- A cause in a test's own fixture: another plan's. Give it a line with its share, and leave it.

## Beside this plan

Many plans add worlds that use `Counting` and `Wire`, and catch-ups are expected. Keep hunks small, don't reformat, and don't move code you don't need to change. If a merge meets another draft in a file your spec names, keep their change, and say so in your report.
