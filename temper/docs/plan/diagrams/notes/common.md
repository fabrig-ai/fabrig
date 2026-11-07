# diagrams: what every task holds

From the request (`temper/docs/plan/diagrams/inputs/request.md`, *For every task of this plan*), which each task obeys:

- **A plan is complete when it lands.** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan: don't ask for them or run them.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. No task here adds a browser test.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test. The acceptance cases are rows of one table test, never a test each. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate; skein's own budgets hold.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely, with no compatibility. A small edit a task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** new rules go under the marker this plan names, `/* ==== ui: chats ==== */`, never at the end of the file.
- **Generated files** (goldens, prints, counted baselines, `temper/docs/design/client/journeys/skein-ui.css`) your change makes stale are yours to regenerate in the same task, by their own tool.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.

Beside the request:

- **The cases are the acceptance, and never change.** `skein/docs/design/draft/diagrams/*` are read, byte for byte. A scene that differs from its case is a bug in the code: no task edits a case, a `README.md` or `preview.html` there. A change of the look is a change of a case, made with the user, and isn't this plan's to make.
- **Zero cost for a page with no flow:** no scratch in the reader until a fence says `mermaid`, no scene's storage in the page part until it draws (`skein/docs/design/ui.md`, 10.1; `skein/docs/design/ui-web.md`, section 8).
- **The contract** is `temper/docs/plan/diagrams/notes/contracts.md`: its names, signatures and data layout bind every task.
