# look-round-21: phone Ask's composer uncovered, and the gallery's first-run Ask

A look plan: the render check after `look-round-20` landed, for `to-begin-two-steps` on `origin/look-requests`, found by its trailer. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. Everything else that check covered passes.

## Tasks

1. **On a phone, Ask temper's tab bar covers its composer.** At 420, live, Start opens Ask temper with the three restricted tabs (Home, Ask and Settings) at `y=937…1000`. The composer sits at `(16, 899.25, 388, 56)`, so its bottom 18.25 px and the model picker are under the tabs. The reservation meant for it, at `temper/crates/temper-client-shell-web/temper.css` near line 3076, nests `:has()` inside `:has()`, which Chromium refuses (`CSS.supports` gives false), so its 64 px never applies. Reserve the tab bar's room in a form browsers accept, such as a class or attribute the shell sets when the bar shows, so the composer and picker sit fully above the tabs on every page that has them. Hold it with a row of an existing browser table test, within the cap of 12.
2. **The gallery's first-run Ask states draw what a fresh project shows.** `ask-first-run`, `ask-typing`, `ask-sending`, `ask-refused` and `night-ask` still show Needs you, Chats, "+ New chat" and More ways to start work at 1440 and 1000, and no tabs at 420. Live, Start gives the restricted rail and three tabs. Give those states the unfinished To begin's facts, then regenerate their prints.

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
