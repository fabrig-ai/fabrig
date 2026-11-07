# ui-goal-asks: standing rules

From the request (`temper/docs/plan/ui-goal-asks/inputs/request.md`, *For every task of this plan*); every task holds them.

- **A plan is complete when it lands.** No walk, render check, screenshot or throwaway browser driver.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure. A story for those tiers is written and compiled, not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely, though this plan needs none. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** on a catch-up, regenerate rather than hand-merge; keep `names.txt` sorted.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task.
- **Help pages:** a page your change grows past its bound is yours to shorten, losing no fact, and the bound is never raised.
- **A test's oracle or helper your correct change trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Failing test first, at the lowest tier:** write each named test, see it fail for the rule's reason, then make it pass.
- **Not yours:** jig's code and design, `message`, `authorize_tool`. The goal's charter is temper/migration's by seam 4 and its next plans replace that text: keep the change small and its words in one paragraph.
