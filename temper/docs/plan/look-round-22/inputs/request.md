# look-round-22: the outside pages' render-check misses

A look plan: the render check after `look-outside-pages` landed, for `Look-request: outside-pages` on `origin/look-requests` (`43a18a5042`). Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. Everything else that check covered passes, the Google return's `then` among it.

## Tasks

1. **The welcome's frame, signed in, shows the person's name and "Sign out"** at the top bar's right, as every other outside page does. Its early return leaves out the top bar's end.
2. **While `me` is read, the top bar alone.** Today `/chats` draws "Project · chats", its filters and "+ New chat", and `/chats/new` draws its composer, before "No project yet" settles at about 940 ms. No page body draws until `me` is read.
3. **The form labels:** "Code" (02) and "Project name" (08) are 13 px on a 20.8 px line. The request has 14 px on 22.4 px. Match it, so the field and its act sit 1.59 px lower, as the request's pages do.
4. **Public pages have "Sign in" at the top bar's right** with or without Google sign-in, as the request says, unqualified. Without Google, it leads to the sign-in page's printed-link words.
5. **The night welcome at 420×900 fills the window:** today a 53 px strip of the day's `#f7f7f5` shows under the night ground.
6. **The server's Google endings use the client's typographic apostrophe:** "didn’t", not "didn't", in every ending.

A browser case is a row of an existing table test, within the cap of 12.

## Sources

- `temper/docs/design/draft/look-requests.md`


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
