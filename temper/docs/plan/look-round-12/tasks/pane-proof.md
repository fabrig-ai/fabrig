+++
id = "pane-proof"
part = "pane-proof"
kind = "feature"
needs = ["ready-pane"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys.md",
]
effort = "high"
minutes = 90
+++

# The reprints, the render check and the account of `model-check-pane`

## Goal

Reprint the journeys that print Ready, prove the landing against the request by rendering, fix what the render finds,
bring the design text in line, and carry the account in the landing's last commit. Every commit carries `Look-request:
model-check-pane`.

Acceptance:

- **The journeys:** every journey printing Ready reprinted, and re-recorded if stale; their tests green. A re-record that
  stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the
  mark's wait in the recording driver and say so.
- **The render check,** `temper/docs/design/draft/look-requests.md`, 4, recorded: identical, or each difference with its
  reason.
- **The screenshots** in `~/src/fabrig/look-round-12/` on your builder, outside any worktree, never committed: each page's
  `-request.png` beside its `-landed.png` at 1440, 1280, 1000 and 420 wide, "Each step" open, and each state in words.
- **The account,** `temper/docs/design/draft/look-requests.md`, 5: in the last commit's message, one row per overlay rule,
  per page change and per state the message names, none left open.

## Inputs

- `temper/docs/plan/look-round-12/notes/common.md`.
- `temper/docs/design/draft/look-requests.md`, 3 (steps 5 to 8), 4, 5, 7.
- `temper/docs/design/client/journeys.md`, 8.
- `temper/docs/design/client/README.md`, 3.5, *The try*, *Ready*.
- `temper/docs/design/client/README.status.md`: its **Not yet** on *The try*, this landing's.
- The request, as the note says, and `ready-pane`'s commits on the trunk.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what the render finds and named in the account:

- `temper/crates/temper-client-view/src/pages/model_check.rs`, `pages/welcome.rs`, their tests, `look.rs`, `words.rs`.
- `temper/tools/gallery/src/**`, `temper/tools/journeys/src/**` (a mark's wait; its `print.rs` heading list, if Ready's
  print asks), and reprinted pages, regenerated.
- `temper/docs/design/client/README.md`, 3.5, `README.status.md`'s entries, `browser.md`, 2.5, and
  `temper/docs/design/client/mockups/STATUS.md`: what the check taught you. The journeys' `README.md` files: a line this
  makes false.

Options, each pre-approved:

- **A state no fact reaches:** flip `model_check::trial` locally, print the gallery state named below, screenshot it, and
  restore; the commit never carries the flip.
- **A state with no gallery name:** a throwaway one, untracked or reverted before the push.

Leave: what `notes/common.md` says is not built here, and `temper/docs/design/client/greenfield/`, whose `app.js` still
says "Checking Claude Code’s sign-in": its README keeps look requests' changes on top of it.

## Steps

1. **Reprint** by `temper/docs/design/client/journeys.md`, 8. `welcome-set-up` and `welcome-playground` print Ready;
   `project-landing` if its tests say so. `I've signed in` replacing at once may stale a recording that walks that way: a
   stale one names its turn and line.
2. **The pages:** from `ba044d7413`, with the overlay at that commit, each against its gallery state with the landed
   stylesheets:
   - `model-check-pane/01.html` against `welcome-ready-checking` under a flip, `State::Running { at: 1 }`;
   - `02.html`, the same, "Each step" open;
   - `03.html` against `welcome-ready-adopted` under a flip, `State::Answered { seconds: 6 }`;
   - `04.html` against `welcome-ready-not-yet`, no flip.

   The Forge line is the world's: no journey reaches these pages, whose values were written by hand. Compare it against
   the words of `welcome-ready-adopted`, and say so.
3. **How,** at 1440, 1280, 1000 and 420 pixels wide, with each disclosure forced open: each element's computed style and
   box, then the screenshots for what styles miss. A difference is fixed, under the trailer, or has its reason. Include: the
   pane at 1280 and just under, its hairline; the form's top margin with no checklist above it; `temper-flow-busy` in the
   pane; the signal's colours on 04; 04's acts at 420.
4. **States in words,** beside the message's words, rendered at the widths:
   - today's, no flip: `welcome-ready-checking`, `welcome-ready-not-yet`, `welcome-ready-none`, `welcome-ready-signing`,
     `welcome-ready-under-way`, `welcome-ready-adopted`, `welcome-model-connected`;
   - a plan's and a key's two steps, by flips with `First::Plan` and `First::Key`: running, answered, stopped;
   - a stop with the account usable, by a flip: "Ready" and its act enabled;
   - a reason with no act, `answer: None`: "Try again", "All ways".
5. **No flash:** walk the way on a fake box in a real browser (`SKEIN_TEST_CHROMIUM`): press "I’ve signed in", and sample
   every frame from the press until the check settles, usable and not: the way's page changes to Ready with no step
   between, and the heading says "Ready" from the first. Say what ran, or why it couldn't. (`journey 11` flickered: that is
   what this replaces.)
6. **The terminal,** `temper/docs/design/draft/look-requests.md`, 3, step 5: read Ready's prints of each state
   (`skein_ui_text::render`, or the view's terminal tests): the same lines as text, no bar, the headings and the order
   keeping their meaning. Say so in the account.
7. **Design text:** `temper/docs/design/client/journeys.md`, 9, the journeys' `README.md` files, and the client's
   `README.md`, section 1: read them; change only a line this makes false.
8. **Check the trailers and that the others stayed out:** `git log --format='%(trailers:key=Look-request,valueonly)'
   origin/main..HEAD | sort -u` names `model-check-pane` on this draft's commits, and `git diff --stat origin/main...HEAD
   -- temper/docs/design/client/journeys/overlay.css temper/docs/design/client/journeys/model-check-pane` is empty. Then
   fmt, clippy on what you changed, its tests, the citations.
9. **The last commit** carries the render check's result and the table in its message, above its trailers; push.

At the least, these rows defer or differ, each with its home:

| Item | Result |
|---|---|
| The trial: its task and charter, how it selects its model, what ends it, who pays | deferred, to `temper/docs/design/draft/model-check.md`, 2 and fabrig/temper; `draft/model-check.status.md` |
| Running at its later steps, 01 and 02's "Claude Code starts on this machine" | drawn by `model_check::line`; the fact deferred, the draft's 3, `model_check::trial` the one function it feeds; today's first step alone |
| Answered, "answered in N s", 03 | drawn; the fact deferred, the draft's 3; a usable account shows the ticked line without it |
| A stop past the first step, a stop with the account usable, a reason in the person's terms | drawn; the fact deferred, the draft's 4 (`RunWait`) |
| ChatGPT or a key | drawn; the fact deferred, the draft's 2 and 3 |
| A sign-in's end goes on to Ready | `sign-in-new-tab`'s `welcome_model_progress`, on its trunk; here a departure's return already lands on Ready |
| Leaving Ready doesn't stop the try | landed as the engine's: each host reports once declared, and the client stops watching |
| Home's "Claude didn't answer when tried", "Try again" | deferred, the draft's 4: no fact once the person has left; `README.status.md` |
| `ConnectModel`'s row holds the same lines | deferred, the draft's 2 and 3; `README.status.md` |
| "Try again" | differs: checks the host again until the draft's 2 |
| The list's accessible name and hidden state words | as Forge's list has them; `README.status.md` |
| The Forge line in 01 to 04 | the world's: no journey reaches these pages, whose values were written by hand |

## Report

The table, the render check's result, the screenshots' directory and the builder (`hostname`), and what
`notes/common.md`, Your report, asks.
