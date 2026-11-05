+++
id = "chats-page"
part = "chats-page"
kind = "feature"
needs = ["new-chat-playground"]
files = [
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 85
+++

# The Chats page

## Goal

`Chats` is a page of the frame, not a pane, drawn as the mock's `chatsPage`.

Acceptance, the request's section 1, each line of it:

- A `Role::Main` with `CHATS_PAGE`, `MAIN` beside it if the frame's rules need it. `Role::Complementary`, the `temper-sidebar`
  rules that only it uses, `look::SIDEBAR`, the "New chat" bar (`start`), `offers_first_chat` and its button go. To begin and
  Settings keep their offers. With no project yet, "No project yet" as today.
- **Head,** **No repository,** **The list,** **Rows,** **Empty,** **"More closed chats"**, as the request and
  `notes/contracts.md`'s tokens and words have them. `repository_context` keeps its condition.
- **The row:** the list's merge order and the row's key stay. One link on the number; its `::after` covers the row, and
  the row is `position: relative`, so that pressing anywhere on the row follows it.
- **Status:** `status()` keeps its token and adds `STATE_NEEDS` when that token is `STATUS_YOURS` or `STATUS_WAITING`,
  on the row alone; a node carries two tokens at most. Lower case by css. **Age:** `TimeForm::Span`, Home's.
- Spend, delegates and "Closed. Its transcript is kept." stay in the view, hidden in the browser by the sheet. The
  project's name leaves the row, in both shells.
- **Phone:** the request's three rules; the frame's top bar and tab bar stay.
- `chats::row` isn't read elsewhere; if it is, those readers keep their words.
- **Gallery:** `chats-no-repository`, `chats-live`, `chats-closed`. `chats-solo`, `chats-empty`, `chats-reference`,
  `chats-first-run-*`, `configure-chats-tag` hold. A configuring chat's " · Configuring" replaces `CONFIGURING_TAG`, which
  goes if nothing reads it.
- `README.status.md`'s **Not yet** entry for 3.5, The Chats page, goes. `browser.status.md`'s 2.5 stays until
  `tool-lines`.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 1.
- `temper/docs/design/client/README.md`, 3.5, *The Chats page*; `temper/crates/temper-client-view/src/pages/home.rs`, its
  `chats` and `row_words`, for the short age and for how `#app > .temper-home` takes the mock's `.main` and `.col`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/tests/{playground,playground_start,configure,frame,times,rail}.rs`: what the new
  page makes stale.
- `temper/crates/temper-client-view/src/pages/{home,chat,inbox}.rs`: only if a helper's signature moves.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/tests/one-process/**`,
  `temper/tests/real-loop/**`, `temper/crates/temper/tests/**`: what walks the list by a role or a name it changes: the
  level-1 "Chats", the "Project" region, the row's project name, the first-chat offer's button. **A change of meaning runs
  its readers' tests.**
- The terminal's goldens; the journeys' prints.

Options, each pre-approved:

- A row's top line in the terminal reads "T9 · Configuring" then status and age on one line, or two; whichever
  `skein-ui-text` prints legibly. It must keep its command number on "T9".
- The "Your chats" list's name stays.
- A test that pressed the first-chat offer on `Chats` goes to Home's, or goes with it.
- If the last `Word` of a removed part is read nowhere, it goes; `FirstChat` stays, `Configure`'s.

## Steps

1. The page. The rules under the marker, replacing the pane's where they stand.
2. Renders beside the mock's `chats.html`: All, Live, Closed, empty, no repository, at 1440, 1000 and 420, with `hello`'s state
   as the request's *The mock* says; `~/src/fabrig/ui-chats/chats-page/`.
3. **Tests,** `tests/chats.rs`: the order; the words with a repository and with none; each filter pressed, and its act; a row's
   parts and the ended token; `STATE_NEEDS` on both tokens and not on another; "None." under every filter; "More closed chats";
   the href of the row's link; the band's link. Each bites. Gallery, goldens, prints.
4. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.
