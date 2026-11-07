# look-round-16: a chat's terminal

At the user's go, 2026-11-07, who tried them and said to keep them exactly so, the commands included: land the look requests `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft` and `chat-terminal-esc`, as one piece, in that order, on `origin/look-requests` above `chat-title`. Find each by its trailer; the branch moves. Pages are in `temper/docs/design/client/journeys/chat-terminal/`: start at `01.html` and click `>_`, as pages 02 to 06 open on the terminal without full screen. Its `README.md` says what each page is, and each commit's message the states and rules.

**The procedure is `temper/docs/design/draft/look-requests.md`: sections 3 and 5, and its account.** Section 4's render check follows landing, temper/ui's. Read each commit and its whole message. `chat-title`, below them, is `look-round-15`, landing now: take it at catch-up, as these are drawn on its pages.

## What it is

A `>_` button beside Chat, Split and Draft turns a chat into a full-screen coding-agent terminal: one top line with `Draft` and `Exit`, every other control a `/command`, and the draft beside it as a document, in temper's own type and figure, on a ground of its own.

## The user's word: keep these exactly

- Line spacing 1.6 in the terminal, and the column left, at 100 characters.
- Every command, `/theme` and `/font` included, with its words as drawn.
- The terminal in DejaVu Sans Mono at 11pt. The fonts move to the shell's `files/fonts/`, where their licence already is.
- Esc: `navigator.keyboard.lock(['Escape'])` once in full screen. In Chromium a short Esc closes a list and stays in full screen, a long one leaves, `Exit` leaves both, and the desktop's own shortcuts still work (the laptop checked these in Chromium 153 with real key presses). Firefox and Safari can't hold the key, so there one Esc leaves and closes the list.

## Decided here, by temper/ui

- **Commands are client actions** on what a chat already has: no new protocol.
- **The mode is remembered per browser,** so another chat opens in it, and `/chats` and `/new` stay in it. Where the client keeps a per-browser choice already, keep it there.
- **Dark comes first.**
- **Not drawn, and a real chat has them:** a goal's question, as a list under the box, and a proposal or Accept card in the conversation. Carry them in the same language: the terminal's type and ground, one line where a line will do. The draft pane's figure takes the shell's tokens alone, so share its rules rather than restating them as the page does.
- **Also, from the same journey:** a draft shows Markdown italics literally, `_Draft, with the user._`, underscores and all (journey 8's Split page). The mock shows it plain: render emphasis in the draft, in both Split and the terminal's document.

## How the cut shapes it

Three or four tasks: the frame and commands, the draft beside it, Esc and full screen, the cards in its language. The terminal client in `temper open` is a separate thing and doesn't change. Focused and fuzzy alone: no browser and no render check in the plan; prints and journeys regenerate by `temper-client-journeys again`. No new browser test: the tier is at its cap. New cases are rows of an existing table test.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/client/journeys/chat-terminal/README.md`

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
