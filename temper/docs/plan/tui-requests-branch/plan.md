# tui-requests-branch: plan

The user's word, 2026-11-08: the TUI's look requests stack on a branch of their own, `tui-requests`, apart from `look-requests`, where the web's requests are worked on beside them. `temper/scripts/look-refresh.sh` keeps either branch on `main`, whichever is checked out, and refuses any other (`temper/docs/plan/tui-requests-branch/inputs/request.md`).

## Parts and tasks

- **branch** (direct, slice 1, no walk): task `branch` (`tasks/branch.md`), medium effort, about 35 minutes, needing nothing.
  - The script: the checked-out branch is accepted if it's `look-requests` or `tui-requests`, and it's that branch's name wherever the script names `look-requests` today: the lease, the range of requests, the merge base, the push and the messages.
  - The tests (`temper/tools/journeys/tests/suite/look_refresh.rs`): the fixture takes its branch. `the_requests_follow_main` gains a `tui-requests` row beside `look-requests`. The checkout refusal story asserts the message naming both branches, on `main` and on a detached HEAD.
  - It removes the two **Not yet** entries the plan opened.

## Critical path

`branch` alone. It starts at the first push, and the plan lands with it.

## What was decided

- **One task, as the request asks.** The change is a branch name made a variable, and one row of an existing story.
- **The design, written by the lead:**
  - `temper/docs/design/client/tui/journeys.md`, 10: the bullet *Requests stack* now names `tui-requests`, apart from `look-requests`, with names starting `tui-`, kept by the same script run with `tui-requests` checked out.
  - `temper/docs/design/client/journeys.md`, 7: the bullet *Kept on `main`* gains one sentence: the script takes `tui-requests` alike, with that branch's lease, requests and push, and refuses any other branch. The rest of the browser's section stays.
  - `client/journeys.status.md` (7) and `client/tui/journeys.status.md` (10) each gain a **Not yet** entry, which the task removes. The spec states that departure from the implementer's text.
  - `draft/look-requests.status.md` gains a **Design departs** entry for 1 and 2. The draft keeps every request on one branch, `look-requests`; the design now puts the TUI's on their own, by the user's word of 2026-11-08. The draft's own text isn't edited.
- **The tests are one row and one message.** The script has tests, so the request asks for a `tui-requests` row and a refusal. `the_requests_follow_main` becomes a two-row table. Its `tui-requests` row bites on the branch check, and on a lease left at `look-requests`: the published `origin/tui-requests` makes such a push stale. The existing checkout story already refuses `main` and a detached HEAD, so it gains only the message naming both branches. Every other story stays on `look-requests`.
- **No name rule in the script.** The `tui-` prefix is the person's convention. The request says nothing else in the script's behaviour changes, so the script doesn't check it.
- **Left as they are:**
  - `temper/AGENTS.md`'s *Look requests* line still names `look-requests` alone. It's the user's guidance, outside the request.
  - The draft `look-requests.md` isn't edited; its status records the departure.
- **No `notes/common.md` or `notes/contracts.md`.** One task, and nothing crosses a part. The request's `For every task of this plan` section serves as the common note.

## Overlaps

These were unavoidable: the design names the branch in these files.

- `temper/docs/design/client/tui/journeys.md` and `tui/journeys.status.md`:
  - `tui-link-colours` (its `again` and `host-local` tasks) changes the status file;
  - `ui-ask-drop/journeys` and `ui-ask-drop/close` change the status file too;
  - this plan's edits are section 10's one bullet, the status file's first word of its paragraph and its last line.
- `temper/docs/design/client/journeys.md` and `journeys.status.md`:
  - `ui-ask-drop/journeys` and `ui-ask-drop/close` change the status file;
  - this plan's edits are one sentence at the end of section 7's *Kept on `main`*, the status file's date (2026-11-08) and one **Not yet** bullet.
- `temper/scripts/look-refresh.sh` and `temper/tools/journeys/tests/suite/look_refresh.rs`: no other draft in flight names them.
