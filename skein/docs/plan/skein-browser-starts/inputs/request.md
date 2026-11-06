# skein-browser-starts: fewer Chromium starts in skein's browser tier

skein's browser tier has a budget of 145 CPU s (`tools/tiers/budgets.toml`) and has passed it in many of its recent runs. It had 81 tests on 11-04 and has 102 today, and each starts a Chromium of its own. A Chromium start costs about 0.6 CPU s on its own, with a blank page as with the bundle, and no flag tried changes that, so starting takes around half of the tier in its better runs. **The lever is fewer starts, without losing what only a browser shows.**

| Binary | Tests | CPU s | Wall s |
|---|---:|---:|---:|
| `skein-browser-world::browser_real` | 33 | 49.4 | 29.9 |
| `skein-ui-web-tests::browser_harness` | 35 | 39.1 | 18.7 |
| `skein-ui-web-tests::browser_stories` | 17 | 29.8 | 40.6 |
| `skein-ui-web-tests::browser_example` | 12 | 20.3 | 10.8 |
| `skein-browser-world::browser_facts` | 5 | 6.0 | 3.0 |

One run, at 144.7 CPU s. Don't read these as exact: the same code reads from 99 to 167.

## What's wanted

**1. A table in `plan.md`, one row per test of the tier, all 102:** what it checks, and whether it stays, merges into which, or goes where. Any test added by the time you cut gets a row too. **Aim: at most about 70 tests at the end.** That's an aim, not a licence: if fewer can go honestly, say so and stop there.

**2. Does each belong in the browser tier?** (8.1's first question.) What it checks decides, never its cost. `browser_harness` is the place to start. Many of its tests check what the harness does with a child process or a peer: arguments, environment values, standard input, exit codes, a link's deadline, a command's teardown, a peer's tokens and shutdown. Examples: `command_arguments_receive_peer_and_server_addresses`, `command_environment_values_receive_peer_and_server_addresses`, `closed_command_stdin_receives_only_eof`, `exited_rejects_a_different_code_naming_the_command`, `repeated_peer_arguments_each_reach_the_server`, `colliding_peer_tokens_reach_their_own_hosts`, `a_lent_peers_assertion_names_it_in_the_failure`. Where no real browser's behaviour is what's checked, the test goes to the lowest world that shows it, with the fake browser, as long as a focused test there costs milliseconds and not a process tree. Where a real browser is part of what's checked (the kit's facts about Chromium, `skein/docs/design/browser.md`, 9), it stays. If the harness can't run without Chromium for one of these, say so, and it stays.

**3. Can tests share a start?** (8.1's second question: setup repeated.) `Harness::run` already takes several stories, each with a person of its own in a browser context (`skein/docs/design/ui-web.md`, 14.1). Tests of one binary that use the same server and setup become one test of several stories or steps. `browser_real`'s `chromium_*` facts, which each open a page in a fresh browser, are the first to read. A merged test must still say which story's step failed. It reruns whole on a flake, and a failure may hide the steps after it: taken, for the budget. Keep apart what truly needs a fresh browser, such as a test of a browser's start, exit or its first page.

**4. What two tab tests wait for.** `tab::a_tab_opens_only_from_a_press` reads 26.8 s of wall for 3.7 CPU s, a quarter of the tier's wall, and `chromium_lists_a_pressed_tab_and_its_cut_opener_but_blocks_an_unpressed_opening` reads 5.6 s for 1.8. Find by reading what each waits on. If a step waits out a deadline to prove something didn't happen, prove it by an event or a shorter bound that shows the same. If reading doesn't show the cause, say so and leave it: nothing here may be run to find out.

**5. Nothing is lost.** Every assertion stays, as a step or as a lower story that exists first and passes there, failing first where it can. Say so per row.

**6. Update the docs that name these:** `skein/docs/design/browser.md`, 9, and `skein/docs/design/ui-web.md`, 14, if what they say about the tier changes.

Three or four tasks, with one binary's merges in one task. `walk = false`. Effort medium.

## Limits

- **The browser tier isn't run in this plan, not a story of it nor by name.** A merged test is written and compiled (`cargo check --tests`). Lower stories in focused run there. The plan is complete when it lands, with no walk, render check or throwaway driver, and we repair if the tier flags it after landing.
- Budgets, timeouts and deadlines are the user's. No test's deadline widens to make room for a merge: if a merged test would need a longer one, it stays two tests, and say so.
- Other trees' tiers call skein's stories and its harness (`skein/docs/design/ui-web.md`, 14). Keep what they call working, or change their callers in the same task; don't change what those stories check.
- Stay out of product code: `skein-ui-*` and `skein-browser`'s behaviour don't change, beyond what item 4 finds in a wait. A product bug goes in the report.
- **Overlaps:** other plans add stories to `skein/tests/ui-web`. Expect catch-ups.

## Sources

Too large for a source, so read by section: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `skein/docs/design/browser.md`, 9; `skein/docs/design/ui-web.md`, 14.

- `skein/tests/ui-web/tests/browser_harness.rs`
- `skein/tests/ui-web/tests/support/tab.rs`
- `skein/tests/browser/tests/browser_real.rs`
- `skein/tests/browser/tests/browser_facts.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **8.1's order, always:** does the test belong here; if so, can it be made faster, at the cause. Moving a test to escape a budget is raising it.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **A test's helper that a merge makes unused goes in the same task.**
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
