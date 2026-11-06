# ui-public-3: what every task holds

From the request (`temper/docs/plan/ui-public-3/inputs/request.md`, *For every task of this plan*), which each task obeys:

- **A plan is complete when it lands.** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan: don't ask for them or run them.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** rules stay under their marker, here `/* ==== ui: public ==== */`, never at the end of the file. Replaced rules go where they stand.
- **Goldens and generated files** (`docs/help/reference/*`, goldens, counted baselines) your change makes stale are yours to regenerate in the same task, with `SKEIN_CODEGEN_WRITE=1` and the test that reads them. On a catch-up, regenerate rather than hand-merge.
- **Help pages:** a page a change grows past its bound is that task's to shorten, losing no facts, and never raises the bound.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.

Beside the request:

- **The design is written.** Each task's `## Inputs` names its sections. A task never edits them, but for dropping the **Not yet** entry it closes in the status file beside them. This departs from the implementer's text, because the status has to fall in the same landing as the code.
- **Stay out of `temper/tests/client-browser/tests/browser_all/browser_public.rs`.** The plan `ui-public-story` is repairing it, and nothing here changes what it reads: ctl's stdout, and the Landings page's words, stay as they are. If you find it would need changing, leave it and say so under `## Left`.
- **The words are one line** (`notes/contracts.md`). Copy them from that note, and compare them whole in tests, whitespace normalized on a Markdown page.
