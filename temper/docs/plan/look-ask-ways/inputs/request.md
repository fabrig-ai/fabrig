# look-ask-ways: Ask temper's opening as ways to pick, drawn

Step 4 of `temper/docs/design/draft/stock-ways.md`'s section 7: the look request `Look-request: ask-temper-ways` on `origin/look-requests`, found by its trailer. Its whole message is the request, "States the pages do not show" and "Before it lands" above all. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5. The user's word of 2026-11-08: it works for real, with every prerequisite in place, and no row of its account is deferred. Those prerequisites are now on main:
- section 3 (`ui-ways-rules`);
- sections 4 to 6, the `ways` read, the derived paths and the views (`ui-ask-m3a`);
- a box owner's link on the owned Forgejo (finding-444's repair).

## What it builds

Every state the message names:
- no heading, and the lede alone;
- the three tabs in one control, ordered from more of you to more of your agents, the current one marked "now";
- each tab's line, its path drawn from the `ways` read (stops, marks, the line, main's bar, dimmer for a goal's branch), its legend and its acts;
- on a phone, the path down the page;
- a stop's card under the path, its caret at the stop, from the read;
- "Use this" as jig's `ChangeSettings` of the read's items, with a toast naming the revision, and a refusal in words;
- "Keep it", marking To begin's first step done while it's left;
- "Yours" for a project matching no way;
- no act for a person who may not change settings;
- the terminal's path as text;
- the opening gone after a first message;
- what it replaces, gone.

Nothing on the page is drawn by hand: every path, mark and card comes from the `ways` read. The tab and card switch in place, as the page's state.

## Its tests

Focused and fuzzy only:
- the opening drawn from a `ways` read over a project with CI and both tiers, matching the request's three pages;
- the same with neither, without "check" and the tiers' stops;
- "Yours";
- "Use this" refused for too few approvers, in words;
- "Keep it" marking the step done;
- no acts for a reader who may not change settings.

A browser case is a row of an existing table test, within the cap of 12. Journeys that the opening's change makes stale are regenerated with `temper-client-journeys again`, as generation.

## Sources

- `temper/docs/design/draft/stock-ways.md`
- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/ask-temper.md`


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
