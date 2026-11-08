# walks-lower: process-temper's fake-box walks go where what they check belongs

The user, 2026-11-01, on process-temper: its "fake-based coverage shrinks to about one short end-to-end test against fakes that proves the wiring through `main` (not a full story); the fakes-only stories' behaviour goes to lower worlds or is deleted as covered there." And 2026-11-06: "tests should stay within budget."

Since then, 12 walks have grown back under `temper/crates/temper/tests/fake_backends/walk/`. On 2026-11-06, in after-landing's records, they take about 81 of the tier's 340 CPU s, as medians per test. The tier's budget is 360, and 5 of that day's 37 runs passed it.

**What decides a test's place is what it checks, never its cost** (skein's `docs/foundation/testing-strategy.md`, 8.1). Moving a test to escape a budget is raising it. Every ruling below rests on what the test checks. **None goes to the browser tier:** `temper/docs/design/client/browser.md`, section 4 says temper's stories are the joint world's and the real loop's and are never run again in a browser, and these show nothing only a browser shows.

## The rulings

On what a person meets:

**Stay on the box, as paths through `main`:**
- `here_key`: the user's one short end-to-end test.
- `sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create`: the plan path. The box starts the fake issuer as a process, and `--fake-backends` journeys stand on it.

**Go lower. What must survive each move:**
- `status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` goes to `temper/tests/client/tests/suite/status.rs`, beside `a_refused_status_run_leaves_home…`. Keep: a person's plan is the only thing that can pay, with no deployment key; no brief before a landing or within the sitting; a brief past it; the plan still usable after. It guards a bug a walk found (Home's brief unfunded), so show it red with that funding reverted.
- `status::home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts` goes to the same file. `a_restart_keeps_the_rows` is already there. Add the count after a real landing, and the skip surviving a client restart.
- `away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` goes to `suite/away.rs`, in the same order: on, a goal decides, Decided carries on, overturn asks again and shows "Still needs you", off and it waits.
- `goal::a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` goes to `suite/task.rs`, beside `a_finished_goals_page_shows_its_report`. Keep the route from the chat to the Goals row to the report.
- `copy::readys_two_copies_are_adopted_in_order_with_the_first_as_home` goes to the joint world. Keep Ready's burst in one client pass, the order, and the first as home.
- `sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create`: `browser_tab` walks the same path to Ready; whatever only this one checks goes to `suite/connect_model.rs`.
- `sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create` goes beside the fake issuer, in its own real-loop test: held sockets don't block a sign-in, and past `CONNECTIONS` it rejects and logs. It guards another bug a walk found, so show it red too.

On the rest:

- `note_parts::a_long_document_is_built_in_parts_on_the_fake_box` goes to `temper/tests/real-loop`, whose hosted agents have the real tools. Keep: the bounded calls and both refusals, a fresh run's brief finding the document, and revising it by lines.
- `chat_document::a_chat_document_is_written_and_revised_on_the_fake_box`: if `note_parts` there holds all it checks, deleted as covered, saying so; otherwise beside it.
- `copy::a_copy_is_made_and_adopted_through_ctl_on_the_fake_box` goes to the joint world beside its sibling, or is deleted as covered if a story of `temper/docs/design/domain/forge.md`, 4.5 already holds it. Read first.

## What's wanted

1. A table in `plan.md`, one row per walk: where it goes and the story that takes it, or stays, with why. Any other walk on the box by the time you cut gets a row too.
2. **Nothing is lost.** Each behaviour above gets its lower story first, failing first where it can, and run there; only then does its walk go. Say so per row.
3. **If a lower version would have to stub what a walk proves, it stays,** with that reason in your report. The lower files were checked to exist on main, not that each can hold these steps. Say where one can't: that's no failure.
4. A walk's quiet wait, such as a sitting, doesn't move as a wait: a lower world's clock is the point of moving it.

**A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5):** the walks that stay are compiled, not run, and the plan is complete when it lands. After-landing is the proof, and its figures are read there; findings after landing are filed back to fab. Stay out of `temper-client-*` crates; a story in `temper/tests/client` is fine beside its others. Budgets and timeouts are the user's. `walk = false`.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories: a walk that stays, or a new process story, is written and compiled (`cargo check --tests`), not run. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
