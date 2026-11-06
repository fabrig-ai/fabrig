+++
id = "look-proof"
part = "proof"
kind = "docs"
needs = ["sheet-story", "welcome-ways"]
files = ["temper/docs/design/client/README.md", "temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.md", "temper/docs/design/client/forge.status.md", "temper/docs/design/client/journeys.md", "temper/docs/design/client/journeys/*/README.md", "temper/docs/design/client/mockups/STATUS.md", "temper/docs/design/draft/repository-copies.status.md", "temper/docs/design/STATUS.md", "temper/docs/design/draft/look-requests.md", "temper/docs/help/**"]
effort = "high"
minutes = 75
+++

## Goal

The landing's last steps (`temper/docs/design/draft/look-requests.md`, 3, steps 6 to 8; 4; 5): every page rendered against the request's, and the account in this task's last commit. **This task departs from the implementer's text:** it writes `docs/design/` and `docs/help/`, as `look-requests.md`, 3, step 8 asks.

Acceptance:
- **The render check:** each of pages 01 to 23, from the request's commit with the request's stylesheet over it, against the same state of the real page with the landed stylesheets and an empty overlay, by a throwaway driver of headless Chromium, comparing each element's computed style and box, then screenshots: at 1440, 1000 and 420 wide (also 1280 for the screenshots), each disclosure and menu forced open. A state in words only (a failed copy's refusal, busy, empty, a forge with several Forgejos) through a gallery page beside the request's words;
- **Screenshots** of each changed page beside the request's are in `~/src/fabrig/look-round-13/` on this builder, outside any worktree, at 1440, 1280, 1000 and 420 wide. The report says which builder;
- **What differs is fixed or explained.** Fix a difference that's a miss, where it stands, under the trailer; explain one that isn't;
- **The account,** in this task's last commit's message: one row per rule of `connect.css`, per page change and per state the message and `README.md` name, `landed`, `differs, and why`, or `deferred, to whom and where it is tracked`; the render check's result. No row open;
- **The design in line:** `README.status.md`'s, `forge.status.md`'s, `ui-web.status.md`'s and `ui.status.md`'s look-round-13 **Not yet** entries go or say what's left; `repository-copies.status.md` says what's left of 9, among it Home's own sheet and the two ways into the sheet the request leaves unmocked; `mockups/STATUS.md` and `journeys.md`, 9, as `look-requests.md` asks; `temper/docs/design/STATUS.md`'s summary; `browser.md`'s tokens (`CONNECT_*`, 2.5) and what it says of a flow's steps; the rest of `README.md` and `forge.md` that still says "Choose", "Ready", "Setting up" page or "Next", or names the steps list (5.2, *A flow's parts*, 5.5, 5.6), brought to what the pages do, without renumbering a section;
- **Help:** `temper/docs/help/` pages that describe these pages say what they now do, within their bounds; `temper/crates/temper/src/tests/help.rs` passes;
- **A lesson:** a line in `look-requests.md` for each miss the render check caught, if any;
- `python3 -I -B tools/citations/citations.py --base origin/main` is clean, `temper-client-view`'s, `temper-client-journeys`' and `temper-docs`' focused tests pass. No tier is run.

## Inputs

- `temper/docs/design/draft/look-requests.md`, all of it; `temper/docs/design/client/journeys.md`, 7 and 8; `DESIGN.md`.
- The request's whole commit: its message and `README.md` are the account's source of rows.
- `temper/docs/plan/look-round-13/notes/common.md`.

## Owns and may change

- Owns the `files` above.
- May change: `temper/crates/temper-client-shell-web/temper.css` and any view file, to fix a miss, with its test; gallery goldens and journey prints that follow, regenerated.
- Each commit carries the trailer `Look-request: connect-flows`.
- A missed page that needs more than a fix of an hour: a row `deferred`, tracked in `README.status.md`, naming what's left.
- Requests above this one on the look-requests branch are other plans'; touch nothing of theirs.

## Steps

1. Extract the request; build the gallery; write the throwaway driver; run it; note each difference.
2. Fix the misses, narrow tests each.
3. Screenshots.
4. Design, status, help.
5. Citations; focused tests; remove the driver; last commit with the account; push.

## Report

As the implementer's text has it: the builder, the screenshots' path, the account, and the questions of `look-requests.md`, 6, if any.

## Answers

2026-11-06 (temper/ui). The user's rule, 2026-11-06: a plan executed by fab runs focused and fuzzy alone, and is complete when it lands. Walks and render checks follow landing, and file findings back to fab. So drop this task's throwaway-driver comparison and screenshots: don't start a browser to compare. temper/ui runs the render check after the plan lands. What's left of this task: regenerate what the gate's focused checks read, such as prints with temper-client-journeys again and generated files; the browser story, compiled and not run; the design and status text; and the procedure's account, saying the render check follows landing.
