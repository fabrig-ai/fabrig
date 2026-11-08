# ui-ways-rules: a project's way is its own

Step 2 of `temper/docs/design/draft/stock-ways.md`'s section 7: its section 3, whole, as the draft says. The user's word of 2026-11-08 settled the draft through laptop/web, and nothing in it is reopened here. It's Ask temper's ways work, and UI with looks.

## What it executes

- **3.1, the stock's rule, per project:** kind 1 loses its stock rule. Kind 2's rule 1 becomes a code constant for every project, as kinds 6 and 7 have: every branch of each of the project's repositories, enforced, CI, up to date, and gate 1, an agent's review, blocking at a clean head. A project's rule 1, removed or reset, goes back to it. Every reader and writer of kind 1's stock moves with it, as 3.1's last point lists.
- **3.2, a project's rule names a branch:** a kind 2 pattern may begin at `branch`, and covers that branch in each repository of the project, those adopted later among them. One beginning at `forge` keeps its meaning. The forge admits `branch` for kind 2 alone, and `propose_settings` and Settings write either form.
- **3.3, Draft first's roles in the stock:**
  - **Charter 5's first edition** is this plan's `inputs/charter5.md`, taken exactly as written. `## shared` is the charter's `instructions`. Each `## part, on: …` is one part, its `on` set as the heading names it, its text running to the next heading. Its author is the owner of the goal's charters, and no task edits its words. If a stock bound or test refuses any of it, stop with `Question:`, naming the bound and the bytes.
  - **Template 4** is as `../protocol/configuration.md` 13.9's, at the stock's models, with its own goal branch among its resources.
  - **The delegation:** the rules' and policy 1's delegation reaches charter 5, role 0 and the merge job, as 13.9 changes them.
  - **The reference's settings file** (`temper/docs/examples/reference/settings.json`) keeps its own editions over the stock, as 3.3 says.
- **The design first:** `configuration.md`, sections 13.2 to 13.4 and 13.9, says what this makes true, with their status files. Then the code.

## Its tests

Focused and fuzzy only, from the draft's section 8:
- **focused:** the stock's rule 1 per project and its reset; a branch pattern over two repositories, one adopted after the rule; the stock's delegation to charter 5; charter 5's parts loaded as written, with their `on` sets;
- **fuzzy:** a branch pattern and a full one, against random branch names.

Tests and worlds that assume kind 1's stock rule change with it.

## Who hears first

`plan.md` names, as one list each:
- the delegation's changes, which the owner of the goal's charters reviews before landing, as its generated settings grant executors by number;
- the landing rules' scope and pattern change, for the TUI's owner and the same owner.

## Sources

- `temper/docs/design/draft/stock-ways.md`
- `temper/docs/design/protocol/configuration.md`
- `temper/crates/temper-service/src/stock.rs`


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
