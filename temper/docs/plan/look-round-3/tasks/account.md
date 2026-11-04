+++
id = "account"
part = "account"
kind = "docs"
needs = ["record-landing"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/journeys/project-landing/README.md",
  "temper/docs/design/client/journeys/welcome-playground/README.md",
  "temper/docs/design/client/journeys.status.md",
  "temper/docs/design/client/mockups/STATUS.md",
]
effort = "high"
minutes = 120
+++

# The render check and the account

## Goal

Prove the landing by rendering, and account for each item, as
`temper/docs/design/draft/look-requests.md`, sections 4 and 5, ask. This task's last commit holds
the account table, and its report holds that table and the render check's result. The screenshots
come back to the user, beside the request's pages.

Acceptance:

- The render check of section 4 has run: `project-landing/01.html` and
  `project-switcher/01.html`, each from the request's commit with the overlay as it stood there,
  against the landed page with an empty overlay, at 1440, 1000 and 420, with each disclosure and
  menu forced open: each element's computed style and box, then the screenshots. Every difference
  is fixed or has its reason.
- The states in words are rendered through the gallery's `home-*` states, or a hand-made page beside
  the request's words, and read against them.
- `~/src/fabrig/look-round-3/project-home/` on this builder holds, outside any worktree, never
  committed: each page's `-request.png` beside its `-landed.png`, at 1440, 1000 and 420, closed and
  with the switcher open, and each state in words.
- The last commit's message holds the account above its trailers: one row per overlay rule, per page
  change, per state the message names, each landed, differing and why, or deferred, to whom and where
  it is tracked. No row left open.
- Rows that differ or defer, at the least, each with its status or question:
  - the kicker's "made just now" and "its CI ran on main and passed.": the plan's question,
    `README.md`, section 11, `README.status.md`;
  - the account's email in the model line: jig's account-facts draft, `README.status.md`;
  - "Open Forgejo" signed in through temper's address, `/forges/1/open` as drawn, and "+ New chat"'s
    `/chats/new`: the design's addresses;
  - "3 repositories": unreached, the home standing in;
  - GitHub's picker and its count: `README.status.md`, 4.4;
  - the inbox's heading stays "Inbox";
  - "the first adopted" reads the rows' order, the forge's ids.
- `README.status.md` has no **Not yet** entry this round closed, `journeys.status.md` says what is
  true, and `mockups/STATUS.md` holds.
- `git log --format='%h %(trailers:key=Look-request,valueonly)' <trunk's base>..HEAD` shows the
  trailer on every commit of the round's landing.

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 7.
- `git show 0197eaea31`; the reports' lines each earlier task left are in its commits' messages and
  `temper/docs/plan/look-round-3/`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-shell-web/temper.css`, `temper/crates/temper-client-view/src/**`,
  `temper/docs/design/client/journeys/**`: a fix the render check finds, its test beside it, its
  reprint with it, carrying the trailer.
- `temper/docs/design/client/README.md`, `browser.md`: a line the render check shows untrue, by
  name in the report.

## Steps

The check, as last round did it:

1. **Two trees of the same shape** under `~/src/fabrig/look-round-3/project-home/`: `request/`, from
   `git archive 0197eaea31 temper/docs/design/client/journeys temper/crates/temper-client-shell-web`,
   overlay as it stood there; and `landed/`, the same two paths copied from your worktree, overlay
   empty, `project-switcher/` copied from `request/`.
2. **A measuring copy of each page,** never the page: a script before `</body>` that opens every
   `details`, drops `hidden` from each menu and dialog, walks the elements in document order and
   writes, for each and for its `::before` and `::after`, `getComputedStyle` and
   `getBoundingClientRect` as JSON into the document, with `innerWidth`.
3. **Chromium** is `$SKEIN_TEST_CHROMIUM`, else `chromium` on `PATH`: `--headless=new --no-sandbox
   --disable-gpu --dump-dom --virtual-time-budget=3000 --window-size=W,2400 file:///...` for the
   figures, `--screenshot=PATH` for the pictures. If Chromium floors `innerWidth` above 420, render
   the page in an iframe of that width from a wrapper page.
4. **Compare,** element by element.
5. **States in words:** `heavy cargo run -p temper-client-gallery -- <dir>`, the `home-*` states and
   the rail's foot offline and connecting, and read each against the message.
6. **The terminal:** print Home in each state through the gallery's terminal or a test, and paste
   the headings' print in the account.
7. **Design text** (`look-requests.md`, section 3, step 8): the client's `README.md`, `browser.md`,
   the journeys' `README.md` files, `journeys.md`, 9, `mockups/STATUS.md`.
8. **Throwaway scripts are gone** before the push; `git status` is clean of them.

## Report

As `notes/common.md`, Your report: the account whole, the render check's result, the builder's
`hostname` and the directory of the screenshots.
