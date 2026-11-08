# ui-drafter: the drafter, a chat charter of its own

Milestone P6 of `temper/docs/design/draft/whole-story.execution.md`: the drafter's charter (`temper/docs/design/draft/surfaces.md`, 11.2, and its instructions), handed here with the whole story's pages. The user's word of 2026-11-08: "we want all this by v0.1.0 ... move fast and parallelize". Nothing in the drafts is reopened.

## What it executes

- **A drafter charter in the stock:** a chat charter of its own, beside charter 1.
  - Its `instructions` are this plan's `inputs/drafter.txt`, taken exactly as written. No task edits its words. If a stock bound or test refuses any of it, stop with `Question:`, naming the bound and the bytes.
  - Like a chat, it reads the project's code through the forge and surveys with sub-agents where its model has them. It writes the draft as the chat's document, with no repository workspace of its own, and it may not call `change` or `goal`.
  - Its tools, hosts, run and models are a chat's, as `surfaces.md` 11.2 says.
- **A template that starts a design session** with the drafter: offered where the person starts a chat, kept as a session, and resumed from its draft and open points.
- **On the web,** that session reads as a chat with its draft in Split (`client/README.md`, 3.5), as a chat's document does today. Its "Start goal…" offers the project's ways of working, kind 7's tracked templates.
- **The design first,** where `surfaces.md`'s status lists lags for 11.2's drafter. Then the code.

The TUI's `/draft`, its pane and its sittings on the launchpad are the TUI owner's milestone T2, built on this. `plan.md` names the charter's and template's numbers and what T2 starts from.

## Its tests

Focused and fuzzy only:
- the drafter's instructions loaded byte for byte;
- its declared tools: no `change`, no `goal`, and no workspace;
- the template starting a session on the drafter;
- the stock's figures and bounds with it added;
- the web drawing the session's draft in Split, as a row of an existing test.

## Sources

- `temper/docs/design/draft/surfaces.md`
- `temper/docs/design/draft/whole-story.execution.md`
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
