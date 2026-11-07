# ui-agent-keys: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/ui-agent-keys/inputs/request.md`; its section `For every task of this plan` holds in full, and what follows is what a task meets most.

## Departures from the implementer's text

- **Decide, don't ask (the user, asleep, takes fabrig/ui's judgement).** A `Question:` is only for what truly blocks the task.
- **A small edit your task needs just outside its file list** (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked.
- **Design text.** A task edits the `docs/design/` lines its `Owns and may change` lists, and any single design line your work proves wrong; the rest stays as this plan's cut wrote it.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a `<tier>-<tree>` profile or `scripts/browser.sh`. No walk, render check, screenshot or throwaway browser driver: temper/qa rewalks after landing and files findings back to fab. This plan writes no browser test.
- **Tests stay cheap.** A new case of one behaviour is a row or a line in an existing test, not a new test. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate.
- **No generic change in skein.** The page is made the same as a chat's; `skein-ui-web` stays as it is.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate, by their own tool. No journey holds an agent's page and this plan changes no wire, so none should go stale; if the focused replay says otherwise, `temper-client-journeys again` is generation, not a walk, and on `a key mark caught a moving page` it is retried at most three times.
