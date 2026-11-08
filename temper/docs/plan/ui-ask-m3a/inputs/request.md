# ui-ask-m3a: the ways, their paths, and the views that draw them

Step 3 of `temper/docs/design/draft/stock-ways.md`'s section 7, and Ask temper's milestone M3a (`temper/docs/design/draft/ask-temper.execution.md`, M3a), exactly as they say. The user's word of 2026-11-08 settled the draft through laptop/web, and nothing in it is reopened here. Step 2's prerequisites are on main: each project's stock rule, branch patterns, and Draft first's roles. It's UI with looks.

## What it executes

- **`stock-ways.md`, section 4:**
  - the three stock ways as data in the `stock` module, their words the stock's (section 2);
  - one path, derived engine-side from what applies to a project's landing branch, with every client drawing what it's given;
  - the order rule: a typical run with one repair;
  - a stop whose setting is absent isn't drawn;
  - no agent stop below the line;
  - under `turn`, the turn's own steps.
- **Section 5:** the forge's `ways { project }` read, watched, with each way's words, its path over this project, its items, the current way or none, and whether the reader may change settings. Also "Use this" and "Keep it" as jig's `ChangeSettings` of the read's items, a project matching no way as "Yours", and a stop's card from the read.
- **Section 6 and `ask-temper.md` 6.2:** `see ways`, and the views `way NAME`, `path OWNER/NAME [BRANCH]`, `proposal tN/K` (its path before and after) and `charter N`, from the same derivation. The browser draws them in a reply.
- **`ask-temper.md`, section 8:** `ways` on the `see` line, and M3a's part of Showing as main holds it after `fd4f43447f`: the first paragraph, the `view` example, the Views line naming `path`, `way`, `charter` and `proposal` alone, and the ways paragraph. M3b's part stays out. Those are Ask temper's instructions, taken as written.
- **The design first,** where the status files lag: `../domain/landing.md` 13.1, the forge's reads, `ask-temper.md`'s views, and their status entries. Then the code.

The look request `ask-temper-ways` is the next plan's, `look-ask-ways`. Draw what the views and the read need in the browser, and leave the opening page's look to it.

## Its tests

`stock-ways.md`, section 8, focused and fuzzy only:
- **focused:** the three ways' paths over a project with CI and both tiers, exactly the look request's three pages (`ask-temper-ways`, on `origin/look-requests`); with neither, the same less "check" and the tiers' stops; `ways`'s current way and "Yours"; "Use this" refused for too few approvers; "Keep it"'s change marking To begin's first step done; each view's rendering;
- **fuzzy:** the derivation over random rules, never an agent's stop for an update or a merge.

## Who hears first

`plan.md` names the `ways` read's shape and the views' blocks, for the TUI's owner, who draws views in the TUI. It names any change to `temper-engine-domain` or `forge_route.rs` for the owner of coding runs.

## Sources

- `temper/docs/design/draft/stock-ways.md`
- `temper/docs/design/draft/ask-temper.md`
- `temper/docs/design/draft/ask-temper.execution.md`
- `temper/docs/design/domain/landing.md`


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
