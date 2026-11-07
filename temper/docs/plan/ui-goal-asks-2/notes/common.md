# ui-goal-asks-2: standing rules

From the request (`temper/docs/plan/ui-goal-asks-2/inputs/request.md`, *For every task of this plan*); every task holds them.

- **A plan is complete when it lands.** No walk, render check, screenshot or throwaway browser driver.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure. A story for those tiers is written and compiled, not run. This plan adds none.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely, though this plan needs none. A small edit just outside your files (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** a new rule goes under its marker comment, `/* ==== ui: needs ==== */` or `/* ==== ui: goals ==== */`, never at the end of the file. Remove a replaced rule where it stands.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** on a catch-up, regenerate rather than hand-merge; keep `names.txt` sorted.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task.
- **`temper-client-journeys again` is generation, not a walk:** a task whose change makes a recording stale runs it, though it drives a browser. A re-record stopping on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Help pages:** a page your change grows past its bound is yours to shorten, losing no fact, and the bound is never raised.
- **A test's oracle or helper your correct change trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Failing test first, at the lowest tier:** write each named test, see it fail for the rule's reason, then make it pass.
- **Your own status entry:** you remove the **Not yet** entry your task closes from `temper/docs/design/client/README.status.md`, though `docs/design/` is otherwise closed (two entries there name this plan's work: remove only what your task closes, and word what stays). Run `python3 -I -B tools/citations/citations.py --base origin/main` before pushing.
