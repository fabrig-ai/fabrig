# ui-small-1: what every task holds

From the request (`temper/docs/plan/ui-small-1/inputs/request.md`, *For every task of this plan*), which each task obeys:

- **A plan is complete when it lands.** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan: don't ask for them or run them.
- **Only focused and fuzzy run.** Never a browser, process or live tier, one of their stories, or a tier's CPU figure. A story of those tiers is compiled, not run. `printed-link` alone has one exception, which its spec states.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely, with no compatibility. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Goldens, prints and generated files** (`docs/help/reference/*`, goldens, counted baselines, the gallery's `names.txt` and `tests.rs`) your change makes stale are yours to regenerate in the same task. On a catch-up, regenerate them rather than merge them by hand.
- **Help pages:** a page a change grows past its bound is that task's to shorten, losing no facts, and never raises the bound.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.

Beside the request:

- **The design is written.** Each task's `## Inputs` names its sections. A task never edits them, but for dropping the **Not yet** entries it closes in the status file beside them, which its `## Owns and may change` names. This departs from the implementer's text.
- **Other plans change the same files** (`temper/docs/plan/ui-small-1/plan.md`, *Overlaps*). Edit only the lines your task names, and leave the rest as you found it.
