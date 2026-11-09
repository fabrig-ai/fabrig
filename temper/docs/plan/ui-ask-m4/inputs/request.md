# ui-ask-m4: Ask temper remembers, and the person's preferences

Milestone M4 of `temper/docs/design/draft/ask-temper.execution.md`, *Remembering*: `temper/docs/design/draft/ask-temper.md`, sections 5.4 (`remember`) and 7 (its memory), with the Remembering part of section 8's instructions, as its entry says. It needs jig's party scope for notes first, a design of jig's, which is shared. Its page, "What temper remembers about you", sits in the person menu beside Deployment, where `ui-p1-rail` left its slot (`temper-client-view`'s `frame.rs`).

## What it executes

- **The design first:** jig's party scope for notes, and the status lags the entry names.
- **Then the build:**
  - the `remember` tool;
  - Ask temper's memory of the person, across its chats;
  - the Remembering part of its instructions, which is mine: the plan writes it from section 8 as the draft gives it, and names it in `plan.md` for my review before landing;
  - the page in the person menu's slot, where the person reads and removes what temper remembers.
- **The rule:** draw what the engine holds, with no placeholder.

## Its tests

Focused and fuzzy only:
- a note remembered, read back in a later chat, and removed from the page;
- party scope, so nobody reads another's notes;
- the page's gallery states, day and night, at a laptop's and a phone's width.

## Who hears first

`plan.md` names every interface change in jig, shared, and every change to `temper-client-domain`, for the TUI's owner, and to `temper-engine-domain` or `forge_route.rs`, for the owner of coding runs.

## Sources

- `temper/docs/design/draft/ask-temper.md`
- `temper/docs/design/draft/ask-temper.execution.md`


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
