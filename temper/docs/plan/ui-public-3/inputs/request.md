# ui-public-3: making a project public says what will show, and the public pages' last words

The user's answer, 2026-11-06, on whether agent-written titles cross on public pages: **titles stay, and making a project public says that anything said in its chats may show.** Going public itself stays the user's word: public reading stays off, and nothing here turns it on.

## 1. Saying so at making a project public

A title is the first line of a spec an agent wrote from what a person said, so a chat's words can reach a stranger. temper/qa's walks saw whole instruction lines in titles. Say so where a person makes a project public, before it happens, in these words:

"Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too."

Find every place a project goes public, and put it in each: `temper ctl project amend --visibility public`'s answer or its help; the configuring chat's instructions and its proposal, which are temper/ui's; a page, if one offers it. If a person can do it with no warning in between, say so in the report. `jig/docs/design/domain/people.md`, 13.3 keeps titles crossing; record there that this is the user's decision, reaffirmed, with these words as its condition, and take it out of `browser.status.md` if it's listed as open.

## 2. temper/qa's two notes from `r-public`

- **finding-434, finding-435:** a stock landing's page ends with a bare "0 stale proofs", with nothing to explain it. Don't draw a zero.
- **The board's link to Landings** is a small muted "LANDINGS" with nothing under it, reading like a heading to an empty section. Make it read as a link to them, per `temper/docs/design/client/browser.md`, 8.3.

## 3. The help

`temper/docs/help/open-public-reading.md` hasn't moved since `ui-public-2`: make it say what the pages now show: a goal's spend alone, a row's words, Landings holding a stock queue's merges, and that no link goes to the forge. It gives the two limits the board states, and section 1's words.

## Limits

One or two tasks. No browser, no screenshots and no render check: the user's rule is that a plan runs focused and fuzzy alone and is complete when it lands. temper/ui and temper/qa look after it lands. Stay out of `browser_public`'s measuring, which `ui-public-story` is repairing. A browser story's words that change are updated and compiled, not run.

## Sources

- `temper/docs/design/client/browser.status.md`
- `temper/docs/help/open-public-reading.md`
- `temper/crates/temper-client-view/src/public/lines.rs`
- `temper/crates/temper-client-view/src/public/pages.rs`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
