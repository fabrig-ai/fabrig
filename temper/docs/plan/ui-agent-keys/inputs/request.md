# ui-agent-keys: PgUp and PgDn read back on an agent's page

temper/qa's post-landing check at `0868043f4d`, on fake backends, high: **PgUp and PgDn do nothing on an agent's page.** From the focused message box, 0 px on both keys, while text streams and just after a turn ends, and focus stays in Message. A goal's untracked page behaves the same. The page shows no error. A chat's page is fine: PgUp moved a screen of 520 px, the place held while words arrived, and PgDn came back. Report on fab's host: `~/src/fabrig/coord/codex-kit/reports/long-chat-1.md`; evidence on hetzner8 under `~/src/fabrig/fab/walks/long-chat-1/keys/`.

**Likely cause, read and not traced further:** `temper-client-shell-web/task-pages.css`, lines 49 and 50, makes `.temper-thread` the scroller on that page and gives `[role=log]` `overflow: visible`, while skein's `ScrollScreen` (`skein-ui-web/src/page.rs`, near 882 to 896) acts on the log, which has nothing to scroll.

## What this plan makes true

An agent's page and a goal's untracked page read back by PgUp and PgDn from the message box as a chat's does, a screen at a time, with Latest and the kept place working as on a chat. Take the smallest fix that makes one scrolling element hold: the log as the scroller on these pages, as on a chat, is the first to try, since a chat's page already proves it. Check that nothing else on those pages relies on `.temper-thread` scrolling, wide or at 420. A generic change in skein isn't wanted unless the page truly can't be the same; if so, say why.

A focused test in the lowest world that shows it, if one can hold which element scrolls: a row of an existing table. temper/qa rewalks after it lands. No browser here, no render check, by the user's rule that a plan is complete when it lands. No new browser test.

Beside it, and only if it's a line: while a long chat's head waits for earlier rows, repeated PgUp moves nothing for a while (88 s and 293 presses to the start of a chat after a sign-in, with 238 sample pairs of no movement). Read why. If a press is simply dropped while a read is pending, keep it. If that's more than small, leave it, and say so in the report.

## Sources

- `temper/crates/temper-client-shell-web/task-pages.css`

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
