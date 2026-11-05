+++
id = "phone-proof"
part = "phone-proof"
kind = "docs"
needs = ["phone-walk"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# The proof: every page beside the mock

## Goal

The plan's last task, and its catch-up. Every page the mock draws is shown beside the client's, on a laptop and on a phone, with its open states, over whatever is on main by now. What differs is fixed or named. What wasn't there yet is named.

Acceptance, screenshots in `~/src/fabrig/ui-phone/proof/` on your builder, outside any worktree, never committed:

- **Your base holds main:** first merge `origin/main` into your branch. Your report says which of `ui-chat-draft` and `ui-away` that brought in, and which wasn't there yet.
- **The sweep runs clean again** (`temper/docs/plan/ui-phone/notes/sweep.md`) over the states main now holds.
- **Renders beside the mock's page at 1440, 1000 and 420 wide,** full page, for each of Home, Needs, Goals, a goal, Chats, a chat, Changes, Settings and the welcome.
- **Their open states:** the switcher, the sheet, a chat with its composer focused, and the tab current on each of the four tabs.
- **At 390 by 844, as `phone.html` frames it:** the mock through `phone.html?p=<page>`; the client's in an iframe of that size, or by CDP's `Emulation.setDeviceMetricsOverride`; a viewport-sized picture, not full page.
- **A table in your report:** each page, state and width, same or different, and why. Fonts that differ only because the mock's are fetched don't count.
- **What differs is fixed** in `temper.css`, in that page's own place, or named by a section of `temper/docs/design/client/browser.status.md` or `README.status.md`, whichever holds that page.

## Inputs

- `temper/docs/plan/ui-phone/inputs/request.md`, *How the cut shapes it* (The proof) and *Beside it*.
- `temper/docs/design/client/greenfield/README.md`, `phone.html`, and `app.js`, `phonePage`.
- `temper/docs/design/client/browser.md`, 2.5 and 7.
- `temper/docs/plan/ui-phone/notes/common.md`, `temper/docs/plan/ui-phone/notes/sweep.md`.

## Owns and may change

Owns the `files` above.

- **Departures from your text:** you merge `origin/main`, a merge and never a rebase, as the first thing you do. On that merge, goldens, prints and the gallery's `names.txt` and `tests.rs` are regenerated rather than hand-merged. You edit the status lines above.
- **Departure from `notes/common.md`:** a page of `ui-chat-draft`'s or `ui-away`'s that's on your base after the merge is a page like any other: check its phone rules, the Away chip and `.decision` rows and the draft's row in the request's sections 1 and 3, and fix a miss. What isn't there yet is named, and stays unbuilt.
- **Reader files, a line or two each, where the sheet alone can't:** a page's view in `temper/crates/temper-client-view/src/pages/` and `look.rs`, its `tests/look.rs`, and the goldens and prints that follow, regenerated.
- **A difference that takes more than a rule's or a small view's change** goes under `## Left`, with its screenshot's path.
- **`box.html`** is drawn from `temper box`'s first run: render it beside `welcome` if its scenario draws, and name it otherwise.

## Steps

1. **Merge `origin/main`,** regenerate what conflicts, and run `heavy cargo nextest run -p temper-client-gallery`.
2. **The sweep,** then **render** each page beside the mock.
3. **Fix and name,** smallest first; run the sweep again after the last fix.
4. **Check:** the implementer's text; the citations check; `cargo fmt`. Commit and push.

## Report

As `notes/common.md`, Your report: the builder, the screenshots' directory, the table, what wasn't there yet.
