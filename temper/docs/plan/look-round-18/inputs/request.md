# look-round-18: Ask temper's place, the render check's two misses

A look plan. The render check after `ui-ask-m1` landed, of look request `ask-temper-place` (`origin/look-requests`, trailer `Look-request: ask-temper-place`, at `180f76e480`; page `temper/docs/design/client/journeys/ask-temper-place/01.html`), passed everything but two. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5.

## Tasks

1. **An Ask temper chat's bar shows " · Ask temper".** The tag is in the DOM but hidden at every width. `temper.css`, near line 1516, has `.temper-session-right > p:not(.temper-byline)` at `display: none`, and that hides it. Gallery `configure-chat-doc` shows the same. Show the tag as the request's page draws it, at 1440, 1000 and 420, without bringing back any other paragraph that rule hides now. The gallery's state for an Ask chat is regenerated with it.
2. **The gallery's first-run states offer Ask temper as a fresh box does.** `project-landing` (the main journey's print), `home-two-left` and the other first-run states have no Ask row and no phone tab, because their worlds lack template 5 as an offered template. A fresh live box offers it. Give those worlds the stock's offered templates as a fresh box has them, then regenerate their prints, goldens and `names.txt` rows as the common notes say.

The Ask page's secondary heading, "How should temper run Hello?", stays for now: later rounds with the user will draw that page. A row's link being the newest Ask chat once one exists is as the request says.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/client/journeys/ask-temper-place/01.html`


## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
