+++
id = "close-chat-proof"
part = "close-chat-proof"
kind = "feature"
needs = ["confirm-sheet"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 70
+++

# The render check, the walk and the account of `close-chat`

## Goal

Prove the landing against the request by rendering, walk each confirmation it serves in a browser, fix what they
find, bring the design text in line, and carry the account in the landing's last commit. Every commit carries
`Look-request: close-chat`.

Acceptance:

- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each difference
  with its reason.
- **The screenshots** in `~/src/fabrig/look-round-10/` on your builder, outside any worktree, never committed: each
  page's `-request.png` beside its `-landed.png` at 1440, 1000 and 420, and each state in words at each. The report
  says which builder.
- **The walk,** step 3: each confirmation pressed both ways, each result recorded.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5: in the last commit's message, one row per overlay
  rule, per page change and per state the message names, none left open.

## Inputs

- `temper/docs/plan/look-round-10/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5, 7.
- `temper/docs/design/client/README.md`, 3.6, *The sheet*, *Closed ends on one line*.
- `temper/docs/design/client/README.status.md`: its **Not yet** on 3.6, this landing's.
- The request, as the note says, and `confirm-sheet`'s commits and report's gallery names on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render or walk finds and named in the account:

- `temper/crates/temper-client-view/src/confirm.rs`, `pages/chat.rs`'s closed lines, `look.rs`, `words.rs` and their
  tests; `temper-client-domain/src/acts.rs`, for a press the walk loses.
- `temper/tools/gallery/src/**`, its `names.txt`, and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.6, `browser.md`, `journeys.md`, 9, and the journeys' `README.md` files: what
  the check taught you.

Options, each pre-approved:

- **A state no gallery name reaches:** a throwaway state, untracked or reverted before the push.
- **A sheet a fake box can't reach in a few minutes:** its gallery state and the client world's test stand for its
  walk; say so in the account.
- **The hand-made values** (01:41, T4, T5, 5 seconds) are the request's: compare the sheet's shape and styles against
  the gallery state's own values, and say so.

Leave: `temper/docs/design/client/greenfield/`, whose README keeps look requests' changes on top of it.

## Steps

1. **The pages,** from `c52e34bd67`, with the overlay at that commit, each against its gallery state with the
   landed stylesheets and an empty overlay:
   - `close-chat/01.html` against `chat-closing`;
   - `02.html` against `chat-closing-work`;
   - `03.html` against `chat-closed`.
2. **How,** at 1440, 1000 and 420 pixels wide (the note's CDP way at 420): each element's computed style and box, then
   the screenshots for what styles miss. `.temper-look-backdrop` against `::backdrop` is a known difference: the real
   one's colour holds. The phone's 44 px acts against the overlay's 38 px is `confirm-sheet`'s, to name.
3. **The states in words,** rendered beside the request's words: each row of the design's table, one fate alone, the
   tasks below it, a failed confirmation, the release's uncertain pair, a long delegate title at 420. Then **the walk,**
   in a browser, on a fake box as `temper/docs/design/client/journeys.md`, 8 records one:
   - Close on a chat: "Keep it" returns to the chat, unchanged; "Close chat" ends on the one line;
   - Stop on a goal: "Keep going", then "Stop goal"; and Pause, which sends at once with no sheet;
   - Accept and Release on Needs you: "Not yet", then the act; the row settles, and a press while the page updates
     isn't lost (`ui-walk-2`'s finding: the button keeps its element);
   - a refused close if the box can give one, else the client world's story stands.
4. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: the at-work sheet, the closed chat's page and
   the prints that hold a chat's end, read for their meaning.
5. **Reprint** any journey whose pages changed, and say if none did. **Bring the design text in line:**
   `mockups/STATUS.md`'s "**Closed** for every ended chat's bar and list row, with the end's clock in the bar when
   supplied" (the bar has no time now), its lines on a dialog if any, `journeys.md`, 9, and take the 3.6 **Not yet** out of
   `README.status.md`, leaving a **Departs** only for a difference you keep.
6. **Run** `python3 -I -B tools/citations/citations.py --base origin/main`, `cargo fmt`, and the focused tests of what you
   changed. **The last commit** carries the account's table in its message, the render result, and the trailer, and is
   pushed.

## Report

As your text says, plus: the builder and the screenshots' directory; the render result by page and width; the walk's
result by confirmation; every row left "differs" with its reason. A deferred row names its owner and where it is tracked.
