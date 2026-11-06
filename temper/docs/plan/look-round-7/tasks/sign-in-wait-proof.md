+++
id = "sign-in-wait-proof"
part = "sign-in-wait-proof"
kind = "feature"
needs = ["sign-in-wait-page"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/mockups/STATUS.md",
]
effort = "high"
minutes = 70
+++

# The render check and the account of `sign-in-new-tab`

## Goal

Prove the landing against the request by rendering, fix what the render finds, bring the mockups'
status in line, and carry the account in the landing's last commit. Every commit carries
`Look-request: sign-in-new-tab`.

Acceptance:

- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each
  difference with its reason.
- **The screenshots** in `~/src/fabrig/look-round-7/sign-in-new-tab/` on your builder, outside any
  worktree, never committed: each page's `-request.png` beside its `-landed.png` at 1440, 1000 and 420,
  closed and with its disclosure open, and each state in words.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5: in the last commit's message, one
  row per overlay rule, per page change and per state the message names, none left open.

## Inputs

- `temper/docs/plan/look-round-7/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5, 7.
- `temper/docs/design/client/README.md`, 3.5, *The sign-in under way*.
- `temper/docs/design/client/README.status.md`: its first **Not yet**, this landing's.
- The request, as the note says, and `sign-in-wait-page`'s commits on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render finds and named in the account:

- `temper/crates/temper-client-view/src/pages/sign_in_wait.rs`, its tests, `look.rs` and `words.rs`.
- `temper/tools/gallery/src/**` and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.5, and `README.status.md`'s entry above: what the check
  taught you.

Options, each pre-approved:

- **A state with no gallery name:** a throwaway one, untracked or reverted before the push.
- **The refused state:** flip `tab_refused` to `true` locally, print `welcome-model-away` and
  `connect-model-signing-in`, screenshot them, and restore it; the commit never carries the flip.

Leave: everything `notes/common.md` says is not built here.

## Steps

1. **The pages:** from `c725deec96`, with the overlay at that commit:
   - `sign-in-new-tab/01.html` against `welcome-set-up/07.html` reprinted, from which it was made;
   - `02.html` and `03.html` against the gallery's `welcome-model-away` with the landed stylesheets and
     its disclosure closed and open. The Forge region and the address are the world's: compare them
     against `welcome-model-forge-done`'s words, and say so.
2. **How,** at 1440, 1000 and 420 pixels wide, with each disclosure and menu forced open: each element's
   computed style and box, then the screenshots for what styles miss. A difference is fixed, under the
   trailer, or has its reason. Include the open field's width against the note's, and the disclosure's
   `margin` against the note above it, which the request's words name.
3. **States in words,** beside the message's words, rendered:
   - `ConnectModel`: `connect-model-signing-in` at the three widths, the note and disclosure as on the
     welcome, and its open paste;
   - a browser that refuses the tab, by the option above;
   - a sign-in that ends: `welcome-model-signin-ready`, `welcome-model-declined`,
     `welcome-model-refused`, `connect-model-signin-ready`, `connect-model-declined`,
     `connect-model-refused`, each with the note gone.
4. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: read the prints of the
   welcome's Away and `ConnectModel` (`skein_ui_text::render`, or the view's terminal tests): headings
   and order keep their meaning, no link, the paste folded under its name. Say so in the account.
5. **Design text:** `temper/docs/design/client/mockups/STATUS.md`, its first run's 4a and 4b: the
   secondary button and `step=model-elsewhere` are what `mockups/` still draws, and the client no
   longer does; the greenfield's `app.js` likewise says "in this tab", under its README's rule that
   look requests' changes stand on top of it, and stays. Journeys' `README.md` files and
   `temper/docs/design/client/journeys.md`, 9: read them; change only a line this makes false.
6. **Check the trailers and that the others stayed out:** `git log
   --format='%(trailers:key=Look-request,valueonly)' origin/main..HEAD | sort -u` names
   `sign-in-new-tab` alone on this draft's commits, and `git diff --stat origin/main...HEAD --
   temper/docs/design/client/journeys/overlay.css temper/docs/design/client/journeys/sign-in-new-tab`
   is empty. Then fmt, clippy on what you changed, its tests, the citations.
7. **The last commit** carries the render check's result and the table in its message, above its
   trailers; push.

At the least, these rows defer or differ, each with its home:

| Item | Result |
|---|---|
| The new tab until the address comes, "Opening ChatGPT's sign-in…" | deferred, to `temper/docs/design/draft/sign-in-new-tab.md`, 2 and fabrig/temper; `README.status.md`, first **Not yet** |
| The new tab at its end, "Sign-in received. You can close this tab." | deferred, the draft's 4; same entry |
| A browser that refuses the tab | drawn by `sign_in_wait::waiting`; the fact deferred, the draft's 2, `sign_in_wait::tab_refused` the one function it feeds |
| The sign-in refused at the press: the new tab closed | its words landed as today; the closing deferred, the draft's 2 |
| A reload while it waits | deferred, the draft's 3; a page come back draws the note with no link today |
| Moving on by itself, in the page's own tab with no `Return` | deferred, the draft's 3 and 5; today a press still leaves and comes back, so the page's words lead the behaviour until it lands |
| Journey walks meeting a new tab | fabrig/temper's, `temper/docs/design/client/journeys.md`, 3.1 and 4.2 |
| `Act::SignInElsewhere`, `ConnectModel::elsewhere` | stand with no entry in the view, the draft's 5 takes them; `README.status.md` |
| 02 and 03's Forge region and address | the world's: no journey reaches this page, whose values were written by hand |

## Report

The table, the render check's result, the screenshots' directory and the builder (`hostname`), and
what `notes/common.md`, Your report, asks.
