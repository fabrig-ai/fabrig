# ui-goal-asks-2: a goal's question reads whole, and starting a goal holds together

temper/qa walked a goal asking its person on a real model, `q-asks-1` and `q-asks-2`, at `42222b0ee5`, `gpt-6.1-sol` at medium, one sample each. The dead-end the user hit is gone: the goal asked under Needs you, the answer woke it, and it landed what was picked. These are what the story tripped over. Reports on fab's host: `~/src/fabrig/coord/codex-kit/reports/q-asks-1.md`, `q-asks-2.md`; evidence on hetzner8 under `~/src/fabrig/fab/walks/q-asks-{1,2}/evidence/`.

## 1. Needs you clips a question with no sign of it: the one that matters

The screen ends mid-word, `Choice: B — "Hello, careful builder`. The model's words were 517 bytes, and the read has `specification.words.cut: true` at 512 (`07-needs-question.png`, full text in `person-task3-full-text.txt`). A question is where a person decides, so they must see all of it.

**Ruling:** a person task's question shows whole under Needs you and on its goal's band, to the words' own bound, with nothing cut for a person to decide on. The engine's cut at `documents.text_bytes` is why; `long-words` met the same cut for a chat's first message. Read the task's whole spec for this page, or carry it, whichever is plainer, and say which. If a cut can't be avoided at a real bound, the page says it's cut and links to the rest.

`Choice:` and `Saw:` also show as plain text, with no choice buttons. Read that against `temper/docs/design/client/README.md`, 3.5 and 5.13's question form: if the design has buttons for a question's choices, draw them and answer on a press; if a person task in words is plain by design, say so in the status, and leave it.

## 2. Goals' "Start one from a draft" goes to `/chats`

The design has it open the Start goal sheet with a Draft path field (README 3.5, 5.14). It goes to `/chats`, and More ways → Goal opens a new draft chat (`24-goals-start-sheet.png`, `25-goals-start-link.png`). Build what the design says. A goal started this way then asks its person as a sheet's does.

## 3. Start goal lets through a goal with a budget of 0

With a draft chat and a configuring chat open, `me` read `pool.left` 0 and `creating.budget` 0, on uncapped `as_allowed`, as open chats' reserves took the pool (`q-asks-1`; on that probe's own budget, not stock: one chat left 100M of 900M). The goal held for funds before any model ran, and neither the sheet nor Amend has a budget field (`04-start-sheet.png`, `07-amend-offered.png`). **Ruling:** with nothing to fund a goal, the sheet says so before it's started, in a person's words, naming what holds the pool: open chats. Read whether stock's own figures can reach this. Funding rules don't change here, and no budget field is added.

The funds card says "give task 5 more budget by deciding its budget proposal", where no such proposal exists (`06-funds-inbox.png`): say what a person can do.

## 4. Smaller

- "2 landed" on a goal, when its finished children were a person task and a read-only advisor and no change had landed (`20-after-advisor.png`): count landings.
- `temper ctl`'s `proposal decide t2/6 accept` on a replaced proposal answers exit 2 and "out of shape; running", with or without `--running stays`, where the help says `decided first`. Check whether the walker's use of the flag caused it, and fix the words if not (`accept-replaced.txt`). `temper ctl` is temper/migration's, and it isn't running.

## Limits

Two or three tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands; temper/qa looks after. `temper-client-journeys again` is generation, and allowed. No new browser test. New cases are rows of an existing table test.

## Sources

- `temper/docs/help/goal.md`

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
