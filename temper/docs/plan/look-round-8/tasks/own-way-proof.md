+++
id = "own-way-proof"
part = "own-way-proof"
kind = "feature"
needs = ["own-way-pane", "own-way-words"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/protocol/documentation.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 70
+++

# The render check and the account of `configure-temper-own-way`

## Goal

Prove the landing against the request by rendering, fix what the render finds, bring the design's text in line, and carry the account in the landing's last commit.

**Departs from the implementer's text:** this task edits the `docs/design/` files in `files`, and any it names below.

Acceptance:

- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each difference with its reason.
- **The screenshots** are in `~/src/fabrig/look-round-8/` on your builder, outside any worktree and never committed: each page's `-request.png` beside its `-landed.png` at 1440, 1000 and 420, and each state in words. The report says which builder.
- **The page still shows whole** at 1366 by 680: the thread's `scrollHeight` and `clientHeight`, in the account.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5, is in the last commit's message: one row per overlay rule, per page change and per state the message names, none left open.
- **The status entries** this landing closes are gone, and the design says what was built.

## Inputs

- `temper/docs/plan/look-round-8/notes/common.md`.
- `temper/docs/plan/look-round-8/notes/contracts.md`: 2 and 5 say what differs already, and why.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5 and 7.
- `temper/docs/design/client/README.md`, 3.5, *temper's own way, beside the page*.
- The request, as the note says, and the other tasks' commits on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render finds and named in the account:

- `temper/crates/temper-client-view/src/pages/configure.rs`, `pages/reference.rs`, `look.rs`, `words.rs` and their tests.
- `temper/tools/gallery/src/**`, and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.5: what the check taught you.
- `temper/docs/design/draft/own-way.status.md`: only if a departure there no longer says what was built.

Options, each pre-approved:

- **A state with no gallery name:** a throwaway one, untracked or reverted before the push.
- **A journey found stale:** reprint it, and record it again if the reprint can't carry it (`temper/docs/design/client/journeys.md`, 8). A re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.

Leave:

- The figures' values: the user's.
- A chat that keeps the reference open: the design closes it.
- skein's flows.

## Steps

1. **The pages,** from `3651707d4c`, with the overlay as it stands there, compared with the landed stylesheets and an empty overlay:
   - `configure-own-way/01.html` against the gallery's `configure`;
   - `02.html` against `configure-reference`.

   The rail and the account's line are the world's: compare them as words and say so. `notes/contracts.md`, 2 and 5, name the differences already decided.
2. **How,** at 1440, 1000 and 420 pixels wide, each disclosure and menu forced open: each element's computed style and box, then the screenshots for what styles miss. A difference is fixed, under the trailer, or has its reason.
3. **States in words,** beside the message's words, rendered:
   - the steps' arrows ending each line where they wrap, at 1000 and 420;
   - on a phone, the card's buttons under the words, each half the width;
   - in Split, the ways one a row, each title and its line on one line, and the card's buttons under its words;
   - the group's name and its three buttons, and on a phone the two it shows;
   - on a phone, the reference alone as Draft, the composer under it (`configure-reference` and `configure-reference-draft` at 420);
   - "Chat" pressed: the pane folded, the group kept;
   - a start out: the starters and "Start from it" disabled, "See how it works" not (`configure-sending`);
   - "How Hello works" with its column: the agent's words, so name the gallery's `configure-chat-document-split` and `notes/contracts.md`, 6, and don't draw a new one;
   - a fresh project, Goals and drafts hidden, whose page has the card and whose reference opens.
4. **Other chats unmoved:** the gallery's `chat-draft-split` and a plain chat at 1440 and 420 against `main`'s stylesheet. The thread's padding and the starters' are the rules at risk; `.temper-doc-inner td` is the request's, and changes a document's cells. Name that in the account.
5. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: read the prints of `configure` and `configure-reference`. Headings and order keep their meaning; the steps are a list; the figures print. Say so in the account.
6. **Reprint:** run the journeys' test. No journey walks this page, so expect none stale; if one is, follow the option above.
7. **Design text:**
   - `README.status.md`: the first **Not yet** of 3.5 for this request goes;
   - `documentation.status.md` and `agent.status.md`: this request's **Not yet** entries go;
   - `journeys.md`, 9: a sentence beside `configure-chat-draft`'s, that the `configure-temper-own-way` request's hand-made `configure-own-way/` pages are not journeys, and the gallery draws those states;
   - `mockups/STATUS.md`: the configuring chat's row names the card and the reference;
   - run `python3 -I -B tools/citations/citations.py --base origin/main`.
8. **Close:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` for what you changed; `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-web`. The last commit's message carries the account, with the trailer. Delete throwaways, then push.

## Report

`Done:`, then:

- which builder, and the screenshots' directory;
- the render check's result, and the 1366 by 680 figures;
- each difference with its reason;
- what the render made you fix;
- the account, whole.
