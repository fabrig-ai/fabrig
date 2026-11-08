# ui-ask-m1: Ask temper, alone and small

The user's word, 2026-11-07: carry out the Ask temper draft. This is its first milestone, M1, *Alone and small* (`temper/docs/design/draft/ask-temper.execution.md`, 3, M1). It is its own input, with `temper/docs/design/draft/ask-temper.md` as the source. Build it greenfield beside the configuring chat. Charter 3 and template 3 stay as they are until the user says otherwise.

**M1's "needs first" is settled.** On 2026-11-07, with Claude Code 2.1.312 on a Claude subscription, run from an empty directory that is no repository, one request with `--system-prompt-file` in place of `--append-system-prompt-file` (plus `--tools ""` and `--disable-slash-commands`) answered, exit 0, with nothing on stderr. So 4.4's replaced prompt is the build, with no fallback to appending.

## What it executes

Exactly M1's entry: `ask-temper.md`, sections 1, 4, 5.1 in part, 5.2, 5.3, 8 and 9.
- Charter 11 and template 5 in the stock beside charter 3 and template 3, charter 11 in the rules' and policy 1's delegation, and "Ask temper" on Settings.
- No repository and no workspace for its runs (4.2), and its tools declared alone, in the asking family (4.3).
- Claude Code's prompt replaced for a run with no workspace (4.4).
- `see` with configuring's addresses (`settings` and its members, `settings landing`, `settings gates`, `revisions`, `revision N`, `proposal`, `help`, `file` and `more`). Every other address answers that it isn't there yet.
- `propose_settings`, with charter 3's mechanics moved into it (5.2), `document` (5.3) and `wait`.
- Section 8's instructions as `ask.txt`, without their Showing and Remembering parts.

Write the design sections it needs first, from `ask-temper.status.md`'s lags, then the code and its tests. Close the lags it closes, and set M1's state in the execution file's table.

## Its tests

`ask-temper.md`, section 10, focused and fuzzy only:
- what temper gives the chat stays within its bound on both shapes;
- the four tools are declared, and no workspace;
- Claude Code's arguments on the fake program;
- M1's addresses, each as a row of one table test where it fits.

M1's walk follows its landing, outside this plan. The user judges the instructions by trying them.

## Shared pieces

Each gets a line in `plan.md` naming what changed, for the heads-ups:
- smith's Claude Code start (4.4);
- the new host tools, in `temper-engine-domain` and `forge_route.rs`;
- `temper-client-domain`, if the Settings entry touches it.

## Sources

- `temper/docs/design/draft/ask-temper.md`
- `temper/docs/design/draft/ask-temper.execution.md`
- `temper/docs/design/draft/ask-temper.status.md`


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
