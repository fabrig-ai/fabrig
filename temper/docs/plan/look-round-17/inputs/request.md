# look-round-17: what the render check found in the chat terminal

A look repair, after landing, per `temper/docs/design/draft/look-requests.md`, 4: each repair carries its request's trailer (`Look-request: chat-terminal`, `chat-terminal-draft`). The check is `~/src/fabrig/coord/codex-kit/reports/look-render-3.md` on fab's host, at `1eae64435c`, at 1440, 1000 and 420; the request pages are on `origin/look-requests`, by their trailers. The user tried the requests and said to keep them exactly so. These are misses, so make the real pages match.

## 1. `>_` doesn't show the terminal: the one a person meets first

Pressing `>_` enters full screen, but the ordinary chat stays until another input or a reload; typing `/` then reveals the terminal. No console error. The request draws full screen **and** the terminal at once. Find why the page doesn't draw on that press, with a failing focused test of the mode's state after the press first, and fix it. The Escape landing's account says the Enter press draws the terminal: check that, too.

## 2. On a phone, the composer under the draft is white

At 420, on page 03 and with the draft open on 04 and 05, the conversation and composer's backing are `rgb(255,255,255)` beneath a dark document, where the request has the terminal's warm dark `#191816`. Quiet text then turns faint on white. The ordinary mobile Split rule's `--gf-surface` wins: scope the terminal's ground above it.

## 3. The draft's scrollbar, desktop

Page 03: the document's scrollbar is `scrollbar-width: thin` with `scrollbar-gutter: auto` in the request; it's `auto` and `stable both-edges` landed. So the text is 20 px narrower and 15 px further right, at 1440 (601 against 621, left 780 against 765) and 1000. The terminal account itself says thin. At 420, `thin` too.

## 4. Two small spacings

- The quiet line's model button has no padding in the request; the reused terminal button brings `1px 1ch` (`4px 1ch` on a phone), so the line is 31.47 against 29.47 px high, and 34.8 against 26.8 on a phone. Pages 02, 03, 06.
- Page 06: the running line sits inside the foot, with 0 px between it and the box. It's a sibling before the foot today, so the foot adds 4 px.

## Leave alone

The report's explained departures stand, each with its account's reason. A chat's title on a 1000-wide bar now wraps to two rows, because the bar holds `>_` and its status words: that's the requests meeting, so leave it, and say so in the account.

## Limits

One task, or two. Focused and fuzzy alone; temper/ui checks after it lands. Prints regenerate by `temper-client-journeys again` where a journey's pages change. No new browser test. New cases are rows of an existing table test.

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
