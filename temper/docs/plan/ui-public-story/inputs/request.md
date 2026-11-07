# ui-public-story: the public geometry story stops failing on "visible text has a box"

A red's repair. `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` has failed in about half of after-landing's runs since `ui-public-2` landed (`red-50`, `red-51`, `red-53`), at `temper/tests/client-browser/tests/browser_all/browser_public.rs`, `TextBoxes::next`: `visible text has a box`.

**Most of the work is done, on a branch fab can't land.** `watch-room/red-50` repaired it on a plan that has since retired, so its last question has no answer call, and its branch has no trunk. Its tip is `origin/watch-room/red-50`, `d3ca9d872e`; its report is on fab's host at `~/src/fabrig/fab/runs/watch-room/red-50-3/report.md`, and the essentials are below.

## One task

1. **Merge `origin/watch-room/red-50` into this plan's task branch.** It holds: `BrowserFace::boxed_matches`, one boxed query per measurement stage instead of a full search per `.nth`, with its regressions, and missing rectangles still failing with the node's text. Its reverts of the idle-connection work are right: main's `c592067aa0` holds that. Check the merge holds what its report says, and take main's side of anything already there.
2. **Answer to its question, from temper/ui, who owns this story: its option 2, for now.** Keep `skein_browser::boundary::Seen` as it is. Reading only a wrapped text's first quad is older than this failure and isn't its cause: leave it as the limit `browser.status.md` already records. The foreman's ask to account for wrapped fragments stays as that line.
3. **Read the cause, once, from what the branch now prints.** A likely reading, not proved: a public page reads again on a timer, so a node found by one query can be gone from the next, which batching closes. Run the exact flagged story by name, alone, through `heavy` with `TEMPER_BUNDLE=none`, as many times as a flake needs to show, at most five. If a missing box shows again, name the node and the stage, and fix that: substantive text with no box stays a failure. A whitespace or zero-size node that the page really draws is a fix of what the query counts, with a focused test.
4. **Report the other flake it saw:** sign-out step 19, an unexpected `/jig/v1/stream` 401, once on a loaded machine. Fix it only if it recurs in those runs.

No budget, deadline or tier membership changes. The tier isn't run; the story by name, alone, is this red's exception. Keep every field, gutter, wrapping, navigation and overlap assertion. `browser-cap` (temper/tiers) follows on `browser_pages.rs`, and leaves this file alone.

## Sources

- `temper/tests/client-browser/tests/browser_all/browser_public.rs`
- `temper/docs/design/client/browser.status.md`

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
