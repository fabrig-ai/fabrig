# browser-cap: temper's browser tier back to its twelve tests

The tier has 17 tests. `temper/docs/design/client/browser.md`, 4.3, is the user's rule: "at most 12 tests", and "past it, stories that share a setup become one test of several steps before any browser-only proof is dropped, and what must leave the browser goes to the lowest world that still shows it, named in its line of 4.6; the cap stays." 4.6 lists twelve. This plan brings the tier back to them.

**What a test checks decides its place, never its cost** (`skein/docs/foundation/testing-strategy.md`, 8.1). Moving a test to escape a budget is raising it, so nothing leaves the browser unless no browser-only proof leaves with it. No budget, timeout or tier membership rule changes.

## The five over the cap

**1. Reading is 4.6's 14, one test, and is four.** `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`, `earlier_words_stay_put_without_browser_scroll_anchoring`, `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` and `a_short_chat_opens_whole_without_a_reading_status`, all in `browser_pages.rs`, become one test of several steps: one `temper serve`, one browser, and as few histories as hold what they check. Each story's assertions stay as its steps, in its order. `chat-reading-c`'s spec kept the short and unanchored stories apart because each is one page load and they expect different things, so **steps with their own page loads meet it.** Keep `overflow-anchor: none` inside the unanchored step, so the other steps stay on default anchoring. Read `reading_tier`, `reading_chat` and `reading_position` first. If a step can't change anchoring on a live page, say so; two tests for reading is then the most, and name what else gives way to stay at 12. A merged test reruns whole on a flake, and a failure hides the steps after it: taken, for the cap.

**2. `the_welcome_sign_in_opens_a_tab_and_moves_on_to_ready` isn't a line of 4.6.** 9 is `ConnectModel`'s sign-in in a new tab, held by `the_sign_in_opens_a_tab_and_the_page_moves_on_to_ready`. Both stand on `Tier::accounts_sign_in`. The new tab is what only a browser shows, and 9 shows it. **The welcome is the first thing a person meets, so it stays in the tier as a step of 9,** with its addresses `/welcome?at=model` and `/welcome?at=ready` and its one authorization and one code exchange. If a step truly can't hold it, what only the welcome checks goes to `temper/tests/client/tests/suite/connect_model.rs`.

**3. `browser_call_line::a_run_of_calls_opens_to_its_rows` isn't a line of 4.6.** Its file says what it's for: a chat's folded call run stays open across new words. What only a browser shows is `details[open]`, which the browser holds and the model doesn't, kept across a patch. The rows hidden, then shown on a press, are a world's, if `tests/client` doesn't hold them already. So: the rows go to the lowest world that shows them unless already held, and the open line kept across a reply becomes steps of a test that already starts a chat, most likely 4 and 5's `enter_starts_one_chat_and_the_composer_keeps_the_focus`. Its `Tier::demo_asked` script may have to become that test's setup. If 4.6's skein parts already prove this for every application (`skein/docs/design/ui-web.md`, 14.2, and `temper/docs/design/client/browser.md`, 4), it goes as covered, saying so. Don't add to skein's browser tier.

## What's wanted

1. **A table in `plan.md`, one row per test of the tier as it stands,** all 17: stays, merges into which, or goes where, with what it checks. **Twelve at the end.** Any test added by the time you cut gets a row too; `browser_model_picker::the_model_picker_at_the_composer`, if it's on main, merges into 4, The composer.
2. **Nothing is lost.** Every assertion the five hold stays as a step, or as a lower story that exists first and passes there, failing first where it can. Say so per row.
3. **`browser.md`'s 4.6 names the twelve,** with its "Not in the tier" list carrying whatever left. 4.1 and 4.2, if a merge changes what they say.
4. **Keep what a merged test reports:** a failure must still say which story's step failed, by the step or a label, so that a red reads as well as it did.
5. **Shared setup, not new machinery.** Prefer the tier's existing helpers. One history made through ctl that several steps read is the aim; a new harness feature isn't, and if one seems needed, say why in the report.

## Limits

- **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** The browser tier isn't run, not a story of it nor by name: a merged test is written and compiled (`cargo check --tests`). Lower stories in `temper/tests/client` run there. The plan is complete when it lands, with no walk, render check or throwaway driver. After-landing is the proof, and we repair if it flags.
- Budgets, timeouts and the cap are the user's. No test's deadline widens to make room for a merge: if a merged test needs a longer one to fit, it's two tests, and say so.
- Stay out of `temper-client-*` product crates: no product change is wanted, and if a merge shows a product bug, report it rather than fix it. `temper/tests/client-browser` and a story in `temper/tests/client` are in scope.
- `walk = false`. Effort medium. Three or four tasks at most.
- **Overlaps:** `watch-room/red-50` changes `browser_public` and the world's measuring; `model-catalogue-2` and `look-round-11` may add to `temper/tests/client`. Expect a catch-up, and keep this plan's edits to `browser_pages.rs` in one task.

## Sources

Too large for a source, so read by section: `temper/docs/design/client/browser.md`, 4, 4.3 and 4.6; `temper/tests/client/tests/suite/connect_model.rs`.

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`
- `temper/tests/client-browser/tests/browser_all/browser_accounts.rs`
- `temper/tests/client-browser/tests/browser_all/browser_call_line.rs`
- `temper/tests/client-browser/src/lib.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A test's helper that the merge makes unused goes in the same task.**
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
