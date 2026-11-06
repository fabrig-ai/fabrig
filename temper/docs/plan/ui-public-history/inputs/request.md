# ui-public-history: public pages say what history they hold, and show what a stock queue landed

temper/qa's finding 430, settled as its pick b (fab's triage, 2026-11-06): temper/qa walks `rung-public` with work done while public; a restart's loss stays the design's default (`jig/docs/design/domain/people.md`, 13.9); temper/ui holds the two things below. Public reading stays off regardless of this plan: going public is the user's word. Titles aren't this plan's: they're with the user, and no title rule moves.

`browser.status.md` lists three limits `ui-public-2` left. This plan closes one and tells a stranger about the other two.

## 1. A stock queue's merges show on public Landings

**What's wrong:** a stock queue's Landings stay empty, since its changes land without a landing by turn. So a stranger reading a project whose work is landing sees "Nothing has landed yet."

**Ruling:** what a stock queue lands shows under the project's public Landings as a recent landing, in the same form as a landing by turn: "Its change:" and its landed commit, a short commit, under the bounds the index already has. Carry only what the public records may carry (`people.md`, 13.3): no words but a title, no person, nothing about a private project. A project that goes private loses them as it loses everything else public. Make `browser.md` 8.3 and `people.md` 13 agree, and take the limit out of `browser.status.md`.

If a public landing's link needs a record a queue merge doesn't have, decide it in the cut and say so: the fact to keep is that a stranger sees that something landed, and which commit.

## 2. The board says what it doesn't hold

**What's wrong:** landed work from before a project went public never shows, and a restart empties ended goals (13.9). A stranger reading "No goals yet." or "Nothing has landed yet." on a project with years of work is told something false.

**Rulings, the words as written here:**
- Under the board's existing line, in the same muted form: "Work that ended before this project went public, or before temper last restarted, isn't shown."
- The board's empty words become "No goals to show yet."
- Empty landings, on the board and the landings page, become "No landings to show yet."
- The projects page stays as it is.

These two limits stay in `browser.status.md`, as limits the page now states. Loading ended goals stays open (`people.md`, section 12).

## Left out

Titles; going public; temper/qa's new edition of `rung-public`, which is its own. Only focused and fuzzy run in the plan, as below: update any browser story this changes, compiled, not run. `browser_public`'s geometry story is being repaired under `red-50`, on the same file: leave its measuring alone and take it at catch-up. Two tasks at most.

## Sources

- `temper/docs/design/client/browser.md`
- `temper/docs/design/client/browser.status.md`
- `jig/docs/design/domain/people.md`
- `temper/crates/temper-client-view/src/public/lines.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
