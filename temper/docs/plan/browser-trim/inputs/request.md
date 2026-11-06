# browser-trim: temper's browser tier back under its budget

The user (2026-11-06): "re browser tests anything not particularly useful we can remove/trim? testing-strategy.md rules". temper's browser tier (`temper/tests/client-browser`) is over its 85 s CPU budget: 94.2 s, wall 85.9 s, 13 tests, at after-landing `ebc08ddafc43`. On 11-03 it was 42.2 s, wall 18.6 s, with 11 tests. **Don't raise a budget, a timeout or a limit.** Follow `skein/docs/foundation/testing-strategy.md`, 8.1's questions in order: does it belong to this tier; if so, can it be made faster; if neither, measurements go back to the user. `temper/docs/design/client/browser.md`, section 4 sets the tier's rule: "Only what a browser alone shows... Each names what only the browser shows." Moving a test to escape a budget stays forbidden: what it checks decides its place.

## The measurements (CPU, wall, at `ebc08ddafc43`, one machine under load: re-measure)

| Test | CPU s | Wall s | On 11-03 (CPU) |
|---|---:|---:|---:|
| `browser_playground::story_a_walks_on_the_built_playground` | 18.0 | 13.6 | ~18.6 |
| `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 15.9 | 23.1 | 4.7 |
| `browser_public::a_visitor_reads_a_public_project_and_loses_it_when_made_private` | 14.1 | 27.3 | none then |
| `browser_pages::latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` | 7.6 | 4.0 | new |
| `browser_sign_in::…keep_the_cookie_contract` | 6.6 | 4.2 | 5.7 |
| `browser_attacks::cross_origin…` | 6.0 | 2.2 | 4.7 |
| `browser_pages::enter_starts_one_chat…` | 5.4 | 2.4 | 2.7 |
| `browser_accounts` (two) | 4.4, 4.3 | 2.0, 1.7 | 3.5, 3.8 |
| `browser_authorize::…` | 4.0 | 1.6 | 4.0 |
| `browser_pages::closing_project_rename…` | 3.5 | 1.8 | new |
| `browser_task_pages::start_goal_opens_chats_for_a_draft` | 2.5 | 1.1 | new |
| `browser_github::…` | 1.9 | 0.9 | 1.9 |

## What to do, in order

1. **Remove what doesn't belong (fabrig/ui's reading; check it).** `start_goal_opens_chats_for_a_draft` presses a link and walks back and forward: `history_keeps…` holds history in the browser, and the link's target belongs in the client world. Remove it from this tier, adding the world test if none holds the link's target. Its file's header still describes an older test: correct `browser_task_pages.rs` and `browser.md`, 4.6 with it; if the file ends up empty, it goes.
2. **Read every other test against the tier's rule:** each must name what only a browser shows. Where a test walks a story a world holds, cut those steps and keep the browser's part; where nothing remains, remove it, with the assertion held lower. Keep every cookie, sign-out, attack and focus assertion. Say row by row what you kept and why.
3. **Find why `history_keeps…` tripled and took 23 s of wall, and why `browser_public` takes 27 s.** Wall far above CPU suggests a wait; don't assume it. A person's page that's slower to settle is a product finding, and the most valuable answer here: profile the page first (a re-render loop, a poll, a stream reconnect, a settling wait). Fix the cause where it lies, in the code under test or the test.
4. **Then the playground and the rest:** a missing cache or needless rebuild only; `story_a` is the tier's one proof of the built playground and belongs.
5. **Measure:** the whole tier, three runs on one idle-as-possible slot, per test and in total. **If it's still over 85 s, change nothing more:** the table, before and after, goes in the report for the user to decide.

## How the cut shapes it

Two or three tasks. Red first wherever a product cause is fixed. No `#[ignore]`, no retries, no longer `within`. Beside it, `ui-walk-3` and fabrig/temper's `chat-reading` touch these pages: expect catch-ups. `look-round-11` isn't started.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `skein/docs/foundation/testing-strategy.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
